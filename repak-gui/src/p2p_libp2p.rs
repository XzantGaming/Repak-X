//! Internet-Wide P2P Network Module using libp2p
//! 
//! Provides NAT traversal, peer discovery, and relay support for
//! true internet-wide P2P file sharing without external servers.
//!
//! Features:
//! - Automatic NAT traversal (hole punching via DCUtR)
//! - Peer discovery via Kademlia DHT
//! - Relay support for peers behind strict NATs
//! - AutoNAT for detecting NAT type
//! - Encrypted connections via Noise protocol

use crate::ip_obfuscation;
use crate::p2p_protocol::{self, FileTransferCodec, FileTransferRequest, FileTransferResponse};
use libp2p::{
    autonat, dcutr, gossipsub, identify, kad,
    multiaddr::Protocol,
    noise, relay, request_response as req_resp, swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use log::{debug, info, warn};
use std::error::Error;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use base64::Engine;

// ============================================================================
// NETWORK BEHAVIOUR
// ============================================================================

/// Combined network behaviour for P2P file sharing
#[derive(NetworkBehaviour)]
pub struct P2PBehaviour {
    /// Peer discovery and routing
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
    /// Peer identification
    pub identify: identify::Behaviour,
    /// Relay client for NAT traversal
    pub relay_client: relay::client::Behaviour,
    /// Direct connection upgrade (hole punching)
    pub dcutr: dcutr::Behaviour,
    /// AutoNAT for detecting NAT type
    pub autonat: autonat::Behaviour,
    /// Gossipsub for broadcasting availability (optional)
    pub gossipsub: gossipsub::Behaviour,
    /// File transfer request/response protocol
    pub file_transfer: req_resp::Behaviour<FileTransferCodec>,
}

// ============================================================================
// P2P NETWORK MANAGER
// ============================================================================

/// Manages the libp2p network for P2P file sharing
pub struct P2PNetwork {
    swarm: Swarm<P2PBehaviour>,
    local_peer_id: PeerId,
    relay_addresses: Vec<Multiaddr>,
}

impl P2PNetwork {
    /// Create a new P2P network instance
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        // Generate a keypair for this peer
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        info!("Local peer ID: {}", local_peer_id);

        // Create Kademlia DHT for peer discovery
        let mut kad_config = kad::Config::default();
        kad_config.set_query_timeout(Duration::from_secs(60));
        let store = kad::store::MemoryStore::new(local_peer_id);
        let kad = kad::Behaviour::with_config(local_peer_id, store, kad_config);

        // Create identify protocol
        let identify = identify::Behaviour::new(identify::Config::new(
            "/repak-p2p/1.0.0".to_string(),
            local_key.public(),
        ));

        // Create relay client
        let (relay_transport, relay_client) = relay::client::new(local_peer_id);

        // Create DCUtR (Direct Connection Upgrade through Relay)
        let dcutr = dcutr::Behaviour::new(local_peer_id);

        // Create AutoNAT
        let autonat = autonat::Behaviour::new(
            local_peer_id,
            autonat::Config {
                retry_interval: Duration::from_secs(30),
                refresh_interval: Duration::from_secs(60),
                boot_delay: Duration::from_secs(5),
                throttle_server_period: Duration::from_secs(1),
                ..Default::default()
            },
        );

        // Create Gossipsub for broadcasting mod availability
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| format!("Gossipsub config error: {}", e))?;
        
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        // Create file transfer protocol
        let file_transfer = p2p_protocol::create_file_transfer_protocol();

        // Combine all behaviours
        let behaviour = P2PBehaviour {
            kad,
            identify,
            relay_client,
            dcutr,
            autonat,
            gossipsub,
            file_transfer,
        };

        // Build the swarm
        let swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_relay_client(noise::Config::new, yamux::Config::default)?
            .with_behaviour(|_, _| Ok(behaviour))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        Ok(Self {
            swarm,
            local_peer_id,
            relay_addresses: Self::default_relay_addresses(),
        })
    }

    /// Get default public relay addresses
    /// These are public relays that can be used for NAT traversal
    fn default_relay_addresses() -> Vec<Multiaddr> {
        vec![
            // Add public relay addresses here
            // For now, we'll bootstrap with known peers
            // In production, you'd want to maintain a list of reliable relays
        ]
    }

    /// Start listening on all available interfaces
    pub fn start_listening(&mut self) -> Result<Vec<Multiaddr>, Box<dyn Error>> {
        // Listen on all interfaces with random port
        let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
        self.swarm.listen_on(listen_addr)?;
        
        // Also listen on IPv6
        let listen_addr_v6: Multiaddr = "/ip6/::/tcp/0".parse()?;
        let _ = self.swarm.listen_on(listen_addr_v6); // Don't fail if IPv6 unavailable

        Ok(self.swarm.listeners().cloned().collect())
    }

    /// Connect to a relay server
    pub fn connect_to_relay(&mut self, relay_addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.dial(relay_addr.clone())?;
        self.relay_addresses.push(relay_addr);
        Ok(())
    }

    /// Bootstrap the DHT with known peers
    pub fn bootstrap(&mut self) -> Result<(), Box<dyn Error>> {
        self.swarm.behaviour_mut().kad.bootstrap()?;
        Ok(())
    }

    /// Advertise that we're sharing a mod pack
    pub fn advertise_share(&mut self, share_code: &str) -> Result<(), Box<dyn Error>> {
        // Put the share code in the DHT so others can find us
        let key = kad::RecordKey::new(&format!("repak-share:{}", share_code));
        let record = kad::Record {
            key: key.clone(),
            value: self.local_peer_id.to_bytes(),
            publisher: Some(self.local_peer_id),
            expires: None,
        };
        
        self.swarm.behaviour_mut().kad.put_record(record, kad::Quorum::One)?;
        info!("Advertised share code: {}", share_code);
        Ok(())
    }

    /// Find a peer by share code
    pub fn find_peer_by_share_code(&mut self, share_code: &str) {
        let key = kad::RecordKey::new(&format!("repak-share:{}", share_code));
        self.swarm.behaviour_mut().kad.get_record(key);
        info!("Searching for share code: {}", share_code);
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    /// Get listening addresses
    pub fn listening_addresses(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }

    /// Get external addresses (as detected by AutoNAT)
    pub fn external_addresses(&self) -> Vec<Multiaddr> {
        self.swarm.external_addresses().cloned().collect()
    }

    /// Request mod pack info from a peer
    pub fn request_pack_info(&mut self, peer: PeerId) -> req_resp::OutboundRequestId {
        self.swarm.behaviour_mut().file_transfer.send_request(&peer, FileTransferRequest::GetPackInfo)
    }

    /// Request a file chunk from a peer
    pub fn request_file_chunk(
        &mut self,
        peer: PeerId,
        filename: String,
        offset: u64,
        size: usize,
    ) -> req_resp::OutboundRequestId {
        self.swarm.behaviour_mut().file_transfer.send_request(
            &peer,
            FileTransferRequest::GetChunk { filename, offset, size },
        )
    }

    /// Send a response to a file transfer request
    pub fn send_response(
        &mut self,
        channel: req_resp::ResponseChannel<FileTransferResponse>,
        response: FileTransferResponse,
    ) -> Result<(), FileTransferResponse> {
        self.swarm.behaviour_mut().file_transfer.send_response(channel, response)
    }

    /// Process network events
    pub async fn next_event(&mut self) -> Option<P2PNetworkEvent> {
        use futures::StreamExt;
        loop {
            match self.swarm.next().await? {
                SwarmEvent::NewListenAddr { address, .. } => {
                    let obfuscated = ip_obfuscation::obfuscate_multiaddr(&address.to_string());
                    info!("Listening on: {}", obfuscated);
                    return Some(P2PNetworkEvent::ListeningOn(address));
                }
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    let addr = endpoint.get_remote_address().to_string();
                    let obfuscated = ip_obfuscation::obfuscate_multiaddr(&addr);
                    info!("Connected to peer: {} at {}", peer_id, obfuscated);
                    return Some(P2PNetworkEvent::PeerConnected(peer_id));
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    info!("Disconnected from peer: {} (cause: {:?})", peer_id, cause);
                    return Some(P2PNetworkEvent::PeerDisconnected(peer_id));
                }
                SwarmEvent::Behaviour(event) => {
                    if let Some(net_event) = self.handle_behaviour_event(event) {
                        return Some(net_event);
                    }
                }
                SwarmEvent::IncomingConnection { .. } => {
                    debug!("Incoming connection");
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    warn!("Outgoing connection error to {:?}: {}", peer_id, error);
                }
                SwarmEvent::IncomingConnectionError { error, .. } => {
                    warn!("Incoming connection error: {}", error);
                }
                _ => {}
            }
        }
    }

    /// Handle behaviour-specific events
    fn handle_behaviour_event(&mut self, event: P2PBehaviourEvent) -> Option<P2PNetworkEvent> {
        match event {
            P2PBehaviourEvent::Kad(kad_event) => {
                match kad_event {
                    kad::Event::OutboundQueryProgressed { result, .. } => {
                        match result {
                            kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(record))) => {
                                info!("Found record in DHT");
                                // Parse peer ID from record value
                                if let Ok(peer_id) = PeerId::from_bytes(&record.record.value) {
                                    return Some(P2PNetworkEvent::SharePeerFound(peer_id));
                                }
                            }
                            kad::QueryResult::Bootstrap(Ok(_)) => {
                                info!("DHT bootstrap successful");
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            P2PBehaviourEvent::Identify(identify_event) => {
                match identify_event {
                    identify::Event::Received { peer_id, info, .. } => {
                        debug!("Identified peer {}: {:?}", peer_id, info);
                        // Add peer to DHT routing table
                        for addr in info.listen_addrs {
                            self.swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                        }
                    }
                    _ => {}
                }
            }
            P2PBehaviourEvent::Dcutr(dcutr_event) => {
                // DCUtR (Direct Connection Upgrade through Relay) events
                debug!("DCUtR event: {:?}", dcutr_event);
            }
            P2PBehaviourEvent::Autonat(autonat_event) => {
                match autonat_event {
                    autonat::Event::StatusChanged { old, new } => {
                        info!("NAT status changed from {:?} to {:?}", old, new);
                        return Some(P2PNetworkEvent::NatStatusChanged(new));
                    }
                    _ => {}
                }
            }
            P2PBehaviourEvent::RelayClient(relay_event) => {
                match relay_event {
                    relay::client::Event::ReservationReqAccepted { relay_peer_id, .. } => {
                        info!("Relay reservation accepted by {}", relay_peer_id);
                        return Some(P2PNetworkEvent::RelayReservationSuccess(relay_peer_id));
                    }
                    _ => {}
                }
            }
            P2PBehaviourEvent::FileTransfer(ft_event) => {
                use req_resp::{Event, Message};
                match ft_event {
                    Event::Message { peer, message } => {
                        match message {
                            Message::Request { request, channel, .. } => {
                                info!("Received file transfer request from {}: {:?}", peer, request);
                                return Some(P2PNetworkEvent::FileTransferRequest { peer, request, channel });
                            }
                            Message::Response { response, .. } => {
                                info!("Received file transfer response from {}: {:?}", peer, response);
                                return Some(P2PNetworkEvent::FileTransferResponse { peer, response });
                            }
                        }
                    }
                    Event::OutboundFailure { peer, error, .. } => {
                        warn!("File transfer outbound failure to {}: {:?}", peer, error);
                    }
                    Event::InboundFailure { peer, error, .. } => {
                        warn!("File transfer inbound failure from {}: {:?}", peer, error);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        None
    }

    /// Dial a peer directly
    pub fn dial_peer(&mut self, peer_id: PeerId, addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        let mut addr_with_peer = addr.clone();
        addr_with_peer.push(Protocol::P2p(peer_id));
        self.swarm.dial(addr_with_peer)?;
        Ok(())
    }
}

// ============================================================================
// EVENTS
// ============================================================================

/// Events emitted by the P2P network
#[derive(Debug)]
pub enum P2PNetworkEvent {
    /// Started listening on an address
    ListeningOn(Multiaddr),
    /// Connected to a peer
    PeerConnected(PeerId),
    /// Disconnected from a peer
    PeerDisconnected(PeerId),
    /// Found a peer sharing the requested content
    SharePeerFound(PeerId),
    /// Hole punching succeeded
    HolePunchingSuccess(PeerId),
    /// NAT status changed
    NatStatusChanged(autonat::NatStatus),
    /// Relay reservation successful
    RelayReservationSuccess(PeerId),
    /// Received a file transfer request
    FileTransferRequest {
        peer: PeerId,
        request: FileTransferRequest,
        channel: req_resp::ResponseChannel<FileTransferResponse>,
    },
    /// Received a file transfer response
    FileTransferResponse {
        peer: PeerId,
        response: FileTransferResponse,
    },
}

// ============================================================================
// SHARE CODE FORMAT
// ============================================================================

/// Share information that can be encoded as a share code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareInfo {
    /// Peer ID of the sharer
    pub peer_id: String,
    /// Multiaddresses where the peer can be reached
    pub addresses: Vec<String>,
    /// Encryption key for the transfer
    pub encryption_key: String,
    /// Share code for DHT lookup
    pub share_code: String,
}

impl ShareInfo {
    /// Encode share info as a base64 string
    pub fn encode(&self) -> Result<String, Box<dyn Error>> {
        let json = serde_json::to_string(self)?;
        Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json))
    }

    /// Decode share info from a base64 string
    pub fn decode(encoded: &str) -> Result<Self, Box<dyn Error>> {
        let json = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded)?;
        let share_info = serde_json::from_slice(&json)?;
        Ok(share_info)
    }

    /// Get obfuscated addresses for display purposes
    pub fn obfuscated_addresses(&self) -> Vec<String> {
        ip_obfuscation::obfuscate_multiaddrs(&self.addresses)
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a multiaddress from a peer ID and relay
pub fn create_relayed_multiaddr(relay_addr: &Multiaddr, relay_peer_id: PeerId, target_peer_id: PeerId) -> Multiaddr {
    let mut addr = relay_addr.clone();
    addr.push(Protocol::P2p(relay_peer_id));
    addr.push(Protocol::P2pCircuit);
    addr.push(Protocol::P2p(target_peer_id));
    addr
}

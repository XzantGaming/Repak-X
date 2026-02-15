import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Share as ShareIcon,
  Download as DownloadIcon,
  Close as CloseIcon,
  ContentCopy as CopyIcon,
  CheckCircle as CheckIcon,
  CloudUpload as UploadIcon,
  CloudDownload as CloudDownloadIcon,
  Wifi as WifiIcon,
  WifiOff as WifiOffIcon,
  Security as SecurityIcon,
  Info as InfoIcon,
  Error as ErrorIcon,
  Cancel as CancelIcon,
  Search as SearchIcon
} from '@mui/icons-material';
import Checkbox from './ui/Checkbox';
import './SharingPanel.css';

import { useAlert } from './AlertHandler';

export default function SharingPanel({ onClose, gamePath, installedMods, selectedMods }) {
  const alert = useAlert();
  const [activeTab, setActiveTab] = useState('share'); // 'share' or 'receive'
  // const [error, setError] = useState(''); // Removed local error state in favor of toasts

  // Share State
  const [packName, setPackName] = useState('');
  const [packDesc, setPackDesc] = useState('');
  const [creatorName, setCreatorName] = useState('User');
  const [shareSession, setShareSession] = useState(null);
  const [isSharing, setIsSharing] = useState(false);
  const [selectedModPaths, setSelectedModPaths] = useState(new Set());
  const [packPreview, setPackPreview] = useState(null);
  const [calculatingPreview, setCalculatingPreview] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');

  // Receive State
  const [connectionString, setConnectionString] = useState('');
  const [clientName, setClientName] = useState('User');
  const [isReceiving, setIsReceiving] = useState(false);
  const [progress, setProgress] = useState(null);
  const [receiveComplete, setReceiveComplete] = useState(false);
  const [isValidCode, setIsValidCode] = useState(null); // null, true, false

  // Initialize selected mods from props
  useEffect(() => {
    if (selectedMods && selectedMods.size > 0) {
      setSelectedModPaths(new Set(selectedMods));
      setPackName(`My Mod Pack (${selectedMods.size} mods)`);
      setPackPreview(null); // Reset preview
    }
  }, [selectedMods]);

  // Poll for status
  useEffect(() => {
    let interval;
    checkStatus();
    interval = setInterval(checkStatus, 1000);
    return () => clearInterval(interval);
  }, []);

  // Validation helper
  const validateConnectionString = (str) => {
    try {
      const decoded = atob(str);
      const shareInfo = JSON.parse(decoded);
      return !!(shareInfo.peer_id && shareInfo.share_code && shareInfo.encryption_key);
    } catch (e) {
      return false;
    }
  };

  // Validation effect
  useEffect(() => {
    const validate = async () => {
      if (!connectionString.trim()) {
        setIsValidCode(null);
        return;
      }

      // Client-side validation (Base64 ShareInfo)
      if (!validateConnectionString(connectionString)) {
        setIsValidCode(false);
        return;
      }

      try {
        const valid = await invoke('p2p_validate_connection_string', { connectionString });
        setIsValidCode(valid);
      } catch (e) {
        setIsValidCode(false);
      }
    };
    const timeout = setTimeout(validate, 500);
    return () => clearTimeout(timeout);
  }, [connectionString]);

  const checkStatus = async () => {
    try {
      const sharing = await invoke('p2p_is_sharing');
      setIsSharing(sharing);

      if (sharing) {
        const session = await invoke('p2p_get_share_session');
        setShareSession(session);
      }

      const receiving = await invoke('p2p_is_receiving');
      setIsReceiving(receiving);

      if (receiving) {
        const prog = await invoke('p2p_get_receive_progress');
        setProgress(prog);
        if (prog && prog.status && prog.status.hasOwnProperty('Completed')) {
          setReceiveComplete(true);
          setIsReceiving(false);
          alert.success('Download Complete', 'All mods have been received and installed.');
        }
      }
    } catch (err) {
      console.error("Status check failed:", err);
    }
  };

  // Helper to get the connection string from session
  const getConnectionString = () => {
    if (!shareSession) return '';
    // If it has connection_string (old ShareSession), use it
    if (shareSession.connection_string) return shareSession.connection_string;
    // If it's ShareInfo (new libp2p), encode it
    if (shareSession.peer_id && shareSession.share_code) {
      try {
        return btoa(JSON.stringify(shareSession));
      } catch (e) {
        console.error("Failed to encode session", e);
        return '';
      }
    }
    return '';
  };

  const handleCalculatePreview = async () => {
    if (selectedModPaths.size === 0) return;
    setCalculatingPreview(true);
    try {
      const preview = await invoke('p2p_create_mod_pack_preview', {
        name: packName || "Untitled",
        description: packDesc || "",
        modPaths: Array.from(selectedModPaths),
        creator: creatorName
      });
      setPackPreview(preview);
    } catch (err) {
      console.error("Preview failed", err);
      alert.error("Calculation Failed", String(err));
    } finally {
      setCalculatingPreview(false);
    }
  };

  const handleStartSharing = async () => {
    if (selectedModPaths.size === 0) {
      alert.error("Select Mods", "Please select at least one mod to share.");
      return;
    }
    if (!packName.trim()) {
      alert.error("Missing Name", "Please enter a pack name.");
      return;
    }

    const toastId = alert.showAlert({
      title: 'Starting Session',
      description: 'Initializing P2P network...',
      color: 'default',
      isLoading: true,
      duration: 0 // Persistent while loading
    });

    try {
      const session = await invoke('p2p_start_sharing', {
        name: packName,
        description: packDesc,
        modPaths: Array.from(selectedModPaths),
        creator: creatorName
      });
      setShareSession(session);
      setIsSharing(true);


      // Update toast to Info (Primary) state instead of Success
      alert.updateToast(toastId, {
        title: 'Sharing Active',
        description: 'Your mod pack is now online.',
        color: 'primary',
        isLoading: false,
        duration: 5000
      });
    } catch (err) {
      alert.updateToast(toastId, {
        title: 'Share Failed',
        description: String(err),
        color: 'danger',
        isLoading: false,
        duration: 5000
      });
    }
  };

  const handleStopSharing = async () => {
    try {
      if (shareSession && shareSession.share_code) {
        await invoke('p2p_stop_sharing', { shareCode: shareSession.share_code });
      } else {
        // Fallback if session is lost but UI thinks we are sharing
        await invoke('p2p_stop_sharing', { shareCode: "" });
      }

      setShareSession(null);
      setIsSharing(false);

      alert.info('Sharing Stopped', 'The sharing session has been terminated.');
    } catch (err) {
      alert.error('Stop Failed', `Failed to stop sharing: ${err}`);
    }
  };

  const handleStartReceiving = async () => {
    if (!connectionString.trim()) {
      alert.error("Missing Code", "Please enter a connection string.");
      return;
    }

    if (!validateConnectionString(connectionString)) {
      alert.error("Invalid Code", "The connection string format is incorrect.");
      return;
    }

    alert.promise(
      async () => {
        // Validate first
        await invoke('p2p_validate_connection_string', { connectionString });

        // Start receiving
        await invoke('p2p_start_receiving', {
          connectionString,
          clientName: clientName
        });

        setIsReceiving(true);
        setReceiveComplete(false);

      },
      {
        loading: { title: 'Connecting', description: 'Establishing connection to host...' },
        success: { title: 'Connected', description: 'Download starting...' },
        error: (err) => ({ title: 'Connection Failed', description: String(err) })
      }
    );
  };

  const handleStopReceiving = async () => {
    try {
      await invoke('p2p_stop_receiving');
      setIsReceiving(false);

      alert.info('Download Cancelled', 'The download was cancelled by user.');
    } catch (err) {
      alert.error('Cancel Failed', `Failed to stop download: ${err}`);
    }
  };

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text);
    alert.info('Copied!', 'Share code copied to clipboard.');
  };

  const toggleModSelection = (path) => {
    const newSet = new Set(selectedModPaths);
    if (newSet.has(path)) {
      newSet.delete(path);
    } else {
      newSet.add(path);
    }
    setSelectedModPaths(newSet);
    setPackPreview(null); // Invalidate preview
  };

  return (
    <div className="p2p-overlay">
      <motion.div
        className="p2p-modal"
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.95 }}
        transition={{ duration: 0.15 }}
      >
        <div className="p2p-header">
          <div className="p2p-title">
            <WifiIcon className="p2p-icon" />
            <h2>Mod Sharing</h2>
          </div>
          <button onClick={onClose} className="btn-icon-close">
            <CloseIcon />
          </button>
        </div>

        <div className="p2p-tabs">
          <button
            className={`p2p-tab ${activeTab === 'share' ? 'active' : ''}`}
            onClick={() => setActiveTab('share')}
          >
            <UploadIcon fontSize="small" /> Share Mods
          </button>
          <button
            className={`p2p-tab ${activeTab === 'receive' ? 'active' : ''}`}
            onClick={() => setActiveTab('receive')}
          >
            <CloudDownloadIcon fontSize="small" /> Receive Mods
          </button>
        </div>

        <div className="p2p-content">


          {activeTab === 'share' && (
            <div className="share-view">
              {!isSharing ? (
                <>
                  <div className="share-layout-grid">
                    <div className="share-left-col">
                      <div className="mod-selection-list">
                        <div className="mod-list-header">
                          <label>Select Mods to Share ({selectedModPaths.size})</label>
                          <div className="search-box">
                            <SearchIcon fontSize="small" className="search-icon" />
                            <input
                              type="text"
                              value={searchTerm}
                              onChange={(e) => setSearchTerm(e.target.value)}
                              placeholder="Search mods..."
                            />
                          </div>
                        </div>
                        <div className="mod-list-scroll">
                          {installedMods.filter(mod => {
                            const filename = mod.path.split('\\').pop();
                            const name = mod.custom_name || filename.replace(/_9999999_P/g, '').replace(/\.pak$/i, '');
                            return name.toLowerCase().includes(searchTerm.toLowerCase());
                          }).map(mod => {
                            const filename = mod.path.split('\\').pop();
                            const displayName = mod.custom_name || filename.replace(/_9999999_P/g, '').replace(/\.pak$/i, '');
                            return (
                              <div
                                key={mod.path}
                                className={`mod-select-item ${selectedModPaths.has(mod.path) ? 'selected' : ''}`}
                                onClick={() => toggleModSelection(mod.path)}
                              >
                                <Checkbox
                                  checked={selectedModPaths.has(mod.path)}
                                  size="sm"
                                />
                                <span className="mod-name">
                                  {displayName}
                                </span>
                              </div>
                            )
                          })}
                        </div>
                      </div>
                    </div>

                    <div className="share-right-col">
                      <div className="form-group">
                        <label>Pack Name</label>
                        <input
                          type="text"
                          value={packName}
                          onChange={(e) => setPackName(e.target.value)}
                          placeholder="e.g. My Repak X modpack"
                          className="p2p-input"
                        />
                      </div>
                      <div className="form-group">
                        <label>Description (Optional)</label>
                        <textarea
                          value={packDesc}
                          onChange={(e) => setPackDesc(e.target.value)}
                          placeholder="Describe what's in this pack..."
                          className="p2p-textarea"
                        />
                      </div>
                      <div className="form-group">
                        <label>Creator Name (Optional)</label>
                        <input
                          type="text"
                          value={creatorName}
                          onChange={(e) => setCreatorName(e.target.value)}
                          placeholder="Your Name"
                          className="p2p-input"
                        />
                      </div>

                      {selectedModPaths.size > 0 && (
                        <div className="pack-preview-section">
                          {!packPreview ? (
                            <button
                              onClick={handleCalculatePreview}
                              className="btn-secondary btn-small"
                              disabled={calculatingPreview}
                            >
                              {calculatingPreview ? "Calculating..." : "Calculate Pack Size"}
                            </button>
                          ) : (
                            <div className="preview-info">
                              <span>Total Size: {(packPreview.total_size / 1024 / 1024).toFixed(2)} MB</span>
                              <span>Files: {packPreview.file_count}</span>
                            </div>
                          )}
                        </div>
                      )}

                      <button onClick={handleStartSharing} className="btn-primary btn-large">
                        <ShareIcon /> Start Sharing
                      </button>
                    </div>
                  </div>
                </>
              ) : (
                <div className="active-share-view">
                  <div className="success-banner">
                    <CheckIcon /> Sharing Active
                  </div>

                  <div className="share-code-display">
                    <label>SHARE CODE</label>
                    <div className="code-box">
                      {getConnectionString()}
                      <button
                        onClick={() => copyToClipboard(getConnectionString())}
                        className="btn-copy"
                        title="Copy to clipboard"
                      >
                        <CopyIcon />
                      </button>
                    </div>
                    <p className="hint">Share this code with your friend to let them download your pack.</p>
                  </div>

                  <div className="session-info">
                    <div className="info-row">
                      <span>Pack Name:</span>
                      <strong>{packName}</strong>
                    </div>
                    <div className="info-row">
                      <span>Creator:</span>
                      <strong>{creatorName}</strong>
                    </div>
                    <div className="info-row">
                      <span>Mods:</span>
                      <strong>{selectedModPaths.size} files</strong>
                    </div>
                    <div className="info-row">
                      <span>Security:</span>
                      <span className="secure-badge"><SecurityIcon fontSize="inherit" /> AES-256 Encrypted</span>
                    </div>
                  </div>

                  <button onClick={handleStopSharing} className="btn-danger btn-large">
                    <WifiOffIcon /> Stop Sharing
                  </button>
                </div>
              )}
            </div>
          )}

          {activeTab === 'receive' && (
            <div className="receive-view">
              {!isReceiving && !receiveComplete ? (
                <>
                  <div className="form-group">
                    <label>Enter Share Code</label>
                    <div className="input-with-validation">
                      <input
                        type="text"
                        value={connectionString}
                        onChange={(e) => setConnectionString(e.target.value)}
                        placeholder="Paste the connection string here..."
                        className={`p2p-input code-input ${isValidCode === true ? 'valid' : isValidCode === false ? 'invalid' : ''}`}
                      />
                      {isValidCode === true && <CheckIcon className="validation-icon valid" />}
                      {isValidCode === false && <CancelIcon className="validation-icon invalid" />}
                    </div>
                  </div>

                  <div className="form-group">
                    <label>Your Name (Optional)</label>
                    <input
                      type="text"
                      value={clientName}
                      onChange={(e) => setClientName(e.target.value)}
                      placeholder="Enter your name"
                      className="p2p-input"
                    />
                  </div>

                  <div className="security-note">
                    <SecurityIcon fontSize="small" />
                    <p>Only connect to people you trust. All transfers are encrypted.</p>
                  </div>

                  <button
                    onClick={handleStartReceiving}
                    className="btn-primary btn-large"
                    disabled={isValidCode !== true}
                  >
                    <DownloadIcon /> Connect & Download
                  </button>
                </>
              ) : (
                <div className="transfer-progress-view">
                  {receiveComplete ? (
                    <div className="completion-state">
                      <CheckIcon className="success-icon-large" />
                      <h3>Download Complete!</h3>
                      <p>All mods have been installed successfully.</p>
                      <button
                        onClick={() => {
                          setReceiveComplete(false);
                          setConnectionString('');
                          setProgress(null);
                          setIsValidCode(null);
                        }}
                        className="btn-secondary"
                      >
                        Download Another
                      </button>
                    </div>
                  ) : (
                    <>
                      <h3>{progress?.status === 'Connecting' ? 'Connecting via relay...' : 'Downloading...'}</h3>
                      {progress && (
                        <div className="progress-container">
                          <div className="progress-info">
                            <span>{progress.current_file}</span>
                            <span>{Math.round((progress.files_completed / progress.total_files) * 100)}%</span>
                          </div>
                          <div className="progress-bar-track">
                            <div
                              className="progress-bar-fill"
                              style={{ width: `${(progress.files_completed / progress.total_files) * 100}%` }}
                            />
                          </div>
                          <div className="progress-stats">
                            <span>{progress.files_completed} / {progress.total_files} files</span>
                            <span>{(progress.bytes_transferred / 1024 / 1024).toFixed(1)} MB transferred</span>
                          </div>
                          <div className="status-badge">
                            {typeof progress.status === 'string' ? progress.status : JSON.stringify(progress.status)}
                          </div>
                        </div>
                      )}
                      <button onClick={handleStopReceiving} className="btn-danger">
                        Cancel Download
                      </button>
                    </>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      </motion.div>
    </div>
  );
};



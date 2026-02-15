use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use log::info;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const DISCORD_APP_ID: &str = "1465997185341718559";

/// Maps color theme names to Discord asset keys
/// These asset keys must be uploaded to the Discord Developer Portal
/// under Rich Presence -> Art Assets with matching names
///
/// Available color presets in the app:
/// - Red (crimson)
/// - Blue (bright blue)
/// - Cyan/Teal (turquoise)
/// - Green (bright green)
/// - Orange
/// - Pink (hot pink/magenta)
fn get_logo_asset_for_theme(theme: &str) -> &'static str {
    match theme.to_lowercase().as_str() {
        "red" | "crimson" => "repakx_logo_red",
        "blue" | "default" => "repakx_logo_blue",
        "green" => "repakx_logo_green",
        "orange" => "repakx_logo_orange",
        "purple" | "violet" => "repakx_logo_purple",
        "pink" | "magenta" | "hotpink" => "repakx_logo_pink",
        _ => "repakx_logo", // Fallback to default
    }
}

pub struct DiscordPresenceManager {
    client: Mutex<Option<DiscordIpcClient>>,
    enabled: Mutex<bool>,
    start_timestamp: i64,
    current_theme: Mutex<String>,
}

impl DiscordPresenceManager {
    pub fn new() -> Self {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Self {
            client: Mutex::new(None),
            enabled: Mutex::new(false),
            start_timestamp,
            current_theme: Mutex::new("default".to_string()),
        }
    }

    /// Set the color theme for the Discord logo
    /// This will be applied on the next activity update
    pub fn set_theme(&self, theme: &str) {
        *self.current_theme.lock() = theme.to_string();
        info!("Discord theme set to: {}", theme);
    }

    /// Get the current theme
    pub fn get_theme(&self) -> String {
        self.current_theme.lock().clone()
    }

    pub fn connect(&self) -> Result<(), String> {
        let mut client_guard = self.client.lock();

        if client_guard.is_some() {
            return Ok(()); // Already connected
        }

        info!("Connecting to Discord...");

        let mut client = DiscordIpcClient::new(DISCORD_APP_ID);

        client
            .connect()
            .map_err(|e| format!("Failed to connect to Discord: {}", e))?;

        info!("Connected to Discord Rich Presence");
        *client_guard = Some(client);
        *self.enabled.lock() = true;

        Ok(())
    }

    pub fn disconnect(&self) -> Result<(), String> {
        let mut client_guard = self.client.lock();

        if let Some(mut client) = client_guard.take() {
            info!("Disconnecting from Discord...");
            let _ = client.clear_activity();
            let _ = client.close();
        }

        *self.enabled.lock() = false;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.client.lock().is_some() && *self.enabled.lock()
    }

    pub fn set_activity(&self, state: &str, details: Option<&str>) -> Result<(), String> {
        let mut client_guard = self.client.lock();

        let client = client_guard.as_mut().ok_or("Discord not connected")?;

        // Get the logo asset based on current theme
        let theme = self.current_theme.lock().clone();
        let logo_asset = get_logo_asset_for_theme(&theme);

        let mut activity_builder = activity::Activity::new()
            .state(state)
            .timestamps(activity::Timestamps::new().start(self.start_timestamp))
            .assets(
                activity::Assets::new()
                    .large_image(logo_asset)
                    .large_text("Repak X - Marvel Rivals Mod Manager"),
            );

        if let Some(details_text) = details {
            activity_builder = activity_builder.details(details_text);
        }

        client
            .set_activity(activity_builder)
            .map_err(|e| format!("Failed to set Discord activity: {}", e))?;

        Ok(())
    }

    /// Set the theme and immediately refresh the activity to show the new logo
    pub fn set_theme_and_refresh(
        &self,
        theme: &str,
        current_state: &str,
        current_details: Option<&str>,
    ) -> Result<(), String> {
        self.set_theme(theme);
        if self.is_connected() {
            self.set_activity(current_state, current_details)?;
        }
        Ok(())
    }

    pub fn set_idle(&self) -> Result<(), String> {
        self.set_activity("Idle", Some("Managing mods"))
    }

    pub fn set_managing_mods(&self, mod_count: usize) -> Result<(), String> {
        let state = format!(
            "Managing {} mod{}",
            mod_count,
            if mod_count == 1 { "" } else { "s" }
        );
        self.set_activity(&state, Some("Repak X"))
    }

    pub fn set_installing_mod(&self, mod_name: &str) -> Result<(), String> {
        self.set_activity("Installing mod", Some(mod_name))
    }

    pub fn set_sharing_mods(&self) -> Result<(), String> {
        self.set_activity("Sharing mods via P2P", Some("Repak X"))
    }

    pub fn set_receiving_mods(&self) -> Result<(), String> {
        self.set_activity("Receiving mods via P2P", Some("Repak X"))
    }

    pub fn clear_activity(&self) -> Result<(), String> {
        let mut client_guard = self.client.lock();

        if let Some(client) = client_guard.as_mut() {
            let _ = client.clear_activity();
        }

        Ok(())
    }
}

impl Default for DiscordPresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedDiscordPresence = Arc<DiscordPresenceManager>;

pub fn create_discord_manager() -> SharedDiscordPresence {
    Arc::new(DiscordPresenceManager::new())
}

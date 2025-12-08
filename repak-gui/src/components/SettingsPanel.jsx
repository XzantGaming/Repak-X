import React, { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { AnimatedThemeToggler } from './ui/AnimatedThemeToggler'
import './SettingsPanel.css'

const ACCENT_COLORS = {
  blue: '#4a9eff',
  purple: '#9c27b0',
  green: '#4CAF50',
  orange: '#ff9800',
  pink: '#e91e63'
};

export default function SettingsPanel({ settings, onSave, onClose, theme, setTheme, accentColor, setAccentColor, gamePath, onAutoDetectGamePath, onBrowseGamePath, isGamePathLoading }) {
  const [globalUsmap, setGlobalUsmap] = useState(settings.globalUsmap || '');
  const [hideSuffix, setHideSuffix] = useState(settings.hideSuffix || false);
  const [usmapStatus, setUsmapStatus] = useState('');
  const [isUpdatingChars, setIsUpdatingChars] = useState(false);
  const [charUpdateStatus, setCharUpdateStatus] = useState('');
  const [isSkippingLauncher, setIsSkippingLauncher] = useState(false);
  const [skipLauncherStatus, setSkipLauncherStatus] = useState('');
  const [isLauncherPatchEnabled, setIsLauncherPatchEnabled] = useState(false);

  const handleSave = () => {
    onSave({
      globalUsmap,
      hideSuffix
    });
    onClose();
  };

  const handleUpdateCharacterData = async () => {
    setIsUpdatingChars(true);
    setCharUpdateStatus('Updating...');
    try {
      const count = await invoke('update_character_data_from_github');
      setCharUpdateStatus(`✓ Successfully updated! ${count} new skins added.`);
    } catch (error) {
      setCharUpdateStatus(`Error: ${error}`);
    } finally {
      setIsUpdatingChars(false);
    }
  };

  const handleCancelUpdate = async () => {
    try {
      await invoke('cancel_character_update');
      setCharUpdateStatus('Cancelling...');
    } catch (error) {
      console.error('Failed to cancel:', error);
    }
  };

  const handleSkipLauncherPatch = async () => {
    setIsSkippingLauncher(true);
    setSkipLauncherStatus('');
    try {
      // TODO: Replace with actual backend call once developed
      await invoke('skip_launcher_patch');
      setIsLauncherPatchEnabled(true);
      setSkipLauncherStatus('✓ Launcher patch applied successfully!');
    } catch (error) {
      setSkipLauncherStatus(`Error: ${error}`);
    } finally {
      setIsSkippingLauncher(false);
    }
  };

  const handleBrowseUsmap = async () => {
    try {
      const selected = await open({
        filters: [{
          name: 'USmap Files',
          extensions: ['usmap']
        }],
        title: 'Select USmap File'
      });

      if (selected) {
        // Call backend to copy file to Usmap/ folder
        const filename = await invoke('copy_usmap_to_folder', { sourcePath: selected });
        setGlobalUsmap(filename);
        setUsmapStatus(`✓ USmap file copied to Usmap folder: ${filename}`);
      }
    } catch (error) {
      console.error('Failed to select USmap:', error);
      setUsmapStatus(`✗ Error: ${error}`);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content settings-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Settings</h2>
          <button className="modal-close" onClick={onClose}>×</button>
        </div>
        
        <div className="modal-body">
          <div className="setting-section">
            <h3>Game Mods Path</h3>
            <div className="setting-group">
              <label>Your game's mods folder path.</label>
              <input
                type="text"
                value={gamePath || ''}
                readOnly
                placeholder="No game path set"
                className="path-input"
              />
            </div>
            <div className="setting-group" style={{ display: 'flex', gap: '0.5rem' }}>
              <button onClick={onAutoDetectGamePath} disabled={isGamePathLoading}>
                {isGamePathLoading ? 'Detecting…' : 'Auto Detect'}
              </button>
              <button onClick={onBrowseGamePath}>
                Browse
              </button>
            </div>
          </div>

          <div className="setting-section">
            <h3>Game Settings</h3>
            <div className="setting-group">
              <label>
                <input
                  type="checkbox"
                  checked={hideSuffix}
                  onChange={(e) => setHideSuffix(e.target.checked)}
                />
                Hide file suffix in mod names
              </label>
            </div>
            <div className="setting-group">
              <label>Global USMAP Path:</label>
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <input
                  type="text"
                  value={globalUsmap}
                  onChange={(e) => setGlobalUsmap(e.target.value)}
                  placeholder="Path to global USMAP file..."
                  className="path-input"
                  readOnly
                />
                <button onClick={handleBrowseUsmap}>
                  Browse
                </button>
              </div>
              {usmapStatus && (
                <p style={{ 
                  fontSize: '0.85rem', 
                  marginTop: '0.5rem',
                  color: usmapStatus.startsWith('✓') ? '#4CAF50' : '#ff5252'
                }}>
                  {usmapStatus}
                </p>
              )}
            </div>
          </div>

          <div className="setting-section">
            <h3>Character Data</h3>
            <div className="setting-group">
              <p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>
                Update the character database from GitHub to support new heroes and skins.
              </p>
              <div style={{ display: 'flex', gap: '0.5rem' }}>
                <button onClick={handleUpdateCharacterData} disabled={isUpdatingChars}>
                  {isUpdatingChars ? 'Updating...' : 'Update from GitHub'}
                </button>
                {isUpdatingChars && (
                  <button onClick={handleCancelUpdate} className="btn-secondary" style={{ borderColor: '#ff5252', color: '#ff5252' }}>
                    Cancel
                  </button>
                )}
              </div>
              {charUpdateStatus && (
                <p style={{ 
                  fontSize: '0.85rem', 
                  marginTop: '0.5rem',
                  color: charUpdateStatus.includes('Error') || charUpdateStatus.includes('Cancelled') ? '#ff5252' : '#4CAF50'
                }}>
                  {charUpdateStatus}
                </p>
              )}
            </div>
          </div>

          <div className="setting-section">
            <h3>Skip Launcher Patch</h3>
            <div className="setting-group">
              <p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>
                Sets <b>launch_record</b> value to 0.
              </p>
              <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                <button onClick={handleSkipLauncherPatch} disabled={isSkippingLauncher}>
                  {isSkippingLauncher ? 'Applying...' : 'Skip Launcher Patch'}
                </button>
                <span style={{ 
                  display: 'inline-flex', 
                  alignItems: 'center', 
                  gap: '0.4rem',
                  fontSize: '0.85rem',
                  fontWeight: 600,
                  color: isLauncherPatchEnabled ? '#4CAF50' : '#ff5252'
                }}>
                  <span style={{
                    width: '8px',
                    height: '8px',
                    borderRadius: '50%',
                    backgroundColor: isLauncherPatchEnabled ? '#4CAF50' : '#ff5252'
                  }}></span>
                  {isLauncherPatchEnabled ? 'Enabled' : 'Disabled'}
                </span>
              </div>
              {skipLauncherStatus && (
                <p style={{ 
                  fontSize: '0.85rem', 
                  marginTop: '0.5rem',
                  color: skipLauncherStatus.includes('Error') ? '#ff5252' : '#4CAF50'
                }}>
                  {skipLauncherStatus}
                </p>
              )}
            </div>
          </div>

          <div className="setting-section">
            <h3>Theme</h3>
            <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
              <AnimatedThemeToggler theme={theme} setTheme={setTheme} />
              <span style={{ fontSize: '0.9rem', opacity: 0.8 }}>
                {theme === 'dark' ? 'Dark Mode' : 'Light Mode'}
              </span>
            </div>
          </div>

          <div className="setting-section">
            <h3>Accent Color</h3>
            <div className="color-options">
              {Object.entries(ACCENT_COLORS).map(([name, color]) => (
                <button
                  key={name}
                  className={`color-option ${accentColor === color ? 'selected' : ''}`}
                  style={{ backgroundColor: color }}
                  onClick={() => setAccentColor(color)}
                  title={name.charAt(0).toUpperCase() + name.slice(1)}
                />
              ))}
            </div>
          </div>
        </div>
        
        <div className="modal-footer">
          <button onClick={handleSave} className="btn-primary">
            Save
          </button>
          <button onClick={onClose} className="btn-secondary">
            Cancel
          </button>
        </div>
      </div>
    </div>
  )
}

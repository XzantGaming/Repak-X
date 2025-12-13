import React, { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { AnimatedThemeToggler } from './ui/AnimatedThemeToggler'
import Checkbox from './ui/Checkbox'
import { LuFolderInput } from "react-icons/lu"
import { RiSparkling2Fill } from "react-icons/ri"
import { IoMdRefresh, IoIosSkipForward } from "react-icons/io"
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
  const [autoOpenDetails, setAutoOpenDetails] = useState(settings.autoOpenDetails || false);
  const [showHeroIcons, setShowHeroIcons] = useState(settings.showHeroIcons || false);
  const [usmapStatus, setUsmapStatus] = useState('');
  const [isUpdatingChars, setIsUpdatingChars] = useState(false);
  const [charUpdateStatus, setCharUpdateStatus] = useState('');
  const [isSkippingLauncher, setIsSkippingLauncher] = useState(false);
  const [skipLauncherStatus, setSkipLauncherStatus] = useState('');
  const [isLauncherPatchEnabled, setIsLauncherPatchEnabled] = useState(false);

  // Check skip launcher status on mount
  React.useEffect(() => {
    const checkStatus = async () => {
      try {
        const isEnabled = await invoke('get_skip_launcher_status');
        setIsLauncherPatchEnabled(isEnabled);
      } catch (error) {
        console.error('Failed to check skip launcher status:', error);
      }
    };
    checkStatus();
  }, []);

  // Clear skip launcher status after 5 seconds
  React.useEffect(() => {
    if (skipLauncherStatus) {
      const timer = setTimeout(() => {
        setSkipLauncherStatus('');
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [skipLauncherStatus]);

  // Clear usmap status after 5 seconds
  React.useEffect(() => {
    if (usmapStatus) {
      const timer = setTimeout(() => {
        setUsmapStatus('');
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [usmapStatus]);

  const handleSave = () => {
    onSave({
      globalUsmap,
      hideSuffix,
      autoOpenDetails,
      showHeroIcons
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
      // Toggle the skip launcher patch
      const isEnabled = await invoke('skip_launcher_patch');
      setIsLauncherPatchEnabled(isEnabled);
      setSkipLauncherStatus(
        isEnabled
          ? '✓ Skip launcher enabled (launch_record = 0)'
          : '✓ Skip launcher disabled (launch_record = 6)'
      );
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
              <p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>Your game's mods folder path.</p>
              <div className="combined-input-group">
                <input
                  type="text"
                  value={gamePath || ''}
                  readOnly
                  placeholder="No game path set"
                  className="integrated-input"
                />
                <div className="input-actions">
                  <button
                    onClick={onAutoDetectGamePath}
                    disabled={isGamePathLoading}
                    className="action-btn"
                    title="Auto Detect Game Path"
                  >
                    <RiSparkling2Fill />
                    {isGamePathLoading ? 'Detecting…' : 'Auto Detect'}
                  </button>
                  <button
                    onClick={onBrowseGamePath}
                    className="action-btn icon-only"
                    title="Browse Folder"
                  >
                    <LuFolderInput size={16} />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div className="setting-section">
            <h3>USMAP Mapping File</h3>
            <div className="setting-group">
              <p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>Global .usmap file path for asset mapping.</p>
              <div className="combined-input-group">
                <input
                  type="text"
                  value={globalUsmap}
                  onChange={(e) => setGlobalUsmap(e.target.value)}
                  placeholder="Path to global USMAP file..."
                  className="integrated-input"
                  readOnly
                />
                <div className="input-actions">
                  <button
                    onClick={handleBrowseUsmap}
                    className="action-btn icon-only"
                    title="Select USmap File"
                  >
                    <LuFolderInput size={16} />
                  </button>
                </div>
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
            <h3>Skip Launcher Patch</h3>
            <div className="setting-group">
              <p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>
                Sets <b>launch_record</b> value to 0.
              </p>
              <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                <button
                  onClick={handleSkipLauncherPatch}
                  disabled={isSkippingLauncher}
                  style={{ display: 'flex', alignItems: 'center', gap: '6px' }}
                >
                  <IoIosSkipForward size={16} />
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
            <h3>Mods View Settings</h3>
            <div className="setting-group">
              <Checkbox
                checked={hideSuffix}
                onChange={(checked) => setHideSuffix(checked)}
              >
                <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Hide file suffix in mod names</span>
              </Checkbox>
              <div style={{ marginTop: '0.5rem' }}>
                <Checkbox
                  checked={autoOpenDetails}
                  onChange={(checked) => setAutoOpenDetails(checked)}
                >
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Auto-open details panel on click</span>
                </Checkbox>
              </div>
              <div style={{ marginTop: '0.5rem' }}>
                <Checkbox
                  checked={showHeroIcons}
                  onChange={(checked) => setShowHeroIcons(checked)}
                >
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Show hero icon and background on mod cards (Experimental)</span>
                </Checkbox>
              </div>
            </div>
          </div>

          <div className="setting-section">
            <h3>Character Database</h3>
            <div className="setting-group">
              <p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>
                Update the character database from GitHub to support new heroes and skins.
              </p>
              <div style={{ display: 'flex', gap: '0.5rem' }}>
                <button
                  onClick={handleUpdateCharacterData}
                  disabled={isUpdatingChars}
                  style={{ display: 'flex', alignItems: 'center', gap: '6px' }}
                >
                  <IoMdRefresh size={18} className={isUpdatingChars ? 'spin-animation' : ''} />
                  {isUpdatingChars ? 'Updating...' : 'Update Heroes Database'}
                </button>
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
            <h3>Theme</h3>
            <div className="setting-group">
              <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '1rem' }}>
                <AnimatedThemeToggler theme={theme} setTheme={setTheme} />
                <span style={{ fontSize: '0.9rem', opacity: 0.8 }}>
                  {theme === 'dark' ? 'Dark Mode' : 'Light Mode'}
                </span>
              </div>

              <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.9rem', opacity: 0.9 }}>Accent Color</label>
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
        </div>

        <div className="modal-footer" style={{ gap: '0.5rem' }}>
          <button
            onClick={onClose}
            className="btn-secondary"
            style={{ padding: '0.4rem 1rem', fontSize: '0.9rem', minWidth: 'auto' }}
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            className="btn-primary"
            style={{ padding: '0.4rem 1rem', fontSize: '0.9rem', minWidth: 'auto' }}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  )
}

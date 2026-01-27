import React, { useState, useEffect } from 'react';
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { AnimatedThemeToggler } from './ui/AnimatedThemeToggler'
import Checkbox from './ui/Checkbox'
import { LuFolderInput } from "react-icons/lu"
import { RiSparkling2Fill } from "react-icons/ri"
import './SettingsPanel.css'
import { useAlert } from './AlertHandler'

const ACCENT_COLORS = {
  repakRed: '#be1c1c',
  blue: '#4a9eff',
  purple: '#9c27b0',
  green: '#4CAF50',
  orange: '#ff9800',
  pink: '#FF96BC'
};



export default function SettingsPanel({ settings, onSave, onClose, theme, setTheme, accentColor, setAccentColor, gamePath, onAutoDetectGamePath, onBrowseGamePath, isGamePathLoading }) {
  const alert = useAlert();
  const [globalUsmap, setGlobalUsmap] = useState(settings.globalUsmap || '');
  const [hideSuffix, setHideSuffix] = useState(settings.hideSuffix || false);
  const [autoOpenDetails, setAutoOpenDetails] = useState(settings.autoOpenDetails || false);
  const [showHeroIcons, setShowHeroIcons] = useState(settings.showHeroIcons || false);
  const [showHeroBg, setShowHeroBg] = useState(settings.showHeroBg || false);
  const [showModType, setShowModType] = useState(settings.showModType || false);
  const [showExperimental, setShowExperimental] = useState(settings.showExperimental || false);
  const [usmapStatus, setUsmapStatus] = useState('');
  const [showRatMode, setShowRatMode] = useState(false);

  // Easter egg: briefly show "Rat Mode" when switching to light theme
  const handleThemeToggle = (newTheme) => {
    if (newTheme === 'light') {
      setShowRatMode(true);
      setTimeout(() => setShowRatMode(false), 300);
    }
    setTheme(newTheme);
  };

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
      showHeroIcons,
      showHeroBg,
      showModType,
      showExperimental
    });
    alert.success('Settings Saved', 'Your preferences have been updated.');
    onClose();
  };

  // Sync local state with props when opening/changing
  useEffect(() => {
    if (settings.globalUsmap) {
      setGlobalUsmap(settings.globalUsmap);
    }
  }, [settings.globalUsmap]);

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
            <h3>Mods View Settings</h3>
            <div className="setting-group">
              <Checkbox
                checked={hideSuffix}
                onChange={(checked) => setHideSuffix(checked)}
              >
                <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Hide file suffix in mod names</span>
              </Checkbox>
              <div>
                <Checkbox
                  checked={autoOpenDetails}
                  onChange={(checked) => setAutoOpenDetails(checked)}
                >
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Auto-open details panel on click</span>
                </Checkbox>
              </div>
              <div>
                <Checkbox
                  checked={showHeroIcons}
                  onChange={(checked) => setShowHeroIcons(checked)}
                >
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Show hero icons on mod cards (experimental)</span>
                </Checkbox>
              </div>
              <div>
                <Checkbox
                  checked={showHeroBg}
                  onChange={(checked) => setShowHeroBg(checked)}
                >
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Show hero background on mod cards (experimental)</span>
                </Checkbox>
              </div>
              <div>
                <Checkbox
                  checked={showModType}
                  onChange={(checked) => setShowModType(checked)}
                >
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Show mod type badge on cards</span>
                </Checkbox>
              </div>
              <div>
                <Checkbox
                  checked={showExperimental}
                  onChange={(checked) => setShowExperimental(checked)}
                >
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Enables "Compact List" view (experimental)</span>
                </Checkbox>
              </div>
            </div>
          </div>

          <div className="setting-section">
            <h3>Theme</h3>
            <div className="setting-group">
              <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '1rem' }}>
                <AnimatedThemeToggler theme={theme} setTheme={handleThemeToggle} />
                <span style={{ fontSize: '0.9rem', opacity: 0.8 }}>
                  {theme === 'dark' ? 'Dark Mode' : (showRatMode ? 'Rat Mode 🐀' : 'Light Mode')}
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

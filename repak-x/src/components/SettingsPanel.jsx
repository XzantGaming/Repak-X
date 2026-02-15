import React, { useState, useEffect } from 'react';
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { AnimatedThemeToggler } from './ui/AnimatedThemeToggler'
import Switch from './ui/Switch'
import Checkbox from './ui/Checkbox'
import { LuFolderInput } from "react-icons/lu"
import { RiSparkling2Fill } from "react-icons/ri"
import { CgPerformance } from "react-icons/cg"
import { MdRefresh } from "react-icons/md"
import { FaDiscord } from "react-icons/fa"
import { RiGraduationCapFill } from "react-icons/ri"
import { BsKeyboardFill } from "react-icons/bs"
import DiscordWidget from './DiscordWidget'
import './SettingsPanel.css'
import { useAlert } from './AlertHandler'
import { motion } from 'framer-motion'

const ACCENT_COLORS = {
  repakRed: '#be1c1c',
  blue: '#4a9eff',
  purple: '#9c27b0',
  green: '#4CAF50',
  orange: '#ff9800',
  pink: '#FF96BC'
};


export default function SettingsPanel({ settings, onSave, onClose, theme, setTheme, accentColor, setAccentColor, gamePath, onAutoDetectGamePath, onBrowseGamePath, isGamePathLoading, onCheckForUpdates, isCheckingUpdates, onReplayTour, onOpenShortcuts }) {
  const alert = useAlert();
  const [globalUsmap, setGlobalUsmap] = useState(settings.globalUsmap || '');
  const [hideSuffix, setHideSuffix] = useState(settings.hideSuffix || false);
  const [autoOpenDetails, setAutoOpenDetails] = useState(settings.autoOpenDetails || false);
  const [showHeroIcons, setShowHeroIcons] = useState(settings.showHeroIcons || false);
  const [showHeroBg, setShowHeroBg] = useState(settings.showHeroBg || false);
  const [showModType, setShowModType] = useState(settings.showModType || false);
  const [showExperimental, setShowExperimental] = useState(settings.showExperimental || false);
  const [autoCheckUpdates, setAutoCheckUpdates] = useState(settings.autoCheckUpdates || false);
  const [parallelProcessing, setLocalParallelProcessing] = useState(settings.parallelProcessing || false);
  const [enableDrp, setEnableDrp] = useState(settings.enableDrp !== false); // Default true if undefined, or check requirements? Actually default false usually safer, but user eager. Let's stick to explicit default or false. Code says "settings.enableDrp || false" usually.
  // Wait, usually I do settings.enableDrp || false.
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
      showExperimental,
      autoCheckUpdates,
      parallelProcessing,
      enableDrp
    });
    alert.success('Settings Saved', 'Your preferences have been updated.');
    onClose();
  };

  // Sync local state with props when opening/changing
  useEffect(() => {
    if (settings.globalUsmap) {
      setGlobalUsmap(settings.globalUsmap);
    }
    if (settings.enableDrp !== undefined) {
      setEnableDrp(settings.enableDrp);
    }
  }, [settings.globalUsmap, settings.enableDrp]);

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
      <motion.div
        className="modal-content settings-modal"
        onClick={(e) => e.stopPropagation()}
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.15 }}
      >
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
            <h3>Updates</h3>
            <div className="setting-group">
              <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '1rem' }}>
                <button
                  onClick={onCheckForUpdates}
                  disabled={isCheckingUpdates}
                  className="action-btn"
                  title="Check for updates now"
                  style={{ minWidth: '140px' }}
                >
                  <MdRefresh className={isCheckingUpdates ? 'spin-icon' : ''} />
                  {isCheckingUpdates ? 'Checking...' : 'Check Now'}
                </button>
                <span style={{ fontSize: '0.8rem', opacity: 0.6 }}>Current Version: v{typeof __APP_VERSION__ !== 'undefined' ? __APP_VERSION__ : '0.0.0'}</span>
              </div>

              <Checkbox
                checked={autoCheckUpdates}
                onChange={(checked) => setAutoCheckUpdates(checked)}
              >
                <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Auto-check for updates on startup</span>
              </Checkbox>
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
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Show hero icons on mod cards</span>
                </Checkbox>
              </div>
              <div>
                <Checkbox
                  checked={showHeroBg}
                  onChange={(checked) => setShowHeroBg(checked)}
                >
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Show hero background on mod cards</span>
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
                  <span style={{ paddingLeft: '4px', fontWeight: 'normal', opacity: 0.9 }}>Enables "Compact List" view</span>
                </Checkbox>
              </div>
            </div>
          </div>

          <div className="setting-section">
            <h3>Experimental</h3>
            <div className="setting-group">
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', alignItems: 'center' }}>
                  <CgPerformance style={{ marginRight: '8px', color: accentColor }} />
                  <span style={{ fontWeight: 'normal', opacity: 0.9 }}>Parallel Processing Mode</span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <span style={{
                    fontSize: '0.85rem',
                    opacity: parallelProcessing ? 1 : 0.8,
                    fontWeight: parallelProcessing ? '900' : '500',
                    fontStyle: parallelProcessing ? 'italic' : 'normal',
                    color: parallelProcessing ? accentColor : 'inherit',
                    textShadow: parallelProcessing ? '2px 2px 0px rgba(0,0,0,0.2)' : 'none',
                    transition: 'all 0.2s ease'
                  }}>
                    {parallelProcessing ? 'BOOST' : 'Normal'}
                  </span>
                  <Switch style={{ marginTop: '0.5rem' }}
                    checked={parallelProcessing}
                    onChange={(checked) => setLocalParallelProcessing(checked)}
                  />
                </div>
              </div>
              <p style={{ fontSize: '0.8rem', opacity: 0.6, marginLeft: '24px', marginTop: '-0.8rem' }}>
                {parallelProcessing
                  ? 'Boost mode uses 75% of available threads for backend operations.'
                  : 'Normal mode uses 50% of available threads for backend operations.'}
              </p>
            </div>
          </div>

          <div className="setting-section">
            <h3>Integrations</h3>
            <div className="setting-group">
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', alignItems: 'center' }}>
                  <FaDiscord style={{ color: '#5865F2', marginRight: '8px' }} />
                  <span style={{ fontWeight: 'normal', opacity: 0.9 }}>Enable Discord Rich Presence</span>
                </div>
                <Switch style={{ marginTop: '0.5rem' }}
                  checked={enableDrp}
                  onChange={(checked) => setEnableDrp(checked)}
                />
              </div>
              <p style={{ fontSize: '0.8rem', opacity: 0.6, marginLeft: '24px', marginTop: '-0.8rem' }}>
                Show your active modding status on Discord.
              </p>
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

          <div className="setting-section">
            <h3>Help</h3>
            <div className="setting-group">
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <span style={{ fontWeight: 'normal', opacity: 0.9 }}>Replay the app tour to learn about key features</span>
                <button
                  onClick={onReplayTour}
                  className="action-btn"
                  title="Replay the onboarding tour"
                  style={{ minWidth: '120px' }}
                >
                  <RiGraduationCapFill style={{ color: accentColor }} /> Replay Tour
                </button>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: '1rem' }}>
                <span style={{ fontSize: '1rem', opacity: 0.9 }}>
                  Press <strong style={{ opacity: 1 }}>F1</strong> anytime to view all available keyboard shortcuts
                </span>
                <button
                  onClick={onOpenShortcuts}
                  className="action-btn"
                  title="View keyboard shortcuts"
                  style={{ minWidth: '120px' }}
                >
                  <BsKeyboardFill style={{ color: accentColor }} /> Shortcuts
                </button>
              </div>
            </div>
          </div>

          <div className="setting-section">
            <h3>Community</h3>
            <div className="setting-group">
              <p style={{ fontSize: '0.95rem', fontWeight: 600, opacity: 0.9, marginBottom: '0.15rem' }}>
                Repak X is built for the community.
              </p>
              <p style={{ fontSize: '0.85rem', opacity: 0.55, marginBottom: '0.5rem', lineHeight: 1.5 }}>
                If you need help, want to report a bug, or have a feature request, join the Discord server and help make Repak X better for everyone.
              </p>
              <DiscordWidget />
            </div>
          </div>

        </div>

        <div className="modal-footer">
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
      </motion.div>
    </div>
  )
}

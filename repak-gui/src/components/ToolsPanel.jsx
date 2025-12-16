import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { IoMdRefresh, IoIosSkipForward } from "react-icons/io";
import { FaFileZipper } from "react-icons/fa6";
import Switch from './ui/Switch';
import './SettingsPanel.css'; // Reuse the same styles

export default function ToolsPanel({ onClose }) {
    const [isUpdatingChars, setIsUpdatingChars] = useState(false);
    const [charUpdateStatus, setCharUpdateStatus] = useState('');
    const [isSkippingLauncher, setIsSkippingLauncher] = useState(false);
    const [skipLauncherStatus, setSkipLauncherStatus] = useState('');
    const [isLauncherPatchEnabled, setIsLauncherPatchEnabled] = useState(false);

    // Check skip launcher status on mount
    useEffect(() => {
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
    useEffect(() => {
        if (skipLauncherStatus) {
            const timer = setTimeout(() => {
                setSkipLauncherStatus('');
            }, 5000);
            return () => clearTimeout(timer);
        }
    }, [skipLauncherStatus]);

    // Clear char update status after 5 seconds
    useEffect(() => {
        if (charUpdateStatus) {
            const timer = setTimeout(() => {
                setCharUpdateStatus('');
            }, 5000);
            return () => clearTimeout(timer);
        }
    }, [charUpdateStatus]);

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

    const handleReCompress = () => {
        // Placeholder - to be implemented
        console.log('ReCompress clicked - placeholder');
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-content settings-modal" onClick={(e) => e.stopPropagation()}>
                <div className="modal-header">
                    <h2>Tools</h2>
                    <button className="modal-close" onClick={onClose}>×</button>
                </div>

                <div className="modal-body">
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
                        <h3>ReCompress</h3>
                        <div className="setting-group">
                            <p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>
                                Apply Oodle compression to IOStore bundles paked with an outdated mod manager. <img src="https://i.imgur.com/N5cCamW.png" alt="" style={{ width: '24px', height: '24px', verticalAlign: 'middle' }} />
                            </p>
                            <div style={{ display: 'flex', gap: '0.5rem' }}>
                                <button
                                    onClick={handleReCompress}
                                    style={{ display: 'flex', alignItems: 'center', gap: '6px' }}
                                >
                                    <FaFileZipper size={16} />
                                    ReCompress
                                </button>
                            </div>
                        </div>
                    </div>

                    <div className="setting-section" style={{ opacity: 0.5, pointerEvents: 'none' }}>
                        <h3>Character LODs Thanos</h3>
                        <div className="setting-group">
                            <p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>
                                Coming soon...
                            </p>
                            <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                                <Switch
                                    isOn={false}
                                    onToggle={() => { }}
                                    disabled={true}
                                />
                                <span style={{ fontSize: '0.9rem' }}>Enable LOD Thanos</span>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="modal-footer" style={{ gap: '0.5rem' }}>
                    <button
                        onClick={onClose}
                        className="btn-primary"
                        style={{ padding: '0.4rem 1rem', fontSize: '0.9rem', minWidth: 'auto' }}
                    >
                        Close
                    </button>
                </div>
            </div>
        </div>
    );
}


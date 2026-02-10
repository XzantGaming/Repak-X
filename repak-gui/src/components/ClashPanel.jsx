import React from 'react'
import { motion } from 'framer-motion'
import { IoMdWarning } from "react-icons/io"
import {
    Close,
    WarningAmberRounded,
    InsertDriveFileOutlined,
    CheckCircleOutline
} from '@mui/icons-material'
import NumberInput from './ui/NumberInput'
import './ClashPanel.css'

const ClashPanel = ({ clashes, mods = [], onSetPriority, onClose }) => {
    return (
        <div className="modal-overlay clash-overlay" onClick={onClose}>
            <motion.div
                className="clash-panel-content"
                onClick={e => e.stopPropagation()}
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.15 }}
            >
                <div className="clash-header">
                    <h2>
                        <IoMdWarning style={{ color: 'var(--danger)' }} />
                        Mod Conflicts
                        {clashes.length > 0 && (
                            <span style={{
                                fontSize: '0.85rem',
                                background: 'rgba(244, 67, 54, 0.1)',
                                color: 'var(--danger)',
                                padding: '2px 8px',
                                borderRadius: '12px',
                                border: '1px solid rgba(244, 67, 54, 0.2)'
                            }}>
                                {clashes.length}
                            </span>
                        )}
                    </h2>
                    <button className="close-icon-btn" onClick={onClose}>
                        <Close fontSize="small" />
                    </button>
                </div>

                <div className="clash-body">
                    {clashes.length === 0 ? (
                        <div className="no-clashes">
                            <CheckCircleOutline className="no-clashes-icon" />
                            <p>No conflicts found! Your mods are clean.</p>
                        </div>
                    ) : (
                        <div className="clash-list">
                            {clashes.map((clash, i) => (
                                <div key={i} className="clash-card">
                                    <div className="clash-file-path">
                                        <InsertDriveFileOutlined fontSize="small" className="clash-file-icon" />
                                        {clash.file_path.replace(/^\/?Game\//, '')}
                                    </div>

                                    <div className="clash-mods-list">
                                        {clash.mod_paths.map(path => {
                                            const mod = mods.find(m => m.path === path)
                                            return (
                                                <div key={path} className="clash-mod-row">
                                                    <span className="clash-mod-badge">Conflicting</span>
                                                    <span className="clash-mod-name" title={path}>
                                                        {path.split(/[/\\]/).pop().replace(/\.pak$/i, '')}
                                                    </span>

                                                    {mod && (
                                                        <div className="clash-priority-wrapper">
                                                            <NumberInput
                                                                value={mod.priority || 0}
                                                                min={0}
                                                                max={99}
                                                                onChange={(val) => onSetPriority && onSetPriority(path, val)}
                                                            />
                                                        </div>
                                                    )}
                                                </div>
                                            )
                                        })}
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>

                <div className="clash-footer">
                    <button className="btn-primary" onClick={onClose}>Close</button>
                </div>
            </motion.div>
        </div>
    )
}

export default ClashPanel

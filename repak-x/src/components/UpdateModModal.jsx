import React, { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { FaExchangeAlt } from 'react-icons/fa'
import { IoWarningOutline } from 'react-icons/io5'
import './UpdateModModal.css'

export default function UpdateModModal({ isOpen, onClose, onConfirm, oldMod, newSourcePath }) {
    const [preserveName, setPreserveName] = useState(true)

    if (!isOpen || !oldMod) return null

    // Extract filenames for display
    const oldName = oldMod.custom_name || oldMod.path.split(/[/\\]/).pop()
    const newName = newSourcePath ? newSourcePath.split(/[/\\]/).pop() : 'New Mod'

    // Extract clean name for the old mod (without extension)
    const oldCleanName = oldName.replace(/\.[^.]+$/, '')
    // Extract extension from new mod to show what full name would look like
    const newExt = newName.match(/\.[^.]+$/)?.[0] || '.pak'

    return (
        <AnimatePresence>
            {isOpen && (
                <div className="modal-overlay" onClick={onClose}>
                    <motion.div
                        className="modal-content update-mod-modal"
                        onClick={e => e.stopPropagation()}
                        initial={{ opacity: 0, scale: 0.95, y: 20 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.95, y: 20 }}
                        transition={{ duration: 0.2 }}
                    >
                        <div className="modal-header">
                            <h2><FaExchangeAlt /> Update Mod</h2>
                        </div>

                        <div className="modal-body">
                            <div className="update-info-grid">
                                <div className="mod-info-box old">
                                    <div className="info-label">CURRENT</div>
                                    <div className="info-name" title={oldMod.path}>{oldName}</div>
                                </div>

                                <div className="arrow-divider">
                                    <motion.div
                                        animate={{ x: [0, 5, 0] }}
                                        transition={{ repeat: Infinity, duration: 2 }}
                                    >
                                        ➜
                                    </motion.div>
                                </div>

                                <div className="mod-info-box new">
                                    <div className="info-label">REPLACE WITH</div>
                                    <div className="info-name" title={newSourcePath}>{newName}</div>
                                </div>
                            </div>

                            <div className="options-section">
                                <label className="checkbox-option">
                                    <input
                                        type="checkbox"
                                        checked={preserveName}
                                        onChange={e => setPreserveName(e.target.checked)}
                                    />
                                    <div className="option-text">
                                        <span className="option-title">Keep existing filename</span>
                                        <span className="option-desc">
                                            The new file will be renamed to <code>{oldCleanName}{newExt}</code> to preserve your load order/overrides.
                                        </span>
                                    </div>
                                </label>

                                {!preserveName && (
                                    <div className="name-warning">
                                        <IoWarningOutline />
                                        <span>Using the new filename (<code>{newName}</code>) might change load order priority.</span>
                                    </div>
                                )}
                            </div>
                        </div>

                        <div className="modal-footer">
                            <button className="cancel-btn" onClick={onClose}>Cancel</button>
                            <button
                                className="confirm-btn primary"
                                onClick={() => onConfirm(preserveName)}
                                autoFocus
                            >
                                Update Mod
                            </button>
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    )
}

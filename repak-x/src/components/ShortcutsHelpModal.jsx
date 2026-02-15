import { motion, AnimatePresence } from 'framer-motion'
import './ShortcutsHelpModal.css'

const shortcuts = [
    { keys: ['Ctrl', 'F'], action: 'Focus search' },
    { keys: ['Ctrl', 'R'], action: 'Full app reload' },
    { keys: ['Ctrl', 'Shift', 'R'], action: 'Refresh mod list' },
    { keys: ['Ctrl', ','], action: 'Open Settings' },
    { keys: ['Esc'], action: 'Close panel / Deselect' },
    { keys: ['Ctrl', 'E'], action: 'Toggle mod enabled' },
    { keys: ['F2'], action: 'Rename mod' },
    { keys: ['↑', '↓', '←', '→'], action: 'Navigate mods list' },
    { keys: ['Enter'], action: 'Open mod details' },
    { keys: ['Shift', 'Click'], action: 'Select range of mods' },
    { keys: ['Shift', 'Ctrl', 'Click'], action: 'Deselect range of mods' },
    { keys: ['F1'], action: 'Show this help' },
]

function ShortcutsHelpModal({ isOpen, onClose }) {
    if (!isOpen) return null

    return (
        <div className="modal-overlay" onClick={onClose}>
            <motion.div
                className="modal-content shortcuts-modal"
                onClick={(e) => e.stopPropagation()}
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.95 }}
                transition={{ duration: 0.15 }}
            >
                <div className="modal-header">
                    <h2>⌨️ Keyboard Shortcuts</h2>
                    <button className="modal-close" onClick={onClose}>×</button>
                </div>
                <div className="shortcuts-list">
                    {shortcuts.map((shortcut, index) => (
                        <div key={index} className="shortcut-row">
                            <div className="shortcut-keys">
                                {shortcut.keys.map((key, i) => (
                                    <span key={i}>
                                        <kbd>{key}</kbd>
                                        {i < shortcut.keys.length - 1 && <span className="key-separator">+</span>}
                                    </span>
                                ))}
                            </div>
                            <div className="shortcut-action">{shortcut.action}</div>
                        </div>
                    ))}
                </div>
                <div className="shortcuts-footer">
                    Press <kbd>Esc</kbd> to close
                </div>
            </motion.div>
        </div>
    )
}

export default ShortcutsHelpModal

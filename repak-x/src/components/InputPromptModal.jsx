import React, { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { VscClose, VscNewFolder } from 'react-icons/vsc';
import './InputPromptModal.css';

/**
 * A styled input prompt modal to replace browser's prompt()
 * 
 * @param {Object} props
 * @param {boolean} props.isOpen - Whether the modal is visible
 * @param {string} props.title - Modal title
 * @param {string} props.placeholder - Input placeholder text
 * @param {string} props.confirmText - Confirm button text
 * @param {string} props.cancelText - Cancel button text (default: "Cancel")
 * @param {React.ReactNode} props.icon - Icon to display in header
 * @param {function} props.onConfirm - Called with input value when confirmed
 * @param {function} props.onCancel - Called when cancelled
 * @param {string} props.initialValue - Initial input value
 * @param {string} props.accentColor - Optional accent color (default uses CSS variable)
 */
const InputPromptModal = ({
    isOpen,
    title = 'Enter value',
    placeholder = 'Enter value...',
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    icon = <VscNewFolder />,
    onConfirm,
    onCancel,
    initialValue = '',
    accentColor
}) => {
    const [value, setValue] = useState(initialValue);
    const inputRef = useRef(null);

    // Reset value and focus input when modal opens
    useEffect(() => {
        if (isOpen) {
            setValue(initialValue);
            // Small delay to ensure modal is rendered before focusing
            setTimeout(() => {
                inputRef.current?.focus();
                inputRef.current?.select();
            }, 100);
        }
    }, [isOpen, initialValue]);

    const handleSubmit = (e) => {
        e.preventDefault();
        if (value.trim()) {
            onConfirm(value.trim());
        }
    };

    const handleKeyDown = (e) => {
        if (e.key === 'Escape') {
            onCancel();
        }
    };

    const handleBackdropClick = (e) => {
        if (e.target === e.currentTarget) {
            onCancel();
        }
    };

    const style = accentColor ? { '--prompt-accent': accentColor } : {};

    return (
        <AnimatePresence>
            {isOpen && (
                <motion.div
                    className="input-prompt-overlay"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.15 }}
                    onClick={handleBackdropClick}
                    onKeyDown={handleKeyDown}
                    style={style}
                >
                    <motion.div
                        className="input-prompt-modal"
                        initial={{ y: -20, opacity: 0, scale: 0.95 }}
                        animate={{ y: 0, opacity: 1, scale: 1 }}
                        exit={{ y: -20, opacity: 0, scale: 0.95 }}
                        transition={{ duration: 0.2, ease: 'easeOut' }}
                    >
                        {/* Header */}
                        <div className="prompt-header">
                            <div className="prompt-icon">
                                {icon}
                            </div>
                            <h3>{title}</h3>
                            <button className="prompt-close" onClick={onCancel}>
                                <VscClose />
                            </button>
                        </div>

                        {/* Content */}
                        <form onSubmit={handleSubmit} className="prompt-content">
                            <input
                                ref={inputRef}
                                type="text"
                                value={value}
                                onChange={(e) => setValue(e.target.value)}
                                placeholder={placeholder}
                                className="prompt-input"
                                autoFocus
                            />

                            {/* Actions */}
                            <div className="prompt-actions">
                                <button
                                    type="button"
                                    className="prompt-btn prompt-btn-cancel"
                                    onClick={onCancel}
                                >
                                    {cancelText}
                                </button>
                                <button
                                    type="submit"
                                    className="prompt-btn prompt-btn-confirm"
                                    disabled={!value.trim()}
                                >
                                    {confirmText}
                                </button>
                            </div>
                        </form>
                    </motion.div>
                </motion.div>
            )}
        </AnimatePresence>
    );
};

export default InputPromptModal;

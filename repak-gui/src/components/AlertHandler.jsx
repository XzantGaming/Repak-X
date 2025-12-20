import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { MdClearAll } from 'react-icons/md';
import './ui/Alert.css';

// Toast Icons
const Icons = {
    success: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
            <polyline points="22 4 12 14.01 9 11.01"></polyline>
        </svg>
    ),
    danger: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="15" y1="9" x2="9" y2="15"></line>
            <line x1="9" y1="9" x2="15" y2="15"></line>
        </svg>
    ),
    warning: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
            <line x1="12" y1="9" x2="12" y2="13"></line>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
        </svg>
    ),
    info: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="16" x2="12" y2="12"></line>
            <line x1="12" y1="8" x2="12.01" y2="8"></line>
        </svg>
    ),
    default: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="16" x2="12" y2="12"></line>
            <line x1="12" y1="8" x2="12.01" y2="8"></line>
        </svg>
    )
};

// Create the Alert Context
const AlertContext = createContext(null);

// Custom hook to use alerts from any component
export const useAlert = () => {
    const context = useContext(AlertContext);
    if (!context) {
        throw new Error('useAlert must be used within an AlertProvider');
    }
    return context;
};

// Default configuration
const DEFAULT_CONFIG = {
    placement: 'bottom-center', // bottom-center as requested
    maxVisible: 5,
    defaultDuration: 5000
};

// Alert Provider Component
export function AlertProvider({ children, placement = DEFAULT_CONFIG.placement }) {
    const [toasts, setToasts] = useState([]);

    // Add a new toast
    const showAlert = useCallback((alertConfig) => {
        const id = Date.now() + Math.random();
        const toast = {
            id,
            color: 'default',
            variant: 'flat',
            ...alertConfig,
            duration: alertConfig.duration ?? DEFAULT_CONFIG.defaultDuration,
            createdAt: Date.now()
        };

        setToasts(prev => [...prev, toast]);
        return id;
    }, []);

    // Dismiss a toast by ID
    const dismissAlert = useCallback((id) => {
        setToasts(prev => prev.filter(toast => toast.id !== id));
    }, []);

    // Dismiss all toasts
    const dismissAllAlerts = useCallback(() => {
        setToasts([]);
    }, []);

    // Update an existing toast
    const updateToast = useCallback((id, updates) => {
        setToasts(prev => prev.map(toast =>
            toast.id === id ? { ...toast, ...updates } : toast
        ));
    }, []);

    // Convenience methods for common toast types
    const success = useCallback((title, description, options = {}) => {
        return showAlert({ color: 'success', title, description, ...options });
    }, [showAlert]);

    const error = useCallback((title, description, options = {}) => {
        return showAlert({ color: 'danger', title, description, ...options });
    }, [showAlert]);

    const warning = useCallback((title, description, options = {}) => {
        return showAlert({ color: 'warning', title, description, ...options });
    }, [showAlert]);

    const info = useCallback((title, description, options = {}) => {
        return showAlert({ color: 'primary', title, description, ...options });
    }, [showAlert]);

    // Promise toast - shows loading state while promise is pending
    const promise = useCallback((promiseOrFn, options = {}) => {
        const {
            loading = { title: 'Loading...', description: 'Please wait' },
            success: successConfig = { title: 'Success', description: 'Operation completed' },
            error: errorConfig = { title: 'Error', description: 'Something went wrong' },
            ...restOptions
        } = options;

        // Create the loading toast
        const id = showAlert({
            ...loading,
            color: 'default',
            isLoading: true,
            duration: 0, // Don't auto-dismiss while loading
            ...restOptions
        });

        // Execute the promise
        const thePromise = typeof promiseOrFn === 'function' ? promiseOrFn() : promiseOrFn;

        thePromise
            .then((result) => {
                // Update to success state
                const successOptions = typeof successConfig === 'function'
                    ? successConfig(result)
                    : successConfig;
                updateToast(id, {
                    ...successOptions,
                    color: 'success',
                    isLoading: false,
                    duration: DEFAULT_CONFIG.defaultDuration
                });
            })
            .catch((err) => {
                // Update to error state
                const errorOptions = typeof errorConfig === 'function'
                    ? errorConfig(err)
                    : errorConfig;
                updateToast(id, {
                    ...errorOptions,
                    color: 'danger',
                    isLoading: false,
                    duration: DEFAULT_CONFIG.defaultDuration
                });
            });

        return id;
    }, [showAlert, updateToast]);

    const contextValue = {
        showAlert,
        dismissAlert,
        dismissAllAlerts,
        updateToast,
        success,
        error,
        warning,
        info,
        promise
    };

    return (
        <AlertContext.Provider value={contextValue}>
            {children}
            <ToastContainer
                toasts={toasts}
                onDismiss={dismissAlert}
                onDismissAll={dismissAllAlerts}
                placement={placement}
            />
        </AlertContext.Provider>
    );
}

// Get animation variants based on placement
function getAnimationVariants(placement) {
    const isBottom = placement.startsWith('bottom');
    const isCenter = placement.includes('center');
    const isLeft = placement.includes('left');

    let initial = { opacity: 0, scale: 0.9 };
    let exit = { opacity: 0, scale: 0.9 };

    if (isCenter) {
        initial = { opacity: 0, y: isBottom ? 50 : -50, scale: 0.9 };
        exit = { opacity: 0, y: isBottom ? 20 : -20, scale: 0.9 };
    } else if (isLeft) {
        initial = { opacity: 0, x: -100, scale: 0.9 };
        exit = { opacity: 0, x: -50, scale: 0.9 };
    } else {
        initial = { opacity: 0, x: 100, scale: 0.9 };
        exit = { opacity: 0, x: 50, scale: 0.9 };
    }

    return {
        initial,
        animate: { opacity: 1, x: 0, y: 0, scale: 1 },
        exit
    };
}

// Toast Container - renders all active toasts with card stacking
function ToastContainer({ toasts, onDismiss, onDismissAll, placement }) {
    const [isHovered, setIsHovered] = useState(false);
    const maxVisible = 3; // Show max 3 stacked cards

    // Get the most recent toasts (reverse so newest is first)
    const recentToasts = [...toasts].reverse().slice(0, maxVisible);

    const isBottom = placement.startsWith('bottom');

    return (
        <div
            className={`toast-container ${placement}`}
            onMouseEnter={() => setIsHovered(true)}
            onMouseLeave={() => setIsHovered(false)}
        >
            {/* Wrapper for stacked cards - relative positioned to contain absolutes */}
            <div
                className="toast-stack-wrapper"
                style={{
                    position: 'relative',
                    width: '100%',
                    minHeight: recentToasts.length > 0 ? '70px' : '0',
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: isBottom ? 'flex-end' : 'flex-start',
                }}
            >
                <AnimatePresence mode="popLayout">
                    {recentToasts.map((toast, index) => (
                        <ToastItem
                            key={toast.id}
                            toast={toast}
                            onDismiss={onDismiss}
                            index={index}
                            total={recentToasts.length}
                            isHovered={isHovered}
                            placement={placement}
                        />
                    ))}
                </AnimatePresence>

                {/* Footer overlay - positioned at bottom-right of deck */}
                {toasts.length > 1 && (
                    <div className="toast-footer-overlay">
                        {toasts.length > maxVisible && (
                            <span className="toast-count-badge">
                                +{toasts.length - maxVisible}
                            </span>
                        )}
                        <button
                            className="toast-clear-all"
                            onClick={onDismissAll}
                            title="Clear all notifications"
                        >
                            <MdClearAll size={14} />
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}

// Individual Toast Item with card stacking effect
function ToastItem({ toast, onDismiss, index, total, isHovered, placement }) {
    const [progress, setProgress] = useState(100);
    const [isPaused, setIsPaused] = useState(false);

    const { id, title, description, color, variant, icon, hideIcon, duration, endContent, isLoading } = toast;

    // Only the front toast (index 0) should auto-dismiss
    const shouldAutoDismiss = index === 0;

    // Handle auto-dismiss with progress
    useEffect(() => {
        if (!shouldAutoDismiss || duration <= 0 || isPaused) return;

        const startTime = Date.now();
        const interval = setInterval(() => {
            const elapsed = Date.now() - startTime;
            const remaining = Math.max(0, 100 - (elapsed / duration) * 100);
            setProgress(remaining);

            if (remaining <= 0) {
                clearInterval(interval);
                onDismiss(id);
            }
        }, 50);

        return () => clearInterval(interval);
    }, [id, duration, isPaused, onDismiss, shouldAutoDismiss]);

    // Get the appropriate icon
    const getIcon = () => {
        if (isLoading) {
            return (
                <svg className="toast-spinner" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <circle cx="12" cy="12" r="10" strokeOpacity="0.25"></circle>
                    <path d="M12 2a10 10 0 0 1 10 10" strokeLinecap="round"></path>
                </svg>
            );
        }
        if (icon) return icon;
        if (color === 'primary' || color === 'secondary') return Icons.info;
        return Icons[color] || Icons.default;
    };

    // Card stacking calculations - expands on hover
    const isBottom = placement.startsWith('bottom');

    // When hovered, expand the cards; when not hovered, stack them tightly
    const stackOffset = isHovered ? 100 : 10; // Fan out on hover (70px between cards)
    const scaleReduction = isHovered ? 0 : 0.05; // No scale reduction when hovered
    const opacityReduction = isHovered ? 0 : 0.2; // Full opacity when hovered

    // Calculate transforms for stacking effect
    const yOffset = isBottom
        ? -index * stackOffset  // Stack upward for bottom placement
        : index * stackOffset;  // Stack downward for top placement
    const scale = 1 - (index * scaleReduction);
    const opacity = index === 0 ? 1 : Math.max(0.6, 1 - (index * opacityReduction));
    const zIndex = 100 - index;

    return (
        <motion.div
            layout
            initial={{
                opacity: 0,
                y: isBottom ? 100 : -100,
                scale: 0.8
            }}
            animate={{
                opacity: opacity,
                y: yOffset,
                scale: scale,
            }}
            exit={{
                opacity: 0,
                y: isBottom ? 50 : -50,
                scale: 0.8
            }}
            transition={{
                type: 'spring',
                damping: 25,
                stiffness: 300,
                mass: 0.8
            }}
            className={`toast-item ${color} ${variant}`}
            style={{
                position: 'absolute',
                bottom: isBottom ? 0 : 'auto',
                top: !isBottom ? 0 : 'auto',
                left: 0,
                right: 0,
                zIndex: zIndex,
                transformOrigin: isBottom ? 'bottom center' : 'top center',
            }}
            onMouseEnter={() => setIsPaused(true)}
            onMouseLeave={() => setIsPaused(false)}
        >
            {!hideIcon && (
                <div className="toast-icon">
                    {getIcon()}
                </div>
            )}

            <div className="toast-content">
                {title && <div className="toast-title">{title}</div>}
                {description && <div className="toast-description">{description}</div>}
            </div>

            {endContent && (
                <div className="toast-end-content">
                    {endContent}
                </div>
            )}

            <button
                className="toast-close"
                onClick={() => onDismiss(id)}
                aria-label="Close"
            >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
            </button>

            {/* Progress bar - only on front toast */}
            {shouldAutoDismiss && duration > 0 && (
                <div className="toast-progress-track">
                    <div
                        className="toast-progress-bar"
                        style={{ transform: `scaleX(${progress / 100})` }}
                    />
                </div>
            )}
        </motion.div>
    );
}

export default AlertProvider;

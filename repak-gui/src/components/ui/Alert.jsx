import React, { useEffect, useState } from 'react';
import './Alert.css';

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
  secondary: (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="12" y1="16" x2="12" y2="12"></line>
      <line x1="12" y1="8" x2="12.01" y2="8"></line>
    </svg>
  )
};

export function Alert({
  title,
  description,
  color = 'default',
  variant = 'flat',
  icon,
  hideIcon = false,
  isVisible = true,
  isClosable = false,
  startContent,
  endContent,
  onClose,
  onVisibleChange,
  closeButtonProps = {},
  className = '',
  ...props
}) {
  // Call onVisibleChange when visibility changes
  useEffect(() => {
    if (onVisibleChange) {
      onVisibleChange(isVisible);
    }
  }, [isVisible, onVisibleChange]);

  if (!isVisible) return null;

  // Determine the icon to display
  const getDefaultIcon = () => {
    if (color === 'primary' || color === 'default') return Icons.info;
    return Icons[color] || Icons.info;
  };

  const displayIcon = icon !== undefined ? icon : getDefaultIcon();
  const showCloseButton = isClosable || onClose;

  return (
    <div
      className={`toast-item ${color} ${variant} ${className}`}
      role="alert"
      {...props}
    >
      {startContent && (
        <div className="toast-start-content">
          {startContent}
        </div>
      )}
      {!hideIcon && displayIcon && (
        <div className="toast-icon">
          {displayIcon}
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
      {showCloseButton && (
        <button
          className="toast-close"
          onClick={onClose}
          aria-label="Close"
          {...closeButtonProps}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      )}
    </div>
  );
}

import React from 'react';
import './ShineBorder.css';

export function ShineBorder({
  children,
  className = '',
  duration = 14,
  shineColor = '#ffffff',
  borderWidth = 1,
  borderRadius = 12,
  style = {},
  ...props
}) {
  return (
    <div
      className={`shine-border ${className}`}
      style={{
        '--duration': `${duration}s`,
        '--shine-color': Array.isArray(shineColor) ? shineColor.join(',') : shineColor,
        '--border-width': `${borderWidth}px`,
        '--border-radius': `${borderRadius}px`,
        ...style
      }}
      {...props}
    >
      {children}
    </div>
  );
}

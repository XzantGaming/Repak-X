import React from 'react';
import './AuroraText.css';

export function AuroraText({
  children,
  className = '',
  speed = 1,
  useAccent = true // When true, uses theme accent colors
}) {
  // By default, use accent colors from CSS variables
  // The gradient is set in CSS to use --accent-primary and --accent-secondary
  const style = {
    '--aurora-speed': `${10 / speed}s`,
  };

  return (
    <span
      className={`aurora-text ${useAccent ? 'aurora-text--accent' : ''} ${className}`}
      style={style}
    >
      {children}
    </span>
  );
}

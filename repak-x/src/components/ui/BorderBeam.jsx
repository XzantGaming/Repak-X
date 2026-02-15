import React from 'react';
import './BorderBeam.css';

export function BorderBeam({
  className = '',
  size = 200,
  duration = 10,
  delay = 0,
  colorFrom = '#ffaa40',
  colorTo = '#9c40ff',
  borderWidth = 1.5,
  style = {},
}) {
  return (
    <div 
      className={`border-beam-path ${className}`}
      style={{
        '--size': `${size}px`,
        '--duration': `${duration}s`,
        '--delay': `${delay}s`,
        '--color-from': colorFrom,
        '--color-to': colorTo,
        '--border-width': `${borderWidth}px`,
        ...style
      }}
    >
      <div className="border-beam-light" />
    </div>
  );
}

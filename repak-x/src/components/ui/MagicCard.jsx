import React, { useRef, useState } from 'react';
import './MagicCard.css';

export function MagicCard({ 
  children, 
  className = '', 
  gradientSize = 200, 
  gradientColor = 'rgba(255, 255, 255, 0.1)',
  gradientOpacity = 0.8,
  ...props 
}) {
  const cardRef = useRef(null);
  const [position, setPosition] = useState({ x: 0, y: 0 });

  const handleMouseMove = (e) => {
    if (!cardRef.current) return;
    const rect = cardRef.current.getBoundingClientRect();
    setPosition({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    });
  };

  return (
    <div
      ref={cardRef}
      className={`magic-card ${className}`}
      onMouseMove={handleMouseMove}
      style={{
        '--mouse-x': `${position.x}px`,
        '--mouse-y': `${position.y}px`,
        '--gradient-size': `${gradientSize}px`,
        '--gradient-color': gradientColor,
        '--gradient-opacity': gradientOpacity,
      }}
      {...props}
    >
      <div className="magic-card-content">{children}</div>
    </div>
  );
}

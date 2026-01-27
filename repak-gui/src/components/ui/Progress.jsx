import React from 'react';
import './Progress.css';

/**
 * repakx-style Progress component
 * 
 * @param {Object} props
 * @param {number} [props.value=0] - Current value (0-100)
 * @param {number} [props.minValue=0] - Minimum value
 * @param {number} [props.maxValue=100] - Maximum value
 * @param {string} [props.label] - Label text to display above the bar
 * @param {boolean} [props.showValueLabel=false] - Whether to show the value text
 * @param {string} [props.size='md'] - 'sm', 'md', 'lg'
 * @param {string} [props.color='primary'] - 'default', 'primary', 'secondary', 'success', 'warning', 'danger'
 * @param {boolean} [props.isIndeterminate=false] - Whether the progress is indeterminate
 * @param {boolean} [props.isStriped=false] - Whether to show stripes
 * @param {string} [props.className] - Additional classes
 */
const Progress = ({
  value = 0,
  minValue = 0,
  maxValue = 100,
  label,
  showValueLabel = false,
  size = 'md',
  color = 'primary',
  isIndeterminate = false,
  isStriped = false,
  className = '',
  ...props
}) => {
  // Calculate percentage for determinate progress
  const percentage = Math.min(Math.max(((value - minValue) / (maxValue - minValue)) * 100, 0), 100);
  
  const classes = [
    'repakx-progress',
    size,
    color,
    isIndeterminate ? 'indeterminate' : '',
    isStriped ? 'striped' : '',
    className
  ].filter(Boolean).join(' ');

  return (
    <div 
      className={classes}
      role="progressbar"
      aria-valuenow={isIndeterminate ? undefined : value}
      aria-valuemin={minValue}
      aria-valuemax={maxValue}
      aria-label={label || 'Loading...'}
      {...props}
    >
      {(label || showValueLabel) && (
        <div className="repakx-progress-header">
          {label && <span className="repakx-progress-label">{label}</span>}
          {showValueLabel && !isIndeterminate && (
            <span className="repakx-progress-value">{Math.round(percentage)}%</span>
          )}
        </div>
      )}
      
      <div className="repakx-progress-track">
        <div 
          className="repakx-progress-indicator" 
          style={{ width: isIndeterminate ? undefined : `${percentage}%` }}
        />
      </div>
    </div>
  );
};

export default Progress;

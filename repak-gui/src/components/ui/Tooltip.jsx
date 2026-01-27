import React, { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import './Tooltip.css';

/**
 * repakx-style Tooltip component
 * 
 * @param {Object} props
 * @param {React.ReactNode} props.children - The trigger element
 * @param {React.ReactNode} props.content - The content of the tooltip
 * @param {string} [props.placement='top'] - 'top', 'bottom', 'left', 'right'
 * @param {string} [props.color='default'] - 'default', 'primary', 'secondary', 'success', 'warning', 'danger'
 * @param {string} [props.size='md'] - 'sm', 'md', 'lg'
 * @param {string} [props.radius='md'] - 'none', 'sm', 'md', 'lg', 'full'
 * @param {number} [props.delay=0] - Delay in ms before opening
 * @param {number} [props.closeDelay=500] - Delay in ms before closing
 * @param {number} [props.offset=7] - Distance from trigger
 * @param {boolean} [props.showArrow=false] - Whether to show the arrow
 * @param {boolean} [props.isDisabled=false] - Whether the tooltip is disabled
 * @param {string} [props.className] - Additional classes for the tooltip content
 */
const Tooltip = ({
  children,
  content,
  placement = 'top',
  color = 'default',
  size = 'md',
  radius = 'md',
  delay = 0,
  closeDelay = 100,
  offset = 7,
  showArrow = false,
  isDisabled = false,
  className = '',
  ...props
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const openTimeout = useRef(null);
  const closeTimeout = useRef(null);

  const handleMouseEnter = () => {
    if (isDisabled) return;
    clearTimeout(closeTimeout.current);
    
    if (delay > 0) {
      openTimeout.current = setTimeout(() => setIsVisible(true), delay);
    } else {
      setIsVisible(true);
    }
  };

  const handleMouseLeave = () => {
    clearTimeout(openTimeout.current);
    
    if (closeDelay > 0) {
      closeTimeout.current = setTimeout(() => setIsVisible(false), closeDelay);
    } else {
      setIsVisible(false);
    }
  };

  useEffect(() => {
    return () => {
      clearTimeout(openTimeout.current);
      clearTimeout(closeTimeout.current);
    };
  }, []);

  // Animation variants with directional slide
  const getAnimationVariants = () => {
    const slideDistance = 8;
    switch (placement) {
      case 'top':
        return {
          initial: { opacity: 0, scale: 0.92, y: slideDistance },
          animate: { opacity: 1, scale: 1, y: 0 },
          exit: { opacity: 0, scale: 0.92, y: slideDistance / 2 }
        };
      case 'bottom':
        return {
          initial: { opacity: 0, scale: 0.92, y: -slideDistance },
          animate: { opacity: 1, scale: 1, y: 0 },
          exit: { opacity: 0, scale: 0.92, y: -slideDistance / 2 }
        };
      case 'left':
        return {
          initial: { opacity: 0, scale: 0.92, x: slideDistance },
          animate: { opacity: 1, scale: 1, x: 0 },
          exit: { opacity: 0, scale: 0.92, x: slideDistance / 2 }
        };
      case 'right':
        return {
          initial: { opacity: 0, scale: 0.92, x: -slideDistance },
          animate: { opacity: 1, scale: 1, x: 0 },
          exit: { opacity: 0, scale: 0.92, x: -slideDistance / 2 }
        };
      default:
        return {
          initial: { opacity: 0, scale: 0.92 },
          animate: { opacity: 1, scale: 1 },
          exit: { opacity: 0, scale: 0.92 }
        };
    }
  };

  // Adjust transform origin based on placement for better animation
  const getTransformOrigin = () => {
    switch (placement) {
      case 'top': return 'bottom center';
      case 'bottom': return 'top center';
      case 'left': return 'center right';
      case 'right': return 'center left';
      default: return 'center';
    }
  };

  return (
    <div 
      className="repakx-tooltip-trigger"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onFocus={handleMouseEnter}
      onBlur={handleMouseLeave}
    >
      {children}
      <AnimatePresence>
        {isVisible && content && (
          <motion.div
            className={`repakx-tooltip-content ${color} ${size} radius-${radius} ${className}`}
            data-placement={placement}
            style={{ 
              '--tooltip-offset': `${offset}px`,
              transformOrigin: getTransformOrigin()
            }}
            initial="initial"
            animate="animate"
            exit="exit"
            variants={getAnimationVariants()}
            transition={{ 
              type: 'spring',
              damping: 20,
              stiffness: 300,
              mass: 0.8
            }}
            {...props}
          >
            {showArrow && <div className="repakx-tooltip-arrow" />}
            {content}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default Tooltip;

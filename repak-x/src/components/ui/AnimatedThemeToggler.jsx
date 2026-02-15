import React, { useRef } from 'react';
import { motion } from "framer-motion";
import { LuMoon, LuSun } from "react-icons/lu";

export function AnimatedThemeToggler({ theme, setTheme, duration = 400 }) {
  const isDark = theme === "dark";
  const buttonRef = useRef(null);

  const toggleTheme = (e) => {
    const newTheme = isDark ? "light" : "dark";
    
    // Check if View Transitions API is supported
    if (document.startViewTransition) {
      // Get button position for the animation origin
      const rect = buttonRef.current?.getBoundingClientRect();
      const x = rect ? rect.left + rect.width / 2 : e.clientX;
      const y = rect ? rect.top + rect.height / 2 : e.clientY;
      
      // Calculate the maximum radius needed to cover the entire screen
      const maxRadius = Math.hypot(
        Math.max(x, window.innerWidth - x),
        Math.max(y, window.innerHeight - y)
      );
      
      // Set CSS custom properties for the animation
      document.documentElement.style.setProperty('--theme-toggle-x', `${x}px`);
      document.documentElement.style.setProperty('--theme-toggle-y', `${y}px`);
      document.documentElement.style.setProperty('--theme-toggle-radius', `${maxRadius}px`);
      document.documentElement.style.setProperty('--theme-toggle-duration', `${duration}ms`);
      
      document.startViewTransition(() => {
        setTheme(newTheme);
      });
    } else {
      // Fallback for browsers without View Transitions API
      setTheme(newTheme);
    }
  };

  return (
    <>
      <style>{`
        ::view-transition-old(root),
        ::view-transition-new(root) {
          animation: none;
          mix-blend-mode: normal;
        }
        
        ::view-transition-old(root) {
          z-index: 1;
        }
        
        ::view-transition-new(root) {
          z-index: 9999;
          animation: theme-toggle-reveal var(--theme-toggle-duration, 400ms) ease-out;
        }
        
        @keyframes theme-toggle-reveal {
          from {
            clip-path: circle(0px at var(--theme-toggle-x, 50%) var(--theme-toggle-y, 50%));
          }
          to {
            clip-path: circle(var(--theme-toggle-radius, 100vmax) at var(--theme-toggle-x, 50%) var(--theme-toggle-y, 50%));
          }
        }
      `}</style>
      <button
        ref={buttonRef}
        onClick={toggleTheme}
        style={{ 
          background: 'var(--bg-dark)', 
          border: '1px solid var(--panel-border)',
          cursor: 'pointer',
          position: 'relative',
          overflow: 'hidden',
          width: '40px',
          height: '40px',
          borderRadius: '50%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: 0
        }}
        aria-label="Toggle theme"
        title={`Switch to ${isDark ? 'light' : 'dark'} mode`}
      >
        <motion.div
          initial={false}
          animate={{
            scale: isDark ? 1 : 0,
            rotate: isDark ? 0 : 90,
            opacity: isDark ? 1 : 0,
          }}
          transition={{ duration: 0.2, ease: "easeInOut" }}
          style={{ position: 'absolute' }}
        >
          <LuMoon size={20} style={{ color: 'var(--text-primary)' }} />
        </motion.div>
        <motion.div
          initial={false}
          animate={{
            scale: isDark ? 0 : 1,
            rotate: isDark ? -90 : 0,
            opacity: isDark ? 0 : 1,
          }}
          transition={{ duration: 0.2, ease: "easeInOut" }}
          style={{ position: 'absolute' }}
        >
          <LuSun size={20} style={{ color: 'var(--text-primary)' }} />
        </motion.div>
      </button>
    </>
  );
}

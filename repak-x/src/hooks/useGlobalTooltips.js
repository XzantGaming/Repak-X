import { useEffect } from 'react';

/**
 * Hook to enhance native browser tooltips with custom styled tooltips
 * Intercepts title attributes and shows custom tooltips instead
 */
export const useGlobalTooltips = () => {
  useEffect(() => {
    let activeTooltip = null;
    let showTimeout = null;
    let hideTimeout = null;

    const createTooltip = (text, targetRect) => {
      const tooltip = document.createElement('div');
      tooltip.className = 'global-tooltip';
      tooltip.textContent = text;
      tooltip.style.visibility = 'hidden'; // Hide while measuring
      document.body.appendChild(tooltip);

      // Force layout to get accurate dimensions
      tooltip.offsetHeight;

      // Position tooltip
      const tooltipRect = tooltip.getBoundingClientRect();
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;

      // Default to top placement
      let top = targetRect.top - tooltipRect.height - 8;
      let left = targetRect.left + targetRect.width / 2 - tooltipRect.width / 2;

      tooltip.style.visibility = ''; // Show after positioning

      // Adjust if tooltip goes off screen
      if (top < 8) {
        // Show below if no space above
        top = targetRect.bottom + 8;
        tooltip.setAttribute('data-placement', 'bottom');
      } else {
        tooltip.setAttribute('data-placement', 'top');
      }

      if (left < 8) {
        left = 8;
      } else if (left + tooltipRect.width > viewportWidth - 8) {
        left = viewportWidth - tooltipRect.width - 8;
      }

      tooltip.style.top = `${top}px`;
      tooltip.style.left = `${left}px`;

      // Trigger animation
      requestAnimationFrame(() => {
        tooltip.classList.add('visible');
      });

      return tooltip;
    };

    const removeTooltip = () => {
      if (activeTooltip) {
        activeTooltip.classList.remove('visible');
        setTimeout(() => {
          if (activeTooltip && activeTooltip.parentNode) {
            activeTooltip.parentNode.removeChild(activeTooltip);
          }
          activeTooltip = null;
        }, 200);
      }
    };

    let currentTarget = null;

    const handleMouseEnter = (e) => {
      const target = e.target.closest('[title]');
      if (!target || target.hasAttribute('data-no-global-tooltip')) return;

      const title = target.getAttribute('title');
      if (!title) return;

      // Store current target
      currentTarget = target;

      // Store original title and remove it to prevent native tooltip
      target.setAttribute('data-original-title', title);
      target.removeAttribute('title');

      clearTimeout(hideTimeout);
      clearTimeout(showTimeout);

      showTimeout = setTimeout(() => {
        if (currentTarget === target) {
          const rect = target.getBoundingClientRect();
          activeTooltip = createTooltip(title, rect);
        }
      }, 500); // 500ms delay like native tooltips
    };

    const handleMouseLeave = (e) => {
      const target = e.target.closest('[data-original-title]');

      // Always clear timeouts and remove tooltip on any mouse leave
      clearTimeout(showTimeout);
      clearTimeout(hideTimeout);

      if (target) {
        // Restore original title
        const originalTitle = target.getAttribute('data-original-title');
        if (originalTitle) {
          target.setAttribute('title', originalTitle);
          target.removeAttribute('data-original-title');
        }
      }

      // Clear current target
      currentTarget = null;

      // Remove tooltip immediately
      removeTooltip();
    };

    const handleMouseDown = () => {
      clearTimeout(showTimeout);
      removeTooltip();
    };

    // Add event listeners
    document.addEventListener('mouseover', handleMouseEnter, true);
    document.addEventListener('mouseout', handleMouseLeave, true);
    document.addEventListener('mousedown', handleMouseDown, true);

    // Cleanup
    return () => {
      document.removeEventListener('mouseover', handleMouseEnter, true);
      document.removeEventListener('mouseout', handleMouseLeave, true);
      document.removeEventListener('mousedown', handleMouseDown, true);
      clearTimeout(showTimeout);
      clearTimeout(hideTimeout);
      removeTooltip();
    };
  }, []);
};

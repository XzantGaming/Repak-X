import React from 'react';
import maskX from '../../assets/app-icons/logomask_X.png';
import maskR from '../../assets/app-icons/logomask_R.png';
import './ModularLogo.css';

/**
 * ModularLogo - A theme-aware layered logo that uses CSS masking.
 * The X layer responds to dark/light theme.
 * The R layer uses the accent color.
 */
export default function ModularLogo({ size = 50, className = '', style: externalStyle = {} }) {
    const style = {
        width: size,
        height: size,
        '--mask-x': `url(${maskX})`,
        '--mask-r': `url(${maskR})`,
        ...externalStyle,
    };

    return (
        <div className={`modular-logo ${className}`} style={style}>
            <div className="modular-logo__x" />
            <div className="modular-logo__r" />
        </div>
    );
}

import React from 'react';
import { FiExternalLink, FiHeart } from 'react-icons/fi';
import { SiDiscord } from 'react-icons/si';
import { open } from '@tauri-apps/plugin-shell';
import { AuroraText } from './ui/AuroraText';
import logo from '../assets/app-icons/RepakIcon-x256.png';
import './CreditsPanel.css';

const CONTRIBUTORS = [
    {
        name: 'Xzant',
        role: 'Backend Developer, Project Founder',
        avatar: 'https://cdn.discordapp.com/avatars/771103606010806283/e666c4287efbcc05d3d851626c5f9e56.webp',
        link: 'https://github.com/XzantGaming',
        badge: null
    },
    {
        name: 'Saturn',
        role: 'Frontend Developer, Vibe-Coder',
        avatar: 'https://i.imgur.com/mPEy8WX.jpeg',
        link: 'https://github.com/0xSaturno',
        badge: null
    }
];

const SPECIAL_THANKS = [
    {
        name: 'Marvel Rivals Modding Server',
        role: 'Where it all started',
        avatar: null,
        icon: '🎮',
        link: 'https://discord.gg/marvelrivalsmodding',
        badge: 'community'
    },
    {
        name: 'Placeholder',
        role: 'Role Description',
        avatar: null,
        icon: '👤',
        link: null,
        badge: null
    },
    {
        name: 'Placeholder',
        role: 'Role Description',
        avatar: null,
        icon: '👤',
        link: null,
        badge: null
    },
    {
        name: 'Placeholder',
        role: 'Role Description',
        avatar: null,
        icon: '👤',
        link: null,
        badge: null
    }
];

export default function CreditsPanel({ onClose, version }) {
    const handleLinkClick = (e, link) => {
        e.preventDefault();
        e.stopPropagation();
        if (link) {
            open(link);
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-content credits-modal" onClick={(e) => e.stopPropagation()}>
                <div className="modal-header">
                    <h2>Credits</h2>
                    <button className="modal-close" onClick={onClose}>×</button>
                </div>

                <div className="modal-body">
                    <div className="credits-content">
                        {/* App Branding */}
                        <div className="credits-branding">
                            <img
                                src={logo}
                                alt="Repak X"
                                className="credits-logo"
                            />
                            <h1 className="credits-app-name">
                                <span className="credits-app-name-repak">Repak </span>
                                <AuroraText className="credits-app-name-x">X</AuroraText>
                            </h1>
                            <p className="credits-version">Version {version || '1.0.0'}</p>
                            <p className="credits-tagline">Mod Manager & Modding Tool for Marvel Rivals</p>
                        </div>

                        {/* Main Contributors */}
                        <div className="credits-section">
                            <h3 className="credits-section-title">Contributors</h3>
                            {CONTRIBUTORS.map((contributor, index) => (
                                <a
                                    key={index}
                                    href={contributor.link || '#'}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="credits-contributor"
                                    onClick={(e) => handleLinkClick(e, contributor.link)}
                                    style={{ cursor: contributor.link ? 'pointer' : 'default' }}
                                >
                                    <div className="credits-avatar">
                                        {contributor.avatar ? (
                                            <img src={contributor.avatar} alt={contributor.name} />
                                        ) : (
                                            contributor.icon || contributor.name.charAt(0)
                                        )}
                                    </div>
                                    <div className="credits-info">
                                        <p className="credits-name">
                                            {contributor.name}
                                            {contributor.badge && (
                                                <span className={`credits-badge ${contributor.badge}`}>
                                                    {contributor.badge === 'ai' ? 'AI' : contributor.badge}
                                                </span>
                                            )}
                                        </p>
                                        <p className="credits-role">{contributor.role}</p>
                                    </div>
                                    {contributor.link && (
                                        <FiExternalLink className="credits-link-icon" />
                                    )}
                                </a>
                            ))}
                        </div>

                        {/* Special Thanks */}
                        <div className="credits-section">
                            <h3 className="credits-section-title">Special Thanks</h3>
                            {SPECIAL_THANKS.map((contributor, index) => (
                                <a
                                    key={index}
                                    href={contributor.link || '#'}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="credits-contributor"
                                    onClick={(e) => handleLinkClick(e, contributor.link)}
                                    style={{ cursor: contributor.link ? 'pointer' : 'default' }}
                                >
                                    <div className="credits-avatar">
                                        {contributor.avatar ? (
                                            <img src={contributor.avatar} alt={contributor.name} />
                                        ) : (
                                            contributor.icon || contributor.name.charAt(0)
                                        )}
                                    </div>
                                    <div className="credits-info">
                                        <p className="credits-name">
                                            {contributor.name}
                                            {contributor.badge && (
                                                <span className={`credits-badge ${contributor.badge}`}>
                                                    {contributor.badge}
                                                </span>
                                            )}
                                        </p>
                                        <p className="credits-role">{contributor.role}</p>
                                    </div>
                                    {contributor.link && (
                                        <SiDiscord className="credits-link-icon" />
                                    )}
                                </a>
                            ))}
                        </div>

                        {/* Thank You Message */}
                        <div className="credits-thanks">
                            <p className="credits-thanks-text">
                                Made with <span className="credits-heart"><FiHeart style={{ verticalAlign: 'middle' }} /></span> for the Marvel Rivals community
                            </p>
                        </div>
                    </div>
                </div>

                <div className="modal-footer">
                    <button onClick={onClose} className="btn-primary">
                        Close
                    </button>
                </div>
            </div>
        </div>
    );
}

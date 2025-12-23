import React from 'react';
import { FiExternalLink, FiHeart } from 'react-icons/fi';
import { SiDiscord } from 'react-icons/si';
import { IoLogoGithub } from 'react-icons/io';
import { open } from '@tauri-apps/plugin-shell';

// Helper to determine link icon
const getLinkIcon = (link) => {
    if (!link) return null;
    if (link.includes('github.com')) return <IoLogoGithub className="credits-link-icon" />;
    if (link.includes('discord')) return <SiDiscord className="credits-link-icon" />;
    return <FiExternalLink className="credits-link-icon" />;
};
import { AuroraText } from './ui/AuroraText';
import ModularLogo from './ui/ModularLogo';
import mrmLogo from '../assets/extra/mrm_logo.png';
import './CreditsPanel.css';

const CONTRIBUTORS = [
    {
        name: 'Xzant',
        role: 'Backend Developer, Project Founder',
        avatar: 'https://cdn.discordapp.com/avatars/771103606010806283/e666c4287efbcc05d3d851626c5f9e56.webp',
        link: 'https://github.com/XzantGaming',
        badge: 'developer'
    },
    {
        name: 'Saturn',
        role: 'Frontend Developer, Vibe-Coder',
        avatar: 'https://i.imgur.com/mPEy8WX.jpeg',
        link: 'https://github.com/0xSaturno',
        badge: 'developer'
    }
];

const SPECIAL_THANKS = [
    {
        name: 'Marvel Rivals Modding Server',
        role: 'Where it all started',
        avatar: mrmLogo,
        icon: '🎮',
        link: 'https://discord.gg/marvelrivalsmodding',
        badge: 'community'
    },
    {
        name: 'Truman Kilen',
        role: 'Developer of original Repak and Retoc libraries',
        avatar: 'https://avatars.githubusercontent.com/u/1144160?v=4',
        link: 'https://github.com/trumank',
        badge: 'developer'
    },
    {
        name: 'Krisan Thyme',
        role: 'For reverse engineering the Rivals skeletal mesh format',
        avatar: 'https://avatars.githubusercontent.com/u/13863112?v=4',
        link: 'https://github.com/KrisanThyme',
        badge: 'developer'
    },
    {
        name: 'amMatt',
        role: 'MR Modding Discord Server Founder',
        avatar: 'https://cdn.discordapp.com/avatars/131187261428465664/c1e8dc637639cfe0d486b1c8ea5c1121.webp',
        link: 'https://github.com/amMattGIT',
        badge: 'developer'
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
                            <ModularLogo size={80} className="credits-logo" />
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
                                    {getLinkIcon(contributor.link)}
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
                                    {getLinkIcon(contributor.link)}
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

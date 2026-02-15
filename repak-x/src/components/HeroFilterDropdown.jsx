import React, { useState, useMemo } from 'react';
import './HeroFilterDropdown.css';

// Group characters by base hero name
const groupCharactersByHero = (characters, modCounts) => {
    const groups = {};

    characters.forEach(char => {
        // Skip special entries
        if (char === '__multi' || char === '__generic') return;

        // Check if it's a skin (contains " - ")
        const dashIndex = char.indexOf(' - ');
        if (dashIndex > 0) {
            const heroName = char.substring(0, dashIndex);
            const skinName = char.substring(dashIndex + 3); // Skip " - "

            if (!groups[heroName]) {
                groups[heroName] = {
                    baseName: heroName,
                    skins: [],
                    baseModCount: modCounts[heroName] || 0
                };
            }
            groups[heroName].skins.push({
                fullName: char,
                skinName,
                modCount: modCounts[char] || 0
            });
        } else {
            // It's a base hero (no skin suffix)
            if (!groups[char]) {
                groups[char] = {
                    baseName: char,
                    skins: [],
                    baseModCount: modCounts[char] || 0
                };
            } else {
                // Hero already exists from skins, just update base mod count
                groups[char].baseModCount = modCounts[char] || 0;
            }
        }
    });

    // Sort heroes and their skins
    return Object.values(groups)
        .sort((a, b) => a.baseName.localeCompare(b.baseName))
        .map(group => ({
            ...group,
            skins: group.skins.sort((a, b) => a.skinName.localeCompare(b.skinName))
        }));
};

// Calculate mod counts per character/skin from modDetails
const calculateModCounts = (modDetails) => {
    const counts = {};

    Object.values(modDetails || {}).forEach(details => {
        if (details?.character_name) {
            counts[details.character_name] = (counts[details.character_name] || 0) + 1;
        }
    });

    return counts;
};

const HeroFilterItem = ({ hero, selectedCharacters, onToggle }) => {
    const [isExpanded, setIsExpanded] = useState(false);

    // Gather all IDs for this hero group: baseName + all skin fullNames
    const allGroupIds = [hero.baseName, ...hero.skins.map(s => s.fullName)];

    // Check how many of this group are selected
    const selectedCountInGroup = allGroupIds.filter(id => selectedCharacters.has(id)).length;
    const isAllSelected = selectedCountInGroup === allGroupIds.length;
    const isSomeSelected = selectedCountInGroup > 0 && !isAllSelected;

    // Check specific selection for the base/default skin
    const isDefaultSelected = selectedCharacters.has(hero.baseName);

    const hasSkins = hero.skins.length > 0;

    // Total mod count for this hero (base + all skins)
    const totalModCount = hero.baseModCount + hero.skins.reduce((sum, s) => sum + s.modCount, 0);

    const handleGroupClick = (e) => {
        e.stopPropagation();
        // If all are selected, deselect all. Otherwise, select all.
        if (isAllSelected) {
            // Send array of IDs to toggle (which will remove them if they are present)
            // But we need to communicate "remove these" vs "add these". 
            // The App.jsx handler needs to be smart or we send a specific signal.
            // Let's assume onToggle handles "toggle these": if we want to force select/deselect,
            // we might need a more robust contract.
            // SIMPLE APPROACH: Pass array. App.jsx will check:
            // "Are ALL of these currently selected?" -> Remove all.
            // "Are SOME or NONE selected?" -> Add all (missing ones).
            onToggle(allGroupIds);
        } else {
            onToggle(allGroupIds);
        }
    };

    const handleExpandClick = (e) => {
        e.stopPropagation();
        setIsExpanded(!isExpanded);
    };

    const handleDefaultSkinClick = (e) => {
        e.stopPropagation();
        onToggle(hero.baseName);
    };

    const handleSkinClick = (e, skinFullName) => {
        e.stopPropagation();
        onToggle(skinFullName);
    };

    // Determine the visual state of the parent button
    const parentClass = `hero-filter-base ${isAllSelected ? 'active' : ''} ${isSomeSelected ? 'partial' : ''}`;

    return (
        <div className="hero-filter-item">
            <div className="hero-filter-row">
                <button
                    className="hero-expand-btn"
                    onClick={handleExpandClick}
                    style={{ visibility: hasSkins || hero.baseModCount > 0 ? 'visible' : 'hidden' }}
                >
                    {isExpanded ? '▼' : '▶'}
                </button>
                <button
                    className={parentClass}
                    onClick={handleGroupClick}
                    title={`Toggle all ${hero.baseName} skins`}
                >
                    <span className="hero-name">{hero.baseName}</span>
                    {totalModCount > 0 && (
                        <span className="hero-mod-count">{totalModCount}</span>
                    )}
                </button>
            </div>

            {/* Nested items - shown when expanded */}
            {isExpanded && (
                <div className="hero-skins-nested">
                    {/* Explicit "Default Skin" entry if base mods exist */}
                    {hero.baseModCount > 0 && (
                        <button
                            className={`hero-skin-item ${isDefaultSelected ? 'active' : ''} default-skin-entry`}
                            onClick={handleDefaultSkinClick}
                            title={`${hero.baseName} (Default Skin)`}
                        >
                            <span className="skin-name">Default Skin</span>
                            <span className="skin-mod-count">{hero.baseModCount}</span>
                        </button>
                    )}

                    {/* Other Skins */}
                    {hero.skins.map(skin => (
                        <button
                            key={skin.fullName}
                            className={`hero-skin-item ${selectedCharacters.has(skin.fullName) ? 'active' : ''}`}
                            onClick={(e) => handleSkinClick(e, skin.fullName)}
                            title={skin.fullName}
                        >
                            <span className="skin-name">{skin.skinName}</span>
                            {skin.modCount > 0 && (
                                <span className="skin-mod-count">{skin.modCount}</span>
                            )}
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
};

export default function HeroFilterDropdown({ availableCharacters, selectedCharacters, modDetails, onToggle }) {
    const modCounts = useMemo(() => calculateModCounts(modDetails), [modDetails]);
    const heroGroups = useMemo(() => groupCharactersByHero(availableCharacters, modCounts), [availableCharacters, modCounts]);

    // Count mods for special categories
    const multiModCount = Object.values(modDetails || {}).filter(d =>
        d?.mod_type?.startsWith('Multiple Heroes')
    ).length;

    const genericModCount = Object.values(modDetails || {}).filter(d =>
        !d?.character_name || d?.character_name === 'Generic' || d?.character_name === 'Unknown'
    ).length;

    return (
        <div className="hero-filter-dropdown">
            {/* Special filters - always visible */}
            <div className="hero-filter-special">
                <button
                    className={`filter-chip-compact ${selectedCharacters.has('__multi') ? 'active' : ''}`}
                    onClick={() => onToggle('__multi')}
                    title="Multiple Heroes"
                >
                    Multi {multiModCount > 0 && <span className="chip-count">{multiModCount}</span>}
                </button>
                <button
                    className={`filter-chip-compact ${selectedCharacters.has('__generic') ? 'active' : ''}`}
                    onClick={() => onToggle('__generic')}
                    title="Generic/Global"
                >
                    Generic {genericModCount > 0 && <span className="chip-count">{genericModCount}</span>}
                </button>
            </div>

            {/* Scrollable hero list */}
            <div className="hero-filter-list">
                {heroGroups.map(hero => (
                    <HeroFilterItem
                        key={hero.baseName}
                        hero={hero}
                        selectedCharacters={selectedCharacters}
                        onToggle={onToggle}
                    />
                ))}
            </div>
        </div>
    );
}

import React, { useState, useRef } from 'react'
import { motion } from 'framer-motion'
import { Tooltip } from '@mui/material'
import { RiDeleteBin2Fill } from 'react-icons/ri'
import { FaTag } from "react-icons/fa6"
import Checkbox from './ui/Checkbox'
import Switch from './ui/Switch'
import NumberInput from './ui/NumberInput'
import './ModsList.css'
import './ModDetailsPanel.css'

const heroImages = import.meta.glob('../assets/hero/*.png', { eager: true })

const toTagArray = (tags) => Array.isArray(tags) ? tags : (tags ? [tags] : [])

// Get hero image by character name
function getHeroImage(heroName, characterData) {
    const fallbackKey = '../assets/hero/9999.png'
    const fallbackImage = heroImages[fallbackKey]?.default
    const charData = characterData || []

    // Return fallback for missing, Unknown, or Multiple Heroes
    if (!heroName) return fallbackImage
    if (heroName.toLowerCase().includes('unknown') || heroName.toLowerCase().includes('multiple')) {
        return fallbackImage
    }

    // Check for ID at start (e.g. "1025XXX" -> 1025)
    const idMatch = heroName.match(/^(10\d{2})/)
    if (idMatch) {
        const id = idMatch[1]
        const key = `../assets/hero/${id}.png`
        if (heroImages[key]) return heroImages[key].default
    }

    // Find by name in character data (partial match)
    const char = charData.find(c => heroName.includes(c.name))
    if (char) {
        const key = `../assets/hero/${char.id}.png`
        if (heroImages[key]?.default) return heroImages[key].default
    }

    // Fallback: Try to find any 4-digit hero ID (10XX) pattern anywhere in name
    const anyIdMatch = heroName.match(/\b(10\d{2})\b/)
    if (anyIdMatch) {
        const id = anyIdMatch[1]
        const key = `../assets/hero/${id}.png`
        if (heroImages[key]) return heroImages[key].default
    }

    // Last resort: Check if heroName exactly matches any character and use their ID
    const charExact = charData.find(c => c.name === heroName)
    if (charExact) {
        const key = `../assets/hero/${charExact.id}.png`
        if (heroImages[key]?.default) return heroImages[key].default
    }

    return fallbackImage
}

// Mod Item Component
function ModItem({
    mod,
    selectedMod,
    selectedMods,
    handleToggleModSelection,
    onSelect,
    handleToggleMod,
    handleSetPriority,
    handleDeleteMod,
    handleRemoveTag,
    formatFileSize,
    hideSuffix,
    onContextMenu,
    showHeroIcons,
    characterName,
    viewMode,
    characterData
}) {
    const [isDeleteHolding, setIsDeleteHolding] = useState(false)
    const holdTimeoutRef = useRef(null)
    const rawName = mod.custom_name || mod.path.split('\\').pop()
    const nameWithoutExt = rawName.replace(/\.[^/.]+$/, '')

    // Identify all trailing priority suffixes (e.g. _9999999_P_9999999_P)
    const suffixGroupMatch = nameWithoutExt.match(/((?:_\d+_P)+)$/i)
    const fullSuffixGroup = suffixGroupMatch ? suffixGroupMatch[1] : ''

    // Extract the last single suffix for display
    const lastSuffixMatch = fullSuffixGroup.match(/(_\d+_P)$/i)
    const suffix = lastSuffixMatch ? lastSuffixMatch[1] : ''

    // Clean name is the base name without ANY priority suffixes
    const cleanBaseName = fullSuffixGroup
        ? nameWithoutExt.substring(0, nameWithoutExt.length - fullSuffixGroup.length)
        : nameWithoutExt

    const cleanName = cleanBaseName
    const shouldShowSuffix = !hideSuffix && suffix
    const tags = toTagArray(mod.custom_tags)
    const MAX_VISIBLE_TAGS = 3
    const visibleTags = tags.slice(0, MAX_VISIBLE_TAGS)
    const hiddenTags = tags.slice(MAX_VISIBLE_TAGS)

    const startDeleteHold = (e) => {
        e.stopPropagation()
        setIsDeleteHolding(true)
        holdTimeoutRef.current = setTimeout(() => {
            handleDeleteMod(mod.path, e.shiftKey)
            setIsDeleteHolding(false)
        }, 2000)
    }

    const cancelDeleteHold = (e) => {
        e?.stopPropagation()
        if (holdTimeoutRef.current) {
            clearTimeout(holdTimeoutRef.current)
            holdTimeoutRef.current = null
        }
        setIsDeleteHolding(false)
    }

    // Get hero image for background/badge
    const heroImage = showHeroIcons ? getHeroImage(characterName, characterData) : null
    const isCardView = viewMode === 'grid' || viewMode === 'compact'

    return (
        <motion.div
            layout
            className={`mod-card ${selectedMods.has(mod.path) ? 'selected' : ''} ${selectedMod?.path === mod.path ? 'viewing' : ''} ${heroImage && showHeroIcons ? 'has-hero-bg' : ''}`}
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            transition={{ duration: 0.2, layout: { duration: 0.3, ease: "circOut" } }}
            onContextMenu={(e) => onContextMenu(e, mod)}
        >
            {/* Blurred hero background for all views */}
            {heroImage && showHeroIcons && (
                <div
                    className="mod-card-hero-bg"
                    style={{ backgroundImage: `url(${heroImage})` }}
                />
            )}
            <div className="mod-main-row">
                <div className="mod-checkbox-wrapper">
                    <Checkbox
                        checked={selectedMods.has(mod.path)}
                        onChange={(checked, e) => {
                            e?.stopPropagation()
                            handleToggleModSelection(mod)
                        }}
                        size="sm"
                        radius="sm"
                        color="primary"
                        className="mod-checkbox"
                    />
                </div>
                {/* Hero icon before name in list view */}
                {heroImage && showHeroIcons && viewMode === 'list' && (
                    <img src={heroImage} alt="" className="mod-hero-icon-inline" />
                )}
                <motion.button
                    type="button"
                    className="mod-name-button"
                    onClick={(e) => {
                        if (e.ctrlKey || e.metaKey) {
                            handleToggleModSelection(mod)
                        } else {
                            onSelect(mod)
                        }
                    }}
                    whileHover={{ color: 'var(--accent-primary)' }}
                    title={rawName}
                >
                    <span className="mod-name-text">
                        {cleanName}
                        {shouldShowSuffix && <span className="mod-name-suffix">{suffix}</span>}
                    </span>
                </motion.button>
            </div>

            {/* Hero icon + Tags row for card views */}
            {(heroImage && showHeroIcons && isCardView) || tags.length > 0 ? (
                <div className="mod-tags-row">
                    {heroImage && showHeroIcons && isCardView && (
                        <Tooltip title={characterName || 'Unknown Hero'}>
                            <img src={heroImage} alt="" className="mod-hero-icon-badge" />
                        </Tooltip>
                    )}
                    {visibleTags.map(tag => (
                        <span key={tag} className="tag">
                            <FaTag />
                            {tag}
                            <button
                                type="button"
                                className="tag-remove"
                                aria-label={`Remove ${tag}`}
                                onClick={(e) => {
                                    e.stopPropagation()
                                    handleRemoveTag(mod.path, tag)
                                }}
                                style={{ background: 'none', border: 'none', color: 'inherit', marginLeft: 4, cursor: 'pointer', fontSize: 13 }}
                            >
                                ×
                            </button>
                        </span>
                    ))}
                    {hiddenTags.length > 0 && (
                        <Tooltip
                            title={
                                <div className="tags-tooltip-content">
                                    {hiddenTags.map(tag => (
                                        <span key={tag}>{tag}</span>
                                    ))}
                                </div>
                            }
                            arrow
                            placement="top"
                            slotProps={{
                                tooltip: {
                                    className: 'tags-tooltip'
                                },
                                arrow: {
                                    className: 'tags-tooltip-arrow'
                                }
                            }}
                        >
                            <span className="tag extra-tags-badge" style={{ cursor: 'help' }}>
                                +{hiddenTags.length}
                            </span>
                        </Tooltip>
                    )}
                </div>
            ) : null}

            <div className="mod-actions-row">
                <span className="mod-size">{formatFileSize(mod.file_size)}</span>
                <div className="actions-right">
                    <NumberInput
                        value={mod.priority || 0}
                        min={0}
                        max={7}
                        onChange={(newPriority) => handleSetPriority(mod.path, newPriority)}
                    />
                    <Tooltip title={mod.enabled ? 'Disable mod' : 'Enable mod'}>
                        <div className="mod-switch-wrapper" onClick={(e) => e.stopPropagation()}>
                            <Switch
                                size="sm"
                                color="primary"
                                checked={mod.enabled}
                                onChange={(_, event) => {
                                    event?.stopPropagation()
                                    handleToggleMod(mod.path)
                                }}
                                className="mod-switch"
                            />
                        </div>
                    </Tooltip>
                    <Tooltip title="Hold 2s to delete">
                        <button
                            className={`hold-delete ${isDeleteHolding ? 'holding' : ''}`}
                            onMouseDown={startDeleteHold}
                            onMouseUp={cancelDeleteHold}
                            onMouseLeave={cancelDeleteHold}
                            onTouchStart={startDeleteHold}
                            onTouchEnd={cancelDeleteHold}
                            aria-label="Hold to delete mod"
                        >
                            <RiDeleteBin2Fill size={18} />
                        </button>
                    </Tooltip>
                </div>
            </div>
        </motion.div >
    )
}

/**
 * ModsList Component
 * Renders the grid/list of mods state, utilizing virtualized rendering if needed (currently explicit)
 */
export default function ModsList({
    mods,
    viewMode,
    selectedMod,
    selectedMods,
    onSelect,
    onToggleSelection,
    onToggleMod,
    onDeleteMod,
    onRemoveTag,
    onSetPriority,
    onContextMenu,
    formatFileSize,
    hideSuffix,
    showHeroIcons,
    modDetails,
    characterData
}) {
    return (
        <div
            key={viewMode}
            className={`mod-list-grid view-${viewMode}`}
            style={{ flex: 1, overflowY: 'auto', padding: '1rem' }}
        >
            {mods.length === 0 ? (
                <div className="empty-state">
                    <p>No mods found in this folder.</p>
                </div>
            ) : (
                mods.map(mod => {
                    const details = modDetails?.[mod.path]
                    return (
                        <ModItem
                            key={mod.path}
                            mod={mod}
                            selectedMod={selectedMod}
                            selectedMods={selectedMods}
                            onSelect={onSelect}
                            handleToggleModSelection={onToggleSelection}
                            handleToggleMod={onToggleMod}
                            handleDeleteMod={onDeleteMod}
                            handleRemoveTag={onRemoveTag}
                            handleSetPriority={onSetPriority}
                            onContextMenu={onContextMenu}
                            formatFileSize={formatFileSize}
                            hideSuffix={hideSuffix}
                            showHeroIcons={showHeroIcons}
                            characterName={details?.character_name}
                            viewMode={viewMode}
                            characterData={characterData}
                        />
                    )
                })
            )}
        </div>
    )
}

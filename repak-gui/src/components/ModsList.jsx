import React, { useState, useRef, useEffect, memo } from 'react'
import { motion } from 'framer-motion'
import { Tooltip } from '@mui/material'
import { RiDeleteBin2Fill } from 'react-icons/ri'
import { FaTag } from "react-icons/fa6"
import Checkbox from './ui/Checkbox'
import Switch from './ui/Switch'
import NumberInput from './ui/NumberInput'
import { toTagArray } from '../utils/tags'
import { formatFileSize } from '../utils/format'
import { detectHeroesWithData } from '../utils/heroes'
import './ModsList.css'
import './ModDetailsPanel.css'

const heroImages = import.meta.glob('../assets/hero/*.png', { eager: true })

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

// Mod Item Component - Memoized for virtualization performance
const ModItem = memo(function ModItem({
    mod,
    selectedMod,
    selectedMods,
    handleToggleModSelection,
    onSelect,
    handleToggleMod,
    handleSetPriority,
    handleDeleteMod,
    handleRemoveTag,
    hideSuffix,
    onContextMenu,
    showHeroIcons,
    showHeroBg,
    showModType,
    characterName,
    category,
    viewMode,
    characterData,
    onRename,
    shouldStartRenaming,
    onClearRenaming,
    gameRunning,
    onRenameBlocked,
    onDeleteBlocked,
    modDetails
}) {
    const [isDeleteHolding, setIsDeleteHolding] = useState(false)
    const [isRenaming, setIsRenaming] = useState(false)
    const [renameValue, setRenameValue] = useState('')
    const holdTimeoutRef = useRef(null)
    const renameInputRef = useRef(null)
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
        if (gameRunning) {
            onDeleteBlocked?.()
            return
        }
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

    // Inline rename handlers
    const startRename = (e) => {
        e?.stopPropagation()
        if (gameRunning) {
            onRenameBlocked?.()
            return
        }
        setRenameValue(cleanName)
        setIsRenaming(true)
    }

    const handleRenameSubmit = () => {
        const trimmed = renameValue.trim()
        if (trimmed && trimmed !== cleanName && onRename) {
            onRename(mod.path, trimmed)
        }
        setIsRenaming(false)
        setRenameValue('')
    }

    const handleRenameCancel = () => {
        setIsRenaming(false)
        setRenameValue('')
    }

    const handleRenameKeyDown = (e) => {
        e.stopPropagation()
        if (e.key === 'Enter') {
            e.preventDefault()
            handleRenameSubmit()
        } else if (e.key === 'Escape') {
            e.preventDefault()
            handleRenameCancel()
        }
    }

    // Focus and select input when renaming starts
    useEffect(() => {
        if (isRenaming && renameInputRef.current) {
            renameInputRef.current.focus()
            renameInputRef.current.select()
        }
    }, [isRenaming])

    // Watch for external trigger to start renaming (from context menu)
    useEffect(() => {
        if (shouldStartRenaming) {
            setRenameValue(cleanName)
            setIsRenaming(true)
            // Clear the external trigger
            if (onClearRenaming) onClearRenaming()
        }
    }, [shouldStartRenaming])

    // Get hero image for background/badge (only if either icons or bg are enabled)
    const heroImage = (showHeroIcons || showHeroBg) ? getHeroImage(characterName, characterData) : null
    const isCardView = viewMode === 'grid' || viewMode === 'compact'

    // Detect heroes for multi-hero mods - check both characterName and mod_type
    const isMultiHero = (
        (characterName && characterName.toLowerCase().includes('multiple')) ||
        (modDetails?.mod_type && typeof modDetails.mod_type === 'string' && modDetails.mod_type.startsWith('Multiple Heroes'))
    )
    const heroesList = isMultiHero && modDetails?.files && Array.isArray(modDetails.files)
        ? detectHeroesWithData(modDetails.files, characterData)
        : []

    return (
        <div
            className={`mod-card ${selectedMods.has(mod.path) ? 'selected' : ''} ${selectedMod?.path === mod.path ? 'viewing' : ''} ${heroImage && showHeroBg ? 'has-hero-bg' : ''}`}
            onContextMenu={(e) => onContextMenu(e, mod)}
        >
            {/* Blurred hero background for all views */}
            {heroImage && showHeroBg && (
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
                {heroImage && showHeroIcons && (viewMode === 'list' || viewMode === 'list-compact') && (
                    isMultiHero && heroesList.length > 0 ? (
                        <Tooltip
                            title={
                                <div className="heroes-list">
                                    {heroesList.map(name => (
                                        <span key={name} className="tag hero-tag">
                                            {getHeroImage(name, characterData) && (
                                                <img src={getHeroImage(name, characterData)} alt="" />
                                            )}
                                            {name}
                                        </span>
                                    ))}
                                </div>
                            }
                            arrow
                            placement="bottom"
                            slotProps={{
                                tooltip: {
                                    className: 'multi-hero-tooltip'
                                },
                                arrow: {
                                    className: 'multi-hero-arrow'
                                }
                            }}
                        >
                            <img src={heroImage} alt="" className="mod-hero-icon-inline" />
                        </Tooltip>
                    ) : (
                        <img
                            src={heroImage}
                            alt=""
                            className="mod-hero-icon-inline"
                            title={characterName || 'Unknown Hero'}
                        />
                    )
                )}
                {isRenaming ? (
                    <div className="mod-rename-wrapper" onClick={(e) => e.stopPropagation()}>
                        <input
                            ref={renameInputRef}
                            type="text"
                            className="mod-rename-input"
                            value={renameValue}
                            onChange={(e) => setRenameValue(e.target.value)}
                            onKeyDown={handleRenameKeyDown}
                            onBlur={handleRenameSubmit}
                        />
                        {shouldShowSuffix && <span className="mod-rename-suffix">{suffix}</span>}
                    </div>
                ) : (
                    <button
                        type="button"
                        className="mod-name-button"
                        onClick={(e) => {
                            if (e.ctrlKey || e.metaKey) {
                                handleToggleModSelection(mod)
                            } else {
                                onSelect(mod)
                            }
                        }}
                        onDoubleClick={startRename}
                        title={`${rawName} (double-click to rename)`}
                    >
                        <span className="mod-name-text">
                            {cleanName}
                            {shouldShowSuffix && <span className="mod-name-suffix">{suffix}</span>}
                        </span>
                    </button>
                )}
            </div>

            {/* Hero icon + Mod Type Badge + Tags row */}
            {((heroImage && showHeroIcons && isCardView) || (showModType && category) || tags.length > 0) ? (
                <div className="mod-tags-row">
                    {heroImage && showHeroIcons && isCardView && (
                        isMultiHero && heroesList.length > 0 ? (
                            <Tooltip
                                title={
                                    <div className="heroes-list">
                                        {heroesList.map(name => (
                                            <span key={name} className="tag hero-tag">
                                                {getHeroImage(name, characterData) && (
                                                    <img src={getHeroImage(name, characterData)} alt="" />
                                                )}
                                                {name}
                                            </span>
                                        ))}
                                    </div>
                                }
                                arrow
                                placement="bottom"
                                slotProps={{
                                    tooltip: {
                                        className: 'multi-hero-tooltip'
                                    },
                                    arrow: {
                                        className: 'multi-hero-arrow'
                                    }
                                }}
                            >
                                <img src={heroImage} alt="" className="mod-hero-icon-badge" />
                            </Tooltip>
                        ) : (
                            <Tooltip title={characterName || 'Unknown Hero'}>
                                <img src={heroImage} alt="" className="mod-hero-icon-badge" />
                            </Tooltip>
                        )
                    )}
                    {/* Type badge at start for card views */}
                    {showModType && category && isCardView && (
                        <span className={`mod-type-badge category-badge ${category.toLowerCase().replace(/\s+/g, '-')}-badge`}>
                            {category}
                        </span>
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
                    {/* Type badge at end for list view */}
                    {showModType && category && !isCardView && (
                        <span className={`mod-type-badge category-badge ${category.toLowerCase().replace(/\s+/g, '-')}-badge`}>
                            {category}
                        </span>
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
                        disabled={gameRunning}
                    />
                    <div className="mod-switch-wrapper" onClick={(e) => e.stopPropagation()} >
                        <Switch title={mod.enabled ? 'Disable mod' : 'Enable mod'}
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
                    <button
                        className={`hold-delete ${isDeleteHolding ? 'holding' : ''}`}
                        onMouseDown={startDeleteHold}
                        onMouseUp={cancelDeleteHold}
                        onMouseLeave={cancelDeleteHold}
                        onTouchStart={startDeleteHold}
                        onTouchEnd={cancelDeleteHold}
                        aria-label="Hold to delete mod"
                        title="Hold 2s to delete"
                    >
                        <RiDeleteBin2Fill size={18} />
                    </button>
                </div>
            </div>
        </div>
    )
})

/**
 * ModsList Component
 * Renders the grid/list of mods with virtualized rendering for list view
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
    hideSuffix,
    showHeroIcons,
    showHeroBg,
    showModType,
    modDetails,
    characterData,
    onRename,
    renamingModPath,
    onClearRenaming,
    gridRef,
    gameRunning,
    onRenameBlocked,
    onDeleteBlocked
}) {
    return (
        <div className="mods-list-wrapper">
            <div
                key={viewMode}
                ref={gridRef}
                className={`mod-list-grid view-${viewMode} ${mods.length === 0 ? 'empty' : ''}`}
                style={{ overflowY: 'auto', height: '100%' }}
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
                                hideSuffix={hideSuffix}
                                showHeroIcons={showHeroIcons}
                                showHeroBg={showHeroBg}
                                showModType={showModType}
                                characterName={details?.character_name}
                                category={details?.category}
                                viewMode={viewMode}
                                characterData={characterData}
                                onRename={onRename}
                                shouldStartRenaming={renamingModPath === mod.path}
                                onClearRenaming={onClearRenaming}
                                gameRunning={gameRunning}
                                onRenameBlocked={onRenameBlocked}
                                onDeleteBlocked={onDeleteBlocked}
                                modDetails={details}
                            />
                        )
                    })
                )}
            </div>
        </div>
    )
}

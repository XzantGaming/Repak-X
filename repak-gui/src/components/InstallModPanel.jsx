import { useState, useEffect } from 'react'
import Switch from './ui/Switch'
import { FaTag } from "react-icons/fa6"
import './InstallModPanel.css'
import characterData from '../data/character_data.json'

const heroImages = import.meta.glob('../assets/hero/*.png', { eager: true })

const hasCookedAssets = (mod = {}) => {
  if (!mod?.is_dir) return false
  return Boolean(mod.auto_fix_mesh || mod.auto_fix_texture || mod.auto_fix_serialize_size)
}

const isRepakLocked = (mod = {}) => hasCookedAssets(mod)

const buildInitialSettings = (mods = []) => {
  return mods.reduce((acc, mod, idx) => {
    const locked = isRepakLocked(mod)
    const defaultToRepak = mod.is_dir ? !locked : Boolean(mod.auto_to_repak)
    const canApplyPatches = mod.contains_uassets !== false // Default to true if undefined

    // For mods with no uassets, we skip repak (IoStore logic) and likely enforce legacy
    const effectiveToRepak = !canApplyPatches ? false : (locked ? false : defaultToRepak)

    acc[idx] = {
      fixMesh: canApplyPatches ? (mod.auto_fix_mesh || false) : false,
      fixTexture: canApplyPatches ? (mod.auto_fix_texture || false) : false,
      fixSerializeSize: canApplyPatches ? (mod.auto_fix_serialize_size || false) : false,
      toRepak: effectiveToRepak,
      forceLegacy: mod.auto_force_legacy || false,
      compression: 'Oodle',
      usmapPath: '',
      customName: '',
      selectedTags: []
    }
    return acc
  }, {})
}

function parseModType(modType) {
  if (!modType) return { character: null, category: 'Unknown', additional: [] }

  // Extract additional categories
  const bracketMatch = modType.match(/\[(.*?)\]/)
  const additional = bracketMatch ? bracketMatch[1].split(',').map(s => s.trim()) : []

  // Clean base string
  let base = modType.replace(/\[.*?\]/, '').trim()
  let character = null
  let category = base

  // Split Character - Category
  if (base.includes(' - ')) {
    const parts = base.split(' - ')
    if (parts.length >= 2) {
      category = parts.pop()
      character = parts.join(' - ')
    }
  }

  return { character, category, additional }
}

export default function InstallModPanel({ mods, allTags, onCreateTag, onInstall, onCancel }) {
  const [openDropdown, setOpenDropdown] = useState(null)
  const [dropdownPos, setDropdownPos] = useState({ x: 0, y: 0 })
  const [modSettings, setModSettings] = useState(() => buildInitialSettings(mods))

  useEffect(() => {
    setModSettings(buildInitialSettings(mods))
  }, [mods])

  useEffect(() => {
    const handleClickOutside = () => setOpenDropdown(null)
    window.addEventListener('click', handleClickOutside)
    return () => window.removeEventListener('click', handleClickOutside)
  }, [])

  const updateModSetting = (idx, key, value) => {
    if (key === 'toRepak' && isRepakLocked(mods[idx])) {
      return
    }

    // When enabling forceLegacy, clear all patch toggles
    if (key === 'forceLegacy' && value === true) {
      setModSettings(prev => ({
        ...prev,
        [idx]: {
          ...prev[idx],
          [key]: value,
          fixMesh: false,
          fixTexture: false,
          fixSerializeSize: false
        }
      }))
      return
    }

    // Prevent enabling patch toggles when in legacy mode or no uassets
    if (['fixMesh', 'fixTexture', 'fixSerializeSize'].includes(key)) {
      if (modSettings[idx]?.forceLegacy || mods[idx]?.contains_uassets === false) {
        return
      }
    }

    setModSettings(prev => ({
      ...prev,
      [idx]: { ...prev[idx], [key]: value }
    }))
  }

  const handleAddTag = (idx, tag) => {
    if (!tag.trim()) return
    const currentTags = modSettings[idx]?.selectedTags || []
    if (!currentTags.includes(tag.trim())) {
      updateModSetting(idx, 'selectedTags', [...currentTags, tag.trim()])
    }
  }

  const handleRemoveTag = (idx, tagToRemove) => {
    const currentTags = modSettings[idx]?.selectedTags || []
    updateModSetting(idx, 'selectedTags', currentTags.filter(t => t !== tagToRemove))
  }

  const handleInstall = () => {
    // Prepare mods with their settings
    const modsToInstall = mods.map((mod, idx) => ({
      ...mod,
      ...modSettings[idx],
      toRepak: isRepakLocked(mod) ? false : (modSettings[idx]?.toRepak || false),
      forceLegacy: modSettings[idx]?.forceLegacy || false
    }))
    onInstall(modsToInstall)
  }

  return (
    <div className="install-mod-overlay">
      <div className="install-mod-panel">
        <div className="install-header">
          <h2>Install Mods</h2>
          <button className="close-btn" onClick={onCancel}>×</button>
        </div>

        <div className="mods-table-container card-mode">
          {mods.length === 0 ? (
            <div className="install-empty-state">No mods detected in the drop.</div>
          ) : (
            <div className="install-mod-grid">
              {mods.map((mod, idx) => {
                const repakLocked = isRepakLocked(mod)
                const repakTitle = repakLocked
                  ? 'Detected loose assets; repak handled automatically'
                  : (mod.is_dir ? 'Folder contains PAK files; ready to repak' : 'Direct PAK - can repak if needed')
                const { character, category, additional } = parseModType(mod.mod_type)
                const modLabel = mod.is_dir ? 'Folder Drop' : 'PAK File'
                const toggleDefinitions = [
                  {
                    key: 'fixMesh',
                    label: 'Patch Skeletal Meshes',
                    hint: 'Applies fixes to skeletal meshes'
                  },
                  {
                    key: 'fixTexture',
                    label: 'Patch Textures',
                    hint: 'Experimental - Removes mipmaps from textures'
                  },
                  {
                    key: 'fixSerializeSize',
                    label: 'Patch Static Meshes',
                    hint: 'Applies fixes to static mesh serialization sizes'
                  }
                ]

                return (
                  <div className="install-mod-card" key={mod.path || idx}>
                    <div className="install-mod-card__header">
                      <div className="install-mod-card__title">
                        <label className="field-label">Custom Name</label>
                        <div className="mod-name-input-wrapper">
                          <input
                            type="text"
                            placeholder="Insert custom name here"
                            value={modSettings[idx]?.customName || ''}
                            onChange={(e) => updateModSetting(idx, 'customName', e.target.value)}
                            className="mod-name-input"
                          />
                          <span className="mod-name-suffix-hint">_9999999_P</span>
                        </div>
                        <span className="install-mod-card__hint" title={mod.path}>
                          {modSettings[idx]?.customName
                            ? `${modSettings[idx].customName}_9999999_P.pak`
                            : mod.mod_name}
                        </span>
                      </div>
                      <span className={`install-mod-card__pill ${mod.is_dir ? 'pill-folder' : 'pill-pak'}`}>
                        {modLabel}
                      </span>
                    </div>

                    <div className="install-mod-card__badges">
                      {character && (
                        <span className={`character-badge ${character.startsWith('Multiple Heroes') ? 'multi-hero' : ''}`}>
                          {getHeroImage(character) && <img src={getHeroImage(character)} alt="" />}
                          {character}
                        </span>
                      )}
                      <span className={`category-badge ${category.toLowerCase().replace(/\s+/g, '-')}-badge`}>
                        {category}
                      </span>
                      {additional.map(tag => (
                        <span key={tag} className={`additional-badge ${tag.toLowerCase()}-badge`}>
                          {tag}
                        </span>
                      ))}
                      {mod.contains_uassets === false && (
                        <span className="no-uassets-badge" title="This mod contains no UAsset files - patch options disabled">
                          No UAssets
                        </span>
                      )}
                    </div>

                    <div className="install-mod-card__toggles">
                      {toggleDefinitions.map(({ key, label, hint }) => {
                        const isLegacyMode = modSettings[idx]?.forceLegacy || false
                        const canApplyPatches = mod.contains_uassets !== false
                        const isLocked = isLegacyMode || !canApplyPatches

                        let hintText = hint
                        if (isLegacyMode) {
                          hintText = 'Disabled in Legacy PAK mode'
                        } else if (!canApplyPatches) {
                          hintText = 'No UAsset files detected'
                        }

                        return (
                          <Switch
                            key={key}
                            size="sm"
                            color="primary"
                            checked={isLocked ? false : (modSettings[idx]?.[key] || false)}
                            onChange={(value) => updateModSetting(idx, key, value)}
                            isDisabled={isLocked}
                            className={`install-toggle ${isLocked ? 'locked' : ''}`}
                          >
                            <div className="install-toggle__text">
                              <span className="install-toggle__label">{label}</span>
                              <span className="install-toggle__hint">
                                {hintText}
                              </span>
                            </div>
                          </Switch>
                        )
                      })}
                    </div>

                    <div className="install-mod-card__tags">
                      <div className="install-mod-card__row">
                        <span className="field-label">Tags</span>
                        <div className="tags-cell">
                          <div className="tags-list">
                            {(modSettings[idx]?.selectedTags || []).map(tag => (
                              <span key={tag} className="tag">
                                <FaTag />
                                {tag}
                                <button
                                  type="button"
                                  className="tag-remove"
                                  onClick={() => handleRemoveTag(idx, tag)}
                                >
                                  ×
                                </button>
                              </span>
                            ))}
                          </div>
                          <div className="add-tag-wrapper" onClick={e => e.stopPropagation()}>
                            <button
                              className="add-tag-btn"
                              onClick={(e) => {
                                const rect = e.currentTarget.getBoundingClientRect()
                                setDropdownPos({ x: rect.left, y: rect.bottom })
                                setOpenDropdown(openDropdown === idx ? null : idx)
                              }}
                              title="Add Tag"
                            >
                              +
                            </button>
                            {openDropdown === idx && (
                              <div
                                className="tag-dropdown"
                                style={{
                                  position: 'fixed',
                                  top: dropdownPos.y,
                                  left: dropdownPos.x
                                }}
                              >
                                <div className="dropdown-item" onClick={() => {
                                  const tag = prompt('Enter new tag name:')
                                  if (tag && tag.trim()) {
                                    handleAddTag(idx, tag)
                                    if (onCreateTag) onCreateTag(tag)
                                  }
                                  setOpenDropdown(null)
                                }}>
                                  + New Tag...
                                </div>
                                {allTags && allTags.length > 0 && <div className="dropdown-separator" />}
                                {allTags && allTags.map(tag => (
                                  <div key={tag} className="dropdown-item" onClick={() => {
                                    handleAddTag(idx, tag)
                                    setOpenDropdown(null)
                                  }}>
                                    {tag}
                                  </div>
                                ))}
                              </div>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>

                    <div className="install-mod-card__footer">
                      {mod.contains_uassets !== false && (
                        <Switch
                          size="md"
                          color="secondary"
                          checked={modSettings[idx]?.toRepak || false}
                          onChange={(value) => updateModSetting(idx, 'toRepak', value)}
                          isDisabled={repakLocked}
                          className={`install-toggle repak-toggle ${repakLocked ? 'locked' : ''}`}
                          title={repakTitle}
                        >
                          <div className="install-toggle__text">
                            <span className="install-toggle__label">Send to Repak</span>
                            <span className="install-toggle__hint">
                              {repakLocked ? 'Loose assets detected' : 'Repaks the pak into IOStore format'}
                            </span>
                          </div>
                        </Switch>
                      )}

                      <Switch
                        size="md"
                        color="warning"
                        checked={mod.contains_uassets === false ? true : (modSettings[idx]?.forceLegacy || false)}
                        onChange={(value) => {
                          if (mod.contains_uassets === false) return
                          updateModSetting(idx, 'forceLegacy', value)
                        }}
                        isDisabled={mod.contains_uassets === false}
                        className={`install-toggle legacy-toggle ${mod.contains_uassets === false ? 'active locked' : (modSettings[idx]?.forceLegacy ? 'active' : '')}`}
                        title="Use when making Audio/Config mods (mods that don't contain uassets)"
                      >
                        <div className="install-toggle__text">
                          <span className="install-toggle__label">Legacy PAK Format</span>
                          <span className="install-toggle__hint">
                            {mod.contains_uassets === false
                              ? 'Forced for non-UAsset mods'
                              : (modSettings[idx]?.forceLegacy
                                ? 'Skipping IoStore conversion'
                                : 'Use for Audio/Config mods (no uassets)')}
                          </span>
                        </div>
                      </Switch>
                    </div>
                  </div>
                )
              })}
            </div>
          )}
        </div>

        {/* Action Buttons */}
        <div className="install-actions">
          <button onClick={onCancel} className="btn-cancel">
            Cancel
          </button>
          <button onClick={handleInstall} className="btn-install">
            Install {mods.length} Mod(s)
          </button>
        </div>
      </div>
    </div>
  )
}

function getHeroImage(heroName) {
  if (!heroName) return null

  // Check for ID at start (e.g. "1025XXX" -> 1025)
  const idMatch = heroName.match(/^(10\d{2})/)
  if (idMatch) {
    const id = idMatch[1]
    const key = `../assets/hero/${id}.png`
    if (heroImages[key]) return heroImages[key].default
  }

  // Find by name (partial match)
  const char = characterData.find(c => heroName.includes(c.name))
  if (!char) return null

  const key = `../assets/hero/${char.id}.png`
  return heroImages[key]?.default
}

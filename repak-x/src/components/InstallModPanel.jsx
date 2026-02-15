import { useState, useEffect, useMemo } from 'react'
import { motion } from 'framer-motion'
import Switch from './ui/Switch'
import { FaTag } from "react-icons/fa6"
import { VscFolder, VscFolderOpened, VscChevronRight, VscChevronDown, VscNewFolder } from 'react-icons/vsc'
import { MdCreateNewFolder } from 'react-icons/md'
import './InstallModPanel.css'
import characterData from '../data/character_data.json'

const heroImages = import.meta.glob('../assets/hero/*.png', { eager: true })

// Folder tree helper functions
const buildTree = (folders) => {
  const root = { id: 'root', name: 'root', children: {}, isVirtual: true }
  const sortedFolders = [...folders].sort((a, b) => a.name.localeCompare(b.name))

  sortedFolders.forEach(folder => {
    const parts = folder.id.split(/[/\\]/)
    let current = root

    parts.forEach((part, index) => {
      if (!current.children[part]) {
        current.children[part] = {
          name: part,
          children: {},
          isVirtual: true,
          fullPath: parts.slice(0, index + 1).join('/')
        }
      }
      current = current.children[part]

      if (index === parts.length - 1) {
        current.id = folder.id
        current.isVirtual = false
        current.originalName = folder.name
      }
    })
  })

  return root
}

const convertToArray = (node) => {
  if (!node.children) return []
  const children = Object.values(node.children).map(child => ({
    ...child,
    children: convertToArray(child)
  }))
  children.sort((a, b) => a.name.localeCompare(b.name))
  return children
}

// Folder node component for the tree
const FolderNode = ({ node, selectedFolderId, onSelect, depth = 0 }) => {
  const [isOpen, setIsOpen] = useState(false)
  const hasChildren = node.children && node.children.length > 0
  const isSelected = selectedFolderId === node.id

  const handleClick = (e) => {
    e.stopPropagation()
    if (!node.isVirtual) {
      onSelect(node.id)
    } else {
      setIsOpen(!isOpen)
    }
  }

  return (
    <div className="imp-folder-node">
      <div
        className={`imp-folder-item ${isSelected ? 'selected' : ''} ${node.isVirtual ? 'virtual' : ''}`}
        onClick={handleClick}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
      >
        <span className="folder-toggle" onClick={(e) => { e.stopPropagation(); setIsOpen(!isOpen) }}>
          {hasChildren ? (isOpen ? <VscChevronDown /> : <VscChevronRight />) : <span style={{ width: 16 }} />}
        </span>
        <span className="folder-icon">
          {isSelected || isOpen ? <VscFolderOpened /> : <VscFolder />}
        </span>
        <span className="folder-name">{node.name}</span>
      </div>

      {hasChildren && isOpen && (
        <div className="imp-folder-children">
          {node.children.map(child => (
            <FolderNode
              key={child.fullPath || child.id}
              node={child}
              selectedFolderId={selectedFolderId}
              onSelect={onSelect}
              depth={depth + 1}
            />
          ))}
        </div>
      )}
    </div>
  )
}

const hasCookedAssets = (mod = {}) => {
  if (!mod?.is_dir) return false
  return Boolean(mod.auto_fix_texture || mod.auto_fix_serialize_size)
}

const isRepakLocked = (mod = {}) => mod.is_dir || hasCookedAssets(mod)

const buildInitialSettings = (mods = []) => {
  return mods.reduce((acc, mod, idx) => {
    const locked = isRepakLocked(mod)
    const defaultToRepak = mod.is_dir ? !locked : Boolean(mod.auto_to_repak)
    const canApplyPatches = mod.contains_uassets !== false // Default to true if undefined

    // For mods with no uassets, we skip repak (IoStore logic) and likely enforce legacy
    const effectiveToRepak = !canApplyPatches ? false : (locked ? false : defaultToRepak)

    acc[idx] = {
      fixTexture: canApplyPatches ? (mod.auto_fix_texture || false) : false,
      fixSerializeSize: canApplyPatches ? (mod.auto_fix_serialize_size || false) : false,
      toRepak: effectiveToRepak,
      forceLegacy: mod.contains_uassets === false ? true : (mod.auto_force_legacy || false),
      compression: 'Oodle',
      usmapPath: '',
      customName: '',
      selectedTags: [],
      installSubfolder: null // Per-mod install destination
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

export default function InstallModPanel({ mods, allTags, folders = [], onCreateTag, onCreateFolder, onInstall, onCancel, onNewTag, onNewFolder }) {
  const [openDropdown, setOpenDropdown] = useState(null)
  const [dropdownPos, setDropdownPos] = useState({ x: 0, y: 0 })
  const [modSettings, setModSettings] = useState(() => buildInitialSettings(mods))
  // Removed global selectedFolderId since we now track it per-mod in modSettings
  const [isCreatingFolder, setIsCreatingFolder] = useState(false)

  // Folder tree data
  const rootFolder = useMemo(() => folders.find(f => f.is_root), [folders])
  const subfolders = useMemo(() => folders.filter(f => !f.is_root), [folders])
  const treeData = useMemo(() => {
    const root = buildTree(subfolders)
    return convertToArray(root)
  }, [subfolders])

  useEffect(() => {
    console.log('[InstallModPanel] Received mods:', mods.length, mods)
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
          fixTexture: false,
          fixSerializeSize: false
        }
      }))
      return
    }

    // Prevent enabling patch toggles when in legacy mode or no uassets
    if (['fixTexture', 'fixSerializeSize'].includes(key)) {
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
      forceLegacy: modSettings[idx]?.forceLegacy || false,
      installSubfolder: modSettings[idx]?.installSubfolder || ''
    }))
    onInstall(modsToInstall)
  }

  const handleNewFolder = (targetModIdx) => {
    if (onNewFolder) {
      onNewFolder(async (name) => {
        if (!name || !name.trim()) return
        setIsCreatingFolder(true)
        try {
          if (onCreateFolder) {
            const newFolderId = await onCreateFolder(name.trim())
            if (newFolderId && typeof targetModIdx === 'number') {
              updateModSetting(targetModIdx, 'installSubfolder', newFolderId)
            }
          }
        } catch (err) {
          console.error('Failed to create folder:', err)
        } finally {
          setIsCreatingFolder(false)
        }
      })
    }
  }

  return (
    <div className="install-mod-overlay">
      <motion.div
        className="install-mod-panel"
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.15 }}
      >
        <div className="install-header">
          <h2>Install Mods</h2>
          <button className="close-btn" onClick={onCancel}>×</button>
        </div>

        {/* Mod Cards */}
        <div className="imp-mods-section">
          {mods.length === 0 ? (
            <div className="install-empty-state">No mods detected in the drop.</div>
          ) : (
            <div className="install-mod-grid">
              {mods.map((mod, idx) => {
                const repakLocked = isRepakLocked(mod)
                const repakTitle = repakLocked
                  ? (mod.is_dir ? 'Folder drops cannot be repaked' : 'Detected loose assets; repak handled automatically')
                  : 'Direct PAK - can repak if needed'
                const { character, category, additional } = parseModType(mod.mod_type)
                const modLabel = mod.is_dir ? 'Folder Drop' : 'PAK File'
                return (
                  <div className="install-mod-card" key={mod.path || idx}>
                    {/* Left: Mod Options */}
                    <div className="install-mod-card__left">
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
                                    onNewTag((tag) => {
                                      if (tag && tag.trim()) {
                                        handleAddTag(idx, tag)
                                        if (onCreateTag) onCreateTag(tag)
                                      }
                                    })
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
                                {repakLocked ? (mod.is_dir ? 'Not available for folder drops' : 'Loose assets detected') : 'Repaks the pak into IOStore format'}
                              </span>
                            </div>
                          </Switch>
                        )}
                      </div>
                    </div>

                    {/* Divider */}
                    {folders.length > 0 && <div className="install-mod-card__divider" />}

                    {/* Right: Folder Picker (inside card) */}
                    {folders.length > 0 && (
                      <div className="install-mod-card__right">
                        <div className="imp-section-header">
                          <MdCreateNewFolder />
                          <span>Install to</span>
                          <button
                            className="imp-btn-new-folder"
                            onClick={() => handleNewFolder(idx)}
                            disabled={isCreatingFolder}
                            title="Create new folder"
                          >
                            <VscNewFolder />
                          </button>
                        </div>

                        <div className="imp-folder-tree-container">
                          {/* Root folder */}
                          {rootFolder && (
                            <div
                              className={`imp-folder-item root-item ${modSettings[idx]?.installSubfolder === rootFolder.id || !modSettings[idx]?.installSubfolder ? 'selected' : ''}`}
                              onClick={() => updateModSetting(idx, 'installSubfolder', rootFolder.id)}
                            >
                              <span className="folder-icon"><VscFolderOpened /></span>
                              <span className="folder-name">{rootFolder.name}</span>
                            </div>
                          )}

                          {/* Subfolders */}
                          <div className="imp-folder-tree">
                            {treeData.map(node => (
                              <FolderNode
                                key={node.fullPath || node.id}
                                node={node}
                                selectedFolderId={modSettings[idx]?.installSubfolder}
                                onSelect={(newId) => updateModSetting(idx, 'installSubfolder', newId)}
                              />
                            ))}
                          </div>
                        </div>
                      </div>
                    )}
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
      </motion.div>
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

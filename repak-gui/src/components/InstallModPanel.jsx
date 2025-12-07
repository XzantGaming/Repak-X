import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './InstallModPanel.css'

const hasCookedAssets = (mod = {}) => {
  if (!mod?.is_dir) return false
  return Boolean(mod.auto_fix_mesh || mod.auto_fix_texture || mod.auto_fix_serialize_size)
}

const isRepakLocked = (mod = {}) => hasCookedAssets(mod)

const buildInitialSettings = (mods = []) => {
  return mods.reduce((acc, mod, idx) => {
    const locked = isRepakLocked(mod)
    const defaultToRepak = mod.is_dir ? !locked : Boolean(mod.auto_to_repak)

    acc[idx] = {
      fixMesh: mod.auto_fix_mesh || false,
      fixTexture: mod.auto_fix_texture || false,
      fixSerializeSize: mod.auto_fix_serialize_size || false,
      toRepak: locked ? false : defaultToRepak,
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
  const [modDetailsMap, setModDetailsMap] = useState({})
  const [detailsLoading, setDetailsLoading] = useState(false)

  useEffect(() => {
    setModSettings(buildInitialSettings(mods))
  }, [mods])

  useEffect(() => {
    let cancelled = false

    const loadDetails = async () => {
      if (!Array.isArray(mods) || mods.length === 0) {
        setModDetailsMap({})
        return
      }

      const uniquePaths = Array.from(new Set(
        mods
          .map(mod => mod?.path)
          .filter(Boolean)
      ))

      if (uniquePaths.length === 0) {
        setModDetailsMap({})
        return
      }

      setDetailsLoading(true)

      const results = await Promise.all(uniquePaths.map(async (path) => {
        try {
          const detail = await invoke('get_mod_details', { modPath: path })
          return { path, detail }
        } catch (error) {
          console.warn('InstallModPanel: failed to load mod details for', path, error)
          return { path, detail: null }
        }
      }))

      if (cancelled) return

      const nextMap = {}
      results.forEach(({ path, detail }) => {
        if (detail) {
          nextMap[path] = detail
        }
      })

      setModDetailsMap(nextMap)
      setDetailsLoading(false)
    }

    loadDetails()

    return () => {
      cancelled = true
    }
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
      toRepak: isRepakLocked(mod) ? false : (modSettings[idx]?.toRepak || false)
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

        {/* Mods Table */}
        <div className="mods-table-container">
          {detailsLoading && (
            <div className="mods-table-hint">
              Analyzing dropped content for hero/type details...
            </div>
          )}
          <table className="mods-table">
            <thead>
              <tr>
                <th>Mod Name</th>
                <th>Type</th>
                <th>Tags</th>
                <th>Fix Mesh</th>
                <th>Fix Texture</th>
                <th>Fix SerializeSize</th>
                <th>To Repak</th>
              </tr>
            </thead>
            <tbody>
              {mods.map((mod, idx) => {
                const details = modDetailsMap[mod.path]
                const typeSource = details?.mod_type || mod.mod_type
                const parsedType = parseModType(typeSource)
                const characterLabel = details?.character_name || parsedType.character
                const categoryLabel = details?.category || parsedType.category || 'Unknown'
                const additionalBadges = (details?.additional_categories && details.additional_categories.length > 0)
                  ? details.additional_categories
                  : parsedType.additional
                const namePlaceholder = details?.mod_name || mod.mod_name
                const repakLocked = isRepakLocked(mod)
                const repakTitle = repakLocked
                  ? 'Detected loose assets; repak handled automatically'
                  : (mod.is_dir ? 'Folder contains PAK files; ready to repak' : 'Direct PAK - can repak if needed')

                return (
                <tr key={idx}>
                  <td>
                    <input
                      type="text"
                      placeholder={namePlaceholder}
                      value={modSettings[idx]?.customName || ''}
                      onChange={(e) => updateModSetting(idx, 'customName', e.target.value)}
                      className="mod-name-input"
                    />
                  </td>
                  <td>
                    <div className="mod-badges" style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                      <>
                        {characterLabel && (
                          <span className={`character-badge ${characterLabel.startsWith('Multiple Heroes') ? 'multi-hero' : ''}`}>
                            {characterLabel}
                          </span>
                        )}
                        <span className={`category-badge ${(categoryLabel || 'unknown').toLowerCase().replace(/\s+/g, '-')}-badge`}>
                          {categoryLabel}
                        </span>
                        {additionalBadges && additionalBadges.map(tag => (
                          <span key={tag} className={`additional-badge ${tag.toLowerCase()}-badge`}>
                            {tag}
                          </span>
                        ))}
                      </>
                    </div>
                  </td>
                  <td>
                    <div className="tags-cell">
                      <div className="tags-list">
                        {(modSettings[idx]?.selectedTags || []).map(tag => (
                          <span key={tag} className="tag">
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
                  </td>
                  <td>
                    <input
                      type="checkbox"
                      checked={modSettings[idx]?.fixMesh || false}
                      onChange={(e) => updateModSetting(idx, 'fixMesh', e.target.checked)}
                    />
                  </td>
                  <td>
                    <input
                      type="checkbox"
                      checked={modSettings[idx]?.fixTexture || false}
                      onChange={(e) => updateModSetting(idx, 'fixTexture', e.target.checked)}
                    />
                  </td>
                  <td>
                    <input
                      type="checkbox"
                      checked={modSettings[idx]?.fixSerializeSize || false}
                      onChange={(e) => updateModSetting(idx, 'fixSerializeSize', e.target.checked)}
                    />
                  </td>
                  <td style={{ minWidth: '120px' }}>
                    <input
                      type="checkbox"
                      checked={modSettings[idx]?.toRepak || false}
                      onChange={(e) => updateModSetting(idx, 'toRepak', e.target.checked)}
                      disabled={repakLocked}
                      style={{ 
                        cursor: repakLocked ? 'not-allowed' : 'pointer',
                        opacity: repakLocked ? 0.5 : 1
                      }}
                      title={repakTitle}
                    />
                  </td>
                </tr>
              )})}
            </tbody>
          </table>
        </div>

        {/* Action Buttons */}
        <div className="install-actions">
          <button onClick={handleInstall} className="btn-install">
            Install {mods.length} Mod(s)
          </button>
          <button onClick={onCancel} className="btn-cancel">
            Cancel
          </button>
        </div>
      </div>
    </div>
  )
}

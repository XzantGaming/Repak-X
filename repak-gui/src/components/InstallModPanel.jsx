import { useState, useEffect } from 'react'
import './InstallModPanel.css'

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
  const [modSettings, setModSettings] = useState(
    mods.reduce((acc, mod, idx) => {
      acc[idx] = {
        fixMesh: mod.auto_fix_mesh || false,
        fixTexture: mod.auto_fix_texture || false,
        fixSerializeSize: mod.auto_fix_serialize_size || false,
        toRepak: mod.auto_to_repak || false,
        compression: 'Oodle',
        usmapPath: '',
        customName: '',
        selectedTags: []
      }
      return acc
    }, {})
  )

  useEffect(() => {
    const handleClickOutside = () => setOpenDropdown(null)
    window.addEventListener('click', handleClickOutside)
    return () => window.removeEventListener('click', handleClickOutside)
  }, [])

  const updateModSetting = (idx, key, value) => {
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
      ...modSettings[idx]
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
              {mods.map((mod, idx) => (
                <tr key={idx}>
                  <td>
                    <input
                      type="text"
                      placeholder={mod.mod_name}
                      value={modSettings[idx]?.customName || ''}
                      onChange={(e) => updateModSetting(idx, 'customName', e.target.value)}
                      className="mod-name-input"
                    />
                  </td>
                  <td>
                    <div className="mod-badges" style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                      {(() => {
                        const { character, category, additional } = parseModType(mod.mod_type)
                        return (
                          <>
                            {character && (
                              <span className={`character-badge ${character.startsWith('Multiple Heroes') ? 'multi-hero' : ''}`}>
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
                          </>
                        )
                      })()}
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
                  <td>
                    <input
                      type="checkbox"
                      checked={modSettings[idx]?.toRepak || false}
                      onChange={(e) => updateModSetting(idx, 'toRepak', e.target.checked)}
                    />
                  </td>
                </tr>
              ))}
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

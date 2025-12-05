import { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Tooltip } from '@mui/material'
import characterData from '../data/character_data.json'
import './ModDetailsPanel.css'

export default function ModDetailsPanel({ mod, initialDetails, onClose }) {
  const [details, setDetails] = useState(initialDetails || null)
  const [loading, setLoading] = useState(!initialDetails)
  const [error, setError] = useState(null)

  useEffect(() => {
    if (mod) {
      if (initialDetails && initialDetails.mod_path === mod.path) {
        setDetails(initialDetails)
        setLoading(false)
      } else if (initialDetails && !initialDetails.mod_path) {
        // Fallback if mod_path isn't in details (older backend version?)
        setDetails(initialDetails)
        setLoading(false)
      } else {
        loadModDetails()
      }
    }
  }, [mod, initialDetails])

  const heroesList = useMemo(() => {
    if (details && details.files) {
      return detectHeroes(details.files)
    }
    return []
  }, [details])

  const additionalBadges = useMemo(() => {
    if (!details) return []
    
    // Use explicit field if available and not empty
    if (details.additional_categories && details.additional_categories.length > 0) {
      return details.additional_categories
    }
    
    // Fallback: Parse from mod_type string
    // Format: "Name - Category [Add1, Add2]"
    if (details.mod_type) {
      const match = details.mod_type.match(/\[(.*?)\]/)
      if (match && match[1]) {
        return match[1].split(',').map(s => s.trim())
      }
    }
    
    return []
  }, [details])

  const loadModDetails = async () => {
    try {
      setLoading(true)
      setError(null)
      console.log('Loading details for:', mod.path)
      const modDetails = await invoke('get_mod_details', { modPath: mod.path })
      console.log('Received details:', modDetails)
      setDetails(modDetails)
    } catch (err) {
      console.error('Failed to load mod details:', err)
      setError(err.toString())
    } finally {
      setLoading(false)
    }
  }

  if (!mod) return null

  return (
    <div className="details-panel">
      <div className="details-header">
        <h2>{mod.custom_name || mod.path.split('\\').pop()}</h2>
      </div>
      
      <div className="details-body">
        {loading ? (
          <div className="loading-state">Loading mod details...</div>
        ) : error ? (
          <div className="error-state">
            <h3>Error Loading Details</h3>
            <p>{error}</p>
            <p className="error-path">Mod path: {mod.path}</p>
          </div>
        ) : details ? (
          <>
            <div className="detail-section">
              <h3>Type & Character</h3>
              <div className="badges-container">
                {details.character_name && (
                  <div className="character-badge" title="Character">
                    {details.character_name}
                  </div>
                )}
                {details.mod_type.startsWith('Multiple Heroes') && (
                  <Tooltip 
                    title={
                      <div className="heroes-list">
                        {heroesList.map(name => (
                          <span key={name} className="tag hero-tag">
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
                    <div className="character-badge multi-hero">
                      {details.mod_type.split(' - ')[0]}
                    </div>
                  </Tooltip>
                )}
                <div className={`category-badge ${details.category ? details.category.toLowerCase().replace(/\s+/g, '-') + '-badge' : ''}`} title="Mod Type">
                  {details.category || 'Unknown'}
                </div>
                {/* Render additional categories (Blueprint, Text) */}
                {additionalBadges.map(cat => (
                  <div 
                    key={cat}
                    className={`additional-badge ${cat.toLowerCase()}-badge`}
                    title={`Contains ${cat}`}
                  >
                    {cat}
                  </div>
                ))}
                {details.is_iostore && (
                  <div className="iostore-badge">IoStore Package</div>
                )}
              </div>
            </div>

            <div className="detail-section">
              <h3>Information</h3>
              <div className="detail-item">
                <span className="detail-label">Files:</span>
                <span className="detail-value">{details.file_count}</span>
              </div>
              <div className="detail-item">
                <span className="detail-label">Size:</span>
                <span className="detail-value">{formatFileSize(details.total_size)}</span>
              </div>
              <div className="detail-item">
                <span className="detail-label">Enabled:</span>
                <span className="detail-value">{mod.enabled ? 'Yes' : 'No'}</span>
              </div>
              {mod.folder_id && (
                <div className="detail-item">
                  <span className="detail-label">Folder:</span>
                  <span className="detail-value">{mod.folder_id}</span>
                </div>
              )}
            </div>
            
            {mod.custom_tags && mod.custom_tags.length > 0 && (
              <div className="detail-section">
                <h3>Tags</h3>
                <div className="tags-list">
                  {mod.custom_tags.map((tag, idx) => (
                    <span key={idx} className="tag">{tag}</span>
                  ))}
                </div>
              </div>
            )}

            <div className="detail-section">
              <h3>File Contents ({details.file_count} files)</h3>
              <div className="file-list">
                {details.files.slice(0, 100).map((file, idx) => (
                  <div key={idx} className="file-item">
                    {getFileIcon(file)} {file}
                  </div>
                ))}
                {details.files.length > 100 && (
                  <div className="file-item-more">
                    ... and {details.files.length - 100} more files
                  </div>
                )}
              </div>
            </div>
          </>
        ) : null}
      </div>
    </div>
  )
}

function getFileIcon(filename) {
  if (filename.endsWith('.uasset')) return '📦'
  if (filename.endsWith('.uexp')) return '📄'
  if (filename.endsWith('.umap')) return '🗺️'
  if (filename.endsWith('.wem') || filename.endsWith('.bnk')) return '🔊'
  if (filename.endsWith('.png') || filename.endsWith('.jpg')) return '🖼️'
  return '📄'
}

function formatFileSize(bytes) {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}

function detectHeroes(files) {
  const heroIds = new Set()
  
  // Regex patterns matching backend logic
  const pathRegex = /(?:Characters|Hero_ST|Hero)\/(\d{4})/
  const filenameRegex = /[_/](10[1-6]\d)(\d{3})/
  
  files.forEach(file => {
    // Check path
    const pathMatch = file.match(pathRegex)
    if (pathMatch) {
      heroIds.add(pathMatch[1])
    }
    
    // Check filename
    const filenameMatch = file.match(filenameRegex)
    if (filenameMatch) {
      heroIds.add(filenameMatch[1])
    }
  })
  
  // Map IDs to names
  const heroNames = new Set()
  heroIds.forEach(id => {
    const char = characterData.find(c => c.id === id)
    if (char) {
      heroNames.add(char.name)
    }
  })
  
  return Array.from(heroNames).sort()
}

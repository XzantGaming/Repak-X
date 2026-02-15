import { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Tooltip } from '@mui/material'
import { FaTag, FaExchangeAlt } from "react-icons/fa"
import FileTree from './FileTree'
import { formatFileSize } from '../utils/format'
import { detectHeroesWithData } from '../utils/heroes'
import './ModDetailsPanel.css'

const heroImages = import.meta.glob('../assets/hero/*.png', { eager: true })

export default function ModDetailsPanel({ mod, initialDetails, onClose, characterData = [], onUpdateMod }) {
  const [details, setDetails] = useState(initialDetails || null)
  const [loading, setLoading] = useState(!initialDetails)
  const [error, setError] = useState(null)

  useEffect(() => {
    let cancelled = false

    const loadDetails = async () => {
      if (!mod) return

      setError(null)

      // Check if we already have details for this mod
      if (initialDetails && (initialDetails.mod_path === mod.path || !initialDetails.mod_path)) {
        setDetails(initialDetails)
        setLoading(false)
        return
      }

      // Need to fetch details
      try {
        setLoading(true)
        setError(null)
        console.log('Loading details for:', mod.path)
        const modDetails = await invoke('get_mod_details', {
          modPath: mod.path
        })

        // Check if this request is still relevant
        if (cancelled) {
          console.log('Request cancelled, ignoring result for:', mod.path)
          return
        }

        console.log('Received details:', modDetails)
        setDetails(modDetails)
      } catch (err) {
        if (cancelled) return
        console.error('Failed to load mod details:', err)
        setError(err.toString())
      } finally {
        if (!cancelled) {
          setLoading(false)
        }
      }
    }

    loadDetails()

    // Cleanup: cancel pending request when mod changes
    return () => {
      cancelled = true
    }
  }, [mod, initialDetails])

  const heroesList = useMemo(() => {
    if (details && details.files && characterData.length > 0) {
      return detectHeroesWithData(details.files, characterData)
    }
    return []
  }, [details, characterData])

  const additionalBadges = useMemo(() => {
    if (!details) return []

    const badges = new Set()

    // 1. Use explicit field if available
    if (details.additional_categories && Array.isArray(details.additional_categories)) {
      details.additional_categories.forEach(c => badges.add(c))
    }

    // 2. Fallback: Parse from mod_type string
    // Format: "Name - Category [Add1, Add2]"
    if (details.mod_type) {
      const match = details.mod_type.match(/\[(.*?)\]/)
      if (match && match[1]) {
        match[1].split(',').forEach(s => badges.add(s.trim()))
      }
    }

    // 3. Explicit check from backend flag (fixing issue where it might be missing from string)
    if (details.has_blueprint && details.category !== 'Blueprint') {
      badges.add('Blueprint')
    }

    return Array.from(badges)
  }, [details])



  if (!mod) return null

  const rawName = mod.custom_name || mod.path.split('\\').pop()
  const nameWithoutExt = rawName.replace(/\.[^/.]+$/, '')
  const suffixMatch = nameWithoutExt.match(/(_\d+_P)$/i)
  const cleanName = suffixMatch ? nameWithoutExt.slice(0, -suffixMatch[0].length) : nameWithoutExt

  return (
    <div className="details-panel">
      <div className="details-header">
        <h2>{cleanName}</h2>
        {onUpdateMod && (
          <button
            className="header-action-btn"
            onClick={onUpdateMod}
            title="Update/Replace Mod File"
            style={{ marginLeft: 'auto' }}
          >
            <FaExchangeAlt /> Update
          </button>
        )}
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
                    {getHeroImage(details.character_name, characterData) && (
                      <img src={getHeroImage(details.character_name, characterData)} alt="" />
                    )}
                    {details.character_name}
                  </div>
                )}
                {details.mod_type.startsWith('Multiple Heroes') && (
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
                {/* No UAssets badge - show if mod has no .uasset files (but not for IoStore bundles) */}
                {!details.is_iostore && details.files && details.files.length > 0 && !details.files.some(f => f.toLowerCase().endsWith('.uasset')) && (
                  <div className="no-uassets-badge" title="This mod contains no UAsset files">No UAssets</div>
                )}
              </div>
            </div>

            <div className="detail-section">
              <h3>Information</h3>
              <div className="detail-item">
                <span className="detail-label">Assets Count:</span>
                <span className="detail-value">{details.file_count}</span>
              </div>
              <div className="detail-item">
                <span className="detail-label">Size:</span>
                <span className="detail-value">{formatFileSize(details.total_size)}</span>
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
                    <span key={idx} className="tag">
                      <FaTag />
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            )}

            <div className="detail-section">
              <div className="detail-section-header">
                <h3>File Contents ({details.file_count} files)</h3>
                <button
                  className="copy-paths-btn"
                  onClick={() => {
                    const allPaths = details.files
                      .map(p => p.replace(/^\/Game\//i, ''))
                      .join('\n')
                    navigator.clipboard.writeText(allPaths).then(() => {
                      // Show feedback
                      const btn = document.querySelector('.copy-paths-btn')
                      if (btn) {
                        const original = btn.textContent
                        btn.textContent = 'Copied!'
                        setTimeout(() => btn.textContent = original, 1500)
                      }
                    })
                  }}
                  title="Copy all file paths to clipboard"
                >
                  Copy All Paths
                </button>
              </div>
              <div className="file-list-container" style={{ border: '1px solid var(--panel-border)', borderRadius: '4px', background: 'var(--bg-darker)' }}>
                <FileTree files={details.files} />
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

function getHeroImage(heroName, characterData) {
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

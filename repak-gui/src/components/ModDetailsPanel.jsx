import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'

export default function ModDetailsPanel({ mod, onClose }) {
  const [details, setDetails] = useState(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  useEffect(() => {
    if (mod) {
      loadModDetails()
    }
  }, [mod])

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
            <p style={{fontSize: '0.9em', color: '#999'}}>Mod path: {mod.path}</p>
          </div>
        ) : details ? (
          <>
            <div className="detail-section">
              <h3>Type & Character</h3>
              <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                {details.character_name && (
                  <div className="character-badge" title="Character">
                    {details.character_name}
                  </div>
                )}
                <div className="category-badge" title="Mod Type">
                  {details.category || 'Unknown'}
                </div>
                {details.is_iostore && (
                  <div className="iostore-badge">IoStore Package</div>
                )}
              </div>
              <div style={{ marginTop: '0.5rem', opacity: 0.7, fontSize: '0.9em' }}>
                Full: {details.mod_type}
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

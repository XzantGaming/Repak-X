import { useState } from 'react'

function parseModType(modType) {
  if (!modType) return { category: 'Unknown', additional: [] }

  const bracketMatch = modType.match(/\[(.*?)\]/)
  const additional = bracketMatch ? bracketMatch[1].split(',').map(s => s.trim()) : []
  let category = modType.replace(/\[.*?\]/, '').trim()

  return { category, additional }
}

export default function ModInstallDialog({ mods, onInstall, onCancel }) {
  const [modConfigs, setModConfigs] = useState(
    mods.map(mod => ({
      ...mod,
      repak: !mod.is_dir,
      fix_mesh: false,
      fix_textures: false,
      fix_serialsize_header: false,
      usmap_path: '',
      mount_point: '../../../',
      path_hash_seed: '00000000',
      compression: 'Oodle',
      custom_tags: [],
      enabled: true
    }))
  )

  const handleConfigChange = (index, field, value) => {
    const updated = [...modConfigs]
    updated[index] = { ...updated[index], [field]: value }
    setModConfigs(updated)
  }

  const handleInstall = () => {
    onInstall(modConfigs.filter(m => m.enabled))
  }

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Install Mods</h2>
          <button className="modal-close" onClick={onCancel}>×</button>
        </div>

        <div className="modal-body">
          <div className="mods-to-install">
            {modConfigs.map((mod, idx) => (
              <div key={idx} className="install-mod-item">
                <div className="install-mod-header">
                  <input
                    type="checkbox"
                    checked={mod.enabled}
                    onChange={(e) => handleConfigChange(idx, 'enabled', e.target.checked)}
                  />
                  <div style={{ flex: 1 }}>
                    <h3>{mod.mod_name}</h3>
                    <div className="mod-badges" style={{ display: 'flex', gap: '0.5rem', marginTop: '0.25rem' }}>
                      {(() => {
                        const { category, additional } = parseModType(mod.mod_type)
                        return (
                          <>
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
                  </div>
                </div>

                {mod.enabled && (
                  <div className="install-mod-options">
                    <div className="option-group">
                      <label>
                        <input
                          type="checkbox"
                          checked={mod.repak}
                          onChange={(e) => handleConfigChange(idx, 'repak', e.target.checked)}
                          disabled={!mod.is_dir}
                        />
                        Repack to PAK
                      </label>

                      <label>
                        <input
                          type="checkbox"
                          checked={mod.fix_mesh}
                          onChange={(e) => handleConfigChange(idx, 'fix_mesh', e.target.checked)}
                        />
                        Fix Mesh Files
                      </label>

                      <label>
                        <input
                          type="checkbox"
                          checked={mod.fix_textures}
                          onChange={(e) => handleConfigChange(idx, 'fix_textures', e.target.checked)}
                        />
                        Fix Textures (NoMipmaps)
                      </label>

                      <label style={{ opacity: 0.5, cursor: 'not-allowed' }} title="Temporarily unavailable">
                        <input
                          type="checkbox"
                          checked={false}
                          disabled
                        />
                        Fix Static Mesh SerializeSize (Unavailable)
                      </label>
                    </div>

                    <div className="option-group">
                      <label>
                        Mount Point:
                        <input
                          type="text"
                          value={mod.mount_point}
                          onChange={(e) => handleConfigChange(idx, 'mount_point', e.target.value)}
                          placeholder="../../../"
                        />
                      </label>

                      <label>
                        Path Hash Seed:
                        <input
                          type="text"
                          value={mod.path_hash_seed}
                          onChange={(e) => handleConfigChange(idx, 'path_hash_seed', e.target.value)}
                          placeholder="00000000"
                        />
                      </label>

                      <label>
                        Compression:
                        <select
                          value={mod.compression}
                          onChange={(e) => handleConfigChange(idx, 'compression', e.target.value)}
                        >
                          <option value="Oodle">Oodle</option>
                          <option value="Zlib">Zlib</option>
                          <option value="Gzip">Gzip</option>
                          <option value="Zstd">Zstd</option>
                          <option value="LZ4">LZ4</option>
                        </select>
                      </label>

                      <label>
                        USmap File:
                        <input
                          type="text"
                          value={mod.usmap_path}
                          onChange={(e) => handleConfigChange(idx, 'usmap_path', e.target.value)}
                          placeholder="Leave empty to use global USmap"
                        />
                      </label>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="modal-footer">
          <button onClick={onCancel} className="btn-secondary">
            Cancel
          </button>
          <button onClick={handleInstall} className="btn-primary">
            Install {modConfigs.filter(m => m.enabled).length} Mod(s)
          </button>
        </div>
      </div>
    </div>
  )
}

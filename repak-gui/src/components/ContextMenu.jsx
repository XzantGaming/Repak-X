import React, { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './ContextMenu.css'

const ContextMenu = ({ x, y, mod, folder, onClose, onAssignTag, onMoveTo, onCreateFolder, folders, onDelete, onToggle, onRename, allTags }) => {
  const [isDeleting, setIsDeleting] = useState(false)
  const [showRenamePanel, setShowRenamePanel] = useState(false)
  const [renameValue, setRenameValue] = useState('')
  const deleteTimeoutRef = useRef(null)
  const renameInputRef = useRef(null)

  useEffect(() => {
    const handleClickOutside = (e) => {
      // Don't close if clicking inside rename panel
      if (showRenamePanel && e.target.closest('.rename-panel')) {
        return
      }
      onClose()
    }
    window.addEventListener('click', handleClickOutside)
    return () => window.removeEventListener('click', handleClickOutside)
  }, [onClose, showRenamePanel])

  useEffect(() => {
    return () => {
      if (deleteTimeoutRef.current) clearTimeout(deleteTimeoutRef.current)
    }
  }, [])

  // Focus input when rename panel opens
  useEffect(() => {
    if (showRenamePanel && renameInputRef.current) {
      renameInputRef.current.focus()
      renameInputRef.current.select()
    }
  }, [showRenamePanel])

  const handleDeleteDown = (e) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDeleting(true)
    deleteTimeoutRef.current = setTimeout(() => {
      onDelete()
      onClose()
    }, 2000)
  }

  const handleDeleteUp = (e) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDeleting(false)
    if (deleteTimeoutRef.current) clearTimeout(deleteTimeoutRef.current)
  }

  // Extract filename parts (base name without suffix, suffix, extension)
  const getFilenameParts = () => {
    if (!mod) return { baseName: '', suffix: '', extension: '.pak' }

    const filename = mod.path.split('\\').pop()
    const extension = filename.match(/\.[^/.]+$/)?.[0] || '.pak'
    const nameWithoutExt = filename.replace(/\.[^/.]+$/, '')

    // Extract priority suffixes (e.g., _9999999_P or multiple like _9999999_P_9999999_P)
    const suffixMatch = nameWithoutExt.match(/((?:_\d+_P)+)$/i)
    const suffix = suffixMatch ? suffixMatch[1] : ''

    // Get base name without the suffix
    const baseName = suffix
      ? nameWithoutExt.substring(0, nameWithoutExt.length - suffix.length)
      : nameWithoutExt

    return { baseName, suffix, extension }
  }

  const handleRenameClick = (e) => {
    e.stopPropagation()
    const { baseName } = getFilenameParts()
    // Use custom_name if set, otherwise use the clean base name
    const currentName = mod.custom_name || baseName
    setRenameValue(currentName)
    setShowRenamePanel(true)
  }

  const handleRenameSubmit = (e) => {
    e?.preventDefault()
    e?.stopPropagation()
    const { baseName } = getFilenameParts()
    const currentName = mod.custom_name || baseName
    if (renameValue.trim() && renameValue.trim() !== currentName) {
      onRename(renameValue.trim())
    }
    onClose()
  }

  const handleRenameKeyDown = (e) => {
    e.stopPropagation()
    if (e.key === 'Enter') {
      handleRenameSubmit(e)
    } else if (e.key === 'Escape') {
      setShowRenamePanel(false)
    }
  }

  // Get filename parts for display
  const { suffix, extension } = mod ? getFilenameParts() : { suffix: '', extension: '.pak' }
  const previewFilename = renameValue.trim() ? `${renameValue.trim()}${suffix}${extension}` : ''


  if (folder) {
    return (
      <div className="context-menu" style={{ top: y, left: x }} onClick={(e) => e.stopPropagation()}>
        <div className="context-menu-header">{folder.name}</div>
        <div className="context-menu-separator" />
        <div
          className={`context-menu-item danger ${isDeleting ? 'holding' : ''}`}
          onMouseDown={handleDeleteDown}
          onMouseUp={handleDeleteUp}
          onMouseLeave={handleDeleteUp}
        >
          <div className="danger-bg" />
          <span style={{ position: 'relative', zIndex: 2 }}>{isDeleting ? 'Hold to delete...' : 'Delete Folder (Hold 2s)'}</span>
        </div>
      </div>
    )
  }

  if (!mod) return null

  return (
    <>
      <div className="context-menu" style={{ top: y, left: x }} onClick={(e) => e.stopPropagation()}>
        <div className="context-menu-header">{mod.custom_name || mod.path.split('\\').pop()}</div>

        <div className="context-menu-item submenu-trigger">
          Assign Tag...
          <div className="submenu">
            <div className="context-menu-item" onClick={() => {
              const tag = prompt('Enter new tag name:');
              if (tag) onAssignTag(tag);
              onClose();
            }}>
              + New Tag...
            </div>
            {allTags && allTags.length > 0 && <div className="context-menu-separator" />}
            {allTags && allTags.map(tag => (
              <div key={tag} className="context-menu-item" onClick={() => { onAssignTag(tag); onClose(); }}>
                {tag}
              </div>
            ))}
          </div>
        </div>

        <div className="context-menu-item submenu-trigger">
          Move to...
          <div className="submenu">
            <div className="context-menu-item" onClick={() => { onCreateFolder(); onClose(); }}>
              + New Folder...
            </div>
            <div className="context-menu-separator" />
            <div className="scrollable-menu-list" style={{ maxHeight: '300px', overflowY: 'auto', paddingRight: '4px' }}>
              {folders.map(f => (
                <div key={f.id} className="context-menu-item" onClick={() => { onMoveTo(f.id); onClose(); }}>
                  {f.name}
                </div>
              ))}
            </div>
            <div className="context-menu-separator" />
            <div className="context-menu-item" onClick={() => { onMoveTo(null); onClose(); }}>
              Root
            </div>
          </div>
        </div>

        <div className="context-menu-separator" />

        <div className="context-menu-item" onClick={() => { onToggle(); onClose(); }}>
          {mod.enabled ? 'Disable' : 'Enable'}
        </div>

        <div className={`context-menu-item ${showRenamePanel ? 'active' : ''}`} onClick={handleRenameClick}>
          Rename
        </div>

        <div
          className={`context-menu-item danger ${isDeleting ? 'holding' : ''}`}
          onMouseDown={handleDeleteDown}
          onMouseUp={handleDeleteUp}
          onMouseLeave={handleDeleteUp}
        >
          <div className="danger-bg" />
          <span style={{ position: 'relative', zIndex: 2 }}>{isDeleting ? 'Hold to delete...' : 'Delete (Hold 2s)'}</span>
        </div>

        <div className="context-menu-separator" />

        <div className="context-menu-item disabled">
          Extract Assets
        </div>
        <div className="context-menu-item" onClick={async () => {
          try {
            await invoke('open_in_explorer', { path: mod.path });
          } catch (e) {
            console.error('Failed to open in explorer:', e);
          }
          onClose();
        }}>
          Open in Explorer
        </div>
        <div className="context-menu-item" onClick={async () => {
          try {
            await invoke('copy_to_clipboard', { text: mod.path });
          } catch (e) {
            console.error('Failed to copy path:', e);
          }
          onClose();
        }}>
          Copy Path
        </div>
      </div>

      {/* Rename Panel - appears next to context menu */}
      {showRenamePanel && (
        <div
          className="rename-panel"
          style={{ top: y + 80, left: x + 190 }}
          onClick={(e) => e.stopPropagation()}
        >
          <div className="rename-panel-header">Rename Mod</div>
          <form onSubmit={handleRenameSubmit}>
            <div className="rename-input-wrapper">
              <input
                ref={renameInputRef}
                type="text"
                className="rename-input"
                value={renameValue}
                onChange={(e) => setRenameValue(e.target.value)}
                onKeyDown={handleRenameKeyDown}
                placeholder="Enter new name..."
              />
              {suffix && <span className="rename-suffix-hint">{suffix}</span>}
            </div>
            {previewFilename && (
              <div className="rename-preview">
                <span className="rename-preview-label">Preview:</span>
                <span className="rename-preview-filename">{previewFilename}</span>
              </div>
            )}
            <div className="rename-panel-actions">
              <button type="button" className="rename-btn cancel" onClick={() => setShowRenamePanel(false)}>
                Cancel
              </button>
              <button type="submit" className="rename-btn confirm">
                Rename
              </button>
            </div>
          </form>
        </div>
      )}
    </>
  )
}

export default ContextMenu

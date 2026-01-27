import React, { useState, useEffect, useRef, useLayoutEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { IoMdWarning } from "react-icons/io"
import './ContextMenu.css'

const ContextMenu = ({ x, y, mod, folder, onClose, onAssignTag, onMoveTo, onCreateFolder, folders, onDelete, onToggle, onRename, onCheckConflicts, allTags, gamePath }) => {
  const [isDeleting, setIsDeleting] = useState(false)
  const deleteTimeoutRef = useRef(null)
  const menuRef = useRef(null)
  const [adjustedPos, setAdjustedPos] = useState({ x, y })

  useEffect(() => {
    const handleClickOutside = () => {
      onClose()
    }
    window.addEventListener('click', handleClickOutside)
    return () => window.removeEventListener('click', handleClickOutside)
  }, [onClose])

  useEffect(() => {
    return () => {
      if (deleteTimeoutRef.current) clearTimeout(deleteTimeoutRef.current)
    }
  }, [])

  // Adjust position to prevent menu from going off-screen
  // Using useLayoutEffect to run after DOM updates but before paint
  useLayoutEffect(() => {
    // First reset to original position
    setAdjustedPos({ x, y })

    // Then measure and adjust in next frame
    requestAnimationFrame(() => {
      if (menuRef.current) {
        const menuRect = menuRef.current.getBoundingClientRect()
        const viewportHeight = window.innerHeight
        const viewportWidth = window.innerWidth

        let newY = y
        let newX = x

        // If menu would go below viewport, flip it to open above cursor
        if (y + menuRect.height > viewportHeight - 10) {
          newY = y - menuRect.height
        }

        // If menu would go off right edge, shift it left
        if (x + menuRect.width > viewportWidth - 10) {
          newX = viewportWidth - menuRect.width - 10
        }

        // Ensure menu doesn't go above or to left of viewport
        newY = Math.max(10, newY)
        newX = Math.max(10, newX)

        if (newX !== x || newY !== y) {
          setAdjustedPos({ x: newX, y: newY })
        }
      }
    })
  }, [x, y])

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

  const handleRenameClick = (e) => {
    e.stopPropagation()
    onRename()
    onClose()
  }

  if (folder) {
    return (
      <div ref={menuRef} className="context-menu" style={{ top: adjustedPos.y, left: adjustedPos.x }} onClick={(e) => e.stopPropagation()}>
        <div className="context-menu-header">{folder.name}</div>
        <div className="context-menu-item" onClick={async () => {
          try {
            // Construct full folder path from gamePath + folder.id
            const separator = gamePath?.includes('\\') ? '\\' : '/';
            const fullPath = gamePath && folder.id ? `${gamePath}${separator}${folder.id}` : folder.id;
            await invoke('open_in_explorer', { path: fullPath });
          } catch (e) {
            console.error('Failed to open folder in explorer:', e);
          }
          onClose();
        }}>
          Open in Explorer
        </div>
        <div className="context-menu-item" onClick={async () => {
          onClose();
          try {
            // Construct full folder path from gamePath + folder.id
            const separator = gamePath?.includes('\\') ? '\\' : '/';
            const fullPath = gamePath && folder.id ? `${gamePath}${separator}${folder.id}` : folder.id;
            await invoke('copy_to_clipboard', { text: fullPath });
          } catch (e) {
            console.error('Failed to copy folder path:', e);
          }
        }}>
          Copy Path
        </div>
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
    <div ref={menuRef} className="context-menu" style={{ top: adjustedPos.y, left: adjustedPos.x }} onClick={(e) => e.stopPropagation()}>
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

      <div className="context-menu-item" onClick={() => { if (onCheckConflicts) onCheckConflicts(); onClose(); }}>
        Check Conflicts <IoMdWarning className="warning-icon-small" style={{ fill: 'var(--accent-primary)' }} />
      </div>

      <div className="context-menu-separator" />

      <div className="context-menu-item" onClick={() => { onToggle(); onClose(); }}>
        {mod.enabled ? 'Disable' : 'Enable'}
      </div>

      <div className="context-menu-item" onClick={handleRenameClick}>
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

      <div className="context-menu-item" onClick={async () => {
        try {
          // Open folder picker dialog
          const { open } = await import('@tauri-apps/plugin-dialog');
          const destFolder = await open({
            directory: true,
            multiple: false,
            title: 'Select destination folder for extracted assets'
          });

          if (destFolder) {
            // Get the mod path - handle both PAK and IoStore
            let modPath = mod.path;

            // If this is an IoStore mod (folder with .utoc), find the .utoc file
            if (mod.is_iostore && mod.utoc_path) {
              modPath = mod.utoc_path;
            }

            const fileCount = await invoke('extract_mod_assets', {
              modPath: modPath,
              destPath: destFolder
            });

            console.log(`Extracted ${fileCount} files to ${destFolder}`);

            // Open the extracted folder in explorer
            const modName = modPath.split(/[/\\]/).pop().replace(/\.[^.]+$/, '');
            await invoke('open_in_explorer', { path: `${destFolder}\\${modName}` });
          }
        } catch (e) {
          console.error('Failed to extract assets:', e);
        }
        onClose();
      }}>
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
  )
}

export default ContextMenu

import React, { useState, useEffect, useRef } from 'react'
import './ContextMenu.css'

const ContextMenu = ({ x, y, mod, onClose, onAssignTag, onMoveTo, onCreateFolder, folders, onDelete, onToggle, allTags }) => {
  const [isDeleting, setIsDeleting] = useState(false)
  const deleteTimeoutRef = useRef(null)

  useEffect(() => {
    const handleClickOutside = () => onClose()
    window.addEventListener('click', handleClickOutside)
    return () => window.removeEventListener('click', handleClickOutside)
  }, [onClose])

  useEffect(() => {
    return () => {
      if (deleteTimeoutRef.current) clearTimeout(deleteTimeoutRef.current)
    }
  }, [])

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

  if (!mod) return null

  return (
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
        Open in Explorer
      </div>
      <div className="context-menu-item disabled">
        Copy Path
      </div>
    </div>
  )
}

export default ContextMenu

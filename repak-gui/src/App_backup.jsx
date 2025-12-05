import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { motion, AnimatePresence } from 'framer-motion'
import { IconButton, Tooltip } from '@mui/material'
import {
  Settings as SettingsIcon,
  Refresh as RefreshIcon,
  CreateNewFolder as CreateNewFolderIcon,
  Search as SearchIcon,
  Clear as ClearIcon,
  ExpandMore as ExpandMoreIcon,
  ChevronRight as ChevronRightIcon,
  Folder as FolderIcon,
  GridView as GridViewIcon,
  ViewModule as ViewModuleIcon,
  ViewList as ViewListIcon,
  Wifi as WifiIcon
} from '@mui/icons-material'
import ModDetailsPanel from './components/ModDetailsPanel'
import InstallModPanel from './components/InstallModPanel'
import SettingsPanel from './components/SettingsPanel'
import SharingPanel from './components/SharingPanel'
import FileTree from './components/FileTree'
import ContextMenu from './components/ContextMenu'
import characterData from './data/character_data.json'
import './App.css'
import './styles/theme.css'
import './styles/Badges.css'
import './styles/Fonts.css'
import logo from './assets/RepakIcon-x256.png'

const toTagArray = (tags) => Array.isArray(tags) ? tags : (tags ? [tags] : [])

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
  
  return Array.from(heroNames)
}

function getAdditionalCategories(details) {
  if (!details) return []
  if (details.additional_categories && details.additional_categories.length > 0) {
    return details.additional_categories
  }
  if (typeof details.mod_type === 'string') {
    const match = details.mod_type.match(/\[(.*?)\]/)
    if (match && match[1]) {
      return match[1].split(',').map(s => s.trim())
    }
  }
  return []
}

// Mod Item Component
function ModItem({ mod, selectedMod, selectedMods, setSelectedMod, handleToggleModSelection, handleToggleMod, handleDeleteMod, handleRemoveTag, formatFileSize, hideSuffix, onContextMenu }) {
  const [isDeleteHolding, setIsDeleteHolding] = useState(false)
  const holdTimeoutRef = useRef(null)
  const rawName = mod.custom_name || mod.path.split('\\').pop()
  const nameWithoutExt = rawName.replace(/\.[^/.]+$/, '')
  const suffixMatch = nameWithoutExt.match(/(_\d+_P)$/i)
  const cleanName = suffixMatch ? nameWithoutExt.slice(0, -suffixMatch[0].length) : nameWithoutExt
  const suffix = suffixMatch ? suffixMatch[0] : ''
  const shouldShowSuffix = !hideSuffix && suffix
  const tags = toTagArray(mod.custom_tags)

  useEffect(() => () => clearTimeout(holdTimeoutRef.current), [])

  const startDeleteHold = (event) => {
    event.stopPropagation()
    clearTimeout(holdTimeoutRef.current)
    setIsDeleteHolding(true)
    holdTimeoutRef.current = setTimeout(() => {
      setIsDeleteHolding(false)
      handleDeleteMod(mod.path)
    }, 2000)
  }

  const cancelDeleteHold = (event) => {
    event.stopPropagation()
    clearTimeout(holdTimeoutRef.current)
    if (isDeleteHolding) setIsDeleteHolding(false)
  }

  return (
    <motion.div 
      className={`mod-card mod-item ${selectedMod === mod ? 'selected' : ''} ${!mod.enabled ? 'disabled' : ''} ${selectedMods.has(mod.path) ? 'bulk-selected' : ''}`}
      initial={{ opacity: 0 }}
      animate={{ opacity: mod.enabled ? 1 : 0.5 }}
      whileHover={{ scale: 1.01 }}
      transition={{ duration: 0.2 }}
      onContextMenu={(e) => onContextMenu(e, mod)}
    >
      <div className="mod-card-row">
        <div className="mod-card-main">
          <Tooltip title="Select mod">
            <input
              type="checkbox"
              checked={selectedMods.has(mod.path)}
              onChange={() => handleToggleModSelection(mod)}
              onClick={(e) => e.stopPropagation()}
              className="modern-checkbox"
            />
          </Tooltip>
          <motion.button 
            type="button"
            className="mod-name-button"
            onClick={(e) => {
              if (e.ctrlKey || e.metaKey) {
                handleToggleModSelection(mod)
              } else {
                setSelectedMod(mod)
              }
            }}
            whileHover={{ color: '#4a9eff' }}
            title={rawName}
          >
            <span className="mod-name-text">
              {cleanName}
              {shouldShowSuffix && <span className="mod-name-suffix">{suffix}</span>}
            </span>
          </motion.button>
        </div>
        <span className="mod-size">{formatFileSize(mod.file_size)}</span>
      </div>
      
      {tags.length > 0 && (
        <div className="tag-container">
          {tags.map(tag => (
            <span key={tag} className="tag">
              {tag}
              <button
                type="button"
                className="tag-remove"
                aria-label={`Remove ${tag}`}
                onClick={(e) => {
                  e.stopPropagation()
                  handleRemoveTag(mod.path, tag)
                }}
              >
                ×
              </button>
            </span>
          ))}
        </div>
      )}
      
      <div className="mod-card-row mod-card-actions">
        <Tooltip title={mod.enabled ? 'Disable mod' : 'Enable mod'}>
          <label
            className={`mod-switch ${mod.enabled ? 'is-on' : ''}`}
            onClick={(e) => e.stopPropagation()}
          >
            <input
              type="checkbox"
              checked={mod.enabled}
              onChange={(e) => {
                e.stopPropagation()
                handleToggleMod(mod.path)
              }}
            />
            <span className="mod-switch-track" />
          </label>
        </Tooltip>
        <Tooltip title="Hold 2s to delete">
          <button
            className={`btn-modern btn-danger hold-delete ${isDeleteHolding ? 'holding' : ''}`}
            onMouseDown={startDeleteHold}
            onMouseUp={cancelDeleteHold}
            onMouseLeave={cancelDeleteHold}
            onTouchStart={startDeleteHold}
            onTouchEnd={cancelDeleteHold}
            aria-label="Hold to delete mod"
          >
            ×
          </button>
        </Tooltip>
      </div>
    </motion.div>
  )
}

function App() {
  // Add these state variables
  const [globalUsmap, setGlobalUsmap] = useState('');
  const [hideSuffix, setHideSuffix] = useState(false);
  
  // Add these new state variables
  const [theme, setTheme] = useState('dark');
  const [accentColor, setAccentColor] = useState('#4a9eff');
  const [showSettings, setShowSettings] = useState(false);
  const [showSharingPanel, setShowSharingPanel] = useState(false);

  const [gamePath, setGamePath] = useState('')
  const [mods, setMods] = useState([])
  const [folders, setFolders] = useState([])
  const [loading, setLoading] = useState(false)
  const [status, setStatus] = useState('')
  const [gameRunning, setGameRunning] = useState(false)
  const [version, setVersion] = useState('')
  const [selectedMod, setSelectedMod] = useState(null)
  const [leftPanelWidth, setLeftPanelWidth] = useState(60) // percentage
  const [isResizing, setIsResizing] = useState(false)
  const [selectedMods, setSelectedMods] = useState(new Set())
  const [showBulkActions, setShowBulkActions] = useState(false)
  const [newTagInput, setNewTagInput] = useState('')
  const [allTags, setAllTags] = useState([])
  const [filterTag, setFilterTag] = useState('')
  const [filterType, setFilterType] = useState('')
  // New: Mod Detection API integration
  const [modDetails, setModDetails] = useState({}) // { [path]: ModDetails }
  const [detailsLoading, setDetailsLoading] = useState(false)
  const [selectedCharacters, setSelectedCharacters] = useState(new Set()) // values: character_name, '__generic', '__multi'
  const [selectedCategories, setSelectedCategories] = useState(new Set()) // category strings
  const [availableCharacters, setAvailableCharacters] = useState([])
  const [availableCategories, setAvailableCategories] = useState([])
  const [showCharacterFilters, setShowCharacterFilters] = useState(true)
  const [showTypeFilters, setShowTypeFilters] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')
  const [expandedFolders, setExpandedFolders] = useState(new Set())
  const [showInstallPanel, setShowInstallPanel] = useState(false)
  const [modsToInstall, setModsToInstall] = useState([])
  const [installLogs, setInstallLogs] = useState([])
  const [showInstallLogs, setShowInstallLogs] = useState(false)
  const [selectedFolderId, setSelectedFolderId] = useState('all')
  const [viewMode, setViewMode] = useState('grid') // 'grid', 'compact', 'list'
  const [contextMenu, setContextMenu] = useState(null) // { x, y, mod }
  // OPTIONAL: user-resizable height
  const [drawerHeight, setDrawerHeight] = useState(380)
  const resizingRef = useRef(false)

  const handleContextMenu = (e, mod) => {
    e.preventDefault()
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      mod
    })
  }

  const closeContextMenu = () => {
    setContextMenu(null)
  }

  useEffect(() => {
    loadInitialData()
    loadTags()
    
    // Listen for install progress
    const unlisten = listen('install_progress', (event) => {
      setStatus(`Installing... ${Math.round(event.payload)}%`)
    })
    
    const unlistenComplete = listen('install_complete', () => {
      setStatus('Installation complete!')
      loadMods()
    })

    const unlistenLogs = listen('install_log', (event) => {
      setInstallLogs(prev => [...prev, event.payload])
      setShowInstallLogs(true)
    })

    // Refresh mod list when character data is updated
    const unlistenCharUpdate = listen('character_data_updated', () => {
      loadMods()
    })

    // Listen for directory changes (new folders, deleted folders, etc.)
    const unlistenDirChanged = listen('mods_dir_changed', () => {
      console.log('Directory changed, reloading mods and folders...')
      loadMods()
      loadFolders()
    })

    // Unified file drop handler function
    const handleFileDrop = async (paths) => {
      if (!paths || paths.length === 0) return
      console.log('Dropped items:', paths)

      try {
        setStatus('Processing dropped items...')
        const modsData = await invoke('parse_dropped_files', { paths })
        if (!modsData || modsData.length === 0) {
          setStatus('No installable mods found in dropped items')
          return
        }
        console.log('Parsed mods:', modsData)
        setModsToInstall(modsData)
        setShowInstallPanel(true)
      } catch (error) {
        console.error('Parse error:', error)
        setStatus(`Error parsing dropped items: ${error}`)
      }
    }

    // Listen for Tauri drag-drop event
    const unlistenDragDrop = listen('tauri://drag-drop', (event) => {
      const files = event.payload.paths || event.payload
      handleFileDrop(files)
    })

    // Listen for Tauri file-drop event
    const unlistenFileDrop = listen('tauri://file-drop', (event) => {
      const files = event.payload.paths || event.payload
      handleFileDrop(files)
    })

    // Add dragover event to prevent default browser behavior
    const preventDefault = (e) => {
      e.preventDefault()
      e.stopPropagation()
    }

    document.addEventListener('dragover', preventDefault)
    document.addEventListener('drop', preventDefault)

    return () => {
      // Cleanup listeners
      unlisten.then(f => f())
      unlistenComplete.then(f => f())
      unlistenCharUpdate.then(f => f())
      unlistenDragDrop.then(f => f())
      unlistenFileDrop.then(f => f())
      unlistenLogs.then(f => f())
      unlistenDirChanged.then(f => f())
      document.removeEventListener('dragover', preventDefault)
      document.removeEventListener('drop', preventDefault)
    }
  }, [])

  useEffect(() => {
    const handleDragEnter = (e) => {
      e.preventDefault()
      setIsDragging(true)
    }

    const handleDragLeave = (e) => {
      e.preventDefault()
      setIsDragging(false)
    }

    document.addEventListener('dragenter', handleDragEnter)
    document.addEventListener('dragleave', handleDragLeave)
    document.addEventListener('drop', () => setIsDragging(false))

    return () => {
      document.removeEventListener('dragenter', handleDragEnter)
      document.removeEventListener('dragleave', handleDragLeave)
      document.removeEventListener('drop', () => setIsDragging(false))
    }
  }, [])

  const loadInitialData = async () => {
    try {
      const path = await invoke('get_game_path')
      setGamePath(path)
      
      const ver = await invoke('get_app_version')
      setVersion(ver)
      
      await loadMods()
      await loadFolders()
      await checkGame()
      
      // Start the file watcher
      await invoke('start_file_watcher')
    } catch (error) {
      console.error('Failed to load initial data:', error)
    }
  }

  const loadMods = async () => {
    try {
      console.log('Loading mods...')
      const modList = await invoke('get_pak_files')
      console.log('Loaded mods:', modList)
      setMods(modList)
      setStatus(`Loaded ${modList.length} mod(s)`)
      // After loading mods, refresh details for each
      preloadModDetails(modList)
    } catch (error) {
      console.error('Error loading mods:', error)
      setStatus('Error loading mods: ' + error)
    }
  }

  // Preload details for all mods using the new Mod Detection API
  const preloadModDetails = async (modList) => {
    if (!Array.isArray(modList) || modList.length === 0) {
      setAvailableCharacters([])
      setAvailableCategories([])
      return
    }

    try {
      setDetailsLoading(true)
      const existing = modDetails
      const pathsToFetch = modList
        .map(m => m.path)
        .filter(p => !existing[p])

      if (pathsToFetch.length === 0) {
        // Already have details; recompute filters source lists
        recomputeFilterSources(modList, modDetails)
        return
      }

      const results = await Promise.allSettled(
        pathsToFetch.map(p => invoke('get_mod_details', { modPath: p }))
      )

      const newMap = { ...existing }
      results.forEach((res, idx) => {
        const path = pathsToFetch[idx]
        if (res.status === 'fulfilled' && res.value) {
          newMap[path] = res.value
        }
      })
      setModDetails(newMap)
      recomputeFilterSources(modList, newMap)
    } catch (e) {
      console.error('Failed to preload mod details:', e)
    } finally {
      setDetailsLoading(false)
    }
  }

  const recomputeFilterSources = (modList, detailsMap) => {
    const charSet = new Set()
    let hasMulti = false
    modList.forEach(m => {
      const d = detailsMap[m.path]
      if (!d) return
      if (d.character_name) charSet.add(d.character_name)
      if (typeof d.mod_type === 'string' && d.mod_type.startsWith('Multiple Heroes')) hasMulti = true
    })
    const catSet = new Set()
    modList.forEach(m => {
      const d = detailsMap[m.path]
      if (!d) return
      if (d.category) catSet.add(d.category)
      const adds = getAdditionalCategories(d)
      adds.forEach(cat => catSet.add(cat))
    })
    setAvailableCharacters(Array.from(charSet).sort((a,b)=>a.localeCompare(b)))
    setAvailableCategories(Array.from(catSet).sort((a,b)=>a.localeCompare(b)))
    // Keep multi-selections if still valid; otherwise prune invalids
    const validChars = new Set(charSet)
    setSelectedCharacters(prev => {
      const next = new Set()
      for (const v of prev) {
        if (v === '__generic' || v === '__multi' || validChars.has(v)) next.add(v)
      }
      return next
    })
    const validCats = new Set(catSet)
    setSelectedCategories(prev => {
      const next = new Set()
      for (const v of prev) {
        if (validCats.has(v)) next.add(v)
      }
      return next
    })
  }

  const loadTags = async () => {
    try {
      const tags = await invoke('get_all_tags')
      setAllTags(tags)
    } catch (error) {
      console.error('Error loading tags:', error)
    }
  }

  const loadFolders = async () => {
    try {
      const folderList = await invoke('get_folders')
      setFolders(folderList)
    } catch (error) {
      console.error('Failed to load folders:', error)
    }
  }

  const checkGame = async () => {
    try {
      const running = await invoke('check_game_running')
      setGameRunning(running)
    } catch (error) {
      console.error('Failed to check game status:', error)
    }
  }

  const handleAutoDetect = async () => {
    try {
      setLoading(true)
      const path = await invoke('auto_detect_game_path')
      setGamePath(path)
      setStatus('Game path detected: ' + path)
      await loadMods()
    } catch (error) {
      setStatus('Failed to auto-detect: ' + error)
    } finally {
      setLoading(false)
    }
  }

  const handleBrowseGamePath = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Marvel Rivals Installation Directory'
      })
      
      if (selected) {
        await invoke('set_game_path', { path: selected })
        setGamePath(selected)
        setStatus('Game path set: ' + selected)
        await loadMods()
      }
    } catch (error) {
      setStatus('Error setting game path: ' + error)
    }
  }

  const handleInstallModClick = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{
          name: 'PAK Files',
          extensions: ['pak']
        }],
        title: 'Select Mods to Install'
      })
      
      if (selected && selected.length > 0) {
        const paths = Array.isArray(selected) ? selected : [selected]
        const modsData = await invoke('parse_dropped_files', { paths })
        setModsToInstall(modsData)
        setShowInstallPanel(true)
      }
    } catch (error) {
      setStatus('Error selecting mods: ' + error)
    }
  }

  const handleDeleteMod = async (modPath) => {
    if (gameRunning) {
      setStatus('Cannot delete mods while game is running')
      return
    }
    // No confirmation prompt needed here, the hold-to-delete button handles the intent
    
    try {
      await invoke('delete_mod', { path: modPath })
      setStatus('Mod deleted')
      await loadMods()
    } catch (error) {
      setStatus('Error deleting mod: ' + error)
    }
  }

  const handleToggleMod = async (modPath) => {
    if (gameRunning) {
      setStatus('Cannot toggle mods while game is running')
      return
    }
    try {
      const newState = await invoke('toggle_mod', { modPath })
      setStatus(newState ? 'Mod enabled' : 'Mod disabled')
      await loadMods()
    } catch (error) {
      setStatus('Error toggling mod: ' + error)
    }
  }

  const handleCreateFolder = async () => {
    const name = prompt('Enter folder name:')
    if (!name) return
    
    try {
      await invoke('create_folder', { name })
      await loadFolders()
      setStatus('Folder created')
    } catch (error) {
      setStatus('Error creating folder: ' + error)
    }
  }

  const handleDeleteFolder = async (folderId) => {
    if (!confirm('Delete this folder? Mods will not be deleted.')) return
    
    try {
      await invoke('delete_folder', { id: folderId })
      await loadFolders()
      await loadMods()
      setStatus('Folder deleted')
    } catch (error) {
      setStatus('Error deleting folder: ' + error)
    }
  }

  const handleToggleModSelection = (mod) => {
    const newSelected = new Set(selectedMods)
    if (newSelected.has(mod.path)) {
      newSelected.delete(mod.path)
    } else {
      newSelected.add(mod.path)
    }
    setSelectedMods(newSelected)
  }

  const handleSelectAll = () => {
    setSelectedMods(new Set(mods.map(m => m.path)))
  }

  const handleDeselectAll = () => {
    setSelectedMods(new Set())
  }

  const handleAssignToFolder = async (folderId) => {
    if (gameRunning) {
      setStatus('Cannot move mods while game is running')
      return
    }

    if (selectedMods.size === 0) {
      setStatus('No mods selected')
      return
    }

    try {
      for (const modPath of selectedMods) {
        await invoke('assign_mod_to_folder', { modPath, folderId })
      }
      setStatus(`Moved ${selectedMods.size} mod(s) to folder!`)
      setSelectedMods(new Set())
      await loadMods()
      await loadFolders()
    } catch (error) {
      setStatus(`Error: ${error}`)
    }
  }

  const handleMoveSingleMod = async (modPath, folderId) => {
    if (gameRunning) {
      setStatus('Cannot move mods while game is running')
      return
    }
    try {
      await invoke('assign_mod_to_folder', { modPath, folderId })
      setStatus('Mod moved to folder')
      await loadMods()
      await loadFolders()
    } catch (error) {
      setStatus('Error moving mod: ' + error)
    }
  }

  const handleAddTagToSingleMod = async (modPath, tag) => {
    try {
      await invoke('add_custom_tag', { modPath, tag })
      setStatus(`Added tag "${tag}"`)
      await loadMods()
      await loadTags()
    } catch (error) {
      setStatus('Error adding tag: ' + error)
    }
  }

  const handleAddCustomTag = async () => {
    if (!newTagInput.trim() || selectedMods.size === 0) return

    try {
      for (const modPath of selectedMods) {
        await invoke('add_custom_tag', { modPath, tag: newTagInput.trim() })
      }
      setStatus(`Added tag "${newTagInput}" to ${selectedMods.size} mod(s)`)
      setNewTagInput('')
      await loadMods()
      await loadTags()
    } catch (error) {
      setStatus(`Error: ${error}`)
    }
  }

  const handleRemoveTag = async (modPath, tag) => {
    try {
      await invoke('remove_custom_tag', { modPath, tag })
      setStatus(`Removed tag "${tag}"`)
      await loadMods()
      await loadTags()
    } catch (error) {
      setStatus(`Error removing tag: ${error}`)
    }
  }

  const handleDragStart = (e, mod) => {
    if (gameRunning) {
      e.preventDefault()
      setStatus('Cannot move mods while game is running')
      return
    }
    console.log('Drag started:', mod.path)
    e.dataTransfer.setData('text', mod.path)
    e.dataTransfer.setData('modpath', mod.path)
    e.dataTransfer.effectAllowed = 'move'
  }

  const handleDragOver = (e) => {
    e.preventDefault()
    e.stopPropagation()
    if (e.dataTransfer.types.includes('modpath')) {
      e.dataTransfer.dropEffect = 'move'
    }
  }

  const handleDropOnFolder = async (e, folderId) => {
    e.preventDefault()
    e.stopPropagation()
    e.currentTarget.classList.remove('drag-over')
    
    if (gameRunning) {
      setStatus('Cannot move mods while game is running')
      return
    }
    
    const modPath = e.dataTransfer.getData('modpath') || e.dataTransfer.getData('text/plain')
    console.log('Drop on folder:', folderId, 'modPath:', modPath)
    
    if (modPath) {
      try {
        console.log('Calling assign_mod_to_folder with:', { modPath, folderId })
        await invoke('assign_mod_to_folder', { modPath, folderId })
        setStatus(`Mod moved to ${folderId}!`)
        await loadMods()
        await loadFolders()
      } catch (error) {
        setStatus(`Error: ${error}`)
        console.error('Error moving mod:', error)
      }
    } else {
      console.error('No modPath in dataTransfer, types:', e.dataTransfer.types)
    }
  }

  const handleResizeStart = (e) => {
    setIsResizing(true)
    e.preventDefault()
  }

  const handleResizeMove = (e) => {
    if (!isResizing) return
    
    const containerWidth = e.currentTarget.offsetWidth || window.innerWidth
    const newLeftWidth = (e.clientX / containerWidth) * 100
    
    // Constrain between 30% and 70%
    if (newLeftWidth >= 30 && newLeftWidth <= 70) {
      setLeftPanelWidth(newLeftWidth)
    }
  }

  const handleResizeEnd = () => {
    setIsResizing(false)
  }

  useEffect(() => {
    if (isResizing) {
      document.addEventListener('mousemove', handleResizeMove)
      document.addEventListener('mouseup', handleResizeEnd)
      return () => {
        document.removeEventListener('mousemove', handleResizeMove)
        document.removeEventListener('mouseup', handleResizeEnd)
      }
    }
  }, [isResizing])

  const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
  }

  // Compute filtered mods
  const filteredMods = mods.filter(mod => {
    // Folder filter
    if (selectedFolderId !== 'all') {
      if (mod.folder_id !== selectedFolderId) return false
    }

    // Search query
    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      const modName = (mod.custom_name || mod.path.split('\\').pop()).toLowerCase()
      if (!modName.includes(query)) return false
    }

    const modTags = toTagArray(mod.custom_tags)

    if (filterTag && !modTags.includes(filterTag)) {
      return false
    }

    // New: Multi-select Character/Hero and Category filters using Mod Detection API
    const hasCharFilter = selectedCharacters.size > 0
    const hasCatFilter = selectedCategories.size > 0
    if (hasCharFilter || hasCatFilter) {
      const d = modDetails[mod.path]
      if (!d) return false // wait for details when filters active
      
      if (hasCatFilter) {
        const mainCatMatch = d.category && selectedCategories.has(d.category)
        const adds = getAdditionalCategories(d)
        const addCatMatch = adds.some(cat => selectedCategories.has(cat))
        if (!mainCatMatch && !addCatMatch) return false
      }

      if (hasCharFilter) {
        const isMulti = typeof d.mod_type === 'string' && d.mod_type.startsWith('Multiple Heroes')
        const isGeneric = !d.character_name && !isMulti
        
        let multiMatch = false
        if (isMulti && d.files) {
          const heroes = detectHeroes(d.files)
          multiMatch = heroes.some(h => selectedCharacters.has(h))
        }

        const match = (
          (d.character_name && selectedCharacters.has(d.character_name)) ||
          (isMulti && selectedCharacters.has('__multi')) ||
          (isGeneric && selectedCharacters.has('__generic')) ||
          multiMatch
        )
        if (!match) return false
      }
    }

    return true
  })

  // Group mods by folder
  const modsByFolder = {}
  modsByFolder['_root'] = filteredMods.filter(m => !m.folder_id)
  folders.forEach(folder => {
    modsByFolder[folder.id] = filteredMods.filter(m => m.folder_id === folder.id)
  })

  const toggleFolder = (folderId) => {
    const newExpanded = new Set(expandedFolders)
    if (newExpanded.has(folderId)) {
      newExpanded.delete(folderId)
    } else {
      newExpanded.add(folderId)
    }
    setExpandedFolders(newExpanded)
  }

  const handleInstallMods = async (modsWithSettings) => {
    try {
      setShowInstallPanel(false)
      setInstallLogs([])
      setShowInstallLogs(true)
      setStatus('Installing mods...')
      await invoke('install_mods', { mods: modsWithSettings })
      setStatus('Mods installed successfully!')
      await loadMods()
      await loadFolders()
    } catch (error) {
      setStatus(`Installation failed: ${error}`)
    }
  }

  const handleSaveSettings = (settings) => {
    setGlobalUsmap(settings.globalUsmap || '')
    setHideSuffix(settings.hideSuffix || false)
    // TODO: Save to backend state
    setStatus('Settings saved')
  }

  // Add this effect to initialize theme
  useEffect(() => {
    const savedTheme = localStorage.getItem('theme') || 'dark';
    const savedAccent = localStorage.getItem('accentColor') || '#4a9eff';
    
    handleThemeChange(savedTheme);
    handleAccentChange(savedAccent);
  }, []);

  // Add these handlers
  const handleThemeChange = (newTheme) => {
    setTheme(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
    localStorage.setItem('theme', newTheme);
  };

  const handleAccentChange = (newAccent) => {
    setAccentColor(newAccent);
    document.documentElement.style.setProperty('--accent-primary', newAccent);
    document.documentElement.style.setProperty('--accent-secondary', newAccent);
    localStorage.setItem('accentColor', newAccent);
  };

  useEffect(() => {
    const onMove = (e) => {
      if (!resizingRef.current) return
      const y = e.clientY
      const vh = window.innerHeight
      const newH = Math.min(Math.max(vh - y, 160), Math.round(vh * 0.85))
      setDrawerHeight(newH)
    }
    const stop = () => { resizingRef.current = false }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', stop)
    window.addEventListener('mouseleave', stop)
    return () => {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', stop)
      window.removeEventListener('mouseleave', stop)
    }
  }, [])

  return (
    <div className="app">
      {showInstallPanel && (
        <InstallModPanel
          mods={modsToInstall}
          onInstall={handleInstallMods}
          onCancel={() => setShowInstallPanel(false)}
        />
      )}

      {showSettings && (
        <SettingsPanel
          settings={{ globalUsmap, hideSuffix }}
          onSave={handleSaveSettings}
          onClose={() => setShowSettings(false)}
          theme={theme}
          setTheme={handleThemeChange}
          accentColor={accentColor}
          setAccentColor={handleAccentChange}
          gamePath={gamePath}
          onAutoDetectGamePath={handleAutoDetect}
          onBrowseGamePath={handleBrowseGamePath}
          isGamePathLoading={loading}
        />
      )}

      {showSharingPanel && (
        <SharingPanel 
          onClose={() => setShowSharingPanel(false)}
          gamePath={gamePath}
          installedMods={mods}
          selectedMods={selectedMods}
        />
      )}

      <header className="header" style={{ display: 'flex', alignItems: 'center' }}>
        <img src={logo} alt="Repak Icon" className="repak-icon" style={{ width: '50px', height: '50px', marginRight: '10px' }} />
        <div style={{ display: 'flex', alignItems: 'baseline', gap: '0.75rem' }}>
          <h1 style={{ margin: 0 }}>Repak GUI Revamped [DEV]</h1>
          <span className="version" style={{ fontSize: '0.9rem', opacity: 0.7 }}>v{version}</span>
        </div>
        <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', marginLeft: 'auto' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer', background: 'rgba(255,0,0,0.1)', padding: '4px 8px', borderRadius: '4px', border: '1px solid rgba(255,0,0,0.3)' }}>
            <input 
              type="checkbox" 
              checked={gameRunning} 
              onChange={(e) => setGameRunning(e.target.checked)} 
            />
            <span style={{ fontSize: '0.8rem', color: '#ff6b6b', fontWeight: 'bold' }}>DEV: Game Running</span>
          </label>
          <button 
            onClick={() => setShowSharingPanel(true)} 
            className="btn-settings"
            title="Share Mods"
          >
            <WifiIcon /> Share
          </button>
          <button 
            onClick={() => setShowSettings(true)} 
            className="btn-settings"
          >
            ⚙️ Settings
          </button>
          {gameRunning && (
            <div className="game-running-indicator">
              <span className="blink-icon">⚠️</span>
              <span className="running-text">Game Running</span>
            </div>
          )}
        </div>
      </header>

      <div className="container">
        {/* Main Action Bar */}
        <div className="main-action-bar">
          <div className="search-wrapper">
            <SearchIcon className="search-icon-large" />
            <input
              type="text"
              placeholder="Search installed mods..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="main-search-input"
            />
            {searchQuery && (
              <button onClick={() => setSearchQuery('')} className="btn-icon-clear">
                <ClearIcon />
              </button>
            )}
          </div>

          <div className="action-controls">
            <select
              value={filterTag}
              onChange={(e) => setFilterTag(e.target.value)}
              className="filter-select-large"
            >
              <option value="">All Tags</option>
              {allTags.map(tag => (
                <option key={tag} value={tag}>{tag}</option>
              ))}
            </select>

            <button onClick={handleInstallModClick} className="btn-install-large">
              <span className="install-icon">+</span>
              <span className="install-text">Install Mod</span>
            </button>
          </div>
        </div>

        {!gamePath && (
          <div className="config-warning">
            ⚠️ Game path not configured. <button onClick={() => setShowSettings(true)} className="btn-link-warning">Configure in Settings</button>
          </div>
        )}

        {/* Main 3-Panel Layout */}
        <div className="main-panels" onMouseMove={handleResizeMove}>
          {/* Wrapper for Left Sidebar and Center Panel */}
          <div className="content-wrapper" style={{ width: `${leftPanelWidth}%`, display: 'flex', height: '100%' }}>
            {/* Left Sidebar - Folders */}
            <div className="left-sidebar">
              {/* Filters Section */}
              <div className="sidebar-filters" style={{ padding: '0.5rem 0.6rem', borderBottom: '1px solid var(--panel-border)' }}>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '0.25rem' }}>
                    <div style={{ fontSize: '0.75rem', opacity: 0.7, fontWeight: 600 }}>FILTERS</div>
                    {(selectedCharacters.size > 0 || selectedCategories.size > 0) && (
                      <button
                        className="btn-ghost-mini"
                        onClick={() => { setSelectedCharacters(new Set()); setSelectedCategories(new Set()) }}
                        title="Clear all filters"
                      >
                        Clear
                      </button>
                    )}
                  </div>

                  {/* Character/Hero Chips */}
                  <div 
                    className="filter-section-header"
                    onClick={() => setShowCharacterFilters(v => !v)}
                    style={{ cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}
                  >
                    <div style={{ fontSize: '0.75rem', opacity: 0.6 }}>Characters {selectedCharacters.size > 0 && `(${selectedCharacters.size})`}</div>
                    <span style={{ fontSize: '0.7rem', opacity: 0.5 }}>{showCharacterFilters ? '\u25bc' : '\u25b6'}</span>
                  </div>
                  {showCharacterFilters && (
                    <div className="filter-chips-scroll">
                      {availableCharacters.map(c => {
                        const active = selectedCharacters.has(c)
                        return (
                          <button
                            key={c}
                            className={`filter-chip-compact ${active ? 'active' : ''}`}
                            onClick={() => setSelectedCharacters(prev => { const next = new Set(prev); active ? next.delete(c) : next.add(c); return next; })}
                            title={c}
                          >
                            {c}
                          </button>
                        )
                      })}
                      {/* Special chips */}
                      <button
                        className={`filter-chip-compact ${selectedCharacters.has('__multi') ? 'active' : ''}`}
                        onClick={() => setSelectedCharacters(prev => { const next = new Set(prev); next.has('__multi') ? next.delete('__multi') : next.add('__multi'); return next; })}
                        title="Multiple Heroes"
                      >
                        Multi
                      </button>
                      <button
                        className={`filter-chip-compact ${selectedCharacters.has('__generic') ? 'active' : ''}`}
                        onClick={() => setSelectedCharacters(prev => { const next = new Set(prev); next.has('__generic') ? next.delete('__generic') : next.add('__generic'); return next; })}
                        title="Generic/Global"
                      >
                        Generic
                      </button>
                    </div>
                  )}

                  {/* Category Chips */}
                  <div 
                    className="filter-section-header"
                    onClick={() => setShowTypeFilters(v => !v)}
                    style={{ cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: '0.25rem' }}
                  >
                    <div style={{ fontSize: '0.75rem', opacity: 0.6 }}>Types {selectedCategories.size > 0 && `(${selectedCategories.size})`}</div>
                    <span style={{ fontSize: '0.7rem', opacity: 0.5 }}>{showTypeFilters ? '\u25bc' : '\u25b6'}</span>
                  </div>
                  {showTypeFilters && (
                    <div className="filter-chips-scroll">
                      {availableCategories.map(cat => {
                        const active = selectedCategories.has(cat)
                        return (
                          <button
                            key={cat}
                            className={`filter-chip-compact ${active ? 'active' : ''}`}
                            onClick={() => setSelectedCategories(prev => { const next = new Set(prev); active ? next.delete(cat) : next.add(cat); return next; })}
                            title={cat}
                          >
                            {cat}
                          </button>
                        )
                      })}
                    </div>
                  )}
                </div>
              </div>
              <div className="sidebar-header">
                <h3>Folders</h3>
                <div style={{ display: 'flex', gap: '4px' }}>
                  <button 
                    onClick={async () => {
                      await loadFolders()
                      await loadMods()
                      setStatus('Folders refreshed')
                    }} 
                    className="btn-icon" 
                    title="Refresh Folders"
                  >
                    <RefreshIcon fontSize="small" />
                  </button>
                  <button onClick={handleCreateFolder} className="btn-icon" title="New Folder">
                    <CreateNewFolderIcon fontSize="small" />
                  </button>
                </div>
              </div>
              <div className="folder-list">
                <div 
                  className={`folder-item ${selectedFolderId === 'all' ? 'active' : ''} ${filteredMods.length === 0 ? 'empty' : ''}`}
                  onClick={() => setSelectedFolderId('all')}
                >
                  <FolderIcon fontSize="small" />
                  <span className="folder-name">All Mods</span>
                  <span className="folder-count">{filteredMods.length}</span>
                </div>
                {folders.map(folder => {
                  const count = filteredMods.filter(m => m.folder_id === folder.id).length;
                  const hasFilters = selectedCharacters.size > 0 || selectedCategories.size > 0;
                  // Hide empty folders when filters are active
                  if (hasFilters && count === 0) return null;
                  
                  return (
                    <div 
                      key={folder.id} 
                      className={`folder-item ${selectedFolderId === folder.id ? 'active' : ''} ${count === 0 ? 'empty' : ''}`}
                      onClick={() => setSelectedFolderId(folder.id)}
                    >
                      <FolderIcon fontSize="small" />
                      <span className="folder-name">{folder.name}</span>
                      <span className="folder-count">{count}</span>
                      <button 
                        onClick={(e) => {
                          e.stopPropagation()
                          handleDeleteFolder(folder.id)
                        }}
                        className="btn-icon-small delete-folder"
                      >
                        ×
                      </button>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Center Panel - Mod List */}
            <div className="center-panel" style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
              <div className="center-header">
                <div className="header-title">
                  <h2>
                    {selectedFolderId === 'all' ? 'All Mods' : 
                     folders.find(f => f.id === selectedFolderId)?.name || 'Unknown Folder'}
                  </h2>
                  <span className="mod-count">({filteredMods.length})</span>
                </div>
                <div className="header-actions">
                  <div className="view-switcher">
                    <button 
                      onClick={() => setViewMode('grid')} 
                      className={`btn-icon-small ${viewMode === 'grid' ? 'active' : ''}`}
                      title="Grid View"
                    >
                      <GridViewIcon fontSize="small" />
                    </button>
                    <button 
                      onClick={() => setViewMode('compact')} 
                      className={`btn-icon-small ${viewMode === 'compact' ? 'active' : ''}`}
                      title="Compact View"
                    >
                      <ViewModuleIcon fontSize="small" />
                    </button>
                    <button 
                      onClick={() => setViewMode('list')} 
                      className={`btn-icon-small ${viewMode === 'list' ? 'active' : ''}`}
                      title="List View"
                    >
                      <ViewListIcon fontSize="small" />
                    </button>
                  </div>
                  <div className="divider-vertical" />
                  <button onClick={loadMods} className="btn-ghost">
                    <RefreshIcon fontSize="small" />
                  </button>
                </div>
              </div>

              {/* Bulk Actions Toolbar */}
              <div className={`bulk-actions-toolbar ${selectedMods.size === 0 ? 'inactive' : ''}`}>
                 <div className="selection-info">
                   {selectedMods.size} selected
                   <button onClick={handleDeselectAll} className="btn-link">Clear</button>
                 </div>
                 <div className="bulk-controls">
                   <select
                      className="toolbar-select"
                      disabled={selectedMods.size === 0}
                      defaultValue=""
                      onChange={(e) => {
                        const folderId = e.target.value
                        if (!folderId) return
                        handleAssignToFolder(folderId)
                        e.target.value = ''
                      }}
                    >
                      <option value="">Move to...</option>
                      {folders.map(f => (
                        <option key={f.id} value={f.id}>{f.name}</option>
                      ))}
                    </select>
                 </div>
              </div>

              <div className={`mod-list-grid view-${viewMode}`} style={{ flex: 1, overflowY: 'auto', padding: '1rem' }}>
                {filteredMods.length === 0 ? (
                  <div className="empty-state">
                    <p>No mods found in this folder.</p>
                  </div>
                ) : (
                  filteredMods.map(mod => (
                    <ModItem 
                      key={mod.path} 
                      mod={mod}
                      selectedMod={selectedMod}
                      selectedMods={selectedMods}
                      setSelectedMod={setSelectedMod}
                      handleToggleModSelection={handleToggleModSelection}
                      handleToggleMod={handleToggleMod}
                      handleDeleteMod={handleDeleteMod}
                      handleRemoveTag={handleRemoveTag}
                      formatFileSize={formatFileSize}
                      hideSuffix={hideSuffix}
                      onContextMenu={handleContextMenu}
                    />
                  ))
                )}
              </div>
            </div>
          </div>

          {/* Resize Handle */}
          <div 
            className="resize-handle"
            onMouseDown={handleResizeStart}
            style={{ left: `${leftPanelWidth}%` }}
          />

          {/* Right Panel - Mod Details (Always Visible) */}
          <div className="right-panel" style={{ width: `${100 - leftPanelWidth}%` }}>
            {selectedMod ? (
              <div className="mod-details-and-contents" style={{ display: 'flex', gap: '1rem', alignItems: 'flex-start' }}>
                <div style={{ flex: 1 }}>
                  <ModDetailsPanel 
                    mod={selectedMod}
                    initialDetails={modDetails[selectedMod.path]}
                    onClose={() => setSelectedMod(null)}
                  />
                </div>

                <div className="selected-mod-contents" style={{ width: '360px', maxWidth: '45%', minWidth: '240px' }}>
                  <h3 style={{ marginTop: 0 }}>Contents</h3>
                  <FileTree files={selectedMod.file_contents || selectedMod.files || selectedMod.file_list || []} />
                </div>
              </div>
            ) : (
               <div className="no-selection">
                 <p>Select a mod to view details</p>
               </div>
             )}
          </div>
        </div>
      </div>

      <motion.div
        className="install-drawer"
        animate={{ height: showInstallLogs ? drawerHeight : 36 }}
        transition={{ type: 'tween', duration: 0.25 }}
      >
        <div
          className="install-drawer-header"
          onClick={() => setShowInstallLogs(v => !v)}
        >
          <span className="status-text">{status || 'Idle'}</span>
          <div
            className="drawer-actions"
            onClick={(e) => e.stopPropagation()}
          >
            <button
              className="btn-link"
              onClick={() => setShowInstallLogs(v => !v)}
            >
              {showInstallLogs ? 'Hide Log ▼' : 'Show Log ▲'}
            </button>
            {installLogs.length > 0 && showInstallLogs && (
              <button
                className="btn-link"
                onClick={() => setInstallLogs([])}
              >
                Clear
              </button>
            )}
          </div>
        </div>
        {showInstallLogs && (
          <div
            className="drawer-resize-handle"
            onMouseDown={(e) => {
              e.stopPropagation()
              resizingRef.current = true
            }}
            title="Drag to resize"
          />
        )}
        <AnimatePresence initial={false}>
          {showInstallLogs && (
            <motion.div
              className="install-drawer-body"
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 12 }}
              transition={{ duration: 0.2 }}
            >
              {installLogs.length === 0 ? (
                <div className="log-empty">Waiting for installation...</div>
              ) : (
                <div className="log-scroll">
                  {installLogs.map((log, i) => (
                    <div key={i} className="log-line">{log}</div>
                  ))}
                </div>
              )}
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          mod={contextMenu.mod}
          onClose={closeContextMenu}
          onAssignTag={(tag) => handleAddTagToSingleMod(contextMenu.mod.path, tag)}
          onMoveTo={(folderId) => handleMoveSingleMod(contextMenu.mod.path, folderId)}
          onCreateFolder={handleCreateFolder}
          folders={folders}
          onDelete={() => handleDeleteMod(contextMenu.mod.path)}
          onToggle={() => handleToggleMod(contextMenu.mod.path)}
          allTags={allTags}
        />
      )}
    </div>
  )
}

export default App

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
  Wifi as WifiIcon,
  ViewSidebar as ViewSidebarIcon,
  PlayArrow as PlayArrowIcon,
  Check as CheckIcon
} from '@mui/icons-material'
import { RiDeleteBin2Fill } from 'react-icons/ri'
import { FaTag } from "react-icons/fa6"
import Checkbox from './components/ui/Checkbox'
import ModDetailsPanel from './components/ModDetailsPanel'
import ModsList from './components/ModsList'
import InstallModPanel from './components/InstallModPanel'
import SettingsPanel from './components/SettingsPanel'
import SharingPanel from './components/SharingPanel'
import FileTree from './components/FileTree'
import FolderTree from './components/FolderTree'
import ContextMenu from './components/ContextMenu'
import LogDrawer from './components/LogDrawer'
import DropZoneOverlay from './components/DropZoneOverlay'
import { AuroraText } from './components/ui/AuroraText'
import Switch from './components/ui/Switch'
import NumberInput from './components/ui/NumberInput'
import characterData from './data/character_data.json'
import './App.css'
import './styles/theme.css'
import './styles/Badges.css'
import './styles/Fonts.css'
import logo from './assets/app-icons/RepakIcon-x256.png'
import ClashPanel from './components/ClashPanel'

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

// ModItem has been moved to src/components/ModsList.jsx

// ClashPanel has been moved to src/components/ClashPanel.jsx

function App() {
  const [globalUsmap, setGlobalUsmap] = useState('');
  const [hideSuffix, setHideSuffix] = useState(false);
  const [autoOpenDetails, setAutoOpenDetails] = useState(false);

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
  const [leftPanelWidth, setLeftPanelWidth] = useState(100) // percentage
  const [lastPanelWidth, setLastPanelWidth] = useState(70) // to restore after collapse (default 30% right panel)
  const [isRightPanelOpen, setIsRightPanelOpen] = useState(false)
  const [isResizing, setIsResizing] = useState(false)
  const [selectedMods, setSelectedMods] = useState(new Set())
  const [showBulkActions, setShowBulkActions] = useState(false)
  const [newTagInput, setNewTagInput] = useState('')
  const [allTags, setAllTags] = useState([])
  const [filterTag, setFilterTag] = useState('')
  const [filterType, setFilterType] = useState('')
  const [modDetails, setModDetails] = useState({}) // { [path]: ModDetails }
  const [detailsLoading, setDetailsLoading] = useState(false)
  const [selectedCharacters, setSelectedCharacters] = useState(new Set()) // values: character_name, '__generic', '__multi'
  const [selectedCategories, setSelectedCategories] = useState(new Set()) // category strings
  const [availableCharacters, setAvailableCharacters] = useState([])
  const [availableCategories, setAvailableCategories] = useState([])
  const [showCharacterFilters, setShowCharacterFilters] = useState(false)
  const [showTypeFilters, setShowTypeFilters] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [expandedFolders, setExpandedFolders] = useState(new Set())
  const [showInstallPanel, setShowInstallPanel] = useState(false)
  const [modsToInstall, setModsToInstall] = useState([])
  const [installLogs, setInstallLogs] = useState([])
  const [selectedFolderId, setSelectedFolderId] = useState('all')
  const [viewMode, setViewMode] = useState('list') // 'grid', 'compact', 'list'
  const [contextMenu, setContextMenu] = useState(null) // { x, y, mod }

  const [clashes, setClashes] = useState([])
  const [showClashPanel, setShowClashPanel] = useState(false)
  const [launchSuccess, setLaunchSuccess] = useState(false)
  const [isDragging, setIsDragging] = useState(false)
  const [dropTargetFolder, setDropTargetFolder] = useState(null)
  const dropTargetFolderRef = useRef(null)

  const handleCheckClashes = async () => {
    try {
      setStatus('Checking for clashes...')
      const result = await invoke('check_mod_clashes')
      setClashes(result)
      setShowClashPanel(true)
      setStatus(`Found ${result.length} clashes`)
    } catch (error) {
      setStatus('Error checking clashes: ' + error)
    }
  }

  const handleSetPriority = async (modPath, priority) => {
    if (gameRunning) {
      setStatus('Cannot change priority while game is running')
      return
    }
    try {
      await invoke('set_mod_priority', { modPath, priority })
      setStatus(`Priority set to ${priority}`)

      // If the modified mod is currently selected, clear selection to force refresh of details
      // This ensures the details panel updates with the new filename (since priority changes filename)
      if (selectedMod && selectedMod.path === modPath) {
        setSelectedMod(null)
      }

      await loadMods()

      // Refresh clash list if panel is open
      if (showClashPanel) {
        const result = await invoke('check_mod_clashes')
        setClashes(result)
      }
    } catch (error) {
      setStatus('Error setting priority: ' + error)
    }
  }

  const handleModSelect = (mod) => {
    setSelectedMod(mod)
    if (autoOpenDetails && !isRightPanelOpen) {
      setLeftPanelWidth(lastPanelWidth > 60 ? lastPanelWidth : 70) // Ensure reasonable width
      setIsRightPanelOpen(true)
    }
  }

  const handleContextMenu = (e, mod) => {
    e.preventDefault()
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      mod
    })
  }

  const handleFolderContextMenu = (e, folder) => {
    e.preventDefault()
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      folder
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

        // Check if we should quick-organize to a folder (using ref for current value in closure)
        const targetFolder = dropTargetFolderRef.current
        if (targetFolder) {
          // Quick organize: directly install to the folder without showing install panel
          console.log('Quick organizing to folder:', targetFolder)
          setStatus(`Quick installing ${modsData.length} mod(s) to ${targetFolder}...`)

          // Prepare mods with folder assignment for direct install
          const modsWithFolder = modsData.map(mod => ({
            ...mod,
            targetFolder: targetFolder
          }))

          try {
            // Install directly
            await invoke('install_mods', { mods: modsWithFolder })
            setStatus(`Installed ${modsData.length} mod(s) to ${targetFolder}!`)
            await loadMods()
            await loadFolders()
          } catch (installError) {
            console.error('Quick install error:', installError)
            setStatus(`Error installing mods: ${installError}`)
          }

          setDropTargetFolder(null) // Reset for next drop
        } else {
          // Normal drop: show install panel
          setModsToInstall(modsData)
          setShowInstallPanel(true)
        }
      } catch (error) {
        console.error('Parse error:', error)
        setStatus(`Error parsing dropped items: ${error}`)
      }
    }

    // Listen for Tauri drag-drop event
    const unlistenDragDrop = listen('tauri://drag-drop', (event) => {
      const files = event.payload.paths || event.payload
      setIsDragging(false)
      handleFileDrop(files)
    })

    // Listen for Tauri file-drop event
    const unlistenFileDrop = listen('tauri://file-drop', (event) => {
      const files = event.payload.paths || event.payload
      setIsDragging(false)
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

  // Tauri drag hover detection - use Tauri's events instead of browser events
  useEffect(() => {
    // Listen for Tauri drag-enter event (when files first enter the window)
    const unlistenDragEnter = listen('tauri://drag-enter', () => {
      console.log('Tauri drag-enter detected')
      setIsDragging(true)
    })

    // Listen for Tauri drag-leave event (when files leave the window)
    const unlistenDragLeave = listen('tauri://drag-leave', () => {
      console.log('Tauri drag-leave detected')
      setIsDragging(false)
    })

    // Also reset on drag-cancelled
    const unlistenDragCancelled = listen('tauri://drag-cancelled', () => {
      console.log('Tauri drag-cancelled detected')
      setIsDragging(false)
    })

    return () => {
      unlistenDragEnter.then(f => f())
      unlistenDragLeave.then(f => f())
      unlistenDragCancelled.then(f => f())
    }
  }, [])

  // Keep the ref in sync with state for access in event listener closures
  useEffect(() => {
    dropTargetFolderRef.current = dropTargetFolder
  }, [dropTargetFolder])

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
    setAvailableCharacters(Array.from(charSet).sort((a, b) => a.localeCompare(b)))
    setAvailableCategories(Array.from(catSet).sort((a, b) => a.localeCompare(b)))
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

  const handleDevInstallPanel = () => {
    const categories = ['Skin', 'Audio', 'UI', 'VFX', 'Mesh', 'Texture']
    const additionalCats = ['Blueprint', 'Text', '']

    const getRandomMod = (i) => {
      const randomChar = characterData[Math.floor(Math.random() * characterData.length)].name
      const randomCat = categories[Math.floor(Math.random() * categories.length)]
      const randomAdd = additionalCats[Math.floor(Math.random() * additionalCats.length)]

      let modType = `${randomChar} - ${randomCat}`
      if (randomAdd) {
        modType += ` [${randomAdd}]`
      }

      return {
        path: `C:\\Fake\\Path\\Mod${i}.pak`,
        mod_name: `Mod${i}.pak`,
        file_size: Math.floor(Math.random() * 1024 * 1024 * 50),
        mod_type: modType,
        auto_fix_mesh: Math.random() > 0.5,
        auto_fix_texture: Math.random() > 0.5,
        auto_fix_serialize_size: Math.random() > 0.5,
        auto_to_repak: Math.random() > 0.5
      }
    }

    setModsToInstall([getRandomMod(1), getRandomMod(2), getRandomMod(3)])
    setShowInstallPanel(true)
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

      // Clear selection if the deleted mod was selected
      if (selectedMod && selectedMod.path === modPath) {
        setSelectedMod(null)
      }

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
    // No confirmation prompt needed here, the hold-to-delete button handles the intent

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

    // Check if folderId corresponds to the root folder (depth 0)
    // If so, pass null to backend to move to root
    const targetFolder = folders.find(f => f.id === folderId)
    const effectiveFolderId = (targetFolder && targetFolder.depth === 0) ? null : folderId

    try {
      for (const modPath of selectedMods) {
        await invoke('assign_mod_to_folder', { modPath, folderId: effectiveFolderId })
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

    // Check if folderId corresponds to the root folder (depth 0)
    const targetFolder = folders.find(f => f.id === folderId)
    const effectiveFolderId = (targetFolder && targetFolder.depth === 0) ? null : folderId

    try {
      await invoke('assign_mod_to_folder', { modPath, folderId: effectiveFolderId })
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

  // Keep the global tag list in sync when the Install panel creates a new tag
  const registerTagFromInstallPanel = (tag) => {
    const trimmed = (tag || '').trim()
    if (!trimmed) return
    setAllTags(prev => prev.includes(trimmed) ? prev : [...prev, trimmed].sort())
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
        // Check if folderId corresponds to the root folder (depth 0)
        const targetFolder = folders.find(f => f.id === folderId)
        const effectiveFolderId = (targetFolder && targetFolder.depth === 0) ? null : folderId

        console.log('Calling assign_mod_to_folder with:', { modPath, folderId: effectiveFolderId })
        await invoke('assign_mod_to_folder', { modPath, folderId: effectiveFolderId })
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

    // Constrain right panel between 25% and 40% (left panel 60% - 75%)
    if (newLeftWidth >= 60 && newLeftWidth <= 75) {
      setLeftPanelWidth(newLeftWidth)
      if (isRightPanelOpen) {
        setLastPanelWidth(newLeftWidth)
      }
    }
  }

  const handleResizeEnd = () => {
    setIsResizing(false)
  }

  const toggleRightPanel = () => {
    if (isRightPanelOpen) {
      // Collapse
      setLastPanelWidth(leftPanelWidth)
      setLeftPanelWidth(100)
      setIsRightPanelOpen(false)
    } else {
      // Expand
      setLeftPanelWidth(lastPanelWidth)
      setIsRightPanelOpen(true)
    }
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

  // Compute base filtered mods (excluding folder filter)
  const baseFilteredMods = mods.filter(mod => {
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

  // Apply folder filter to get final list for display
  const filteredMods = baseFilteredMods.filter(mod => {
    if (selectedFolderId === 'all') return true

    // Match exact folder OR subfolder
    // e.g. if selected is "Category", match "Category" and "Category/Sub"
    return mod.folder_id === selectedFolderId ||
      (mod.folder_id && mod.folder_id.startsWith(selectedFolderId + '/'))
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
      setStatus('Installing mods...')

      await invoke('install_mods', { mods: modsWithSettings })

      // Mirror tag assignment flow used by the main list/context menu
      const typeTracker = {}
      for (const mod of modsWithSettings) {
        const modType = mod.mod_type || 'Unknown'
        const count = typeTracker[modType] || 0
        const minNines = 7 + count
        const name = mod.customName || mod.mod_name
        const filename = `${normalizeModBaseName(name, minNines)}.pak`

        if (mod.selectedTags && mod.selectedTags.length > 0) {
          const separator = gamePath.includes('\\') ? '\\' : '/'
          const fullPath = `${gamePath}${separator}${filename}`

          for (const tag of mod.selectedTags) {
            try {
              await invoke('add_custom_tag', { modPath: fullPath, tag })
            } catch (e) {
              console.error(`Failed to add tag ${tag} to ${fullPath}:`, e)
            }
          }
        }

        typeTracker[modType] = count + 1
      }

      setStatus('Mods installed successfully!')
      await loadMods()
      await loadFolders()
      await loadTags()
    } catch (error) {
      setStatus(`Installation failed: ${error}`)
    }
  }

  const handleSaveSettings = (settings) => {
    setGlobalUsmap(settings.globalUsmap || '')
    setHideSuffix(settings.hideSuffix || false)
    setAutoOpenDetails(settings.autoOpenDetails || false)
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

  return (
    <div className="app">
      {showInstallPanel && (
        <InstallModPanel
          mods={modsToInstall}
          allTags={allTags}
          onCreateTag={registerTagFromInstallPanel}
          onInstall={handleInstallMods}
          onCancel={() => setShowInstallPanel(false)}
        />
      )}

      {showClashPanel && (
        <ClashPanel
          clashes={clashes}
          mods={mods}
          onSetPriority={handleSetPriority}
          onClose={() => setShowClashPanel(false)}
        />
      )}

      {showSettings && (
        <SettingsPanel
          settings={{ globalUsmap, hideSuffix, autoOpenDetails }}
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

      {/* Drop Zone Overlay */}
      <DropZoneOverlay
        isVisible={isDragging}
        folders={folders}
        onInstallDrop={() => {
          // Just signals intent - actual files come from Tauri event
          setDropTargetFolder(null)
        }}
        onQuickOrganizeDrop={(folderId) => {
          // Store the target folder for when Tauri fires the drop event
          setDropTargetFolder(folderId)
        }}
        onClose={() => setIsDragging(false)}
      />


      <header className="header" style={{ display: 'flex', alignItems: 'center' }}>
        <img src={logo} alt="Repak Icon" className="repak-icon" style={{ width: '50px', height: '50px', marginRight: '10px' }} />
        <div style={{ display: 'flex', alignItems: 'baseline', gap: '0.75rem' }}>
          <h1 style={{ margin: 0 }}>Repak <AuroraText className="font-bbh-bartle">X</AuroraText> [DEV]</h1>
          <span className="version" style={{ fontSize: '0.9rem', opacity: 0.7 }}>v{version}</span>
        </div>
        <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', marginLeft: 'auto' }}>
          <button
            className="btn-settings"
            title="Launch Rivals"
            style={{
              background: launchSuccess ? 'rgba(76, 175, 80, 0.15)' : 'rgba(74, 158, 255, 0.1)',
              color: launchSuccess ? '#4CAF50' : '#4a9eff',
              border: launchSuccess ? '1px solid rgba(76, 175, 80, 0.5)' : '1px solid rgba(74, 158, 255, 0.3)',
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              fontWeight: 600,
              transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
              padding: '6px 16px',
              minWidth: '100px',
              justifyContent: 'center'
            }}
            onClick={async () => {
              if (launchSuccess) return
              try {
                await invoke('launch_game')
                setStatus('Game launched')
                setLaunchSuccess(true)
                setTimeout(() => setLaunchSuccess(false), 3000)
              } catch (error) {
                setStatus('Error launching game: ' + error)
              }
            }}
          >
            <AnimatePresence mode="wait">
              {launchSuccess ? (
                <motion.span
                  key="success"
                  initial={{ opacity: 0, scale: 0.5 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.5 }}
                  style={{ display: 'flex', alignItems: 'center', gap: '8px' }}
                >
                  <CheckIcon fontSize="small" /> Launched
                </motion.span>
              ) : (
                <motion.span
                  key="play"
                  initial={{ opacity: 0, scale: 0.5 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.5 }}
                  style={{ display: 'flex', alignItems: 'center', gap: '8px' }}
                >
                  <PlayArrowIcon fontSize="small" /> Launch Game
                </motion.span>
              )}
            </AnimatePresence>
          </button>

          <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer', background: 'rgba(255,0,0,0.1)', padding: '4px 8px', borderRadius: '4px', border: '1px solid rgba(255,0,0,0.3)' }}>
            <input
              type="checkbox"
              checked={gameRunning}
              onChange={(e) => setGameRunning(e.target.checked)}
            />
            <span style={{ fontSize: '0.8rem', color: '#ff6b6b', fontWeight: 'bold' }}>DEV: Game Running</span>
          </label>
          <button
            onClick={handleDevInstallPanel}
            style={{
              background: 'rgba(255, 165, 0, 0.1)',
              color: 'orange',
              border: '1px solid rgba(255, 165, 0, 0.3)',
              padding: '4px 8px',
              borderRadius: '4px',
              fontSize: '0.8rem',
              fontWeight: 'bold',
              cursor: 'pointer'
            }}
          >
            DEV: Install Panel
          </button>
          {gameRunning && (
            <div className="game-running-indicator">
              <span className="blink-icon">⚠️</span>
              <span className="running-text">Game Running</span>
            </div>
          )}
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
            <SettingsIcon /> Settings
          </button>
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
          <motion.div
            className="content-wrapper"
            animate={{ width: `${leftPanelWidth}%` }}
            transition={isResizing ? { duration: 0 } : { type: "tween", ease: "circOut", duration: 0.35 }}
            style={{ display: 'flex', height: '100%' }}
          >
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
                  <button onClick={handleCreateFolder} className="btn-icon" title="New Folder">
                    <CreateNewFolderIcon fontSize="small" />
                  </button>
                </div>
              </div>
              <div className="folder-list">
                <FolderTree
                  folders={folders}
                  selectedFolderId={selectedFolderId}
                  onSelect={setSelectedFolderId}
                  onDelete={handleDeleteFolder}
                  onContextMenu={handleFolderContextMenu}
                  getCount={(id) => {
                    if (id === 'all') return baseFilteredMods.length;
                    return baseFilteredMods.filter(m =>
                      m.folder_id === id ||
                      (m.folder_id && m.folder_id.startsWith(id + '/'))
                    ).length;
                  }}
                  hasFilters={selectedCharacters.size > 0 || selectedCategories.size > 0}
                />
              </div>
            </div>

            {/* Center Panel - Mod List */}
            <div className="center-panel" style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
              <div className="center-header">
                <div className="header-title">
                  <h2>
                    {selectedFolderId === 'all' ? 'All Mods' :
                      folders.find(f => f.id === selectedFolderId)?.name || 'Unknown Folder'}
                    <span className="mod-count" style={{ marginLeft: '0.75rem', opacity: 0.5, fontSize: '0.65em', fontWeight: 'normal' }}>
                      ({filteredMods.filter(m => m.enabled).length} / {filteredMods.length} Enabled)
                    </span>
                  </h2>
                </div>
                <div className="header-actions">
                  <button onClick={handleCheckClashes} className="btn-ghost" title="Check for conflicts" style={{ marginRight: '0.5rem', fontSize: '0.8rem' }}>
                    ⚠️ Check Conflicts
                  </button>
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
                  <button
                    onClick={toggleRightPanel}
                    className={`btn-ghost ${!isRightPanelOpen ? 'active' : ''}`}
                    title={isRightPanelOpen ? "Collapse Details" : "Expand Details"}
                  >
                    <ViewSidebarIcon fontSize="small" style={{ transform: isRightPanelOpen ? 'none' : 'rotate(180deg)' }} />
                  </button>
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

              <ModsList
                mods={filteredMods}
                viewMode={viewMode}
                selectedMod={selectedMod}
                selectedMods={selectedMods}
                onSelect={handleModSelect}
                onToggleSelection={handleToggleModSelection}
                onToggleMod={handleToggleMod}
                onDeleteMod={handleDeleteMod}
                onRemoveTag={handleRemoveTag}
                onSetPriority={handleSetPriority}
                onContextMenu={handleContextMenu}
                formatFileSize={formatFileSize}
                hideSuffix={hideSuffix}
              />
            </div>
          </motion.div>

          {/* Resize Handle */}
          {/* Resize Handle */}
          <motion.div
            className="resize-handle"
            onMouseDown={handleResizeStart}
            animate={{ left: `${leftPanelWidth}%` }}
            transition={isResizing ? { duration: 0 } : { type: "tween", ease: "circOut", duration: 0.35 }}
          />

          {/* Right Panel - Mod Details (Always Visible) */}
          <motion.div
            className="right-panel"
            animate={{ width: `${100 - leftPanelWidth}%` }}
            transition={isResizing ? { duration: 0 } : { type: "tween", ease: "circOut", duration: 0.35 }}
            style={{ display: 'flex' }}
          >
            {selectedMod ? (
              <div className="mod-details-and-contents" style={{ display: 'flex', gap: '1rem', height: '100%', overflow: 'hidden' }}>
                <div style={{ flex: 1, minWidth: '200px', height: '100%' }}>
                  <ModDetailsPanel
                    mod={selectedMod}
                    initialDetails={modDetails[selectedMod.path]}
                    onClose={() => setSelectedMod(null)}
                  />
                </div>

                <div className="selected-mod-contents" style={{ width: '360px', maxWidth: '45%', minWidth: '200px', height: '100%', overflow: 'auto' }}>
                  <h3 style={{ marginTop: 0 }}>Contents</h3>
                  <FileTree files={selectedMod.file_contents || selectedMod.files || selectedMod.file_list || []} />
                </div>
              </div>
            ) : (
              <div className="no-selection">
                <p>Select a mod to view details</p>
              </div>
            )}
          </motion.div>
        </div>
      </div >

      <LogDrawer
        status={status}
        logs={installLogs}
        onClear={() => setInstallLogs([])}
      />

      {
        contextMenu && (
          <ContextMenu
            x={contextMenu.x}
            y={contextMenu.y}
            mod={contextMenu.mod}
            folder={contextMenu.folder}
            onClose={closeContextMenu}
            onAssignTag={(tag) => contextMenu.mod && handleAddTagToSingleMod(contextMenu.mod.path, tag)}
            onMoveTo={(folderId) => contextMenu.mod && handleMoveSingleMod(contextMenu.mod.path, folderId)}
            onCreateFolder={handleCreateFolder}
            folders={folders}
            onDelete={() => {
              if (contextMenu.folder) {
                handleDeleteFolder(contextMenu.folder.id)
              } else if (contextMenu.mod) {
                handleDeleteMod(contextMenu.mod.path)
              }
            }}
            onToggle={() => contextMenu.mod && handleToggleMod(contextMenu.mod.path)}
            allTags={allTags}
          />
        )
      }
    </div >
  )
}

export default App

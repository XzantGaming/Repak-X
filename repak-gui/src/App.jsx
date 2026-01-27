import { useState, useEffect, useRef, lazy, Suspense } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { motion, AnimatePresence } from 'framer-motion'
import { useDebouncedCallback } from 'use-debounce'
import { IconButton, Tooltip } from '@mui/material'
import {
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
  ViewHeadline as ViewHeadlineIcon,
  ViewSidebar as ViewSidebarIcon,
  PlayArrow as PlayArrowIcon,
  Check as CheckIcon,
} from '@mui/icons-material'
import { RiDeleteBin2Fill } from 'react-icons/ri'
import { MdDriveFileMoveOutline } from "react-icons/md"
import { FaTag, FaToolbox } from "react-icons/fa6"
import { IoMdWifi, IoIosSettings, IoMdWarning } from "react-icons/io"
import { GrInstall } from "react-icons/gr"
import Checkbox from './components/ui/Checkbox'
import ModDetailsPanel from './components/ModDetailsPanel'
import ModsList from './components/ModsList'
import FileTree from './components/FileTree'
import FolderTree from './components/FolderTree'
import ContextMenu from './components/ContextMenu'
import LogDrawer from './components/LogDrawer'
import DropZoneOverlay from './components/DropZoneOverlay'
import ExtensionModOverlay from './components/ExtensionModOverlay'
import QuickOrganizeOverlay from './components/QuickOrganizeOverlay'
import InputPromptModal from './components/InputPromptModal'

import { AuroraText } from './components/ui/AuroraText'
import { AlertProvider, useAlert } from './components/AlertHandler'
import { useGlobalTooltips } from './hooks/useGlobalTooltips'
import Switch from './components/ui/Switch'
import NumberInput from './components/ui/NumberInput'
import characterDataStatic from './data/character_data.json'
import './App.css'
import './styles/theme.css'
import './styles/Badges.css'
import './styles/Fonts.css'
import './styles/GlobalTooltips.css'
import ModularLogo from './components/ui/ModularLogo'
import HeroFilterDropdown from './components/HeroFilterDropdown'
import CustomDropdown from './components/CustomDropdown'
import ShortcutsHelpModal from './components/ShortcutsHelpModal'

// Utility functions
import { toTagArray } from './utils/tags'
import { detectHeroes } from './utils/heroes'
import { formatFileSize, normalizeModBaseName } from './utils/format'
import { getAdditionalCategories } from './utils/mods'

import TitleBar from './components/TitleBar'

// Lazy load heavy panels
const InstallModPanel = lazy(() => import('./components/InstallModPanel'))
const SettingsPanel = lazy(() => import('./components/SettingsPanel'))
const CreditsPanel = lazy(() => import('./components/CreditsPanel'))
const ToolsPanel = lazy(() => import('./components/ToolsPanel'))
const SharingPanel = lazy(() => import('./components/SharingPanel'))
const ClashPanel = lazy(() => import('./components/ClashPanel'))

function App() {
  const [globalUsmap, setGlobalUsmap] = useState('');
  const [hideSuffix, setHideSuffix] = useState(false);
  const [autoOpenDetails, setAutoOpenDetails] = useState(false);
  const [showHeroIcons, setShowHeroIcons] = useState(false);
  const [showHeroBg, setShowHeroBg] = useState(false);
  const [showModType, setShowModType] = useState(false);
  const [showExperimental, setShowExperimental] = useState(false);

  const [theme, setTheme] = useState('dark');
  const [accentColor, setAccentColor] = useState('#4a9eff');

  // Panel visibility state - grouped for cleaner management
  const [panels, setPanels] = useState({
    settings: false,
    tools: false,
    sharing: false,
    credits: false,
    install: false,
    clash: false,
    shortcuts: false
  });

  // Helper to open/close a specific panel
  const setPanel = (panelName, isOpen) => {
    setPanels(prev => ({ ...prev, [panelName]: isOpen }));
  };

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

  // Search state with debounce
  const [searchQuery, setSearchQuery] = useState('') // Actual filter query (debounced)
  const [localSearch, setLocalSearch] = useState('') // Input value (immediate)

  const debouncedSetSearch = useDebouncedCallback((value) => {
    setSearchQuery(value)
  }, 300)

  const handleSearchChange = (e) => {
    const value = e.target.value
    setLocalSearch(value)
    debouncedSetSearch(value)
  }

  const [expandedFolders, setExpandedFolders] = useState(new Set())
  const [modsToInstall, setModsToInstall] = useState([])
  const [installLogs, setInstallLogs] = useState([])
  const [modLoadingProgress, setModLoadingProgress] = useState(0) // 0-100 for progress, -1 for indeterminate
  const [isModsLoading, setIsModsLoading] = useState(false) // Track if mods are being loaded
  const [selectedFolderId, setSelectedFolderId] = useState('all')
  const [viewMode, setViewMode] = useState('list') // 'grid', 'compact', 'list'
  const [contextMenu, setContextMenu] = useState(null) // { x, y, mod }
  const [isLogDrawerOpen, setIsLogDrawerOpen] = useState(false)

  const [clashes, setClashes] = useState([])
  const [launchSuccess, setLaunchSuccess] = useState(false)
  const [characterData, setCharacterData] = useState(characterDataStatic)
  const [isDragging, setIsDragging] = useState(false)
  const [dropTargetFolder, setDropTargetFolder] = useState(null)
  const [renamingModPath, setRenamingModPath] = useState(null) // Track which mod should start inline renaming
  const [extensionModPath, setExtensionModPath] = useState(null) // Path of mod received from browser extension
  const [quickOrganizePaths, setQuickOrganizePaths] = useState(null) // Paths of PAKs to quick-organize (no uassets)
  const [newFolderPrompt, setNewFolderPrompt] = useState(null) // {paths: []} when prompting for new folder name
  const dropTargetFolderRef = useRef(null)
  const searchInputRef = useRef(null)
  const modsGridRef = useRef(null)
  const gameRunningRef = useRef(false)

  // Bulk delete state
  const [isDeletingBulk, setIsDeletingBulk] = useState(false)
  const deleteBulkTimeoutRef = useRef(null)

  // Alert system hook
  const alert = useAlert();

  // Global tooltips - replaces native browser tooltips with styled ones
  useGlobalTooltips();

  const handleCheckClashes = async () => {
    try {
      setStatus('Checking for clashes...')
      const result = await invoke('check_mod_clashes')
      setClashes(result)
      setPanel('clash', true)
      setStatus(`Found ${result.length} clashes`)
    } catch (error) {
      setStatus('Error checking clashes: ' + error)
    }
  }

  const handleCheckSingleModClashes = async (mod) => {
    try {
      setStatus(`Checking conflicts for ${mod.customName || mod.mod_name || 'mod'}...`)
      const conflicts = await invoke('check_single_mod_conflicts', { modPath: mod.path })

      // Transform SingleModConflict objects to ModClash format for the ClashPanel
      // Backend SingleModConflict: { conflicting_mod_path, conflicting_mod_name, overlapping_files, ... }
      // Frontend ModClash expected: { file_path, mod_paths: [path1, path2] }

      const fileMap = new Map() // file_path -> Set(mod_paths)

      if (conflicts && conflicts.length > 0) {
        conflicts.forEach(conflict => {
          conflict.overlapping_files.forEach(file => {
            if (!fileMap.has(file)) {
              fileMap.set(file, new Set())
            }
            // Add both the checked mod and the conflicting mod
            fileMap.get(file).add(mod.path)
            fileMap.get(file).add(conflict.conflicting_mod_path)
          })
        })
      }

      const transformedClashes = Array.from(fileMap.entries()).map(([file_path, modPathsSet]) => ({
        file_path,
        mod_paths: Array.from(modPathsSet)
      }))

      setClashes(transformedClashes)
      setPanel('clash', true)
      setStatus(`Found ${transformedClashes.length} conflicts for this mod`)
    } catch (error) {
      setStatus('Error checking mod conflicts: ' + error)
      console.error(error)
    }
  }

  const handleSetPriority = async (modPath, priority) => {
    if (gameRunning) {
      alert.warning(
        'Game Running',
        'Cannot change priority while game is running.'
      )
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
      if (panels.clash) {
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

  // Bulk Delete Handlers
  const handleBulkDelete = async () => {
    if (selectedMods.size === 0) return

    try {
      setStatus(`Deleting ${selectedMods.size} mods...`)
      setModLoadingProgress(-1)
      setIsModsLoading(true)

      const modPaths = Array.from(selectedMods)
      let deletedCount = 0
      let errors = []

      for (const path of modPaths) {
        try {
          // Find mod details to check if it's a folder mod or regular file
          // If we don't have details, try delete_mod anyway
          await invoke('delete_mod', { modPath: path })
          deletedCount++
        } catch (e) {
          console.error(`Failed to delete ${path}:`, e)
          errors.push(path)
        }
      }

      if (errors.length > 0) {
        alert.warning(
          'Bulk Delete Incomplete',
          `Deleted ${deletedCount} mods. Failed to delete ${errors.length} mods.`
        )
      } else {
        alert.success('Bulk Delete', `Successfully deleted ${deletedCount} mods.`)
      }

      setSelectedMods(new Set()) // Clear selection
      await loadMods()
      await loadFolders()

    } catch (e) {
      console.error('Bulk delete failed:', e)
      setStatus('Bulk delete failed: ' + e)
    } finally {
      setIsModsLoading(false)
      setModLoadingProgress(0)
    }
  }

  const handleBulkDeleteDown = (e) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDeletingBulk(true)
    deleteBulkTimeoutRef.current = setTimeout(() => {
      handleBulkDelete()
      setIsDeletingBulk(false)
    }, 2000)
  }

  const handleBulkDeleteUp = (e) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDeletingBulk(false)
    if (deleteBulkTimeoutRef.current) clearTimeout(deleteBulkTimeoutRef.current)
  }

  useEffect(() => {
    loadInitialData()
    loadTags()

    // Listen for install progress
    const unlisten = listen('install_progress', (event) => {
      const progress = Math.round(event.payload)
      setStatus(`Installing... ${progress}%`)
      setModLoadingProgress(progress)
      setIsModsLoading(true)
    })

    const unlistenComplete = listen('install_complete', () => {
      setStatus('Installation complete!')
      setIsModsLoading(false)
      setModLoadingProgress(0)
      loadMods()
    })

    const unlistenLogs = listen('install_log', (event) => {
      setInstallLogs(prev => [...prev, event.payload])
    })

    // Refresh mod list when character data is updated
    const unlistenCharUpdate = listen('character_data_updated', async () => {
      try {
        const data = await invoke('get_character_data')
        setCharacterData(data)
      } catch (err) {
        console.error('Failed to refresh character data:', err)
      }
      loadMods()
    })

    // Listen for directory changes (new folders, deleted folders, etc.)
    const unlistenDirChanged = listen('mods_dir_changed', () => {
      console.log('Directory changed, reloading mods and folders...')
      loadMods()
      loadFolders()
    })

    // Listen for mods received from browser extension via repakx:// protocol
    const unlistenExtensionMod = listen('extension-mod-received', (event) => {
      const filePath = event.payload
      console.log('Received mod from extension:', filePath)
      setExtensionModPath(filePath)
    })

    // Listen for extension mod errors
    const unlistenExtensionError = listen('extension-mod-error', (event) => {
      console.error('Extension mod error:', event.payload)
      alert.error('Extension Error', event.payload)
    })

    // Listen for general toast notifications from Rust backend
    const unlistenToast = listen('toast_notification', (event) => {
      const { type, title, description, duration } = event.payload

      // Map Rust type to AlertHandler method
      const showAlertByType = {
        'danger': () => alert.error(title, description, { duration: duration ?? 5000 }),
        'warning': () => alert.warning(title, description, { duration: duration ?? 5000 }),
        'success': () => alert.success(title, description, { duration: duration ?? 5000 }),
        'primary': () => alert.info(title, description, { duration: duration ?? 5000 }),
        'default': () => alert.showAlert({ color: 'default', title, description, duration: duration ?? 5000 })
      }

      const showFn = showAlertByType[type] || showAlertByType['default']
      showFn()
    })

    // Listen for game crash notifications
    const unlistenCrash = listen('game_crash_detected', (event) => {
      const payload = event.payload

      // Build enhanced description for crashes
      let enhancedDesc = payload.description
      if (payload.is_mesh_crash) {
        enhancedDesc += '\n\n💡 Tip: Try disabling "Fix Mesh" for this mod'
      }

      // Show persistent error toast for crashes
      alert.crash(payload.title, enhancedDesc, {
        action: {
          label: 'Details',
          onClick: () => {
            const report = [
              '--- CRASH REPORT ---',
              `Timestamp: ${new Date().toLocaleString()}`,
              `Type: ${payload.crash_type || 'Unknown'}`,
              `Error: ${payload.error_message || 'N/A'}`,
              `Asset: ${payload.asset_path || 'N/A'}`,
              `Is Mesh Crash: ${payload.is_mesh_crash ? 'Yes' : 'No'}`,
              '-------------------',
              'Full Details for dev debugging:',
              JSON.stringify(payload, null, 2),
              '-------------------'
            ]
            setInstallLogs(report)
            setIsLogDrawerOpen(true)
            alert.info('Crash Details', 'Report opened in Log Drawer.')
          }
        }
      })

      // Log detailed crash info to console for debugging
      console.error('Game Crash Detected:', {
        crashType: payload.crash_type,
        assetPath: payload.asset_path,
        details: payload.details,
        isMeshCrash: payload.is_mesh_crash,
        crashFolder: payload.crash_folder
      })
    })

    // Check for crashes from previous game sessions
    invoke('check_for_previous_crash').catch(err => {
      console.error('Failed to check for previous crashes:', err)
    })

    // Unified file drop handler function
    const handleFileDrop = async (paths) => {
      if (!paths || paths.length === 0) return
      console.log('Dropped items:', paths)

      // Check if we should quick-organize to a folder (using ref for current value in closure)
      const targetFolder = dropTargetFolderRef.current
      if (targetFolder) {
        // Special case: user dropped on "New Folder" target
        if (targetFolder === '__NEW_FOLDER__') {
          // Show the custom folder name prompt modal
          setNewFolderPrompt({ paths })
          setDropTargetFolder(null)
          return
        }

        // Check if any dropped items are folders with uassets that need proper processing
        try {
          const modsData = await invoke('parse_dropped_files', { paths })
          const hasFolderWithUassets = modsData.some(mod =>
            mod.is_dir === true && mod.contains_uassets !== false
          )

          if (hasFolderWithUassets) {
            // Cancel quick-organize and show alert
            setDropTargetFolder(null)
            alert.warning(
              'Cannot Quick-Organize Folder Mods',
              'Folder mods with UAssets need to be processed. Please drop them on the Install Mods area.',
              { duration: 8000 }
            )
            return
          }
        } catch (parseError) {
          console.error('Parse error during quick organize check:', parseError)
          // If parsing fails, we still try quick organize (might be simple PAK files)
        }

        // Quick organize: directly install to the folder without showing install panel
        console.log('Quick organizing to folder:', targetFolder)

        const pathCount = paths.length
        const pathsCopy = [...paths]
        const folderName = targetFolder

        setDropTargetFolder(null) // Reset for next drop

        // Start progress bar (indeterminate since quick_organize doesn't report progress)
        setIsModsLoading(true)
        setModLoadingProgress(-1)

        // Use promise toast for loading state and result
        alert.promise(
          (async () => {
            try {
              await invoke('quick_organize', { paths: pathsCopy, targetFolder: folderName })
              await loadMods()
              await loadFolders()
              setStatus(`Installed ${pathCount} item(s) to ${folderName}!`)

              // Show warning after success if game is running
              if (gameRunningRef.current) {
                alert.warning(
                  'Game Running',
                  'Mods installed, but changes will only take effect after restarting the game.',
                  { duration: 8000 }
                )
              }

              return { count: pathCount, folder: folderName }
            } finally {
              setIsModsLoading(false)
              setModLoadingProgress(0)
            }
          })(),
          {
            loading: {
              title: 'Quick Installing',
              description: `Copying ${pathCount} file${pathCount > 1 ? 's' : ''} to "${folderName}"...`
            },
            success: (result) => ({
              title: 'Installation Complete',
              description: `Installed ${result.count} mod${result.count > 1 ? 's' : ''} to "${result.folder}"`
            }),
            error: (err) => ({
              title: 'Installation Failed',
              description: String(err)
            })
          }
        )

        return
      }

      try {
        setStatus('Processing dropped items...')
        const modsData = await invoke('parse_dropped_files', { paths })
        if (!modsData || modsData.length === 0) {
          setStatus('No installable mods found in dropped items')
          return
        }
        console.log('Parsed mods:', modsData)

        // Check if ALL mods are PAK files with no uassets - if so, use quick organize
        const allPaksWithNoUassets = modsData.every(mod =>
          mod.is_dir === false && mod.contains_uassets === false
        )

        if (allPaksWithNoUassets && modsData.length > 0) {
          // Skip install panel, show quick organize folder picker
          console.log('All mods are PAKs with no uassets, using quick organize')
          setQuickOrganizePaths(paths)
          return
        }

        // Normal drop: show install panel
        setModsToInstall(modsData)
        setPanel('install', true)
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
      unlistenExtensionMod.then(f => f())
      unlistenExtensionError.then(f => f())
      unlistenToast.then(f => f())
      unlistenCrash.then(f => f())
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

  // Keep gameRunning ref in sync for event listener closures
  useEffect(() => {
    gameRunningRef.current = gameRunning
  }, [gameRunning])

  // Periodically check game running state every 5 seconds
  useEffect(() => {
    const intervalId = setInterval(() => {
      checkGame()
    }, 5000)

    return () => clearInterval(intervalId)
  }, [])

  const loadInitialData = async () => {
    try {
      const path = await invoke('get_game_path')
      setGamePath(path)

      const ver = await invoke('get_app_version')
      setVersion(ver)

      // Fetch character data from backend (up-to-date from GitHub sync)
      try {
        const charData = await invoke('get_character_data')
        setCharacterData(charData)
      } catch (charErr) {
        console.error('Failed to fetch character data:', charErr)
      }

      // Fetch stored USMAP path
      try {
        const usmapPath = await invoke('get_usmap_path')
        setGlobalUsmap(usmapPath)
      } catch (err) {
        console.error('Failed to fetch usmap path:', err)
      }

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
      setIsModsLoading(true)
      setModLoadingProgress(-1) // Indeterminate while fetching list
      setStatus('Loading mods...')

      const modList = await invoke('get_pak_files')
      console.log('Loaded mods:', modList)
      setMods(modList)
      setStatus(`Loading ${modList.length} mod(s) details...`)

      // After loading mods, refresh details for each (with progress tracking)
      await preloadModDetails(modList)

      setStatus(`Loaded ${modList.length} mod(s)`)
    } catch (error) {
      console.error('Error loading mods:', error)
      setStatus('Error loading mods: ' + error)
    } finally {
      setIsModsLoading(false)
      setModLoadingProgress(0)
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
        setModLoadingProgress(100)
        return
      }

      // Track progress as details are loaded
      let completedCount = 0
      const totalCount = pathsToFetch.length
      setModLoadingProgress(0)

      const results = await Promise.allSettled(
        pathsToFetch.map(async (p) => {
          const result = await invoke('get_mod_details', { modPath: p })
          completedCount++
          setModLoadingProgress(Math.round((completedCount / totalCount) * 100))
          return result
        })
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

      // Add single-character mods
      if (d.character_name && !d.character_name.startsWith('Multiple Heroes')) {
        charSet.add(d.character_name)
      }

      // For Multiple Heroes mods, extract individual character names from files
      if (typeof d.mod_type === 'string' && d.mod_type.startsWith('Multiple Heroes')) {
        hasMulti = true
        // Extract individual heroes from the mod's file list
        if (d.files && Array.isArray(d.files)) {
          const heroes = detectHeroes(d.files)
          heroes.forEach(h => charSet.add(h))
        }
      }
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
        setPanel('install', true)
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
    setPanel('install', true)
  }

  const handleDeleteMod = async (modPath) => {
    if (gameRunning) {
      alert.warning(
        'Game Running',
        'Cannot delete mods while game is running.'
      )
      return
    }
    // No confirmation prompt needed here, the hold-to-delete button handles the intent

    try {
      // Strip .bak_repak extension to get base path for proper deletion of all associated files
      const basePath = modPath.replace(/\.bak_repak$/i, '.pak')
      await invoke('delete_mod', { path: basePath })
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
      alert.warning(
        'Game Running',
        'Cannot toggle mods while game is running.'
      )
      return
    }
    try {
      const newState = await invoke('toggle_mod', { modPath })
      setStatus(newState ? 'Mod enabled' : 'Mod disabled')

      // Extract the base name (without extension) to find the mod after toggle
      // The path changes from .pak to .bak_repak or vice versa
      const baseName = modPath.replace(/\.(pak|bak_repak)$/i, '')

      await loadMods()

      // Update selectedMod if the toggled mod was selected
      // Find the mod by matching the base path (without extension)
      if (selectedMod && selectedMod.path === modPath) {
        // After loadMods, mods state is updated - find the matching mod
        setMods(prevMods => {
          const updatedMod = prevMods.find(m =>
            m.path.replace(/\.(pak|bak_repak)$/i, '') === baseName
          )
          if (updatedMod) {
            setSelectedMod(updatedMod)
          }
          return prevMods
        })
      }
    } catch (error) {
      setStatus('Error toggling mod: ' + error)
    }
  }

  const handleCreateFolder = () => {
    setNewFolderPrompt({ paths: [] })
  }

  // Create a folder and return its ID (for use by overlay components)
  const handleCreateFolderAndReturn = async (name) => {
    if (!name) throw new Error('Folder name is required')

    try {
      await invoke('create_folder', { name })
      await loadFolders()
      setStatus('Folder created')
      // The folder ID is just the folder name
      return name
    } catch (error) {
      setStatus('Error creating folder: ' + error)
      throw error
    }
  }

  // Handle new folder prompt confirmation (from drop zone or manual creation)
  const handleNewFolderConfirm = async (folderName) => {
    if (!newFolderPrompt) return

    const paths = newFolderPrompt.paths || []
    const pathCount = paths.length
    const pathsCopy = [...paths]

    // Check specific to quick organize flow
    const isQuickOrganize = pathCount > 0

    setNewFolderPrompt(null) // Close the modal

    // Start progress bar (indeterminate)
    // Only show full loading UI if we are doing a quick organize (installing files)
    // For simple folder creation, we can just use the status bar, or a lighter loader
    if (isQuickOrganize) {
      setIsModsLoading(true)
      setModLoadingProgress(-1)
    }

    // Use promise toast for loading state and result
    alert.promise(
      (async () => {
        try {
          // Create the folder first
          await invoke('create_folder', { name: folderName })
          await loadFolders()

          if (isQuickOrganize) {
            // Then quick organize to the new folder
            await invoke('quick_organize', { paths: pathsCopy, targetFolder: folderName })
            await loadMods()
            await loadFolders()
            setStatus(`Installed ${pathCount} item(s) to "${folderName}"!`)

            return { count: pathCount, folder: folderName, isInstall: true }
          } else {
            setStatus(`Folder "${folderName}" created`)
            return { folder: folderName, isInstall: false }
          }
        } finally {
          setIsModsLoading(false)
          setModLoadingProgress(0)
        }
      })(),
      {
        loading: {
          title: isQuickOrganize ? 'Creating Folder & Installing' : 'Creating Folder',
          description: isQuickOrganize
            ? `Creating "${folderName}" and copying ${pathCount} file${pathCount > 1 ? 's' : ''}...`
            : `Creating folder "${folderName}"...`
        },
        success: (result) => ({
          title: result.isInstall ? 'Installation Complete' : 'Folder Created',
          description: result.isInstall
            ? `Created folder and installed ${result.count} mod${result.count > 1 ? 's' : ''}`
            : `Successfully created "${result.folder}"`
        }),
        error: (err) => ({
          title: 'Operation Failed',
          description: String(err)
        })
      }
    )
  }

  const handleDeleteFolder = async (folderId) => {
    // No confirmation prompt needed here, the hold-to-delete button handles the intent

    try {
      await invoke('delete_folder', { id: folderId })
      if (selectedFolderId === folderId) {
        setSelectedFolderId('all')
      }

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
      alert.warning(
        'Game Running',
        'Cannot move mods while game is running.'
      )
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

    // Clear the mod details panel to prevent stale reference crashes
    setSelectedMod(null)

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
      alert.warning(
        'Game Running',
        'Cannot move mods while game is running.'
      )
      return
    }

    // Check if folderId corresponds to the root folder (depth 0)
    const targetFolder = folders.find(f => f.id === folderId)
    const effectiveFolderId = (targetFolder && targetFolder.depth === 0) ? null : folderId

    // Clear the mod details panel if the moved mod was selected
    if (selectedMod && selectedMod.path === modPath) {
      setSelectedMod(null)
    }

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

  // Rename a mod (calls backend to rename actual file)
  const handleRenameMod = async (modPath, newName) => {
    if (gameRunning) {
      alert.warning(
        'Game Running',
        'Cannot rename mods while game is running.'
      )
      return
    }

    try {
      // TODO: Implement rename_mod command in backend
      await invoke('rename_mod', { modPath, newName })
      setStatus(`Renamed to "${newName}"`)
      await loadMods()
    } catch (error) {
      // Fallback: if command doesn't exist yet, just show error
      setStatus(`Error renaming mod: ${error}`)
      console.error('rename_mod not implemented yet:', error)
    }
  }

  // Handle installing a mod received from the browser extension
  const handleExtensionModInstall = async (targetFolderId) => {
    if (!extensionModPath) return

    const modPath = extensionModPath // Copy path before clearing state

    // Close the overlay immediately
    setExtensionModPath(null)

    // Start progress bar (indeterminate)
    setIsModsLoading(true)
    setModLoadingProgress(-1)

    // Use promise toast for loading state and result
    alert.promise(
      (async () => {
        try {
          await invoke('quick_organize', {
            paths: [modPath],
            targetFolder: targetFolderId || null
          })

          await loadMods()
          await loadFolders()
          setStatus(`Mod installed successfully!`)

          // Show warning after success if game is running
          if (gameRunning) {
            alert.warning(
              'Game Running',
              'Mods installed, but changes will only take effect after restarting the game.',
              { duration: 8000 }
            )
          }

          return {}
        } finally {
          setIsModsLoading(false)
          setModLoadingProgress(0)
        }
      })(),
      {
        loading: {
          title: 'Installing from Extension',
          description: 'Copying mod file...'
        },
        success: () => ({
          title: 'Installation Complete',
          description: 'Mod installed successfully from browser extension'
        }),
        error: (err) => ({
          title: 'Installation Failed',
          description: String(err)
        })
      }
    )
  }

  // Handle quick organize for PAKs with no uassets (skips install panel)
  const handleQuickOrganizeInstall = async (targetFolderId) => {
    if (!quickOrganizePaths || quickOrganizePaths.length === 0) return

    const pathCount = quickOrganizePaths.length
    const pathsCopy = [...quickOrganizePaths] // Copy paths before clearing state

    // Close the overlay immediately
    setQuickOrganizePaths(null)

    // Start progress bar (indeterminate)
    setIsModsLoading(true)
    setModLoadingProgress(-1)

    // Use promise toast for loading state and result
    alert.promise(
      (async () => {
        try {
          await invoke('quick_organize', {
            paths: pathsCopy,
            targetFolder: targetFolderId || null
          })

          await loadMods()
          await loadFolders()
          setStatus(`${pathCount} PAK file(s) copied successfully!`)

          // Show warning after success if game is running
          if (gameRunning) {
            alert.warning(
              'Game Running',
              'Mods installed, but changes will only take effect after restarting the game.',
              { duration: 8000 }
            )
          }

          return { count: pathCount }
        } finally {
          setIsModsLoading(false)
          setModLoadingProgress(0)
        }
      })(),
      {
        loading: {
          title: 'Quick Installing',
          description: `Copying ${pathCount} PAK file${pathCount > 1 ? 's' : ''}...`
        },
        success: (result) => ({
          title: 'Installation Complete',
          description: `Successfully installed ${result.count} mod${result.count > 1 ? 's' : ''}`
        }),
        error: (err) => ({
          title: 'Installation Failed',
          description: String(err)
        })
      }
    )
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

  // Compute base filtered mods (excluding folder filter)
  const baseFilteredMods = mods.filter(mod => {
    // Hide LODs_Disabler mods from the list - they are controlled via Tools panel
    const modName = mod.mod_name || mod.custom_name || mod.path.split('\\').pop() || ''
    if (modName.toLowerCase().includes('lods_disabler') || mod.path.toLowerCase().includes('lods_disabler')) {
      return false
    }

    // Search query
    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      const displayName = (mod.custom_name || mod.path.split('\\').pop()).toLowerCase()
      if (!displayName.includes(query)) return false
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

  // Keyboard shortcuts handler (must be after filteredMods is defined)
  useEffect(() => {
    const handleKeyDown = (e) => {
      const key = e.key.toLowerCase()
      const ctrl = e.ctrlKey || e.metaKey
      const shift = e.shiftKey

      // Skip if typing in an input field (except Escape and Ctrl+F)
      const isInputActive = e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA'
      if (isInputActive && key !== 'escape' && !(ctrl && key === 'f')) return

      // F5 - Disable browser refresh
      if (key === 'f5') {
        e.preventDefault()
        return
      }
      // Ctrl+R - Full app reload (reset all states)
      else if (ctrl && key === 'r' && !shift) {
        e.preventDefault()
        alert.info('App Reloaded', 'Refreshed all data and resetted states.')
        loadInitialData()
      }
      // Ctrl+F - Focus search
      else if (ctrl && key === 'f') {
        e.preventDefault()
        searchInputRef.current?.focus()
      }
      // Ctrl+Shift+R - Refresh mods only
      else if (ctrl && shift && key === 'r') {
        e.preventDefault()
        alert.info('Mods Refreshed', 'Refreshed mods list.')
        loadMods()
      }
      // Ctrl+, - Settings
      else if (ctrl && key === ',') {
        e.preventDefault()
        setPanel('settings', true)
      }
      // Escape - Close panels or deselect
      else if (key === 'escape') {
        if (panels.shortcuts) setPanel('shortcuts', false)
        else if (panels.settings) setPanel('settings', false)
        else if (panels.tools) setPanel('tools', false)
        else if (panels.sharing) setPanel('sharing', false)
        else if (panels.install) setPanel('install', false)
        else if (panels.clash) setPanel('clash', false)
        else if (selectedMod) setSelectedMod(null)
      }
      // Ctrl+E - Toggle mod enabled/disabled
      else if (ctrl && key === 'e' && selectedMod) {
        e.preventDefault()
        handleToggleMod(selectedMod.path)
      }
      // F2 - Rename mod
      else if (key === 'f2' && selectedMod) {
        e.preventDefault()
        if (gameRunning) {
          alert.warning(
            'Game Running',
            'Cannot rename mods while game is running.'
          )
          return
        }
        setRenamingModPath(selectedMod.path)
      }
      // Enter - Open mod details
      else if (key === 'enter' && selectedMod && !isRightPanelOpen) {
        e.preventDefault()
        setLeftPanelWidth(lastPanelWidth > 60 ? lastPanelWidth : 70)
        setIsRightPanelOpen(true)
      }
      // Arrow navigation
      else if (['arrowup', 'arrowdown', 'arrowleft', 'arrowright'].includes(key)) {
        if (filteredMods.length === 0) return
        e.preventDefault()

        const currentIndex = selectedMod
          ? filteredMods.findIndex(m => m.path === selectedMod.path)
          : -1

        let newIndex = currentIndex

        if (viewMode === 'list' || viewMode === 'list-compact') {
          // List view: only up/down
          if (key === 'arrowup') newIndex = Math.max(0, currentIndex - 1)
          else if (key === 'arrowdown') newIndex = Math.min(filteredMods.length - 1, currentIndex + 1)
        } else {
          // Grid/Card view: all 4 directions
          // Calculate actual items per row by measuring the grid layout
          let itemsPerRow = 1
          const grid = modsGridRef.current
          if (grid) {
            const items = grid.querySelectorAll('.mod-card')
            if (items.length >= 2) {
              // Count how many items share the same top offset (are in the first row)
              const firstTop = items[0].offsetTop
              let count = 0
              for (const item of items) {
                if (item.offsetTop === firstTop) count++
                else break
              }
              itemsPerRow = Math.max(1, count)
            }
          }
          if (key === 'arrowup') newIndex = Math.max(0, currentIndex - itemsPerRow)
          else if (key === 'arrowdown') newIndex = Math.min(filteredMods.length - 1, currentIndex + itemsPerRow)
          else if (key === 'arrowleft') newIndex = Math.max(0, currentIndex - 1)
          else if (key === 'arrowright') newIndex = Math.min(filteredMods.length - 1, currentIndex + 1)
        }

        if (newIndex !== currentIndex && newIndex >= 0 && newIndex < filteredMods.length) {
          setSelectedMod(filteredMods[newIndex])
        } else if (currentIndex === -1 && filteredMods.length > 0) {
          setSelectedMod(filteredMods[0])
        }
      }
      // F1 - Show shortcuts help
      else if (key === 'f1') {
        e.preventDefault()
        setPanel('shortcuts', true)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [
    selectedMod, panels, viewMode,
    filteredMods, isRightPanelOpen, lastPanelWidth
  ])

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
    setPanel('install', false)
    setInstallLogs([])

    const modCount = modsWithSettings.length

    // Start progress bar (indeterminate until backend sends progress events)
    setIsModsLoading(true)
    setModLoadingProgress(-1)

    // Use promise toast for loading state and result
    // The backend spawns threads and returns immediately, so we need to wait
    // for the install_complete event to know when installation is actually done
    alert.promise(
      (async () => {
        // Create a promise that resolves when install_complete event fires
        const installCompletePromise = new Promise((resolve, reject) => {
          let unlistenComplete = null
          let unlistenError = null
          let timeoutId = null

          // Set a reasonable timeout (10 minutes for large mods)
          timeoutId = setTimeout(() => {
            if (unlistenComplete) unlistenComplete()
            if (unlistenError) unlistenError()
            reject(new Error('Installation timed out after 10 minutes'))
          }, 10 * 60 * 1000)

          // Listen for success
          listen('install_complete', () => {
            clearTimeout(timeoutId)
            if (unlistenComplete) unlistenComplete()
            if (unlistenError) unlistenError()
            resolve()
          }).then(unlisten => { unlistenComplete = unlisten })

          // Listen for failure (from toast_events via toast_notification)
          listen('toast_notification', (event) => {
            // Check if this is an installation failure toast
            if (event.payload?.title === 'Installation Failed') {
              clearTimeout(timeoutId)
              if (unlistenComplete) unlistenComplete()
              if (unlistenError) unlistenError()
              reject(new Error(event.payload?.description || 'Installation failed'))
            }
          }).then(unlisten => { unlistenError = unlisten })
        })

        // Start the installation (returns immediately since backend spawns threads)
        await invoke('install_mods', { mods: modsWithSettings })

        // Wait for the actual installation to complete
        await installCompletePromise

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

        await loadMods()
        await loadFolders()
        await loadTags()
        setStatus('Mods installed successfully!')

        // Show warning after success if game is running
        if (gameRunning) {
          alert.warning(
            'Game Running',
            'Mods installed, but changes will only take effect after restarting the game.',
            { duration: 8000 }
          )
        }

        return { count: modCount }
      })(),
      {
        loading: {
          title: 'Installing Mods',
          description: `Processing ${modCount} mod${modCount > 1 ? 's' : ''}...`
        },
        success: (result) => ({
          title: 'Installation Complete',
          description: `Successfully installed ${result.count} mod${result.count > 1 ? 's' : ''}`
        }),
        error: (err) => ({
          title: 'Installation Failed',
          description: String(err)
        })
      }
    )
  }

  const handleSaveSettings = (settings) => {
    setGlobalUsmap(settings.globalUsmap || '')
    setHideSuffix(settings.hideSuffix || false)
    setAutoOpenDetails(settings.autoOpenDetails || false)
    setShowHeroIcons(settings.showHeroIcons || false)
    setShowHeroBg(settings.showHeroBg || false)
    setShowModType(settings.showModType || false)
    setShowExperimental(settings.showExperimental || false)

    // Save to localStorage for persistence
    localStorage.setItem('hideSuffix', JSON.stringify(settings.hideSuffix || false))
    localStorage.setItem('autoOpenDetails', JSON.stringify(settings.autoOpenDetails || false))
    localStorage.setItem('showHeroIcons', JSON.stringify(settings.showHeroIcons || false))
    localStorage.setItem('showHeroBg', JSON.stringify(settings.showHeroBg || false))
    localStorage.setItem('showHeroBg', JSON.stringify(settings.showHeroBg || false))
    localStorage.setItem('showModType', JSON.stringify(settings.showModType || false))
    localStorage.setItem('showExperimental', JSON.stringify(settings.showExperimental || false))

    // Revert to normal list view if disabling experimental features while in compact list
    if (settings.showExperimental === false && viewMode === 'list-compact') {
      handleViewModeChange('list')
    }

    setStatus('Settings saved')
  }

  // Add this effect to initialize theme and view settings
  useEffect(() => {
    const savedTheme = localStorage.getItem('theme') || 'dark';
    const savedAccent = localStorage.getItem('accentColor') || '#4a9eff';
    const savedViewMode = localStorage.getItem('viewMode') || 'list';

    // Load Mods View Settings
    const savedHideSuffix = JSON.parse(localStorage.getItem('hideSuffix') || 'false');
    const savedAutoOpenDetails = JSON.parse(localStorage.getItem('autoOpenDetails') || 'false');
    const savedShowHeroIcons = JSON.parse(localStorage.getItem('showHeroIcons') || 'false');
    const savedShowHeroBg = JSON.parse(localStorage.getItem('showHeroBg') || 'false');
    const savedShowModType = JSON.parse(localStorage.getItem('showModType') || 'false');
    const savedShowExperimental = JSON.parse(localStorage.getItem('showExperimental') || 'false');

    handleThemeChange(savedTheme);
    handleAccentChange(savedAccent);
    setViewMode(savedViewMode);
    setHideSuffix(savedHideSuffix);
    setAutoOpenDetails(savedAutoOpenDetails);
    setShowHeroIcons(savedShowHeroIcons);
    setShowHeroBg(savedShowHeroBg);
    setShowModType(savedShowModType);
    setShowExperimental(savedShowExperimental);
  }, []);

  // Add these handlers
  const handleThemeChange = (newTheme) => {
    setTheme(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
    localStorage.setItem('theme', newTheme);
  };

  // 4-color palettes for aurora gradient animation
  const AURORA_PALETTES = {
    '#be1c1c': ['#be1c1c', '#ff9800', '#ffcc00', '#ff6b35'], // Repak Red: warm fire tones
    '#4a9eff': ['#4a9eff', '#a855f7', '#ff6b9d', '#38bdf8'], // Blue: cool to pink
    '#9c27b0': ['#9c27b0', '#e91e63', '#00bcd4', '#7c3aed'], // Purple: vibrant mix
    '#4CAF50': ['#4CAF50', '#8bc34a', '#00e676', '#e91e63'], // Green: nature with pop
    '#ff9800': ['#ff9800', '#ff5722', '#ffc107', '#4a9eff'], // Orange: sunset vibes
    '#FF96BC': ['#FF96BC', '#f472b6', '#c084fc', '#fda4af'], // Pink: soft pastel tones
  };

  const handleAccentChange = (newAccent) => {
    setAccentColor(newAccent);
    document.documentElement.style.setProperty('--accent-primary', newAccent);
    document.documentElement.style.setProperty('--accent-secondary', newAccent);
    // Set 4-color aurora palette for gradient animations
    const palette = AURORA_PALETTES[newAccent] || ['#be1c1c', '#ff9800', '#ffcc00', '#ff6b35'];
    document.documentElement.style.setProperty('--aurora-color-1', palette[0]);
    document.documentElement.style.setProperty('--aurora-color-2', palette[1]);
    document.documentElement.style.setProperty('--aurora-color-3', palette[2]);
    document.documentElement.style.setProperty('--aurora-color-4', palette[3]);
    localStorage.setItem('accentColor', newAccent);
  };

  const handleViewModeChange = (newMode) => {
    setViewMode(newMode);
    localStorage.setItem('viewMode', newMode);
  };

  // Remove static splash screen
  useEffect(() => {
    const splash = document.getElementById('splash-screen');
    if (splash) {
      splash.style.transition = 'opacity 0.4s ease-out';
      splash.style.opacity = '0';
      setTimeout(() => splash.remove(), 400);
    }
  }, []);

  return (
    <div className="app">
      <TitleBar />
      <Suspense fallback={null}>
        {panels.install && (
          <InstallModPanel
            mods={modsToInstall}
            allTags={allTags}
            folders={folders}
            onCreateTag={registerTagFromInstallPanel}
            onCreateFolder={handleCreateFolderAndReturn}
            onInstall={handleInstallMods}
            onCancel={() => setPanel('install', false)}
          />
        )}

        {panels.clash && (
          <ClashPanel
            clashes={clashes}
            mods={mods}
            onSetPriority={handleSetPriority}
            onClose={() => setPanel('clash', false)}
          />
        )}

        {panels.settings && (
          <SettingsPanel
            settings={{ globalUsmap, hideSuffix, autoOpenDetails, showHeroIcons, showHeroBg, showModType, showExperimental }}
            onSave={handleSaveSettings}
            onClose={() => setPanel('settings', false)}
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

        {panels.credits && (
          <CreditsPanel
            onClose={() => setPanel('credits', false)}
            version={version}
          />
        )}

        {panels.tools && (
          <ToolsPanel
            onClose={() => setPanel('tools', false)}
            mods={mods}
            onToggleMod={handleToggleMod}
          />
        )}

        {panels.sharing && (
          <SharingPanel
            onClose={() => setPanel('sharing', false)}
            gamePath={gamePath}
            installedMods={mods}
            selectedMods={selectedMods}
          />
        )}
      </Suspense>

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
        onNewFolderDrop={() => {
          // Special marker to indicate we should prompt for new folder on drop
          setDropTargetFolder('__NEW_FOLDER__')
        }}
        onClose={() => setIsDragging(false)}
        onCreateFolder={handleCreateFolderAndReturn}
      />

      {/* Extension Mod Overlay - for mods received from browser extension */}
      <ExtensionModOverlay
        isVisible={!!extensionModPath}
        filePath={extensionModPath}
        folders={folders}
        onInstall={handleExtensionModInstall}
        onCancel={() => setExtensionModPath(null)}
        onCreateFolder={handleCreateFolderAndReturn}
      />

      {/* Quick Organize Overlay - for PAK files with no uassets */}
      <QuickOrganizeOverlay
        isVisible={!!quickOrganizePaths && quickOrganizePaths.length > 0}
        paths={quickOrganizePaths || []}
        folders={folders}
        onInstall={handleQuickOrganizeInstall}
        onCancel={() => setQuickOrganizePaths(null)}
        onCreateFolder={handleCreateFolderAndReturn}
      />

      {/* New Folder Prompt Modal - for creating folders during drop */}
      <InputPromptModal
        isOpen={!!newFolderPrompt}
        title={newFolderPrompt?.paths?.length > 0 ? "Create Folder & Install" : "Create New Folder"}
        placeholder="Enter folder name..."
        confirmText={newFolderPrompt?.paths?.length > 0 ? "Create & Install" : "Create"}
        onConfirm={handleNewFolderConfirm}
        onCancel={() => {
          setNewFolderPrompt(null)
          setStatus('Folder creation cancelled')
        }}
      />


      <header className="header">
        <div
          className="header-branding"
          onClick={() => setPanel('credits', true)}
          title="View Credits"
        >
          <ModularLogo size={50} className="repak-icon" />
          <div className="header-title-group">
            <h1 className="font-logo">Repak <AuroraText className="font-logo">X</AuroraText> </h1> <h4>[DEV]</h4>
            <span className="version">v{version}</span>
          </div>
        </div>
        <div className="header-actions-right">
          <button
            className="btn-settings"
            title={gameRunning ? "Game is currently running" : "Launch Rivals"}
            style={{
              background: gameRunning
                ? 'rgba(255, 152, 0, 0.15)'
                : launchSuccess
                  ? 'rgba(76, 175, 80, 0.15)'
                  : 'rgba(74, 158, 255, 0.1)',
              color: gameRunning
                ? '#ff9800'
                : launchSuccess
                  ? '#4CAF50'
                  : '#4a9eff',
              border: gameRunning
                ? '1px solid rgba(255, 152, 0, 0.5)'
                : launchSuccess
                  ? '1px solid rgba(76, 175, 80, 0.5)'
                  : '1px solid rgba(74, 158, 255, 0.3)',
              cursor: gameRunning ? 'default' : 'pointer'
            }}
            onClick={async () => {
              if (gameRunning || launchSuccess) return
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
              {gameRunning ? (
                <motion.span
                  key="running"
                  className="launch-button-content"
                  initial={{ opacity: 0, scale: 0.5 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.5 }}
                >
                  <span className="blink-icon">⚠️</span> Game Running
                </motion.span>
              ) : launchSuccess ? (
                <motion.span
                  key="success"
                  className="launch-button-content"
                  initial={{ opacity: 0, scale: 0.5 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.5 }}
                >
                  <CheckIcon fontSize="small" /> Launched
                </motion.span>
              ) : (
                <motion.span
                  key="play"
                  className="launch-button-content"
                  initial={{ opacity: 0, scale: 0.5 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.5 }}
                >
                  <PlayArrowIcon fontSize="small" /> Launch Game
                </motion.span>
              )}
            </AnimatePresence>
          </button>

          <button
            onClick={() => setPanel('sharing', true)}
            className="btn-settings"
            title="Share Mods"
          >
            <IoMdWifi size={20} /> Share
          </button>
          <button
            onClick={() => setPanel('tools', true)}
            className="btn-settings"
            title="Tools"
          >
            <FaToolbox size={20} /> Tools
          </button>
          <button
            onClick={() => setPanel('settings', true)}
            className="btn-settings"
          >
            <IoIosSettings size={20} /> Settings
          </button>
        </div>
      </header>

      <div className="container">
        {/* Main Action Bar */}
        <div className="main-action-bar">
          <div className="search-wrapper">
            <SearchIcon className="search-icon-large" />
            <input
              ref={searchInputRef}
              type="text"
              placeholder="Search installed mods..."
              value={localSearch}
              onChange={handleSearchChange}
              className="main-search-input"
            />
            {localSearch && (
              <IconButton
                size="small"
                className="clear-search-btn"
                onClick={() => {
                  setLocalSearch('')
                  debouncedSetSearch('') // Clear immediately
                  setSearchQuery('')
                  searchInputRef.current?.focus()
                }}
              >
                <ClearIcon fontSize="small" />
              </IconButton>
            )}
          </div>

          <div className="action-controls">
            <CustomDropdown
              options={[{ value: '', label: 'View All' }, ...allTags]}
              value={filterTag}
              onChange={setFilterTag}
              placeholder="All Tags"
              icon={<FaTag style={{ fontSize: '1.2rem', opacity: 1, color: 'var(--accent-primary)' }} />}
            />

            <button onClick={handleInstallModClick} className="btn-install-large">
              <GrInstall className="install-icon" />
              <span className="install-text">Install Mod</span>
            </button>
          </div>
        </div>

        {!gamePath && (
          <div className="config-warning">
            ⚠️ Game path not configured. <button onClick={() => setPanel('settings', true)} className="btn-link-warning">Configure in Settings</button>
          </div>
        )}

        {/* Main 3-Panel Layout */}
        <div className="main-panels" onMouseMove={handleResizeMove}>
          {/* Wrapper for Left Sidebar and Center Panel */}
          <motion.div
            className="content-wrapper"
            animate={{ width: `${leftPanelWidth}%` }}
            transition={isResizing ? { duration: 0 } : { type: "tween", ease: "circOut", duration: 0.35 }}

          >
            {/* Left Sidebar - Folders */}
            <div className="left-sidebar">
              {/* Filters Section */}
              <div className="sidebar-filters">
                <div className="sidebar-filters-inner">
                  <div className="filter-title-row">
                    <div className="filter-label">FILTERS</div>
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
                  >
                    <div className="filter-label-secondary">Characters {selectedCharacters.size > 0 && `(${selectedCharacters.size})`}</div>
                    <span className="filter-chevron">{showCharacterFilters ? '\u25bc' : '\u25b6'}</span>
                  </div>
                  {showCharacterFilters && (
                    <HeroFilterDropdown
                      availableCharacters={availableCharacters}
                      selectedCharacters={selectedCharacters}
                      modDetails={modDetails}
                      onToggle={(target) => setSelectedCharacters(prev => {
                        const next = new Set(prev);

                        if (Array.isArray(target)) {
                          // Bulk toggle logic
                          const allSelected = target.every(t => next.has(t));
                          if (allSelected) {
                            // If all are selected, deselect all
                            target.forEach(t => next.delete(t));
                          } else {
                            // Otherwise, select all (add missing ones)
                            target.forEach(t => next.add(t));
                          }
                        } else {
                          // Single toggle logic
                          next.has(target) ? next.delete(target) : next.add(target);
                        }

                        return next;
                      })}
                    />
                  )}

                  {/* Category Chips */}
                  <div
                    className="filter-section-header with-margin"
                    onClick={() => setShowTypeFilters(v => !v)}
                  >
                    <div className="filter-label-secondary">Types {selectedCategories.size > 0 && `(${selectedCategories.size})`}</div>
                    <span className="filter-chevron">{showTypeFilters ? '\u25bc' : '\u25b6'}</span>
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
                <div className="sidebar-header-actions">
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
            <div className="center-panel">
              <div className="center-header">
                <div className="header-title">
                  <h2>
                    {selectedFolderId === 'all' ? 'All Mods' :
                      folders.find(f => f.id === selectedFolderId)?.name || 'Unknown Folder'}
                    <span className="mod-count">
                      ({filteredMods.filter(m => m.enabled).length} / {filteredMods.length} Enabled)
                    </span>
                  </h2>
                </div>
                <div className="header-actions">
                  <button onClick={handleCheckClashes} className="btn-ghost btn-check-conflicts" title="Check for conflicts">
                    <IoMdWarning className="warning-icon" style={{ color: 'var(--accent-primary)', width: '18px', height: '18px' }} /> Check Conflicts
                  </button>
                  <div className="divider-vertical" />
                  <div className="view-switcher">
                    <button
                      onClick={() => handleViewModeChange('grid')}
                      className={`btn-icon-small ${viewMode === 'grid' ? 'active' : ''}`}
                      title="Grid View"
                    >
                      <GridViewIcon fontSize="small" />
                    </button>
                    <button
                      onClick={() => handleViewModeChange('compact')}
                      className={`btn-icon-small ${viewMode === 'compact' ? 'active' : ''}`}
                      title="Compact Grid View"
                    >
                      <ViewModuleIcon fontSize="small" />
                    </button>
                    <button
                      onClick={() => handleViewModeChange('list')}
                      className={`btn-icon-small ${viewMode === 'list' ? 'active' : ''}`}
                      title="List View"
                    >
                      <ViewListIcon fontSize="small" />
                    </button>
                    {showExperimental && (
                      <button
                        onClick={() => handleViewModeChange('list-compact')}
                        className={`btn-icon-small ${viewMode === 'list-compact' ? 'active' : ''}`}
                        title="Compact List View (Experimental)"
                      >
                        <ViewHeadlineIcon fontSize="small" />
                      </button>
                    )}
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
                  <button onClick={loadMods} className="btn-ghost" title="Refresh list">
                    <RefreshIcon fontSize="small" />
                  </button>
                </div>
              </div>

              {/* Bulk Actions Toolbar */}
              <div className={`bulk-actions-toolbar ${selectedMods.size === 0 ? 'inactive' : ''}`}>
                <div className="selection-info" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <span>{selectedMods.size} selected</span>
                  <button onClick={() => {
                    const allPaths = filteredMods.map(m => m.path)
                    setSelectedMods(new Set(allPaths))
                  }} className="btn-ghost" style={{ padding: '4px 12px', height: '32px' }}>Select All</button>
                  <button onClick={handleDeselectAll} className="btn-ghost" style={{ padding: '4px 12px', height: '32px' }}>Clear</button>
                </div>
                <div className="bulk-controls">
                  <div style={{ width: '200px', height: '40px' }}>
                    <CustomDropdown
                      icon={<MdDriveFileMoveOutline style={{ fontSize: '1.2rem', opacity: 0.7 }} />}
                      options={[
                        { value: 'root', label: 'Root (~mods)' }, // Option to move back to root
                        ...folders.filter(f => f.name !== '~mods').map(f => ({ value: f.id, label: f.name }))
                      ]}
                      value="" // Always reset after selection locally handled by onChange logic below
                      onChange={(val) => {
                        if (!val) return
                        if (val === 'root') handleAssignToFolder(null) // Handle move to root
                        else handleAssignToFolder(val)
                      }}
                      placeholder="Move to..."
                      disabled={selectedMods.size === 0}
                    />
                  </div>

                  <div
                    className={`btn-ghost danger ${isDeletingBulk ? 'holding' : ''}`}
                    onMouseDown={handleBulkDeleteDown}
                    onMouseUp={handleBulkDeleteUp}
                    onMouseLeave={handleBulkDeleteUp}
                    style={{ marginLeft: '1rem', height: '40px' }}
                    title="Hold 2s to delete selected mods"
                  >
                    <div className="danger-bg" />
                    <span style={{ position: 'relative', zIndex: 2, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                      <RiDeleteBin2Fill />
                      {`Delete (${selectedMods.size})`}
                    </span>
                  </div>
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
                hideSuffix={hideSuffix}
                showHeroIcons={showHeroIcons}
                showHeroBg={showHeroBg}
                showModType={showModType}
                modDetails={modDetails}
                characterData={characterData}
                onRename={handleRenameMod}
                onCheckConflicts={handleCheckSingleModClashes}
                renamingModPath={renamingModPath}
                onClearRenaming={() => setRenamingModPath(null)}
                gridRef={modsGridRef}
                gameRunning={gameRunning}
                onRenameBlocked={() => alert.warning(
                  'Game Running',
                  'Cannot rename mods while game is running.'
                )}
                onDeleteBlocked={() => alert.warning(
                  'Game Running',
                  'Cannot delete mods while game is running.'
                )}
              />
            </div>
          </motion.div>

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
          >
            {selectedMod ? (
              <div className="mod-details-and-contents">
                <div className="mod-details-wrapper">
                  <ModDetailsPanel
                    mod={selectedMod}
                    initialDetails={modDetails[selectedMod.path]}
                    onClose={() => setSelectedMod(null)}
                    characterData={characterData}
                  />
                </div>

                <div className="selected-mod-contents">
                  <h3>Contents</h3>
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
        progress={modLoadingProgress}
        isLoading={isModsLoading}
        isOpen={isLogDrawerOpen}
        onToggle={() => setIsLogDrawerOpen(v => !v)}
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
            onRename={() => {
              if (contextMenu.mod) {
                if (gameRunning) {
                  alert.warning(
                    'Game Running',
                    'Cannot rename mods while game is running.'
                  )
                  return
                }
                setRenamingModPath(contextMenu.mod.path)
              }
            }}
            onCheckConflicts={() => contextMenu.mod && handleCheckSingleModClashes(contextMenu.mod)}
            allTags={allTags}
            gamePath={gamePath}
          />
        )
      }

      <ShortcutsHelpModal
        isOpen={panels.shortcuts}
        onClose={() => setPanel('shortcuts', false)}
      />
    </div >
  )
}

// Wrap App with AlertProvider
function AppWithAlerts() {
  return (
    <AlertProvider>
      <App />
    </AlertProvider>
  );
}

export default AppWithAlerts

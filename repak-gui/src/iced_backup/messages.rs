// Message types for the Iced application
// These represent all possible user interactions and events

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    // Application lifecycle
    Loaded,
    FontLoaded(Result<(), String>),
    
    // Search and filtering
    SearchChanged(String),
    SearchCleared,
    FilterToggled(String),
    CustomFilterToggled(String),
    TagFilterEnabled(bool),
    CustomTagFilterEnabled(bool),
    FiltersCleared,
    
    // Mod operations
    ModSelected(usize),
    ModToggled(usize),
    ModDeleted(usize),
    ModsDeleted(Vec<usize>),
    ModRenamed(usize, String),
    ModMoved(usize, Option<String>), // mod index, folder id
    
    // Selection mode
    SelectionModeToggled,
    ModSelectionToggled(usize),
    SelectAllMods,
    DeselectAllMods,
    
    // Folder operations
    FolderCreated(String),
    FolderDeleted(String),
    FolderRenamed(String, String),
    FolderToggled(String),
    FolderExpanded(String, bool),
    
    // Tag operations
    TagAdded(usize, String),
    TagRemoved(usize, String),
    BulkTagAdded(String),
    BulkTagRemoved(String),
    TagRenamed(String, String),
    TagDeleted(String),
    
    // File operations
    FileDropped(Vec<PathBuf>),
    FilesSelected(Vec<PathBuf>),
    GamePathChanged(String),
    GamePathBrowse,
    GamePathSet(PathBuf),
    
    // Settings window
    SettingsOpened,
    SettingsClosed,
    FontSizeChanged(f32),
    UIScaleChanged(f32),
    CompactModeToggled,
    CustomFontToggled,
    CustomPaletteToggled,
    ApplyPaletteInLightModeToggled,
    HideInternalSuffixToggled,
    ConfirmOnDeleteToggled,
    ShowTagChipsToggled,
    AutoCheckUpdatesToggled,
    ThemeChanged,
    UsmapFileSelected,
    UsmapFileCleared,
    
    // Palette editor
    PaletteEditorOpened,
    PaletteEditorClosed,
    PaletteColorChanged(PaletteColor, [u8; 4]),
    PaletteReset,
    PalettePresetSaved(String),
    PalettePresetLoaded(String),
    PalettePresetDeleted(String),
    
    // Tag manager
    TagManagerOpened,
    TagManagerClosed,
    NewTagInputChanged(String),
    NewTagCreated,
    
    // Update system
    UpdateCheckStarted,
    UpdateCheckCompleted(Result<UpdateInfo, String>),
    UpdateDownloadStarted,
    UpdateDownloadCompleted(Result<PathBuf, String>),
    UpdateInstalled,
    
    // Dialogs
    ConfirmationDialogOpened(ConfirmationDialog),
    ConfirmationDialogClosed(bool), // true if confirmed
    
    // Background tasks
    ModInstallStarted,
    ModInstallProgress(f32),
    ModInstallCompleted(Result<(), String>),
    FileWatcherEvent,
    
    // Error handling
    ErrorOccurred(String),
    ErrorDismissed,
    
    // Mod selection and details (ModSelected already defined above in Mod operations)
    SelectionCleared,
    FileInTableSelected(usize),
    
    // Mod installation
    FilesDropped(Vec<PathBuf>),
    ModsParsed(Vec<crate::install_mod_core::InstallableMod>),
    InstallModClicked,
    InstallDialogClosed,
    InstallConfirmed,
    InstallCancelled,
    InstallOptionChanged(InstallOption),
    InstallProgress(f32, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaletteColor {
    Accent,
    PanelFill,
    WindowFill,
    WidgetInactive,
    WidgetHovered,
    WidgetActive,
    WidgetOpen,
    Text,
    ToggleOnBg,
    ToggleOffBg,
    ToggleBorder,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationDialog {
    DeleteMod(usize),
    DeleteMods(Vec<usize>),
    DeleteFolder(String),
    RestartRequired,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallOption {
    FixMesh(bool),
    FixTexture(bool),
    ToIoStore(bool),
    Compression(String),
}

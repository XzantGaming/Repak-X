import React, { useState, useMemo, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { VscFolder, VscFolderOpened, VscChevronRight, VscChevronDown, VscClose, VscNewFolder, VscCheck } from 'react-icons/vsc';
import { MdContentCopy, MdCreateNewFolder } from 'react-icons/md';
import { BsFiletypeRaw } from 'react-icons/bs';
import './QuickOrganizeOverlay.css';

// Simplified folder tree (reusing logic from ExtensionModOverlay)
const buildTree = (folders) => {
    const root = { id: 'root', name: 'root', children: {}, isVirtual: true };
    const sortedFolders = [...folders].sort((a, b) => a.name.localeCompare(b.name));

    sortedFolders.forEach(folder => {
        const parts = folder.id.split(/[/\\]/);
        let current = root;

        parts.forEach((part, index) => {
            if (!current.children[part]) {
                current.children[part] = {
                    name: part,
                    children: {},
                    isVirtual: true,
                    fullPath: parts.slice(0, index + 1).join('/')
                };
            }
            current = current.children[part];

            if (index === parts.length - 1) {
                current.id = folder.id;
                current.isVirtual = false;
                current.originalName = folder.name;
            }
        });
    });

    return root;
};

const convertToArray = (node) => {
    if (!node.children) return [];
    const children = Object.values(node.children).map(child => ({
        ...child,
        children: convertToArray(child)
    }));
    children.sort((a, b) => a.name.localeCompare(b.name));
    return children;
};

// Folder node component
const FolderNode = ({ node, selectedFolderId, onSelect, depth = 0 }) => {
    const [isOpen, setIsOpen] = useState(true);
    const hasChildren = node.children && node.children.length > 0;
    const isSelected = selectedFolderId === node.id;

    const handleClick = (e) => {
        e.stopPropagation();
        if (!node.isVirtual) {
            onSelect(node.id);
        } else {
            setIsOpen(!isOpen);
        }
    };

    return (
        <div className="qo-folder-node">
            <div
                className={`qo-folder-item ${isSelected ? 'selected' : ''} ${node.isVirtual ? 'virtual' : ''}`}
                onClick={handleClick}
                style={{ paddingLeft: `${depth * 16 + 8}px` }}
            >
                <span className="folder-toggle" onClick={(e) => { e.stopPropagation(); setIsOpen(!isOpen); }}>
                    {hasChildren ? (isOpen ? <VscChevronDown /> : <VscChevronRight />) : <span style={{ width: 16 }} />}
                </span>
                <span className="folder-icon">
                    {isSelected || isOpen ? <VscFolderOpened /> : <VscFolder />}
                </span>
                <span className="folder-name">{node.name}</span>
            </div>

            {hasChildren && isOpen && (
                <div className="qo-folder-children">
                    {node.children.map(child => (
                        <FolderNode
                            key={child.fullPath || child.id}
                            node={child}
                            selectedFolderId={selectedFolderId}
                            onSelect={onSelect}
                            depth={depth + 1}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};

const QuickOrganizeOverlay = ({
    isVisible,
    paths = [],
    folders = [],
    onInstall,
    onCancel,
    onCreateFolder
}) => {
    const [selectedFolderId, setSelectedFolderId] = useState(null);
    const [isInstalling, setIsInstalling] = useState(false);
    const [isCreatingFolder, setIsCreatingFolder] = useState(false);
    const folderTreeRef = useRef(null);

    const rootFolder = useMemo(() => folders.find(f => f.is_root), [folders]);
    const subfolders = useMemo(() => folders.filter(f => !f.is_root), [folders]);
    const treeData = useMemo(() => {
        const root = buildTree(subfolders);
        return convertToArray(root);
    }, [subfolders]);

    // Extract filenames from paths
    const fileNames = useMemo(() => {
        if (!paths || paths.length === 0) return [];
        return paths.map(p => {
            const parts = p.split(/[/\\]/);
            return parts[parts.length - 1];
        });
    }, [paths]);

    // Reset state when overlay becomes visible
    useEffect(() => {
        if (isVisible) {
            setSelectedFolderId(null);
            setIsInstalling(false);
        }
    }, [isVisible]);

    const handleInstall = async () => {
        if (isInstalling) return;

        setIsInstalling(true);
        try {
            await onInstall(selectedFolderId);
        } catch (err) {
            console.error('Quick organize failed:', err);
        } finally {
            setIsInstalling(false);
        }
    };

    const handleBackdropClick = (e) => {
        if (e.target === e.currentTarget) {
            onCancel();
        }
    };

    const [newFolderName, setNewFolderName] = useState('');
    const [isAskingFolderName, setIsAskingFolderName] = useState(false);

    const handleNewFolderClick = () => {
        setNewFolderName('');
        setIsAskingFolderName(true);
    };

    const handleCancelNewFolder = () => {
        setIsAskingFolderName(false);
        setNewFolderName('');
    };

    const handleConfirmNewFolder = async () => {
        if (!newFolderName || !newFolderName.trim()) return;

        setIsCreatingFolder(true);
        try {
            if (onCreateFolder) {
                const newFolderId = await onCreateFolder(newFolderName.trim());
                if (newFolderId) {
                    setSelectedFolderId(newFolderId);
                }
            }
            setIsAskingFolderName(false);
            setNewFolderName('');
        } catch (err) {
            console.error('Failed to create folder:', err);
        } finally {
            setIsCreatingFolder(false);
        }
    };

    return (
        <AnimatePresence>
            {isVisible && (
                <motion.div
                    className="quick-organize-overlay"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    onClick={handleBackdropClick}
                >
                    <motion.div
                        className="quick-organize-panel"
                        initial={{ y: 50, opacity: 0, scale: 0.95 }}
                        animate={{ y: 0, opacity: 1, scale: 1 }}
                        exit={{ y: 50, opacity: 0, scale: 0.95 }}
                        transition={{ duration: 0.25, ease: 'easeOut' }}
                    >
                        {/* Header */}
                        <div className="qo-header">
                            <div className="qo-icon">
                                <BsFiletypeRaw />
                            </div>
                            <div className="qo-title">
                                <h2>Quick Copy - No Processing Needed</h2>
                                <p className="file-info">
                                    {paths.length} PAK file{paths.length > 1 ? 's' : ''} (Audio/Config)
                                </p>
                            </div>
                            <button className="close-btn" onClick={onCancel}>
                                <VscClose />
                            </button>
                        </div>

                        {/* File List */}
                        {fileNames.length > 0 && (
                            <div className="qo-file-list">
                                {fileNames.slice(0, 5).map((name, idx) => (
                                    <div key={idx} className="qo-file-item">
                                        <MdContentCopy className="file-icon" />
                                        <span className="file-name">{name}</span>
                                    </div>
                                ))}
                                {fileNames.length > 5 && (
                                    <div className="qo-file-more">
                                        +{fileNames.length - 5} more file(s)
                                    </div>
                                )}
                            </div>
                        )}

                        {/* Content */}
                        <div className="qo-content">
                            <div className="folder-section">
                                <div className="section-header">
                                    <MdCreateNewFolder />
                                    <span>Choose destination folder</span>
                                    {isAskingFolderName ? (
                                        <div className="new-folder-input-container">
                                            <input
                                                type="text"
                                                value={newFolderName}
                                                onChange={(e) => setNewFolderName(e.target.value)}
                                                placeholder="Folder name"
                                                autoFocus
                                                onKeyDown={(e) => {
                                                    if (e.key === 'Enter') handleConfirmNewFolder();
                                                    if (e.key === 'Escape') handleCancelNewFolder();
                                                }}
                                                onClick={(e) => e.stopPropagation()}
                                            />
                                            <button className="btn-confirm-folder" onClick={handleConfirmNewFolder} title="Confirm">
                                                <VscCheck />
                                            </button>
                                            <button className="btn-cancel-folder" onClick={handleCancelNewFolder} title="Cancel">
                                                <VscClose />
                                            </button>
                                        </div>
                                    ) : (
                                        <button
                                            className="btn-new-folder"
                                            onClick={handleNewFolderClick}
                                            disabled={isCreatingFolder}
                                            title="Create new folder"
                                        >
                                            <VscNewFolder />
                                            {isCreatingFolder ? 'Creating...' : 'New Folder'}
                                        </button>
                                    )}
                                </div>

                                <div className="folder-tree-container" ref={folderTreeRef}>
                                    {/* Root folder */}
                                    {rootFolder && (
                                        <div
                                            className={`qo-folder-item root-item ${selectedFolderId === rootFolder.id ? 'selected' : ''}`}
                                            onClick={() => setSelectedFolderId(rootFolder.id)}
                                        >
                                            <span className="folder-icon"><VscFolderOpened /></span>
                                            <span className="folder-name">{rootFolder.name}</span>
                                        </div>
                                    )}

                                    {/* Subfolders */}
                                    <div className="qo-folder-tree">
                                        {treeData.map(node => (
                                            <FolderNode
                                                key={node.fullPath || node.id}
                                                node={node}
                                                selectedFolderId={selectedFolderId}
                                                onSelect={setSelectedFolderId}
                                            />
                                        ))}
                                    </div>
                                </div>

                                {selectedFolderId && (
                                    <div className="selected-hint">
                                        Copying to: <strong>{selectedFolderId}</strong>
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* Footer */}
                        <div className="qo-footer">
                            <button className="btn-cancel" onClick={onCancel}>
                                Cancel
                            </button>
                            <button
                                className={`btn-copy ${isInstalling ? 'loading' : ''}`}
                                onClick={handleInstall}
                                disabled={isInstalling}
                            >
                                {isInstalling ? 'Copying...' : `Copy ${paths.length} File${paths.length > 1 ? 's' : ''}`}
                            </button>
                        </div>
                    </motion.div>
                </motion.div>
            )}
        </AnimatePresence>
    );
};

export default QuickOrganizeOverlay;

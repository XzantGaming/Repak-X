import React, { useState, useMemo, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { motion, AnimatePresence } from 'framer-motion';
import { VscFolder, VscFolderOpened, VscChevronRight, VscChevronDown } from 'react-icons/vsc';
import { MdInstallDesktop, MdCreateNewFolder } from 'react-icons/md';
import './DropZoneOverlay.css';

// Simplified folder tree for the overlay
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

// Folder node with data attribute for position detection
const DropFolderNode = ({ node, selectedFolderId, onSelect, depth = 0 }) => {
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
        <div className="drop-folder-node">
            <div
                className={`drop-folder-item ${isSelected ? 'selected' : ''} ${node.isVirtual ? 'virtual' : ''}`}
                data-folder-id={node.isVirtual ? undefined : node.id}
                data-dropzone="folder"
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
                <div className="drop-folder-children">
                    {node.children.map(child => (
                        <DropFolderNode
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

const DropZoneOverlay = ({
    isVisible,
    folders = [],
    onInstallDrop,
    onQuickOrganizeDrop,
    onClose
}) => {
    const [hoveredZone, setHoveredZone] = useState(null); // 'install' | 'organize'
    const [selectedFolderId, setSelectedFolderId] = useState(null);
    const overlayRef = useRef(null);

    const rootFolder = useMemo(() => folders.find(f => f.is_root), [folders]);
    const subfolders = useMemo(() => folders.filter(f => !f.is_root), [folders]);
    const treeData = useMemo(() => {
        const root = buildTree(subfolders);
        return convertToArray(root);
    }, [subfolders]);

    // Reset state when overlay becomes visible
    useEffect(() => {
        if (isVisible) {
            setHoveredZone(null);
            setSelectedFolderId(null);
        }
    }, [isVisible]);

    // Listen to Tauri drag-over event for position-based detection
    useEffect(() => {
        if (!isVisible) return;

        const handleDragOver = (event) => {
            const position = event.payload?.position;
            if (!position) return;

            const { x, y } = position;

            // Find element at this position
            const element = document.elementFromPoint(x, y);
            if (!element) return;

            // Check if over install zone
            const installZone = element.closest('[data-dropzone="install"]');
            if (installZone) {
                setHoveredZone('install');
                setSelectedFolderId(null);
                onInstallDrop?.();
                return;
            }

            // Check if over a specific folder
            const folderItem = element.closest('[data-folder-id]');
            if (folderItem) {
                const folderId = folderItem.getAttribute('data-folder-id');
                setHoveredZone('organize');
                setSelectedFolderId(folderId);
                onQuickOrganizeDrop?.(folderId);
                return;
            }

            // Check if over organize zone (but not specific folder)
            const organizeZone = element.closest('[data-dropzone="organize"]');
            if (organizeZone) {
                setHoveredZone('organize');
                // Keep current folder selection if any
                if (selectedFolderId) {
                    onQuickOrganizeDrop?.(selectedFolderId);
                } else {
                    onInstallDrop?.();
                }
                return;
            }
        };

        const unlistenDragOver = listen('tauri://drag-over', handleDragOver);

        return () => {
            unlistenDragOver.then(f => f());
        };
    }, [isVisible, selectedFolderId, onInstallDrop, onQuickOrganizeDrop]);

    return (
        <AnimatePresence>
            {isVisible && (
                <motion.div
                    ref={overlayRef}
                    className="dropzone-overlay"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.2 }}
                >
                    <div className="dropzone-container">
                        {/* Install Zone */}
                        <motion.div
                            className={`dropzone-card install-zone ${hoveredZone === 'install' ? 'active' : ''}`}
                            data-dropzone="install"
                            initial={{ x: -50, opacity: 0 }}
                            animate={{ x: 0, opacity: 1 }}
                            transition={{ delay: 0.1 }}
                        >
                            <div className="zone-icon">
                                <MdInstallDesktop />
                            </div>
                            <h2>Install Mods</h2>
                            <p>Drop files here to open the install panel with full configuration options</p>
                            <div className="zone-hint">
                                Supports .pak, .zip, .rar, .7z, folders
                            </div>
                        </motion.div>

                        {/* Quick Organize Zone */}
                        <motion.div
                            className={`dropzone-card organize-zone ${hoveredZone === 'organize' ? 'active' : ''}`}
                            data-dropzone="organize"
                            initial={{ x: 50, opacity: 0 }}
                            animate={{ x: 0, opacity: 1 }}
                            transition={{ delay: 0.1 }}
                        >
                            <div className="zone-icon">
                                <MdCreateNewFolder />
                            </div>
                            <h2>Quick Organize</h2>
                            <p>Hover over a folder below, then drop to install there</p>

                            <div className="folder-tree-container">
                                {/* Root folder */}
                                {rootFolder && (
                                    <div
                                        className={`drop-folder-item root-item ${selectedFolderId === rootFolder.id ? 'selected' : ''}`}
                                        data-folder-id={rootFolder.id}
                                        data-dropzone="folder"
                                        onClick={() => setSelectedFolderId(rootFolder.id)}
                                    >
                                        <span className="folder-icon"><VscFolderOpened /></span>
                                        <span className="folder-name">{rootFolder.name}</span>
                                    </div>
                                )}

                                {/* Subfolders */}
                                <div className="drop-folder-tree">
                                    {treeData.map(node => (
                                        <DropFolderNode
                                            key={node.fullPath || node.id}
                                            node={node}
                                            selectedFolderId={selectedFolderId}
                                            onSelect={setSelectedFolderId}
                                        />
                                    ))}
                                </div>
                            </div>

                            {selectedFolderId && (
                                <div className="selected-folder-hint">
                                    Drop to install into: <strong>{selectedFolderId}</strong>
                                </div>
                            )}
                        </motion.div>
                    </div>
                </motion.div>
            )}
        </AnimatePresence>
    );
};

export default DropZoneOverlay;

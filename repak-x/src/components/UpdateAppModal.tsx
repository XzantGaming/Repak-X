import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { MdDownload, MdOpenInNew, MdClose } from 'react-icons/md';
import './UpdateAppModal.css';

type UpdateInfo = {
    latest?: string;
    url?: string;
    asset_url?: string;
    changelog?: string;
    [key: string]: any;
};

type UpdateDownloadProgress = {
    status?: string;
    percentage?: number;
};

type ParsedBlock =
    | { type: 'heading'; text: string }
    | { type: 'listItem'; text: string }
    | { type: 'text'; text: string };

function parseChangelog(raw: string): ParsedBlock[] {
    const blocks: ParsedBlock[] = [];
    for (const line of raw.split('\n')) {
        const trimmed = line.trim();
        if (!trimmed) continue;

        if (trimmed.startsWith('### ')) {
            blocks.push({ type: 'heading', text: trimmed.slice(4) });
        } else if (trimmed.startsWith('## ')) {
            continue;
        } else if (trimmed.startsWith('- ')) {
            blocks.push({ type: 'listItem', text: trimmed.slice(2) });
        } else {
            blocks.push({ type: 'text', text: trimmed });
        }
    }
    return blocks;
}

function renderInlineMarkdown(text: string): React.ReactNode {
    const parts = text.split(/(\*\*[^*]+\*\*|__[^_]+__|\*[^*\n]+\*)/g);
    return parts.map((part, index) => {
        if (part.startsWith('**') && part.endsWith('**') && part.length > 4) {
            return <strong key={index}>{part.slice(2, -2)}</strong>;
        }
        if (part.startsWith('__') && part.endsWith('__') && part.length > 4) {
            return <strong key={index}>{part.slice(2, -2)}</strong>;
        }
        if (part.startsWith('*') && part.endsWith('*') && part.length > 2) {
            return <em key={index}>{part.slice(1, -1)}</em>;
        }
        return <React.Fragment key={index}>{part}</React.Fragment>;
    });
}

function renderChangelogBlocks(raw: string): React.ReactNode {
    const blocks = parseChangelog(raw);
    const elements: React.ReactNode[] = [];
    let listItems: string[] = [];
    let key = 0;

    const flushList = () => {
        if (listItems.length === 0) return;
        elements.push(
            <ul key={key++} style={{ listStyle: 'none', paddingLeft: '1rem', margin: '0.25rem 0' }}>
                {listItems.map((item, index) => (
                    <li key={index} style={{ position: 'relative', padding: '0.2rem 0 0.2rem 0.75rem', color: 'rgba(255,255,255,0.75)' }}>
                        <span
                            style={{
                                position: 'absolute',
                                left: 0,
                                top: '0.85em',
                                width: '5px',
                                height: '5px',
                                borderRadius: '50%',
                                background: 'var(--accent-primary, #4a9eff)'
                            }}
                        />
                        {renderInlineMarkdown(item)}
                    </li>
                ))}
            </ul>
        );
        listItems = [];
    };

    for (const block of blocks) {
        if (block.type === 'listItem') {
            listItems.push(block.text);
            continue;
        }

        flushList();
        if (block.type === 'heading') {
            elements.push(
                <h3 key={key++} style={{ fontSize: '0.95rem', fontWeight: 700, color: '#fff', margin: '0.7rem 0 0.25rem 0' }}>
                    {renderInlineMarkdown(block.text)}
                </h3>
            );
        } else {
            elements.push(
                <p key={key++} style={{ fontSize: '0.9rem', lineHeight: 1.5, color: 'rgba(255,255,255,0.75)', margin: '0.25rem 0' }}>
                    {renderInlineMarkdown(block.text)}
                </p>
            );
        }
    }

    flushList();
    console.debug('[Updates] Parsed update modal changelog blocks', { count: blocks.length });
    return elements;
}

type UpdateAppModalProps = {
    isOpen: boolean;
    updateInfo: UpdateInfo | null;
    downloadProgress: UpdateDownloadProgress | null;
    downloadedPath: string | null;
    onDownload: () => void;
    onApply: () => void;
    onOpenReleasePage: (url: string) => void;
    onClose: () => void;
};

export default function UpdateAppModal({
    isOpen,
    updateInfo,
    downloadProgress,
    downloadedPath,
    onDownload,
    onApply,
    onOpenReleasePage,
    onClose
}: UpdateAppModalProps) {
    if (!isOpen || !updateInfo) return null;

    const isDownloading = downloadProgress?.status === 'downloading';
    const isReady = downloadProgress?.status === 'ready' || downloadedPath;
    const downloadPercent = downloadProgress?.percentage ?? 0;

    return (
        <AnimatePresence>
            <motion.div
                className="modal-overlay"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                onClick={onClose}
            >
                <motion.div
                    className="modal-content update-modal"
                    initial={{ scale: 0.9, opacity: 0 }}
                    animate={{ scale: 1, opacity: 1 }}
                    exit={{ scale: 0.9, opacity: 0 }}
                    onClick={(e: React.MouseEvent<HTMLDivElement>) => e.stopPropagation()}
                >
                    <div className="modal-header">
                        <h2>🎉 Update Available!</h2>
                        <button className="modal-close" onClick={onClose}>
                            <MdClose />
                        </button>
                    </div>

                    <div className="modal-body">
                        <p className="update-version">
                            Version <strong>{updateInfo.latest}</strong> is available
                        </p>

                        {updateInfo.changelog && !isDownloading && !isReady && (
                            <div className="update-changelog">
                                {renderChangelogBlocks(updateInfo.changelog)}
                            </div>
                        )}

                        {isDownloading && downloadProgress && (
                            <div className="download-progress">
                                <div className="progress-bar">
                                    <div
                                        className="progress-fill"
                                        style={{ width: `${downloadPercent}%` }}
                                    />
                                </div>
                                <span className="progress-text">
                                    {downloadPercent.toFixed(0)}%
                                </span>
                            </div>
                        )}

                        {isReady && (
                            <p className="update-ready">
                                ✅ Download complete! Click "Install & Restart" to apply the update.
                            </p>
                        )}
                    </div>

                    <div className="modal-footer">
                        {!isReady && !isDownloading && (
                            <>
                                <button
                                    className="btn-secondary"
                                    onClick={() => onOpenReleasePage(updateInfo.url || '')}
                                    disabled={!updateInfo.url}
                                >
                                    <MdOpenInNew /> View Release
                                </button>
                                {updateInfo.asset_url && (
                                    <button
                                        className="btn-primary"
                                        onClick={onDownload}
                                    >
                                        <MdDownload /> Download Update
                                    </button>
                                )}
                            </>
                        )}

                        {isReady && (
                            <button
                                className="btn-primary"
                                onClick={onApply}
                            >
                                Install & Restart
                            </button>
                        )}

                        <button className="btn-secondary" onClick={onClose}>
                            Later
                        </button>
                    </div>
                </motion.div>
            </motion.div>
        </AnimatePresence>
    );
}

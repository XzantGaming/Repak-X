import React, { useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { MdClose } from 'react-icons/md';
import './ChangelogModal.css';

type ChangelogModalProps = {
    isOpen: boolean;
    version: string;
    changelog: string;
    onClose: () => void;
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
            // Skip version headers — we already show it in the modal title
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
    // Handles markdown-style inline formatting inside normal lines
    // Supported: **bold**, __bold__, *italic*
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

function renderBlocks(blocks: ParsedBlock[]) {
    const elements: React.ReactNode[] = [];
    let currentList: string[] = [];
    let key = 0;

    const flushList = () => {
        if (currentList.length > 0) {
            elements.push(
                <ul key={key++} className="changelog-list">
                    {currentList.map((item, i) => (
                        <li key={i}>{renderInlineMarkdown(item)}</li>
                    ))}
                </ul>
            );
            currentList = [];
        }
    };

    for (const block of blocks) {
        if (block.type === 'listItem') {
            currentList.push(block.text);
        } else {
            flushList();
            if (block.type === 'heading') {
                elements.push(
                    <h3 key={key++} className="changelog-section-title">{renderInlineMarkdown(block.text)}</h3>
                );
            } else {
                elements.push(
                    <p key={key++} className="changelog-text">{renderInlineMarkdown(block.text)}</p>
                );
            }
        }
    }
    flushList();

    return elements;
}

export default function ChangelogModal({
    isOpen,
    version,
    changelog,
    onClose
}: ChangelogModalProps) {
    const renderedContent = useMemo(() => {
        if (!changelog) return null;
        const parsed = parseChangelog(changelog);
        console.debug('[ChangelogModal] Parsed changelog blocks', { count: parsed.length });
        return renderBlocks(parsed);
    }, [changelog]);

    if (!isOpen) return null;

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
                    className="modal-content changelog-modal"
                    initial={{ scale: 0.9, opacity: 0 }}
                    animate={{ scale: 1, opacity: 1 }}
                    exit={{ scale: 0.9, opacity: 0 }}
                    onClick={(e: React.MouseEvent<HTMLDivElement>) => e.stopPropagation()}
                >
                    <div className="modal-header">
                        <h2>📋 What's New in v{version}</h2>
                        <button className="modal-close" onClick={onClose}>
                            <MdClose />
                        </button>
                    </div>

                    <div className="modal-body">
                        {renderedContent ? (
                            <div className="changelog-content">
                                {renderedContent}
                            </div>
                        ) : (
                            <p className="changelog-empty">No changelog available for this version.</p>
                        )}
                    </div>

                    <div className="modal-footer">
                        <button className="btn-primary" onClick={onClose}>
                            Got it!
                        </button>
                    </div>
                </motion.div>
            </motion.div>
        </AnimatePresence>
    );
}

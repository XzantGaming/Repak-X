import { useState, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { CgMinimizeAlt } from "react-icons/cg";
import { CgMaximizeAlt } from "react-icons/cg";
import { MdMinimize } from "react-icons/md";
import { GrClose } from 'react-icons/gr';
import './TitleBar.css';
import ModularLogo from './ui/ModularLogo';

const TitleBar = () => {
    const [appWindow, setAppWindow] = useState(null);
    const [isMaximized, setIsMaximized] = useState(false);
    const [isFocused, setIsFocused] = useState(true);

    useEffect(() => {
        // Get the current window instance
        const win = getCurrentWindow();
        setAppWindow(win);

        // Check initial state
        win.isMaximized().then(setIsMaximized);

        // --- Event Listeners Placeholders ---

        // Listen for maximize/unmaximize
        // Note: Tauri v2 might require a different event listener setup or polling if direct events aren't exposed easily in frontend wrapper yet,
        // but typically we can listen to 'tauri://resize' or similar if needed. 
        // For now, we update state on button click, which is the primary trigger.
        // Real event listeners for state changes from OS triggers (like Win+Up) might need 'tauri://resize' listener.

        const checkMaximizedState = async () => {
            if (win) {
                const max = await win.isMaximized();
                setIsMaximized(max);
            }
        };

        // Poll for resize changes to update maximize icon accurately (e.g. if user uses Snap Layouts)
        const resizeInterval = setInterval(checkMaximizedState, 1000); // Poll every second as a fallback

        // Focus listeners
        const unlistenFocus = win.listen('tauri://focus', () => {
            // console.log('Window focused');
            setIsFocused(true);
        });

        const unlistenBlur = win.listen('tauri://blur', () => {
            // console.log('Window blurred');
            setIsFocused(false);
        });

        return () => {
            clearInterval(resizeInterval);
            unlistenFocus.then(f => f());
            unlistenBlur.then(f => f());
        };
    }, []);

    const handleMinimize = () => {
        appWindow?.minimize();
    };

    const handleMaximize = async () => {
        if (appWindow) {
            await appWindow.toggleMaximize();
            // Update state immediately after toggle
            const max = await appWindow.isMaximized();
            setIsMaximized(max);
        }
    };

    const handleClose = () => {
        appWindow?.close();
    };

    return (
        <div className={`titlebar ${isFocused ? 'focused' : 'blurred'}`}>
            <div className="titlebar-drag-region" data-tauri-drag-region>
                {/* Icon and Title */}
                <ModularLogo size={16} className="titlebar-icon" />
                <span className="titlebar-title">Repak X</span>
            </div>

            <div className="titlebar-controls">
                <button className="titlebar-button" onClick={handleMinimize} title="Minimize">
                    <MdMinimize />
                </button>
                <button className="titlebar-button" onClick={handleMaximize} title={isMaximized ? "Restore" : "Maximize"}>
                    {isMaximized ? <CgMinimizeAlt /> : <CgMaximizeAlt />}
                </button>
                <button className="titlebar-button close" onClick={handleClose} title="Close">
                    <GrClose />
                </button>
            </div>
        </div>
    );
};

export default TitleBar;

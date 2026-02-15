/**
 * Hero/character detection utilities
 */

import characterDataStatic from '../data/character_data.json';

/**
 * Detects hero names from a list of file paths
 * Uses regex patterns matching the backend logic for character IDs
 * 
 * @param {string[]} files - Array of file paths
 * @returns {string[]} Array of detected hero names
 */
export function detectHeroes(files) {
    const heroIds = new Set();

    // Regex patterns matching backend logic
    const pathRegex = /(?:Characters|Hero_ST|Hero)\/(\d{4})/;
    const filenameRegex = /[_/](10[1-6]\d)(\d{3})/;

    files.forEach(file => {
        // Check path first - primary detection method
        const pathMatch = file.match(pathRegex);
        if (pathMatch) {
            heroIds.add(pathMatch[1]);
            return; // Skip filename check to avoid false positives from shared assets
        }

        // Fallback: Check filename only if path didn't match
        const filename = file.split('/').pop() || '';
        if (!filename.toLowerCase().startsWith('mi_')) {
            const filenameMatch = filename.match(filenameRegex);
            if (filenameMatch) {
                heroIds.add(filenameMatch[1]);
            }
        }
    });

    // Map IDs to names
    const heroNames = new Set();
    heroIds.forEach(id => {
        const char = characterDataStatic.find(c => c.id === id);
        if (char) {
            heroNames.add(char.name);
        }
    });

    return Array.from(heroNames);
}

/**
 * Detects heroes with custom character data (for dynamic updates)
 * 
 * @param {string[]} files - Array of file paths
 * @param {Object[]} characterData - Character data array
 * @returns {string[]} Array of detected hero names
 */
export function detectHeroesWithData(files, characterData) {
    const heroIds = new Set();

    const pathRegex = /(?:Characters|Hero_ST|Hero)\/(\d{4})/;
    const filenameRegex = /[_/](10[1-6]\d)(\d{3})/;

    files.forEach(file => {
        const pathMatch = file.match(pathRegex);
        if (pathMatch) {
            heroIds.add(pathMatch[1]);
            return;
        }

        const filename = file.split('/').pop() || '';
        if (!filename.toLowerCase().startsWith('mi_')) {
            const filenameMatch = filename.match(filenameRegex);
            if (filenameMatch) {
                heroIds.add(filenameMatch[1]);
            }
        }
    });

    const heroNames = new Set();
    heroIds.forEach(id => {
        const char = characterData.find(c => c.id === id);
        if (char) {
            heroNames.add(char.name);
        }
    });

    return Array.from(heroNames);
}

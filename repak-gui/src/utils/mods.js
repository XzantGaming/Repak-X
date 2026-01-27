/**
 * Mod-related utility functions
 */

/**
 * Extracts additional categories from mod details
 * Categories can come from additional_categories array or mod_type string
 * 
 * @param {Object} details - Mod details object
 * @returns {string[]} Array of additional category strings
 */
export function getAdditionalCategories(details) {
    if (!details) return [];

    // Direct additional_categories array
    if (details.additional_categories && details.additional_categories.length > 0) {
        return details.additional_categories;
    }

    // Parse from mod_type string (e.g., "Skin [Blueprint, VFX]")
    if (typeof details.mod_type === 'string') {
        const match = details.mod_type.match(/\[(.*?)\]/);
        if (match && match[1]) {
            return match[1].split(',').map(s => s.trim());
        }
    }

    return [];
}

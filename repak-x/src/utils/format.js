/**
 * Formatting utility functions
 */

/**
 * Formats bytes to human-readable file size
 * @param {number} bytes - Size in bytes
 * @returns {string} Formatted size string (e.g., "1.5 MB")
 */
export function formatFileSize(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Generate a normalized mod filename with priority suffix
 * e.g. "My Cool Mod" with minNines=7 -> "My_Cool_Mod_9999999_P"
 * 
 * @param {string} name - Original mod name
 * @param {number} minNines - Number of 9s in priority suffix (default: 7)
 * @returns {string} Normalized filename
 */
export function normalizeModBaseName(name, minNines = 7) {
    // Clean the name: remove existing suffixes and extension
    let cleanName = name
        .replace(/\.pak$/i, '')           // Remove .pak extension
        .replace(/_\d+_P$/gi, '')         // Remove existing priority suffix
        .replace(/\s+/g, '_')             // Replace spaces with underscores
        .replace(/[^\w_-]/g, '')          // Remove special characters

    // Generate the priority suffix
    const nines = '9'.repeat(minNines);
    return `${cleanName}_${nines}_P`;
}

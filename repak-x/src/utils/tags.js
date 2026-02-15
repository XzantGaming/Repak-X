/**
 * Tag utility functions
 */

/**
 * Converts tags from various formats to a consistent array
 * @param {string|string[]|null|undefined} tags - Tags in any format
 * @returns {string[]} Array of tag strings
 */
export const toTagArray = (tags) => Array.isArray(tags) ? tags : (tags ? [tags] : []);

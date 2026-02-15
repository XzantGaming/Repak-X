import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const heroDir = path.join(__dirname, '..');
const idsFile = path.join(__dirname, 'herobase_ids.txt');

// Additional mappings for names that don't directly match after normalization
const aliases = {
  'spiderman': 'spidey',
  'emmafrost': 'frost',
  'phoenix': 'jean',
  'rocketraccoon': 'rocketracoon', // Handling the single 'c' in the filename
  'doctorstrange': 'drstrange',
  'misterfantastic': 'mrfantastic'
};

try {
  if (!fs.existsSync(idsFile)) {
    console.error('IDs file not found:', idsFile);
    process.exit(1);
  }

  const content = fs.readFileSync(idsFile, 'utf-8');
  const lines = content.split(/\r?\n/); // Handle both CRLF and LF

  console.log('Starting rename process...');

  let renameCount = 0;
  let missingCount = 0;

  lines.forEach(line => {
    if (!line.trim()) return;

    const parts = line.split(' - ');
    if (parts.length < 2) return;

    const id = parts[0].trim();
    const rawName = parts[1].trim();

    // Normalize: lower case, remove non-alphanumeric chars
    // e.g. "Cloak & Dagger" -> "cloakdagger"
    let normalized = rawName.toLowerCase().replace(/[^a-z0-9]/g, '');

    // Determine the expected filename
    let filenameStr = normalized;
    if (aliases[normalized]) {
      filenameStr = aliases[normalized];
    }

    const filename = `${filenameStr}.png`;
    const oldPath = path.join(heroDir, filename);
    const newPath = path.join(heroDir, `${id}.png`);

    // Check if the source file exists
    if (fs.existsSync(oldPath)) {
      // Check if target already exists to avoid overwriting or redundant ops
      if (fs.existsSync(newPath)) {
        console.log(`Target ${id}.png already exists. Skipping ${filename}.`);
      } else {
        fs.renameSync(oldPath, newPath);
        console.log(`✅ Renamed: ${filename} -> ${id}.png`);
        renameCount++;
      }
    } else {
      // If the file is already renamed, it won't be found under the old name
      if (fs.existsSync(newPath)) {
        // console.log(`ℹ️  ${rawName} already verified as ${id}.png`);
      } else {
        console.log(`❌ File not found for: ${rawName} (Expected: ${filename})`);
        missingCount++;
      }
    }
  });

  console.log('-----------------------------------');
  console.log(`Process complete.`);
  console.log(`Renamed: ${renameCount}`);
  console.log(`Missing/Skipped: ${missingCount}`);

} catch (error) {
  console.error('An error occurred:', error);
}

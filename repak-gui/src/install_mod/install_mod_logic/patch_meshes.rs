use colored::Colorize;
use log::{error, info, warn};
use path_slash::PathExt;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::path::PathBuf;
use uasset_mesh_patch_rivals::Logger;
use uasset_mesh_patch_rivals::PatchFixer;

struct PrintLogger;

impl Logger for PrintLogger {
    fn log<S: Into<String>>(&self, buf: S) {
        let s = Into::<String>::into(buf);
        info!("[Mesh Patcher] {}", s);
    }
}

/// Simple mesh patch wrapper - calls mesh_patch_with_source with no source directory
pub fn mesh_patch(paths: &mut Vec<PathBuf>, mod_dir: &PathBuf) -> Result<(), repak::Error> {
    mesh_patch_with_source(paths, mod_dir, None)
}

/// Mesh patch with optional source directory to check for existing patched_files marker.
/// This prevents double-patching skeletal meshes from cooked directory mods that were already patched.
pub fn mesh_patch_with_source(paths: &mut Vec<PathBuf>, mod_dir: &PathBuf, source_mod_dir: Option<&PathBuf>) -> Result<(), repak::Error> {
    let uasset_files = paths
        .iter()
        .filter(|p| {
            p.extension().and_then(|ext| ext.to_str()) == Some("uasset")
                && p.to_str().map_or(false, |s| s.to_lowercase().contains("meshes"))
        })
        .cloned()
        .collect::<Vec<PathBuf>>();

    let patched_cache_file = mod_dir.join("patched_files");
    info!("Patching files...");
    let file = OpenOptions::new()
        .read(true) // Allow reading
        .write(true) // Allow writing
        .create(true)
        .truncate(false) // Create the file if it doesn't exist
        .open(&patched_cache_file)?;

    // Read patched files from the working directory cache
    let mut patched_files = BufReader::new(&file)
        .lines()
        .filter_map(|l| l.ok())
        .collect::<Vec<_>>();
    
    // Also check for patched_files in the source mod directory (for cooked directory mods)
    // This prevents double-patching meshes that were already patched before
    if let Some(source_dir) = source_mod_dir {
        let source_patched_file = source_dir.join("patched_files");
        if source_patched_file.exists() {
            info!("Found existing patched_files marker in source mod directory: {:?}", source_patched_file);
            if let Ok(source_file) = File::open(&source_patched_file) {
                let source_patched: Vec<String> = BufReader::new(source_file)
                    .lines()
                    .filter_map(|l| l.ok())
                    .collect();
                info!("Loaded {} previously patched file entries from source mod", source_patched.len());
                patched_files.extend(source_patched);
            }
        }
    }

    let mut cache_writer = BufWriter::new(&file);

    paths.push(patched_cache_file);
    let print_logger = PrintLogger;
    let mut fixer = PatchFixer {
        logger: print_logger,
    };
    'outer: for uassetfile in &uasset_files {
        let mut sizes: Vec<i64> = vec![];
        let mut offsets: Vec<i64> = vec![];

        let Some(dir_path) = uassetfile.parent() else {
            warn!("Could not get parent directory for file: {:?}, skipping", uassetfile);
            continue 'outer;
        };
        let uexp_file = dir_path.join(
            uassetfile
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.replace(".uasset", ".uexp"))
                .unwrap_or_else(|| {
                    warn!("Could not convert filename to string: {:?}", uassetfile);
                    "unknown.uexp".to_string()
                }),
        );

        if !uexp_file.exists() {
            warn!("UEXP file does not exist: {:?}, skipping mesh patching for this file", uexp_file);
            continue 'outer;
        }

        let rel_uasset = match uassetfile
            .strip_prefix(mod_dir)
            .ok()
            .and_then(|p| p.to_slash())
        {
            Some(path) => path,
            None => {
                error!("File not in input directory or failed to convert to slash path: {:?}", uassetfile);
                continue 'outer;
            }
        };

        let rel_uexp = match uexp_file
            .strip_prefix(mod_dir)
            .ok()
            .and_then(|p| p.to_slash())
        {
            Some(path) => path,
            None => {
                error!("File not in input directory or failed to convert to slash path: {:?}", uexp_file);
                continue 'outer;
            }
        };

        for i in &patched_files {
            if i.as_str() == rel_uexp.as_ref() as &str || i.as_str() == rel_uasset.as_ref() as &str {
                info!(
                    "Skipping {} (File has already been patched before)",
                    i.yellow()
                );
                continue 'outer;
            }
        }

        // No backup files needed - we're working in a temp directory
        info!("Processing {}", uassetfile.to_str().unwrap_or("<invalid_path>").yellow());
        let mut rdr = BufReader::new(File::open(uassetfile.clone())?);
        let (exp_cnt, exp_offset) = fixer.read_uasset(&mut rdr)?;
        fixer.read_exports(&mut rdr, &mut sizes, &mut offsets, exp_offset, exp_cnt)?;

        let uasset_file_size = fs::metadata(uassetfile)?.len();
        let tmpfile = format!("{}.temp", uexp_file.to_str().unwrap_or("unknown"));

        drop(rdr);

        let mut r = BufReader::new(File::open(&uexp_file)?);
        let mut o = BufWriter::new(File::create(&tmpfile)?);

        let exp_rd = fixer.read_uexp(&mut r, uasset_file_size, uexp_file.to_str().unwrap_or("unknown"), &mut o, &offsets);
        match exp_rd {
            Ok(_) => {}
            Err(e) => match e.kind() {
                ErrorKind::InvalidData => {
                    error!("Invalid data error during mesh patching: {}", e.to_string());
                    fs::remove_file(&tmpfile).ok(); // Clean up temp file
                    continue 'outer;
                }
                ErrorKind::Other => {
                    fs::remove_file(&tmpfile)?;
                    continue 'outer;
                }
                _ => {
                    error!("Unexpected error during mesh patching: {}", e.to_string());
                    fs::remove_file(&tmpfile).ok(); // Clean up temp file
                    continue 'outer;
                }
            },
        }
        // fs::remove_file(&uexp_file)?;

        fs::copy(&tmpfile, &uexp_file)?;
        unsafe {
            fixer.clean_uasset(uassetfile.clone(), &sizes)?;
        }

        writeln!(&mut cache_writer, "{}", &rel_uasset)?;
        writeln!(&mut cache_writer, "{}", &rel_uexp)?;

        fs::remove_file(&tmpfile)?;
        cache_writer.flush()?;
    }

    info!("Done patching files!!");
    Ok(())
}
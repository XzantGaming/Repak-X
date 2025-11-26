use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use retoc::{action_manifest, ActionManifest, Config, FGuid};
use std::path::PathBuf;

// Simplified file entry for Iced version
#[derive(Clone, Debug)]
pub struct UtocFileEntry {
    pub file_path: String,
    pub bulkdata_chunks: usize,
    pub packagedata_chunks: usize,
}

pub fn read_utoc(utoc_path: &Path, _pak_reader: &repak::PakReader, _pak_path: &Path) -> Vec<UtocFileEntry> {
    let action_mn = ActionManifest::new(PathBuf::from(utoc_path));
    let mut config = Config {
        container_header_version_override: None,
        ..Default::default()
    };

    let aes_toc =
        retoc::AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74")
            .unwrap();

    config.aes_keys.insert(FGuid::default(), aes_toc.clone());
    let config = Arc::new(config);

    let ops = action_manifest(action_mn, config).expect("Failed to read utoc");
    let ret = ops.oplog.entries.iter().map(|entry| {
        let name = entry.packagestoreentry.packagename.clone();
        UtocFileEntry {
            file_path: name,
            bulkdata_chunks: entry.bulkdata.len(),
            packagedata_chunks: entry.packagedata.len(),
        }
    }).collect::<Vec<_>>();

    ret
}
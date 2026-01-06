use retoc::iostore::IoStore;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: dump_container_header <path_to_utoc>");
        std::process::exit(1);
    }
    
    let utoc_path = &args[1];
    println!("Loading IoStore from: {}", utoc_path);
    
    let iostore = IoStore::open(utoc_path)?;
    
    println!("\n=== Container Header Info ===");
    println!("Package count: {}", iostore.package_ids().count());
    
    for package_id in iostore.package_ids() {
        println!("\nPackage ID: 0x{:016X}", package_id.0);
        
        if let Some(store_entry) = iostore.package_store_entry(package_id) {
            println!("  export_bundles_size: {}", store_entry.export_bundles_size);
            println!("  export_bundle_count: {}", store_entry.export_bundle_count);
            println!("  imported_packages count: {}", store_entry.imported_packages.len());
            
            for (i, imported_pkg_id) in store_entry.imported_packages.iter().enumerate() {
                println!("    imported_packages[{}]: 0x{:016X}", i, imported_pkg_id.0);
            }
        }
    }
    
    Ok(())
}

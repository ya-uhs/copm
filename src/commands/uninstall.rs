use crate::config::copm_json::CopmJson;
use crate::config::lock::CopmLock;
use crate::error::CopmError;
use crate::installer;
use crate::paths;

pub fn run(package: &str, global: bool) -> Result<(), CopmError> {
    let lock_path = paths::copm_lock_path();
    let lock = CopmLock::load(&lock_path)?;

    let locked = lock.packages.iter().find(|p| p.name == package);
    let target_types = locked.map(|p| p.targets.clone()).unwrap_or_default();
    let installed_files = locked.map(|p| p.installed_files.clone()).unwrap_or_default();

    installer::uninstall_targets(package, &target_types, &installed_files, global)?;
    println!("Uninstalled {package}");

    // Update copm.json and copm.lock if they exist
    let copm_json_path = paths::copm_json_path();
    if !global && copm_json_path.exists() {
        let mut config = CopmJson::load(&copm_json_path)?;
        config.remove_dependency(package);
        config.save(&copm_json_path)?;

        if lock_path.exists() {
            let mut lock = CopmLock::load(&lock_path)?;
            lock.remove_package(package);
            lock.save(&lock_path)?;
        }
        println!("Updated copm.json and copm.lock");
    }

    Ok(())
}

use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::{consts::ZELLIJ_PLUGIN_PERMISSIONS_CACHE, data::PermissionType};

pub type GrantedPermission = HashMap<String, Vec<PermissionType>>;

#[derive(Default, Debug)]
pub struct PermissionCache {
    path: PathBuf,
    granted: GrantedPermission,
    pending: GrantedPermission,
}

impl PermissionCache {
    pub fn cache(&mut self, plugin_name: String, permissions: Vec<PermissionType>) {
        self.pending
            .insert(plugin_name.clone(), permissions.clone());
        self.granted.insert(plugin_name, permissions);
    }

    pub fn get_permissions(&self, plugin_name: String) -> Option<&Vec<PermissionType>> {
        self.granted.get(&plugin_name)
    }

    pub fn check_permissions(
        &self,
        plugin_name: String,
        permissions_to_check: &Vec<PermissionType>,
    ) -> bool {
        if let Some(target) = self.granted.get(&plugin_name) {
            let mut all_granted = true;
            for permission in permissions_to_check {
                if !target.contains(permission) {
                    all_granted = false;
                }
            }
            return all_granted;
        }

        false
    }

    pub fn from_path_or_default(cache_path: Option<PathBuf>) -> Self {
        let cache_path = cache_path.unwrap_or(ZELLIJ_PLUGIN_PERMISSIONS_CACHE.to_path_buf());

        let granted = match fs::read_to_string(cache_path.clone()) {
            Ok(raw_string) => PermissionCache::from_string(raw_string).unwrap_or_default(),
            Err(e) => {
                log::error!("Failed to read permission cache file: {}", e);
                GrantedPermission::default()
            },
        };

        PermissionCache {
            path: cache_path,
            granted,
            pending: GrantedPermission::default(),
        }
    }

    pub fn write_to_file(&mut self) -> std::io::Result<()> {
        // Lock a stable sibling: locking the cache inode would stop protecting it
        // after atomic replacement. Readers see either complete version.
        let mut lock_path = self.path.as_os_str().to_os_string();
        lock_path.push(".lock");
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(PathBuf::from(lock_path))?;
        lock.lock()?;
        let mut granted = match fs::read_to_string(&self.path) {
            Ok(raw) => Self::from_string(raw)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => GrantedPermission::default(),
            Err(e) => return Err(e),
        };
        // Never merge the object's old snapshot: it may contain since-revoked
        // grants. Each changed plugin replaces its complete permission set.
        granted.extend(self.pending.clone());
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        write!(file, "{}", Self::to_string(&granted))?;
        file.as_file().sync_all()?;
        file.persist(&self.path).map_err(|e| e.error)?;
        self.granted = granted;
        self.pending.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotfiles_stale_writers_preserve_distinct_plugin_grants() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("permissions.kdl");
        let mut first = PermissionCache::from_path_or_default(Some(path.clone()));
        let mut second = PermissionCache::from_path_or_default(Some(path.clone()));
        first.cache(
            "navigation".into(),
            vec![PermissionType::ReadApplicationState],
        );
        second.cache(
            "status".into(),
            vec![PermissionType::ChangeApplicationState],
        );
        first.write_to_file().unwrap();
        second.write_to_file().unwrap();
        let actual = PermissionCache::from_path_or_default(Some(path));
        assert_eq!(
            actual.get_permissions("navigation".into()),
            Some(&vec![PermissionType::ReadApplicationState])
        );
        assert_eq!(
            actual.get_permissions("status".into()),
            Some(&vec![PermissionType::ChangeApplicationState])
        );
    }

    #[test]
    fn dotfiles_stale_snapshots_do_not_restore_denied_or_removed_permissions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("permissions.kdl");
        let mut original = PermissionCache::from_path_or_default(Some(path.clone()));
        original.cache(
            "navigation".into(),
            vec![
                PermissionType::ReadApplicationState,
                PermissionType::ChangeApplicationState,
            ],
        );
        original.cache("status".into(), vec![PermissionType::ReadApplicationState]);
        original.write_to_file().unwrap();
        let mut stale = PermissionCache::from_path_or_default(Some(path.clone()));
        let mut change = PermissionCache::from_path_or_default(Some(path.clone()));
        change.cache(
            "navigation".into(),
            vec![PermissionType::ReadApplicationState],
        );
        change.cache("status".into(), vec![]);
        change.write_to_file().unwrap();
        stale.cache("other".into(), vec![PermissionType::ChangeApplicationState]);
        stale.write_to_file().unwrap();
        // Reusing a previously saved object must not replay its old updates.
        original.write_to_file().unwrap();
        let actual = PermissionCache::from_path_or_default(Some(path));
        assert_eq!(
            actual.get_permissions("navigation".into()),
            Some(&vec![PermissionType::ReadApplicationState])
        );
        assert_eq!(actual.get_permissions("status".into()), Some(&vec![]));
        assert!(actual.get_permissions("other".into()).is_some());
    }

    #[test]
    fn dotfiles_invalid_cache_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("permissions.kdl");
        let mut cache = PermissionCache::from_path_or_default(Some(path.clone()));
        cache.cache("navigation".into(), vec![]);
        fs::write(&path, "invalid { ").unwrap();
        assert_eq!(
            cache.write_to_file().unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "invalid { ");
        fs::remove_file(&path).unwrap();
        cache.write_to_file().unwrap();
        let actual = PermissionCache::from_path_or_default(Some(path));
        assert_eq!(actual.get_permissions("navigation".into()), Some(&vec![]));
    }

    #[test]
    fn dotfiles_process_writers_preserve_distinct_plugin_grants() {
        const CHILD: &str = "ZELLIJ_PERMISSION_CACHE_TEST_CHILD";
        const DIRECTORY: &str = "ZELLIJ_PERMISSION_CACHE_TEST_DIRECTORY";
        fn wait_for(path: &std::path::Path) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while !path.exists() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "waiting for {:?}",
                    path
                );
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        if let Ok(plugin) = std::env::var(CHILD) {
            let dir = PathBuf::from(std::env::var_os(DIRECTORY).unwrap());
            let mut cache =
                PermissionCache::from_path_or_default(Some(dir.join("permissions.kdl")));
            cache.cache(plugin.clone(), vec![PermissionType::ReadApplicationState]);
            fs::write(dir.join(format!("ready-{plugin}")), "").unwrap();
            wait_for(&dir.join("go"));
            cache.write_to_file().unwrap();
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let mut children = Vec::new();
        for plugin in ["navigation", "status"] {
            children.push(
                std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "input::permission::tests::dotfiles_process_writers_preserve_distinct_plugin_grants",
                        "--nocapture",
                    ])
                    .env(CHILD, plugin)
                    .env(DIRECTORY, dir.path())
                    .spawn()
                    .unwrap(),
            );
        }
        // Both processes must hold stale snapshots before either writes.
        for plugin in ["navigation", "status"] {
            wait_for(&dir.path().join(format!("ready-{plugin}")));
        }
        fs::write(dir.path().join("go"), "").unwrap();
        for child in &mut children {
            assert!(child.wait().unwrap().success());
        }
        let actual =
            PermissionCache::from_path_or_default(Some(dir.path().join("permissions.kdl")));
        for plugin in ["navigation", "status"] {
            assert_eq!(
                actual.get_permissions(plugin.into()),
                Some(&vec![PermissionType::ReadApplicationState])
            );
        }
    }

    #[test]
    fn dotfiles_open_readers_keep_complete_snapshot_during_replacement() {
        use std::io::Read;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("permissions.kdl");
        let mut cache = PermissionCache::from_path_or_default(Some(path.clone()));
        cache.cache(
            "navigation".into(),
            vec![PermissionType::ReadApplicationState],
        );
        cache.write_to_file().unwrap();
        let old = fs::read_to_string(&path).unwrap();
        let mut reader = std::fs::File::open(&path).unwrap();
        cache.cache(
            "status".into(),
            vec![PermissionType::ChangeApplicationState],
        );
        cache.write_to_file().unwrap();
        let mut observed = String::new();
        reader.read_to_string(&mut observed).unwrap();
        assert_eq!(observed, old);
        let actual = PermissionCache::from_path_or_default(Some(path));
        assert_eq!(
            actual.get_permissions("navigation".into()),
            Some(&vec![PermissionType::ReadApplicationState])
        );
        assert_eq!(
            actual.get_permissions("status".into()),
            Some(&vec![PermissionType::ChangeApplicationState])
        );
    }
}

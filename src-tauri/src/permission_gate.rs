use crate::error::{JarvisError, Result};
use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Read,
    Search,
    Write,
    Create,
    GitRead,
    GitWrite,
    Command,
    Quarantine,
}

#[derive(Clone, Default)]
pub struct PermissionGate {
    roots: Arc<RwLock<HashMap<String, PathBuf>>>,
    locked: Arc<RwLock<bool>>,
}

impl PermissionGate {
    pub fn replace_roots(&self, roots: impl IntoIterator<Item = (String, PathBuf)>) {
        *self.roots.write().expect("permission roots lock") = roots.into_iter().collect();
    }
    pub fn add_root(&self, id: String, root: PathBuf) {
        self.roots
            .write()
            .expect("permission roots lock")
            .insert(id, root);
    }
    pub fn revoke(&self, id: &str) {
        self.roots
            .write()
            .expect("permission roots lock")
            .remove(id);
    }
    pub fn set_locked(&self, locked: bool) {
        *self.locked.write().expect("lock state") = locked;
    }
    pub fn is_locked(&self) -> bool {
        *self.locked.read().expect("lock state")
    }

    pub fn validate_authorization_root(path: &Path) -> Result<PathBuf> {
        if !path.is_absolute() {
            return Err(JarvisError::Security(
                "project root must be absolute".into(),
            ));
        }
        let canonical = path.canonicalize().map_err(|_| {
            JarvisError::Security("selected project directory does not exist".into())
        })?;
        if !canonical.is_dir() {
            return Err(JarvisError::Security(
                "selected project must be a directory".into(),
            ));
        }
        let home = std::env::var_os("HOME").map(PathBuf::from);
        if home.as_deref() == Some(canonical.as_path()) {
            return Err(JarvisError::Security(
                "the home directory cannot be authorized as a project".into(),
            ));
        }
        if let Some(home) = home {
            for protected in [
                "Desktop",
                "Documents",
                "Downloads",
                "Library",
                "Pictures",
                "Movies",
                "Music",
            ] {
                if canonical == home.join(protected) {
                    return Err(JarvisError::Security(format!("the entire {protected} folder cannot be authorized; choose a specific project folder")));
                }
            }
        }
        Ok(canonical)
    }

    pub fn resolve(
        &self,
        project_id: &str,
        requested: &Path,
        operation: Operation,
    ) -> Result<PathBuf> {
        if self.is_locked() {
            return Err(JarvisError::Locked);
        }
        if operation == Operation::Quarantine {
            return Err(JarvisError::Security(
                "quarantine requires the dedicated confirmation workflow".into(),
            ));
        }
        if requested
            .components()
            .any(|c| matches!(c, Component::ParentDir))
        {
            return Err(JarvisError::Security(
                "parent traversal is forbidden".into(),
            ));
        }
        let roots = self.roots.read().expect("permission roots lock");
        let root = roots
            .get(project_id)
            .ok_or(JarvisError::UnauthorizedProject)?;
        let candidate = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            root.join(requested)
        };
        let canonical = if candidate.exists() {
            candidate
                .canonicalize()
                .map_err(|e| JarvisError::Security(e.to_string()))?
        } else {
            if !matches!(operation, Operation::Write | Operation::Create) {
                return Err(JarvisError::Security("target does not exist".into()));
            }
            let parent = candidate
                .parent()
                .ok_or_else(|| JarvisError::Security("target has no parent".into()))?;
            parent
                .canonicalize()
                .map_err(|e| JarvisError::Security(e.to_string()))?
                .join(
                    candidate
                        .file_name()
                        .ok_or_else(|| JarvisError::Security("invalid target".into()))?,
                )
        };
        if canonical != *root && !canonical.starts_with(root) {
            return Err(JarvisError::Security(
                "path escapes the authorized project root".into(),
            ));
        }
        self.reject_protected(&canonical, operation)?;
        Ok(canonical)
    }

    fn reject_protected(&self, path: &Path, _operation: Operation) -> Result<()> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let secret = name == ".env"
            || name.starts_with(".env.")
            || name.ends_with(".pem")
            || name.ends_with(".key")
            || name.contains("service-account")
            || name.contains("credentials")
            || name.contains("token-cache")
            || name == "id_rsa"
            || name == "id_ed25519";
        if secret {
            return Err(JarvisError::Security(
                "secret/protected file content cannot be read or placed in AI context".into(),
            ));
        }
        Ok(())
    }

    pub fn root(&self, project_id: &str) -> Result<PathBuf> {
        if self.is_locked() {
            return Err(JarvisError::Locked);
        }
        self.roots
            .read()
            .expect("permission roots lock")
            .get(project_id)
            .cloned()
            .ok_or(JarvisError::UnauthorizedProject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    fn fixture() -> (tempfile::TempDir, PermissionGate, String, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let authorized = temp.path().join("Users/test/AuthorizedProject");
        let private = temp.path().join("Users/test/PrivateFiles");
        fs::create_dir_all(&authorized).unwrap();
        fs::create_dir_all(&private).unwrap();
        fs::write(authorized.join("main.rs"), "safe").unwrap();
        fs::write(authorized.join(".env"), "SECRET=x").unwrap();
        fs::write(private.join("private.txt"), "private").unwrap();
        let gate = PermissionGate::default();
        gate.add_root("project".into(), authorized.canonicalize().unwrap());
        (temp, gate, "project".into(), authorized, private)
    }

    #[test]
    fn authorized_read_works() {
        let (_t, g, id, _, _) = fixture();
        assert!(g
            .resolve(&id, Path::new("main.rs"), Operation::Read)
            .is_ok());
    }
    #[test]
    fn traversal_is_blocked() {
        let (_t, g, id, _, _) = fixture();
        assert!(g
            .resolve(
                &id,
                Path::new("../PrivateFiles/private.txt"),
                Operation::Read
            )
            .is_err());
    }
    #[test]
    fn absolute_private_path_is_blocked() {
        let (_t, g, id, _, p) = fixture();
        assert!(g
            .resolve(&id, &p.join("private.txt"), Operation::Read)
            .is_err());
    }
    #[test]
    fn sibling_access_is_blocked() {
        let (_t, g, id, a, _) = fixture();
        assert!(g
            .resolve(&id, &a.join("../PrivateFiles/private.txt"), Operation::Read)
            .is_err());
    }
    #[test]
    fn secret_files_are_blocked() {
        let (_t, g, id, _, _) = fixture();
        assert!(g.resolve(&id, Path::new(".env"), Operation::Read).is_err());
    }
    #[test]
    fn private_keys_are_blocked() {
        let (_t, g, id, a, _) = fixture();
        fs::write(a.join("deploy.pem"), "private").unwrap();
        assert!(g
            .resolve(&id, Path::new("deploy.pem"), Operation::Read)
            .is_err());
    }
    #[test]
    fn safe_edit_target_works() {
        let (_t, g, id, _, _) = fixture();
        assert!(g
            .resolve(&id, Path::new("main.rs"), Operation::Write)
            .is_ok());
    }
    #[test]
    fn revoke_is_immediate() {
        let (_t, g, id, _, _) = fixture();
        g.revoke(&id);
        assert!(g
            .resolve(&id, Path::new("main.rs"), Operation::Read)
            .is_err());
    }
    #[test]
    fn lock_blocks_all_access() {
        let (_t, g, id, _, _) = fixture();
        g.set_locked(true);
        assert!(matches!(
            g.resolve(&id, Path::new("main.rs"), Operation::Read),
            Err(JarvisError::Locked)
        ));
    }
    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_blocked() {
        let (_t, g, id, a, p) = fixture();
        symlink(&p, a.join("escape")).unwrap();
        assert!(g
            .resolve(&id, Path::new("escape/private.txt"), Operation::Read)
            .is_err());
    }
    #[test]
    fn no_delete_operation_exists() {
        let (_t, g, id, _, _) = fixture();
        assert!(g
            .resolve(&id, Path::new("main.rs"), Operation::Quarantine)
            .is_err());
    }
    #[test]
    fn home_and_personal_folders_are_blocked_when_unauthorized() {
        let (_t, g, id, _, _) = fixture();
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            assert!(g.resolve(&id, &home, Operation::Read).is_err());
            for name in ["Desktop", "Documents", "Downloads", "Library"] {
                assert!(g.resolve(&id, &home.join(name), Operation::Read).is_err());
            }
        }
    }
}

//! Host directories as seen by a wasix guest.
//!
//! wasix resolves a preopen through its own virtual filesystem and has no read-only mount, and
//! its host filesystem follows symlinks out of the mounted directory. [`GrantFs`] closes both:
//! it serves one host directory, refuses every mutation when the grant is read-only, and refuses
//! any path whose real location is outside the directory.

use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use wasmer_wasix::virtual_fs::{
    FileOpener, FileSystem, FsError, Metadata, OpenOptions, OpenOptionsConfig, ReadDir, Result,
    VirtualFile, host_fs,
};

/// One granted host directory, mounted at the guest path the grant names.
#[derive(Debug)]
pub(crate) struct GrantFs {
    inner: host_fs::FileSystem,
    /// The directory's canonical host path; every resolved path must stay under it.
    root: PathBuf,
    read_only: bool,
}

impl GrantFs {
    pub(crate) fn new(
        handle: tokio::runtime::Handle,
        root: &Path,
        read_only: bool,
    ) -> Result<Arc<Self>> {
        let root = std::fs::canonicalize(root).map_err(|_| FsError::EntryNotFound)?;
        if !root.is_dir() {
            return Err(FsError::BaseNotDirectory);
        }
        let inner = host_fs::FileSystem::new(handle, &root)?;
        Ok(Arc::new(Self {
            inner,
            root,
            read_only,
        }))
    }

    /// Refuse `path` if following symlinks would leave the granted directory. The deepest
    /// existing ancestor is resolved, so a path that does not exist yet is judged by its parent.
    fn confine(&self, path: &Path) -> Result<()> {
        let mut host = self.root.clone();
        for component in path.components() {
            match component {
                Component::Normal(part) => host.push(part),
                Component::ParentDir => {
                    host.pop();
                    if !host.starts_with(&self.root) {
                        return Err(FsError::PermissionDenied);
                    }
                }
                _ => {}
            }
        }
        let mut existing = host.as_path();
        while !existing.exists() {
            match existing.parent() {
                Some(parent) => existing = parent,
                None => return Err(FsError::PermissionDenied),
            }
        }
        match std::fs::canonicalize(existing) {
            Ok(real) if real.starts_with(&self.root) => Ok(()),
            _ => Err(FsError::PermissionDenied),
        }
    }

    fn checked(&self, path: &Path) -> Result<()> {
        self.confine(path)
    }

    fn writable(&self, path: &Path) -> Result<()> {
        if self.read_only {
            return Err(FsError::PermissionDenied);
        }
        self.confine(path)
    }
}

impl FileOpener for GrantFs {
    fn open(
        &self,
        path: &Path,
        conf: &OpenOptionsConfig,
    ) -> Result<Box<dyn VirtualFile + Send + Sync + 'static>> {
        if conf.would_mutate() {
            self.writable(path)?;
        } else {
            self.checked(path)?;
        }
        self.inner
            .new_open_options()
            .options(conf.clone())
            .open(path)
    }
}

impl FileSystem for GrantFs {
    fn readlink(&self, path: &Path) -> Result<PathBuf> {
        self.checked(path)?;
        self.inner.readlink(path)
    }

    fn read_dir(&self, path: &Path) -> Result<ReadDir> {
        self.checked(path)?;
        self.inner.read_dir(path)
    }

    fn create_dir(&self, path: &Path) -> Result<()> {
        self.writable(path)?;
        self.inner.create_dir(path)
    }

    fn create_symlink(&self, _source: &Path, _target: &Path) -> Result<()> {
        Err(FsError::PermissionDenied)
    }

    fn hard_link(&self, _source: &Path, _target: &Path) -> Result<()> {
        Err(FsError::PermissionDenied)
    }

    fn remove_dir(&self, path: &Path) -> Result<()> {
        self.writable(path)?;
        self.inner.remove_dir(path)
    }

    fn rename<'a>(
        &'a self,
        from: &'a Path,
        to: &'a Path,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.writable(from)?;
            self.writable(to)?;
            self.inner.rename(from, to).await
        })
    }

    fn metadata(&self, path: &Path) -> Result<Metadata> {
        self.checked(path)?;
        self.inner.metadata(path)
    }

    fn symlink_metadata(&self, path: &Path) -> Result<Metadata> {
        self.checked(path)?;
        self.inner.symlink_metadata(path)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        self.writable(path)?;
        self.inner.remove_file(path)
    }

    fn new_open_options(&self) -> OpenOptions<'_> {
        OpenOptions::new(self)
    }
}

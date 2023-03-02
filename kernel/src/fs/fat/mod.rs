mod file;
mod io;

use core::{cell::SyncUnsafeCell, marker::PhantomData};

use device_cache::BLOCK_SIZE;
use errno::Errno;
pub use io::FatIO;

use alloc::{collections::BTreeMap, sync::Arc};
use id_alloc::*;
use kernel_sync::SpinLock;
use spin::Lazy;
use vfs::*;

use crate::{
    arch::timer::{get_time, get_time_ns, get_time_sec, get_time_us},
    config::FS_IMG_SIZE,
};

pub type FatTP = fatfs::DefaultTimeProvider;
pub type FatOCC = fatfs::LossyOemCpConverter;
type FatFile = fatfs::File<'static, FatIO, FatTP, FatOCC>;
type FatDir = fatfs::Dir<'static, FatIO, FatTP, FatOCC>;

/// Fat virtual file system.
pub struct FatVFS {
    /// fat file system
    pub fs: SpinLock<fatfs::FileSystem<FatIO, FatTP, FatOCC>>,

    /// Inode number mapped to [`FatFile`].
    pub files: SpinLock<BTreeMap<usize, FatFile>>,

    /// Inode number mapped to [`FatDir`].
    pub dirs: SpinLock<BTreeMap<usize, FatDir>>,

    /// VFS super block
    pub sb: SuperBlock<FatVFSImpl>,

    /// Inode number allocator (for fat filesystem without inodes)
    pub alloc: SpinLock<RecycleAllocator>,
}

static FAT_VFS: Lazy<FatVFS> = Lazy::new(|| {
    SpinLock::new(FatVFS {
        fs: SpinLock::new(
            fatfs::FileSystem::new(
                FatIO::new(),
                fatfs::FsOptions::new().update_accessed_date(true),
            )
            .unwrap(),
        ),
        files: SpinLock::new(BTreeMap::new()),
        dirs: SpinLock::new(BTreeMap::new()),
        sb: SuperBlock::mount("fat", BLOCK_SIZE, FS_IMG_SIZE, MountFlags::empty()),
        alloc: SpinLock::new(RecycleAllocator::new(0)),
    })
});

pub struct FatVFSImpl;

impl VFS for FatVFSImpl {
    fn alloc_inode() -> VFSResult<Arc<Inode<Self>>> {
        Ok(Arc::new(Inode {
            phantom: PhantomData,
            ino: FAT_VFS.alloc.lock().alloc(),
            block_size: BLOCK_SIZE,
            inner: SyncUnsafeCell::new(InodeMutInner {
                mode: InodeMode::empty(),
            }),
            locked: SpinLock::new(InodeLockedInner {
                file_size: 0,
                nlink: 1,
                state: InodeState::New,
                atime: TimeSpec {
                    sec: get_time_sec(),
                    nsec: get_time_ns(),
                },
                mtime: TimeSpec {
                    sec: get_time_sec(),
                    nsec: get_time_ns(),
                },
                ctime: TimeSpec {
                    sec: get_time_sec(),
                    nsec: get_time_ns(),
                },
            }),
        }))
    }

    fn destroy_inode(inode: &Inode<Self>) -> VFSResult {
        if inode.get_mode().contains(InodeMode::S_IFREG) {
            FAT_VFS.files.lock().remove(&inode.ino);
        } else {
            FAT_VFS.dirs.lock().remove(&inode.ino);
        }
        FAT_VFS.alloc.lock().dealloc(inode.ino);
        Ok(())
    }

    fn lookup(dir: &Path, name: &str) -> VFSResult<Arc<Inode<Self>>> {
        let fs = FAT_VFS.fs.lock();
        let root = fs.root_dir();
        if let Ok(file) =  root.open_file(name) {
            
        }
    }

    fn create(
        pdir: &Arc<Dentry<Self>>,
        name: &str,
        mode: InodeMode,
    ) -> VFSResult<Arc<Inode<Self>>> {
        if let Some(pdir) = pdir.get_inode() {
            if let Some(pdir) = FAT_VFS.dirs.lock().get(&pdir.ino) {
                let file = pdir.create_file(name).map_err(|err| io::from(err))?;
                let inode = Self::alloc_inode()?;
                inode.set_mode(mode);
                FAT_VFS.files.lock().insert(inode.ino, file).unwrap();
                return Ok(inode);
            }
        }
        Err(Errno::ENOTDIR)
    }

    fn mkdir(pdir: &Arc<Dentry<Self>>, name: &str, mode: InodeMode) -> VFSResult<Arc<Inode<Self>>> {
        if let Some(pdir) = pdir.get_inode() {
            let mut dirs = FAT_VFS.dirs.lock();
            if let Some(pdir) = dirs.get(&pdir.ino) {
                let file = pdir.create_dir(name).map_err(|err| io::from(err))?;
                let inode = Self::alloc_inode()?;
                inode.set_mode(mode);
                dirs.insert(inode.ino, file).unwrap();
                return Ok(inode);
            }
        }
        Err(Errno::ENOTDIR)
    }

    /// Fat file system does not support hard link, thus we only check the existance of the [`Inode`].
    fn unlink(pdir: &Dentry<Self>, dentry: &Dentry<Self>) -> VFSResult {
        let pdir = FAT_VFS
            .dirs
            .lock()
            .get(&pdir.get_inode().unwrap().ino)
            .unwrap();
        pdir.remove(&dentry.name).map_err(|err| io::from(err))?;
        Err(Errno::ENOTDIR)
    }
}

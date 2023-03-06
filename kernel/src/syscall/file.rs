use alloc::string::String;
use core::mem::size_of;
use errno::Errno;
use log::trace;
use syscall_interface::*;
use vfs::{OpenFlags, Path, SeekWhence, StatMode};

use crate::{
    arch::mm::VirtAddr,
    error::KernelResult,
    fs::{open, unlink},
    task::{curr_task, Task},
};

use super::SyscallImpl;

/// Resolves absolute path with directory file descriptor and pathname.
///
/// If the pathname is relative, then it is interpreted relative to the directory
/// referred to by the file descriptor dirfd .
///
/// If pathname is relative and dirfd is the special value [`AT_FDCWD`], then pathname
/// is interpreted relative to the current working directory of the calling process.
///
/// If pathname is absolute, then dirfd is ignored.
pub fn resolve_path(task: &Task, dirfd: usize, pathname: String) -> KernelResult<Path> {
    if pathname.starts_with("/") {
        Ok(Path::new(pathname.as_str()))
    } else {
        let mut path = task.get_dir(dirfd)?;
        path.extend(pathname.as_str());
        Ok(path)
    }
}

impl SyscallFile for SyscallImpl {
    fn write(fd: usize, buf: *const u8, count: usize) -> SyscallResult {
        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();

        // Translate user buffer into kernel string.
        let buf = curr_mm
            .get_buf_mut(VirtAddr::from(buf as usize), count)
            .map_err(|_| Errno::EFAULT)?;
        drop(curr_mm);

        // Get the file with the given file descriptor.
        let file = curr.fd_manager.lock().get(fd).map_err(|_| Errno::EBADF)?;
        drop(curr);

        let mut write_len = 0;
        for bytes in buf.inner {
            if let Some(count) = file.write(bytes) {
                write_len += count;
            } else {
                break;
            }
        }
        Ok(write_len)
    }

    fn read(fd: usize, buf: *mut u8, count: usize) -> SyscallResult {
        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();

        // Get the real buffer translated into physical address.
        let buf = curr_mm
            .get_buf_mut(VirtAddr::from(buf as usize), count)
            .map_err(|_| Errno::EFAULT)?;
        drop(curr_mm);

        // Get the file with the given file descriptor.
        let file = curr.fd_manager.lock().get(fd).map_err(|_| Errno::EBADF)?;
        drop(curr);

        let mut read_len = 0;
        for bytes in buf.inner {
            if let Some(count) = file.read(bytes) {
                read_len += count;
            } else {
                break;
            }
        }
        Ok(read_len)
    }

    fn close(fd: usize) -> SyscallResult {
        let curr = curr_task().unwrap();
        curr.fd_manager
            .lock()
            .remove(fd)
            .map_err(|err| Errno::from(err))?;
        Ok(0)
    }

    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult {
        let flags = OpenFlags::from_bits(flags as u32);
        let mode = StatMode::from_bits(mode as u32);
        if flags.is_none() {
            return Err(Errno::EINVAL);
        }

        let curr = curr_task().unwrap();
        let flags = flags.unwrap();

        if flags.contains(OpenFlags::O_CREAT) && mode.is_none()
            || flags.contains(OpenFlags::O_WRONLY | OpenFlags::O_RDWR)
        {
            return Err(Errno::EINVAL);
        }

        let mut mm = curr.mm.lock();
        let path = resolve_path(&curr, dirfd, mm.get_str(VirtAddr::from(pathname as usize))?)?;

        trace!("OPEN {:?} {:?}", path, flags);

        let mut fd_manager = curr.fd_manager.lock();
        fd_manager
            .push(open(path, flags)?)
            .map_err(|err| err.into())
    }

    fn lseek(fd: usize, off: usize, whence: usize) -> SyscallResult {
        match (|| {
            let whence = SeekWhence::try_from(whence);
            if whence.is_err() {
                return Err(Errno::EINVAL);
            }

            let curr = curr_task().unwrap();
            let file = curr.fd_manager.lock().get(fd).map_err(|_| Errno::EBADF)?;

            if usize::MAX - file.get_off() < off {
                return Err(Errno::EINVAL);
            }

            if let Some(off) = file.seek(off, whence.unwrap()) {
                Ok(off)
            } else {
                Err(Errno::ESPIPE)
            }
        })() {
            Ok(off) => Ok(off),
            Err(_) => Ok(usize::MAX),
        }
    }

    fn readv(fd: usize, iov: *const IoVec, iovcnt: usize) -> SyscallResult {
        let iov_size = size_of::<IoVec>();
        let iov = VirtAddr::from(iov as usize);
        if iov.value() & (iov_size - 1) != 0 {
            return Err(Errno::EINVAL);
        }

        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();
        let buf = curr_mm.get_buf_mut(iov, iovcnt * iov_size)?;
        drop(curr_mm);
        drop(curr);

        let mut read_len = 0;
        for bytes in buf.into_iter().step_by(iov_size) {
            let iov = unsafe { &*(bytes as *const IoVec) };
            match Self::read(fd, iov.iov_base as *mut _, iov.iov_len) {
                Ok(count) => read_len += count,
                Err(_) => break,
            }
        }
        Ok(read_len)
    }

    fn writev(fd: usize, iov: *const IoVec, iovcnt: usize) -> SyscallResult {
        let iov_size = size_of::<IoVec>();
        let iov = VirtAddr::from(iov as usize);
        if iov.value() & (iov_size - 1) != 0 {
            return Err(Errno::EINVAL);
        }

        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();
        let buf = curr_mm.get_buf_mut(iov, iovcnt * iov_size)?;
        drop(curr_mm);
        drop(curr);

        let mut write_len = 0;
        for bytes in buf.into_iter().step_by(iov_size) {
            let iov = unsafe { &*(bytes as *const IoVec) };
            match Self::write(fd, iov.iov_base as *const _, iov.iov_len) {
                Ok(count) => write_len += count,
                Err(_) => break,
            }
        }
        Ok(write_len)
    }

    fn unlinkat(dirfd: usize, pathname: *const u8, flags: usize) -> SyscallResult {
        // curr.do_unlinkat(dirfd, pathname, flags)
        // .map_err(|err| Errno::from(err))?;
        if flags == AT_REMOVEDIR {
            unimplemented!()
        } else if flags == 0 {
            {
                let curr = curr_task().unwrap();
                let mut mm = curr.mm.lock();
                let path =
                    { resolve_path(&curr, dirfd, mm.get_str(VirtAddr::from(pathname as usize))?)? };

                trace!("UNLINKAT {:?}", path);

                unlink(path)?;

                Ok(0)
            }
        } else {
            Err(Errno::EINVAL)
        }
    }
}

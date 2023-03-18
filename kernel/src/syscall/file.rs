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
    task::{cpu, Task},
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
        let curr = cpu().curr.as_ref().unwrap();

        // Translate user buffer into kernel string.
        let buf = curr.mm().get_buf_mut(VirtAddr::from(buf as usize), count)?;

        // Get the file with the given file descriptor.
        let file = curr.files().get(fd)?;

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
        let curr = cpu().curr.as_ref().unwrap();

        // Get the real buffer translated into physical address.
        let buf = curr.mm().get_buf_mut(VirtAddr::from(buf as usize), count)?;

        // Get the file with the given file descriptor.
        let file = curr.files().get(fd)?;

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
        cpu().curr.as_ref().unwrap().files().remove(fd)?;
        Ok(0)
    }

    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult {
        let flags = OpenFlags::from_bits(flags as u32);
        let mode = StatMode::from_bits(mode as u32);
        if flags.is_none() {
            return Err(Errno::EINVAL);
        }

        let curr = cpu().curr.as_ref().unwrap();
        let flags = flags.unwrap();

        if flags.contains(OpenFlags::O_CREAT) && mode.is_none()
            || flags.contains(OpenFlags::O_WRONLY | OpenFlags::O_RDWR)
        {
            return Err(Errno::EINVAL);
        }

        let mut curr_mm = curr.mm();
        let path = resolve_path(
            &curr,
            dirfd,
            curr_mm.get_str(VirtAddr::from(pathname as usize))?,
        )?;

        trace!("OPEN {:?} {:?}", path, flags);

        Ok(curr.files().push(open(path, flags)?)?)
    }

    fn lseek(fd: usize, off: usize, whence: usize) -> SyscallResult {
        match (|| {
            let whence = SeekWhence::try_from(whence);
            if whence.is_err() {
                return Err(Errno::EINVAL);
            }

            let file = cpu().curr.as_ref().unwrap().files().get(fd)?;

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
        let buf = cpu()
            .curr
            .as_ref()
            .unwrap()
            .mm()
            .get_buf_mut(iov, iovcnt * iov_size)?;

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
        let buf = cpu()
            .curr
            .as_ref()
            .unwrap()
            .mm()
            .get_buf_mut(iov, iovcnt * iov_size)?;

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
        if flags == AT_REMOVEDIR {
            unimplemented!()
        } else if flags == 0 {
            {
                let curr = cpu().curr.as_ref().unwrap();
                let mut curr_mm = curr.mm();
                let path = {
                    resolve_path(
                        &curr,
                        dirfd,
                        curr_mm.get_str(VirtAddr::from(pathname as usize))?,
                    )?
                };

                trace!("UNLINKAT {:?}", path);

                unlink(path)?;

                Ok(0)
            }
        } else {
            Err(Errno::EINVAL)
        }
    }
}

use crate::SyscallResult;

/// Wait for any child; id is ignored.
pub const P_ALL: usize = 0;
/// Wait for the child whose process ID matches id.
pub const P_PID: usize = 1;
/// Wait for any child whose process group ID matches id. Since Linux 5.4,
/// if id is zero, then wait for any child that is in the same process group
/// as the caller's process group at the time of the call
pub const P_PGID: usize = 2;
/// Wait for the child referred to by the PID file descriptor specified in id.
/// (See pidfd_open(2) for further information on PID file descriptors.)
pub const P_PIDFD: usize = 3;

pub trait SyscallProc {
    /// Terminate the calling process.
    fn exit(status: usize) -> !;

    /// Create a child process. This provides more precise control over what pieces of execution context
    /// are shared between the calling process and the child process.
    fn clone(flags: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> SyscallResult {
        Ok(0)
    }

    /// Execute the program referred to by pathname.
    ///
    /// This causes the program that is currently being run by the calling
    /// process to be replaced with a new program, with newly initialized
    /// stack, heap, and (initialized and uninitialized) data segments.
    fn execve(pathname: usize, argv: usize, envp: usize) -> SyscallResult {
        Ok(0)
    }

    /// Wait for process to change state and obtain information about it.
    ///
    /// A state change is considered to be:
    /// 1. a child terminated; in the case of a terminated child, performing a wait allows
    /// the system to release the resources associated with the child; if a wait is not performed,
    /// then the terminated child remains in a `Zombie` state.
    /// 2. a child was stopped by a signal;
    /// 3. a child was resumed by a signal.
    ///
    /// # Argument
    ///
    /// The value of pid can be: 
    /// - `< -1`: meaning wait for any child process whose process group ID is equal to the absolute value of pid.
    /// - `-1`: meaning wait for any child process.
    /// - `0`: meaning wait for any child process whose process group ID is equal to that of the calling process.
    /// - `> 0`: meaning wait for the child whose process ID is equal the value of `pid`.
    fn wait4(pid: isize, wstatus: usize, options: usize, rusage: usize) -> SyscallResult {
        Ok(0)
    }

    /// Provides more precise control over which child state changes to wait for.
    fn waittid(idtype: usize, id: isize, infop: usize, options: usize) -> SyscallResult {
        Ok(0)
    }

    /// Get process identification, always successfully
    fn getpid() -> SyscallResult {
        Ok(0)
    }

    /// Get thread identification, always successfully
    fn gettid() -> SyscallResult {
        Ok(0)
    }

    /// Sets the clear_child_tid value for the calling thread to `tidptr`.
    ///
    /// # Return
    /// Always returns the caller's thread ID.
    fn set_tid_address(tidptr: usize) -> SyscallResult {
        Ok(0)
    }

    /// Changes the location of the program break, which defines the end
    /// of the process's data segment (i.e., the program break is the first
    /// location after the end of the uninitialized data segment). Increasing
    /// the program break has the effect of allocating memory to the process;
    /// decreasing the break deallocates memory.
    ///
    /// # Return
    /// On success, returns the new break, otherwise returns the current break.
    ///
    /// # Error
    /// - `ENOMEM`: Run out of memory.
    fn brk(brk: usize) -> SyscallResult {
        Ok(0)
    }

    /// The munmap() system call deletes the mappings for the specified
    /// address range, and causes further references to addresses within
    /// the range to generate invalid memory references. The region is
    /// also automatically unmapped when the process is terminated. On
    /// the other hand, closing the file descriptor does not unmap the region.
    ///
    /// The address `addr` must be a multiple of the page size (but `length`
    /// need not be).  All pages containing a part of the indicated range
    /// are unmapped, and subsequent references to these pages will
    /// generate `SIGSEGV`.  It is not an error if the indicated range does
    /// not contain any mapped pages.
    ///
    /// # Error
    /// - `EINVAL`:`addr` is not aligned to the page size. Or `len` is 0.
    /// - `ENOMEM`: unmapping a region in the middle of an existing mapping,
    /// since this results in two smaller mappings on either side of the
    /// region being unmapped.
    ///
    /// # Reference
    /// - Linux: `https://elixir.bootlin.com/linux/latest/source/mm/mmap.c#L2757`.
    fn munmap(addr: usize, len: usize) -> SyscallResult {
        Ok(0)
    }

    /// Creates a new mapping in the virtual address space of the calling process.
    ///
    /// # Argument
    /// - `addr`: the starting address for the new mapping.
    /// - `len`: the length of the mapping (which must be greater than 0).
    /// - `prot`: the desired memory protection of the mapping (and must not conflict
    /// with the open mode of the file).
    /// - `flags`:The flags argument determines whether updates to the mapping are
    /// visible to other processes mapping the same region, and whether updates are
    /// carried through to the underlying file. This behavior is determined by including
    /// **exactly one** of the flag values.
    /// - `fd`: The contents of a file mapping are initialized using `length` bytes
    /// at `off` in the file (or other object) referred to by the file descriptor `fd`.
    /// After `mmap()` returns, the file descriptor can be closed immediately without
    /// invalidating the mapping.
    /// - `off`: the starting offset in the file
    ///
    /// If addr is NULL, then the kernel chooses the (page-aligned) address at which
    /// to create the mapping; this is the most portable method of creating a new mapping.
    /// If addr is not NULL, then the kernel takes it as a hint about where to place the mapping;
    /// on Linux, the kernel will pick a nearby page boundary (but always above or equal
    /// to the value specified by /proc/sys/vm/mmap_min_addr) and attempt to create the
    /// mapping there. If another mapping already exists there, the kernel picks a new
    /// address that may or may not depend on the hint.  The address of the new mapping is
    /// returned as the result of the call.
    ///
    /// # Error
    /// - `EINVAL`:
    ///     - too large or unaligned `addr`, `len` or `off`.
    ///     - flags contained none of MAP_PRIVATE, MAP_SHARED, or MAP_SHARED_VALIDATE.
    ///     - `len` is 0.
    /// - `ENOMEM`: unmapping a region in the middle of an existing mapping.
    /// - `EACCESS`: A file descriptor refers to a non-regular file. Or a file mapping was
    /// requested, but fd is not open for reading. Or MAP_SHARED was requested and PROT_WRITE
    /// is set, but fd is not open in read/write (O_RDWR) mode. Or PROT_WRITE is set, but the
    /// file is append-only.
    fn mmap(
        addr: usize,
        len: usize,
        prot: usize,
        flags: usize,
        fd: usize,
        off: usize,
    ) -> SyscallResult {
        Ok(0)
    }

    /// Changes the access protections for the calling process's memory pages containing any part
    /// of the address range in the interval `[addr, addr+len-1]`.  addr must be aligned to a page boundary.
    /// 
    /// If the calling process tries to access memory in a manner that violates the protections, then the
    /// kernel generates a `SIGSEGV` signal for the process.
    /// 
    /// # Error
    /// - `EACCESS`: The memory cannot be given the specified access. This can happen, for example, if you mmap
    /// (2) a file to which you have read-only access, then ask mprotect() to mark it PROT_WRITE.
    /// - `EINVAL`: Addr is not a valid pointer, or not a multiple of the system page size. Invalid flags 
    /// specified in prot.
    /// - `ENOMEM`: 
    ///   - Internal kernel structures could not be allocated.
    ///   - Addresses in the range `[addr, addr+len-1]` are invalid for the address space of the process, or 
    /// specify one or more pages that are not mapped.
    ///   - Changing the protection of a memory region would result in the total number of mappings with 
    /// distinct attributes (e.g., read versus read/write protection) exceeding the allowed maximum.
    fn mprotect(addr: usize, len: usize, prot: usize) -> SyscallResult {
        Ok(0)
    }
}

//! File descriptors are a part of the POSIX API. Each Unix process (except perhaps
//! daemons) should have three standard POSIX file descriptors, corresponding to the
//! three standard streams:
//!
//! - 0: Standard input (STDIN)
//! - 1: Standard output (STDOUT)
//! - 2: Standard error (STDERR)

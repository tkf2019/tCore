/// The `siginfo_t` data type is a structure with the following fields:
///
/// `si_signo`, `si_errno` and `si_code` are defined for all signals. (si_errno is
/// generally unused on Linux.) The rest of the struct may be a union, so that
/// one should read only the fields that are meaningful for the given signal.
#[derive(Debug, Clone, Copy)]
pub struct SigInfo {
    /// Signal number
    pub signo: i32,

    /// An error value
    pub errno: i32,

    /// Signal code
    pub code: i32,
}

/* SIGCHLD si_codes */
/// child has exited
pub const CLD_EXITED: usize = 1;
/// child was killed
pub const CLD_KILLED: usize = 2;
/// child terminated abnormally
pub const CLD_DUMPED: usize = 3;
/// traced child has trapped
pub const CLD_TRAPPED: usize = 4;
/// child has stopped
pub const CLD_STOPPED: usize = 5;
/// stopped child has continued
pub const CLD_CONTINUED: usize = 6;
pub const NSIGCHLD: usize = 6;
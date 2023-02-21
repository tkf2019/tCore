/// The `siginfo_t` data type is a structure with the following fields:
/// 
/// `si_signo`, `si_errno` and `si_code` are defined for all signals. (si_errno is
/// generally unused on Linux.) The rest of the struct may be a union, so that
/// one should read only the fields that are meaningful for the given signal.
#[derive(Debug, Clone, Copy)]
pub struct SigInfo {
    /// Signal number
    pub si_signo: i32,

    /// An error value
    pub si_errno: i32,

    /// Signal code
    pub si_code: i32,
}

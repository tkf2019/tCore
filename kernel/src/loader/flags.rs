numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord)]
    #[allow(non_camel_case_types)]
    #[doc = "Auxiliary entry type. See Linux `/include/uapi/linux/auxvec.h`"]
    pub enum AuxType {
        /// End of vector
        AT_NULL = 0,
        /// Entry should be ignored
        AT_IGNORE = 1,
        /// File descriptor of program
        AT_EXECFD = 2,
        /// Program headers for program
        AT_PHDR = 3,
        /// Size of program header entry
        AT_PHENT = 4,
        /// Number of program headers
        AT_PHNUM = 5,
        /// System page size
        AT_PAGESZ = 6,
        /// Base address of interpreter
        AT_BASE = 7,
        /// Flags
        AT_FLAGS = 8,
        /// Entry point of program
        AT_ENTRY = 9,
        /// Program is not ELF
        AT_NOTELF = 10,
        /// Real uid
        AT_UID = 11,
        /// Effective uid
        AT_EUID = 12,
        /// Real gid
        AT_GID = 13,
        /// Effective gid
        AT_EGID = 14,
        /// Frequency of times()
        AT_CLKTCK = 17,
        /// Secure mode boolean
        AT_SECURE = 23,
        /// Random
        AT_RANDOM = 25,
    }
}

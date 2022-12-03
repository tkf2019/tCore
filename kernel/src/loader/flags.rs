numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord)]
    #[allow(non_camel_case_types)]
    #[doc = "Auxiliary entry type. See Linux `/include/uapi/linux/auxvec.h`"]
    pub enum AuxType {
        /// End of vector
        NULL = 0,
        /// Entry should be ignored
        IGNORE = 1,
        /// File descriptor of program
        EXECFD = 2,
        /// Program headers for program
        PHDR = 3,
        /// Size of program header entry
        PHENT = 4,
        /// Number of program headers
        PHNUM = 5,
        /// System page size
        PAGESZ = 6,
        /// Base address of interpreter
        BASE = 7,
        /// Flags
        FLAGS = 8,
        /// Entry point of program
        ENTRY = 9,
        /// Program is not ELF
        NOTELF = 10,
        /// Real uid
        UID = 11,
        /// Effective uid
        EUID = 12,
        /// Real gid
        GID = 13,
        /// Effective gid
        EGID = 14,
        /// Frequency of times()
        CLKTCK = 17,
        /// Secure mode boolean
        SECURE = 23,
        ///
        /// Random
        RANDOM = 25,
    }
}

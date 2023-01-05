use riscv::register::{satp, ucause, uepc, uscratch, utval, utvec, medeleg};

use crate::{sedeleg, sideleg, suitt, upidaddr};

const TEST_MAGIC: usize = 0x12345678;

pub unsafe fn test_register() {
    ucause::write(TEST_MAGIC);
    assert_eq!(ucause::read().bits(), TEST_MAGIC);

    uscratch::write(TEST_MAGIC);
    assert_eq!(uscratch::read(), TEST_MAGIC);

    uepc::write(TEST_MAGIC);
    assert_eq!(uepc::read(), TEST_MAGIC);

    utval::write(TEST_MAGIC);
    assert_eq!(utval::read(), TEST_MAGIC);

    utvec::write(TEST_MAGIC, utvec::TrapMode::Direct);
    assert_eq!(
        utvec::read().bits(),
        TEST_MAGIC + utvec::TrapMode::Direct as usize
    );

    sideleg::write(TEST_MAGIC);
    assert_eq!(sideleg::read(), 0x10);

    sedeleg::set_breakpoint();
    sedeleg::clear_breakpoint();

    // suitt::write(TEST_MAGIC);
    // assert_eq!(suitt::read().bits(), TEST_MAGIC);

    // upidaddr::write(TEST_MAGIC);
    // assert_eq!(upidaddr::read(), TEST_MAGIC);
}

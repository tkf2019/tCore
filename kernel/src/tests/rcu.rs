use alloc::boxed::Box;
use kernel_sync::RcuCell;
use spin::Lazy;

static LOCKED_DATA: Lazy<RcuCell<Box<usize>>> = Lazy::new(|| {
    RcuCell::new(Box::new(0));
});

pub fn test() {
    
}

extern crate std;

use self::std::println;

use super::*;

fn is_canonical_va(va: usize) -> bool {
    true
}
const fn canonicalize_va(va: usize) -> usize {
    va
}

implement_address!(
    VirtAddr,
    "virtual",
    "v",
    is_canonical_va,
    canonicalize_va,
    page,
    0x1000
);

implement_page_frame!(Page, "virtual", "v", VirtAddr, 0x1000, usize::MAX / 0x1000);

implement_page_frame_range!(PageRange, "physical", virt, Page, VirtAddr, 0x1000);

#[test]
fn it_works() {
    let mut va = VirtAddr::new(0).unwrap();
    va += 0x1001;
    assert!(va.page_offset() == 1);
    assert!(va != VirtAddr::new(0x1000).unwrap());

    let p = Page::from(va);
    let v: VirtAddr = p.into();
    assert!(p.number() == 1);
    assert!(Page::ceil(v) == Page::from(1));

    let pr = PageRange::from_virt_addr(0x1002.into(), 0x3124);
    println!("Page range: {:#?}", &pr);
    let mut iter = pr.into_iter().map(|p| p.number());
    assert!(iter.next() == Some(1));
    assert!(iter.next() == Some(2));
    assert!(iter.next() == Some(3));
    assert!(iter.next() == Some(4));
    assert!(iter.next() == None);
}

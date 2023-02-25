extern crate alloc;
extern crate std;

use alloc::sync::Arc;
use kernel_sync::SpinLock;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    println,
};

use device_cache::*;

struct BlockFile(SpinLock<File>);

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SIZE, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64))
            .expect("Error when seeking!");
        assert_eq!(
            file.write(buf).unwrap(),
            BLOCK_SIZE,
            "Not a complete block!"
        );
    }
}

#[test]
#[allow(unused)]
fn test() {
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("test.txt")
        .unwrap();
    f.set_len(16 * 2048 * 512).unwrap();
    let block_file = Arc::new(BlockFile(SpinLock::new(f)));
    let mut cache = LRUBlockCache::new(4);

    let block = cache.get_block(1, block_file.clone());
    let mut block = block.lock();
    let s: &mut [u8; 40] = block.get_mut(0);
    "Hello World!\n\0"
        .bytes()
        .zip(s.iter_mut())
        .for_each(|(b, ptr)| *ptr = b);

    cache.get_block(0, block_file.clone());
    cache.get_block(1, block_file.clone());
    cache.get_block(3, block_file.clone());
    cache.get_block(2, block_file.clone());
    println!("{:#?}", cache);
    cache.get_block(4, block_file.clone());
    println!("{:#?}", cache);
}

use easy_fs::{BlockDevice, EasyFileSystem};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};

const BLOCK_SZ: usize = 512;

struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
}

pub fn easy_fs_pack(cases: &Vec<&str>, target: &str, img: &str) -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(img)?;
        f.set_len(64 * 2048 * 512).unwrap();
        f
    })));
    let efs = EasyFileSystem::create(block_file, 64 * 2048, 1);
    let root_inode = Arc::new(EasyFileSystem::root_inode(&efs));
    for case in cases {
        println!("{}", format!("{}/{}", target, case));
        // load app data from host file system
        let mut host_file = File::open(format!("{}/{}", target, case)).unwrap();
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();
        // create a file in easy-fs
        let inode = root_inode.create(case).unwrap();
        // write data to easy-fs
        inode.write_at(0, all_data.as_slice());
        // println!("{}", all_data.len());
    }
    println!("List Testcases in EFS: ");
    // list app
    for case in root_inode.readdir() {
        println!("{}", case);
    }
    Ok(())
}
use clap::Args;

#[derive(Args, Default)]
pub struct PackArgs {
    /// Which filesystem used to pack
    #[clap(long, default_value = "easy-fs")]
    pub pack_fs: Option<String>,

    /// Target path.
    ///
    /// - `easy-fs`: Executable files listed in target directory will be packed.
    ///
    /// - `fat32`: All files listed in target directory will be packed, multi-level directory supported.
    #[clap(long)]
    pub pack_target: Option<String>,

    /// Image path.
    #[clap(long, default_value = ".")]
    pack_image: Option<String>,

    /// Image size in blocks.
    #[clap(long, default_value_t = 1024)]
    pack_size: usize,

    /// Image block size in bytes.
    #[clap(long, default_value_t = 512)]
    pack_bsize: usize,
}

pub mod pack_easy_fs {
    use easy_fs::{BlockDevice, EasyFileSystem};
    use std::{
        fs::{File, OpenOptions},
        io::{Read, Seek, SeekFrom, Write},
        sync::{Arc, Mutex},
    };

    use super::PackArgs;

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

    impl PackArgs {
        pub fn pack_easy_fs(&self, cases: &Vec<&str>, target: String) -> std::io::Result<()> {
            let image = self.pack_image.as_ref().unwrap().clone() + "easy-fs.img";
            let block_file = Arc::new(BlockFile(Mutex::new({
                let f = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&image)?;
                f.set_len(64 * 2048 * 512).unwrap();
                f
            })));
            println!("Easy-fs image: {}", image);
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
            }
            println!("List Testcases in EFS: ");
            // list app
            for case in root_inode.readdir() {
                println!("{}", case);
            }
            Ok(())
        }
    }
}

pub mod pack_fat32 {
    use fatfs::{format_volume, FileSystem, FormatVolumeOptions, FsOptions, StdIoWrapper, Write};
    use fscommon::BufStream;
    use std::{
        fs::{self, DirEntry, File},
        io::{self, Read},
    };

    use super::PackArgs;

    fn traverse_dir(file: DirEntry, target_dir: String, names: &mut Vec<String>) {
        let file_name = file.file_name().into_string().unwrap();
        if file.path().is_dir() {
            println!("dir: {}", file.file_name().into_string().unwrap());
            names.push(format!("{}{}/", target_dir, file_name));
            for inner_entry in fs::read_dir(file.path()).unwrap() {
                traverse_dir(
                    inner_entry.unwrap(),
                    format!("{}{}/", target_dir, file_name),
                    names,
                );
            }
        } else {
            names.push(format!("{}{}", target_dir, file_name));
        }
    }

    fn traverse_fat_dir<'a>(
        root: &fatfs::Dir<
            '_,
            fatfs::StdIoWrapper<fscommon::BufStream<std::fs::File>>,
            fatfs::ChronoTimeProvider,
            fatfs::LossyOemCpConverter,
        >,
        file: fatfs::DirEntry<
            '_,
            fatfs::StdIoWrapper<fscommon::BufStream<std::fs::File>>,
            fatfs::ChronoTimeProvider,
            fatfs::LossyOemCpConverter,
        >,
        dir_now: String,
    ) {
        if dir_now != "" {
            print!("\t");
        }
        println!("{}", file.file_name());
        if file.is_dir() {
            let inner_dir = dir_now + file.file_name().as_str() + "/";
            println!("{}", &inner_dir);
            for dir_entry in root.open_dir(inner_dir.as_str()).unwrap().iter() {
                let file = dir_entry.unwrap();
                // Escape hidden files or directories.
                if !file.file_name().starts_with(".") {
                    traverse_fat_dir(root, file, inner_dir.clone());
                }
            }
        }
    }

    fn create_new_fs(name: &str) -> io::Result<()> {
        let img_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&name)
            .unwrap();
        img_file.set_len(256 * 2048 * 512).unwrap();
        let buf_file = BufStream::new(img_file);
        format_volume(
            &mut StdIoWrapper::from(buf_file),
            FormatVolumeOptions::new(),
        )
        .unwrap();
        Ok(())
    }

    impl PackArgs {
        pub fn pack_fat32(&self) {
            let target = self.pack_target.as_ref().unwrap().clone();
            let image = self.pack_image.as_ref().unwrap().clone() + "fat32.img";
            create_new_fs(&image.as_str()).unwrap();
            println!("FAT32 image: {}", image);

            let mut user_apps: Vec<String> = vec![];
            for dir_entry in fs::read_dir(&target).unwrap() {
                let file = dir_entry.unwrap();
                traverse_dir(file, String::from(""), &mut user_apps);
            }

            let img_file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&image)
                .unwrap();
            let buf_file = BufStream::new(img_file);
            let options = FsOptions::new().update_accessed_date(true);
            let fs = FileSystem::new(buf_file, options).unwrap();
            let root = fs.root_dir();

            for app in user_apps {
                if app.ends_with("/") {
                    println!("user dir: {}", app.as_str());
                    root.create_dir(app.as_str()).unwrap();
                } else {
                    let mut origin_file = File::open(format!("{}{}", target, app)).unwrap();
                    let mut all_data: Vec<u8> = Vec::new();
                    origin_file.read_to_end(&mut all_data).unwrap();
                    println!("User app: {}", app.as_str());
                    let mut file_in_fs = root.create_file(app.as_str()).unwrap();
                    file_in_fs.write_all(all_data.as_slice()).unwrap();
                }
            }

            // List user apps in fat32.
            for dir_entry in root.iter() {
                traverse_fat_dir(&root, dir_entry.unwrap(), String::from(""));
            }
        }
    }
}

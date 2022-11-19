use alloc::{collections::LinkedList, sync::Arc};
use core::{any::Any, fmt};
use spin::Mutex;

use crate::CacheUnit;

/// Trait for block devices
/// which reads and writes data in the unit of blocks
pub trait BlockDevice: Send + Sync + Any {
    /// Read contiguous blocks from a block device.
    /// # Argument
    /// - `block_id`: the first block (sector) identification to read.
    /// - `buf`: the buffer to write.
    fn read_block(&self, block_id: usize, buf: &mut [u8]);

    /// Write data from buffer to contiguous blocks.
    /// # Argument
    /// - `block_id`: the first block (sector) identification to write.
    /// - `buf`: the buffer to read.
    fn write_block(&self, block_id: usize, buf: &[u8]);
}

pub const BLOCK_SIZE: usize = 512;

pub struct BlockCacheUnit {
    /// Block identification (offset) in the block device.
    id: usize,

    /// Inner data.
    data: [u8; BLOCK_SIZE],

    /// Target block device.
    device: Arc<dyn BlockDevice>,

    /// If set, the block is modified and need to be synchronized to
    /// the target device.
    dirty: bool,
}

impl CacheUnit for BlockCacheUnit {
    fn sync(&mut self) {
        if self.dirty {
            self.dirty = false;
            self.device.write_block(self.id, &self.data);
        }
    }

    fn addr(&self, offset: usize) -> usize {
        &self.data[offset] as *const _ as usize
    }

    fn set_dirty(&mut self) {
        self.dirty = true;
    }

    fn size(&self) -> usize {
        BLOCK_SIZE
    }

    fn id(&self) -> usize {
        self.id
    }
}

impl BlockCacheUnit {
    pub fn new(block_id: usize, block_dev: Arc<dyn BlockDevice>) -> Self {
        let mut data = [0u8; BLOCK_SIZE];
        block_dev.read_block(block_id, &mut data);
        Self {
            id: block_id,
            data,
            device: block_dev,
            dirty: false,
        }
    }
}

impl Drop for BlockCacheUnit {
    fn drop(&mut self) {
        self.sync();
    }
}

pub struct LRUBlockCache {
    max_size: usize,
    inner: LinkedList<(usize, Arc<Mutex<BlockCacheUnit>>)>,
}

impl LRUBlockCache {
    pub fn new(size: usize) -> Self {
        Self {
            max_size: size,
            inner: LinkedList::new(),
        }
    }

    pub fn get_block(
        &mut self,
        block_id: usize,
        block_dev: Arc<dyn BlockDevice>,
    ) -> Arc<Mutex<BlockCacheUnit>> {
        let inner = &mut self.inner;
        let result = inner
            .iter_mut()
            .enumerate()
            .find(|(_, pair)| pair.0 == block_id)
            .map(|(index, pair)| (index, pair.clone()));
        if let Some((index, pair)) = result {
            // Detach the block from the linked list.
            inner.remove(index);
            // Attach this block to the back of the linked list.
            inner.push_back((pair.0, pair.1.clone()));
            pair.1
        } else {
            if inner.len() == self.max_size {
                if let Some((index, _)) = inner
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Arc::strong_count(&pair.1) == 1)
                {
                    inner.remove(index);
                } else {
                    panic!("Run out of queue cache. Consider increase the size of this cache");
                }
            }
            let unit = Arc::new(Mutex::new(BlockCacheUnit::new(block_id, block_dev)));
            inner.push_back((block_id, unit.clone()));
            unit
        }
    }
}

impl fmt::Debug for LRUBlockCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blocks in Cache (id, rc): [")?;
        for pair in self.inner.iter() {
            write!(f, " ({}, {})", pair.0, Arc::strong_count(&pair.1))?;
        }
        write!(f, " ]")?;
        Ok(())
    }
}

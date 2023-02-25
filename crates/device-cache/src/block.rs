use alloc::{
    collections::{LinkedList, VecDeque},
    sync::Arc,
};
use core::{any::Any, fmt};
use kernel_sync::SpinLock;

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

pub trait BlockCache {
    /// The maximum number of block cache units.
    fn capacity(&self) -> usize;

    /// Get a shared pointer to the target block in the block device.
    fn get_block(
        &mut self,
        block_id: usize,
        block_dev: Arc<dyn BlockDevice>,
    ) -> Arc<SpinLock<BlockCacheUnit>>;

    /// Synchronize all block cache units to block device.
    fn sync_all(&self);
}

pub struct FIFOBlockCache {
    max_size: usize,
    inner: VecDeque<(usize, Arc<SpinLock<BlockCacheUnit>>)>,
}

impl FIFOBlockCache {
    pub fn new(size: usize) -> Self {
        Self {
            max_size: size,
            inner: VecDeque::new(),
        }
    }
}

impl BlockCache for FIFOBlockCache {
    fn capacity(&self) -> usize {
        self.max_size
    }

    fn get_block(
        &mut self,
        block_id: usize,
        block_dev: Arc<dyn BlockDevice>,
    ) -> Arc<SpinLock<BlockCacheUnit>> {
        if let Some(pair) = self.inner.iter().find(|pair| pair.0 == block_id) {
            Arc::clone(&pair.1)
        } else {
            // substitute
            if self.inner.len() == self.max_size {
                // from front to tail
                if let Some((idx, _)) = self
                    .inner
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Arc::strong_count(&pair.1) == 1)
                {
                    self.inner.drain(idx..=idx);
                } else {
                    panic!("Run out of BlockCache!");
                }
            }
            // load block into mem and push back
            let block_cache = Arc::new(SpinLock::new(BlockCacheUnit::new(
                block_id,
                Arc::clone(&block_dev),
            )));
            self.inner.push_back((block_id, Arc::clone(&block_cache)));
            block_cache
        }
    }

    fn sync_all(&self) {
        for (_, unit) in self.inner.iter() {
            unit.lock().sync();
        }
    }
}

impl fmt::Debug for FIFOBlockCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blocks in Cache (id, rc): [")?;
        for pair in self.inner.iter() {
            write!(f, " ({}, {})", pair.0, Arc::strong_count(&pair.1))?;
        }
        write!(f, " ]")?;
        Ok(())
    }
}

pub struct LRUBlockCache {
    max_size: usize,
    inner: LinkedList<(usize, Arc<SpinLock<BlockCacheUnit>>)>,
}

impl LRUBlockCache {
    pub fn new(size: usize) -> Self {
        Self {
            max_size: size,
            inner: LinkedList::new(),
        }
    }
}
impl BlockCache for LRUBlockCache {
    fn capacity(&self) -> usize {
        self.max_size
    }

    fn get_block(
        &mut self,
        block_id: usize,
        block_dev: Arc<dyn BlockDevice>,
    ) -> Arc<SpinLock<BlockCacheUnit>> {
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
            let unit = Arc::new(SpinLock::new(BlockCacheUnit::new(block_id, block_dev)));
            inner.push_back((block_id, unit.clone()));
            unit
        }
    }

    fn sync_all(&self) {
        for (_, unit) in self.inner.iter() {
            unit.lock().sync();
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

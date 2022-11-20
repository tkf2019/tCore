use alloc::sync::Arc;
use spin::{Lazy, Mutex};
use tcache::{BlockCache, BlockCacheUnit, BlockDevice, FIFOBlockCache};

const CACHE_SIZE: usize = 16;

/// The global block cache manager
pub static BLOCK_CACHE_MANAGER: Lazy<Mutex<FIFOBlockCache>> =
    Lazy::new(|| Mutex::new(FIFOBlockCache::new(CACHE_SIZE)));

/// Get the block cache corresponding to the given block id and block device
pub fn get_block_cache(
    block_id: usize,
    block_device: Arc<dyn BlockDevice>,
) -> Arc<Mutex<BlockCacheUnit>> {
    BLOCK_CACHE_MANAGER.lock().get_block(block_id, block_device)
}
/// Sync all block cache to block device
pub fn block_cache_sync_all() {
    BLOCK_CACHE_MANAGER.lock().sync_all();
}

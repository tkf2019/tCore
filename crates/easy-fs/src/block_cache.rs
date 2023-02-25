use alloc::sync::Arc;
use device_cache::{BlockCache, BlockCacheUnit, BlockDevice, FIFOBlockCache};
use kernel_sync::SpinLock;
use spin::Lazy;

const CACHE_SIZE: usize = 16;

/// The global block cache manager
pub static BLOCK_CACHE_MANAGER: Lazy<SpinLock<FIFOBlockCache>> =
    Lazy::new(|| SpinLock::new(FIFOBlockCache::new(CACHE_SIZE)));

/// Get the block cache corresponding to the given block id and block device
pub fn get_block_cache(
    block_id: usize,
    block_device: Arc<dyn BlockDevice>,
) -> Arc<SpinLock<BlockCacheUnit>> {
    BLOCK_CACHE_MANAGER.lock().get_block(block_id, block_device)
}
/// Sync all block cache to block device
pub fn block_cache_sync_all() {
    BLOCK_CACHE_MANAGER.lock().sync_all();
}

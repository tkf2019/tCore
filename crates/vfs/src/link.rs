use alloc::collections::BTreeMap;
use kernel_sync::SpinLock;
use spin::Lazy;

use super::path::Path;

/// Virtual path mapped to real path.
static LINK_PATH_MAP: Lazy<SpinLock<BTreeMap<Path, Path>>> =
    Lazy::new(|| SpinLock::new(BTreeMap::new()));

/// Real path mapped to hard link count.
static LINK_COUNT_MAP: Lazy<SpinLock<BTreeMap<Path, usize>>> =
    Lazy::new(|| SpinLock::new(BTreeMap::new()));

/// Gets the real path of a given path.
///
/// Returns a `clone` of the path if the path is not existing in the map,
/// since no link has been made for this path.
pub fn get_path(path: &Path) -> Path {
    let path_map = LINK_PATH_MAP.lock();
    match path_map.get(path) {
        Some(path) => path.clone(),
        None => path.clone(),
    }
}

/// Get the number of hard links of a given `real` path.
pub fn get_nlink(path: &Path) -> usize {
    let count_map = LINK_COUNT_MAP.lock();
    match count_map.get(path) {
        Some(nlink) => *nlink,
        None => 1,
    }
}

/// Adds a link with no existance check.
///
/// The number of links will be counted since a virtual path requires a link to real path.
pub fn add_link(real_path: &Path, user_path: &Path) {
    let mut path_map = LINK_PATH_MAP.lock();
    let mut count_map = LINK_COUNT_MAP.lock();
    *count_map.entry(real_path.clone()).or_insert(1) += 1;
    path_map.insert(user_path.clone(), real_path.clone());
}

/// Removes a link maintained by a virutal or real path with no
/// existance check.
///
/// Returns the real path if the file referred by the real path
/// needs to be deleted.
pub fn remove_link(path: &Path) -> Option<Path> {
    let mut path_map = LINK_PATH_MAP.lock();
    let mut count_map = LINK_COUNT_MAP.lock();
    match path_map.remove(path) {
        // A virtual path
        Some(real_path) => {
            let count = count_map.get_mut(&real_path).unwrap();
            *count -= 1;
            if *count == 0 {
                count_map.remove(&real_path).unwrap();
                return Some(real_path.clone());
            }
            None
        }
        // A real path
        None => {
            match count_map.get_mut(path) {
                // The real path has been linked to.
                Some(count) => {
                    *count -= 1;
                    if *count == 0 {
                        count_map.remove(path).unwrap();
                        return Some(path.clone());
                    }
                    None
                }
                // The real path has not been linked. Do nothing.
                None => Some(path.clone()),
            }
        }
    }
}

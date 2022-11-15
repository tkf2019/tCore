## 11.17

### Rust-fatfs

- `OemCpConverter`: Provides a custom implementation for a short name encoding/decoding.
- `TimeProvider`: A current time and date provider. Provides a custom implementation for a time resolution used when updating directory entry time fields.

### maturin

- File system object: A memory mapped device.
- Why get cpu id from `tp`? Why not `mhartid` register?
- `fscommon::BufStream` derived `Read`, `Write` and `Seek` traits. Why use BufStream but not a custom block cache.
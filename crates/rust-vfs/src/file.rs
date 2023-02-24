pub struct File {}

pub trait FileOperations {
    /// Reads bytes from this file to the buffer.
    ///
    /// Returns the number of bytes read from this file.
    /// Returns [`None`] if the file is not readable.
    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        None
    }

    /// Writes bytes from the buffer to this file.
    ///
    /// Returns the number of bytes written to this file.
    /// Returns [`None`] if the file is not writable.
    fn write(&self, buf: &[u8]) -> Option<usize> {
        None
    }
}
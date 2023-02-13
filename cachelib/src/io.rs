use std::fs::File;
use std::io::{Read, Seek};

pub fn get_reader(file: File) -> Result<impl Read + Seek, String> {
    // Compatibility on other systems
    #[cfg(not(unix))]
    {
        use std::io::BufReader;
        // Make sure reads are aligned with each 40 byte line, 4096 is the standard block size (or a multiple of it) on most systems
        const BUFFER_SIZE: usize = 40 * 4096;
        Ok(BufReader::with_capacity(BUFFER_SIZE, file))
    }
    // Memory map the file for speed on unix systems
    #[cfg(unix)]
    {
        use std::io::Cursor;
        use memmap2::{Advice, Mmap};
        // MMap saves about 6ms (16% for small set size caches, negligible for fully associative) for 700-800MB example files
        unsafe {
            let m = Mmap::map(&file).map_err(|e| format!("Couldn't memory map the file: {e}"))?;
            m.advise(Advice::Sequential).map_err(|e| format!("Failed to provide access advice to the OS, {e}"))?;
            Ok(Cursor::new(m))
        }
    }
}
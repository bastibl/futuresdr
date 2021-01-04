use anyhow::Result;

use crate::runtime::buffer::pagesize;
use crate::runtime::config;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ffi::CString;

#[derive(Debug)]
pub struct DoubleMappedTempFile {
    addr: *mut libc::c_void,
    size: usize,
}

impl DoubleMappedTempFile {
    pub fn new(size: usize) -> Result<DoubleMappedTempFile> {
        let page_size = pagesize();
        if size % page_size != 0 {
            bail!("size ({}) not a multiple of page size ({})", size, page_size);
        }

        let tmp = config::get_or_default::<String>("tmp_dir", "/tmp/".to_owned());
        let mut path = PathBuf::new();
        path.push(tmp);
        path.push("buffer-XXXXXX");
        let cstring = CString::new(path.into_os_string().as_bytes()).unwrap();
        let path = cstring.as_bytes_with_nul().as_ptr();

        let fd;
        let buff;
        unsafe {
            fd = libc::mkstemp(path as *mut libc::c_char);
            ensure!(fd >= 0, "tempfile could not be created");

            let ret = libc::unlink(path as *const libc::c_char);
            if ret < 0 {
                libc::close(fd);
                bail!("unlinking failed");
            }

            let ret = libc::ftruncate(fd, 2 * size as libc::off_t);
            if ret < 0 {
                libc::close(fd);
                bail!("truncate failed");
            }

            buff = libc::mmap(0 as *mut libc::c_void, 2 * size, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0);
            if buff == libc::MAP_FAILED {
                libc::close(fd);
                bail!("mmap placeholder failed");
            }

            let ret = libc::munmap(buff.offset(size as isize), size);
            if ret < 0 {
                libc::munmap(buff, size);
                libc::close(fd);
                bail!("munmap second half of placeholder failed");
            }

            let buff2 = libc::mmap(buff.offset(size as isize), size, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_FIXED | libc::MAP_SHARED, fd, 0);
            if buff2 != buff.offset(size as isize) {
                libc::munmap(buff, size);
                libc::close(fd);
                bail!("mmapped second half at wrong address");
            }

            let ret = libc::close(fd);
            if ret < 0 {
                bail!("failed to close temp file");
            }
        }

        Ok(DoubleMappedTempFile {
            addr: buff,
            size,
        })
    }
}

impl Drop for DoubleMappedTempFile {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.addr, self.size * 2);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem;
    use std::slice;

    #[test]
    fn tmp_file() {
        let ps = 3 * pagesize();
        let b = DoubleMappedTempFile::new(ps);
        assert!(b.is_ok());
        let b = b.unwrap();

        unsafe {
            let b1 = slice::from_raw_parts_mut::<u64>(b.addr as *mut u64, ps / mem::size_of::<u64>());
            let b2 = slice::from_raw_parts_mut::<u64>(b.addr.offset(b.size as isize) as *mut u64, ps / mem::size_of::<u64>());
            for (i, v) in b1.iter_mut().enumerate() {
                *v = i as u64;
            }
            assert_eq!(b1, b2);
        }
    }
}

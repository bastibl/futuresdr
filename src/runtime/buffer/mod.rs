pub mod double_mapped_temp_file;

pub use double_mapped_temp_file::DoubleMappedTempFile;

pub fn pagesize() -> usize {
    unsafe {
        let ps = libc::sysconf(libc::_SC_PAGESIZE);
        if ps < 0 {
            panic!("could not determince page size");
        }
        ps as usize
    }
}

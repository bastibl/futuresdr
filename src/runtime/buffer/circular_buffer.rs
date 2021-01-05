use std::cmp;
use vec_arena::Arena;

use crate::runtime::buffer::DoubleMappedTempFile;
use crate::runtime::buffer::pagesize;


// everything is measured in items, e.g., offsets, capacity, space available

#[derive(Debug)]
pub struct Writer {
    buffer: DoubleMappedTempFile,
    offset: usize,
    readers: Arena<ReaderState>,
    capacity: usize,
    item_size: usize,
}

#[derive(Debug)]
struct ReaderState {
    offset: usize,
}

impl Writer {

    pub fn new(item_size: usize, min_bytes: usize) -> Writer {
        let page_size = pagesize();
        let mut buffer_size = page_size;

        while (buffer_size < min_bytes) || (buffer_size % item_size != 0) {
            buffer_size += page_size;
        }

        Writer {
            buffer: DoubleMappedTempFile::new(buffer_size).unwrap(),
            offset: 0,
            readers: Arena::new(),
            capacity: buffer_size / item_size,
            item_size,
        }
    }

    fn space_available(&self) -> usize {
        let mut space = self.capacity;

        for (_, reader) in self.readers.iter() {

            if reader.offset <= self.offset {
                space = cmp::min(space, reader.offset + self.capacity - 1 - self.offset);
            } else {
                space = cmp::min(space, reader.offset - 1 - self.offset);
            }
        }

        space
    }

    pub fn produce(&mut self, amount: usize) {
        debug_assert!(amount <= self.space_available());
        self.offset = (self.offset + amount) % self.capacity;
    }

    pub fn consume(&mut self, reader_id: usize, amount: usize) {
        let reader = self.readers.get_mut(reader_id).unwrap();

        debug_assert!(amount <=
            if reader.offset < self.offset {
                reader.offset + self.capacity - 1 - self.offset
            } else {
                reader.offset - 1 - self.offset
            }
        );

        reader.offset = (reader.offset + amount) % self.capacity;
    }

    pub fn buffer_bytes(&self) -> (*mut u8, usize) {
        let space = self.space_available();
        unsafe {
            (
                self.buffer.addr().offset((self.offset * self.item_size) as isize) as *mut u8,
                space * self.item_size,
            )
        }
    }

    pub fn add_reader(&mut self) -> Reader {
        let id = self.readers.insert(ReaderState { offset: self.offset });

        Reader {
            ptr: self.buffer.addr(),
            write_offset: self.offset,
            read_offset: self.offset,
            capacity: self.capacity,
            item_size: self.item_size,
            id,
        }
    }

    pub fn item_size(&self) -> usize {
        self.item_size
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn n_reader(&self) -> usize {
        self.readers.len()
    }
}

unsafe impl Send for Writer {}


#[derive(Debug)]
pub struct Reader {
    pub ptr: *const std::ffi::c_void,
    pub write_offset: usize,
    pub read_offset: usize,
    pub capacity: usize,
    pub item_size: usize,
    pub id: usize,
}

impl Reader {
    pub fn space_available(&self) -> usize {
        if self.read_offset > self.write_offset {
            self.write_offset + self.capacity - self.read_offset
        } else {
            self.write_offset - self.read_offset
        }
    }

    pub fn buffer_bytes(&self) -> (*const u8, usize) {
        unsafe {(
            self.ptr.offset((self.read_offset * self.item_size) as isize) as *const u8,
            self.space_available() * self.item_size,
        )}
    }

    pub fn produce(&mut self, amount: usize) {
        debug_assert!(amount <=
            if self.read_offset <= self.write_offset {
                self.read_offset + self.capacity - 1 - self.write_offset
            } else {
                self.read_offset - 1 - self.write_offset
            });

        self.write_offset = (self.write_offset + amount) % self.capacity
    }

    pub fn consume(&mut self, amount: usize) {
        debug_assert!(amount <= self.space_available());
        self.read_offset = (self.read_offset + amount) % self.capacity;
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn item_size(&self) -> usize {
        self.item_size
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

unsafe impl Send for Reader {}


#[cfg(test)]
mod tests {
    use super::*;
    use std::slice;

    #[test]
    fn circ_buffer() {
        let ps = pagesize();
        let item_size = 8;
        let mut w = Writer::new(item_size, 123);

        assert_eq!(w.item_size(), item_size);
        assert_eq!((w.capacity() * item_size) % ps, 0);
        assert_eq!(w.n_reader(), 0);
        assert_eq!(w.space_available(), w.capacity());

        let mut r = w.add_reader();
        assert_eq!(r.capacity(), w.capacity());
        assert_eq!(r.space_available(), 0);
        assert_eq!(w.space_available(), w.capacity() - 1);
        assert_eq!(w.n_reader(), 1);

        let (buff, size) = w.buffer_bytes();
        assert_eq!(size, w.space_available() * item_size);

        unsafe {
            let buff = slice::from_raw_parts_mut::<u64>(buff as *mut u64, size / item_size);
            for i in 0..10 {
                buff[i] = i as u64;
            }
        }

        w.produce(3);
        w.produce(7);
        r.produce(10);
        assert_eq!(r.space_available(), 10);
        assert_eq!(w.space_available(), w.capacity() - 1 - 10);

        let (buff, size) = r.buffer_bytes();
        unsafe {
            let buff = slice::from_raw_parts_mut::<u64>(buff as *mut u64, size / item_size);
            for i in 0..r.space_available() {
                assert!(buff[i] == i as u64);
            }
        }

        r.consume(6);
        w.consume(r.id(), 6);
        assert_eq!(r.space_available(), 4);
        assert_eq!(w.space_available(), w.capacity() - 1 - 4);
    }
}

use std::{
    ffi::{c_void, CStr, CString},
    fs::File,
    io::{BufReader, Read},
    os::raw::c_char,
    ptr,
};
use zstd::stream::read::Decoder;

pub const BUF_SIZE: usize = 0x10000;

struct DecoderWrapper<'a> {
    buf: [u8; BUF_SIZE],
    pos: usize,
    cap: usize,
    decoder: Decoder<'a, BufReader<File>>,
}

impl<'a> DecoderWrapper<'a> {
    pub fn new(f: File) -> DecoderWrapper<'a> {
        let decoder = Decoder::new(f).expect("Cannot create decoder");
        DecoderWrapper {
            buf: [0u8; BUF_SIZE],
            pos: 0,
            cap: 0,
            decoder,
        }
    }

    pub fn read_line(&mut self, line: &mut Vec<u8>) -> std::io::Result<usize> {
        line.clear();
        let mut len = 0;

        loop {
            if self.pos >= self.cap {
                match self.decoder.read(&mut self.buf) {
                    Ok(0) => return Ok(len),
                    Ok(buf_len) => {
                        self.pos = 0;
                        self.cap = buf_len;
                    }
                    Err(e) => return Err(e),
                }
            }
            match memchr::memchr(0xAu8, &self.buf[self.pos..self.cap]) {
                Some(i) => {
                    line.extend_from_slice(&self.buf[self.pos..=(self.pos + i)]);
                    self.pos = self.pos + i + 1;
                    len += i + 1;
                    return Ok(len);
                }
                None => {
                    line.extend_from_slice(&self.buf[self.pos..self.cap]);
                    len += self.cap - self.pos;
                    self.pos = self.cap;
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn zstd_line_read_new<'a>(zstd_file_path: *const c_char) -> *mut c_void {
    let r_zstd_file_path = unsafe { CStr::from_ptr(zstd_file_path) };
    let file = File::open(r_zstd_file_path.to_str().unwrap());
    if file.is_err() {
        eprintln!("Cannot open file {}", r_zstd_file_path.to_str().unwrap());
        return ptr::null::<c_void>() as *mut c_void;
    }
    let file = file.unwrap();
    let wrapper = DecoderWrapper::new(file);
    Box::into_raw(Box::new(wrapper)) as *mut c_void
}

#[no_mangle]
pub extern "C" fn zstd_line_read<'a>(reader: *mut c_void) -> *const c_char {
    let wrapper: *mut DecoderWrapper<'a> = reader as *mut DecoderWrapper<'a>;
    let mut line = Vec::with_capacity(BUF_SIZE);
    unsafe {
        match (*wrapper).read_line(&mut line) {
            Ok(len) => {
                if len == 0 {
                    return ptr::null();
                } else {
                    return CString::from_vec_unchecked(line).into_raw();
                }
            }
            Err(e) => {
                panic!(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;

    #[test]
    fn test_wrapper_basic() {
        let f = File::open("test1.txt.zst").unwrap();
        let mut wrapper = DecoderWrapper::new(f);
        let mut buf = Vec::with_capacity(0x100);
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 14);
        assert_eq!(buf, b"The Happening\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 6);
        assert_eq!(buf, b"ABCDE\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 1);
        assert_eq!(buf, b"\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 2);
        assert_eq!(buf, b"1\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 5);
        assert_eq!(buf, b"Titi\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 0);
    }

    #[test]
    fn test_wrapper_small_buf() {
        let f = File::open("test1.txt.zst").unwrap();
        let mut wrapper = DecoderWrapper::new(f);
        let mut buf = Vec::with_capacity(0x1);
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 14);
        assert_eq!(buf, b"The Happening\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 6);
        assert_eq!(buf, b"ABCDE\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 1);
        assert_eq!(buf, b"\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 2);
        assert_eq!(buf, b"1\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 5);
        assert_eq!(buf, b"Titi\n");
        assert_eq!(wrapper.read_line(&mut buf).unwrap(), 0);
    }

    #[test]
    fn test_c_api_basic() {
        let cstr = CString::new("test1.txt.zst").unwrap();
        let reader = zstd_line_read_new(cstr.as_ptr());
        let line = zstd_line_read(reader);
        assert_eq!(
            dbg!(unsafe { CStr::from_ptr(line) }.to_bytes()),
            b"The Happening\n"
        );
        let line = zstd_line_read(reader);
        assert_eq!(dbg!(unsafe { CStr::from_ptr(line) }.to_bytes()), b"ABCDE\n");
        let line = zstd_line_read(reader);
        assert_eq!(dbg!(unsafe { CStr::from_ptr(line) }.to_bytes()), b"\n");
        let line = zstd_line_read(reader);
        assert_eq!(dbg!(unsafe { CStr::from_ptr(line) }.to_bytes()), b"1\n");
        let line = zstd_line_read(reader);
        assert_eq!(dbg!(unsafe { CStr::from_ptr(line) }.to_bytes()), b"Titi\n");
        let line = zstd_line_read(reader);
        assert!(line.is_null());
    }
}

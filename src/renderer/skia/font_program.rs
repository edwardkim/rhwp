//! 선택한 OS face의 실제 SFNT 원본과 collection index를 보존한다.

use skia_safe::Typeface;

pub(super) fn typeface_program_bytes(typeface: &Typeface) -> Option<(Vec<u8>, u32)> {
    // CoreText의 stream은 같은 테이블을 다른 SFNT 컨테이너로 재포장할 수 있다.
    // 이름만으로 원본을 고르지 않고 모든 테이블이 같은 로컬 파일만 원본으로 인정한다.
    #[cfg(target_os = "macos")]
    if let Some(program) = coretext::original_program(typeface) {
        return Some(program);
    }
    if let Some((data, index)) = typeface.to_existing_font_data() {
        return Some((data.as_bytes().to_vec(), index));
    }
    typeface.to_font_bytes()
}

#[cfg(target_os = "macos")]
mod coretext {
    use super::Typeface;
    use std::{
        ffi::{c_char, c_void, CString, OsString},
        io::Read,
        os::unix::ffi::OsStringExt,
        path::PathBuf,
    };

    const UTF8: u32 = 0x0800_0100;
    const MAX_BYTES: u64 = 32 * 1024 * 1024;
    const MAX_FACES: u32 = 64;

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFStringCreateWithCString(
            allocator: *const c_void,
            value: *const c_char,
            encoding: u32,
        ) -> *const c_void;
        fn CFRelease(value: *const c_void);
        fn CFGetTypeID(value: *const c_void) -> usize;
        fn CFEqual(a: *const c_void, b: *const c_void) -> u8;
        fn CFURLGetTypeID() -> usize;
        fn CFURLCopyScheme(url: *const c_void) -> *const c_void;
        fn CFURLGetFileSystemRepresentation(
            url: *const c_void,
            resolve: u8,
            buffer: *mut u8,
            length: isize,
        ) -> u8;
    }
    #[link(name = "CoreText", kind = "framework")]
    unsafe extern "C" {
        static kCTFontURLAttribute: *const c_void;
        fn CTFontCreateWithName(
            name: *const c_void,
            size: f64,
            matrix: *const c_void,
        ) -> *const c_void;
        fn CTFontCopyAttribute(font: *const c_void, attribute: *const c_void) -> *const c_void;
    }

    struct Owned(*const c_void);
    impl Owned {
        fn new(value: *const c_void) -> Option<Self> {
            (!value.is_null()).then_some(Self(value))
        }
        fn string(value: &str) -> Option<Self> {
            let value = CString::new(value).ok()?;
            // Create 규칙으로 받은 객체만 Owned가 해제한다.
            Self::new(unsafe { CFStringCreateWithCString(std::ptr::null(), value.as_ptr(), UTF8) })
        }
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            unsafe { CFRelease(self.0) };
        }
    }

    fn source_path(typeface: &Typeface) -> Option<PathBuf> {
        let name = Owned::string(&typeface.post_script_name()?)?;
        let font = Owned::new(unsafe { CTFontCreateWithName(name.0, 0.0, std::ptr::null()) })?;
        let url = Owned::new(unsafe { CTFontCopyAttribute(font.0, kCTFontURLAttribute) })?;
        if unsafe { CFGetTypeID(url.0) != CFURLGetTypeID() } {
            return None;
        }
        let scheme = Owned::new(unsafe { CFURLCopyScheme(url.0) })?;
        let file = Owned::string("file")?;
        if unsafe { CFEqual(scheme.0, file.0) } == 0 {
            return None;
        }
        let mut buffer = vec![0; 32768];
        if unsafe {
            CFURLGetFileSystemRepresentation(url.0, 1, buffer.as_mut_ptr(), buffer.len() as isize)
        } == 0
        {
            return None;
        }
        let end = buffer.iter().position(|byte| *byte == 0)?;
        buffer.truncate(end);
        Some(PathBuf::from(OsString::from_vec(buffer)))
    }

    pub(super) fn original_program(typeface: &Typeface) -> Option<(Vec<u8>, u32)> {
        let path = source_path(typeface)?;
        let file = std::fs::File::open(path).ok()?;
        let length = file.metadata().ok()?.len();
        if length == 0 || length > MAX_BYTES {
            return None;
        }
        // metadata 확인 뒤 파일이 바뀌어도 읽기 상한은 유지한다.
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1).read_to_end(&mut bytes).ok()?;
        if bytes.len() as u64 > MAX_BYTES {
            return None;
        }
        let count = ttf_parser::fonts_in_collection(&bytes).unwrap_or(1);
        if count == 0 || count > MAX_FACES {
            return None;
        }
        let tags = typeface.read_table_tags()?;
        for index in 0..count {
            let Ok(face) = ttf_parser::Face::parse(&bytes, index) else {
                continue;
            };
            let source = face.raw_face();
            if source.table_records.len() as usize != tags.len() {
                continue;
            }
            let same = tags.iter().all(|tag| {
                let Some(actual) = typeface.copy_table_data(*tag) else {
                    return false;
                };
                source.table(ttf_parser::Tag(*tag)) == Some(actual.as_bytes())
            });
            if same {
                return Some((bytes, index));
            }
        }
        None
    }
}

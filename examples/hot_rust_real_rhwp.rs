use rhwp::{DocumentCore, HwpError};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fmt, io};

const PATCHABLE_ENTRY_BYTES: usize = 16;
const PATCH_EXPORT: &str = "hot_rust_patch_render_page_svg_native";
const MARKER: &str = "hot-rust-real-rhwp-patched";

type RenderPageFn = fn(&DocumentCore, u32) -> Result<String, HwpError>;

#[derive(Debug)]
enum PatchError {
    #[cfg(target_arch = "aarch64")]
    MisalignedArm64Branch {
        old_fn: usize,
    },
    MissingPatchPadding {
        entry: Vec<u8>,
        needed: usize,
    },
    Protect(&'static str, io::Error),
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(target_arch = "aarch64")]
            Self::MisalignedArm64Branch { old_fn } => {
                write!(f, "ARM64 branch source is not 4-byte aligned ({old_fn:#x})")
            }
            Self::MissingPatchPadding { entry, needed } => write!(
                f,
                "function entry does not start with at least {needed} bytes of recognized patch padding: {entry:02x?}"
            ),
            Self::Protect(op, source) => write!(f, "{op} failed: {source}"),
        }
    }
}

impl Error for PatchError {}

struct PatchBytes {
    bytes: Vec<u8>,
    kind: &'static str,
}

struct BuiltPatch {
    root: PathBuf,
    dylib: PathBuf,
}

impl Drop for BuiltPatch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let sample = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "samples/para-001.hwp".to_string());
    let page_num = std::env::args()
        .nth(2)
        .map(|raw| raw.parse::<u32>())
        .transpose()?
        .unwrap_or(0);

    println!("sample: {sample}");
    println!("page: {page_num}");

    let bytes = fs::read(&sample)?;
    let core = hwp(DocumentCore::from_bytes(&bytes), "parse sample")?;
    println!("page_count: {}", core.page_count());

    let original_fn: RenderPageFn = DocumentCore::render_page_svg_native;
    let old_addr = original_fn as *const () as usize;
    println!("target: DocumentCore::render_page_svg_native = {old_addr:#x}");

    unsafe {
        let entry = core::slice::from_raw_parts(old_addr as *const u8, PATCHABLE_ENTRY_BYTES);
        println!("entry bytes before: {entry:02x?}");
    }

    let before = hwp(original_fn(&core, page_num), "render before patch")?;
    println!(
        "before: len={} marker={}",
        before.len(),
        before.contains(MARKER)
    );
    assert!(
        !before.contains(MARKER),
        "sample already contains the patch marker before patching"
    );

    let patch = build_patch_dylib(PATCH_EXPORT)?;
    println!("patch dylib: {}", patch.dylib.display());

    let library = unsafe { dylib::Library::open(&patch.dylib)? };
    let new_addr = unsafe { library.symbol(PATCH_EXPORT)? as usize };
    let replacement: RenderPageFn = unsafe { std::mem::transmute(new_addr) };
    println!("{PATCH_EXPORT} = {new_addr:#x}");
    println!("image distance: {:#x} bytes", old_addr.abs_diff(new_addr));

    let direct = hwp(replacement(&core, page_num), "direct replacement call")?;
    println!(
        "direct dylib: len={} marker={}",
        direct.len(),
        direct.contains(MARKER)
    );
    assert!(
        direct.contains(MARKER),
        "replacement dylib did not return the expected marker"
    );

    unsafe {
        patch_to_external(old_addr, new_addr)?;
    }

    unsafe {
        let entry = core::slice::from_raw_parts(old_addr as *const u8, PATCHABLE_ENTRY_BYTES);
        println!("entry bytes after:  {entry:02x?}");
    }

    let after_via_ptr = hwp(
        original_fn(&core, page_num),
        "render after patch via fn ptr",
    )?;
    let after_via_method = hwp(
        core.render_page_svg_native(page_num),
        "render after patch via method",
    )?;
    println!(
        "after via fn ptr: len={} marker={}",
        after_via_ptr.len(),
        after_via_ptr.contains(MARKER)
    );
    println!(
        "after via method: len={} marker={}",
        after_via_method.len(),
        after_via_method.contains(MARKER)
    );

    assert!(
        after_via_ptr.contains(MARKER),
        "function pointer call did not route through the patched entry"
    );
    assert!(
        after_via_method.contains(MARKER),
        "method call did not route through the patched entry"
    );

    println!("OK: patched the real rhwp render_page_svg_native entry in-process.");

    drop(library);
    drop(patch);
    Ok(())
}

fn hwp<T>(result: Result<T, HwpError>, context: &str) -> Result<T, Box<dyn Error>> {
    result.map_err(|err| format!("{context}: {err}").into())
}

unsafe fn patch_to_external(old_fn: usize, new_fn: usize) -> Result<(), PatchError> {
    let patch = encode_absolute_jump(old_fn, new_fn)?;
    if patch.bytes.len() > PATCHABLE_ENTRY_BYTES {
        return Err(PatchError::MissingPatchPadding {
            entry: Vec::new(),
            needed: patch.bytes.len(),
        });
    }

    let site = old_fn as *mut u8;
    let entry = core::slice::from_raw_parts(site, PATCHABLE_ENTRY_BYTES);
    if !has_patch_padding(entry, patch.bytes.len()) {
        return Err(PatchError::MissingPatchPadding {
            entry: entry.to_vec(),
            needed: patch.bytes.len(),
        });
    }

    println!(
        "patch: {old_fn:#x} -> {new_fn:#x}, kind {}, bytes {:02x?}",
        patch.kind, patch.bytes
    );
    platform::write_code(site, &patch.bytes).map_err(|err| PatchError::Protect("code patch", err))
}

#[cfg(target_arch = "x86_64")]
fn encode_absolute_jump(_old_fn: usize, new_fn: usize) -> Result<PatchBytes, PatchError> {
    let mut bytes = Vec::with_capacity(14);
    bytes.extend_from_slice(&[0xff, 0x25, 0x00, 0x00, 0x00, 0x00]);
    bytes.extend_from_slice(&(new_fn as u64).to_le_bytes());

    Ok(PatchBytes {
        bytes,
        kind: "x86_64 jmp [rip+0] absolute",
    })
}

#[cfg(target_arch = "aarch64")]
fn encode_absolute_jump(old_fn: usize, new_fn: usize) -> Result<PatchBytes, PatchError> {
    if old_fn % 4 != 0 {
        return Err(PatchError::MisalignedArm64Branch { old_fn });
    }

    let mut bytes = Vec::with_capacity(16);
    bytes.extend_from_slice(&0x5800_0050_u32.to_le_bytes());
    bytes.extend_from_slice(&0xd61f_0200_u32.to_le_bytes());
    bytes.extend_from_slice(&(new_fn as u64).to_le_bytes());

    Ok(PatchBytes {
        bytes,
        kind: "aarch64 ldr literal + br absolute",
    })
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("absolute jump encoders are implemented for x86_64 and aarch64 only");

#[cfg(target_arch = "x86_64")]
fn has_patch_padding(entry: &[u8], needed: usize) -> bool {
    let mut offset = 0;
    while offset < needed {
        let Some(len) = x86_nop_len(&entry[offset..]) else {
            return false;
        };
        offset += len;
    }
    true
}

#[cfg(target_arch = "x86_64")]
fn x86_nop_len(bytes: &[u8]) -> Option<usize> {
    let mut i = 0;
    while matches!(bytes.get(i), Some(0x66 | 0x2e | 0x3e | 0x26 | 0x64 | 0x65)) {
        i += 1;
    }

    match bytes.get(i)? {
        0x90 => Some(i + 1),
        0x0f if bytes.get(i + 1) == Some(&0x1f) => {
            let modrm = *bytes.get(i + 2)?;
            let mode = modrm >> 6;
            let rm = modrm & 0b111;

            let mut len = i + 3;
            if mode != 0b11 && rm == 0b100 {
                len += 1;
            }

            len += match (mode, rm) {
                (0b00, 0b101) => 4,
                (0b01, _) => 1,
                (0b10, _) => 4,
                _ => 0,
            };

            (len <= bytes.len()).then_some(len)
        }
        _ => None,
    }
}

#[cfg(target_arch = "aarch64")]
fn has_patch_padding(entry: &[u8], needed: usize) -> bool {
    const ARM64_NOP: [u8; 4] = [0x1f, 0x20, 0x03, 0xd5];

    let needed_instructions = needed.div_ceil(4);
    entry
        .chunks_exact(4)
        .take(needed_instructions)
        .all(|instruction| instruction == ARM64_NOP)
}

fn build_patch_dylib(export_symbol: &str) -> Result<BuiltPatch, Box<dyn Error>> {
    validate_rust_export_identifier(export_symbol)?;

    let root = std::env::temp_dir().join(format!(
        "hot-rust-real-rhwp-patch-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir)?;

    let rhwp_path = toml_string(&std::env::current_dir()?);
    fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "hot-rust-real-rhwp-patch"
version = "0.1.0"
edition = "2021"

[lib]
name = "hot_rust_real_rhwp_patch"
crate-type = ["cdylib"]

[dependencies]
rhwp = {{ path = "{rhwp_path}" }}

[profile.dev]
opt-level = 0
"#
        ),
    )?;

    fs::write(
        src_dir.join("lib.rs"),
        format!(
            r#"use rhwp::{{DocumentCore, HwpError}};

const MARKER: &str = "{MARKER}";

#[no_mangle]
pub fn {export_symbol}(_core: &DocumentCore, page_num: u32) -> Result<String, HwpError> {{
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" data-page=\"{{}}\"><!-- {{}} --></svg>",
        page_num,
        MARKER
    ))
}}
"#
        ),
    )?;

    let mut command = Command::new(cargo_command());
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env("RUSTC_BOOTSTRAP", "1")
        .env("RUSTFLAGS", "-Zpatchable-function-entry=16");

    println!("building patch crate with cargo...");
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!(
            "patch cargo build failed with {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let dylib = root
        .join("target")
        .join("debug")
        .join(dylib_filename("hot_rust_real_rhwp_patch"));
    if !dylib.exists() {
        return Err(format!("patch dylib was not produced at {}", dylib.display()).into());
    }

    Ok(BuiltPatch { root, dylib })
}

fn validate_rust_export_identifier(symbol: &str) -> Result<(), Box<dyn Error>> {
    let mut chars = symbol.chars();
    let Some(first) = chars.next() else {
        return Err("empty patch export symbol".into());
    };

    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(format!("patch export is not a Rust identifier: {symbol}").into());
    }

    if chars.any(|ch| !(ch == '_' || ch.is_ascii_alphanumeric())) {
        return Err(format!("patch export is not a Rust identifier: {symbol}").into());
    }

    Ok(())
}

fn cargo_command() -> std::ffi::OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

fn dylib_filename(name: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        format!("lib{name}.dylib")
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        format!("lib{name}.so")
    }

    #[cfg(windows)]
    {
        format!("{name}.dll")
    }
}

fn toml_string(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

#[cfg(unix)]
mod dylib {
    use std::ffi::{CStr, CString};
    use std::io;
    use std::os::raw::{c_char, c_int, c_void};
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    const RTLD_NOW: c_int = 0x2;
    const RTLD_LOCAL: c_int = 0x4;

    #[cfg_attr(target_os = "linux", link(name = "dl"))]
    extern "C" {
        fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> c_int;
        fn dlerror() -> *const c_char;
    }

    pub struct Library {
        handle: *mut c_void,
    }

    impl Library {
        pub unsafe fn open(path: &Path) -> io::Result<Self> {
            clear_error();
            let path = CString::new(path.as_os_str().as_bytes())?;
            let handle = dlopen(path.as_ptr(), RTLD_NOW | RTLD_LOCAL);
            if handle.is_null() {
                return Err(last_error());
            }

            Ok(Self { handle })
        }

        pub unsafe fn symbol(&self, name: &str) -> io::Result<*mut c_void> {
            clear_error();
            let name = CString::new(name)?;
            let symbol = dlsym(self.handle, name.as_ptr());
            if symbol.is_null() {
                return Err(last_error());
            }

            Ok(symbol)
        }
    }

    impl Drop for Library {
        fn drop(&mut self) {
            unsafe {
                dlclose(self.handle);
            }
        }
    }

    unsafe fn clear_error() {
        let _ = dlerror();
    }

    unsafe fn last_error() -> io::Error {
        let err = dlerror();
        if err.is_null() {
            io::Error::last_os_error()
        } else {
            io::Error::other(CStr::from_ptr(err).to_string_lossy().into_owned())
        }
    }
}

#[cfg(windows)]
mod dylib {
    use std::ffi::CString;
    use std::io;
    use std::path::Path;

    extern "system" {
        fn LoadLibraryA(filename: *const i8) -> isize;
        fn GetProcAddress(module: isize, proc_name: *const i8) -> *mut std::ffi::c_void;
        fn FreeLibrary(module: isize) -> i32;
    }

    pub struct Library {
        handle: isize,
    }

    impl Library {
        pub unsafe fn open(path: &Path) -> io::Result<Self> {
            let path = CString::new(path.to_string_lossy().as_bytes())?;
            let handle = LoadLibraryA(path.as_ptr());
            if handle == 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(Self { handle })
        }

        pub unsafe fn symbol(&self, name: &str) -> io::Result<*mut std::ffi::c_void> {
            let name = CString::new(name)?;
            let symbol = GetProcAddress(self.handle, name.as_ptr());
            if symbol.is_null() {
                return Err(io::Error::last_os_error());
            }

            Ok(symbol)
        }
    }

    impl Drop for Library {
        fn drop(&mut self) {
            unsafe {
                FreeLibrary(self.handle);
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::io;

    const PAGE_EXECUTE_READWRITE: u32 = 0x40;

    extern "system" {
        fn VirtualProtect(addr: *mut u8, size: usize, new_prot: u32, old_prot: *mut u32) -> i32;
        fn FlushInstructionCache(process: isize, addr: *const u8, size: usize) -> i32;
        fn GetCurrentProcess() -> isize;
    }

    pub unsafe fn write_code(site: *mut u8, bytes: &[u8]) -> io::Result<()> {
        let mut old_prot = 0_u32;
        if VirtualProtect(
            site,
            super::PATCHABLE_ENTRY_BYTES,
            PAGE_EXECUTE_READWRITE,
            &mut old_prot,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }

        core::ptr::copy_nonoverlapping(bytes.as_ptr(), site, bytes.len());

        let mut restored_prot = 0_u32;
        if VirtualProtect(
            site,
            super::PATCHABLE_ENTRY_BYTES,
            old_prot,
            &mut restored_prot,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }

        if FlushInstructionCache(GetCurrentProcess(), site, super::PATCHABLE_ENTRY_BYTES) == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::c_void;
    use std::io;

    const PROT_READ: i32 = 0x01;
    const PROT_WRITE: i32 = 0x02;
    const PROT_EXEC: i32 = 0x04;

    const VM_PROT_READ: i32 = 0x01;
    const VM_PROT_WRITE: i32 = 0x02;
    const VM_PROT_EXECUTE: i32 = 0x04;
    const VM_PROT_COPY: i32 = 0x10;

    const _SC_PAGESIZE: i32 = 29;
    const KERN_SUCCESS: i32 = 0;
    const KERN_NO_SPACE: i32 = 3;
    const VM_FLAGS_FIXED: i32 = 0x0000;
    const VM_FLAGS_ANYWHERE: i32 = 0x0001;
    const VM_FLAGS_OVERWRITE: i32 = 0x4000;
    const VM_INHERIT_COPY: i32 = 1;

    extern "C" {
        static mach_task_self_: u32;
        fn mach_vm_allocate(target: u32, address: *mut u64, size: u64, flags: i32) -> i32;
        fn mach_vm_deallocate(target: u32, address: u64, size: u64) -> i32;
        fn mach_vm_protect(
            target_task: u32,
            address: u64,
            size: u64,
            set_maximum: i32,
            new_protection: i32,
        ) -> i32;
        fn mach_vm_remap(
            target_task: u32,
            target_address: *mut u64,
            size: u64,
            mask: u64,
            flags: i32,
            src_task: u32,
            src_address: u64,
            copy: i32,
            cur_protection: *mut i32,
            max_protection: *mut i32,
            inheritance: i32,
        ) -> i32;
        fn mach_vm_write(target_task: u32, address: u64, data: usize, data_count: u32) -> i32;
        fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
        fn sysconf(name: i32) -> isize;
    }

    #[cfg(target_arch = "aarch64")]
    extern "C" {
        fn sys_dcache_flush(start: *mut c_void, len: usize);
        fn sys_icache_invalidate(start: *mut c_void, len: usize);
    }

    pub unsafe fn write_code(site: *mut u8, bytes: &[u8]) -> io::Result<()> {
        let (page_start, page_len) = page_span(site as usize, super::PATCHABLE_ENTRY_BYTES);

        if let Err(protect_err) = protect(
            page_start,
            page_len,
            PROT_READ | PROT_WRITE | PROT_EXEC,
            VM_PROT_READ | VM_PROT_WRITE | VM_PROT_EXECUTE | VM_PROT_COPY,
        ) {
            return match write_with_mach_vm(site, bytes, &protect_err) {
                Ok(()) => Ok(()),
                Err(write_err) => write_with_remapped_copy(site, bytes, &write_err),
            };
        }

        core::ptr::copy_nonoverlapping(bytes.as_ptr(), site, bytes.len());
        flush_instruction_cache(site, bytes.len());

        protect(
            page_start,
            page_len,
            PROT_READ | PROT_EXEC,
            VM_PROT_READ | VM_PROT_EXECUTE,
        )?;

        Ok(())
    }

    fn page_span(addr: usize, len: usize) -> (usize, usize) {
        let page_size = unsafe { sysconf(_SC_PAGESIZE) };
        let page_size = if page_size > 0 {
            page_size as usize
        } else {
            16 * 1024
        };
        let page_start = addr & !(page_size - 1);
        let page_end = (addr + len + page_size - 1) & !(page_size - 1);
        (page_start, page_end - page_start)
    }

    unsafe fn protect(
        page_start: usize,
        page_len: usize,
        mprotect_prot: i32,
        vm_prot: i32,
    ) -> io::Result<()> {
        if mprotect(page_start as *mut c_void, page_len, mprotect_prot) == 0 {
            return Ok(());
        }

        let mprotect_err = io::Error::last_os_error();
        let kr = mach_vm_protect(
            mach_task_self_,
            page_start as u64,
            page_len as u64,
            0,
            vm_prot,
        );
        if kr == 0 {
            return Ok(());
        }

        let max_kr = if vm_prot & VM_PROT_WRITE != 0 {
            mach_vm_protect(
                mach_task_self_,
                page_start as u64,
                page_len as u64,
                1,
                vm_prot,
            )
        } else {
            kr
        };
        if max_kr == 0 {
            let retry_kr = mach_vm_protect(
                mach_task_self_,
                page_start as u64,
                page_len as u64,
                0,
                vm_prot,
            );
            if retry_kr == 0 {
                return Ok(());
            }

            return Err(io::Error::new(
                mprotect_err.kind(),
                format!(
                    "mprotect failed with {mprotect_err}; mach_vm_protect current returned {kr}; max returned {max_kr}; retry returned {retry_kr}"
                ),
            ));
        }

        Err(io::Error::new(
            mprotect_err.kind(),
            format!(
                "mprotect failed with {mprotect_err}; mach_vm_protect current returned {kr}; max returned {max_kr}"
            ),
        ))
    }

    unsafe fn write_with_mach_vm(
        site: *mut u8,
        bytes: &[u8],
        protect_err: &io::Error,
    ) -> io::Result<()> {
        let kr = mach_vm_write(
            mach_task_self_,
            site as u64,
            bytes.as_ptr() as usize,
            bytes.len() as u32,
        );
        if kr != 0 {
            return Err(io::Error::new(
                protect_err.kind(),
                format!("{protect_err}; mach_vm_write returned {kr}"),
            ));
        }

        flush_instruction_cache(site, bytes.len());
        Ok(())
    }

    unsafe fn write_with_remapped_copy(
        site: *mut u8,
        bytes: &[u8],
        prior_err: &io::Error,
    ) -> io::Result<()> {
        let (page_start, page_len) = page_span(site as usize, super::PATCHABLE_ENTRY_BYTES);
        let page_offset = (site as usize) - page_start;

        let mut scratch = 0_u64;
        let alloc_kr = mach_vm_allocate(
            mach_task_self_,
            &mut scratch,
            page_len as u64,
            VM_FLAGS_ANYWHERE,
        );
        if alloc_kr != KERN_SUCCESS {
            return Err(io::Error::new(
                prior_err.kind(),
                format!("{prior_err}; frida-style scratch allocation returned {alloc_kr}"),
            ));
        }

        core::ptr::copy_nonoverlapping(page_start as *const u8, scratch as *mut u8, page_len);
        core::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            (scratch as usize + page_offset) as *mut u8,
            bytes.len(),
        );
        flush_caches(scratch as *mut u8, page_len);

        let protect_scratch_kr = mach_vm_protect(
            mach_task_self_,
            scratch,
            page_len as u64,
            0,
            VM_PROT_READ | VM_PROT_EXECUTE,
        );
        if protect_scratch_kr != KERN_SUCCESS {
            mach_vm_deallocate(mach_task_self_, scratch, page_len as u64);
            return Err(io::Error::new(
                prior_err.kind(),
                format!("{prior_err}; frida-style scratch RX returned {protect_scratch_kr}"),
            ));
        }

        let mut target = page_start as u64;
        let mut cur = 0_i32;
        let mut max = 0_i32;
        let mut remap_kr = mach_vm_remap(
            mach_task_self_,
            &mut target,
            page_len as u64,
            0,
            VM_FLAGS_OVERWRITE | VM_FLAGS_FIXED,
            mach_task_self_,
            scratch,
            1,
            &mut cur,
            &mut max,
            VM_INHERIT_COPY,
        );

        if remap_kr == KERN_NO_SPACE {
            let preprotect_kr = mach_vm_protect(
                mach_task_self_,
                page_start as u64,
                page_len as u64,
                0,
                VM_PROT_READ | VM_PROT_WRITE | VM_PROT_COPY,
            );
            if preprotect_kr == KERN_SUCCESS {
                target = page_start as u64;
                remap_kr = mach_vm_remap(
                    mach_task_self_,
                    &mut target,
                    page_len as u64,
                    0,
                    VM_FLAGS_OVERWRITE | VM_FLAGS_FIXED,
                    mach_task_self_,
                    scratch,
                    1,
                    &mut cur,
                    &mut max,
                    VM_INHERIT_COPY,
                );

                let _ = mach_vm_protect(
                    mach_task_self_,
                    page_start as u64,
                    page_len as u64,
                    0,
                    VM_PROT_READ | VM_PROT_EXECUTE,
                );
            }
        }

        mach_vm_deallocate(mach_task_self_, scratch, page_len as u64);

        if remap_kr != KERN_SUCCESS {
            return Err(io::Error::new(
                prior_err.kind(),
                format!("{prior_err}; frida-style remap copy returned {remap_kr}"),
            ));
        }

        flush_caches(page_start as *mut u8, page_len);
        println!("code patch: direct write failed ({prior_err}); frida-style remap copy succeeded");
        Ok(())
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn flush_instruction_cache(site: *mut u8, len: usize) {
        sys_icache_invalidate(site as *mut c_void, len);
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn flush_instruction_cache(_site: *mut u8, _len: usize) {}

    #[cfg(target_arch = "aarch64")]
    unsafe fn flush_caches(site: *mut u8, len: usize) {
        sys_dcache_flush(site as *mut c_void, len);
        sys_icache_invalidate(site as *mut c_void, len);
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn flush_caches(_site: *mut u8, _len: usize) {}
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod platform {
    use std::io;

    pub unsafe fn write_code(_site: *mut u8, _bytes: &[u8]) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "code-memory writes are implemented for Windows and macOS only",
        ))
    }
}

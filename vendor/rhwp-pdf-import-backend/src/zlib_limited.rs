//! Private zlib-rs state with request-scoped, fallible allocation.
//!
//! Audited against exactly 0.6.7: inflate/deflate init each allocate one block
//! containing state and all scratch space. Returned blocks may be zeroed and
//! reused by another synchronous codec in the same request. Copy/reset/dictionary APIs are not
//! exposed here. Re-audit the allocation and pointer contracts on an upgrade.

use crate::error::{Error, Result};
use crate::import_limits::ImportBudget;
use std::ffi::{c_uint, c_void};
use zlib_rs::{
    c_api::z_stream, deflate, inflate, DeflateConfig, DeflateFlush, InflateConfig, InflateFlush,
    ReturnCode,
};

pub(crate) struct Memory<'a> {
    budget: &'a ImportBudget,
    bytes: Vec<u8>,
}

impl<'a> Memory<'a> {
    pub(crate) fn new(budget: &'a ImportBudget) -> Self {
        Self {
            budget,
            bytes: Vec::new(),
        }
    }
}

unsafe extern "C" fn allocate(opaque: *mut c_void, count: c_uint, size: c_uint) -> *mut c_void {
    // SAFETY: Stream exclusively borrows this live, stationary context through
    // all synchronous codec calls, including init and end.
    let memory = unsafe { &mut *opaque.cast::<Memory<'_>>() };
    let Some(bytes) = (count as usize)
        .checked_mul(size as usize)
        .filter(|&n| n > 0)
    else {
        memory.budget.reject("zlib allocation size");
        return std::ptr::null_mut();
    };
    if !memory.bytes.is_empty() {
        memory.budget.reject("unexpected zlib allocation");
        return std::ptr::null_mut();
    }
    match memory.budget.take_codec_scratch(bytes) {
        Ok(Some(buffer)) => memory.bytes = buffer,
        Ok(None) => {
            if memory.budget.charge_codec_allocation(bytes).is_err() {
                return std::ptr::null_mut();
            }
            if memory.bytes.try_reserve_exact(bytes).is_err() {
                memory.budget.reject("zlib allocation");
                return std::ptr::null_mut();
            }
        },
        Err(_) => return std::ptr::null_mut(),
    }
    // zlib-rs requests alignment overhead and aligns the returned byte buffer
    // internally. Charge the entire request, including that overhead.
    memory.bytes.resize(bytes, 0);
    memory.bytes.fill(0);
    memory.bytes.as_mut_ptr().cast()
}

unsafe extern "C" fn free(opaque: *mut c_void, pointer: *mut c_void) {
    // SAFETY: zlib-rs returns the original allocation pointer, not its aligned
    // state pointer. The context outlives end and is exclusively borrowed.
    let memory = unsafe { &mut *opaque.cast::<Memory<'_>>() };
    if pointer == memory.bytes.as_mut_ptr().cast() {
        memory
            .budget
            .recycle_codec_scratch(std::mem::take(&mut memory.bytes));
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Deflate,
    Inflate { raw: bool },
}

pub(crate) struct Stream<'a> {
    stream: z_stream,
    budget: &'a ImportBudget,
    mode: Mode,
    // The context cannot move or be dropped until codec teardown has run.
    _memory: std::marker::PhantomData<&'a mut Memory<'a>>,
}

impl<'a> Stream<'a> {
    pub(crate) fn deflater(memory: &'a mut Memory<'a>) -> Result<Self> {
        Self::new(memory, Mode::Deflate)
    }

    pub(crate) fn inflater(memory: &'a mut Memory<'a>, raw: bool) -> Result<Self> {
        Self::new(memory, Mode::Inflate { raw })
    }

    fn new(memory: &'a mut Memory<'a>, mode: Mode) -> Result<Self> {
        memory.budget.check()?;
        let mut this = Self {
            stream: z_stream {
                zalloc: Some(allocate),
                zfree: Some(free),
                opaque: (memory as *mut Memory<'a>).cast(),
                ..z_stream::default()
            },
            budget: memory.budget,
            mode,
            _memory: std::marker::PhantomData,
        };
        let status = match mode {
            Mode::Deflate => deflate::init(&mut this.stream, DeflateConfig::default()),
            Mode::Inflate { raw } => inflate::init(
                &mut this.stream,
                InflateConfig {
                    window_bits: if raw { -15 } else { 15 },
                },
            ),
        };
        this.budget.check()?;
        if status != ReturnCode::Ok {
            return Err(Error::Decode("bounded zlib initialization failed".into()));
        }
        Ok(this)
    }

    /// One synchronous step, borrowing buffers only for this call. Lengths in
    /// z_stream are u32; callers retain and feed any remainder on 64-bit hosts.
    pub(crate) fn step(
        &mut self,
        input: &[u8],
        output: &mut [u8],
        finish: bool,
    ) -> Result<(ReturnCode, usize, usize)> {
        self.budget.check()?;
        let input_len = input.len().min(u32::MAX as usize);
        let output_len = output.len().min(u32::MAX as usize);
        self.stream.next_in = input.as_ptr();
        self.stream.avail_in = input_len as u32;
        self.stream.next_out = output.as_mut_ptr();
        self.stream.avail_out = output_len as u32;
        // SAFETY: successful init chose the matching mode; buffers are live,
        // initialized, disjoint, and at least as long as the advertised sizes.
        // No state/header/dictionary pointer is exposed to callers.
        let status = unsafe {
            match self.mode {
                Mode::Deflate => {
                    deflate::DeflateStream::from_stream_mut(&mut self.stream).map(|state| {
                        deflate::deflate(
                            state,
                            if finish {
                                DeflateFlush::Finish
                            } else {
                                DeflateFlush::NoFlush
                            },
                        )
                    })
                },
                Mode::Inflate { .. } => inflate::InflateStream::from_stream_mut(&mut self.stream)
                    .map(|state| inflate::inflate(state, InflateFlush::NoFlush)),
            }
        };
        let consumed = input_len - self.stream.avail_in as usize;
        let written = output_len - self.stream.avail_out as usize;
        self.stream.next_in = std::ptr::null();
        self.stream.avail_in = 0;
        self.stream.next_out = std::ptr::null_mut();
        self.stream.avail_out = 0;
        self.budget.check()?;
        Ok((
            status.ok_or_else(|| Error::Decode("invalid bounded zlib state".into()))?,
            consumed,
            written,
        ))
    }
}

impl Drop for Stream<'_> {
    fn drop(&mut self) {
        // SAFETY: the state belongs to the matching init. Failed init leaves
        // state null or already frees it. end never dereferences input/output,
        // and the allocator context is still live here.
        unsafe {
            match self.mode {
                Mode::Deflate => {
                    if let Some(state) = deflate::DeflateStream::from_stream_mut(&mut self.stream) {
                        let _ = deflate::end(state);
                    }
                },
                Mode::Inflate { .. } => {
                    if let Some(state) = inflate::InflateStream::from_stream_mut(&mut self.stream) {
                        inflate::end(state);
                    }
                },
            }
        }
    }
}

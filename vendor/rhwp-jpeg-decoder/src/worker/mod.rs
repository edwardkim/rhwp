use crate::allocation::Allocator;
mod immediate;
#[cfg(not(target_arch = "wasm32"))]
mod multithreaded;
#[cfg(all(not(target_arch = "wasm32"), feature = "rayon"))]
mod rayon;

use crate::decoder::{choose_color_convert_func, ColorTransform};
use crate::error::Result;
use crate::parser::{Component, Dimensions};
use crate::upsampler::Upsampler;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;

pub struct RowData {
    pub index: usize,
    pub component: Component,
    pub quantization_table: Arc<[u16; 64]>,
    // Reserved on the request thread before handing ownership to a worker.
    pub result: Vec<u8>,
}

pub trait Worker {
    fn start(&mut self, row_data: RowData) -> Result<()>;
    fn append_row(&mut self, row: (usize, Vec<i16>)) -> Result<()>;
    fn get_result(&mut self, index: usize) -> Result<Vec<u8>>;
    /// Default implementation for spawning multiple tasks.
    fn append_rows(
        &mut self,
        row: &mut dyn Iterator<Item = Result<(usize, Vec<i16>)>>,
    ) -> Result<()> {
        for item in row {
            self.append_row(item?)?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub enum PreferWorkerKind {
    Immediate,
    Multithreaded,
}

#[derive(Default)]
pub struct WorkerScope {
    inner: core::cell::RefCell<Option<WorkerScopeInner>>,
}

enum WorkerScopeInner {
    #[cfg(all(not(target_arch = "wasm32"), feature = "rayon"))]
    Rayon(Box<rayon::Scoped>),
    #[cfg(not(target_arch = "wasm32"))]
    Multithreaded(multithreaded::MpscWorker),
    Immediate(Box<immediate::ImmediateWorker>),
}

impl WorkerScope {
    pub fn with<T>(with: impl FnOnce(&Self) -> T) -> T {
        with(&WorkerScope {
            inner: RefCell::default(),
        })
    }

    pub fn get_or_init_worker<T>(
        &self,
        prefer: PreferWorkerKind,
        alloc: Allocator<'_>,
        f: impl FnOnce(&mut dyn Worker) -> Result<T>,
    ) -> Result<T> {
        let mut inner = self.inner.borrow_mut();
        if inner.is_none() {
            let worker = match prefer {
                #[cfg(all(not(target_arch = "wasm32"), feature = "rayon"))]
                PreferWorkerKind::Multithreaded => {
                    alloc.charge(core::mem::size_of::<rayon::Scoped>(), "rayon worker")?;
                    WorkerScopeInner::Rayon(Default::default())
                }
                #[allow(unreachable_patterns)]
                #[cfg(not(target_arch = "wasm32"))]
                PreferWorkerKind::Multithreaded => {
                    WorkerScopeInner::Multithreaded(Default::default())
                }
                _ => {
                    alloc.charge(
                        core::mem::size_of::<immediate::ImmediateWorker>(),
                        "immediate worker",
                    )?;
                    WorkerScopeInner::Immediate(Default::default())
                }
            };
            *inner = Some(worker);
        }
        f(match inner.as_mut().unwrap() {
            #[cfg(all(not(target_arch = "wasm32"), feature = "rayon"))]
            WorkerScopeInner::Rayon(worker) => worker.as_mut(),
            #[cfg(not(target_arch = "wasm32"))]
            WorkerScopeInner::Multithreaded(worker) => worker,
            WorkerScopeInner::Immediate(worker) => worker.as_mut(),
        })
    }
}

pub fn compute_image_parallel(
    components: &[Component],
    data: Vec<Vec<u8>>,
    output_size: Dimensions,
    color_transform: ColorTransform,
    alloc: Allocator<'_>,
) -> Result<Vec<u8>> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "rayon"))]
    return rayon::compute_image_parallel(components, data, output_size, color_transform, alloc);

    #[allow(unreachable_code)]
    {
        let color_convert_func = choose_color_convert_func(components.len(), color_transform)?;
        let upsampler = Upsampler::new(components, output_size.width, output_size.height, alloc)?;
        let line_size = output_size.width as usize * components.len();
        let mut image = alloc.filled(
            0u8,
            alloc.product(&[line_size, output_size.height as usize])?,
            "interleaved output",
        )?;

        for (row, line) in image.chunks_mut(line_size).enumerate() {
            upsampler.upsample_and_interleave_row(
                &data,
                row,
                output_size.width as usize,
                line,
                color_convert_func,
            )?;
        }

        Ok(image)
    }
}

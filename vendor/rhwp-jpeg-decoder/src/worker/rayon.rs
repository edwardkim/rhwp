use crate::allocation::Allocator;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

use crate::decoder::{choose_color_convert_func, ColorTransform};
use crate::error::Result;
use crate::idct::dequantize_and_idct_block;
use crate::parser::Component;
use crate::upsampler::Upsampler;
use crate::{decoder::MAX_COMPONENTS, parser::Dimensions};

use std::sync::Arc;

use super::{RowData, Worker};

/// Technically similar to `immediate::ImmediateWorker` but we copy it since we may prefer
/// different style of managing the memory allocation, something that multiple actors can access in
/// parallel.
#[derive(Default)]
struct ImmediateWorker {
    offsets: [usize; MAX_COMPONENTS],
    results: [Vec<u8>; MAX_COMPONENTS],
    components: [Option<Component>; MAX_COMPONENTS],
    quantization_tables: [Option<Arc<[u16; 64]>>; MAX_COMPONENTS],
}

#[derive(Clone, Copy)]
struct ComponentMetadata {
    block_width: usize,
    block_count: usize,
    line_stride: usize,
    dct_scale: usize,
}

#[derive(Default)]
pub struct Scoped {
    inner: ImmediateWorker,
}

impl ImmediateWorker {
    pub fn start_immediate(&mut self, data: RowData) {
        self.offsets[data.index] = 0;
        self.results[data.index] = data.result;
        self.components[data.index] = Some(data.component);
        self.quantization_tables[data.index] = Some(data.quantization_table);
    }

    pub fn get_result_immediate(&mut self, index: usize) -> Vec<u8> {
        core::mem::take(&mut self.results[index])
    }

    pub fn component_metadata(&self, index: usize) -> Option<ComponentMetadata> {
        let component = self.components[index].as_ref()?;
        let block_size = component.block_size;
        let block_width = block_size.width as usize;
        let block_count = block_size.width as usize * component.vertical_sampling_factor as usize;
        let line_stride = block_size.width as usize * component.dct_scale;
        let dct_scale = component.dct_scale;

        Some(ComponentMetadata {
            block_width,
            block_count,
            line_stride,
            dct_scale,
        })
    }

    pub fn append_row_locked(
        quantization_table: Arc<[u16; 64]>,
        metadata: ComponentMetadata,
        data: Vec<i16>,
        result_block: &mut [u8],
    ) {
        // Convert coefficients from a MCU row to samples.
        let ComponentMetadata {
            block_count,
            line_stride,
            block_width,
            dct_scale,
        } = metadata;

        assert_eq!(data.len(), block_count * 64);

        let mut output_buffer = [0; 64];
        for i in 0..block_count {
            let x = (i % block_width) * dct_scale;
            let y = (i / block_width) * dct_scale;

            let coefficients: &[i16; 64] = &data[i * 64..(i + 1) * 64].try_into().unwrap();

            // Write to a temporary intermediate buffer, a 8x8 'image'.
            dequantize_and_idct_block(
                dct_scale,
                coefficients,
                &quantization_table,
                8,
                &mut output_buffer,
            );

            let write_back = &mut result_block[y * line_stride + x..];

            let buffered_lines = output_buffer.chunks_mut(8);
            let back_lines = write_back.chunks_mut(line_stride);

            for (buf, back) in buffered_lines.zip(back_lines).take(dct_scale) {
                back[..dct_scale].copy_from_slice(&buf[..dct_scale]);
            }
        }
    }
}

impl Worker for Scoped {
    fn start(&mut self, row_data: RowData) -> Result<()> {
        self.inner.start_immediate(row_data);
        Ok(())
    }

    fn append_row(&mut self, row: (usize, Vec<i16>)) -> Result<()> {
        let inner = &mut self.inner;
        let (index, data) = row;

        let quantization_table = inner.quantization_tables[index].as_ref().unwrap().clone();
        let metadata = inner.component_metadata(index).unwrap();
        let result_block = &mut inner.results[index][inner.offsets[index]..];
        inner.offsets[index] += metadata.bytes_used();

        ImmediateWorker::append_row_locked(quantization_table, metadata, data, result_block);
        Ok(())
    }

    fn get_result(&mut self, index: usize) -> Result<Vec<u8>> {
        let result = self.inner.get_result_immediate(index);
        Ok(result)
    }

    // Magic sauce, these _may_ run in parallel.
    fn append_rows(
        &mut self,
        iter: &mut dyn Iterator<Item = Result<(usize, Vec<i16>)>>,
    ) -> Result<()> {
        let inner = &mut self.inner;
        rayon::in_place_scope(|scope| {
            let metadatas = [
                inner.component_metadata(0),
                inner.component_metadata(1),
                inner.component_metadata(2),
                inner.component_metadata(3),
            ];

            let [res0, res1, res2, res3] = &mut inner.results;

            // Lazily get the blocks. Note: if we've already collected results from a component
            // then the result vector has already been deallocated/taken. But no more tasks should
            // be created for it.
            let mut result_blocks = [
                res0.get_mut(inner.offsets[0]..).unwrap_or(&mut []),
                res1.get_mut(inner.offsets[1]..).unwrap_or(&mut []),
                res2.get_mut(inner.offsets[2]..).unwrap_or(&mut []),
                res3.get_mut(inner.offsets[3]..).unwrap_or(&mut []),
            ];

            // First we schedule everything, making sure their index is right etc.
            for item in iter {
                let (index, data) = item?;
                let metadata = metadatas[index].unwrap();
                let quantization_table = inner.quantization_tables[index].as_ref().unwrap().clone();

                inner.offsets[index] += metadata.bytes_used();
                let (result_block, tail) =
                    core::mem::take(&mut result_blocks[index]).split_at_mut(metadata.bytes_used());
                result_blocks[index] = tail;

                scope.spawn(move |_| {
                    ImmediateWorker::append_row_locked(
                        quantization_table,
                        metadata,
                        data,
                        result_block,
                    )
                });
            }
            Ok(())
        })
    }
}

impl ComponentMetadata {
    fn bytes_used(&self) -> usize {
        self.block_count * self.dct_scale * self.dct_scale
    }
}

pub fn compute_image_parallel(
    components: &[Component],
    data: Vec<Vec<u8>>,
    output_size: Dimensions,
    color_transform: ColorTransform,
    alloc: Allocator<'_>,
) -> Result<Vec<u8>> {
    let color_convert_func = choose_color_convert_func(components.len(), color_transform)?;
    let upsampler = Upsampler::new(components, output_size.width, output_size.height, alloc)?;
    let line_size = output_size.width as usize * components.len();
    let mut image = alloc.filled(
        0u8,
        alloc.product(&[line_size, output_size.height as usize])?,
        "interleaved output",
    )?;

    image
        .par_chunks_mut(line_size)
        .with_max_len(1)
        .enumerate()
        .try_for_each(|(row, line)| {
            upsampler.upsample_and_interleave_row(
                &data,
                row,
                output_size.width as usize,
                line,
                color_convert_func,
            )
        })?;

    Ok(image)
}

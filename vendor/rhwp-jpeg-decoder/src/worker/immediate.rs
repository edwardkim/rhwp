use super::{RowData, Worker};
use crate::alloc::sync::Arc;
use crate::decoder::MAX_COMPONENTS;
use crate::error::Result;
use crate::idct::dequantize_and_idct_block;
use crate::parser::Component;
use alloc::vec::Vec;
use core::mem;

pub struct ImmediateWorker {
    offsets: [usize; MAX_COMPONENTS],
    results: [Vec<u8>; MAX_COMPONENTS],
    components: [Option<Component>; MAX_COMPONENTS],
    quantization_tables: [Option<Arc<[u16; 64]>>; MAX_COMPONENTS],
}

impl Default for ImmediateWorker {
    fn default() -> Self {
        ImmediateWorker {
            offsets: [0; MAX_COMPONENTS],
            results: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            components: [None, None, None, None],
            quantization_tables: [None, None, None, None],
        }
    }
}

impl ImmediateWorker {
    pub fn start_immediate(&mut self, data: RowData) {
        assert!(self.results[data.index].is_empty());

        self.offsets[data.index] = 0;
        self.results[data.index] = data.result;
        self.components[data.index] = Some(data.component);
        self.quantization_tables[data.index] = Some(data.quantization_table);
    }

    pub fn append_row_immediate(&mut self, (index, data): (usize, Vec<i16>)) {
        // Convert coefficients from a MCU row to samples.

        let component = self.components[index].as_ref().unwrap();
        let quantization_table = self.quantization_tables[index].as_ref().unwrap();
        let block_count =
            component.block_size.width as usize * component.vertical_sampling_factor as usize;
        let line_stride = component.block_size.width as usize * component.dct_scale;

        assert_eq!(data.len(), block_count * 64);

        for i in 0..block_count {
            let x = (i % component.block_size.width as usize) * component.dct_scale;
            let y = (i / component.block_size.width as usize) * component.dct_scale;

            let coefficients = data[i * 64..(i + 1) * 64].try_into().unwrap();
            let output = &mut self.results[index][self.offsets[index] + y * line_stride + x..];

            dequantize_and_idct_block(
                component.dct_scale,
                coefficients,
                quantization_table,
                line_stride,
                output,
            );
        }

        self.offsets[index] += block_count * component.dct_scale * component.dct_scale;
    }

    pub fn get_result_immediate(&mut self, index: usize) -> Vec<u8> {
        mem::take(&mut self.results[index])
    }
}

impl Worker for ImmediateWorker {
    fn start(&mut self, data: RowData) -> Result<()> {
        self.start_immediate(data);
        Ok(())
    }
    fn append_row(&mut self, row: (usize, Vec<i16>)) -> Result<()> {
        self.append_row_immediate(row);
        Ok(())
    }
    fn get_result(&mut self, index: usize) -> Result<Vec<u8>> {
        Ok(self.get_result_immediate(index))
    }
}

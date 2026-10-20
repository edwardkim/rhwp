use crate::allocation::Allocator;
use crate::decoder::{Decoder, MAX_COMPONENTS};
use crate::error::{Error, Result};
use crate::huffman::HuffmanDecoder;
use crate::marker::Marker;
use crate::parser::Predictor;
use crate::parser::{Component, FrameInfo, ScanInfo};
use std::io::Read;

impl<R: Read> Decoder<'_, R> {
    /// decode_scan_lossless
    pub fn decode_scan_lossless(
        &mut self,
        frame: &FrameInfo,
        scan: &ScanInfo,
    ) -> Result<(Option<Marker>, Vec<Vec<u16>>)> {
        let ncomp = scan.component_indices.len();
        let npixel = self.allocation.product(&[
            frame.image_size.height as usize,
            frame.image_size.width as usize,
        ])?;
        assert!(ncomp <= MAX_COMPONENTS);
        let mut results = self
            .allocation
            .filled(Vec::new(), ncomp, "lossless result slots")?;
        for plane in &mut results {
            *plane = self.allocation.filled(0u16, npixel, "lossless plane")?;
        }

        let components: Vec<Component> = self.allocation.collect(
            scan.component_indices
                .iter()
                .map(|&i| frame.components[i].clone()),
            "lossless scan components",
        )?;

        // Verify that all required huffman tables has been set.
        if scan
            .dc_table_indices
            .iter()
            .any(|&i| self.dc_huffman_tables[i].is_none())
        {
            return Err(Error::Format(
                "scan makes use of unset dc huffman table".to_owned(),
            ));
        }

        let mut huffman = HuffmanDecoder::new();
        let reader = &mut self.reader;
        let mut mcus_left_until_restart = self.restart_interval;
        let mut expected_rst_num = 0;
        let mut ra = [0u16; MAX_COMPONENTS];
        let mut rb = [0u16; MAX_COMPONENTS];
        let mut rc = [0u16; MAX_COMPONENTS];

        let width = frame.image_size.width as usize;
        let height = frame.image_size.height as usize;

        let mut differences =
            self.allocation
                .filled(Vec::new(), ncomp, "lossless difference slots")?;
        for plane in &mut differences {
            *plane = self.allocation.capacity(npixel, "lossless differences")?;
        }
        for _mcu_y in 0..height {
            for _mcu_x in 0..width {
                if self.restart_interval > 0 {
                    if mcus_left_until_restart == 0 {
                        match huffman.take_marker(reader)? {
                            Some(Marker::RST(n)) => {
                                if n != expected_rst_num {
                                    return Err(Error::Format(format!(
                                        "found RST{} where RST{} was expected",
                                        n, expected_rst_num
                                    )));
                                }

                                huffman.reset();

                                expected_rst_num = (expected_rst_num + 1) % 8;
                                mcus_left_until_restart = self.restart_interval;
                            }
                            Some(marker) => {
                                return Err(Error::Format(format!(
                                    "found marker {:?} inside scan where RST{} was expected",
                                    marker, expected_rst_num
                                )))
                            }
                            None => {
                                return Err(Error::Format(format!(
                                    "no marker found where RST{} was expected",
                                    expected_rst_num
                                )))
                            }
                        }
                    }

                    mcus_left_until_restart -= 1;
                }

                for (i, _component) in components.iter().enumerate() {
                    let dc_table = self.dc_huffman_tables[scan.dc_table_indices[i]]
                        .as_ref()
                        .unwrap();
                    let value = huffman.decode(reader, dc_table)?;
                    let diff = match value {
                        0 => 0,
                        1..=15 => huffman.receive_extend(reader, value)? as i32,
                        16 => 32768,
                        _ => {
                            // Section F.1.2.1.1
                            // Table F.1
                            return Err(Error::Format(
                                "invalid DC difference magnitude category".to_owned(),
                            ));
                        }
                    };
                    differences[i].push(diff);
                }
            }
        }

        if scan.predictor_selection == Predictor::Ra {
            for (i, _component) in components.iter().enumerate() {
                // calculate the top left pixel
                let diff = differences[i][0];
                let prediction = 1 << (frame.precision - scan.point_transform - 1) as i32;
                let result = ((prediction + diff) & 0xFFFF) as u16; // modulo 2^16
                let result = result << scan.point_transform;
                results[i][0] = result;

                // calculate leftmost column, using top pixel as predictor
                let mut previous = result;
                for mcu_y in 1..height {
                    let diff = differences[i][mcu_y * width];
                    let prediction = previous as i32;
                    let result = ((prediction + diff) & 0xFFFF) as u16; // modulo 2^16
                    let result = result << scan.point_transform;
                    results[i][mcu_y * width] = result;
                    previous = result;
                }

                // calculate rows, using left pixel as predictor
                for mcu_y in 0..height {
                    for mcu_x in 1..width {
                        let diff = differences[i][mcu_y * width + mcu_x];
                        let prediction = results[i][mcu_y * width + mcu_x - 1] as i32;
                        let result = ((prediction + diff) & 0xFFFF) as u16; // modulo 2^16
                        let result = result << scan.point_transform;
                        results[i][mcu_y * width + mcu_x] = result;
                    }
                }
            }
        } else {
            for mcu_y in 0..height {
                for mcu_x in 0..width {
                    for (i, _component) in components.iter().enumerate() {
                        let diff = differences[i][mcu_y * width + mcu_x];

                        // The following lines could be further optimized, e.g. moving the checks
                        // and updates of the previous values into the prediction function or
                        // iterating such that diagonals with mcu_x + mcu_y = const are computed at
                        // the same time to exploit independent predictions in this case
                        if mcu_x > 0 {
                            ra[i] = results[i][mcu_y * frame.image_size.width as usize + mcu_x - 1];
                        }
                        if mcu_y > 0 {
                            rb[i] =
                                results[i][(mcu_y - 1) * frame.image_size.width as usize + mcu_x];
                            if mcu_x > 0 {
                                rc[i] = results[i]
                                    [(mcu_y - 1) * frame.image_size.width as usize + (mcu_x - 1)];
                            }
                        }
                        let prediction = predict(
                            ra[i] as i32,
                            rb[i] as i32,
                            rc[i] as i32,
                            scan.predictor_selection,
                            scan.point_transform,
                            frame.precision,
                            mcu_x,
                            mcu_y,
                            self.restart_interval > 0
                                && mcus_left_until_restart == self.restart_interval - 1,
                        );
                        let result = ((prediction + diff) & 0xFFFF) as u16; // modulo 2^16
                        results[i][mcu_y * width + mcu_x] = result << scan.point_transform;
                    }
                }
            }
        }

        let mut marker = huffman.take_marker(&mut self.reader)?;
        while let Some(Marker::RST(_)) = marker {
            marker = self.read_marker().ok();
        }
        Ok((marker, results))
    }
}

/// H.1.2.1
#[allow(clippy::too_many_arguments)]
fn predict(
    ra: i32,
    rb: i32,
    rc: i32,
    predictor: Predictor,
    point_transform: u8,
    input_precision: u8,
    ix: usize,
    iy: usize,
    restart: bool,
) -> i32 {
    if (ix == 0 && iy == 0) || restart {
        // start of first line or restart
        if input_precision > 1 + point_transform {
            1 << (input_precision - point_transform - 1)
        } else {
            0
        }
    } else if iy == 0 {
        // rest of first line
        ra
    } else if ix == 0 {
        // start of other line
        rb
    } else {
        // use predictor Table H.1
        match predictor {
            Predictor::NoPrediction => 0,
            Predictor::Ra => ra,
            Predictor::Rb => rb,
            Predictor::Rc => rc,
            Predictor::RaRbRc1 => ra + rb - rc,
            Predictor::RaRbRc2 => ra + ((rb - rc) >> 1),
            Predictor::RaRbRc3 => rb + ((ra - rc) >> 1),
            Predictor::RaRb => (ra + rb) / 2,
        }
    }
}

pub fn compute_image_lossless(
    frame: &FrameInfo,
    mut data: Vec<Vec<u16>>,
    alloc: Allocator<'_>,
) -> Result<Vec<u8>> {
    if data.is_empty() || data.iter().any(Vec::is_empty) {
        return Err(Error::Format("not all components have data".to_owned()));
    }
    let output_size = frame.output_size;
    let components = &frame.components;
    let ncomp = components.len();

    if ncomp == 1 {
        let decoded = convert_to_u8(frame, data.remove(0), alloc)?;
        Ok(decoded)
    } else {
        let mut decoded: Vec<u16> = alloc.filled(
            0u16,
            alloc.product(&[
                ncomp,
                output_size.width as usize,
                output_size.height as usize,
            ])?,
            "lossless interleaved",
        )?;
        for (x, chunk) in decoded.chunks_mut(ncomp).enumerate() {
            for (i, (component_data, _)) in data.iter().zip(components.iter()).enumerate() {
                chunk[i] = component_data[x];
            }
        }
        let decoded = convert_to_u8(frame, decoded, alloc)?;
        Ok(decoded)
    }
}

fn convert_to_u8(frame: &FrameInfo, data: Vec<u16>, alloc: Allocator<'_>) -> Result<Vec<u8>> {
    if frame.precision == 8 {
        alloc.collect(data.iter().map(|x| *x as u8), "lossless output")
    } else {
        // we output native endian, which is the standard for image-rs
        let ne_bytes = alloc.collect(
            data.iter().map(|x| x.to_ne_bytes()),
            "lossless endian scratch",
        )?;
        let mut out = alloc.capacity(alloc.product(&[ne_bytes.len(), 2])?, "lossless output")?;
        for bytes in ne_bytes {
            out.extend_from_slice(&bytes);
        }
        Ok(out)
    }
}

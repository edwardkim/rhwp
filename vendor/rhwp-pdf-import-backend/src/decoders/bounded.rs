use super::{normalize_filter_name, DecodeParams};
use crate::error::{Error, Result};
use crate::import_limits::ImportBudget;
use brotli_decompressor as brotli_codec;
use std::borrow::Cow;

pub(super) fn decode(
    data: &[u8],
    filters: &[String],
    params: Option<&DecodeParams>,
    budget: &ImportBudget,
) -> Result<Vec<u8>> {
    budget.check()?;
    if filters.len() > budget.limits.filters {
        return Err(budget.reject("filter chain depth"));
    }
    let mut current = Cow::Borrowed(data);
    if filters.is_empty() {
        let mut output = budget.output();
        output.append(data)?;
        current = Cow::Owned(output.finish());
    }
    for filter in filters {
        let decoded = match normalize_filter_name(filter)? {
            "FlateDecode" => super::flate_limited::decode(&current, budget)?,
            "BrotliDecode" => decode_brotli(&current, budget)?,
            "LZWDecode" => {
                let mut output = budget.output();
                let result = weezl::decode::Decoder::new(weezl::BitOrder::Msb, 8)
                    .into_stream(&mut output)
                    .decode_all(current.as_ref());
                budget.check()?;
                result
                    .status
                    .map_err(|_| Error::Decode("invalid bounded LZW stream".into()))?;
                output.finish()
            },
            "RunLengthDecode" => run_length(&current, budget)?,
            "ASCIIHexDecode" => ascii_hex(&current, budget)?,
            "ASCII85Decode" => ascii85(&current, budget)?,
            // Pixel codecs are decoded later by the image extractor.
            "DCTDecode" | "CCITTFaxDecode" | "JBIG2Decode" | "JPXDecode" => {
                let mut output = budget.output();
                output.append(&current)?;
                output.finish()
            },
            _ => unreachable!(),
        };
        current = Cow::Owned(decoded);
    }
    if let Some(params) = params.filter(|p| p.predictor != 1) {
        if current.is_empty() {
            return Ok(current.into_owned());
        }
        let row = params
            .columns
            .checked_mul(params.colors)
            .and_then(|n| n.checked_mul(params.bits_per_component))
            .and_then(|n| n.checked_add(7))
            .map(|n| n / 8)
            .filter(|&n| n > 0 && n <= current.len() && n <= budget.limits.stream_bytes)
            .ok_or_else(|| budget.reject("predictor row size"))?;
        if ![1, 2, 4, 8, 16].contains(&params.bits_per_component)
            || params.colors == 0
            || params.colors > row
        {
            return Err(budget.reject("predictor layout"));
        }
        let stride = row
            .checked_add(usize::from(params.predictor >= 10))
            .ok_or_else(|| budget.reject("predictor stride overflow"))?;
        if !current.len().is_multiple_of(stride) {
            return Err(Error::Decode("invalid predictor row length".into()));
        }
        budget.charge_decoded(current.len())?;
        budget.charge_decoded(
            row.checked_mul(2)
                .ok_or_else(|| budget.reject("predictor scratch"))?,
        )?;
        current = Cow::Owned(super::decode_predictor(&current, params)?);
    }
    budget.check()?;
    Ok(current.into_owned())
}

#[derive(Default)]
struct CodecMemory<T>(Vec<T>);

impl<T> brotli_codec::SliceWrapper<T> for CodecMemory<T> {
    fn slice(&self) -> &[T] {
        &self.0
    }
}

impl<T> brotli_codec::SliceWrapperMut<T> for CodecMemory<T> {
    fn slice_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

struct CodecAllocator<'a>(&'a ImportBudget);

impl<T: Clone + Default> brotli_codec::Allocator<T> for CodecAllocator<'_> {
    type AllocatedMemory = CodecMemory<T>;

    fn alloc_cell(&mut self, count: usize) -> Self::AllocatedMemory {
        let Some(bytes) = count.checked_mul(std::mem::size_of::<T>()) else {
            self.0.reject("codec allocation bytes");
            return CodecMemory::default();
        };
        if self.0.charge_codec_allocation(bytes).is_err() {
            return CodecMemory::default();
        }
        let mut memory = Vec::new();
        if memory.try_reserve_exact(count).is_err() {
            self.0.reject("codec allocation");
            return CodecMemory::default();
        }
        memory.resize(count, T::default());
        CodecMemory(memory)
    }

    fn free_cell(&mut self, memory: Self::AllocatedMemory) {
        drop(memory);
    }
}

fn decode_brotli(data: &[u8], budget: &ImportBudget) -> Result<Vec<u8>> {
    // The stream API permits checking constructor allocation failure before the
    // decoder can index its initial Huffman table. All subsequent production
    // allocations in decompressor 6.0.1 return an allocation error on empty memory.
    let mut state = brotli_codec::BrotliState::new_strict(
        CodecAllocator(budget),
        CodecAllocator(budget),
        CodecAllocator(budget),
    );
    budget.check()?;
    let mut input_offset = 0;
    let mut available_in = data.len();
    let mut total_out = 0;
    let mut output = budget.output();
    loop {
        let before = input_offset;
        let mut chunk = [0u8; 4096];
        let mut output_offset = 0;
        let mut available_out = chunk.len();
        let result = brotli_codec::BrotliDecompressStream(
            &mut available_in,
            &mut input_offset,
            data,
            &mut available_out,
            &mut output_offset,
            &mut chunk,
            &mut total_out,
            &mut state,
        );
        budget.check()?;
        output.append(&chunk[..output_offset])?;
        match result {
            brotli_codec::BrotliResult::ResultSuccess => return Ok(output.finish()),
            brotli_codec::BrotliResult::NeedsMoreOutput
                if input_offset != before || output_offset != 0 => {},
            _ => return Err(Error::Decode("invalid or truncated bounded Brotli stream".into())),
        }
    }
}

fn run_length(data: &[u8], budget: &ImportBudget) -> Result<Vec<u8>> {
    let mut output = budget.output();
    let mut position = 0;
    while position < data.len() {
        let length = data[position];
        position += 1;
        match length {
            0..=127 => {
                let end = position + usize::from(length) + 1;
                let bytes = data
                    .get(position..end)
                    .ok_or_else(|| Error::Decode("truncated RunLength stream".into()))?;
                output.append(bytes)?;
                position = end;
            },
            128 => break,
            _ => {
                let byte = *data
                    .get(position)
                    .ok_or_else(|| Error::Decode("truncated RunLength run".into()))?;
                position += 1;
                let scratch = [byte; 128];
                output.append(&scratch[..257 - usize::from(length)])?;
            },
        }
    }
    Ok(output.finish())
}

fn ascii_hex(data: &[u8], budget: &ImportBudget) -> Result<Vec<u8>> {
    let mut output = budget.output();
    let mut high = None;
    for &byte in data {
        if byte == b'>' {
            break;
        }
        if byte.is_ascii_whitespace() {
            continue;
        }
        let digit = (byte as char)
            .to_digit(16)
            .ok_or_else(|| Error::Decode("invalid ASCIIHex digit".into()))?
            as u8;
        if let Some(value) = high.take() {
            output.append(&[value * 16 + digit])?;
        } else {
            high = Some(digit);
        }
    }
    if let Some(value) = high {
        output.append(&[value * 16])?;
    }
    Ok(output.finish())
}

fn ascii85(data: &[u8], budget: &ImportBudget) -> Result<Vec<u8>> {
    let mut output = budget.output();
    let mut acc = 0u32;
    let mut count = 0;
    for &byte in data {
        match byte {
            b'~' => break,
            b'z' if count == 0 => output.append(&[0; 4])?,
            b'!'..=b'u' => {
                acc = acc
                    .checked_mul(85)
                    .and_then(|n| n.checked_add(u32::from(byte - b'!')))
                    .ok_or_else(|| Error::Decode("ASCII85 overflow".into()))?;
                count += 1;
                if count == 5 {
                    output.append(&acc.to_be_bytes())?;
                    count = 0;
                    acc = 0;
                }
            },
            _ if byte.is_ascii_whitespace() => {},
            _ => return Err(Error::Decode("invalid ASCII85 input".into())),
        }
    }
    if count == 1 {
        return Err(Error::Decode("incomplete ASCII85 group".into()));
    }
    if count > 1 {
        for _ in count..5 {
            acc = acc
                .checked_mul(85)
                .and_then(|n| n.checked_add(84))
                .ok_or_else(|| Error::Decode("ASCII85 padding overflow".into()))?;
        }
        output.append(&acc.to_be_bytes()[..count - 1])?;
    }
    Ok(output.finish())
}

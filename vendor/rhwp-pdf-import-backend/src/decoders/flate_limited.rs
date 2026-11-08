use crate::error::{Error, Result};
use crate::import_limits::ImportBudget;
use crate::zlib_limited::{Memory, Stream};
use zlib_rs::ReturnCode;

pub(super) fn decode(data: &[u8], budget: &ImportBudget) -> Result<Vec<u8>> {
    match decode_format(data, budget, false) {
        Ok(bytes) => Ok(bytes),
        Err(Error::Decode(_)) => {
            // Retain the existing raw Deflate fallback, but never retry a
            // resource failure. Both attempts consume the same request budget.
            budget.check()?;
            decode_format(data, budget, true)
        },
        Err(error) => Err(error),
    }
}

fn decode_format(data: &[u8], budget: &ImportBudget, raw: bool) -> Result<Vec<u8>> {
    let mut memory = Memory::new(budget);
    let mut decoder = Stream::inflater(&mut memory, raw)?;
    let mut output = budget.output();
    let mut position = 0;
    let mut chunk = [0u8; 4096];
    loop {
        let (status, consumed, written) = decoder.step(&data[position..], &mut chunk, false)?;
        position += consumed;
        output.append(&chunk[..written])?;
        match status {
            ReturnCode::StreamEnd => return Ok(output.finish()),
            ReturnCode::Ok if consumed != 0 || written != 0 => {},
            _ => return Err(Error::Decode("invalid or truncated bounded Flate stream".into())),
        }
    }
}

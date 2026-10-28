//! The image JPEG adapter calls `Read::read_to_end` before applying its limits.
//! Override that allocation boundary instead of relying on reader size hints.
use crate::import_limits::ImportBudget;
use std::io::{self, BufRead, Cursor, Read, Seek, SeekFrom};

pub(super) struct Input<'a> {
    cursor: Cursor<&'a [u8]>,
    budget: &'a ImportBudget,
}

impl<'a> Input<'a> {
    pub(super) fn new(bytes: &'a [u8], budget: &'a ImportBudget) -> Self {
        Self {
            cursor: Cursor::new(bytes),
            budget,
        }
    }
}

impl Read for Input<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(output)
    }

    fn read_to_end(&mut self, output: &mut Vec<u8>) -> io::Result<usize> {
        let start = self
            .cursor
            .position()
            .min(self.cursor.get_ref().len() as u64) as usize;
        let remaining = &self.cursor.get_ref()[start..];
        self.budget
            .charge_codec_allocation(remaining.len())
            .map_err(|error| io::Error::other(error.to_string()))?;
        output.try_reserve_exact(remaining.len()).map_err(|_| {
            io::Error::other(self.budget.reject("JPEG input allocation").to_string())
        })?;
        output.extend_from_slice(remaining);
        let count = remaining.len();
        self.cursor.set_position(self.cursor.get_ref().len() as u64);
        Ok(count)
    }
}

impl BufRead for Input<'_> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.cursor.fill_buf()
    }
    fn consume(&mut self, count: usize) {
        self.cursor.consume(count);
    }
}

impl Seek for Input<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.cursor.seek(position)
    }
}

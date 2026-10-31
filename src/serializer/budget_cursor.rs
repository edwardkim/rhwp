//! Explicit, request-local backing policy for synchronous container writers.
use crate::storage::Storage;
use std::io::{self, Cursor, Seek, SeekFrom, Write};

pub(crate) struct BudgetCursor<'a, S> {
    cursor: Cursor<Vec<u8>>,
    storage: &'a S,
    maximum: usize,
    failed: bool,
}
impl<'a, S: Storage> BudgetCursor<'a, S>
where
    S::Error: std::fmt::Display,
{
    pub(crate) fn new(storage: &'a S, maximum: usize) -> Self {
        Self {
            cursor: Cursor::new(Vec::new()),
            storage,
            maximum,
            failed: false,
        }
    }
    pub(crate) fn into_inner(self) -> Vec<u8> {
        self.cursor.into_inner()
    }
    fn reject(&mut self, message: impl std::fmt::Display) -> io::Error {
        self.failed = true;
        io::Error::other(message.to_string())
    }
    fn check(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(io::Error::other(
                "SERIALIZE_RESOURCE_LIMIT: writer already failed",
            ));
        }
        if let Err(error) = self.storage.bytes(0) {
            return Err(self.reject(error));
        }
        Ok(())
    }
}
impl<S: Storage> Write for BudgetCursor<'_, S>
where
    S::Error: std::fmt::Display,
{
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        self.check()?;
        if input.is_empty() {
            return Ok(0);
        }
        let end = usize::try_from(self.cursor.position())
            .ok()
            .and_then(|pos| pos.checked_add(input.len()));
        let Some(end) = end.filter(|end| *end <= self.maximum) else {
            return Err(self.reject("SERIALIZE_RESOURCE_LIMIT: output byte limit"));
        };
        let additional = end.saturating_sub(self.cursor.get_ref().len());
        if let Err(error) = self.storage.reserve(self.cursor.get_mut(), additional) {
            return Err(self.reject(error));
        }
        self.cursor.write(input)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.check()
    }
}
impl<S: Storage> Seek for BudgetCursor<'_, S>
where
    S::Error: std::fmt::Display,
{
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        self.check()?;
        let position = match from {
            SeekFrom::Start(pos) => i128::from(pos),
            SeekFrom::Current(delta) => i128::from(self.cursor.position()) + i128::from(delta),
            SeekFrom::End(delta) => self.cursor.get_ref().len() as i128 + i128::from(delta),
        };
        if position < 0 {
            return Err(self.reject("SERIALIZE_RESOURCE_LIMIT: invalid seek"));
        }
        if position > self.maximum as i128 {
            return Err(self.reject("SERIALIZE_RESOURCE_LIMIT: output seek limit"));
        }
        self.cursor.set_position(position as u64);
        Ok(position as u64)
    }
}

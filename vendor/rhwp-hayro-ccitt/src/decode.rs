use crate::bit_reader::BitReader;
use crate::state_machine::{
    BLACK_STATES, INVALID, MODE_STATES, State, TERMINAL, VALUE_MASK, WHITE_STATES,
};
use crate::{Color, DecodeError, Result};

/// End-of-facsimile-block marker (T.6 Section 2.4.1.1).
/// Two consecutive EOL codes: 000000000001 000000000001.
pub(crate) const EOFB: u32 = 0x1001;

/// 2D coding modes (T.4 Section 4.2.1.3.2, T.6 Section 2.2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Pass mode (T.4 Section 4.2.1.3.2a, T.6 Section 2.2.3.1).
    Pass,
    /// Horizontal mode (T.4 Section 4.2.1.3.2c, T.6 Section 2.2.3.3).
    Horizontal,
    /// Vertical mode with offset (T.4 Section 4.2.1.3.2b, T.6 Section 2.2.3.2).
    Vertical(i8),
}

impl BitReader<'_> {
    /// Decode a run length using the given state machine (T.4 Section 4.1.1, T.6 Section 2.2.4).
    ///
    /// Run lengths 0-63 use terminating codes.
    /// Run lengths 64+ use one or more make-up codes followed by a terminating code.
    #[inline(always)]
    fn decode_run_inner(&mut self, states: &[State]) -> Result<u32> {
        rhwp_codec_resource::work(1)?;
        let mut total: u32 = 0;
        let mut state: usize = 0;

        loop {
            rhwp_codec_resource::work(1)?;
            let bit = self.read_bit()?;

            let transition = if bit == 0 {
                states[state].on_0
            } else {
                states[state].on_1
            };

            if transition == INVALID {
                return Err(DecodeError::InvalidCode);
            } else if transition & TERMINAL != 0 {
                let len = (transition & VALUE_MASK) as u32;
                total = total.checked_add(len).ok_or(DecodeError::Overflow)?;

                // For decoding black/white runs, less than 64 means we have
                // a terminating code. For mode decoding, all values are less
                // than 64 anyway, so this condition can be used for all methods.
                if len < 64 {
                    return Ok(total);
                }

                state = 0;
            } else {
                state = transition as usize;
            }
        }
    }

    /// Decode a white run length.
    #[inline(always)]
    fn decode_white_run(&mut self) -> Result<u32> {
        rhwp_codec_resource::work(1)?;
        self.decode_run_inner(&WHITE_STATES)
            // See 0506179.pdf. We are lenient and check whether perhaps
            // the opposite color works.
            .or_else(|_| self.decode_run_inner(&BLACK_STATES))
    }

    /// Decode a black run length.
    #[inline(always)]
    fn decode_black_run(&mut self) -> Result<u32> {
        rhwp_codec_resource::work(1)?;
        self.decode_run_inner(&BLACK_STATES)
            // See 0506179.pdf. We are lenient and check whether perhaps
            // the opposite color works.
            .or_else(|_| self.decode_run_inner(&WHITE_STATES))
    }

    /// Decode a run length for the specified color.
    #[inline(always)]
    pub(crate) fn decode_run(&mut self, color: Color) -> Result<u32> {
        rhwp_codec_resource::work(1)?;
        match color {
            Color::White => self.decode_white_run(),
            Color::Black => self.decode_black_run(),
        }
    }

    /// Decode a 2D mode code.
    #[inline(always)]
    pub(crate) fn decode_mode(&mut self) -> Result<Mode> {
        rhwp_codec_resource::work(1)?;
        let mode_value = self.decode_run_inner(&MODE_STATES)?;

        Ok(match mode_value {
            0 => Mode::Pass,
            1 => Mode::Horizontal,
            2 => Mode::Vertical(0),
            3 => Mode::Vertical(1),
            4 => Mode::Vertical(2),
            5 => Mode::Vertical(3),
            6 => Mode::Vertical(-1),
            7 => Mode::Vertical(-2),
            8 => Mode::Vertical(-3),
            _ => return Err(DecodeError::InvalidCode),
        })
    }

    /// Read EOL (End-of-Line) codes if present (T.4 Section 4.1.2).
    ///
    /// EOL is defined as `000000000001` (11 zeros followed by a 1).
    /// Fill bits (T.4 Section 4.1.3) may precede the EOL as a variable-length
    /// string of zeros.
    #[inline(always)]
    pub(crate) fn read_eol_if_available(&mut self) -> usize {
        let mut count = 0;

        // T.4 Section 4.1.2: EOL = 000000000001
        // T.4 Section 4.1.3: Fill = variable length string of 0s before EOL
        loop {
            let mut fill_bits = 0;

            // Let's limit the maximum number of fill bits to prevent
            // exponential explosion in malformed files.
            const MAX_FILL_BITS: usize = 24;

            while fill_bits < MAX_FILL_BITS {
                match self.peak_bits(fill_bits + 1) {
                    Ok(0) => fill_bits += 1,
                    _ => break,
                }
            }

            if fill_bits >= 11 && self.peak_bits(fill_bits + 1) == Ok(1) {
                // Found EOL with fill bits, consume all of it.
                self.read_bits(fill_bits + 1).unwrap();
                count += 1;
                continue;
            }

            return count;
        }
    }
}

//! Field flags for interactive PDF form fields.
//!
//! Implements field flags per ISO 32000-1:2008 Section 12.7.3 (Field Flags).
//!
//! Each field type has specific flags that control its behavior:
//! - Common flags apply to all field types
//! - Text field flags (Tx) control multiline, password, etc.
//! - Button field flags (Btn) distinguish checkboxes, radio buttons, and push buttons
//! - Choice field flags (Ch) control combo boxes and list boxes

use bitflags::bitflags;

bitflags! {
    /// Common field flags applicable to all field types.
    ///
    /// Per PDF spec Table 221 (Field flags common to all field types).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FieldFlags: u32 {
        /// Bit 1: Field is read-only; user cannot change the value
        const READ_ONLY = 1 << 0;

        /// Bit 2: Field is required; must have a value before submit
        const REQUIRED = 1 << 1;

        /// Bit 3: Field should not be exported by submit-form action
        const NO_EXPORT = 1 << 2;
    }
}

bitflags! {
    /// Text field flags (field type Tx).
    ///
    /// Per PDF spec Table 228 (Field flags specific to text fields).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TextFieldFlags: u32 {
        // --- Common flags (bits 1-3) ---
        /// Bit 1: Field is read-only
        const READ_ONLY = 1 << 0;
        /// Bit 2: Field is required
        const REQUIRED = 1 << 1;
        /// Bit 3: Field should not be exported
        const NO_EXPORT = 1 << 2;

        // --- Text-specific flags ---
        /// Bit 13: Text may include multiple lines
        const MULTILINE = 1 << 12;

        /// Bit 14: Text should be displayed as asterisks (password)
        const PASSWORD = 1 << 13;

        /// Bit 21: File path should be submitted as field value
        const FILE_SELECT = 1 << 20;

        /// Bit 23: Text should not be spell-checked
        const DO_NOT_SPELL_CHECK = 1 << 22;

        /// Bit 24: Text should not scroll beyond visible area
        const DO_NOT_SCROLL = 1 << 23;

        /// Bit 25: Field is divided into equally spaced positions (comb)
        /// MaxLen must be set when using this flag
        const COMB = 1 << 24;

        /// Bit 26: Field contains rich text
        const RICH_TEXT = 1 << 25;
    }
}

bitflags! {
    /// Button field flags (field type Btn).
    ///
    /// Per PDF spec Table 226 (Field flags specific to button fields).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ButtonFieldFlags: u32 {
        // --- Common flags (bits 1-3) ---
        /// Bit 1: Field is read-only
        const READ_ONLY = 1 << 0;
        /// Bit 2: Field is required
        const REQUIRED = 1 << 1;
        /// Bit 3: Field should not be exported
        const NO_EXPORT = 1 << 2;

        // --- Button-specific flags ---
        /// Bit 15: (for checkbox/radio) No toggle to off; at least one in group must be on
        const NO_TOGGLE_TO_OFF = 1 << 14;

        /// Bit 16: This is a radio button (if not set and not PUSHBUTTON, it's a checkbox)
        const RADIO = 1 << 15;

        /// Bit 17: This is a push button (performs action, doesn't retain value)
        const PUSHBUTTON = 1 << 16;

        /// Bit 26: Radio buttons in unison - all with same /V value turn on together
        const RADIOS_IN_UNISON = 1 << 25;
    }
}

bitflags! {
    /// Choice field flags (field type Ch).
    ///
    /// Per PDF spec Table 230 (Field flags specific to choice fields).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ChoiceFieldFlags: u32 {
        // --- Common flags (bits 1-3) ---
        /// Bit 1: Field is read-only
        const READ_ONLY = 1 << 0;
        /// Bit 2: Field is required
        const REQUIRED = 1 << 1;
        /// Bit 3: Field should not be exported
        const NO_EXPORT = 1 << 2;

        // --- Choice-specific flags ---
        /// Bit 18: This is a combo box (dropdown); if not set, it's a list box
        const COMBO = 1 << 17;

        /// Bit 19: (combo only) User may enter custom text
        const EDIT = 1 << 18;

        /// Bit 20: Options should be sorted alphabetically
        const SORT = 1 << 19;

        /// Bit 22: (list only) Allow multiple selections
        const MULTI_SELECT = 1 << 21;

        /// Bit 23: Text should not be spell-checked (for editable combo)
        const DO_NOT_SPELL_CHECK = 1 << 22;

        /// Bit 27: Value is committed when selection changes (not on blur)
        const COMMIT_ON_SEL_CHANGE = 1 << 26;
    }
}

impl Default for FieldFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for TextFieldFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for ButtonFieldFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for ChoiceFieldFlags {
    fn default() -> Self {
        Self::empty()
    }
}

/// Text alignment for form fields.
///
/// Per PDF spec Section 12.7.3.3 (Variable Text).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignment {
    /// Left-aligned (Q=0)
    #[default]
    Left,
    /// Centered (Q=1)
    Center,
    /// Right-aligned (Q=2)
    Right,
}

impl TextAlignment {
    /// Get the PDF Q value for this alignment.
    pub fn q_value(&self) -> i64 {
        match self {
            Self::Left => 0,
            Self::Center => 1,
            Self::Right => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_flags_bits() {
        assert_eq!(FieldFlags::READ_ONLY.bits(), 1);
        assert_eq!(FieldFlags::REQUIRED.bits(), 2);
        assert_eq!(FieldFlags::NO_EXPORT.bits(), 4);
    }

    #[test]
    fn test_text_field_flags_bits() {
        assert_eq!(TextFieldFlags::MULTILINE.bits(), 1 << 12);
        assert_eq!(TextFieldFlags::PASSWORD.bits(), 1 << 13);
        assert_eq!(TextFieldFlags::COMB.bits(), 1 << 24);
    }

    #[test]
    fn test_button_field_flags_bits() {
        assert_eq!(ButtonFieldFlags::RADIO.bits(), 1 << 15);
        assert_eq!(ButtonFieldFlags::PUSHBUTTON.bits(), 1 << 16);
        assert_eq!(ButtonFieldFlags::NO_TOGGLE_TO_OFF.bits(), 1 << 14);
    }

    #[test]
    fn test_choice_field_flags_bits() {
        assert_eq!(ChoiceFieldFlags::COMBO.bits(), 1 << 17);
        assert_eq!(ChoiceFieldFlags::EDIT.bits(), 1 << 18);
        assert_eq!(ChoiceFieldFlags::MULTI_SELECT.bits(), 1 << 21);
    }

    #[test]
    fn test_combined_flags() {
        let flags = TextFieldFlags::REQUIRED | TextFieldFlags::MULTILINE;
        assert!(flags.contains(TextFieldFlags::REQUIRED));
        assert!(flags.contains(TextFieldFlags::MULTILINE));
        assert!(!flags.contains(TextFieldFlags::PASSWORD));
    }

    #[test]
    fn test_text_alignment_q_value() {
        assert_eq!(TextAlignment::Left.q_value(), 0);
        assert_eq!(TextAlignment::Center.q_value(), 1);
        assert_eq!(TextAlignment::Right.q_value(), 2);
    }

    #[test]
    fn test_default_flags() {
        assert_eq!(FieldFlags::default(), FieldFlags::empty());
        assert_eq!(TextFieldFlags::default(), TextFieldFlags::empty());
        assert_eq!(ButtonFieldFlags::default(), ButtonFieldFlags::empty());
        assert_eq!(ChoiceFieldFlags::default(), ChoiceFieldFlags::empty());
    }
}

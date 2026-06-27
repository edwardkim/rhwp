//! Form control caption display helpers.
//!
//! HWPX stores form captions with UI-caption escaping semantics. In observed
//! Hancom output, `&&` in a form caption displays as one literal `&`, while the
//! stored value and XML roundtrip must remain unchanged.

use std::borrow::Cow;

pub(crate) fn display_form_caption(caption: &str) -> Cow<'_, str> {
    if !caption.contains("&&") {
        return Cow::Borrowed(caption);
    }

    let mut out = String::with_capacity(caption.len());
    let mut chars = caption.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' && chars.peek() == Some(&'&') {
            chars.next();
            out.push('&');
        } else {
            out.push(ch);
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::display_form_caption;
    use std::borrow::Cow;

    #[test]
    fn collapses_double_ampersand_for_form_caption_display() {
        assert_eq!(display_form_caption("R&&D"), "R&D");
        assert_eq!(display_form_caption("IP R&&D연계"), "IP R&D연계");
        assert_eq!(
            display_form_caption("R&&D 자율성트랙(일반)"),
            "R&D 자율성트랙(일반)"
        );
        assert_eq!(display_form_caption("&&&&"), "&&");
    }

    #[test]
    fn preserves_single_ampersand() {
        assert_eq!(display_form_caption("R&D"), "R&D");
        assert_eq!(display_form_caption("A&B&C"), "A&B&C");
    }

    #[test]
    fn borrows_when_no_display_escape_exists() {
        assert!(matches!(
            display_form_caption("plain caption"),
            Cow::Borrowed("plain caption")
        ));
    }
}

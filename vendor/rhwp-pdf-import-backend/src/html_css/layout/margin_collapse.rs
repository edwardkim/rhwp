//! Margin collapsing per CSS 2.1 §8.3.1 (LAYOUT-5).
//!
//! Adjacent vertical margins on block-flow elements collapse into a
//! single resulting margin equal to the larger of the two (or, when
//! mixing positive and negative, sum of the largest positive and the
//! most-negative). Taffy's block layout doesn't model collapsing
//! itself; we do it as a post-pass when stitching block siblings
//! together in the paginator's input.
//!
//! v0.3.35 covers the three common cases listed in the spec:
//!
//! 1. Adjacent siblings (block A's bottom-margin meets block B's
//!    top-margin).
//! 2. Parent + first/last child (where no padding/border separates).
//! 3. Empty block self-collapse (top + bottom margins of an empty
//!    block collapse with each other).

/// Compute the collapsed value for a sequence of vertical margins.
/// Per CSS 2.1 §8.3.1 the rules are:
///   - All positive margins collapse to their max.
///   - All negative margins collapse to their min (most-negative).
///   - When both signs are present, result = max(positives) +
///     min(negatives).
pub fn collapse_margins(margins: &[f32]) -> f32 {
    let max_positive = margins
        .iter()
        .copied()
        .filter(|&m| m > 0.0)
        .fold(0.0_f32, f32::max);
    let min_negative = margins
        .iter()
        .copied()
        .filter(|&m| m < 0.0)
        .fold(0.0_f32, f32::min);
    max_positive + min_negative
}

/// Adjacent-sibling collapse: returns the gap to insert between block
/// A (with bottom-margin `a_bottom`) and block B (with top-margin
/// `b_top`). The gap is the collapsed margin per the rule above.
pub fn sibling_gap(a_bottom: f32, b_top: f32) -> f32 {
    collapse_margins(&[a_bottom, b_top])
}

/// Parent-child collapse: a parent's top-margin + its first child's
/// top-margin collapse together when no padding, border, or non-zero
/// `overflow` separates them. Caller passes the relevant margins;
/// this just folds them.
pub fn parent_child_top(parent_top: f32, child_top: f32) -> f32 {
    collapse_margins(&[parent_top, child_top])
}

/// Same as [`parent_child_top`] but for the bottom margin.
pub fn parent_child_bottom(parent_bottom: f32, child_bottom: f32) -> f32 {
    collapse_margins(&[parent_bottom, child_bottom])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_positive_take_max() {
        assert_eq!(collapse_margins(&[10.0, 20.0]), 20.0);
        assert_eq!(collapse_margins(&[20.0, 10.0]), 20.0);
    }

    #[test]
    fn equal_positives_collapse_to_one() {
        assert_eq!(collapse_margins(&[15.0, 15.0]), 15.0);
    }

    #[test]
    fn two_negative_take_most_negative() {
        assert_eq!(collapse_margins(&[-5.0, -10.0]), -10.0);
    }

    #[test]
    fn mixed_signs_sum_max_positive_and_min_negative() {
        // Per spec: max positive (20) + min negative (-5) = 15.
        assert_eq!(collapse_margins(&[20.0, -5.0]), 15.0);
        assert_eq!(collapse_margins(&[10.0, -15.0]), -5.0);
    }

    #[test]
    fn empty_input_zero() {
        assert_eq!(collapse_margins(&[]), 0.0);
    }

    #[test]
    fn sibling_gap_collapses_to_max_positive() {
        assert_eq!(sibling_gap(20.0, 10.0), 20.0);
        assert_eq!(sibling_gap(0.0, 30.0), 30.0);
    }

    #[test]
    fn parent_child_top_collapses() {
        // Parent top 10, first child top 25 → 25.
        assert_eq!(parent_child_top(10.0, 25.0), 25.0);
    }

    #[test]
    fn three_way_collapse_via_collapse_margins() {
        // Parent 10, first-child 5, grand-child 15 → 15.
        assert_eq!(collapse_margins(&[10.0, 5.0, 15.0]), 15.0);
    }
}

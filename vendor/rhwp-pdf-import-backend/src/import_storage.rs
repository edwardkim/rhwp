//! Request-owned temporary payloads in backend postprocessing.
//! Counts requested backing and stable-sort scratch, not allocator overhead/RSS.
use crate::{
    error::{Error, Result},
    import_limits::ImportBudget,
};

#[derive(Clone, Copy)]
pub(crate) struct ImportStorage<'a>(pub(crate) Option<&'a ImportBudget>);
impl ImportStorage<'_> {
    pub(crate) fn reject(self, reason: &'static str) -> Error {
        match self.0 {
            Some(budget) => budget.reject(reason),
            None => Error::ImportResourceLimit(reason),
        }
    }
    pub(crate) fn bytes(self, count: usize) -> Result<()> {
        if let Some(budget) = self.0 {
            budget.charge_decoded(count)?;
        }
        Ok(())
    }
    pub(crate) fn string_capacity(self, count: usize) -> Result<String> {
        self.bytes(count)?;
        let mut value = String::new();
        value
            .try_reserve_exact(count)
            .map_err(|_| self.reject("text string allocation"))?;
        Ok(value)
    }
    pub(crate) fn clone_text_span(
        self,
        span: &crate::layout::TextSpan,
    ) -> Result<crate::layout::TextSpan> {
        let bytes = span
            .text
            .len()
            .checked_add(span.font_name.len())
            .and_then(|n| n.checked_add(std::mem::size_of_val(span.char_widths.as_slice())))
            .and_then(|n| n.checked_add(std::mem::size_of_val(span.char_x_offsets.as_slice())))
            .ok_or_else(|| self.reject("text span copy size"))?;
        self.bytes(bytes)?;
        Ok(span.clone())
    }
    pub(crate) fn lowercase_bytes(self, text: &str) -> Result<usize> {
        self.bytes(0)?;
        let mut count = 0usize;
        // Rust 1.93.1 has one contextual str lowercase mapping: sigma.
        // Its two alternatives both occupy two UTF-8 bytes, so the scalar
        // iterator gives the exact output byte count without allocating.
        for character in text.chars().flat_map(char::to_lowercase) {
            count = count
                .checked_add(character.len_utf8())
                .ok_or_else(|| self.reject("text lowercase size"))?;
        }
        Ok(count)
    }
    pub(crate) fn lowercase(self, text: &str) -> Result<String> {
        let count = self.lowercase_bytes(text)?;
        // alloc::str::convert_while_ascii starts with input.len() capacity;
        // String::push then uses RawVec's doubling / minimum-eight growth.
        // Reserve cumulative requested payload before delegating to the same
        // standard producer, preserving contextual Unicode casing.
        let mut capacity = text.len();
        let mut work = capacity;
        while capacity < count {
            capacity = capacity
                .checked_mul(2)
                .ok_or_else(|| self.reject("text lowercase growth"))?
                .max(8);
            work = work
                .checked_add(capacity)
                .ok_or_else(|| self.reject("text lowercase work"))?;
        }
        self.bytes(work)?;
        Ok(text.to_lowercase())
    }
    pub(crate) fn vec<T>(self, count: usize) -> Result<Vec<T>> {
        let bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(|| self.reject("text scratch size overflow"))?;
        self.bytes(bytes)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| self.reject("text scratch allocation"))?;
        Ok(result)
    }
    pub(crate) fn collect<T, I>(self, input: I) -> Result<Vec<T>>
    where
        I: IntoIterator<Item = T>,
    {
        let input = input.into_iter();
        let mut result = self.vec(input.size_hint().0)?;
        for value in input {
            if result.len() == result.capacity() {
                self.reserve(&mut result, 1)?;
            }
            result.push(value);
        }
        self.bytes(0)?;
        Ok(result)
    }
    pub(crate) fn partition<T: Copy>(
        self,
        input: &[T],
        take: impl Fn(&T) -> bool,
    ) -> Result<(Vec<T>, Vec<T>)> {
        self.bytes(0)?;
        let left_count = input.iter().filter(|value| take(value)).count();
        let mut left = self.vec(left_count)?;
        let mut right = self.vec(input.len() - left_count)?;
        for value in input {
            if take(value) {
                left.push(*value);
            } else {
                right.push(*value);
            }
        }
        Ok((left, right))
    }
    pub(crate) fn grouped_values<T>(self, mut input: Vec<(i32, T)>) -> Result<Vec<Vec<T>>> {
        self.sort_by(&mut input, |a, b| a.0.cmp(&b.0))?;
        let mut groups: Vec<Vec<T>> = Vec::new();
        let mut previous = None;
        for (key, value) in input {
            if previous != Some(key) {
                self.push(&mut groups, Vec::new())?;
                previous = Some(key);
            }
            self.push(groups.last_mut().unwrap(), value)?;
        }
        Ok(groups)
    }
    pub(crate) fn sort_by<T>(
        self,
        values: &mut [T],
        compare: impl FnMut(&T, &T) -> std::cmp::Ordering,
    ) -> Result<()> {
        // Preserve stable ties; a full-length backing covers the sort's scratch.
        self.bytes(std::mem::size_of_val(values))?;
        values.sort_by(compare);
        Ok(())
    }

    pub(crate) fn reserve<T>(self, values: &mut Vec<T>, additional: usize) -> Result<()> {
        self.bytes(0)?;
        let required = values
            .len()
            .checked_add(additional)
            .ok_or_else(|| self.reject("text scratch growth size"))?;
        if required > values.capacity() {
            let capacity = required.max(values.capacity().saturating_mul(2)).max(4);
            let bytes = capacity
                .checked_mul(std::mem::size_of::<T>())
                .ok_or_else(|| self.reject("text scratch growth bytes"))?;
            self.bytes(bytes)?;
            values
                .try_reserve_exact(capacity - values.len())
                .map_err(|_| self.reject("text scratch growth allocation"))?;
        }
        Ok(())
    }
    pub(crate) fn push<T>(self, values: &mut Vec<T>, value: T) -> Result<()> {
        self.reserve(values, 1)?;
        values.push(value);
        Ok(())
    }
    pub(crate) fn reorder<T>(self, values: &mut [T], order: &[usize]) -> Result<()> {
        if values.len() != order.len() {
            return Err(self.reject("text permutation size"));
        }
        let mut destination = self.vec(order.len())?;
        destination.resize(order.len(), usize::MAX);
        for (new, &old) in order.iter().enumerate() {
            let slot = destination
                .get_mut(old)
                .ok_or_else(|| self.reject("text permutation index"))?;
            if *slot != usize::MAX {
                return Err(self.reject("text permutation duplicate"));
            }
            *slot = new;
        }
        for index in 0..destination.len() {
            while destination[index] != index {
                let next = destination[index];
                values.swap(index, next);
                destination.swap(index, next);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn lowercase_reserves_growth_before_the_standard_producer_and_keeps_unicode_context() {
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 9,
            ..Default::default()
        })
        .unwrap();
        assert!(ImportStorage(Some(&tiny)).lowercase("İ").is_err());
        assert_eq!(tiny.snapshot().decoded_bytes, 0);
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        assert_eq!(ImportStorage(Some(&fresh)).lowercase("ΟΣ ΟΣΑ İ").unwrap(), "ος οσα i\u{0307}");
        assert_eq!(fresh.snapshot().decoded_bytes, 42);
        let ascii = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 3,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(ImportStorage(Some(&ascii)).lowercase("ABC").unwrap(), "abc");
        assert_eq!(ascii.snapshot().decoded_bytes, 3);
    }
    use super::*;
    use crate::{geometry::Rect, import_limits::PdfImportLimits, layout::TextSpan};

    #[test]
    fn owned_span_payload_copy_is_reserved_before_clone() {
        let original = TextSpan {
            text: "owned text".repeat(12),
            font_name: "owned font".into(),
            char_widths: vec![2.0, 3.0],
            char_x_offsets: vec![0.0, 2.0],
            ..Default::default()
        };
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(ImportStorage(Some(&tiny))
            .clone_text_span(&original)
            .is_err());
        assert_eq!(tiny.snapshot().decoded_bytes, 0);
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        let copied = ImportStorage(Some(&fresh))
            .clone_text_span(&original)
            .unwrap();
        assert_eq!(copied.text, original.text);
        assert_eq!(copied.font_name, original.font_name);
        assert_eq!(copied.char_widths, original.char_widths);
        assert_eq!(copied.char_x_offsets, original.char_x_offsets);
    }

    #[test]
    fn grouped_values_keep_ascending_keys_and_source_order_with_non_clone_payloads() {
        let input = || {
            vec![
                (2, Owned("first".into())),
                (1, Owned("before".into())),
                (2, Owned("second".into())),
            ]
        };
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(ImportStorage(Some(&tiny)).grouped_values(input()).is_err());
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        let groups = ImportStorage(Some(&fresh)).grouped_values(input()).unwrap();
        assert_eq!(
            groups
                .iter()
                .flat_map(|g| g.iter().map(|v| v.0.as_str()))
                .collect::<Vec<_>>(),
            ["before", "first", "second"]
        );
        assert_eq!(groups.iter().map(Vec::len).collect::<Vec<_>>(), [1, 2]);
    }
    struct Owned(String);
    fn values() -> [Owned; 3] {
        [
            Owned("first".into()),
            Owned("second".into()),
            Owned("third".into()),
        ]
    }
    #[test]
    fn permutation_moves_owned_values_and_checks_scratch_before_mutation() {
        let mut values = values();
        let low = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(ImportStorage(Some(&low))
            .reorder(&mut values, &[2, 0, 1])
            .is_err());
        assert_eq!(values.each_ref().map(|value| value.0.as_str()), ["first", "second", "third"]);
        assert_eq!(low.snapshot().decoded_bytes, 0);
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        ImportStorage(Some(&fresh))
            .reorder(&mut values, &[2, 0, 1])
            .unwrap();
        assert_eq!(values.each_ref().map(|value| value.0.as_str()), ["third", "first", "second"]);
    }
    #[test]
    fn invalid_permutations_fail_before_swapping_owned_content() {
        for order in [&[2, 2, 0][..], &[3, 0, 1], &[0, 1]] {
            let mut values = values();
            let budget = ImportBudget::new(PdfImportLimits::default()).unwrap();
            assert!(ImportStorage(Some(&budget))
                .reorder(&mut values, order)
                .is_err());
            assert!(budget.check().is_err());
            assert_eq!(
                values.each_ref().map(|value| value.0.as_str()),
                ["first", "second", "third"]
            );
        }
    }
    #[test]
    fn row_grouping_stops_at_its_scratch_budget_and_recovers() {
        let spans = vec![
            TextSpan {
                text: "A".into(),
                bbox: Rect::new(0., 20., 10., 8.),
                font_size: 8.,
                ..Default::default()
            },
            TextSpan {
                text: "B".into(),
                bbox: Rect::new(20., 20., 10., 8.),
                font_size: 8.,
                ..Default::default()
            },
            TextSpan {
                text: "C".into(),
                bbox: Rect::new(0., 50., 10., 8.),
                font_size: 8.,
                ..Default::default()
            },
        ];
        for limit in [1, 100] {
            let budget = ImportBudget::new(PdfImportLimits {
                decoded_bytes: limit,
                ..Default::default()
            })
            .unwrap();
            assert!(crate::utils::try_snap_baselines_to_rows(
                &spans,
                &[0, 1, 2],
                ImportStorage(Some(&budget))
            )
            .is_err());
            assert!(budget.check().is_err());
            assert!(crate::utils::try_snap_baselines_to_rows(
                &spans,
                &[],
                ImportStorage(Some(&budget))
            )
            .is_err());
        }
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        let rows = crate::utils::try_snap_baselines_to_rows(
            &spans,
            &[0, 1, 2],
            ImportStorage(Some(&fresh)),
        )
        .unwrap();
        assert_eq!(rows[0], rows[1]);
        assert_ne!(rows[1], rows[2]);
        assert!(rows
            .iter()
            .all(|row| spans.iter().any(|span| span.bbox.y == *row)));
        assert_eq!(
            spans
                .iter()
                .map(|span| span.text.as_str())
                .collect::<Vec<_>>(),
            ["A", "B", "C"]
        );
    }
}

//! Shared angle grouping and upright sort, moving source spans by permutation.
use crate::{error::Result, import_storage::ImportStorage, layout::TextSpan};

pub(crate) fn reorder(
    spans: &mut [TextSpan],
    separate_horizontal: bool,
    allocation: ImportStorage<'_>,
) -> Result<()> {
    allocation.bytes(0)?;
    let mut groups: Vec<(f32, Vec<usize>)> = Vec::new();
    let mut order = allocation.vec(spans.len())?;
    for (index, span) in spans.iter().enumerate() {
        let key = span.rotation_degrees;
        if separate_horizontal && key == 0.0 {
            order.push(index);
            continue;
        }
        if let Some(group) = groups
            .iter_mut()
            .find(|(angle, _)| (*angle - key).abs() < 0.5)
        {
            allocation.push(&mut group.1, index)?;
        } else {
            let mut members = allocation.vec(1)?;
            members.push(index);
            allocation.push(&mut groups, (key, members))?;
        }
    }
    for (degrees, mut group) in groups {
        let (sin, cos) = (-degrees).to_radians().sin_cos();
        allocation.sort_by(&mut group, |&a, &b| {
            let a = &spans[a];
            let b = &spans[b];
            let ax = a.bbox.x * cos - a.bbox.y * sin;
            let ay = a.bbox.x * sin + a.bbox.y * cos;
            let bx = b.bbox.x * cos - b.bbox.y * sin;
            let by = b.bbox.x * sin + b.bbox.y * cos;
            crate::utils::row_aware_span_cmp(ay, ax, by, bx)
        })?;
        order.extend(group);
    }
    allocation.reorder(spans, &order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        geometry::Rect,
        import_limits::{ImportBudget, PdfImportLimits},
    };
    #[test]
    fn rotated_order_is_atomic_on_budget_failure_and_keeps_horizontal_then_upright_runs() {
        let mut spans: Vec<_> = [
            ("later", 20.0, 90.0),
            ("earlier", 10.0, 90.0),
            ("horizontal", 100.0, 0.0),
        ]
        .into_iter()
        .map(|(text, x, rotation_degrees)| TextSpan {
            text: text.into(),
            bbox: Rect::new(x, 10.0, 4.0, 4.0),
            rotation_degrees,
            ..Default::default()
        })
        .collect();
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(reorder(&mut spans, true, ImportStorage(Some(&tiny))).is_err());
        assert_eq!(
            spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(),
            ["later", "earlier", "horizontal"]
        );
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        reorder(&mut spans, true, ImportStorage(Some(&fresh))).unwrap();
        assert_eq!(
            spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(),
            ["horizontal", "earlier", "later"]
        );
    }
}

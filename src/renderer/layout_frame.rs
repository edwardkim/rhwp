//! Physical-row geometry for paragraph layout.

use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrameExclusion {
    pub(crate) horizontal: Range<i32>,
    pub(crate) vertical: Range<i32>,
}

/// Mutable geometry of one layout flow.
///
/// A carve is tentative. It may move the candidate row to an exact geometry
/// event, but it does not commit text or advance past a completed row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayoutFrame {
    pub(crate) horizontal: Range<i32>,
    pub(crate) top: i32,
    pub(crate) exclusions: Vec<FrameExclusion>,
    pub(crate) current_intervals: Vec<Range<i32>>,
    pub(crate) next_geometry_event: Option<i32>,
    pub(crate) minimum_width: i32,
}

impl LayoutFrame {
    /// Carve the current physical row into ordered horizontal intervals.
    pub(crate) fn carve(&mut self, band_height: i32) -> &[Range<i32>] {
        let old_max = self.next_geometry_event.map(|_| {
            self.current_intervals
                .iter()
                .map(|interval| interval.end - interval.start)
                .max()
                .unwrap_or(0)
        });
        let mut candidate_top = self.top;

        loop {
            let bottom = candidate_top.saturating_add(band_height.max(0));
            let base = self.horizontal.clone();
            let mut intervals = vec![base.clone()];
            let mut next_geometry_event = None;
            let minimum_width = self.minimum_width.max(0);
            let mut adequate = base.end - base.start >= minimum_width;

            for exclusion in &self.exclusions {
                if exclusion.vertical.start >= bottom || candidate_top >= exclusion.vertical.end {
                    continue;
                }

                let before = intervals.clone();
                let left = exclusion.horizontal.start;
                let right = exclusion.horizontal.end;
                let mut carved = Vec::with_capacity(intervals.len() + 1);

                for interval in &intervals {
                    if interval.end <= left || right <= interval.start {
                        carved.push(interval.clone());
                    } else if interval.start < left && right < interval.end {
                        carved.push(interval.start..left);
                        carved.push(right..interval.end);
                    } else if interval.start < left {
                        carved.push(interval.start..left);
                    } else if right < interval.end {
                        carved.push(right..interval.end);
                    } else {
                        carved.push(interval.start..interval.start);
                    }
                }

                let changed = before != carved;
                intervals = carved;
                if changed {
                    next_geometry_event = Some(
                        next_geometry_event.map_or(exclusion.vertical.end, |event: i32| {
                            event.min(exclusion.vertical.end)
                        }),
                    );
                }

                adequate = intervals
                    .iter()
                    .any(|interval| interval.end - interval.start >= minimum_width);
                if !adequate {
                    break;
                }
            }

            let mut index = 0;
            while intervals.len() > 1 && index < intervals.len() {
                if intervals[index].end - intervals[index].start < minimum_width {
                    intervals.remove(index);
                } else {
                    index += 1;
                }
            }

            let retry = if !adequate {
                next_geometry_event.filter(|event| *event > candidate_top)
            } else {
                let new_max = intervals
                    .iter()
                    .map(|interval| interval.end - interval.start)
                    .max()
                    .unwrap_or(0);
                old_max
                    .filter(|previous| *previous > new_max)
                    .and(next_geometry_event)
                    .filter(|event| *event > candidate_top)
            };

            if let Some(event) = retry {
                candidate_top = event;
                continue;
            }

            self.top = candidate_top;
            self.current_intervals = intervals;
            self.next_geometry_event = next_geometry_event;
            return &self.current_intervals;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taller_candidate_recarves_before_the_row_is_committed() {
        let mut frame = LayoutFrame {
            horizontal: 0..100,
            top: 0,
            exclusions: vec![FrameExclusion {
                horizontal: 0..60,
                vertical: 1_000..3_000,
            }],
            current_intervals: Vec::new(),
            next_geometry_event: None,
            minimum_width: 1,
        };

        let carved = frame.carve(900);
        assert_eq!(carved.len(), 1);
        assert_eq!(carved[0], 0..100);
        assert_eq!(frame.top, 0);

        let carved = frame.carve(2_000);
        assert_eq!(carved.len(), 1);
        assert_eq!(carved[0], 60..100);
        assert_eq!(frame.next_geometry_event, Some(3_000));

        frame.top = 3_000;
        let carved = frame.carve(2_000);
        assert_eq!(carved.len(), 1);
        assert_eq!(carved[0], 0..100);
        assert_eq!(frame.top, 3_000);
    }
}

//! Physical-row geometry for paragraph layout.

use std::ops::Range;

use crate::model::paragraph::LineSeg;

const SEGMENT_BOUNDARY_TAGS: u32 = LineSeg::TAG_FIRST_SEGMENT | LineSeg::TAG_LAST_SEGMENT;

/// The vertical values shared by every horizontal segment in one physical row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FrameRowMetrics {
    pub(crate) vertical_pos: i32,
    pub(crate) line_height: i32,
    pub(crate) text_height: i32,
    pub(crate) baseline_distance: i32,
    pub(crate) line_spacing: i32,
}

impl FrameRowMetrics {
    fn from_line_seg(seg: &LineSeg) -> Self {
        Self {
            vertical_pos: seg.vertical_pos,
            line_height: seg.line_height,
            text_height: seg.text_height,
            baseline_distance: seg.baseline_distance,
            line_spacing: seg.line_spacing,
        }
    }
}

/// One horizontal part of a physical row before it is flattened into a
/// `LineSeg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowSegment {
    pub(crate) text_range: Range<u32>,
    pub(crate) horizontal: Range<i32>,
    /// The source tag retains provenance and line properties. Projection owns
    /// the FIRST/LAST boundary bits because they describe this row's group.
    pub(crate) source_tag: u32,
}

impl RowSegment {
    pub(crate) fn new(text_range: Range<u32>, horizontal: Range<i32>, source_tag: u32) -> Self {
        Self {
            text_range,
            horizontal,
            source_tag: source_tag & !SEGMENT_BOUNDARY_TAGS,
        }
    }

    fn from_single_line_seg(seg: &LineSeg) -> Option<Self> {
        (seg.is_first_segment() && seg.is_last_segment() && seg.segment_width > 0).then(|| {
            Self::new(
                seg.text_start..seg.text_start,
                seg.column_start..seg.column_start.saturating_add(seg.segment_width),
                seg.tag,
            )
        })
    }
}

/// A completed physical row. Its vertical values are recorded once, while its
/// horizontal segments retain their left-to-right order until projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhysicalRow {
    metrics: FrameRowMetrics,
    segments: Vec<RowSegment>,
}

impl PhysicalRow {
    fn from_single_line_seg(seg: &LineSeg) -> Option<Self> {
        Some(Self {
            metrics: FrameRowMetrics::from_line_seg(seg),
            segments: vec![RowSegment::from_single_line_seg(seg)?],
        })
    }
}

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
    rows: Vec<PhysicalRow>,
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

    /// Retain stored rows only when each stored `LineSeg` is a complete,
    /// single-segment physical row. A split FIRST/LAST group has to be
    /// reflowed as one row instead of being flattened into this scalar entry.
    pub(crate) fn retain_preserved_single_segment_rows(&mut self, line_segs: &[LineSeg]) -> bool {
        let Some(rows) = line_segs
            .iter()
            .map(PhysicalRow::from_single_line_seg)
            .collect::<Option<Vec<_>>>()
        else {
            return false;
        };

        if let Some(last) = rows.last() {
            self.top = last
                .metrics
                .vertical_pos
                .saturating_add(last.metrics.line_height)
                .saturating_add(last.metrics.line_spacing);
            self.current_intervals.clear();
            self.next_geometry_event = None;
        }
        self.rows.extend(rows);
        true
    }

    /// Commit the row carved at the current frame position.
    ///
    /// The caller supplies exactly one text result for every interval returned
    /// by `carve`. The frame gives all of them one vertical position, retains
    /// them as one physical row, then advances exactly once.
    pub(crate) fn commit_carved_row(
        &mut self,
        mut metrics: FrameRowMetrics,
        segments: Vec<RowSegment>,
    ) -> Option<usize> {
        if self.current_intervals.is_empty()
            || segments.len() != self.current_intervals.len()
            || segments
                .iter()
                .zip(&self.current_intervals)
                .any(|(segment, interval)| segment.horizontal != *interval)
        {
            return None;
        }

        metrics.vertical_pos = self.top;
        let row_index = self.rows.len();
        self.rows.push(PhysicalRow { metrics, segments });
        self.top = self
            .top
            .saturating_add(metrics.line_height)
            .saturating_add(metrics.line_spacing);
        // A carved interval belongs only to the row just committed. Requiring
        // a new carve prevents a caller from advancing this frame twice with
        // stale geometry.
        self.current_intervals.clear();
        Some(row_index)
    }

    pub(crate) fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Flatten retained physical rows at the document boundary.
    pub(crate) fn project_line_segs(&self) -> Vec<LineSeg> {
        let mut projected = Vec::new();

        for row in &self.rows {
            let last_segment = row.segments.len().saturating_sub(1);
            for (segment_index, segment) in row.segments.iter().enumerate() {
                let mut tag = segment.source_tag & !SEGMENT_BOUNDARY_TAGS;
                if segment_index == 0 {
                    tag |= LineSeg::TAG_FIRST_SEGMENT;
                }
                if segment_index == last_segment {
                    tag |= LineSeg::TAG_LAST_SEGMENT;
                }
                projected.push(LineSeg {
                    text_start: segment.text_range.start,
                    vertical_pos: row.metrics.vertical_pos,
                    line_height: row.metrics.line_height,
                    text_height: row.metrics.text_height,
                    baseline_distance: row.metrics.baseline_distance,
                    line_spacing: row.metrics.line_spacing,
                    column_start: segment.horizontal.start,
                    segment_width: segment
                        .horizontal
                        .end
                        .saturating_sub(segment.horizontal.start),
                    tag,
                });
            }
        }

        projected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(line_height: i32, line_spacing: i32) -> FrameRowMetrics {
        FrameRowMetrics {
            vertical_pos: -1,
            line_height,
            text_height: line_height - 10,
            baseline_distance: line_height - 20,
            line_spacing,
        }
    }

    fn frame(horizontal: Range<i32>, top: i32, exclusions: Vec<FrameExclusion>) -> LayoutFrame {
        LayoutFrame {
            horizontal,
            top,
            exclusions,
            current_intervals: Vec::new(),
            next_geometry_event: None,
            minimum_width: 1,
            rows: Vec::new(),
        }
    }

    #[test]
    fn taller_candidate_recarves_before_the_row_is_committed() {
        let mut frame = frame(
            0..100,
            0,
            vec![FrameExclusion {
                horizontal: 0..60,
                vertical: 1_000..3_000,
            }],
        );

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

    #[test]
    fn committed_row_projects_one_complete_lineseg_group_and_advances_once() {
        let mut frame = frame(10..110, 500, Vec::new());
        let carved = frame.carve(900);
        assert_eq!(carved.len(), 1);
        assert_eq!(carved[0], 10..110);

        assert_eq!(
            frame.commit_carved_row(
                metrics(900, 100),
                vec![RowSegment::new(
                    7..31,
                    10..110,
                    LineSeg::TAG_IMPLEMENTATION_PROPERTY | LineSeg::TAG_FIRST_SEGMENT,
                )],
            ),
            Some(0)
        );
        assert_eq!(frame.top, 1_500);
        assert_eq!(frame.row_count(), 1);
        assert_eq!(frame.commit_carved_row(metrics(900, 100), Vec::new()), None);
        assert_eq!(frame.top, 1_500);

        let projected = frame.project_line_segs();
        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].text_start, 7);
        assert_eq!(projected[0].vertical_pos, 500);
        assert_eq!(projected[0].column_start, 10);
        assert_eq!(projected[0].segment_width, 100);
        assert!(projected[0].is_first_segment());
        assert!(projected[0].is_last_segment());
        assert_ne!(projected[0].tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY, 0);
    }

    #[test]
    fn retains_only_complete_single_segment_rows() {
        let stored = [
            LineSeg {
                text_start: 0,
                vertical_pos: 100,
                line_height: 300,
                text_height: 280,
                baseline_distance: 250,
                line_spacing: 20,
                column_start: 40,
                segment_width: 60,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            },
            LineSeg {
                text_start: 12,
                vertical_pos: 420,
                line_height: 300,
                text_height: 280,
                baseline_distance: 250,
                line_spacing: 20,
                column_start: 40,
                segment_width: 60,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE | LineSeg::TAG_IMPLEMENTATION_PROPERTY,
            },
        ];
        let mut frame = frame(0..100, 0, Vec::new());

        assert!(frame.retain_preserved_single_segment_rows(&stored));
        assert_eq!(frame.row_count(), 2);
        assert_eq!(frame.top, 740);
        let projected = frame.project_line_segs();
        assert_eq!(projected.len(), 2);
        assert_eq!(projected[1].text_start, 12);
        assert_eq!(projected[1].vertical_pos, 420);
        assert!(projected
            .iter()
            .all(|segment| { segment.is_first_segment() && segment.is_last_segment() }));

        let split_row = [
            LineSeg {
                tag: LineSeg::TAG_FIRST_SEGMENT,
                ..Default::default()
            },
            LineSeg {
                tag: LineSeg::TAG_LAST_SEGMENT,
                ..Default::default()
            },
        ];
        assert!(!frame.retain_preserved_single_segment_rows(&split_row));
        assert_eq!(frame.row_count(), 2);

        let empty_geometry = [LineSeg {
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }];
        assert!(!frame.retain_preserved_single_segment_rows(&empty_geometry));
        assert_eq!(frame.row_count(), 2);
    }

    #[test]
    fn one_physical_row_projects_each_carved_interval_with_shared_metrics() {
        let mut frame = frame(
            0..100,
            200,
            vec![FrameExclusion {
                horizontal: 35..65,
                vertical: 0..1_000,
            }],
        );
        assert_eq!(frame.carve(400), &[0..35, 65..100]);

        assert_eq!(
            frame.commit_carved_row(
                metrics(400, 50),
                vec![
                    RowSegment::new(0..4, 0..35, LineSeg::TAG_FIRST_SEGMENT),
                    RowSegment::new(
                        4..9,
                        65..100,
                        LineSeg::TAG_LAST_SEGMENT | LineSeg::TAG_IMPLEMENTATION_PROPERTY,
                    ),
                ],
            ),
            Some(0)
        );
        assert_eq!(frame.top, 650);

        let projected = frame.project_line_segs();
        assert_eq!(projected.len(), 2);
        assert_eq!(projected[0].vertical_pos, 200);
        assert_eq!(projected[1].vertical_pos, 200);
        assert_eq!(projected[0].line_height, projected[1].line_height);
        assert!(projected[0].is_first_segment());
        assert!(!projected[0].is_last_segment());
        assert!(!projected[1].is_first_segment());
        assert!(projected[1].is_last_segment());
        assert_eq!(projected[0].column_start, 0);
        assert_eq!(projected[0].segment_width, 35);
        assert_eq!(projected[1].column_start, 65);
        assert_eq!(projected[1].segment_width, 35);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IntRect {
    pub(crate) x0: u32,
    pub(crate) y0: u32,
    pub(crate) x1: u32,
    pub(crate) y1: u32,
}

impl IntRect {
    pub(crate) fn from_ltrb(x0: u32, y0: u32, x1: u32, y1: u32) -> Self {
        Self { x0, y0, x1, y1 }
    }

    pub(crate) fn from_xywh(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self {
            x0: x,
            y0: y,
            x1: x + w,
            y1: y + h,
        }
    }

    pub(crate) fn width(&self) -> u32 {
        // See B-11.
        self.x1 - self.x0
    }

    pub(crate) fn height(&self) -> u32 {
        // See B-11.
        self.y1 - self.y0
    }

    pub(crate) fn intersect(&self, other: Self) -> Self {
        if self.x1 < other.x0 || other.x1 < self.x0 || self.y1 < other.y0 || other.y1 < self.y0 {
            Self::from_xywh(0, 0, 0, 0)
        } else {
            Self::from_ltrb(
                u32::max(self.x0, other.x0),
                u32::max(self.y0, other.y0),
                u32::min(self.x1, other.x1),
                u32::min(self.y1, other.y1),
            )
        }
    }
}

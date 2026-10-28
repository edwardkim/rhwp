//! Bounded source-primitive witness carried by an unsafe-to-copy PNG chunk.
//! The bitmap remains the native-format fallback; rhwp can replay the exact
//! axial Butt stroke rather than filtering each cropped half independently.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Witness {
    png_sha256: [u8; 32],
    stroke: PdfStroke,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PdfStroke {
    pub frame: [f64; 2],
    pub endpoints: [f64; 4],
    pub width: f64,
    pub color: u32,
    pub alpha: f64,
    pub dash: Vec<f64>,
    pub phase: f64,
    /// Additional disconnected axial subpaths, with dash phase reset at each MoveTo.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub segments: Vec<[f64; 4]>,
}
impl PdfStroke {
    fn valid(&self) -> bool {
        let [x, y, a, b] = self.endpoints;
        self.frame
            .iter()
            .chain(self.endpoints.iter())
            .chain([&self.width, &self.alpha, &self.phase])
            .all(|v| v.is_finite() && v.abs() <= 100_000.)
            && self.frame.iter().all(|v| *v > 0.)
            && self.width > 0.
            && (0. ..=1.).contains(&self.alpha)
            && ((x == a && y != b) || (y == b && x != a))
            && self.dash.len() <= 64
            && self
                .dash
                .iter()
                .all(|v| v.is_finite() && *v >= 0. && *v <= 100_000.)
            && (self.dash.is_empty() || self.dash.iter().sum::<f64>() > 0.)
            && self.segments.len() < 32
            && self.segments.iter().all(|[x, y, a, b]| {
                [x, y, a, b]
                    .iter()
                    .all(|v| v.is_finite() && v.abs() <= 100_000.)
                    && ((x == a && y != b) || (y == b && x != a))
            })
    }
    pub(crate) fn attach(&self, png: &mut Vec<u8>) -> Option<()> {
        if !self.valid() || png.get(..16)? != b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR" {
            return None;
        }
        let data = serde_json::to_vec(&Witness {
            png_sha256: Sha256::digest(&*png).into(),
            stroke: Self {
                frame: self.frame,
                endpoints: self.endpoints,
                width: self.width,
                color: self.color,
                alpha: self.alpha,
                dash: self.dash.clone(),
                phase: self.phase,
                segments: self.segments.clone(),
            },
        })
        .ok()?;
        if data.len() > 4096 {
            return None;
        }
        let mut chunk = Vec::with_capacity(data.len() + 12);
        chunk.extend((data.len() as u32).to_be_bytes());
        chunk.extend(b"rhST");
        chunk.extend(data);
        chunk.extend(crc32fast::hash(&chunk[4..]).to_be_bytes());
        png.splice(33..33, chunk);
        Some(())
    }
    pub(crate) fn read(png: &[u8]) -> Option<Self> {
        if png.get(..16)? != b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR" || png.get(37..41)? != b"rhST" {
            return None;
        }
        let n = u32::from_be_bytes(png.get(33..37)?.try_into().ok()?) as usize;
        if n > 4096 {
            return None;
        }
        let data = png.get(41..41 + n)?;
        let crc = u32::from_be_bytes(png.get(41 + n..45 + n)?.try_into().ok()?);
        if crc32fast::hash(png.get(37..41 + n)?) != crc {
            return None;
        }
        let witness: Witness = serde_json::from_slice(data).ok()?;
        let mut hash = Sha256::new();
        hash.update(&png[..33]);
        hash.update(png.get(45 + n..)?);
        let actual: [u8; 32] = hash.finalize().into();
        (actual == witness.png_sha256 && witness.stroke.valid()).then_some(witness.stroke)
    }
    /// Actual painted rectangles in source-frame points, including phase and
    /// path direction. Bounds/crop are applied by the common paint projection.
    pub(crate) fn rectangles(&self) -> Option<Vec<[f64; 4]>> {
        let mut all = Vec::new();
        for endpoints in std::iter::once(self.endpoints).chain(self.segments.iter().copied()) {
            all.extend(self.segment_rectangles(endpoints)?);
            if all.len() > 8192 {
                return None;
            }
        }
        Some(all)
    }
    pub(crate) fn segment_rectangles(&self, endpoints: [f64; 4]) -> Option<Vec<[f64; 4]>> {
        let [x, y, a, b] = endpoints;
        let horizontal = y == b;
        let length = if horizontal {
            (a - x).abs()
        } else {
            (b - y).abs()
        };
        let sign = if (horizontal && a > x) || (!horizontal && b > y) {
            1.
        } else {
            -1.
        };
        let mut dash = self.dash.clone();
        if dash.len() % 2 == 1 {
            dash.extend(self.dash.iter().copied());
        }
        let mut rects = Vec::new();
        let rectangle = |start: f64, end: f64| {
            if horizontal {
                let p = x + sign * start;
                let q = x + sign * end;
                [p.min(q), y - self.width / 2., p.max(q), y + self.width / 2.]
            } else {
                let p = y + sign * start;
                let q = y + sign * end;
                [x - self.width / 2., p.min(q), x + self.width / 2., p.max(q)]
            }
        };
        if dash.is_empty() {
            return Some(vec![rectangle(0., length)]);
        }
        let mut phase = self.phase.rem_euclid(dash.iter().sum());
        let mut index = 0;
        while phase >= dash[index] {
            phase -= dash[index];
            index = (index + 1) % dash.len();
        }
        let mut at = 0.;
        let mut remain = dash[index] - phase;
        for _ in 0..8192 {
            let end = (at + remain).min(length);
            if index % 2 == 0 && end > at {
                rects.push(rectangle(at, end));
            }
            if end >= length {
                return Some(rects);
            }
            at = end;
            index = (index + 1) % dash.len();
            remain = dash[index];
        }
        None
    }
}

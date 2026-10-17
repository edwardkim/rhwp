//! Solid path colour state (ISO 32000-1:2008 8.6.5.5, 8.6.8 Table 74).
//! Resolve resource names in their current page/Form scope, then retain the
//! resolved space in q/Q state. The bounded importer has no ICC CMM enabled;
//! ICCBased uses its declared Alternate (or the specified N-based default).
//! Other colour families remain unsupported here, rather than guessing RGB
//! from an arbitrary operand count.

use crate::{
    content::Operator,
    document::PdfDocument,
    error::{Error, Result},
    extractors::paths::{PathExtractor, PathGraphicsState},
    layout::Color,
    object::Object,
};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy)]
pub(crate) enum DeviceSpace {
    Gray,
    Rgb,
    Cmyk,
}

impl DeviceSpace {
    fn named(name: &str) -> Option<Self> {
        match name {
            "DeviceGray" => Some(Self::Gray),
            "DeviceRGB" => Some(Self::Rgb),
            "DeviceCMYK" => Some(Self::Cmyk),
            _ => None,
        }
    }

    fn components(self) -> usize {
        match self {
            Self::Gray => 1,
            Self::Rgb => 3,
            Self::Cmyk => 4,
        }
    }

    fn rgb(self, c: &[f32]) -> (f32, f32, f32) {
        match self {
            Self::Gray => (c[0], c[0], c[0]),
            Self::Rgb => (c[0], c[1], c[2]),
            Self::Cmyk => crate::color::cmyk_to_rgb(c[0], c[1], c[2], c[3]),
        }
    }
}

/// Fixed-size, allocation-free colour state copied by q/Q. ICC ranges are
/// input ranges; they must not be stretched to [0,1] like image Decode arrays.
#[derive(Debug, Clone, Copy)]
pub(crate) enum PathColorSpace {
    Device(DeviceSpace),
    IccAlternate {
        device: DeviceSpace,
        ranges: [[f32; 2]; 4],
    },
    Unsupported,
}

impl PathColorSpace {
    pub(crate) const GRAY: Self = Self::Device(DeviceSpace::Gray);

    fn rgb(self, components: &[f32]) -> Option<(f32, f32, f32)> {
        let (device, ranges) = match self {
            Self::Device(device) => (device, [[0.0, 1.0]; 4]),
            Self::IccAlternate { device, ranges } => (device, ranges),
            Self::Unsupported => return None,
        };
        if components.len() != device.components() || !components.iter().all(|c| c.is_finite()) {
            return None;
        }
        let mut c = [0.0; 4];
        for (i, value) in components.iter().enumerate() {
            c[i] = value.clamp(ranges[i][0], ranges[i][1]).clamp(0.0, 1.0);
        }
        Some(device.rgb(&c))
    }

    fn initial(self) -> Option<(f32, f32, f32)> {
        match self {
            Self::Device(DeviceSpace::Cmyk) => self.rgb(&[0.0, 0.0, 0.0, 1.0]),
            Self::Device(device) | Self::IccAlternate { device, .. } => {
                self.rgb(&[0.0; 4][..device.components()])
            },
            Self::Unsupported => None,
        }
    }
}

fn depth_check(doc: &PdfDocument, depth: usize) -> Result<()> {
    if let Some(budget) = doc.import_budget() {
        budget.check()?;
        if depth >= budget.limits.reference_depth {
            return Err(budget.reject("path colour space reference depth"));
        }
    } else if depth >= 32 {
        return Err(Error::Decode("path colour space reference depth".into()));
    }
    Ok(())
}

fn shallow<'a>(doc: &PdfDocument, value: &'a Object, mut depth: usize) -> Result<Cow<'a, Object>> {
    let mut value = Cow::Borrowed(value);
    loop {
        depth_check(doc, depth)?;
        let Some(reference) = value.as_reference() else {
            return Ok(value);
        };
        // load_object retains the request's visit/copy accounting. Do not
        // recursively copy an entire resource/profile dictionary here.
        value = Cow::Owned(doc.load_object(reference)?);
        depth += 1;
    }
}

fn resolve(
    doc: &PdfDocument,
    object: &Object,
    resources: Option<&Object>,
    depth: usize,
) -> Result<PathColorSpace> {
    depth_check(doc, depth)?;
    let object = shallow(doc, object, depth)?;
    if let Some(name) = object.as_name() {
        return resolve_name(doc, name, resources, depth);
    }
    let Some(array) = object
        .as_array()
        .filter(|a| a.len() == 2 && a[0].as_name() == Some("ICCBased"))
    else {
        return Ok(PathColorSpace::Unsupported);
    };
    let profile = shallow(doc, &array[1], depth + 1)?;
    let Some(dict) = profile.as_dict() else {
        return Ok(PathColorSpace::Unsupported);
    };
    let Some(n) = dict.get("N") else {
        return Ok(PathColorSpace::Unsupported);
    };
    let n = shallow(doc, n, depth + 1)?.as_integer();
    let default = match n {
        Some(1) => DeviceSpace::Gray,
        Some(3) => DeviceSpace::Rgb,
        Some(4) => DeviceSpace::Cmyk,
        _ => return Ok(PathColorSpace::Unsupported),
    };
    let device = if let Some(alternate) = dict.get("Alternate") {
        match resolve(doc, alternate, resources, depth + 1)? {
            PathColorSpace::Device(device) if device.components() == default.components() => device,
            _ => return Ok(PathColorSpace::Unsupported),
        }
    } else {
        default
    };
    let mut ranges = [[0.0, 1.0]; 4];
    if let Some(range) = dict.get("Range") {
        let range = shallow(doc, range, depth + 1)?;
        let Some(values) = range
            .as_array()
            .filter(|v| v.len() == device.components() * 2)
        else {
            return Ok(PathColorSpace::Unsupported);
        };
        for (target, pair) in ranges.iter_mut().zip(values.chunks_exact(2)) {
            for i in 0..2 {
                let value = shallow(doc, &pair[i], depth + 1)?;
                let Some(number) = value
                    .as_real()
                    .or_else(|| value.as_integer().map(|n| n as f64))
                else {
                    return Ok(PathColorSpace::Unsupported);
                };
                target[i] = number as f32;
            }
            if !target.iter().all(|v| v.is_finite()) || target[0] > target[1] {
                return Ok(PathColorSpace::Unsupported);
            }
        }
    }
    Ok(PathColorSpace::IccAlternate { device, ranges })
}

fn resolve_name(
    doc: &PdfDocument,
    name: &str,
    resources: Option<&Object>,
    depth: usize,
) -> Result<PathColorSpace> {
    depth_check(doc, depth)?;
    // Built-in names never resolve through /ColorSpace (Table 74).
    if let Some(device) = DeviceSpace::named(name) {
        return Ok(PathColorSpace::Device(device));
    }
    if name == "Pattern" {
        return Ok(PathColorSpace::Unsupported);
    }
    if let Some(resources) = resources {
        let res = shallow(doc, resources, depth + 1)?;
        if let Some(dictionary) = res.as_dict().and_then(|d| d.get("ColorSpace")) {
            let dictionary = shallow(doc, dictionary, depth + 1)?;
            if let Some(value) = dictionary.as_dict().and_then(|d| d.get(name)) {
                return resolve(doc, value, Some(resources), depth + 1);
            }
        }
    }
    return Ok(PathColorSpace::Unsupported);
}

impl PdfDocument {
    /// One colour operator implementation for page and nested Form extraction.
    pub(crate) fn process_path_color_operator(
        &self,
        op: &Operator,
        extractor: &mut PathExtractor,
        state: &mut PathGraphicsState,
    ) -> Result<bool> {
        use Operator::*;
        let (stroke, space, rgb) = match op {
            SetStrokeColorSpace { name } | SetFillColorSpace { name } => {
                let space = resolve_name(self, name, extractor.current_resources(), 0)?;
                (matches!(op, SetStrokeColorSpace { .. }), Some(space), space.initial())
            },
            SetStrokeColor { components }
            | SetFillColor { components }
            | SetStrokeColorN {
                components,
                name: None,
            }
            | SetFillColorN {
                components,
                name: None,
            } => {
                let stroke = matches!(op, SetStrokeColor { .. } | SetStrokeColorN { .. });
                let space = if stroke {
                    state.stroke_color_space
                } else {
                    state.fill_color_space
                };
                (stroke, None, space.rgb(components))
            },
            SetStrokeGray { gray } | SetFillGray { gray } => (
                matches!(op, SetStrokeGray { .. }),
                Some(PathColorSpace::GRAY),
                PathColorSpace::GRAY.rgb(&[*gray]),
            ),
            SetStrokeRgb { r, g, b } | SetFillRgb { r, g, b } => {
                let space = PathColorSpace::Device(DeviceSpace::Rgb);
                (matches!(op, SetStrokeRgb { .. }), Some(space), space.rgb(&[*r, *g, *b]))
            },
            SetStrokeCmyk { c, m, y, k } | SetFillCmyk { c, m, y, k } => {
                let space = PathColorSpace::Device(DeviceSpace::Cmyk);
                (matches!(op, SetStrokeCmyk { .. }), Some(space), space.rgb(&[*c, *m, *y, *k]))
            },
            _ => return Ok(false),
        };
        let (current_space, color) = if stroke {
            (&mut state.stroke_color_space, &mut state.stroke_color_rgb)
        } else {
            (&mut state.fill_color_space, &mut state.fill_color_rgb)
        };
        if let Some(space) = space {
            *current_space = space;
        }
        if let Some(rgb) = rgb {
            *color = rgb;
            let color = Color::new(rgb.0, rgb.1, rgb.2);
            if stroke {
                extractor.set_stroke_color(color);
            } else {
                extractor.set_fill_color(color);
            }
        }
        Ok(true)
    }
}

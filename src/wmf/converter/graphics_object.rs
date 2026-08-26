use crate::wmf::{imports::*, parser::*};

#[derive(Clone, Debug)]
pub enum GraphicsObject {
    Brush(Brush),
    Font(Font),
    Palette(Palette),
    Pen(Pen),
    Region(Region),
    Null,
}

#[derive(Clone, Debug)]
pub struct GraphicsObjects(Vec<GraphicsObject>, GraphicsObject);

impl GraphicsObjects {
    pub fn new(v: usize) -> Self {
        Self(vec![GraphicsObject::Null; v], GraphicsObject::Null)
    }

    pub fn delete(&mut self, i: usize) {
        // 손상 WMF 의 DELETEOBJECT 가 개체 표 범위를 벗어난 인덱스를 주면 직접 색인은
        // 패닉(DoS)한다. `get` 과 같은 관용 규약으로 범위 밖 삭제는 무시한다.
        if let Some(slot) = self.0.get_mut(i) {
            *slot = GraphicsObject::Null;
        }
    }

    pub fn get(&self, i: usize) -> &GraphicsObject {
        self.0.get(i).unwrap_or(&self.1)
    }

    pub fn push(&mut self, g: GraphicsObject) {
        for (i, v) in self.0.iter_mut().enumerate() {
            if matches!(&v, GraphicsObject::Null) {
                self.0[i] = g;
                break;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectedGraphicsObject {
    pub brush: Brush,
    pub font: Font,
    pub palette: Option<Palette>,
    pub pen: Pen,
    pub region: Option<Region>,
}

impl Default for SelectedGraphicsObject {
    fn default() -> Self {
        SelectedGraphicsObject {
            brush: Brush::Null,
            font: Font {
                height: 12,
                width: 12,
                escapement: 0,
                orientation: 0,
                weight: 0,
                italic: false,
                underline: false,
                strike_out: false,
                charset: CharacterSet::ANSI_CHARSET,
                out_precision: OutPrecision::OUT_DEFAULT_PRECIS,
                clip_precision: ClipPrecision::CLIP_DEFAULT_PRECIS,
                quality: FontQuality::DEFAULT_QUALITY,
                pitch_and_family: PitchAndFamily {
                    family: FamilyFont::FF_DONTCARE,
                    pitch: PitchFont::DEFAULT_PITCH,
                },
                facename: "System".to_owned(),
                fallback_facename: vec!["System".to_owned()],
            },
            palette: None,
            pen: Pen {
                style: PenStyleSubsection {
                    style: PenStyle::PS_SOLID,
                    end_cap: PenStyle::PS_ENDCAP_FLAT,
                    line_join: PenStyle::PS_JOIN_MITER,
                    typ: PenStyle::PS_SOLID,
                },
                width: PointS { x: 1, y: 0 },
                color_ref: ColorRef::black(),
            },
            region: None,
        }
    }
}

impl SelectedGraphicsObject {
    pub fn brush(mut self, brush: Brush) -> Self {
        self.brush = brush;
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette.into();
        self
    }

    pub fn pen(mut self, pen: Pen) -> Self {
        self.pen = pen;
        self
    }

    pub fn region(mut self, region: Region) -> Self {
        self.region = region.into();
        self
    }
}

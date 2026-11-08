use std::collections::BTreeMap;
use std::fmt::Write;

use base64::Engine;

use super::{
    objects::{self, Object},
    stream::{self, f32_at, i16_at, matrix, rect, u32_at},
};

pub(super) struct Budget {
    pub points: usize,
    nodes: usize,
    bytes: usize,
    clip_id: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            points: 262144,
            nodes: 65536,
            bytes: 16 * 1024 * 1024,
            clip_id: 0,
        }
    }
}

impl Budget {
    /// 표시 전 객체에 저장하는 encoded 문자열도 전체 재생 예산에 포함한다.
    pub(super) fn reserve_object_bytes(&mut self, bytes: usize) -> Option<()> {
        self.bytes = self.bytes.checked_sub(bytes)?;
        Some(())
    }
}

#[derive(Clone)]
struct State {
    world: [f64; 6],
    clips: Vec<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            world: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            clips: Vec::new(),
        }
    }
}

struct Player {
    state: State,
    saved: Vec<(u32, State)>,
    objects: BTreeMap<u16, Object>,
    definitions: String,
    output: String,
}

pub(super) fn render(
    bytes: &[u8],
    viewport: (f64, f64, f64, f64),
    depth: usize,
    budget: &mut Budget,
) -> Option<String> {
    let (left, top, w, h) = viewport;
    if depth >= 16 || [left, top, w, h].iter().any(|n| !n.is_finite()) || w <= 0.0 || h <= 0.0 {
        return None;
    }
    let mut player = Player {
        state: State::default(),
        saved: Vec::new(),
        objects: BTreeMap::new(),
        definitions: String::new(),
        output: String::new(),
    };
    for record in stream::records(bytes)? {
        let flags = record.flags;
        let data = record.data;
        match record.kind {
            0x4008 => {
                if flags & 0x8000 != 0 || flags & 0xff > 63 {
                    return None;
                }
                let object = objects::parse((flags >> 8) & 0x7f, data, depth, budget)?;
                player.objects.insert(flags & 0xff, object);
            }
            0x402a => player.state.world = matrix(data)?,
            0x402b => player.state.world = State::default().world,
            0x402c => {
                let new = matrix(data)?;
                player.state.world = if flags & 0x2000 != 0 {
                    multiply(player.state.world, new)
                } else {
                    multiply(new, player.state.world)
                };
                if player.state.world.iter().any(|n| !n.is_finite()) {
                    return None;
                }
            }
            0x4025 => {
                if player.saved.len() >= 256 {
                    return None;
                }
                player.saved.push((u32_at(data, 0)?, player.state.clone()));
            }
            0x4026 => {
                let id = u32_at(data, 0)?;
                // 상태 ID의 대소로 저장 순서를 추측하지 않는다.
                let index = player.saved.iter().rposition(|(saved, _)| *saved == id)?;
                player.state = player.saved.remove(index).1;
                player.saved.truncate(index);
            }
            0x4030 if flags & 0xff == 2 && f32_at(data, 0)? == 1.0 => {}
            0x4032 => player.clip(Some(rect(data, 0)?), flags, budget)?,
            0x4033 => {
                let Object::Path(path) = player.objects.get(&(flags & 0xff))? else {
                    return None;
                };
                let node = format!("<path d=\"{path}\" fill-rule=\"evenodd\"/>");
                player.clip(Some(node), flags, budget)?;
            }
            0x4034 => {
                let Object::Region(region) = player.objects.get(&(flags & 0xff))? else {
                    return None;
                };
                player.clip(region.clone(), flags, budget)?;
            }
            0x4031 => player.state.clips.clear(),
            0x4014 | 0x4015 => {
                let Object::Path(path) = player.objects.get(&(flags & 0xff))? else {
                    return None;
                };
                let style = if record.kind == 0x4014 {
                    if flags & 0x8000 == 0 {
                        return None;
                    }
                    fill(u32_at(data, 0)?)
                } else {
                    let id = u16::try_from(u32_at(data, 0)?).ok()?;
                    let Object::Pen(style) = player.objects.get(&id)? else {
                        return None;
                    };
                    style.clone()
                };
                let node = format!("<path d=\"{path}\" fill-rule=\"evenodd\" {style}/>");
                player.emit(node, budget)?;
            }
            0x400a if flags & 0x8000 != 0 && flags & 0x4000 == 0 => {
                let style = fill(u32_at(data, 0)?);
                let count = usize::try_from(u32_at(data, 4)?).ok()?;
                if count > data.len().saturating_sub(8) / 16 {
                    return None;
                }
                for i in 0..count {
                    let node = format!("<g {style}>{}</g>", rect(data, 8 + 16 * i)?);
                    player.emit(node, budget)?;
                }
            }
            0x401b => player.image(flags, data, budget)?,
            // 렌더 품질 힌트는 geometry·내용을 바꾸지 않는다. 사용하지 않는 플래그를
            // 조판 예외로 해석하지 않으며 실제 그리기 레코드는 모두 위에서 수용한다.
            0x4001 | 0x4002 | 0x4004 | 0x401e | 0x401f | 0x4021 | 0x4022 | 0x4024 => {}
            _ => return None,
        }
    }
    Some(format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"{left} {top} {w} {h}\"><defs>{}</defs>{}</svg>",
        player.definitions, player.output))
}

impl Player {
    fn emit(&mut self, node: String, budget: &mut Budget) -> Option<()> {
        let reserve = node
            .len()
            .checked_add(self.state.clips.len().checked_mul(100)?)?
            .checked_add(200)?;
        budget.nodes = budget.nodes.checked_sub(1)?;
        budget.bytes = budget.bytes.checked_sub(reserve)?;
        for id in &self.state.clips {
            write!(self.output, "<g clip-path=\"url(#{id})\">").ok()?;
        }
        write!(
            self.output,
            "<g transform=\"{}\">{node}</g>",
            transform(self.state.world)
        )
        .ok()?;
        for _ in &self.state.clips {
            self.output.push_str("</g>");
        }
        Some(())
    }

    fn clip(&mut self, shape: Option<String>, flags: u16, budget: &mut Budget) -> Option<()> {
        let mode = (flags >> 8) & 15;
        if !matches!(mode, 0 | 1) {
            return None;
        }
        if mode == 0 {
            self.state.clips.clear();
        }
        if let Some(shape) = shape {
            budget.nodes = budget.nodes.checked_sub(1)?;
            budget.bytes = budget.bytes.checked_sub(shape.len().checked_add(250)?)?;
            budget.clip_id += 1;
            let id = format!("emfplus_clip{}", budget.clip_id);
            // clipPath의 직접 자식 도형에 변환을 적용한다. 그룹을 넣으면
            // 브라우저가 클리핑 도형으로 수용하지 않아 전체 그림이 사라진다.
            let shape = shape.strip_suffix("/>")?;
            write!(self.definitions, "<clipPath id=\"{id}\" clipPathUnits=\"userSpaceOnUse\">{shape} transform=\"{}\"/></clipPath>", transform(self.state.world)).ok()?;
            self.state.clips.push(id);
        }
        Some(())
    }

    fn image(&mut self, flags: u16, data: &[u8], budget: &mut Budget) -> Option<()> {
        let integer_points = flags & 0x4000 != 0;
        if flags & 0x0800 != 0
            || data.len() != if integer_points { 40 } else { 52 }
            || u32_at(data, 4)? != 2
            || u32_at(data, 24)? != 3
        {
            return None;
        }
        let attribute_id = u16::try_from(u32_at(data, 0)?).ok()?;
        if !matches!(self.objects.get(&attribute_id)?, Object::ImageAttributes) {
            return None;
        }
        let Object::Image(image) = self.objects.get(&(flags & 0xff))? else {
            return None;
        };
        let (sx, sy, sw, sh) = (
            f32_at(data, 8)?,
            f32_at(data, 12)?,
            f32_at(data, 16)?,
            f32_at(data, 20)?,
        );
        if sw <= 0.0 || sh <= 0.0 {
            return None;
        }
        // C=1의 signed Point와 C=0의 PointF는 같은 목적 삼각형으로 이어진다.
        let point = |i: usize| {
            if integer_points {
                Some((
                    f64::from(i16_at(data, 28 + i * 4)?),
                    f64::from(i16_at(data, 30 + i * 4)?),
                ))
            } else {
                Some((f32_at(data, 28 + i * 8)?, f32_at(data, 32 + i * 8)?))
            }
        };
        let (x0, y0) = point(0)?;
        let (x1, y1) = point(1)?;
        let (x2, y2) = point(2)?;
        let (a, b, c, d) = (
            (x1 - x0) / sw,
            (y1 - y0) / sw,
            (x2 - x0) / sh,
            (y2 - y0) / sh,
        );
        let placement = [a, b, c, d, x0 - a * sx - c * sy, y0 - b * sx - d * sy];
        let encoded = base64::engine::general_purpose::STANDARD.encode(image.as_bytes());
        let xml = roxmltree::Document::parse(image).ok()?;
        let root = xml.root_element();
        let w: f64 = root.attribute("width")?.parse().ok()?;
        let h: f64 = root.attribute("height")?.parse().ok()?;
        let bounds: Vec<f64> = root
            .attribute("viewBox")?
            .split_ascii_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        let [left, top, _, _] = bounds.as_slice() else {
            return None;
        };
        // TileFlipXY를 원본 좌표에서 재생한다. SrcRect 바깥으로 그리지 않는다.
        budget.clip_id += 1;
        let pattern = format!("emfplus_image{}", budget.clip_id);
        let mut tiles = String::new();
        for (mirror_x, mirror_y) in [(false, false), (true, false), (false, true), (true, true)] {
            let (ax, ay) = (
                if mirror_x { -1.0 } else { 1.0 },
                if mirror_y { -1.0 } else { 1.0 },
            );
            let tx = if mirror_x { 2.0 * (left + w) } else { 0.0 };
            let ty = if mirror_y { 2.0 * (top + h) } else { 0.0 };
            write!(tiles,"<image x=\"{left}\" y=\"{top}\" width=\"{w}\" height=\"{h}\" href=\"data:image/svg+xml;base64,{encoded}\" transform=\"matrix({ax} 0 0 {ay} {tx} {ty})\"/>").ok()?;
        }
        let node=format!("<g transform=\"{}\"><defs><pattern id=\"{pattern}\" patternUnits=\"userSpaceOnUse\" x=\"{left}\" y=\"{top}\" width=\"{}\" height=\"{}\" viewBox=\"{left} {top} {} {}\">{tiles}</pattern></defs><rect x=\"{sx}\" y=\"{sy}\" width=\"{sw}\" height=\"{sh}\" fill=\"url(#{pattern})\"/></g>", transform(placement),2.0*w,2.0*h,2.0*w,2.0*h);
        self.emit(node, budget)
    }
}

fn fill(color: u32) -> String {
    format!(
        "fill=\"#{:06x}\" fill-opacity=\"{}\" stroke=\"none\"",
        color & 0xffffff,
        f64::from(color >> 24) / 255.0
    )
}

fn transform(m: [f64; 6]) -> String {
    format!(
        "matrix({} {} {} {} {} {})",
        m[0], m[1], m[2], m[3], m[4], m[5]
    )
}

fn multiply(p: [f64; 6], q: [f64; 6]) -> [f64; 6] {
    let [a, b, c, d, e, f] = p;
    let [aa, bb, cc, dd, ee, ff] = q;
    [
        a * aa + b * cc,
        a * bb + b * dd,
        c * aa + d * cc,
        c * bb + d * dd,
        e * aa + f * cc + ee,
        e * bb + f * dd + ff,
    ]
}

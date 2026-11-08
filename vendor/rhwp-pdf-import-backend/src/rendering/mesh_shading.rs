//! Mesh and function-based shadings (ISO 32000-1 §8.7.4.5.5–§8.7.4.5.8
//! and §8.7.4.5.2).
//!
//! tiny-skia's gradient shaders cover only the axial (Type 2) and radial
//! (Type 3) shadings. The remaining shading dictionary types carry an
//! explicit geometry stream (Types 4–7) or a colour function evaluated
//! over a domain rectangle (Type 1); none map onto a tiny-skia shader, so
//! they are rasterised here by hand:
//!
//! - **Type 4** free-form Gouraud triangles — a bit-packed vertex stream
//!   with a per-vertex edge flag that stitches triangles together.
//! - **Type 5** lattice-form Gouraud triangles — a flag-free vertex grid
//!   of `/VerticesPerRow` columns, tessellated row by row.
//! - **Type 6** Coons patch meshes — cubic-Bézier boundary patches with
//!   four corner colours, subdivided into a Gouraud grid.
//! - **Type 7** tensor-product patch meshes — like Type 6 but with four
//!   extra interior control points (bicubic surface).
//! - **Type 1** function-based shadings — a `/Function` sampled over the
//!   `/Domain` rectangle, mapped through `/Matrix`.
//!
//! Colours read from the stream (or produced by the shading's optional
//! `/Function`) are handed back to the caller through a `resolve_color`
//! closure so they travel the same colour-space resolution path (§8.6) as
//! every other painted colour. Triangles are filled with barycentric
//! interpolation of the per-vertex RGBA; patches interpolate the four
//! corner RGBAs bilinearly across the subdivision grid.
//!
//! All stream reads are bounded (a short/malformed stream stops decoding
//! and paints what was decoded so far) and every count is capped, so a
//! hostile or corrupt shading can neither panic nor hang — it simply
//! paints less. This mirrors the pre-existing "unsupported shading →
//! leave unpainted" behaviour when nothing at all can be decoded.

use crate::document::PdfDocument;
use crate::error::Result;
use crate::object::Object;
use std::collections::HashMap;
use tiny_skia::{Mask, Pixmap, Transform};

/// Hard cap on the number of triangles rasterised for one shading. Meshes
/// this large are almost always malformed; the cap bounds worst-case work.
const MAX_TRIANGLES: usize = 4_000_000;
/// Hard cap on Coons/tensor patches decoded for one shading.
const MAX_PATCHES: usize = 500_000;
/// Upper bound on the per-patch subdivision grid (N×N cells).
const MAX_SUBDIV: usize = 10;
/// Upper bound on the Type 1 domain sampling grid (N×N nodes).
const MAX_TYPE1_GRID: usize = 128;

/// A colour resolver: maps colour-space components (already in the
/// shading's `/ColorSpace`) to straight-alpha RGBA. Supplied by the
/// renderer so mesh colours travel the standard §8.6 resolution path.
pub(crate) type ColorResolver<'a> = dyn Fn(&[f32]) -> Option<(f32, f32, f32, f32)> + 'a;

/// Entry point invoked from the `sh`/`render_shading` dispatcher for
/// shading types tiny-skia cannot express as a gradient shader.
///
/// `shading` is the shading dictionary; `shading_obj` is the full resolved
/// object (a stream for Types 4–7, whose bytes carry the geometry).
/// `transform` maps shading space to device space. `resolve_color` maps
/// colour-space components to RGBA. Returns `Ok(())` in every non-fatal
/// case — an unsupported bit depth or malformed stream logs and paints
/// nothing rather than erroring.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_mesh_shading(
    pixmap: &mut Pixmap,
    shading: &HashMap<String, Object>,
    shading_obj: &Object,
    shading_type: i64,
    transform: Transform,
    doc: &PdfDocument,
    clip_mask: Option<&Mask>,
    resolve_color: &ColorResolver<'_>,
) -> Result<()> {
    // The optional `/Function` remaps a single parametric value carried by
    // each vertex/patch corner (or, for Type 1, the 2-D domain point) into
    // the shading colour space's components. When absent the stream carries
    // the colour-space components directly.
    let function = shading
        .get("Function")
        .and_then(|f| doc.resolve_object(f).ok());

    // Resolve a set of stream/function colour components to RGBA, routing
    // through `/Function` first when present.
    // A Type 0 function carries its samples in a stream. `to_rgba` runs once
    // per grid point — up to 129x129 for a Type 1 shading — so decoding that
    // stream inside the evaluation made the cost of the samples proportional
    // to the grid rather than paid once. Decode it here instead.
    let sampled_bytes: Option<Vec<u8>> = function.as_ref().and_then(|f| {
        let is_sampled = f
            .as_dict()
            .and_then(|d| d.get("FunctionType"))
            .and_then(|o| o.as_integer())
            == Some(0);
        is_sampled.then(|| f.decode_stream_data().ok()).flatten()
    });

    let to_rgba = |comps: &[f32]| -> (f32, f32, f32, f32) {
        let cs_comps: Vec<f32> = match &function {
            Some(f) => eval_pdf_function(f, doc, comps, sampled_bytes.as_deref())
                .unwrap_or_else(|| comps.to_vec()),
            None => comps.to_vec(),
        };
        resolve_color(&cs_comps).unwrap_or((0.0, 0.0, 0.0, 1.0))
    };

    match shading_type {
        1 => render_function_based(pixmap, shading, transform, clip_mask, &to_rgba),
        4..=7 => {
            let data = match shading_obj.decode_stream_data() {
                Ok(d) => d,
                Err(e) => {
                    log::debug!("Mesh shading type {shading_type}: stream decode failed: {e}");
                    return Ok(());
                },
            };
            let params = match MeshParams::parse(shading) {
                Some(p) => p,
                None => {
                    log::debug!("Mesh shading type {shading_type}: missing/invalid stream params");
                    return Ok(());
                },
            };
            match shading_type {
                4 => {
                    let tris = decode_type4_stream(&data, &params, MAX_TRIANGLES);
                    rasterize_raw_triangles(pixmap, &tris, transform, clip_mask, &to_rgba);
                },
                5 => {
                    let tris = decode_type5_stream(&data, &params, MAX_TRIANGLES);
                    rasterize_raw_triangles(pixmap, &tris, transform, clip_mask, &to_rgba);
                },
                6 | 7 => {
                    let is_tensor = shading_type == 7;
                    let patches = decode_patches(&data, is_tensor, &params, MAX_PATCHES);
                    render_patches(pixmap, &patches, is_tensor, transform, clip_mask, &to_rgba);
                },
                _ => unreachable!(),
            }
            Ok(())
        },
        other => {
            log::debug!("Unsupported shading type {other} in mesh renderer");
            Ok(())
        },
    }
}

// ===========================================================================
// Bit-packed stream reader (MSB-first, no per-field byte alignment).
// ===========================================================================

/// Sequential MSB-first bit reader over a byte slice. Every read is
/// bounded: once the cursor passes the end of the buffer, reads return
/// `None` and decoding stops gracefully.
struct BitReader<'a> {
    data: &'a [u8],
    bit_pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, bit_pos: 0 }
    }

    /// Total bits remaining.
    fn remaining(&self) -> usize {
        (self.data.len() * 8).saturating_sub(self.bit_pos)
    }

    /// Read `nbits` (0..=32) as an unsigned integer, MSB first. Returns
    /// `None` when fewer than `nbits` remain.
    fn read_bits(&mut self, nbits: u32) -> Option<u64> {
        let nbits = nbits as usize;
        if nbits == 0 {
            return Some(0);
        }
        if nbits > 32 || self.remaining() < nbits {
            return None;
        }
        let mut value: u64 = 0;
        for _ in 0..nbits {
            let byte = self.data[self.bit_pos >> 3];
            let bit = (byte >> (7 - (self.bit_pos & 7))) & 1;
            value = (value << 1) | bit as u64;
            self.bit_pos += 1;
        }
        Some(value)
    }
}

/// Map a raw `nbits`-wide unsigned integer onto `[lo, hi]` per a `/Decode`
/// pair (§8.7.4.5.5). `2^nbits - 1` is the maximum representable value.
fn decode_value(raw: u64, nbits: u32, lo: f32, hi: f32) -> f32 {
    let max = if nbits >= 64 {
        u64::MAX
    } else {
        (1u64 << nbits) - 1
    };
    if max == 0 {
        return lo;
    }
    let t = raw as f32 / max as f32;
    lo + t * (hi - lo)
}

// ===========================================================================
// Shared mesh stream parameters.
// ===========================================================================

/// Bit widths and the `/Decode` ranges shared by Types 4–7.
struct MeshParams {
    bits_per_flag: u32,
    bits_per_coord: u32,
    bits_per_comp: u32,
    /// `/VerticesPerRow` (Type 5 only; ignored otherwise).
    vertices_per_row: usize,
    /// `[x, y, c0, c1, ...]` decode ranges. `ncomps == decode.len() - 2`.
    decode: Vec<(f32, f32)>,
}

impl MeshParams {
    fn parse(shading: &HashMap<String, Object>) -> Option<Self> {
        let bits_per_coord = shading.get("BitsPerCoordinate")?.as_integer()? as u32;
        let bits_per_comp = shading.get("BitsPerComponent")?.as_integer()? as u32;
        // BitsPerFlag is absent on Type 5 lattices; default to a byte.
        let bits_per_flag = shading
            .get("BitsPerFlag")
            .and_then(|o| o.as_integer())
            .unwrap_or(8) as u32;
        if bits_per_coord == 0 || bits_per_coord > 32 || bits_per_comp == 0 || bits_per_comp > 32 {
            return None;
        }
        if bits_per_flag > 32 {
            return None;
        }
        let decode_arr = shading.get("Decode")?.as_array()?;
        let decode: Vec<(f32, f32)> = decode_arr
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| (num(&c[0]), num(&c[1])))
            .collect();
        // Need at least the x and y ranges plus one colour component.
        if decode.len() < 3 {
            return None;
        }
        let vertices_per_row = shading
            .get("VerticesPerRow")
            .and_then(|o| o.as_integer())
            .unwrap_or(0)
            .max(0) as usize;
        Some(Self {
            bits_per_flag,
            bits_per_coord,
            bits_per_comp,
            vertices_per_row,
            decode,
        })
    }

    fn ncomps(&self) -> usize {
        self.decode.len() - 2
    }

    /// Read one `(x, y, comps)` vertex body (no flag) from the reader.
    fn read_vertex(&self, reader: &mut BitReader) -> Option<RawVertex> {
        let rx = reader.read_bits(self.bits_per_coord)?;
        let ry = reader.read_bits(self.bits_per_coord)?;
        let x = decode_value(rx, self.bits_per_coord, self.decode[0].0, self.decode[0].1);
        let y = decode_value(ry, self.bits_per_coord, self.decode[1].0, self.decode[1].1);
        let comps = self.read_color(reader)?;
        Some(RawVertex { x, y, comps })
    }

    /// Read the colour components (no coordinates) from the reader.
    fn read_color(&self, reader: &mut BitReader) -> Option<Vec<f32>> {
        let n = self.ncomps();
        let mut comps = Vec::with_capacity(n);
        for i in 0..n {
            let raw = reader.read_bits(self.bits_per_comp)?;
            let (lo, hi) = self.decode[2 + i];
            comps.push(decode_value(raw, self.bits_per_comp, lo, hi));
        }
        Some(comps)
    }
}

/// A vertex in shading space with its raw colour-space components.
#[derive(Clone, Debug)]
struct RawVertex {
    x: f32,
    y: f32,
    comps: Vec<f32>,
}

// ===========================================================================
// Type 4 — free-form Gouraud triangles.
// ===========================================================================

/// Decode a Type 4 free-form Gouraud-triangle stream into raw triangles.
/// The per-vertex edge flag stitches successive triangles: flag 0 starts a
/// fresh triangle (its two successors complete it); flags 1/2 reuse an edge
/// of the previous triangle (§8.7.4.5.5, Table 84).
fn decode_type4_stream(data: &[u8], p: &MeshParams, max_tris: usize) -> Vec<[RawVertex; 3]> {
    let mut reader = BitReader::new(data);
    let mut tris: Vec<[RawVertex; 3]> = Vec::new();
    // Previous triangle vertices (va, vb, vc) for shared-edge flags.
    let mut prev: Option<[RawVertex; 3]> = None;

    while reader.remaining() >= p.bits_per_flag as usize && tris.len() < max_tris {
        let flag = match reader.read_bits(p.bits_per_flag) {
            Some(f) => f,
            None => break,
        };
        let v = match p.read_vertex(&mut reader) {
            Some(v) => v,
            None => break,
        };

        let tri = if flag == 0 {
            // New triangle: read the two remaining vertices (their flags are
            // 0 per spec and are consumed and ignored).
            let _f2 = reader.read_bits(p.bits_per_flag);
            let v2 = match p.read_vertex(&mut reader) {
                Some(v) => v,
                None => break,
            };
            let _f3 = reader.read_bits(p.bits_per_flag);
            let v3 = match p.read_vertex(&mut reader) {
                Some(v) => v,
                None => break,
            };
            [v, v2, v3]
        } else {
            let prev_tri = match &prev {
                Some(t) => t,
                // A shared-edge flag with no predecessor is malformed; stop.
                None => break,
            };
            match flag {
                // Share the (vb, vc) edge of the previous triangle.
                1 => [prev_tri[1].clone(), prev_tri[2].clone(), v],
                // Share the (va, vc) edge of the previous triangle.
                2 => [prev_tri[0].clone(), prev_tri[2].clone(), v],
                _ => break,
            }
        };

        prev = Some(tri.clone());
        tris.push(tri);
    }
    tris
}

// ===========================================================================
// Type 5 — lattice-form Gouraud triangles.
// ===========================================================================

/// Decode a Type 5 lattice-form stream (no flags) into raw triangles.
/// Vertices are read row by row (`/VerticesPerRow` columns); each pair of
/// adjacent rows forms a strip of two triangles per cell (§8.7.4.5.6).
fn decode_type5_stream(data: &[u8], p: &MeshParams, max_tris: usize) -> Vec<[RawVertex; 3]> {
    let vpr = p.vertices_per_row;
    if vpr < 2 {
        return Vec::new();
    }
    let mut reader = BitReader::new(data);
    let mut tris: Vec<[RawVertex; 3]> = Vec::new();
    let mut prev_row: Option<Vec<RawVertex>> = None;

    loop {
        // Read a full row; a short final row ends the mesh.
        let mut row = Vec::with_capacity(vpr);
        for _ in 0..vpr {
            match p.read_vertex(&mut reader) {
                Some(v) => row.push(v),
                None => break,
            }
        }
        if row.len() < vpr {
            break;
        }
        if let Some(top) = &prev_row {
            for i in 0..vpr - 1 {
                if tris.len() + 2 > max_tris {
                    return tris;
                }
                // Two triangles per lattice cell.
                tris.push([top[i].clone(), top[i + 1].clone(), row[i].clone()]);
                tris.push([top[i + 1].clone(), row[i + 1].clone(), row[i].clone()]);
            }
        }
        prev_row = Some(row);
    }
    tris
}

// ===========================================================================
// Types 6 & 7 — Coons / tensor-product patches.
// ===========================================================================

type Pt = (f32, f32);

/// A decoded patch: 12 boundary control points (`p1..p12`), 4 interior
/// points (`p13..p16`, tensor only; zeroed for Coons) and 4 corner colour
/// component arrays (`c1..c4`).
struct Patch {
    boundary: [Pt; 12],
    interior: [Pt; 4],
    colors: [Vec<f32>; 4],
}

/// Decode a Type 6 (Coons) or Type 7 (tensor) patch stream. Shared-edge
/// flags reuse four boundary points and two corner colours of the previous
/// patch (§8.7.4.5.7 Table 85 / §8.7.4.5.8 Table 86).
fn decode_patches(data: &[u8], is_tensor: bool, p: &MeshParams, max_patches: usize) -> Vec<Patch> {
    let mut reader = BitReader::new(data);
    let mut patches: Vec<Patch> = Vec::new();
    let mut prev: Option<Patch> = None;
    let total_points = if is_tensor { 16 } else { 12 };

    let read_point = |r: &mut BitReader| -> Option<Pt> {
        let rx = r.read_bits(p.bits_per_coord)?;
        let ry = r.read_bits(p.bits_per_coord)?;
        Some((
            decode_value(rx, p.bits_per_coord, p.decode[0].0, p.decode[0].1),
            decode_value(ry, p.bits_per_coord, p.decode[1].0, p.decode[1].1),
        ))
    };

    while reader.remaining() >= p.bits_per_flag as usize && patches.len() < max_patches {
        let flag = match reader.read_bits(p.bits_per_flag) {
            Some(f) => f,
            None => break,
        };

        let mut boundary = [(0.0f32, 0.0f32); 12];
        let mut interior = [(0.0f32, 0.0f32); 4];
        let mut colors: [Vec<f32>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        // Index of the first freshly-read boundary point and colour.
        let (start_pt, start_color) = if flag == 0 {
            (0usize, 0usize)
        } else {
            let prev_patch = match &prev {
                Some(pp) => pp,
                None => break,
            };
            // Reuse four boundary points and two corner colours of the
            // previous patch, selected by the shared-edge flag.
            let (pts, cols): ([usize; 4], [usize; 2]) = match flag {
                1 => ([3, 4, 5, 6], [1, 2]),
                2 => ([6, 7, 8, 9], [2, 3]),
                3 => ([9, 10, 11, 0], [3, 0]),
                _ => break,
            };
            for (dst, &src) in pts.iter().enumerate() {
                boundary[dst] = prev_patch.boundary[src];
            }
            colors[0] = prev_patch.colors[cols[0]].clone();
            colors[1] = prev_patch.colors[cols[1]].clone();
            (4usize, 2usize)
        };

        // Read the remaining boundary points.
        let mut ok = true;
        for slot in boundary.iter_mut().take(12).skip(start_pt) {
            match read_point(&mut reader) {
                Some(pt) => *slot = pt,
                None => {
                    ok = false;
                    break;
                },
            }
        }
        // Tensor patches carry four extra interior points after the
        // boundary; Coons patches do not.
        if ok && is_tensor && total_points == 16 {
            for slot in interior.iter_mut() {
                match read_point(&mut reader) {
                    Some(pt) => *slot = pt,
                    None => {
                        ok = false;
                        break;
                    },
                }
            }
        }
        if ok {
            for slot in colors.iter_mut().take(4).skip(start_color) {
                match p.read_color(&mut reader) {
                    Some(c) => *slot = c,
                    None => {
                        ok = false;
                        break;
                    },
                }
            }
        }
        if !ok {
            break;
        }

        let patch = Patch {
            boundary,
            interior,
            colors,
        };
        // Keep a copy for the next patch's shared-edge reference.
        prev = Some(Patch {
            boundary: patch.boundary,
            interior: patch.interior,
            colors: patch.colors.clone(),
        });
        patches.push(patch);
    }
    patches
}

/// Cubic Bézier point at parameter `t` over four control points.
fn bezier(p0: Pt, p1: Pt, p2: Pt, p3: Pt, t: f32) -> Pt {
    let mt = 1.0 - t;
    let b0 = mt * mt * mt;
    let b1 = 3.0 * mt * mt * t;
    let b2 = 3.0 * mt * t * t;
    let b3 = t * t * t;
    (
        b0 * p0.0 + b1 * p1.0 + b2 * p2.0 + b3 * p3.0,
        b0 * p0.1 + b1 * p1.1 + b2 * p2.1 + b3 * p3.1,
    )
}

/// Evaluate a tensor-product (Type 7) patch surface at `(s, t)`.
/// The 16 control points map onto a 4×4 grid; the surface is the bicubic
/// Bézier combination.
fn tensor_surface(b: &[Pt; 12], interior: &[Pt; 4], s: f32, t: f32) -> Pt {
    // 4×4 control grid `g[row][col]`, row → t (bottom..top), col → s.
    // Boundary ordering per §8.7.4.5.8 Figure; interior p13..p16 fill the
    // centre.
    let g: [[Pt; 4]; 4] = [
        [b[0], b[11], b[10], b[9]],             // row0 (bottom): p1,p12,p11,p10
        [b[1], interior[3], interior[2], b[8]], // row1: p2,p16,p15,p9
        [b[2], interior[0], interior[1], b[7]], // row2: p3,p13,p14,p8
        [b[3], b[4], b[5], b[6]],               // row3 (top): p4,p5,p6,p7
    ];
    let bt = bernstein(t);
    let bs = bernstein(s);
    let mut x = 0.0;
    let mut y = 0.0;
    for (r, brow) in g.iter().enumerate() {
        for (c, pt) in brow.iter().enumerate() {
            let w = bt[r] * bs[c];
            x += w * pt.0;
            y += w * pt.1;
        }
    }
    (x, y)
}

/// Cubic Bernstein basis weights at `t`.
#[inline]
fn bernstein(t: f32) -> [f32; 4] {
    let mt = 1.0 - t;
    [mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t]
}

/// Evaluate a Coons patch surface point at `(s, t)` directly from the 12
/// boundary control points (bilinearly-blended Coons formula).
fn coons_point(b: &[Pt; 12], s: f32, t: f32) -> Pt {
    let left = bezier(b[0], b[1], b[2], b[3], t);
    let right = bezier(b[9], b[8], b[7], b[6], t);
    let bottom = bezier(b[0], b[11], b[10], b[9], s);
    let top = bezier(b[3], b[4], b[5], b[6], s);
    let (p1, p4, p7, p10) = (b[0], b[3], b[6], b[9]);
    let blend = |lb: f32, rb: f32, bb: f32, tb: f32, c1: f32, c4: f32, c7: f32, c10: f32| -> f32 {
        (1.0 - t) * bb + t * tb + (1.0 - s) * lb + s * rb
            - ((1.0 - s) * (1.0 - t) * c1 + s * (1.0 - t) * c10 + (1.0 - s) * t * c4 + s * t * c7)
    };
    (
        blend(left.0, right.0, bottom.0, top.0, p1.0, p4.0, p7.0, p10.0),
        blend(left.1, right.1, bottom.1, top.1, p1.1, p4.1, p7.1, p10.1),
    )
}

/// Rasterise decoded patches by subdividing each into an adaptive N×N grid
/// of Gouraud cells. Corner colours are resolved once per patch and
/// interpolated bilinearly across the grid.
fn render_patches(
    pixmap: &mut Pixmap,
    patches: &[Patch],
    is_tensor: bool,
    transform: Transform,
    clip_mask: Option<&Mask>,
    to_rgba: &dyn Fn(&[f32]) -> (f32, f32, f32, f32),
) {
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);
    for patch in patches {
        // Resolve the four corner colours (RGBA) once.
        let c = [
            to_rgba(&patch.colors[0]),
            to_rgba(&patch.colors[1]),
            to_rgba(&patch.colors[2]),
            to_rgba(&patch.colors[3]),
        ];

        // Adaptive subdivision from the device-space size of the corners.
        let corners = [
            patch.boundary[0],
            patch.boundary[3],
            patch.boundary[6],
            patch.boundary[9],
        ];
        let dev: Vec<Pt> = corners.iter().map(|&p| map_pt(transform, p)).collect();
        let mut extent = 0.0f32;
        for i in 0..dev.len() {
            for j in i + 1..dev.len() {
                let d = ((dev[i].0 - dev[j].0).powi(2) + (dev[i].1 - dev[j].1).powi(2)).sqrt();
                extent = extent.max(d);
            }
        }
        // Skip patches wholly outside the canvas (cheap corner test).
        if dev
            .iter()
            .all(|p| p.0 < 0.0 || p.0 > w || p.1 < 0.0 || p.1 > h)
            && !bbox_intersects_canvas(&dev, w, h)
        {
            continue;
        }
        let n = ((extent / 16.0).ceil() as usize).clamp(1, MAX_SUBDIV);

        // Precompute grid nodes: device point + bilinear RGBA per (i, j).
        let node = |i: usize, j: usize| -> ((f32, f32), (f32, f32, f32, f32)) {
            let s = i as f32 / n as f32;
            let t = j as f32 / n as f32;
            let sp = if is_tensor {
                tensor_surface(&patch.boundary, &patch.interior, s, t)
            } else {
                coons_point(&patch.boundary, s, t)
            };
            let dp = map_pt(transform, sp);
            // Bilinear corner-colour blend: c1@(0,0) c2@(0,1) c3@(1,1) c4@(1,0).
            let col = bilerp_rgba(c[0], c[1], c[2], c[3], s, t);
            (dp, col)
        };

        for i in 0..n {
            for j in 0..n {
                let (p00, c00) = node(i, j);
                let (p10, c10) = node(i + 1, j);
                let (p01, c01) = node(i, j + 1);
                let (p11, c11) = node(i + 1, j + 1);
                fill_gouraud_triangle(pixmap, clip_mask, (p00, c00), (p10, c10), (p11, c11));
                fill_gouraud_triangle(pixmap, clip_mask, (p00, c00), (p11, c11), (p01, c01));
            }
        }
    }
}

/// Bilinear blend of four RGBA corners. `s`, `t` in `[0, 1]`; corners map
/// c1@(0,0), c2@(0,1), c3@(1,1), c4@(1,0).
fn bilerp_rgba(
    c1: (f32, f32, f32, f32),
    c2: (f32, f32, f32, f32),
    c3: (f32, f32, f32, f32),
    c4: (f32, f32, f32, f32),
    s: f32,
    t: f32,
) -> (f32, f32, f32, f32) {
    let w1 = (1.0 - s) * (1.0 - t);
    let w2 = (1.0 - s) * t;
    let w3 = s * t;
    let w4 = s * (1.0 - t);
    (
        w1 * c1.0 + w2 * c2.0 + w3 * c3.0 + w4 * c4.0,
        w1 * c1.1 + w2 * c2.1 + w3 * c3.1 + w4 * c4.1,
        w1 * c1.2 + w2 * c2.2 + w3 * c3.2 + w4 * c4.2,
        w1 * c1.3 + w2 * c2.3 + w3 * c3.3 + w4 * c4.3,
    )
}

// ===========================================================================
// Type 1 — function-based shading.
// ===========================================================================

/// Render a function-based (Type 1) shading: evaluate `/Function` over the
/// `/Domain` rectangle on an adaptive grid, map each node through `/Matrix`
/// then the shading→device transform, and fill the grid cells as Gouraud
/// quads (§8.7.4.5.2).
fn render_function_based(
    pixmap: &mut Pixmap,
    shading: &HashMap<String, Object>,
    transform: Transform,
    clip_mask: Option<&Mask>,
    to_rgba: &dyn Fn(&[f32]) -> (f32, f32, f32, f32),
) -> Result<()> {
    // Domain [x0 x1 y0 y1], default [0 1 0 1].
    let (dx0, dx1, dy0, dy1) = shading
        .get("Domain")
        .and_then(|o| o.as_array())
        .filter(|a| a.len() >= 4)
        .map(|a| (num(&a[0]), num(&a[1]), num(&a[2]), num(&a[3])))
        .unwrap_or((0.0, 1.0, 0.0, 1.0));

    // Matrix maps domain space to the shading's target coordinate space.
    let matrix = shading
        .get("Matrix")
        .and_then(|o| o.as_array())
        .filter(|a| a.len() >= 6)
        .map(|a| {
            [
                num(&a[0]),
                num(&a[1]),
                num(&a[2]),
                num(&a[3]),
                num(&a[4]),
                num(&a[5]),
            ]
        })
        .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    // Map a domain point → device point (Matrix then shading→device CTM).
    let map_domain = |u: f32, v: f32| -> Pt {
        let sx = matrix[0] * u + matrix[2] * v + matrix[4];
        let sy = matrix[1] * u + matrix[3] * v + matrix[5];
        map_pt(transform, (sx, sy))
    };

    // Grid resolution from the device extent of the domain corners.
    let corners = [
        map_domain(dx0, dy0),
        map_domain(dx1, dy0),
        map_domain(dx1, dy1),
        map_domain(dx0, dy1),
    ];
    let mut extent = 0.0f32;
    for i in 0..corners.len() {
        for j in i + 1..corners.len() {
            let d = ((corners[i].0 - corners[j].0).powi(2) + (corners[i].1 - corners[j].1).powi(2))
                .sqrt();
            extent = extent.max(d);
        }
    }
    let n = (extent.ceil() as usize).clamp(2, MAX_TYPE1_GRID);

    // Precompute the (device point, RGBA) grid.
    let mut grid: Vec<((f32, f32), (f32, f32, f32, f32))> = Vec::with_capacity((n + 1) * (n + 1));
    for i in 0..=n {
        for j in 0..=n {
            let u = dx0 + (dx1 - dx0) * (i as f32 / n as f32);
            let v = dy0 + (dy1 - dy0) * (j as f32 / n as f32);
            // `to_rgba` runs the `/Function` on the 2-D domain point.
            let rgba = to_rgba(&[u, v]);
            grid.push((map_domain(u, v), rgba));
        }
    }
    let at = |i: usize, j: usize| grid[i * (n + 1) + j];
    for i in 0..n {
        for j in 0..n {
            let a = at(i, j);
            let b = at(i + 1, j);
            let c = at(i + 1, j + 1);
            let d = at(i, j + 1);
            fill_gouraud_triangle(pixmap, clip_mask, a, b, c);
            fill_gouraud_triangle(pixmap, clip_mask, a, c, d);
        }
    }
    Ok(())
}

// ===========================================================================
// Rasterisation.
// ===========================================================================

/// Resolve each raw triangle's per-vertex colour and rasterise it with
/// barycentric colour interpolation.
fn rasterize_raw_triangles(
    pixmap: &mut Pixmap,
    tris: &[[RawVertex; 3]],
    transform: Transform,
    clip_mask: Option<&Mask>,
    to_rgba: &dyn Fn(&[f32]) -> (f32, f32, f32, f32),
) {
    for tri in tris {
        let mut verts = [((0.0f32, 0.0f32), (0.0f32, 0.0f32, 0.0f32, 0.0f32)); 3];
        for (k, rv) in tri.iter().enumerate() {
            verts[k] = (map_pt(transform, (rv.x, rv.y)), to_rgba(&rv.comps));
        }
        fill_gouraud_triangle(pixmap, clip_mask, verts[0], verts[1], verts[2]);
    }
}

/// Map a shading-space point through the shading→device transform.
#[inline]
fn map_pt(transform: Transform, p: Pt) -> Pt {
    let mut pt = tiny_skia::Point { x: p.0, y: p.1 };
    transform.map_point(&mut pt);
    (pt.x, pt.y)
}

/// True when the device bounding box of the corner points overlaps the
/// canvas rectangle `[0, w] × [0, h]`.
fn bbox_intersects_canvas(corners: &[Pt], w: f32, h: f32) -> bool {
    let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &(x, y) in corners {
        minx = minx.min(x);
        miny = miny.min(y);
        maxx = maxx.max(x);
        maxy = maxy.max(y);
    }
    minx <= w && maxx >= 0.0 && miny <= h && maxy >= 0.0
}

type ColoredVertex = ((f32, f32), (f32, f32, f32, f32));

/// Barycentric-interpolated triangle fill directly into the pixmap's
/// premultiplied RGBA buffer, honouring the clip mask. Colours are
/// straight-alpha RGBA per vertex.
fn fill_gouraud_triangle(
    pixmap: &mut Pixmap,
    clip_mask: Option<&Mask>,
    v0: ColoredVertex,
    v1: ColoredVertex,
    v2: ColoredVertex,
) {
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    if width == 0 || height == 0 {
        return;
    }
    let (p0, c0) = v0;
    let (p1, c1) = v1;
    let (p2, c2) = v2;

    // Device-space bounding box, clamped to the canvas.
    let minx = p0.0.min(p1.0).min(p2.0).floor().max(0.0) as i32;
    let maxx = p0.0.max(p1.0).max(p2.0).ceil().min(width as f32) as i32;
    let miny = p0.1.min(p1.1).min(p2.1).floor().max(0.0) as i32;
    let maxy = p0.1.max(p1.1).max(p2.1).ceil().min(height as f32) as i32;
    if minx >= maxx || miny >= maxy {
        return;
    }

    // Barycentric denominator; a degenerate (zero-area) triangle is skipped.
    let denom = (p1.1 - p2.1) * (p0.0 - p2.0) + (p2.0 - p1.0) * (p0.1 - p2.1);
    if denom.abs() < 1e-9 {
        return;
    }
    let inv_denom = 1.0 / denom;

    let mask_data = clip_mask.map(|m| m.data());
    let dest = pixmap.data_mut();

    for py in miny..maxy {
        for px in minx..maxx {
            let fx = px as f32 + 0.5;
            let fy = py as f32 + 0.5;
            let w0 = ((p1.1 - p2.1) * (fx - p2.0) + (p2.0 - p1.0) * (fy - p2.1)) * inv_denom;
            let w1 = ((p2.1 - p0.1) * (fx - p2.0) + (p0.0 - p2.0) * (fy - p2.1)) * inv_denom;
            let w2 = 1.0 - w0 - w1;
            // Small epsilon so shared edges between adjacent triangles fill.
            if w0 < -1e-4 || w1 < -1e-4 || w2 < -1e-4 {
                continue;
            }

            let mut a = w0 * c0.3 + w1 * c1.3 + w2 * c2.3;
            if a <= 0.0 {
                continue;
            }
            let pixel_idx = (py * width + px) as usize;
            if let Some(md) = mask_data {
                if let Some(&m) = md.get(pixel_idx) {
                    a *= m as f32 / 255.0;
                    if a <= 0.0 {
                        continue;
                    }
                }
            }
            let r = w0 * c0.0 + w1 * c1.0 + w2 * c2.0;
            let g = w0 * c0.1 + w1 * c1.1 + w2 * c2.1;
            let b = w0 * c0.2 + w1 * c1.2 + w2 * c2.2;
            blend_premul(dest, pixel_idx * 4, r, g, b, a);
        }
    }
}

/// Source-over blend of a straight-alpha colour into a premultiplied RGBA8
/// destination pixel.
#[inline]
fn blend_premul(dest: &mut [u8], off: usize, r: f32, g: f32, b: f32, a: f32) {
    if off + 3 >= dest.len() {
        return;
    }
    let a = a.clamp(0.0, 1.0);
    let sr = r.clamp(0.0, 1.0) * a;
    let sg = g.clamp(0.0, 1.0) * a;
    let sb = b.clamp(0.0, 1.0) * a;
    let inv = 1.0 - a;
    let dr = dest[off] as f32 / 255.0;
    let dg = dest[off + 1] as f32 / 255.0;
    let db = dest[off + 2] as f32 / 255.0;
    let da = dest[off + 3] as f32 / 255.0;
    dest[off] = ((sr + dr * inv) * 255.0).round().clamp(0.0, 255.0) as u8;
    dest[off + 1] = ((sg + dg * inv) * 255.0).round().clamp(0.0, 255.0) as u8;
    dest[off + 2] = ((sb + db * inv) * 255.0).round().clamp(0.0, 255.0) as u8;
    dest[off + 3] = ((a + da * inv) * 255.0).round().clamp(0.0, 255.0) as u8;
}

// ===========================================================================
// PDF function evaluation (Types 0, 2, 3, 4) — used by `/Function`-driven
// mesh colours and by Type 1 function-based shadings.
// ===========================================================================

/// Evaluate a PDF function object against `inputs`, returning the output
/// components. Supports an array of 1-output functions plus function types
/// 0 (sampled), 2 (exponential), 3 (stitching) and 4 (PostScript). Returns
/// `None` for unsupported shapes so callers fall back to the raw inputs.
fn eval_pdf_function(
    func: &Object,
    doc: &PdfDocument,
    inputs: &[f32],
    sampled_bytes: Option<&[u8]>,
) -> Option<Vec<f32>> {
    if let Object::Array(arr) = func {
        // An array of n single-output functions, one per colour component.
        let mut out = Vec::with_capacity(arr.len());
        for f in arr {
            let resolved = doc.resolve_object(f).ok()?;
            // Each element is its own function; the caller's prepared stream
            // belongs to `func`, not to these.
            let mut r = eval_pdf_function(&resolved, doc, inputs, None)?;
            out.append(&mut r);
        }
        return Some(out);
    }

    let dict = func.as_dict()?;
    let ftype = dict.get("FunctionType").and_then(|o| o.as_integer())?;
    match ftype {
        2 => eval_type2(dict, inputs),
        3 => eval_type3(dict, doc, inputs),
        0 => match sampled_bytes {
            Some(bytes) => eval_type0(bytes, dict, inputs),
            // No prepared stream: a nested or array member, which this
            // optimisation does not cover. It still decodes per call.
            None => eval_type0(&func.decode_stream_data().ok()?, dict, inputs),
        },
        4 => eval_type4(func, dict, inputs),
        _ => None,
    }
}

/// Type 2 exponential interpolation: `f(x) = C0 + x^N (C1 - C0)`.
fn eval_type2(dict: &HashMap<String, Object>, inputs: &[f32]) -> Option<Vec<f32>> {
    let x = *inputs.first()?;
    let c0 = dict
        .get("C0")
        .and_then(|o| o.as_array())
        .map(|a| a.iter().map(num).collect::<Vec<_>>())
        .unwrap_or_else(|| vec![0.0]);
    let c1 = dict
        .get("C1")
        .and_then(|o| o.as_array())
        .map(|a| a.iter().map(num).collect::<Vec<_>>())
        .unwrap_or_else(|| vec![1.0]);
    let n = dict.get("N").map(num).unwrap_or(1.0);
    let xp = x.abs().powf(n) * x.signum();
    let len = c0.len().max(c1.len());
    Some(
        (0..len)
            .map(|i| {
                let a = c0.get(i).copied().unwrap_or(0.0);
                let b = c1.get(i).copied().unwrap_or(0.0);
                a + xp * (b - a)
            })
            .collect(),
    )
}

/// Type 3 stitching: select the sub-function whose sub-domain contains the
/// input, remap the input through `/Encode`, and evaluate it.
fn eval_type3(
    dict: &HashMap<String, Object>,
    doc: &PdfDocument,
    inputs: &[f32],
) -> Option<Vec<f32>> {
    let x = *inputs.first()?;
    let funcs = dict.get("Functions").and_then(|o| o.as_array())?;
    if funcs.is_empty() {
        return None;
    }
    let domain = dict.get("Domain").and_then(|o| o.as_array())?;
    let (d0, d1) = (num(domain.first()?), num(domain.get(1)?));
    let bounds: Vec<f32> = dict
        .get("Bounds")
        .and_then(|o| o.as_array())
        .map(|a| a.iter().map(num).collect())
        .unwrap_or_default();
    let encode: Vec<f32> = dict
        .get("Encode")
        .and_then(|o| o.as_array())
        .map(|a| a.iter().map(num).collect())
        .unwrap_or_default();

    let xc = x.clamp(d0.min(d1), d0.max(d1));
    // Find the sub-function index `k`.
    let mut k = 0usize;
    while k < bounds.len() && xc >= bounds[k] {
        k += 1;
    }
    k = k.min(funcs.len() - 1);
    // Sub-domain [lo, hi) for segment k.
    let lo = if k == 0 { d0 } else { bounds[k - 1] };
    let hi = if k < bounds.len() { bounds[k] } else { d1 };
    let (e0, e1) = (
        encode.get(2 * k).copied().unwrap_or(0.0),
        encode.get(2 * k + 1).copied().unwrap_or(1.0),
    );
    let xe = if (hi - lo).abs() < f32::EPSILON {
        e0
    } else {
        e0 + (xc - lo) * (e1 - e0) / (hi - lo)
    };
    let sub = doc.resolve_object(&funcs[k]).ok()?;
    eval_pdf_function(&sub, doc, &[xe], None)
}

/// Type 4 PostScript calculator function.
fn eval_type4(func: &Object, dict: &HashMap<String, Object>, inputs: &[f32]) -> Option<Vec<f32>> {
    let bytes = func.decode_stream_data().ok()?;
    let domain = pairs(dict.get("Domain"));
    let range = pairs(dict.get("Range"));
    let in64: Vec<f64> = inputs.iter().map(|&v| v as f64).collect();
    let out = crate::functions::evaluate_type4_clamped(&bytes, &in64, &domain, &range).ok()?;
    Some(out.into_iter().map(|v| v as f32).collect())
}

/// Type 0 sampled function with multilinear interpolation. Supports up to 2
/// input dimensions (enough for Type 1 shadings and 1-D `/Function` mesh
/// colours). Bounded sample reads; returns `None` for out-of-support
/// shapes.
fn eval_type0(bytes: &[u8], dict: &HashMap<String, Object>, inputs: &[f32]) -> Option<Vec<f32>> {
    let domain = pairs(dict.get("Domain"));
    let range = pairs(dict.get("Range"));
    let size: Vec<usize> = dict
        .get("Size")
        .and_then(|o| o.as_array())?
        .iter()
        .map(|o| o.as_integer().unwrap_or(0).max(0) as usize)
        .collect();
    let bps = dict.get("BitsPerSample").and_then(|o| o.as_integer())? as u32;
    let m = size.len();
    let n = range.len();
    if m == 0 || m > 2 || n == 0 || n > 8 || bps == 0 || bps > 32 {
        return None;
    }
    if size.contains(&0) || domain.len() < m {
        return None;
    }
    // `/Size` is document controlled. Bounding the declared grid by what the
    // sample stream holds keeps the stride and flat-index products below from
    // wrapping a bogus grid into a valid-looking offset.
    let declared_bits = size
        .iter()
        .try_fold(1usize, |acc, &s| acc.checked_mul(s))
        .and_then(|samples| samples.checked_mul(n))
        .and_then(|values| values.checked_mul(bps as usize));
    if declared_bits.is_none_or(|bits| bits > bytes.len() * 8) {
        log::warn!(
            "Skipping sampled function: /Size {:?} with {} outputs at {} bits declares \
             more samples than the {}-byte sample stream holds",
            size,
            n,
            bps,
            bytes.len()
        );
        return None;
    }
    let encode: Vec<f32> = dict
        .get("Encode")
        .and_then(|o| o.as_array())
        .map(|a| a.iter().map(num).collect())
        .unwrap_or_else(|| {
            // Default Encode: [0 (Size_i - 1)] per input dimension.
            size.iter()
                .flat_map(|&s| [0.0, (s.saturating_sub(1)) as f32])
                .collect()
        });
    let decode: Vec<(f32, f32)> = dict
        .get("Decode")
        .and_then(|o| o.as_array())
        .map(|a| {
            a.as_chunks::<2>()
                .0
                .iter()
                .map(|c| (num(&c[0]), num(&c[1])))
                .collect()
        })
        .unwrap_or_else(|| range.iter().map(|r| (r[0] as f32, r[1] as f32)).collect());

    // Encode each input to a continuous grid coordinate in [0, Size_i - 1].
    let mut e = [0.0f32; 2];
    for i in 0..m {
        let (d0, d1) = (domain[i][0] as f32, domain[i][1] as f32);
        let x = inputs
            .get(i)
            .copied()
            .unwrap_or(0.0)
            .clamp(d0.min(d1), d0.max(d1));
        let (en0, en1) = (
            encode.get(2 * i).copied().unwrap_or(0.0),
            encode
                .get(2 * i + 1)
                .copied()
                .unwrap_or((size[i] - 1) as f32),
        );
        let ec = if (d1 - d0).abs() < f32::EPSILON {
            en0
        } else {
            en0 + (x - d0) * (en1 - en0) / (d1 - d0)
        };
        e[i] = ec.clamp(0.0, (size[i] - 1) as f32);
    }

    let max_sample = if bps >= 32 {
        u32::MAX as f64
    } else {
        ((1u64 << bps) - 1) as f64
    };

    // Fetch one output-sample vector at integer grid coordinates.
    let sample = |coord: &[usize; 2]| -> Vec<f32> {
        let mut flat = 0usize;
        let mut stride = 1usize;
        for i in 0..m {
            flat += coord[i].min(size[i] - 1) * stride;
            stride *= size[i];
        }
        (0..n)
            .map(|o| {
                let bit_off = (flat * n + o) * bps as usize;
                let raw = read_bits_at(bytes, bit_off, bps).unwrap_or(0);
                (raw as f64 / max_sample) as f32
            })
            .collect()
    };

    // Multilinear interpolation over the 2^m surrounding grid corners.
    let corners = 1usize << m;
    let mut acc = vec![0.0f32; n];
    for c in 0..corners {
        let mut coord = [0usize; 2];
        let mut weight = 1.0f32;
        for i in 0..m {
            let base = e[i].floor() as usize;
            let frac = e[i] - base as f32;
            let hi = (c >> i) & 1;
            if hi == 1 {
                coord[i] = (base + 1).min(size[i] - 1);
                weight *= frac;
            } else {
                coord[i] = base;
                weight *= 1.0 - frac;
            }
        }
        if weight == 0.0 {
            continue;
        }
        let s = sample(&coord);
        for o in 0..n {
            acc[o] += weight * s[o];
        }
    }

    // Map normalised samples through /Decode.
    Some(
        acc.iter()
            .enumerate()
            .map(|(o, &v)| {
                let (lo, hi) = decode.get(o).copied().unwrap_or((0.0, 1.0));
                lo + v * (hi - lo)
            })
            .collect(),
    )
}

/// Read `nbits` (≤32) MSB-first from an arbitrary bit offset. Returns
/// `None` when the range exceeds the buffer.
fn read_bits_at(bytes: &[u8], bit_off: usize, nbits: u32) -> Option<u32> {
    let nbits = nbits as usize;
    if nbits == 0 {
        return Some(0);
    }
    if bit_off + nbits > bytes.len() * 8 {
        return None;
    }
    let mut value: u32 = 0;
    for i in 0..nbits {
        let pos = bit_off + i;
        let byte = bytes[pos >> 3];
        let bit = (byte >> (7 - (pos & 7))) & 1;
        value = (value << 1) | bit as u32;
    }
    Some(value)
}

// ===========================================================================
// Small numeric helpers.
// ===========================================================================

/// Numeric coercion for `Object` (Integer or Real → f32; else 0).
fn num(o: &Object) -> f32 {
    o.as_real()
        .map(|v| v as f32)
        .or_else(|| o.as_integer().map(|i| i as f32))
        .unwrap_or(0.0)
}

/// Flatten a `[min max min max ...]` array object into `[[min, max], ...]`.
fn pairs(o: Option<&Object>) -> Vec<[f64; 2]> {
    o.and_then(|o| o.as_array())
        .map(|a| {
            a.as_chunks::<2>()
                .0
                .iter()
                .map(|c| {
                    let lo = c[0]
                        .as_real()
                        .or_else(|| c[0].as_integer().map(|i| i as f64))
                        .unwrap_or(0.0);
                    let hi = c[1]
                        .as_real()
                        .or_else(|| c[1].as_integer().map(|i| i as f64))
                        .unwrap_or(0.0);
                    [lo, hi]
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_reader_reads_msb_first_and_bounds() {
        // 0b1011_0010, 0b1100_0000
        let data = [0b1011_0010u8, 0b1100_0000u8];
        let mut r = BitReader::new(&data);
        assert_eq!(r.read_bits(1), Some(1));
        assert_eq!(r.read_bits(3), Some(0b011));
        assert_eq!(r.read_bits(4), Some(0b0010));
        assert_eq!(r.read_bits(2), Some(0b11));
        // Two bits left after this (the trailing zeros); asking for more
        // than remain returns None.
        assert_eq!(r.read_bits(8), None);
    }

    #[test]
    fn decode_value_maps_endpoints() {
        // 8-bit: 0 → lo, 255 → hi, midpoint ≈ centre.
        assert!((decode_value(0, 8, -2.0, 2.0) - (-2.0)).abs() < 1e-6);
        assert!((decode_value(255, 8, -2.0, 2.0) - 2.0).abs() < 1e-6);
        assert!((decode_value(128, 8, 0.0, 1.0) - 0.5019608).abs() < 1e-4);
    }

    /// Build a minimal Type 4 vertex stream by hand and confirm the decoder
    /// reconstructs the two triangles (one fresh, one edge-shared).
    #[test]
    fn type4_stream_decodes_flags_and_triangles() {
        // 8 bits per flag/coord/component; 1 colour component.
        // Decode: x∈[0,1] y∈[0,1] c∈[0,1].
        let params = MeshParams {
            bits_per_flag: 8,
            bits_per_coord: 8,
            bits_per_comp: 8,
            vertices_per_row: 0,
            decode: vec![(0.0, 1.0), (0.0, 1.0), (0.0, 1.0)],
        };
        // Helper to push a vertex: flag, x, y, c (all bytes).
        let mut bytes: Vec<u8> = Vec::new();
        let mut push_v = |flag: u8, x: u8, y: u8, c: u8| {
            bytes.extend_from_slice(&[flag, x, y, c]);
        };
        // First triangle: three flag-0 vertices.
        push_v(0, 0, 0, 0); // (0,0) colour 0
        push_v(0, 255, 0, 255); // (1,0) colour 1
        push_v(0, 0, 255, 128); // (0,1) colour ~0.5
                                // Second triangle: flag 1 reuses edge (vb, vc) of the first.
        push_v(1, 255, 255, 255); // (1,1) colour 1

        let tris = decode_type4_stream(&bytes, &params, 100);
        assert_eq!(tris.len(), 2, "expected two triangles");
        // First triangle corners.
        assert!((tris[0][0].x - 0.0).abs() < 1e-4 && (tris[0][0].y - 0.0).abs() < 1e-4);
        assert!((tris[0][1].x - 1.0).abs() < 1e-4);
        assert!((tris[0][2].y - 1.0).abs() < 1e-4);
        // Shared triangle: first two vertices are the previous vb, vc.
        assert!((tris[1][0].x - 1.0).abs() < 1e-4); // prev vb = (1,0)
        assert!((tris[1][1].y - 1.0).abs() < 1e-4); // prev vc = (0,1)
        assert!((tris[1][2].x - 1.0).abs() < 1e-4 && (tris[1][2].y - 1.0).abs() < 1e-4);
    }

    /// A Type 5 lattice of 2 rows × 3 columns tessellates into 4 triangles.
    #[test]
    fn type5_lattice_tessellates_rows() {
        let params = MeshParams {
            bits_per_flag: 8,
            bits_per_coord: 8,
            bits_per_comp: 8,
            vertices_per_row: 3,
            decode: vec![(0.0, 1.0), (0.0, 1.0), (0.0, 1.0)],
        };
        let mut bytes: Vec<u8> = Vec::new();
        // Row 0 (y=0): three vertices at x=0,0.5,1.
        for &x in &[0u8, 128, 255] {
            bytes.extend_from_slice(&[x, 0, 0]); // x, y=0, colour
        }
        // Row 1 (y=1): three vertices.
        for &x in &[0u8, 128, 255] {
            bytes.extend_from_slice(&[x, 255, 255]);
        }
        let tris = decode_type5_stream(&bytes, &params, 100);
        // (vpr-1) cells × 2 triangles = 4.
        assert_eq!(tris.len(), 4);
    }

    /// Barycentric colour interpolation: a triangle with red/green/blue
    /// corners yields the exact vertex colour at each corner and a mixed
    /// colour near the centroid.
    #[test]
    fn gouraud_triangle_interpolates_colours() {
        let mut pixmap = Pixmap::new(10, 10).unwrap();
        let v0 = ((1.0, 1.0), (1.0, 0.0, 0.0, 1.0)); // red
        let v1 = ((8.0, 1.0), (0.0, 1.0, 0.0, 1.0)); // green
        let v2 = ((1.0, 8.0), (0.0, 0.0, 1.0, 1.0)); // blue
        fill_gouraud_triangle(&mut pixmap, None, v0, v1, v2);
        let data = pixmap.data();
        let px = |x: usize, y: usize| {
            let o = (y * 10 + x) * 4;
            (data[o], data[o + 1], data[o + 2], data[o + 3])
        };
        // Near v0 corner → mostly red.
        let (r, g, b, a) = px(1, 1);
        assert!(a > 0, "corner must be painted");
        assert!(r > g && r > b, "near red corner should be reddish: {r},{g},{b}");
    }

    /// A degenerate (collinear) triangle must paint nothing and not panic.
    #[test]
    fn degenerate_triangle_is_skipped() {
        let mut pixmap = Pixmap::new(8, 8).unwrap();
        let c = (1.0, 1.0, 1.0, 1.0);
        fill_gouraud_triangle(&mut pixmap, None, ((0.0, 0.0), c), ((4.0, 4.0), c), ((8.0, 8.0), c));
        assert!(pixmap.data().iter().all(|&b| b == 0), "no pixels painted");
    }

    /// End-to-end: decode a hand-built Type 4 stream and rasterise it
    /// through the full triangle path into a pixmap, asserting the shaded
    /// region actually receives non-background pixels.
    #[test]
    fn type4_renders_non_background_pixels() {
        let params = MeshParams {
            bits_per_flag: 8,
            bits_per_coord: 8,
            bits_per_comp: 8,
            vertices_per_row: 0,
            // Coordinates decode into the 0..40 device range directly.
            decode: vec![(0.0, 40.0), (0.0, 40.0), (0.0, 1.0)],
        };
        // One big triangle covering a chunk of a 50×50 canvas, solid red.
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&[0, 0, 0, 255]); // flag0 (0,0) c=1
        bytes.extend_from_slice(&[0, 255, 0, 255]); // flag0 (40,0) c=1
        bytes.extend_from_slice(&[0, 128, 255, 255]); // flag0 (20,40) c=1

        let tris = decode_type4_stream(&bytes, &params, 100);
        assert_eq!(tris.len(), 1);

        let mut pixmap = Pixmap::new(50, 50).unwrap();
        // Resolver maps a single component to a red-scaled colour.
        let to_rgba = |c: &[f32]| -> (f32, f32, f32, f32) {
            let v = c.first().copied().unwrap_or(0.0);
            (v, 0.0, 0.0, 1.0)
        };
        rasterize_raw_triangles(&mut pixmap, &tris, Transform::identity(), None, &to_rgba);

        // The triangle centroid (~20,13) must be painted red.
        let data = pixmap.data();
        let o = (13 * 50 + 20) * 4;
        assert!(data[o] > 100, "shaded region should be red, got r={}", data[o]);
        assert!(data[o + 3] > 0, "shaded region should be opaque");
        // A corner well outside the triangle stays background (transparent).
        let corner = (48 * 50 + 48) * 4;
        assert_eq!(data[corner + 3], 0, "outside triangle stays background");
    }

    /// Minimal PDF document used purely as a `&PdfDocument` argument.
    /// `eval_pdf_function` touches it only to resolve `Reference` objects;
    /// the fixtures below are inline, so any successfully-parsed PDF works.
    fn fixture_doc() -> PdfDocument {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"%PDF-1.4\n");
        let cat_off = buf.len();
        buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        let pages_off = buf.len();
        buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\n");
        let xref_off = buf.len();
        buf.extend_from_slice(b"xref\n0 3\n0000000000 65535 f \n");
        buf.extend_from_slice(format!("{:010} 00000 n \n", cat_off).as_bytes());
        buf.extend_from_slice(format!("{:010} 00000 n \n", pages_off).as_bytes());
        buf.extend_from_slice(
            format!("trailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off)
                .as_bytes(),
        );
        PdfDocument::from_bytes(buf).expect("fixture PDF parses")
    }

    /// Sampled (Type 0) function stream: 2 inputs, 2×2 grid, 8-bit RGB
    /// samples — red, green, blue, yellow at the four grid corners.
    fn sampled_rgb_function() -> Object {
        let mut dict = HashMap::new();
        dict.insert("FunctionType".to_string(), Object::Integer(0));
        dict.insert(
            "Domain".to_string(),
            Object::Array(vec![
                Object::Real(0.0),
                Object::Real(1.0),
                Object::Real(0.0),
                Object::Real(1.0),
            ]),
        );
        dict.insert(
            "Range".to_string(),
            Object::Array(vec![
                Object::Real(0.0),
                Object::Real(1.0),
                Object::Real(0.0),
                Object::Real(1.0),
                Object::Real(0.0),
                Object::Real(1.0),
            ]),
        );
        dict.insert(
            "Size".to_string(),
            Object::Array(vec![Object::Integer(2), Object::Integer(2)]),
        );
        dict.insert("BitsPerSample".to_string(), Object::Integer(8));
        // Sample order: (0,0) (1,0) (0,1) (1,1).
        let samples: [u8; 12] = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0];
        Object::Stream {
            dict,
            data: crate::import_limits::StreamData::copy_from_slice(&samples),
        }
    }

    /// The prepared-bytes fast path and the per-call decode fallback must
    /// agree on every input, and both must yield the grid's interpolated
    /// values — a wrong or empty prepared buffer reads as all-zero samples
    /// (`read_bits_at` → `unwrap_or(0)`) and fails the expected values.
    #[test]
    fn type0_prepared_bytes_matches_per_call_decode() {
        let doc = fixture_doc();
        let func = sampled_rgb_function();
        let prepared = func.decode_stream_data().expect("stream decodes");

        let cases: &[([f32; 2], [f32; 3])] = &[
            ([0.0, 0.0], [1.0, 0.0, 0.0]),
            ([1.0, 0.0], [0.0, 1.0, 0.0]),
            ([0.0, 1.0], [0.0, 0.0, 1.0]),
            ([1.0, 1.0], [1.0, 1.0, 0.0]),
            ([0.5, 0.5], [0.5, 0.5, 0.25]),
            ([0.25, 0.75], [0.375, 0.25, 0.5625]),
        ];
        for (inputs, expected) in cases {
            let fast = eval_pdf_function(&func, &doc, inputs, Some(&prepared))
                .expect("fast path evaluates");
            let slow =
                eval_pdf_function(&func, &doc, inputs, None).expect("fallback path evaluates");
            assert_eq!(fast, slow, "paths diverge at {inputs:?}");
            assert_eq!(fast.len(), 3);
            for (o, (&got, &want)) in fast.iter().zip(expected).enumerate() {
                assert!(
                    (got - want).abs() < 1e-4,
                    "output {o} at {inputs:?}: got {got}, want {want}"
                );
            }
        }
    }

    /// End-to-end: a Type 1 shading whose `/Function` is a sampled stream
    /// goes through the hoisted-bytes path in `render_mesh_shading`; every
    /// probed pixel must match a per-point fallback evaluation of the same
    /// function.
    #[test]
    fn type1_shading_renders_sampled_function_via_prepared_bytes() {
        let doc = fixture_doc();
        let func = sampled_rgb_function();

        let mut shading = HashMap::new();
        shading.insert("ShadingType".to_string(), Object::Integer(1));
        shading.insert(
            "Domain".to_string(),
            Object::Array(vec![
                Object::Real(0.0),
                Object::Real(1.0),
                Object::Real(0.0),
                Object::Real(1.0),
            ]),
        );
        shading.insert(
            "Matrix".to_string(),
            Object::Array(vec![
                Object::Real(16.0),
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(16.0),
                Object::Real(0.0),
                Object::Real(0.0),
            ]),
        );
        shading.insert("Function".to_string(), func.clone());

        let mut pixmap = Pixmap::new(16, 16).unwrap();
        let resolve = |c: &[f32]| -> Option<(f32, f32, f32, f32)> {
            Some((
                c.first().copied().unwrap_or(0.0),
                c.get(1).copied().unwrap_or(0.0),
                c.get(2).copied().unwrap_or(0.0),
                1.0,
            ))
        };
        render_mesh_shading(
            &mut pixmap,
            &shading,
            &Object::Dictionary(shading.clone()),
            1,
            Transform::identity(),
            &doc,
            None,
            &resolve,
        )
        .unwrap();

        let data = pixmap.data();
        for &(px, py) in &[(4usize, 4usize), (12, 4), (4, 12), (12, 12), (8, 8)] {
            let (u, v) = ((px as f32 + 0.5) / 16.0, (py as f32 + 0.5) / 16.0);
            let want =
                eval_pdf_function(&func, &doc, &[u, v], None).expect("fallback path evaluates");
            let o = (py * 16 + px) * 4;
            assert_eq!(data[o + 3], 255, "pixel ({px},{py}) must be opaque");
            for (ch, &w) in want.iter().enumerate().take(3) {
                let got = data[o + ch] as f32 / 255.0;
                assert!(
                    (got - w).abs() < 8.0 / 255.0,
                    "pixel ({px},{py}) channel {ch}: got {got}, want {w}"
                );
            }
        }
    }

    /// Type 2 exponential function evaluates the endpoint interpolation.
    #[test]
    fn type2_function_interpolates() {
        let mut dict = HashMap::new();
        dict.insert("FunctionType".to_string(), Object::Integer(2));
        dict.insert(
            "C0".to_string(),
            Object::Array(vec![Object::Real(0.0), Object::Real(0.0), Object::Real(0.0)]),
        );
        dict.insert(
            "C1".to_string(),
            Object::Array(vec![Object::Real(1.0), Object::Real(0.5), Object::Real(0.0)]),
        );
        dict.insert("N".to_string(), Object::Integer(1));
        let out = eval_type2(&dict, &[0.5]).unwrap();
        assert!((out[0] - 0.5).abs() < 1e-6);
        assert!((out[1] - 0.25).abs() < 1e-6);
        assert!((out[2] - 0.0).abs() < 1e-6);
    }
}

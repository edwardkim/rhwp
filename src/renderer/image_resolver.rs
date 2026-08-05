use std::cell::RefCell;
use std::io::Cursor;

use crate::model::image::ImageEffect;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::image_header::{
    canvaskit_encoded_image_header, CANVASKIT_MAX_IMAGE_DIMENSION, CANVASKIT_MAX_IMAGE_PIXELS,
};
use crate::renderer::render_tree::{
    ImageNode, REAL_PICTURE_WATERMARK_BRIGHTNESS, REAL_PICTURE_WATERMARK_CHROMA_GAIN,
    REAL_PICTURE_WATERMARK_CONTRAST, REAL_PICTURE_WATERMARK_CORRECTION_BIAS,
    REAL_PICTURE_WATERMARK_CORRECTION_MATRIX, REAL_PICTURE_WATERMARK_FILL_CHROMA_GAIN,
    REAL_PICTURE_WATERMARK_FILL_WHITE_BLEND, REAL_PICTURE_WATERMARK_SATURATION,
    REAL_PICTURE_WATERMARK_WHITE_BLEND,
};

// ── 변환 결과 메모 ──
//
// 편집 한 번에 레이어 트리가 여러 벌 만들어지고(본문 캔버스 / overlay / JSON),
// 그때마다 같은 그림이 다시 변환된다. JPEG 은 회색인지 알아내려고 **전체를 디코드**
// 하므로, 2MB 사진 한 장이 키 입력마다 수백 ms 를 먹었다 (#2520).
//
// 변환은 입력 바이트만으로 결과가 정해지는 순수 함수다. 그래서 내용 지문을 키로 쓰면
// 문서 쪽에서 무효화를 알려 줄 필요가 없다 — 바이트가 바뀌면 키가 바뀐다.
//
// 세 경로(paint/builder.rs, paint/json.rs, renderer/skia/image_conv.rs)와 svg·web_canvas·
// emf 경로는 모두 `&[u8]` 만 넘긴다. `bin_data_id` 로 키를 잡으려면 그 전부에 신원을
// 실어 날라야 하고, EMF 안에 박힌 BMP 처럼 애초에 BinData 가 아닌 그림도 있다.

/// 메모 상한(byte). 회색 JPEG 은 PNG 로 재인코딩한 결과를 들고 있어야 하므로 바이트로
/// 제한한다.
const MAX_MEMO_BYTES: usize = 16 * 1024 * 1024;

/// 항목 수 상한. 변환하지 않는 색 사진은 결과가 `None` 이라 바이트를 전혀 차지하지 않아
/// 바이트 예산만으로는 영영 밀려나지 않는다. 조회가 선형 탐색이라 항목 수도 묶는다.
const MAX_MEMO_ENTRIES: usize = 64;

/// 변환 종류. 같은 바이트라도 어떤 변환을 거쳤느냐에 따라 결과가 다르다.
#[derive(Clone, Copy)]
enum Conversion {
    Bmp,
    Pcx,
    Tiff,
    GrayscaleJpeg,
    WatermarkJpeg,
    RealPictureTone,
    RealPictureFillTone,
}

#[derive(Default)]
struct ConversionMemo {
    /// (키, 결과) — 접근 순서대로, 최근 것이 뒤.
    entries: Vec<(u64, Option<Vec<u8>>)>,
    /// 지금 들고 있는 결과 바이트 합.
    bytes: usize,
}

impl ConversionMemo {
    fn get(&mut self, key: u64) -> Option<Option<Vec<u8>>> {
        let idx = self.entries.iter().position(|(k, _)| *k == key)?;
        let entry = self.entries.remove(idx);
        let hit = entry.1.clone();
        self.entries.push(entry);
        Some(hit)
    }

    fn insert(&mut self, key: u64, value: Option<Vec<u8>>) {
        let size = value.as_ref().map_or(0, Vec::len);
        if size > MAX_MEMO_BYTES {
            return;
        }
        while self.bytes + size > MAX_MEMO_BYTES || self.entries.len() >= MAX_MEMO_ENTRIES {
            let (_, evicted) = self.entries.remove(0);
            self.bytes -= evicted.as_ref().map_or(0, Vec::len);
        }
        self.bytes += size;
        self.entries.push((key, value));
    }
}

thread_local! {
    /// WASM 은 단일 스레드라 `thread_local` + `RefCell` 로 충분하다
    /// (`layout::text_measurement` 의 측정 캐시와 같은 방식).
    static CONVERSION_MEMO: RefCell<ConversionMemo> = RefCell::new(ConversionMemo::default());
}

// 실제로 변환을 수행한 횟수 — 메모가 듣는지 보는 테스트용.
#[cfg(test)]
thread_local! {
    static CONVERSIONS_RUN: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn memoized(
    conversion: Conversion,
    data: &[u8],
    convert: impl FnOnce() -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let key = conversion_key(conversion, data);
    if let Some(hit) = CONVERSION_MEMO.with(|memo| memo.borrow_mut().get(key)) {
        return hit;
    }

    let converted = convert();
    #[cfg(test)]
    CONVERSIONS_RUN.with(|runs| runs.set(runs.get() + 1));
    CONVERSION_MEMO.with(|memo| memo.borrow_mut().insert(key, converted.clone()));
    converted
}

/// 내용 지문 — 바이트 전체를 해싱한다.
///
/// 앞뒤 일부만 뽑는 표본 키는 쓰지 않는다. 무압축 BMP 는 같은 치수면 길이가 정확히
/// 같고 고정 헤더 뒤에 원시 픽셀이 이어지므로, 위아래 여백이 균일한 두 그림이 길이·앞·뒤
/// 표본까지 전부 같아진다 — 가운데만 다른 그림이 남의 변환 결과를 받는다.
///
/// 해싱은 바이트 수에 비례하지만 상수가 작다. 3.7MB 기준 1.35ms(릴리스)로, 이 메모가
/// 없앤 285ms 짜리 JPEG 전체 디코드에 비하면 무시할 만하다. `blake3`(2.90ms)도 재 봤지만
/// 이 키는 세션 안에서만 쓰고 밖으로 나가지 않으므로 싼 쪽을 쓴다.
fn conversion_key(conversion: Conversion, data: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    (conversion as u8).hash(&mut hasher);
    data.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn resolve_image_payload(image: &ImageNode) -> Option<ResolvedImagePayload> {
    let data = image.data.as_deref()?;
    let mime = detect_image_mime_type(data);

    match mime {
        "image/bmp" => bmp_bytes_to_png_bytes(data).map(|data| ResolvedImagePayload {
            data,
            mime: "image/png",
            kind: ResolvedImageKind::FormatConverted,
            suppress_effects: false,
        }),
        "image/x-pcx" => pcx_bytes_to_png_bytes(data).map(|data| ResolvedImagePayload {
            data,
            mime: "image/png",
            kind: ResolvedImageKind::FormatConverted,
            suppress_effects: false,
        }),
        "image/tiff" => tiff_bytes_to_png_bytes(data).map(|data| ResolvedImagePayload {
            data,
            mime: "image/png",
            kind: ResolvedImageKind::FormatConverted,
            suppress_effects: false,
        }),
        // 브라우저는 WMF 를 디코드하지 못한다 — `<img>` 로 내보내면 naturalWidth=0 인 깨진
        // 그림이 된다. `svg.rs`·`web_canvas.rs` 는 각자 내보내기 직전에 변환하지만, DOM
        // `<img>` 경로(`getSourceImageBytes`·layer tree base64)는 여기를 지나므로 변환이
        // 여기에도 있어야 한다.
        "image/x-wmf" => {
            crate::renderer::svg::convert_wmf_to_svg(data).map(|data| ResolvedImagePayload {
                data,
                mime: "image/svg+xml",
                kind: ResolvedImageKind::FormatConverted,
                suppress_effects: false,
            })
        }
        // EMF 도 WMF 와 같은 처지다 — 브라우저가 디코드하지 못하고, 변환기
        // (`emf::convert_to_standalone_svg`)는 있는데 OLE 프리뷰 경로에서만 쓰이고
        // 있었다. 10k 스윕에서 직접 삽입 EMF 그림 16문서가 octet-stream 으로 새어
        // 나가는 것이 확인됐다.
        "image/x-emf" => {
            crate::emf::convert_to_standalone_svg(data).map(|data| ResolvedImagePayload {
                data,
                mime: "image/svg+xml",
                kind: ResolvedImageKind::FormatConverted,
                suppress_effects: false,
            })
        }
        "application/postscript" => {
            dos_eps_preview_bytes(data).map(|(mime, data)| ResolvedImagePayload {
                data,
                mime,
                kind: ResolvedImageKind::FormatConverted,
                suppress_effects: false,
            })
        }
        "image/jpeg" if is_watermark_image(image) => {
            watermark_jpeg_bytes_to_hancom_baked_png_bytes(data).map(|data| ResolvedImagePayload {
                data,
                mime: "image/png",
                kind: ResolvedImageKind::BakedWatermark,
                suppress_effects: true,
            })
        }
        "image/jpeg" => grayscale_jpeg_bytes_to_png_bytes(data).map(|data| ResolvedImagePayload {
            data,
            mime: "image/png",
            kind: ResolvedImageKind::FormatConverted,
            suppress_effects: false,
        }),
        _ => None,
    }
}

/// 그림 op 이 실제로 내보내는 바이트와 mime 을 결정한다 (Task #3315).
///
/// `resolve_image_payload` 는 변환에 **성공한** 경우만 payload 를 주고, 실패하면 `None` 이라
/// 호출부가 원본 바이트로 되돌아간다. 그래서 "JSON 에 실린 바이트"는 두 분기의 합이었고,
/// `paint/json.rs` 가 그 되돌림 사슬을 사본으로 들고 있었다. base64 를 생략한 뒤 키로 같은
/// 바이트를 되돌려주려면 **최종 결과가 한 곳에서만 정해져야** 한다 — 두 곳에 두면 갈라진다.
///
/// `bakes_watermark` 는 `paint::source_image_key` 의 variant 판정과 같은 값이다. 키가 이미
/// variant 를 담고 있으므로 키로 조회할 때는 `ImageNode` 없이도 같은 바이트를 재현한다.
/// 워터마크 bake 가 실패하면 회색 JPEG 경로로 내려가는데, 이는 `resolved == None` 일 때
/// json 쪽 되돌림이 하던 것과 같은 순서다.
pub(crate) fn emitted_image_bytes(
    data: &[u8],
    bakes_watermark: bool,
) -> (&'static str, std::borrow::Cow<'_, [u8]>) {
    let mime = detect_image_mime_type(data);
    // 변환 결과 mime 을 바이트와 함께 나른다 — WMF 는 PNG 가 아니라 SVG 로 나가므로
    // "변환에 성공하면 PNG" 로 접으면 mime 이 바이트와 어긋난다.
    let converted: Option<(&'static str, Vec<u8>)> = match mime {
        "image/bmp" => bmp_bytes_to_png_bytes(data).map(|png| ("image/png", png)),
        "image/x-pcx" => pcx_bytes_to_png_bytes(data).map(|png| ("image/png", png)),
        "image/tiff" => tiff_bytes_to_png_bytes(data).map(|png| ("image/png", png)),
        // `resolve_image_payload` 와 같은 분기 — 두 함수가 갈라지면 mime 만 아는 소비자
        // (`emitted_image_mime`)와 바이트를 받는 소비자가 다른 그림을 보게 된다.
        "image/x-wmf" => {
            crate::renderer::svg::convert_wmf_to_svg(data).map(|svg| ("image/svg+xml", svg))
        }
        "image/x-emf" => {
            crate::emf::convert_to_standalone_svg(data).map(|svg| ("image/svg+xml", svg))
        }
        "application/postscript" => dos_eps_preview_bytes(data),
        "image/jpeg" if bakes_watermark => watermark_jpeg_bytes_to_hancom_baked_png_bytes(data)
            .or_else(|| grayscale_jpeg_bytes_to_png_bytes(data))
            .map(|png| ("image/png", png)),
        "image/jpeg" => grayscale_jpeg_bytes_to_png_bytes(data).map(|png| ("image/png", png)),
        _ => None,
    };
    match converted {
        Some((converted_mime, bytes)) => (converted_mime, std::borrow::Cow::Owned(bytes)),
        None => (mime, std::borrow::Cow::Borrowed(data)),
    }
}

/// 그림 op 이 내보내는 mime 만 (Task #3315).
///
/// 바이트가 필요 없는 소비자 — 배치 정보만 주는 좁은 질의 — 를 위한 것이다. `resolved` 가
/// 붙어 있으면 그 mime 이 최종값이므로 변환을 다시 돌지 않는다. 큰 JPEG 은 메모 키 해싱만으로
/// 3.7MB 당 1.35 ms 라, 매 편집에 도는 경로에서는 이 절약이 그대로 이득이다.
///
/// `resolved` 가 없을 때 "그러면 변환이 실패했다는 뜻이니 원본 mime 이다" 로 단축하지 않고
/// `emitted_image_bytes` 에 위임한다. 그 전제는 `paint/builder.rs` 가 트리를 만들 때만
/// 성립하고, 직접 조립한 `PaintOp::Image { resolved: None }` 에는 성립하지 않는다 — 구조가
/// 보장하지 않는 불변식에 기대면 조용히 갈라진다.
pub(crate) fn emitted_image_mime(
    data: &[u8],
    resolved: Option<&ResolvedImagePayload>,
    bakes_watermark: bool,
) -> &'static str {
    match resolved {
        Some(payload) => payload.mime,
        None => emitted_image_bytes(data, bakes_watermark).0,
    }
}

pub(crate) fn image_node_with_resolved_payload(
    image: &ImageNode,
    resolved: Option<&ResolvedImagePayload>,
) -> ImageNode {
    let mut image = image.clone();
    if let Some(payload) = resolved {
        image.data = Some(payload.data.clone());
        if payload.suppress_effects {
            image.effect = ImageEffect::RealPic;
            image.brightness = 0;
            image.contrast = 0;
        }
    }
    image
}

/// 워터마크 bake 대상 판정. `paint::source_image_key` 의 variant 결정과 같은 술어를 써야
/// 키가 가리키는 바이트와 실제로 내보내는 바이트가 어긋나지 않는다 (Task #3315).
///
/// mime 검사는 포함하지 않는다 — 호출부가 JPEG 분기 안에서 쓴다.
pub(crate) fn is_watermark_image(image: &ImageNode) -> bool {
    !matches!(image.effect, ImageEffect::RealPic) && (image.brightness != 0 || image.contrast != 0)
}

/// BMP 바이트를 PNG 바이트로 재인코딩한다. 실패 시 None 반환.
///
/// 브라우저는 SVG `<image>` 내부의 `data:image/bmp` URI를 표준 지원하지 않으므로,
/// SVG 임베딩 전에 PNG로 변환해 호환성을 확보한다.
pub(crate) fn bmp_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    memoized(Conversion::Bmp, data, || {
        use image::ImageFormat;

        if let Some(img) = decode_image_with_format_limited(data, ImageFormat::Bmp) {
            let mut out = Vec::new();
            img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
                .ok()?;
            return Some(out);
        }
        oversized_bmp_to_downscaled_png_bytes(data)
    })
}

/// CanvasKit 디코드 한도(8192px·32Mpix)를 넘는 초대형 BMP 폴백 (#4064).
///
/// 실문서의 A4 전면 스캔 BMP(34~61Mpix)가 한도 거부로 원본 그대로 방출되면
/// SVG `<image>` 는 data URI BMP 를 표준 지원하지 않아 빈 그림이 된다.
/// 한도 안으로 다운스케일해 PNG 로 낸다 — 상한(경성 한도)을 넘거나 헤더가
/// 깨진 바이트는 여기서도 None 이다.
fn oversized_bmp_to_downscaled_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    use image::ImageFormat;

    // 시추 깊이: 한도의 8배(≈65k px · 256Mpix). 그 위는 손상 헤더로 간주한다
    // (실측: 깨진 BMP 헤더가 w=16,318,939 같은 값을 담아 온다).
    const HARD_MAX_DIMENSION: u32 = CANVASKIT_MAX_IMAGE_DIMENSION * 8;
    const HARD_MAX_PIXELS: u64 = CANVASKIT_MAX_IMAGE_PIXELS * 8;

    let header = canvaskit_encoded_image_header(data)?;
    if header.format != crate::renderer::image_header::CanvasKitEncodedImageFormat::Bmp {
        return None;
    }
    let (w, h) = (header.width, header.height);
    let pixels = u64::from(w).checked_mul(u64::from(h))?;
    if w == 0 || h == 0 || w > HARD_MAX_DIMENSION || h > HARD_MAX_DIMENSION {
        return None;
    }
    if pixels > HARD_MAX_PIXELS {
        return None;
    }

    let mut reader = image::ImageReader::with_format(Cursor::new(data), ImageFormat::Bmp);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(HARD_MAX_DIMENSION);
    limits.max_image_height = Some(HARD_MAX_DIMENSION);
    limits.max_alloc = Some(HARD_MAX_PIXELS.saturating_mul(4));
    reader.limits(limits);
    let img = reader.decode().ok()?;

    let ratio = f64::min(
        f64::from(CANVASKIT_MAX_IMAGE_DIMENSION) / f64::from(w.max(h)),
        (CANVASKIT_MAX_IMAGE_PIXELS as f64 / pixels as f64).sqrt(),
    )
    .min(1.0);
    let tw = ((f64::from(w) * ratio) as u32).max(1);
    let th = ((f64::from(h) * ratio) as u32).max(1);
    let img = img.thumbnail(tw, th);

    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// DOS EPS 바이너리(preamble `C5 D0 D3 C6`)의 내장 프리뷰를 꺼내 변환한다 (#4062).
///
/// 텍스트 PostScript 는 자체 인터프리터가 없어 변환 불가지만, DOS EPS 헤더는
/// WMF/TIFF 프리뷰의 오프셋·길이를 담는다 (Adobe EPSF 3.0 §5.2). 프리뷰가
/// 있으면 기존 변환기(WMF→SVG, TIFF→PNG)로 잇는다. 프리뷰가 없거나 손상이면
/// None — 호출부가 원본으로 되돌아간다.
pub(crate) fn dos_eps_preview_bytes(data: &[u8]) -> Option<(&'static str, Vec<u8>)> {
    if data.len() < 30 || !data.starts_with(&[0xC5, 0xD0, 0xD3, 0xC6]) {
        return None;
    }
    let le_u32 = |off: usize| -> Option<usize> {
        let b: [u8; 4] = data.get(off..off + 4)?.try_into().ok()?;
        Some(u32::from_le_bytes(b) as usize)
    };
    let section = |off_at: usize, len_at: usize| -> Option<&[u8]> {
        let off = le_u32(off_at)?;
        let len = le_u32(len_at)?;
        if len == 0 {
            return None;
        }
        data.get(off..off.checked_add(len)?)
    };
    // 헤더 배치: [4..8) PS off, [8..12) PS len, [12..16) WMF off, [16..20) WMF len,
    // [20..24) TIFF off, [24..28) TIFF len.
    if let Some(wmf) = section(12, 16) {
        if let Some(svg) = crate::renderer::svg::convert_wmf_to_svg(wmf) {
            return Some(("image/svg+xml", svg));
        }
    }
    if let Some(tiff) = section(20, 24) {
        if let Some(png) = tiff_bytes_to_png_bytes(tiff) {
            return Some(("image/png", png));
        }
    }
    None
}

/// TIFF 바이트를 PNG 바이트로 재인코딩한다. 실패 시 None 반환.
///
/// 브라우저와 rsvg는 SVG `<image>` 내부의 `data:image/tiff` URI를 안정적으로
/// 렌더링하지 못하므로, SVG/Canvas/HTML 임베딩 전에 PNG로 변환한다.
pub(crate) fn tiff_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    memoized(Conversion::Tiff, data, || {
        use image::ImageFormat;

        if let Some(img) = decode_image_with_format_limited(data, ImageFormat::Tiff) {
            let mut out = Vec::new();
            img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
                .ok()?;
            return Some(out);
        }
        uncompressed_palette_tiff_to_png_bytes(data)
    })
}

/// image 크레이트의 tiff 디코더가 지원하지 않는 8-bit 팔레트
/// (PhotometricInterpretation=3) TIFF 폴백 (#4064).
///
/// 10k 스윕에서 걸린 실문서 스캔본이 전부 이 형태(비압축·8bps·단일 plane)라
/// 그 범위만 좁게 디코드한다 — 압축·다중 plane 팔레트는 여기서도 None 이다.
fn uncompressed_palette_tiff_to_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    use image::{ImageFormat, RgbaImage};

    let le = match data.get(..4)? {
        [0x49, 0x49, 0x2A, 0x00] => true,
        [0x4D, 0x4D, 0x00, 0x2A] => false,
        _ => return None,
    };
    let u16_at = |off: usize| -> Option<u16> {
        let b: [u8; 2] = data.get(off..off + 2)?.try_into().ok()?;
        Some(if le {
            u16::from_le_bytes(b)
        } else {
            u16::from_be_bytes(b)
        })
    };
    let u32_at = |off: usize| -> Option<u32> {
        let b: [u8; 4] = data.get(off..off + 4)?.try_into().ok()?;
        Some(if le {
            u32::from_le_bytes(b)
        } else {
            u32::from_be_bytes(b)
        })
    };

    let ifd = u32_at(4)? as usize;
    let entry_count = u16_at(ifd)? as usize;
    // (tag, type, count, value-field offset). SHORT/LONG 단일값은 값 필드에
    // 좌측 정렬로 인라인, 배열은 값 필드가 데이터 오프셋이다 (TIFF 6.0 §2).
    let mut width = 0u32;
    let mut height = 0u32;
    let mut bits_per_sample = 0u32;
    let mut compression = 0u32;
    let mut photometric = u32::MAX;
    let mut strip_offsets: Vec<u32> = Vec::new();
    let mut strip_byte_counts: Vec<u32> = Vec::new();
    let mut color_map_off = 0usize;
    let mut color_map_len = 0usize;
    for i in 0..entry_count {
        let e = ifd + 2 + i * 12;
        let tag = u16_at(e)?;
        let typ = u16_at(e + 2)?;
        let count = u32_at(e + 4)? as usize;
        let inline_scalar = || -> Option<u32> {
            match typ {
                3 => u16_at(e + 8).map(u32::from),
                4 => u32_at(e + 8),
                _ => None,
            }
        };
        let array = |out: &mut Vec<u32>| -> Option<()> {
            let elem = match typ {
                3 => 2usize,
                4 => 4usize,
                _ => return None,
            };
            let base = if count * elem <= 4 {
                e + 8
            } else {
                u32_at(e + 8)? as usize
            };
            for j in 0..count {
                out.push(match typ {
                    3 => u32::from(u16_at(base + j * elem)?),
                    _ => u32_at(base + j * elem)?,
                });
            }
            Some(())
        };
        match tag {
            256 => width = inline_scalar()?,
            257 => height = inline_scalar()?,
            258 if count == 1 => bits_per_sample = inline_scalar()?,
            258 => return None, // 다중 plane 팔레트는 관측 밖
            259 => compression = inline_scalar()?,
            262 => photometric = inline_scalar()?,
            273 => array(&mut strip_offsets)?,
            279 => array(&mut strip_byte_counts)?,
            320 => {
                if typ != 3 {
                    return None;
                }
                color_map_off = u32_at(e + 8)? as usize;
                color_map_len = count;
            }
            _ => {}
        }
    }

    if photometric != 3 || compression != 1 || bits_per_sample != 8 {
        return None;
    }
    let pixels = u64::from(width).checked_mul(u64::from(height))?;
    if width == 0
        || height == 0
        || width > CANVASKIT_MAX_IMAGE_DIMENSION
        || height > CANVASKIT_MAX_IMAGE_DIMENSION
        || pixels > CANVASKIT_MAX_IMAGE_PIXELS
    {
        return None;
    }
    // 8bps 팔레트의 ColorMap 은 R·G·B 각 256개, 16-bit 값 (TIFF 6.0 §5).
    if color_map_len != 3 * 256 || strip_offsets.len() != strip_byte_counts.len() {
        return None;
    }
    let palette_component =
        |i: usize| -> Option<u8> { Some((u16_at(color_map_off + i * 2)? >> 8) as u8) };

    let pixel_count = usize::try_from(pixels).ok()?;
    let mut indices = Vec::with_capacity(pixel_count);
    for (off, len) in strip_offsets.iter().zip(strip_byte_counts.iter()) {
        let strip = data.get(*off as usize..(*off as usize).checked_add(*len as usize)?)?;
        indices.extend_from_slice(strip);
        if indices.len() >= pixel_count {
            break;
        }
    }
    if indices.len() < pixel_count {
        return None;
    }

    let mut rgba = vec![0u8; pixel_count.checked_mul(4)?];
    for (dst, &idx) in rgba.chunks_exact_mut(4).zip(indices.iter()) {
        dst[0] = palette_component(idx as usize)?;
        dst[1] = palette_component(256 + idx as usize)?;
        dst[2] = palette_component(512 + idx as usize)?;
        dst[3] = 255;
    }
    let img = RgbaImage::from_raw(width, height, rgba)?;
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// Browser SVG/Canvas decoders can expose stale color planes in old Photoshop
/// grayscale JPEGs. Re-encode only visually gray JPEGs to PNG so color photos
/// keep the compact JPEG path.
pub(crate) fn grayscale_jpeg_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    memoized(Conversion::GrayscaleJpeg, data, || {
        grayscale_jpeg_bytes_to_png_bytes_uncached(data)
    })
}

fn grayscale_jpeg_bytes_to_png_bytes_uncached(data: &[u8]) -> Option<Vec<u8>> {
    use image::ImageFormat;

    if detect_image_mime_type(data) != "image/jpeg" {
        return None;
    }

    let mut img = decode_image_with_format_limited(data, ImageFormat::Jpeg)?.to_rgba8();
    if img.width() == 0 || img.height() == 0 {
        return None;
    }

    let has_photoshop_profile = data
        .windows(b"Adobe Photoshop".len())
        .any(|chunk| chunk == b"Adobe Photoshop")
        || data
            .windows(b"Adobe_CM".len())
            .any(|chunk| chunk == b"Adobe_CM");
    let is_gray = img.pixels().all(|px| {
        let [r, g, b, _] = px.0;
        let min = r.min(g).min(b);
        let max = r.max(g).max(b);
        max.saturating_sub(min) <= 2
    });
    let is_luma_plane_gray = has_photoshop_profile
        && img.pixels().all(|px| {
            let [_, g, b, _] = px.0;
            g.abs_diff(128) <= 2 && b.abs_diff(128) <= 2
        });
    if is_luma_plane_gray {
        for px in img.pixels_mut() {
            let gray = px.0[0];
            px.0 = [gray, gray, gray, px.0[3]];
        }
    } else if !is_gray {
        return None;
    }

    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// PCX 바이트를 PNG 바이트로 재인코딩한다. 실패 시 None 반환.
///
/// 브라우저는 PCX 포맷을 native 렌더링하지 못하므로 (구형 ZSoft Paintbrush 포맷),
/// SVG 임베딩 전에 PNG로 변환해 호환성을 확보한다.
/// paletted PCX (8bpp) 와 RGB PCX (24bpp) 모두 지원.
///
/// **투명 처리**: PCX 자체는 알파 채널을 지원하지 않지만, HWP 의 PCX 임베드는
/// 보통 BehindText (글뒤로) 배경/로고 용도로 흰색 (255,255,255) 영역을 투명으로
/// 보여야 한다 (한컴 호환). 변환 시 흰색 픽셀을 투명 알파로 매핑한 RGBA PNG 를
/// 출력한다.
pub(crate) fn pcx_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    memoized(Conversion::Pcx, data, || {
        pcx_bytes_to_png_bytes_uncached(data)
    })
}

fn pcx_bytes_to_png_bytes_uncached(data: &[u8]) -> Option<Vec<u8>> {
    use image::{ImageFormat, RgbaImage};

    let mut reader = pcx::Reader::new(Cursor::new(data)).ok()?;
    let width = reader.width() as u32;
    let height = reader.height() as u32;
    let pixels = u64::from(width).checked_mul(u64::from(height))?;
    if width == 0
        || height == 0
        || width > CANVASKIT_MAX_IMAGE_DIMENSION
        || height > CANVASKIT_MAX_IMAGE_DIMENSION
        || pixels > CANVASKIT_MAX_IMAGE_PIXELS
    {
        return None;
    }
    let pixel_count = usize::try_from(pixels).ok()?;
    let mut rgba = vec![0u8; pixel_count.checked_mul(4)?];
    if reader.is_paletted() {
        let row_bytes = width as usize;
        let mut indices = vec![0u8; row_bytes * height as usize];
        for y in 0..height as usize {
            reader
                .next_row_paletted(&mut indices[y * row_bytes..(y + 1) * row_bytes])
                .ok()?;
        }
        let mut palette = vec![0u8; 256 * 3];
        reader.read_palette(&mut palette).ok()?;
        for (dst, &idx) in rgba.chunks_exact_mut(4).zip(indices.iter()) {
            let p = idx as usize * 3;
            let r = palette[p];
            let g = palette[p + 1];
            let b = palette[p + 2];
            dst[0] = r;
            dst[1] = g;
            dst[2] = b;
            dst[3] = if r == 255 && g == 255 && b == 255 {
                0
            } else {
                255
            };
        }
    } else {
        let row_bytes_rgb = width as usize * 3;
        let mut rgb_row = vec![0u8; row_bytes_rgb];
        for y in 0..height as usize {
            reader.next_row_rgb(&mut rgb_row).ok()?;
            for (x, src) in rgb_row.chunks_exact(3).enumerate() {
                let dst = &mut rgba[(y * width as usize + x) * 4..(y * width as usize + x) * 4 + 4];
                dst[0] = src[0];
                dst[1] = src[1];
                dst[2] = src[2];
                dst[3] = if src[0] == 255 && src[1] == 255 && src[2] == 255 {
                    0
                } else {
                    255
                };
            }
        }
    }
    let img = RgbaImage::from_raw(width, height, rgba)?;
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

fn apply_real_picture_watermark_tone_rgb(r: u8, g: u8, b: u8) -> [u8; 3] {
    let mut rgb = [r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0];

    let saturation = REAL_PICTURE_WATERMARK_SATURATION;
    rgb = [
        (0.213 + 0.787 * saturation) * rgb[0]
            + (0.715 - 0.715 * saturation) * rgb[1]
            + (0.072 - 0.072 * saturation) * rgb[2],
        (0.213 - 0.213 * saturation) * rgb[0]
            + (0.715 + 0.285 * saturation) * rgb[1]
            + (0.072 - 0.072 * saturation) * rgb[2],
        (0.213 - 0.213 * saturation) * rgb[0]
            + (0.715 - 0.715 * saturation) * rgb[1]
            + (0.072 + 0.928 * saturation) * rgb[2],
    ];

    let contrast = REAL_PICTURE_WATERMARK_CONTRAST;
    let contrast_intercept = 0.5 - 0.5 * contrast;
    let brightness = REAL_PICTURE_WATERMARK_BRIGHTNESS;
    for channel in &mut rgb {
        *channel = (*channel * contrast + contrast_intercept) * brightness;
    }

    let corrected = [
        REAL_PICTURE_WATERMARK_CORRECTION_MATRIX[0][0] * rgb[0]
            + REAL_PICTURE_WATERMARK_CORRECTION_MATRIX[0][1] * rgb[1]
            + REAL_PICTURE_WATERMARK_CORRECTION_MATRIX[0][2] * rgb[2]
            + REAL_PICTURE_WATERMARK_CORRECTION_BIAS[0],
        REAL_PICTURE_WATERMARK_CORRECTION_MATRIX[1][0] * rgb[0]
            + REAL_PICTURE_WATERMARK_CORRECTION_MATRIX[1][1] * rgb[1]
            + REAL_PICTURE_WATERMARK_CORRECTION_MATRIX[1][2] * rgb[2]
            + REAL_PICTURE_WATERMARK_CORRECTION_BIAS[1],
        REAL_PICTURE_WATERMARK_CORRECTION_MATRIX[2][0] * rgb[0]
            + REAL_PICTURE_WATERMARK_CORRECTION_MATRIX[2][1] * rgb[1]
            + REAL_PICTURE_WATERMARK_CORRECTION_MATRIX[2][2] * rgb[2]
            + REAL_PICTURE_WATERMARK_CORRECTION_BIAS[2],
    ];

    let luma = 0.2126 * corrected[0] + 0.7152 * corrected[1] + 0.0722 * corrected[2];
    let chroma_corrected =
        corrected.map(|channel| luma + (channel - luma) * REAL_PICTURE_WATERMARK_CHROMA_GAIN);

    chroma_corrected.map(|channel| {
        let channel = channel.clamp(0.0, 1.0);
        let channel = channel + (1.0 - channel) * REAL_PICTURE_WATERMARK_WHITE_BLEND;
        (channel.clamp(0.0, 1.0) * 255.0).round() as u8
    })
}

fn apply_real_picture_watermark_fill_tone_rgb(r: u8, g: u8, b: u8) -> [u8; 3] {
    let [r, g, b] = apply_real_picture_watermark_tone_rgb(r, g, b);
    let rgb = [r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0];
    let luma = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
    let adjusted =
        rgb.map(|channel| luma + (channel - luma) * REAL_PICTURE_WATERMARK_FILL_CHROMA_GAIN);

    adjusted.map(|channel| {
        let channel = 0.78 + (channel - 0.78) * 1.89;
        let highlight = ((luma - 0.68) / 0.32).clamp(0.0, 1.0);
        let highlight_desat = highlight.powf(1.2) * 0.38;
        let channel = channel + (luma - channel) * highlight_desat;
        let white_blend = REAL_PICTURE_WATERMARK_FILL_WHITE_BLEND
            * (luma.powf(1.25) * 2.45 + highlight.powf(1.25) * 0.75);
        let channel = channel + (1.0 - channel) * white_blend;
        (channel.clamp(0.0, 1.0) * 255.0).round() as u8
    })
}

/// RealPic 색상 워터마크 preset을 한컴 뷰어에 가까운 색상 PNG로 변환한다.
pub(crate) fn real_picture_watermark_bytes_to_hancom_tone_png_bytes(
    data: &[u8],
) -> Option<Vec<u8>> {
    memoized(Conversion::RealPictureTone, data, || {
        real_picture_watermark_bytes_to_tone_png_bytes(data, apply_real_picture_watermark_tone_rgb)
    })
}

pub(crate) fn real_picture_watermark_fill_bytes_to_hancom_tone_png_bytes(
    data: &[u8],
) -> Option<Vec<u8>> {
    memoized(Conversion::RealPictureFillTone, data, || {
        real_picture_watermark_bytes_to_tone_png_bytes(
            data,
            apply_real_picture_watermark_fill_tone_rgb,
        )
    })
}

fn real_picture_watermark_bytes_to_tone_png_bytes(
    data: &[u8],
    tone: fn(u8, u8, u8) -> [u8; 3],
) -> Option<Vec<u8>> {
    use image::ImageFormat;

    let format = image::guess_format(data).ok()?;
    let mut img = decode_image_with_format_limited(data, format)?.to_rgba8();
    for px in img.pixels_mut() {
        let [r, g, b] = tone(px.0[0], px.0[1], px.0[2]);
        px.0 = [r, g, b, px.0[3]];
    }

    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// 워터마크 JPEG 를 한컴 PDF 정답지에 가까운 회색 톤 PNG 로 변환한다.
pub(crate) fn watermark_jpeg_bytes_to_hancom_baked_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    memoized(Conversion::WatermarkJpeg, data, || {
        watermark_jpeg_bytes_to_hancom_baked_png_bytes_uncached(data)
    })
}

fn watermark_jpeg_bytes_to_hancom_baked_png_bytes_uncached(data: &[u8]) -> Option<Vec<u8>> {
    use image::ImageFormat;

    let mut img = decode_image_with_format_limited(data, ImageFormat::Jpeg)?.to_rgba8();
    let width = img.width();
    let height = img.height();
    if width == 0 || height == 0 {
        return None;
    }

    fn is_near_white(px: [u8; 4]) -> bool {
        px[0] >= 245 && px[1] >= 245 && px[2] >= 245
    }

    let mut border_total = 0u64;
    let mut border_near_white = 0u64;
    for x in 0..width {
        for y in [0, height - 1] {
            border_total += 1;
            if is_near_white(img.get_pixel(x, y).0) {
                border_near_white += 1;
            }
        }
    }
    if height > 2 {
        for y in 1..height - 1 {
            for x in [0, width - 1] {
                border_total += 1;
                if is_near_white(img.get_pixel(x, y).0) {
                    border_near_white += 1;
                }
            }
        }
    }

    let mut all_near_white = 0u64;
    for px in img.pixels() {
        if is_near_white(px.0) {
            all_near_white += 1;
        }
    }

    let pixel_total = (width as u64) * (height as u64);
    if (border_near_white as f64 / border_total as f64) < 0.85
        || (all_near_white as f64 / pixel_total as f64) < 0.20
    {
        return None;
    }

    fn map_watermark_gray(gray: f64) -> u8 {
        let value = if gray < 50.0 {
            198.0 + 0.46 * gray
        } else if gray < 80.0 {
            221.0 + 0.47 * (gray - 50.0)
        } else if gray < 100.0 {
            235.1 + 0.14 * (gray - 80.0)
        } else if gray < 120.0 {
            237.9 + 0.385 * (gray - 100.0)
        } else if gray < 160.0 {
            245.6 + 0.1625 * (gray - 120.0)
        } else {
            252.1 + 0.032 * (gray - 160.0)
        };
        value.clamp(0.0, 255.0).round() as u8
    }

    for px in img.pixels_mut() {
        if is_near_white(px.0) {
            px.0 = [255, 255, 255, 255];
        } else {
            let gray = 0.299 * px.0[0] as f64 + 0.587 * px.0[1] as f64 + 0.114 * px.0[2] as f64;
            let mapped = map_watermark_gray(gray);
            px.0 = [mapped, mapped, mapped, 255];
        }
    }

    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// 이미지 데이터에서 MIME 타입 감지
pub(crate) fn detect_image_mime_type(data: &[u8]) -> &'static str {
    if data.len() >= 8 {
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return "image/png";
        }
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return "image/jpeg";
        }
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return "image/gif";
        }
        if data.starts_with(&[0x42, 0x4D]) {
            return "image/bmp";
        }
        if data.starts_with(&[0xD7, 0xCD, 0xC6, 0x9A])
            || data.starts_with(&[0x01, 0x00, 0x09, 0x00])
        {
            return "image/x-wmf";
        }
        // EMF: EMR_HEADER(Type=1) + offset 40 의 " EMF" 시그니처 (MS-EMF 2.3.4.2)
        if data.len() >= 44
            && data.starts_with(&[0x01, 0x00, 0x00, 0x00])
            && &data[40..44] == b" EMF"
        {
            return "image/x-emf";
        }
        if data.starts_with(&[0x49, 0x49, 0x2A, 0x00])
            || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
        {
            return "image/tiff";
        }
    }
    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        return "image/webp";
    }
    // PCX 버전바이트는 0(2.5)·2(2.8 팔레트)·3(2.8 무팔레트)·4·5 가 유효하다.
    // v5 만 잡으면 v2.8 이 octet-stream 으로 새어 변환 분기에 걸리지 못한다.
    // 인코딩바이트 0x01(RLE)까지 봐야 0x0A 로 시작하는 다른 바이트와 안 섞인다.
    if data.len() >= 3 && data[0] == 0x0A && matches!(data[1], 0 | 2 | 3 | 4 | 5) && data[2] == 0x01
    {
        return "image/x-pcx";
    }
    // PostScript: 텍스트 EPS(`%!PS`)와 DOS EPS 바이너리(`C5 D0 D3 C6` preamble).
    // 후자는 내장 WMF/TIFF 프리뷰로 변환 가능하다 (#4062).
    if data.starts_with(b"%!PS") || data.starts_with(&[0xC5, 0xD0, 0xD3, 0xC6]) {
        return "application/postscript";
    }
    // [#3460] HWPX BinData 는 SVG 를 그대로 담을 수 있다(`<hc:img>` → `Format="svg"`).
    // 여기서 놓치면 data URI 가 application/octet-stream 으로 나가 브라우저·rsvg 가
    // 그리지 않고 빈 공간이 된다. WASM 판별기(web_canvas)는 이미 같은 분기를 갖고 있다.
    if crate::renderer::svg_fragment::is_svg_prefix(data) {
        return "image/svg+xml";
    }
    "application/octet-stream"
}

fn decode_image_with_format_limited(
    data: &[u8],
    format: image::ImageFormat,
) -> Option<image::DynamicImage> {
    if matches!(
        format,
        image::ImageFormat::Png | image::ImageFormat::Jpeg | image::ImageFormat::Bmp
    ) && !canvaskit_encoded_image_header(data).is_some_and(|header| {
        header.is_within_decode_limits()
            && matches!(
                (format, header.format),
                (
                    image::ImageFormat::Png,
                    crate::renderer::image_header::CanvasKitEncodedImageFormat::Png
                ) | (
                    image::ImageFormat::Jpeg,
                    crate::renderer::image_header::CanvasKitEncodedImageFormat::Jpeg
                ) | (
                    image::ImageFormat::Bmp,
                    crate::renderer::image_header::CanvasKitEncodedImageFormat::Bmp
                )
            )
    }) {
        return None;
    }

    let mut reader = image::ImageReader::with_format(Cursor::new(data), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(CANVASKIT_MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(CANVASKIT_MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(CANVASKIT_MAX_IMAGE_PIXELS.saturating_mul(4));
    reader.limits(limits);
    reader.decode().ok()
}

#[cfg(test)]
mod tests {
    use super::{
        bmp_bytes_to_png_bytes, emitted_image_bytes, grayscale_jpeg_bytes_to_png_bytes,
        is_watermark_image, resolve_image_payload, watermark_jpeg_bytes_to_hancom_baked_png_bytes,
        ConversionMemo, CANVASKIT_MAX_IMAGE_DIMENSION, CONVERSIONS_RUN, MAX_MEMO_BYTES,
        MAX_MEMO_ENTRIES,
    };
    use crate::model::image::ImageEffect;
    use crate::paint::ResolvedImageKind;
    use crate::renderer::render_tree::ImageNode;
    use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
    use std::io::Cursor;

    fn jpeg_from_pixels(width: u32, height: u32, pixels: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
        let mut img = RgbImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                img.put_pixel(x, y, Rgb(pixels(x, y)));
            }
        }

        let mut out = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Jpeg)
            .expect("encode jpeg");
        out
    }

    /// 지금까지 실제로 수행된 변환 횟수.
    fn conversions_run() -> usize {
        CONVERSIONS_RUN.with(|runs| runs.get())
    }

    fn bmp_with_middle_band(mid: [u8; 3]) -> Vec<u8> {
        let mut img = RgbImage::new(64, 64);
        for y in 0..64u32 {
            for x in 0..64u32 {
                let px = if (24..40).contains(&y) {
                    Rgb(mid)
                } else {
                    Rgb([255, 255, 255])
                };
                img.put_pixel(x, y, px);
            }
        }

        let mut out = Vec::new();
        img.write_to(&mut Cursor::new(&mut out), ImageFormat::Bmp)
            .expect("encode bmp");
        out
    }

    /// 앞뒤 일부만 뽑는 표본 키는 다른 그림을 한 항목으로 묶는다.
    ///
    /// 무압축 BMP 는 같은 치수면 길이가 정확히 같고, 고정 헤더 뒤에 원시 픽셀이 아래에서
    /// 위로 이어진다. 위아래 여백이 같으면 길이·앞 4KiB·뒤 4KiB 가 전부 일치해, 가운데만
    /// 다른 두 그림이 남의 변환 결과를 받는다.
    #[test]
    fn images_sharing_length_and_edges_do_not_share_a_result() {
        let red = bmp_with_middle_band([200, 30, 30]);
        let blue = bmp_with_middle_band([30, 30, 200]);

        assert_ne!(red, blue);
        assert_eq!(red.len(), blue.len(), "같은 치수 무압축 BMP 는 길이가 같다");
        assert_eq!(&red[..4096], &blue[..4096], "앞 4KiB 가 같다");
        assert_eq!(
            &red[red.len() - 4096..],
            &blue[blue.len() - 4096..],
            "뒤 4KiB 가 같다"
        );

        let red_png = bmp_bytes_to_png_bytes(&red).expect("red bmp converts");
        let blue_png = bmp_bytes_to_png_bytes(&blue).expect("blue bmp converts");
        let decoded = image::load_from_memory(&blue_png)
            .expect("decode converted blue")
            .to_rgb8();

        assert_ne!(red_png, blue_png);
        assert_eq!(
            decoded.get_pixel(32, 32),
            &Rgb([30, 30, 200]),
            "파란 그림 자리에 빨간 그림이 나오면 안 된다"
        );
    }

    /// 같은 그림을 여러 번 해석해도 변환은 한 번만 한다 (#2520).
    ///
    /// 편집 한 번에 레이어 트리가 여러 벌 만들어져 같은 그림이 여러 번 들어온다.
    /// 색 사진은 결과가 `None` 이라 종전에는 **전체 디코드 결과를 매번 버렸다** —
    /// 2MB 사진 한 장에 키 입력당 수백 ms 가 들던 자리다.
    #[test]
    fn repeated_resolve_of_same_image_converts_once() {
        let jpeg = jpeg_from_pixels(24, 24, |x, y| [(x * 7) as u8, 40, (y * 9) as u8]);
        let image = ImageNode::new(1, Some(jpeg));

        let before = conversions_run();
        for _ in 0..3 {
            assert!(resolve_image_payload(&image).is_none());
        }
        assert_eq!(
            conversions_run() - before,
            1,
            "같은 그림은 한 번만 변환해야 한다"
        );
    }

    /// 메모가 다른 그림의 결과를 흘리지 않는다.
    #[test]
    fn different_images_keep_their_own_results() {
        let gray = jpeg_from_pixels(2, 2, |x, y| {
            let g = 120 + (x + y) as u8;
            [g, g, g]
        });
        let color = jpeg_from_pixels(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                [210, 48, 48]
            } else {
                [48, 110, 210]
            }
        });

        assert!(grayscale_jpeg_bytes_to_png_bytes(&gray).is_some());
        assert!(grayscale_jpeg_bytes_to_png_bytes(&color).is_none());
        // 순서를 바꿔도 각자의 결과가 나온다 (둘 다 메모에 들어간 뒤).
        assert!(grayscale_jpeg_bytes_to_png_bytes(&color).is_none());
        assert!(grayscale_jpeg_bytes_to_png_bytes(&gray).is_some());
    }

    /// 같은 바이트라도 변환 종류가 다르면 결과가 다르므로 따로 센다.
    #[test]
    fn same_bytes_under_different_conversions_do_not_share_an_entry() {
        let jpeg = jpeg_from_pixels(12, 12, |x, y| {
            let g = 200 + ((x + y) % 8) as u8;
            [g, g, g]
        });

        let before = conversions_run();
        let _ = grayscale_jpeg_bytes_to_png_bytes(&jpeg);
        let _ = watermark_jpeg_bytes_to_hancom_baked_png_bytes(&jpeg);
        assert_eq!(
            conversions_run() - before,
            2,
            "변환 종류가 다르면 메모 항목도 달라야 한다"
        );
    }

    /// 메모는 상한 안에서만 자란다 — 오래된 것부터 밀려난다.
    #[test]
    fn memo_stays_within_its_byte_budget() {
        let mut memo = ConversionMemo::default();
        let chunk = MAX_MEMO_BYTES / 4 + 1;
        for key in 0..8u64 {
            memo.insert(key, Some(vec![0u8; chunk]));
        }

        assert!(memo.bytes <= MAX_MEMO_BYTES);
        assert!(memo.get(0).is_none(), "가장 오래된 항목은 밀려나 있다");
        assert!(memo.get(7).is_some(), "가장 최근 항목은 남아 있다");
    }

    /// 결과가 `None` 인 항목은 바이트를 차지하지 않으므로 항목 수로 묶는다.
    #[test]
    fn memo_bounds_entry_count_even_when_results_are_empty() {
        let mut memo = ConversionMemo::default();
        for key in 0..(MAX_MEMO_ENTRIES as u64 * 2) {
            memo.insert(key, None);
        }

        assert_eq!(memo.bytes, 0);
        assert!(memo.entries.len() <= MAX_MEMO_ENTRIES);
        assert!(memo.get(0).is_none(), "가장 오래된 항목은 밀려나 있다");
    }

    #[test]
    fn grayscale_jpeg_is_normalized_to_png() {
        let jpeg = jpeg_from_pixels(2, 2, |x, y| {
            let g = 180 + (x + y) as u8;
            [g, g, g]
        });

        let png = grayscale_jpeg_bytes_to_png_bytes(&jpeg).expect("gray jpeg should normalize");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn color_jpeg_keeps_jpeg_path() {
        let jpeg = jpeg_from_pixels(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                [220, 64, 64]
            } else {
                [64, 120, 220]
            }
        });

        assert!(grayscale_jpeg_bytes_to_png_bytes(&jpeg).is_none());
    }

    #[test]
    fn tiff_image_payload_is_normalized_to_png() {
        let mut img = RgbImage::new(2, 2);
        for y in 0..2 {
            for x in 0..2 {
                img.put_pixel(x, y, Rgb([32 + x as u8, 96 + y as u8, 160]));
            }
        }
        let mut tiff = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut tiff), ImageFormat::Tiff)
            .expect("encode tiff");

        let image = ImageNode::new(1, Some(tiff));
        let resolved = resolve_image_payload(&image).expect("tiff should resolve");

        assert_eq!(resolved.mime, "image/png");
        assert_eq!(resolved.kind, ResolvedImageKind::FormatConverted);
        assert!(resolved.data.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(!resolved.suppress_effects);
    }

    #[test]
    fn oversized_compact_bmp_is_rejected_before_layer_conversion() {
        let mut bmp = vec![0u8; 58];
        bmp[..2].copy_from_slice(b"BM");
        bmp[2..6].copy_from_slice(&58u32.to_le_bytes());
        bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
        bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
        bmp[18..22].copy_from_slice(&8193i32.to_le_bytes());
        bmp[22..26].copy_from_slice(&8193i32.to_le_bytes());
        bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
        bmp[28..30].copy_from_slice(&8u16.to_le_bytes());
        bmp[30..34].copy_from_slice(&1u32.to_le_bytes());
        bmp[54..58].copy_from_slice(&[0, 1, 0, 1]);

        assert!(bmp_bytes_to_png_bytes(&bmp).is_none());
        assert!(resolve_image_payload(&ImageNode::new(1, Some(bmp))).is_none());
    }

    /// 변환에 성공하는 최소 PCX v2.8 (버전바이트 0x03, 1bpp 모노크롬 8×1).
    ///
    /// 10k 스윕에서 걸린 실문서의 헤더 선두(`0A 03 01 01`)와 같은 배치다. 128B 헤더 뒤에
    /// RLE 행 데이터가 온다 — bytes_per_line=2 이므로 한 행은 데이터 1B + 패딩 1B.
    fn minimal_pcx_v28() -> Vec<u8> {
        let mut out = vec![0u8; 128];
        out[0] = 0x0A; // Identifier
        out[1] = 0x03; // Version 2.8 without palette
        out[2] = 0x01; // RLE encoding
        out[3] = 0x01; // bits per pixel
        out[8..10].copy_from_slice(&7u16.to_le_bytes()); // xmax → width 8
        out[12..14].copy_from_slice(&300u16.to_le_bytes()); // hdpi
        out[14..16].copy_from_slice(&300u16.to_le_bytes()); // vdpi
        out[65] = 1; // color planes
        out[66..68].copy_from_slice(&2u16.to_le_bytes()); // bytes per line (짝수 패딩)
        out[68..70].copy_from_slice(&1u16.to_le_bytes()); // palette info
        out.extend_from_slice(&[0xAA, 0x00]); // RLE 리터럴: 행 데이터 + 패딩
        out
    }

    /// PCX v2.8 도 v5 와 같은 계약이다 — 판별돼 PNG 로 변환돼 나가야 한다 (#4065).
    ///
    /// 판별기가 v5(`0A 05`)만 인식하면 v2.8 은 octet-stream 으로 새어
    /// `pcx_bytes_to_png_bytes` 분기에 걸리지 못하고, 소비자가 디코드할 수 없는
    /// 원본 바이트가 그대로 나간다 (10k 스윕 실문서 1건).
    #[test]
    fn pcx_v28_is_detected_and_emitted_as_png() {
        let pcx = minimal_pcx_v28();
        assert_eq!(
            super::detect_image_mime_type(&pcx),
            "image/x-pcx",
            "합성 바이트가 PCX 로 판별되지 않으면 이 테스트는 아무것도 검증하지 않는다"
        );

        let (mime, bytes) = emitted_image_bytes(&pcx, false);
        assert_eq!(mime, "image/png", "PCX 는 PNG 로 변환돼 나가야 한다");
        assert!(
            bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "mime 만 바꾸고 바이트가 원본 PCX 면 소비자는 여전히 못 그린다"
        );

        let resolved = resolve_image_payload(&ImageNode::new(9, Some(pcx)))
            .expect("v2.8 도 resolve 경로에서 변환돼야 한다");
        assert_eq!(resolved.mime, "image/png");
        assert_eq!(resolved.kind, ResolvedImageKind::FormatConverted);
    }

    /// image 크레이트가 거부하는 8-bit 팔레트 TIFF (photometric=3) — 10k 스윕
    /// 실문서 스캔본(비압축·8bps)과 같은 배치의 합성 바이트 (#4064).
    fn minimal_palette_tiff() -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00]); // II*\0 (LE)
        out.extend_from_slice(&8u32.to_le_bytes()); // IFD offset
        let entry = |out: &mut Vec<u8>, tag: u16, typ: u16, count: u32, value: u32| {
            out.extend_from_slice(&tag.to_le_bytes());
            out.extend_from_slice(&typ.to_le_bytes());
            out.extend_from_slice(&count.to_le_bytes());
            out.extend_from_slice(&value.to_le_bytes());
        };
        // IFD: 8 entries × 12B + count(2) + next(4) = 102B → 데이터는 110부터.
        let strip_off = 110u32;
        let color_map_off = strip_off + 4; // 2×2 인덱스 4B 뒤
        out.extend_from_slice(&8u16.to_le_bytes());
        entry(&mut out, 256, 3, 1, 2); // ImageWidth = 2
        entry(&mut out, 257, 3, 1, 2); // ImageLength = 2
        entry(&mut out, 258, 3, 1, 8); // BitsPerSample = 8
        entry(&mut out, 259, 3, 1, 1); // Compression = none
        entry(&mut out, 262, 3, 1, 3); // Photometric = palette
        entry(&mut out, 273, 4, 1, strip_off); // StripOffsets
        entry(&mut out, 279, 4, 1, 4); // StripByteCounts
        entry(&mut out, 320, 3, 3 * 256, color_map_off); // ColorMap
        out.extend_from_slice(&0u32.to_le_bytes()); // next IFD 없음
        assert_eq!(out.len(), strip_off as usize);
        out.extend_from_slice(&[0, 1, 1, 0]); // 인덱스 2×2
                                              // ColorMap: R[256]·G[256]·B[256], 16-bit. 색 0=빨강, 1=파랑.
        let mut plane = |c0: u16, c1: u16| {
            let mut v = vec![0u16; 256];
            v[0] = c0;
            v[1] = c1;
            for e in v {
                out.extend_from_slice(&e.to_le_bytes());
            }
        };
        plane(0xFF00, 0x0000); // R
        plane(0x0000, 0x0000); // G
        out.extend_from_slice(
            &{
                let mut v = vec![0u16; 256];
                v[1] = 0xFF00;
                v.iter().flat_map(|e| e.to_le_bytes()).collect::<Vec<_>>()
            }[..],
        ); // B
        out
    }

    /// 팔레트 TIFF 도 PNG 로 변환돼 나가야 한다 (#4064). image 크레이트 tiff
    /// 디코더가 photometric=RGBPalette 를 지원하지 않아 실문서 스캔본 6op 가
    /// 원본 TIFF 그대로 새던 부류를 폴백 디코더로 고정한다.
    #[test]
    fn palette_tiff_is_emitted_as_png_via_fallback_decoder() {
        let tiff = minimal_palette_tiff();
        assert_eq!(super::detect_image_mime_type(&tiff), "image/tiff");
        // 전제 고정: image 크레이트가 이 형태를 못 읽어야 폴백이 검증된다.
        assert!(
            super::decode_image_with_format_limited(&tiff, image::ImageFormat::Tiff).is_none(),
            "image 크레이트가 팔레트 TIFF 를 읽게 되면 이 폴백은 접어도 된다"
        );

        let (mime, bytes) = emitted_image_bytes(&tiff, false);
        assert_eq!(mime, "image/png");
        let png = image::load_from_memory(&bytes)
            .expect("PNG 디코드")
            .to_rgba8();
        assert_eq!((png.width(), png.height()), (2, 2));
        assert_eq!(
            png.get_pixel(0, 0).0,
            [255, 0, 0, 255],
            "색 0 = 팔레트 빨강"
        );
        assert_eq!(
            png.get_pixel(1, 0).0,
            [0, 0, 255, 255],
            "색 1 = 팔레트 파랑"
        );
    }

    /// CanvasKit 한도(8192px·32Mpix)를 넘는 BMP 는 거부 대신 한도 안으로
    /// 다운스케일해 PNG 로 낸다 (#4064). SVG `<image>` 는 data URI BMP 를 표준
    /// 지원하지 않아 거부하면 빈 그림이 된다 — A4 전면 스캔 실문서 4건.
    #[test]
    fn oversized_bmp_is_downscaled_to_png_instead_of_rejected() {
        use image::{DynamicImage, ImageFormat, RgbImage};
        let wide = RgbImage::from_fn(8400, 2, |x, _| image::Rgb([(x % 251) as u8, 90, 200]));
        let mut bmp = Vec::new();
        DynamicImage::ImageRgb8(wide)
            .write_to(&mut Cursor::new(&mut bmp), ImageFormat::Bmp)
            .expect("encode bmp");

        let png = bmp_bytes_to_png_bytes(&bmp).expect("한도 초과 BMP 는 다운스케일로 살린다");
        let decoded = image::load_from_memory(&png).expect("PNG 디코드");
        assert!(
            decoded.width() <= CANVASKIT_MAX_IMAGE_DIMENSION,
            "다운스케일 결과가 한도 안이어야 한다 (실측 {})",
            decoded.width()
        );
        assert!(decoded.width() > 0 && decoded.height() > 0);
    }

    /// DOS EPS 바이너리는 내장 TIFF/WMF 프리뷰로 변환돼 나가야 한다 (#4062).
    /// 텍스트 PostScript 는 변환기가 없어 여전히 원본 그대로다 — 그 경계도
    /// 함께 고정한다.
    #[test]
    fn dos_eps_with_tiff_preview_is_emitted_as_png() {
        use image::{DynamicImage, ImageFormat, RgbImage};
        let mut tiff = Vec::new();
        DynamicImage::ImageRgb8(RgbImage::from_pixel(2, 2, image::Rgb([10, 200, 30])))
            .write_to(&mut Cursor::new(&mut tiff), ImageFormat::Tiff)
            .expect("encode tiff");

        let mut eps = Vec::new();
        eps.extend_from_slice(&[0xC5, 0xD0, 0xD3, 0xC6]); // preamble
        let header_len = 30u32;
        eps.extend_from_slice(&header_len.to_le_bytes()); // PS offset (더미)
        eps.extend_from_slice(&0u32.to_le_bytes()); // PS length
        eps.extend_from_slice(&0u32.to_le_bytes()); // WMF offset
        eps.extend_from_slice(&0u32.to_le_bytes()); // WMF length
        eps.extend_from_slice(&header_len.to_le_bytes()); // TIFF offset
        eps.extend_from_slice(&(tiff.len() as u32).to_le_bytes()); // TIFF length
        eps.extend_from_slice(&[0, 0]); // checksum 자리
        assert_eq!(eps.len(), header_len as usize);
        eps.extend_from_slice(&tiff);

        assert_eq!(
            super::detect_image_mime_type(&eps),
            "application/postscript"
        );
        let (mime, bytes) = emitted_image_bytes(&eps, false);
        assert_eq!(mime, "image/png", "TIFF 프리뷰가 PNG 로 나가야 한다");
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));

        let text_ps = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\n".to_vec();
        assert_eq!(
            super::detect_image_mime_type(&text_ps),
            "application/postscript"
        );
        let (mime, bytes) = emitted_image_bytes(&text_ps, false);
        assert_eq!(mime, "application/postscript", "텍스트 PS 는 변환기가 없다");
        assert_eq!(bytes.as_ref(), text_ps.as_slice());
    }
}

#[cfg(test)]
mod emitted_bytes_key_agreement_tests {
    //! Task #3315: 신원 키의 variant 가 JSON 경로와 **같은 바이트**를 가리키는지 고정한다.
    //!
    //! `getPageLayerTree` 가 base64 를 생략하면 소비자는 `getSourceImageBytes(key)` 로 바이트를
    //! 받는다. 두 경로는 `emitted_image_bytes` 를 함께 쓰므로 변환 사슬 자체는 갈라질 수 없지만,
    //! **넘기는 술어가 다르다** —
    //!
    //! - JSON 경로: `is_watermark_image(image)` (ImageNode 를 직접 본다)
    //! - 키 조회 경로: `parse_source_image_key(key)` 로 되읽은 variant
    //!
    //! 그 둘이 어긋나면 워터마크 그림에 원본 JPEG 을 돌려주고도 성공한 것처럼 보인다. 여기서
    //! 고정하는 것은 "키만으로 같은 바이트를 재현할 수 있다"는 계약이고, 이 테스트가 만드는
    //! 변환 분기(BMP·TIFF·회색 JPEG·워터마크 JPEG·변환 실패 되돌림)마다 확인한다.

    use super::{
        detect_image_mime_type, emitted_image_bytes, emitted_image_mime, is_watermark_image,
        resolve_image_payload,
    };
    use crate::model::image::ImageEffect;
    use crate::paint::{parse_source_image_key, source_image_key};
    use crate::renderer::render_tree::ImageNode;
    use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
    use std::io::Cursor;

    const EPOCH: u32 = 5;

    fn encoded(
        width: u32,
        height: u32,
        format: ImageFormat,
        pixel: impl Fn(u32, u32) -> [u8; 3],
    ) -> Vec<u8> {
        let mut img = RgbImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                img.put_pixel(x, y, Rgb(pixel(x, y)));
            }
        }
        let mut out = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut out), format)
            .expect("encode");
        out
    }

    fn gray_jpeg() -> Vec<u8> {
        encoded(8, 8, ImageFormat::Jpeg, |x, y| {
            let g = 150 + ((x + y) % 32) as u8;
            [g, g, g]
        })
    }

    fn color_jpeg() -> Vec<u8> {
        encoded(8, 8, ImageFormat::Jpeg, |x, y| {
            if (x + y) % 2 == 0 {
                [220, 64, 64]
            } else {
                [64, 120, 220]
            }
        })
    }

    /// 워터마크 bake 가 **실제로 성공하는** JPEG.
    ///
    /// `watermark_jpeg_bytes_to_hancom_baked_png_bytes` 는 테두리의 85% 이상과 전체의 20% 이상이
    /// 흰색에 가까워야 bake 한다. 단색·격자 같은 합성 이미지는 그 조건을 못 맞춰 `None` 이
    /// 되고, 그러면 `wmpng` 와 `src` 두 분기가 같은 되돌림 결과로 수렴해 **variant 가 갈렸는지
    /// 확인할 수 없다**. 그래서 흰 바탕에 작은 어두운 얼룩을 둔 모양으로 만든다.
    fn watermark_shaped_jpeg() -> Vec<u8> {
        encoded(48, 48, ImageFormat::Jpeg, |x, y| {
            let center = (20..28).contains(&x) && (20..28).contains(&y);
            if center {
                [40, 40, 40]
            } else {
                [255, 255, 255]
            }
        })
    }

    /// 그림 하나에 대해 두 경로의 (mime, bytes) 가 같은지 확인한다.
    fn assert_paths_agree(label: &str, image: &ImageNode) {
        let data = image.data.as_deref().expect("바이트가 있어야 한다");

        // JSON 경로가 쓰는 술어.
        let (json_mime, json_bytes) = emitted_image_bytes(data, is_watermark_image(image));

        // 키 조회 경로 — 키를 문자열로 내보내고 되읽어 variant 만으로 재현한다.
        let key = source_image_key(EPOCH, image).expect("bin_data_id != 0 이면 키가 나온다");
        let (epoch, bin_data_id, variant) =
            parse_source_image_key(&key).expect("발급한 키는 되읽을 수 있어야 한다");
        assert_eq!(epoch, EPOCH, "{label}: 키의 세대가 어긋난다");
        assert_eq!(
            bin_data_id, image.bin_data_id,
            "{label}: 키의 id 가 어긋난다"
        );
        let (key_mime, key_bytes) = emitted_image_bytes(data, variant.bakes_watermark());

        assert_eq!(
            json_mime, key_mime,
            "{label}: mime 이 갈렸다 — 소비자가 Blob 타입을 잘못 정한다 (key={key})"
        );
        assert_eq!(
            json_bytes.as_ref(),
            key_bytes.as_ref(),
            "{label}: 바이트가 갈렸다 — 키로 받은 그림이 인라인 base64 와 다르다 (key={key})"
        );
    }

    fn watermarked(mut image: ImageNode) -> ImageNode {
        // `is_watermark_image` 가 참이 되는 최소 조건.
        image.effect = ImageEffect::GrayScale;
        image.brightness = 20;
        image
    }

    #[test]
    fn issue_3315_key_variant_reproduces_json_bytes_for_covered_conversion_branches() {
        let png = encoded(4, 4, ImageFormat::Png, |_, _| [10, 20, 30]);
        let bmp = encoded(16, 16, ImageFormat::Bmp, |x, _| [x as u8 * 4, 90, 200]);
        let tiff = encoded(4, 4, ImageFormat::Tiff, |x, y| {
            [32 + x as u8, 96 + y as u8, 160]
        });

        let cases: Vec<(&str, ImageNode)> = vec![
            // 변환 없음 — 원본 mime 그대로.
            ("PNG", ImageNode::new(1, Some(png.clone()))),
            // 브라우저가 못 읽는 포맷 → PNG 변환.
            ("BMP", ImageNode::new(2, Some(bmp))),
            ("TIFF", ImageNode::new(3, Some(tiff))),
            // 회색 JPEG → PNG 정규화.
            ("회색 JPEG", ImageNode::new(4, Some(gray_jpeg()))),
            // 색 JPEG → 변환 없음(정규화 대상이 아니다).
            ("색 JPEG", ImageNode::new(5, Some(color_jpeg()))),
            // 워터마크 bake — variant 가 `wmpng` 로 갈리는 유일한 축. bake 가 실제로 성공하는
            // 모양이어야 두 분기의 결과가 달라지고, 그래야 갈라짐을 잡을 수 있다.
            (
                "워터마크 JPEG(bake 성공)",
                watermarked(ImageNode::new(6, Some(watermark_shaped_jpeg()))),
            ),
            // bake 술어는 참인데 bake 가 실패하는 모양 — 두 분기가 같은 되돌림으로 수렴한다.
            (
                "워터마크 술어 + bake 실패",
                watermarked(ImageNode::new(7, Some(gray_jpeg()))),
            ),
            // JPEG 이 아니면 효과가 붙어도 bake 대상이 아니다 — variant 는 `src` 로 남아야 한다.
            ("효과 붙은 PNG", watermarked(ImageNode::new(8, Some(png)))),
        ];

        for (label, image) in &cases {
            assert_paths_agree(label, image);
        }
    }

    /// WMF 는 SVG 로 나가야 한다 — 원본 WMF 를 그대로 내보내면 브라우저가 못 그린다.
    ///
    /// `svg.rs`·`web_canvas.rs` 는 각자 내보내기 직전에 `convert_wmf_to_svg` 를 부르므로
    /// export-svg 와 canvas 백엔드는 멀쩡했다. 그런데 DOM `<img>` 경로(`getSourceImageBytes`
    /// 로 바이트를 받아 `Blob` 을 만드는 studio, layer tree 의 인라인 base64)는 이 함수를
    /// 지나는데 여기에 WMF 분기가 없어서, 표 안 차트 같은 WMF 그림이 `image/x-wmf` Blob 으로
    /// 나가 `naturalWidth === 0` 인 깨진 그림이 됐다(관세청 월간 수출입 현황 1쪽).
    ///
    /// 변환기 자체는 멀쩡했으므로, 이 테스트가 고정하는 것은 **변환이 이 경로에도 걸려 있는가**다.
    #[test]
    fn wmf_is_emitted_as_svg_not_raw_wmf() {
        let wmf = minimal_wmf();
        assert_eq!(
            detect_image_mime_type(&wmf),
            "image/x-wmf",
            "합성 바이트가 WMF 로 판별되지 않으면 이 테스트는 아무것도 검증하지 않는다"
        );

        let (mime, bytes) = emitted_image_bytes(&wmf, false);
        assert_eq!(
            mime, "image/svg+xml",
            "WMF 는 SVG 로 변환돼 나가야 한다 — 브라우저는 WMF 를 디코드하지 못한다"
        );
        assert!(
            crate::renderer::svg_fragment::is_svg_prefix(&bytes),
            "mime 만 바꾸고 바이트가 원본 WMF 면 소비자는 여전히 못 그린다"
        );

        // 바이트를 안 받는 소비자(좁은 질의의 mime 필드)도 같은 답을 내야 한다.
        let image = ImageNode::new(9, Some(wmf.clone()));
        let resolved = resolve_image_payload(&image);
        assert_eq!(
            emitted_image_mime(&wmf, resolved.as_ref(), false),
            "image/svg+xml",
            "mime 만 아는 경로가 갈리면 Blob 타입이 바이트와 어긋난다"
        );

        // 두 경로(JSON 인라인 · 키 조회)가 같은 바이트를 준다.
        assert_paths_agree("WMF", &image);
    }

    /// 변환에 성공하는 최소 WMF.
    ///
    /// `tests/wmf_poly_negative_point_count_no_panic.rs` 의 합성 방식과 같다 —
    /// `META_HEADER`(type=1, headersize=9 words) 뒤에 창 좌표를 세우는 레코드와 사각형
    /// 하나를 넣고 `META_EOF` 로 닫는다. 빈 메타파일이 아니라 **그릴 것이 있는** 모양이어야
    /// 변환 결과가 실제 SVG 도형을 담는다.
    fn minimal_wmf() -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        let push_u16 = |v: u16, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());
        let push_i16 = |v: i16, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());
        let push_u32 = |v: u32, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());

        // ── META_HEADER ──
        push_u16(1, &mut out); // Type: memory metafile
        push_u16(9, &mut out); // HeaderSize: 9 words
        push_u16(0x0300, &mut out); // Version 3.0
        push_u32(0, &mut out); // SizeLow
        push_u32(0, &mut out); // SizeHigh 자리
        push_u16(0, &mut out); // NumberOfObjects
        push_u32(0, &mut out); // MaxRecord
        push_u16(0, &mut out); // NumberOfMembers

        // ── META_SETWINDOWEXT (0x020C): size 5 words ──
        push_u32(5, &mut out);
        push_u16(0x020C, &mut out);
        push_i16(100, &mut out); // Height
        push_i16(100, &mut out); // Width

        // ── META_RECTANGLE (0x041B): size 7 words, (bottom, right, top, left) ──
        push_u32(7, &mut out);
        push_u16(0x041B, &mut out);
        push_i16(80, &mut out);
        push_i16(80, &mut out);
        push_i16(20, &mut out);
        push_i16(20, &mut out);

        // ── META_EOF ──
        push_u32(3, &mut out);
        push_u16(0x0000, &mut out);

        out
    }

    /// EMF 도 WMF 와 같은 계약이다 — SVG 로 변환돼 나가야 한다.
    ///
    /// 저장소에 `emf` 변환기가 있었지만 OLE 프리뷰 경로에서만 쓰였고, 판별기에 EMF
    /// 매직이 없어 직접 삽입 EMF 그림은 `application/octet-stream` 으로 새어 나갔다
    /// (10k 스윕에서 16문서·109op 확인). 이 테스트는 ① 판별이 `image/x-emf` 인지
    /// 먼저 못박고 ② 변환이 이 경로에 걸려 있는지 ③ mime 만 아는 소비자와 바이트를
    /// 받는 소비자가 같은 답을 내는지 확인한다.
    #[test]
    fn emf_is_emitted_as_svg_not_raw_emf() {
        let emf = minimal_emf();
        assert_eq!(
            detect_image_mime_type(&emf),
            "image/x-emf",
            "합성 바이트가 EMF 로 판별되지 않으면 이 테스트는 아무것도 검증하지 않는다"
        );

        let (mime, bytes) = emitted_image_bytes(&emf, false);
        assert_eq!(
            mime, "image/svg+xml",
            "EMF 는 SVG 로 변환돼 나가야 한다 — 브라우저는 EMF 를 디코드하지 못한다"
        );
        assert!(
            crate::renderer::svg_fragment::is_svg_prefix(&bytes),
            "mime 만 바꾸고 바이트가 원본 EMF 면 소비자는 여전히 못 그린다"
        );

        let image = ImageNode::new(11, Some(emf.clone()));
        let resolved = resolve_image_payload(&image);
        assert_eq!(
            emitted_image_mime(&emf, resolved.as_ref(), false),
            "image/svg+xml",
            "mime 만 아는 경로가 갈리면 Blob 타입이 바이트와 어긋난다"
        );

        assert_paths_agree("EMF", &image);
    }

    /// 변환에 성공하는 최소 EMF.
    ///
    /// `src/emf/tests.rs` 의 `fixture_minimal_header_eof` 와 같은 헤더 배치다 —
    /// EMR_HEADER(88B, frame 100×50 ×0.01mm) 뒤에 EMR_RECTANGLE 하나를 넣고
    /// EMR_EOF 로 닫는다. **그릴 것이 있는** 모양이어야 변환 결과가 실제 SVG
    /// 도형을 담는다.
    fn minimal_emf() -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        let push_u16 = |v: u16, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());
        let push_u32 = |v: u32, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());
        let push_i32 = |v: i32, buf: &mut Vec<u8>| buf.extend_from_slice(&v.to_le_bytes());

        // ── EMR_HEADER (88B) ──
        push_u32(1, &mut out); // Type=1
        push_u32(88, &mut out); // Size=88
        for v in [0, 0, 1000, 500] {
            push_i32(v, &mut out); // Bounds RECTL (장치 좌표)
        }
        for v in [0, 0, 10000, 5000] {
            push_i32(v, &mut out); // Frame RECTL (0.01mm)
        }
        push_u32(0x464D_4520, &mut out); // Signature " EMF"
        push_u32(0x0001_0000, &mut out); // Version
        push_u32(88 + 24 + 20, &mut out); // Bytes: header + rectangle + eof
        push_u32(3, &mut out); // Records
        push_u16(1, &mut out); // Handles
        push_u16(0, &mut out); // Reserved
        push_u32(0, &mut out); // nDescription
        push_u32(0, &mut out); // offDescription
        push_u32(0, &mut out); // nPalEntries
        push_i32(1920, &mut out); // Device SIZEL
        push_i32(1080, &mut out);
        push_i32(508, &mut out); // Millimeters SIZEL
        push_i32(286, &mut out);
        assert_eq!(out.len(), 88);

        // ── EMR_RECTANGLE (0x2B): RectL 16B ──
        push_u32(0x2B, &mut out);
        push_u32(24, &mut out);
        for v in [10, 20, 110, 120] {
            push_i32(v, &mut out);
        }

        // ── EMR_EOF (0x0E): 20B ──
        push_u32(0x0E, &mut out);
        push_u32(20, &mut out);
        push_u32(0, &mut out); // nPalEntries
        push_u32(0, &mut out); // offPalEntries
        push_u32(20, &mut out); // SizeLast

        out
    }

    /// bake 가 성공하는 모양에서 두 variant 의 바이트가 **실제로 다름**을 먼저 못박는다.
    ///
    /// 이 단언이 없으면 위 등가성 테스트가 "두 경로가 같다"를 확인하는 게 아니라 "두 분기가
    /// 애초에 구분되지 않는다"를 확인하는 것일 수 있다.
    #[test]
    fn issue_3315_watermark_variant_actually_changes_the_bytes() {
        let jpeg = watermark_shaped_jpeg();
        let (baked_mime, baked) = emitted_image_bytes(&jpeg, true);
        let (plain_mime, plain) = emitted_image_bytes(&jpeg, false);

        assert_eq!(baked_mime, "image/png", "bake 결과는 PNG 다");
        assert_ne!(
            baked.as_ref(),
            plain.as_ref(),
            "이 모양에서 bake 가 돌지 않으면 variant 갈라짐을 검증할 수 없다"
        );
        // mime 은 갈리지 않는다 — 흰 바탕 그림은 비-워터마크 경로에서도 회색 JPEG 으로 판정돼
        // PNG 로 정규화된다. 즉 이 축의 차이는 **바이트에만** 나타나므로, mime 만 비교하는
        // 검증은 워터마크 갈라짐을 놓친다.
        assert_eq!(plain_mime, "image/png");
    }

    #[test]
    fn issue_3315_variant_marks_watermark_bake_only_for_jpeg() {
        let png = encoded(4, 4, ImageFormat::Png, |_, _| [1, 2, 3]);
        let jpeg = color_jpeg();

        // JPEG + 효과 → wmpng
        let key = source_image_key(EPOCH, &watermarked(ImageNode::new(1, Some(jpeg.clone()))))
            .expect("키");
        assert!(
            key.ends_with(":wmpng"),
            "JPEG 워터마크는 wmpng 여야 한다: {key}"
        );

        // JPEG + 효과 없음 → src
        let key = source_image_key(EPOCH, &ImageNode::new(1, Some(jpeg))).expect("키");
        assert!(
            key.ends_with(":src"),
            "효과 없는 JPEG 은 src 여야 한다: {key}"
        );

        // PNG + 효과 → src (bake 는 JPEG 경로에만 있다)
        let key = source_image_key(EPOCH, &watermarked(ImageNode::new(1, Some(png)))).expect("키");
        assert!(
            key.ends_with(":src"),
            "PNG 은 효과가 붙어도 bake 하지 않으므로 src 여야 한다: {key}"
        );
    }

    /// 변환이 **실패**하면 두 경로가 함께 원본으로 되돌아가야 한다.
    ///
    /// 되돌림이 한쪽에만 있으면 키로 받은 바이트가 인라인과 달라진다. 헤더만 그럴듯한 손상
    /// BMP 로 그 경로를 태운다 — `resolve_image_payload` 는 `None` 을 주고, 두 경로 모두
    /// `emitted_image_bytes` 안에서 같은 되돌림을 밟는다.
    #[test]
    fn issue_3315_failed_conversion_falls_back_identically_on_both_paths() {
        let mut broken_bmp = vec![0u8; 64];
        broken_bmp[..2].copy_from_slice(b"BM");
        let image = ImageNode::new(9, Some(broken_bmp.clone()));

        let (mime, bytes) = emitted_image_bytes(&broken_bmp, is_watermark_image(&image));
        assert_eq!(mime, "image/bmp", "변환 실패 시 감지한 원본 mime 을 쓴다");
        assert_eq!(
            bytes.as_ref(),
            &broken_bmp[..],
            "변환 실패 시 원본 바이트를 쓴다"
        );

        assert_paths_agree("손상 BMP", &image);
    }

    /// 신원 키를 낼 수 없는 그림은 키 조회로 되찾을 수 없다 — 그래서 생략 대상이 아니다.
    #[test]
    fn issue_3315_synthetic_images_have_no_key() {
        let png = encoded(2, 2, ImageFormat::Png, |_, _| [0, 0, 0]);
        // bin_data_id == 0 — 문서 BinData 에 대응하지 않는 합성 그림.
        assert!(source_image_key(EPOCH, &ImageNode::new(0, Some(png))).is_none());
        // 바이트를 내보내지 않는 op 도 키가 없다.
        assert!(source_image_key(EPOCH, &ImageNode::new(1, None)).is_none());
    }
}

//! Adobe Illustrator EPS(PostScript) 아트워크를 SVG 로 옮긴다.
//!
//! HWP 문서의 `BinData` 에는 확장자 `.ai`/`.eps` 로 **텍스트 PostScript** 가 그대로 들어 있는
//! 경우가 있다. 브라우저·resvg·skia 중 무엇도 PostScript 를 디코드하지 못하므로, 변환이 없으면
//! 그 그림 자리는 빈칸이 된다(#5513).
//!
//! 범용 PostScript 인터프리터가 아니다. AI 가 자기 프롤로그(`Adobe_IllustratorA_AI5` 등)에
//! 정의해 두는 **아트워크 연산자**만 읽는다. 구현 기준은
//! *Adobe Illustrator File Format Specification (23 February 1998, AI 7.0)* 이고, 연산자별
//! 피연산자 순서·의미는 그 문서를 따랐다. 프롤로그가 실제로 하는 일과 사양이 갈리는 자리
//! (예: 사용자 색 `x` 의 마지막 피연산자는 tint 가 아니라 **gray**, 농도는 `1 - gray`)는
//! 파일이 들고 다니는 프롤로그 정의로 확인해 사양 쪽 서술을 따랐다.
//!
//! ## 옮기는 것
//!
//! - 경로: `m l L c C v V y Y h H`, 칠하기 `F f S s B b N n`, 오려내기 `W`, 복합경로 `*u *U`,
//!   묶음 `u U`, 마스크 묶음 `q Q`·다층 마스크 `Mb Md MB`
//! - 색: 회색 `g G`, CMYK `k K`, 별색 `x X`, RGB `Xa XA`, 일반 사용자색 `Xx XX`
//! - 선: `w J j M d`, 칠 규칙 `XR`
//! - 그러데이션: 설정부의 `Bd`/`%_Bs`/`BD` 정의와 본문의 `Bb`/`Bh`/`Bg`/`Bm`/`Bc`/`BB` 인스턴스
//!   → SVG `linearGradient`/`radialGradient` (중간점 `midPoint` 는 보간 정지점을 하나 더 넣어 근사)
//! - 무늬: 설정부의 `%AI3_BeginPattern` 타일과 본문의 `p`/`P` → SVG `pattern`
//! - 래스터: `XI` (16진 ASCII, 회색/RGB/CMYK/이미지마스크) → PNG `image`
//! - 글자: `To Tp TP TO`, `Tf Tr Ta Tl Ts Tz Tt Tc Tw`, `Tm Td T*`, `Tx Tj TX`
//! - 레이어 `Lb Ln LB`, 안내선 `*`, 팔레트 `Pb…PB`·비인쇄 `Np` 건너뛰기
//!
//! ## 옮기지 못하는 것
//!
//! - `XF`/`XG` 연결 이미지 — 픽셀이 파일 밖에 있다.
//! - 이진(`bin-ascii=1`) 래스터 — EPS 는 16진으로 쓰는 것이 보통이라 우선순위를 뒤로 두었다.
//! - 글자 모양은 시스템 글꼴로 대체한다. AI 가 쓰던 글꼴이 없으면 자간이 달라진다.
//! - AI 9 이후의 **네이티브 `.ai`** 는 PostScript 가 아니라 PDF 컨테이너다(`%PDF` 로 시작).
//!   그 바이트는 애초에 `application/postscript` 로 판정되지 않으므로 여기 오지 않는다.
//!
//! 그릴 것이 하나도 없으면 `None` 을 돌려 호출부가 "그림 없음" 표시로 내려가게 한다.

use std::collections::HashMap;
use std::io::Cursor;

/// 변환 결과 SVG 의 상한 — 병적인 입력이 메모리를 먹는 것을 막는다.
const MAX_SVG_BYTES: usize = 16 * 1024 * 1024;
/// 해석할 토큰 수 상한.
const MAX_TOKENS: usize = 8_000_000;
/// 래스터 한 장의 픽셀 수 상한.
const MAX_IMAGE_PIXELS: usize = 32 * 1024 * 1024;

/// EPS 바이트가 Adobe Illustrator 아트워크면 SVG 로 옮긴다.
pub fn convert_ai_artwork_to_svg(data: &[u8]) -> Option<Vec<u8>> {
    if !data.starts_with(b"%!PS") {
        return None;
    }
    let bbox = find_bounding_box(data)?;
    let encoding = platform_encoding(data);
    let setup = setup_slice(data);
    let gradients = parse_gradients(setup);
    let patterns = parse_patterns(setup, encoding);

    let mut interp = Interp::new(bbox, encoding, gradients, patterns);
    interp.run(artwork_slice(data));
    interp.finish()
}

// ─────────────────────────────────────────────────────────────────────────────
// 헤더 · 구간
// ─────────────────────────────────────────────────────────────────────────────

/// `%%BoundingBox` / `%%HiResBoundingBox` 주석에서 경계 상자를 읽는다.
///
/// `(atend)` 인 파일은 트레일러에 실제 값이 있으므로 **파일 전체**에서 숫자가 있는 선언을
/// 찾는다. 고해상도 상자가 있으면 그쪽이 더 정확하다.
fn find_bounding_box(data: &[u8]) -> Option<[f64; 4]> {
    let mut plain = None;
    let mut hires = None;
    for line in data.split(|&b| b == b'\n' || b == b'\r') {
        if let Some(rest) = line.strip_prefix(b"%%HiResBoundingBox:") {
            if let Some(v) = parse_four(rest) {
                hires = Some(v);
            }
        } else if let Some(rest) = line.strip_prefix(b"%%BoundingBox:") {
            if let Some(v) = parse_four(rest) {
                plain = Some(v);
            }
        }
    }
    let bbox = hires.or(plain)?;
    if bbox[2] - bbox[0] <= 0.0 || bbox[3] - bbox[1] <= 0.0 {
        return None;
    }
    Some(bbox)
}

fn parse_four(rest: &[u8]) -> Option<[f64; 4]> {
    let text = String::from_utf8_lossy(rest);
    let mut it = text.split_whitespace();
    let mut out = [0.0f64; 4];
    for slot in out.iter_mut() {
        *slot = it.next()?.parse::<f64>().ok()?;
    }
    Some(out)
}

/// 글자 문자열의 바이트 인코딩 — AI 는 **만든 플랫폼의 코드페이지**로 쓴다.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Encoding {
    MacRoman,
    WinAnsi,
}

fn platform_encoding(data: &[u8]) -> Encoding {
    let head = &data[..data.len().min(4096)];
    let has = |needle: &[u8]| head.windows(needle.len()).any(|w| w == needle);
    if has(b"Macintosh") || has(b"(Apple)") {
        Encoding::MacRoman
    } else {
        Encoding::WinAnsi
    }
}

/// 설정부(프롤로그 뒤 ~ `%%EndSetup`) — 그러데이션·무늬 정의가 사는 곳.
fn setup_slice(data: &[u8]) -> &[u8] {
    let start = find_marker(data, b"%%BeginSetup")
        .or_else(|| find_marker(data, b"%%EndProlog"))
        .unwrap_or(0);
    let end = find_marker(&data[start..], b"%%EndSetup")
        .map(|off| start + off)
        .unwrap_or(data.len());
    &data[start..end]
}

/// 아트워크 구간만 잘라낸다.
///
/// 프롤로그(프로시저 정의)는 해석하지 않는다 — 정의 안의 이름을 연산자로 읽으면 있지도 않은
/// 도형이 생긴다. 시작은 `%%EndSetup`(없으면 `%%EndProlog`), 끝은 `%%Trailer` 다.
fn artwork_slice(data: &[u8]) -> &[u8] {
    let start = find_marker(data, b"%%EndSetup")
        .or_else(|| find_marker(data, b"%%EndProlog"))
        .unwrap_or(0);
    let end = find_marker(&data[start..], b"%%Trailer")
        .map(|off| start + off)
        .unwrap_or(data.len());
    &data[start..end]
}

fn find_marker(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + needle.len())
}

// ─────────────────────────────────────────────────────────────────────────────
// 색
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
struct Rgb(u8, u8, u8);

impl Rgb {
    fn black() -> Self {
        Rgb(0, 0, 0)
    }

    fn from_unit(r: f64, g: f64, b: f64) -> Self {
        let q = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        Rgb(q(r), q(g), q(b))
    }

    fn from_gray(g: f64) -> Self {
        Self::from_unit(g, g, g)
    }

    /// CMYK → RGB. AI 프롤로그가 `setcmykcolor` 를 정의하는 식 그대로다(사양 5.4.1):
    /// `red = 1 - min(1, cyan + black)`. 곱셈식(`(1-c)(1-k)`)을 쓰면 AI 자신의 화면과 색이
    /// 어긋난다.
    fn from_cmyk(c: f64, m: f64, y: f64, k: f64) -> Self {
        let f = |v: f64| 1.0 - (v.clamp(0.0, 1.0) + k.clamp(0.0, 1.0)).min(1.0);
        Self::from_unit(f(c), f(m), f(y))
    }

    /// 별색 농도 — 사양: "A custom color's CMYK values are each multiplied by the tint value".
    fn from_cmyk_tint(c: f64, m: f64, y: f64, k: f64, tint: f64) -> Self {
        let t = tint.clamp(0.0, 1.0);
        Self::from_cmyk(c * t, m * t, y * t, k * t)
    }

    /// RGB 별색 농도 — 농도가 낮을수록 흰색에 가까워진다.
    fn from_rgb_tint(r: f64, g: f64, b: f64, tint: f64) -> Self {
        let t = tint.clamp(0.0, 1.0);
        Self::from_unit(
            1.0 - (1.0 - r) * t,
            1.0 - (1.0 - g) * t,
            1.0 - (1.0 - b) * t,
        )
    }

    fn mix(self, other: Rgb) -> Rgb {
        let m = |a: u8, b: u8| (((a as u16) + (b as u16)) / 2) as u8;
        Rgb(m(self.0, other.0), m(self.1, other.1), m(self.2, other.2))
    }

    fn css(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 행렬 (PostScript `[a b c d tx ty]`)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
struct Mat {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Mat {
    fn identity() -> Self {
        Mat {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    fn from_slice(v: &[f64]) -> Self {
        Mat {
            a: v.first().copied().unwrap_or(1.0),
            b: v.get(1).copied().unwrap_or(0.0),
            c: v.get(2).copied().unwrap_or(0.0),
            d: v.get(3).copied().unwrap_or(1.0),
            e: v.get(4).copied().unwrap_or(0.0),
            f: v.get(5).copied().unwrap_or(0.0),
        }
    }

    /// `self` 를 나중에 적용한다 — `self ∘ inner`.
    fn concat(self, inner: Mat) -> Mat {
        Mat {
            a: self.a * inner.a + self.c * inner.b,
            b: self.b * inner.a + self.d * inner.b,
            c: self.a * inner.c + self.c * inner.d,
            d: self.b * inner.c + self.d * inner.d,
            e: self.a * inner.e + self.c * inner.f + self.e,
            f: self.b * inner.e + self.d * inner.f + self.f,
        }
    }

    fn svg(&self) -> String {
        format!(
            "matrix({} {} {} {} {} {})",
            fmt(self.a),
            fmt(self.b),
            fmt(self.c),
            fmt(self.d),
            fmt(self.e),
            fmt(self.f)
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 어휘 분석
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Name(String),
    /// `/이름` 리터럴 — 글꼴 이름이 이 형태로 온다.
    Lit(String),
    /// `(...)` 문자열의 원시 바이트 (이스케이프 해석 후).
    Str(Vec<u8>),
    ArrayOpen,
    ArrayClose,
    /// `[ … ]` 로 닫힌 수 배열 — 파선 패턴·변환 행렬이 이 형태로 온다.
    Array(Vec<f64>),
}

struct Lexer<'a> {
    s: &'a [u8],
    i: usize,
}

fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n' | 0 | 0x0c)
}

fn is_delim(b: u8) -> bool {
    is_space(b)
        || matches!(
            b,
            b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
        )
}

impl<'a> Lexer<'a> {
    fn new(s: &'a [u8]) -> Self {
        Self { s, i: 0 }
    }

    fn line_at(&self, at: usize) -> &'a [u8] {
        let end = self.s[at..]
            .iter()
            .position(|&b| b == b'\n' || b == b'\r')
            .map(|p| at + p)
            .unwrap_or(self.s.len());
        &self.s[at..end]
    }

    fn skip_line(&mut self) {
        while self.i < self.s.len() && self.s[self.i] != b'\n' && self.s[self.i] != b'\r' {
            self.i += 1;
        }
    }

    /// `%%BeginData`/`%%BeginBinary` 뒤의 이진 덩어리는 토큰이 아니다 — 끝 주석까지 건너뛴다.
    fn skip_until_comment(&mut self, end_marker: &[u8]) {
        while self.i < self.s.len() {
            let line = self.line_at(self.i);
            let done = line.starts_with(end_marker);
            self.skip_line();
            self.i = (self.i + 1).min(self.s.len());
            if done {
                return;
            }
        }
    }

    /// 래스터 픽셀은 `%` 로 시작하는 줄에 16진으로 실린다 — 그 줄들만 모은다.
    fn take_hex_comment_lines(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let save = self.i;
            while self.i < self.s.len() && is_space(self.s[self.i]) {
                self.i += 1;
            }
            if self.i >= self.s.len() || self.s[self.i] != b'%' {
                self.i = save;
                return out;
            }
            let line = self.line_at(self.i);
            let body = &line[1..];
            if body.starts_with(b"AI") || body.starts_with(b"%") || body.is_empty() {
                // `%AI5_EndRaster` 같은 구조 주석에서 멈춘다.
                self.i = save;
                return out;
            }
            if !body.iter().all(|b| b.is_ascii_hexdigit() || is_space(*b)) {
                self.i = save;
                return out;
            }
            out.extend(body.iter().copied().filter(|b| b.is_ascii_hexdigit()));
            self.skip_line();
        }
    }

    fn next(&mut self) -> Option<Tok> {
        loop {
            while self.i < self.s.len() && is_space(self.s[self.i]) {
                self.i += 1;
            }
            if self.i >= self.s.len() {
                return None;
            }
            if self.s[self.i] == b'%' {
                let line = self.line_at(self.i);
                if line.starts_with(b"%%BeginData") {
                    self.skip_line();
                    self.skip_until_comment(b"%%EndData");
                    continue;
                }
                if line.starts_with(b"%%BeginBinary") {
                    self.skip_line();
                    self.skip_until_comment(b"%%EndBinary");
                    continue;
                }
                // 구조 주석 중 흐름을 바꾸는 것은 이름 토큰으로 올려 보낸다.
                if line.starts_with(b"%AI5_BeginPalette") || line.starts_with(b"%AI3_BeginPalette")
                {
                    self.skip_line();
                    self.skip_until_comment(b"%AI5_EndPalette");
                    continue;
                }
                self.skip_line();
                continue;
            }
            break;
        }

        let b = self.s[self.i];
        match b {
            b'(' => {
                self.i += 1;
                Some(Tok::Str(self.read_string()))
            }
            b'<' => {
                self.i += 1;
                let start = self.i;
                while self.i < self.s.len() && self.s[self.i] != b'>' {
                    self.i += 1;
                }
                let hex = &self.s[start..self.i];
                self.i = (self.i + 1).min(self.s.len());
                Some(Tok::Str(hex_decode(hex)))
            }
            b'[' => {
                self.i += 1;
                Some(Tok::ArrayOpen)
            }
            b']' => {
                self.i += 1;
                Some(Tok::ArrayClose)
            }
            b'{' | b'}' | b')' | b'>' => {
                self.i += 1;
                Some(Tok::Name(String::from_utf8_lossy(&[b]).into_owned()))
            }
            b'/' => {
                self.i += 1;
                let start = self.i;
                while self.i < self.s.len() && !is_delim(self.s[self.i]) {
                    self.i += 1;
                }
                Some(Tok::Lit(
                    String::from_utf8_lossy(&self.s[start..self.i]).into_owned(),
                ))
            }
            _ => {
                let start = self.i;
                while self.i < self.s.len() && !is_delim(self.s[self.i]) {
                    self.i += 1;
                }
                if self.i == start {
                    self.i += 1;
                    return self.next();
                }
                let text = String::from_utf8_lossy(&self.s[start..self.i]).into_owned();
                match text.parse::<f64>() {
                    Ok(v) if v.is_finite() => Some(Tok::Num(v)),
                    _ => Some(Tok::Name(text)),
                }
            }
        }
    }

    /// `(` 를 지난 상태에서 문자열 본문을 읽는다. PostScript 이스케이프를 푼다.
    fn read_string(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut depth = 1;
        while self.i < self.s.len() {
            let b = self.s[self.i];
            match b {
                b'\\' => {
                    self.i += 1;
                    if self.i >= self.s.len() {
                        break;
                    }
                    let e = self.s[self.i];
                    match e {
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'\n' | b'\r' => {}
                        b'0'..=b'7' => {
                            let mut v = 0u32;
                            let mut n = 0;
                            while n < 3
                                && self.i < self.s.len()
                                && (b'0'..=b'7').contains(&self.s[self.i])
                            {
                                v = v * 8 + (self.s[self.i] - b'0') as u32;
                                self.i += 1;
                                n += 1;
                            }
                            self.i -= 1;
                            out.push((v & 0xff) as u8);
                        }
                        other => out.push(other),
                    }
                    self.i += 1;
                }
                b'(' => {
                    depth += 1;
                    out.push(b);
                    self.i += 1;
                }
                b')' => {
                    depth -= 1;
                    self.i += 1;
                    if depth == 0 {
                        break;
                    }
                    out.push(b);
                }
                _ => {
                    out.push(b);
                    self.i += 1;
                }
            }
        }
        out
    }
}

fn hex_decode(src: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len() / 2);
    let mut hi: Option<u8> = None;
    for &b in src {
        let v = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => continue,
        };
        match hi {
            None => hi = Some(v),
            Some(h) => {
                out.push((h << 4) | v);
                hi = None;
            }
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// 그러데이션 정의 (설정부)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct GradientStop {
    color: Rgb,
    /// 0~1 로 정규화한 램프 위치.
    offset: f64,
    /// 다음 정지점까지의 중간점(13~87%). SVG 에 없는 개념이라 보간 정지점으로 근사한다.
    mid: f64,
}

#[derive(Clone, Debug)]
struct GradientDef {
    radial: bool,
    stops: Vec<GradientStop>,
}

/// 설정부의 `%AI5_BeginGradient` 블록을 읽는다.
///
/// 정지점 줄은 `colorSpec colorStyle midPoint rampPoint %_Bs` 인데 연산자 이름이 **주석 안**에
/// 있다(`%_Bs`). 그래서 일반 어휘 분석으로는 잡히지 않고 이 전용 스캐너가 필요하다.
fn parse_gradients(setup: &[u8]) -> HashMap<String, GradientDef> {
    let mut out = HashMap::new();
    let text = String::from_utf8_lossy(setup);
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if !trimmed.ends_with(" Bd") {
            continue;
        }
        // `(name) type nColors Bd`
        let Some((name, rest)) = split_ps_string(trimmed) else {
            continue;
        };
        let nums: Vec<f64> = rest
            .split_whitespace()
            .filter_map(|t| t.parse::<f64>().ok())
            .collect();
        let radial = nums.first().copied().unwrap_or(0.0) != 0.0;
        let mut stops = Vec::new();
        for body in lines.by_ref() {
            let body = body.trim();
            if body.starts_with("BD") {
                break;
            }
            if let Some(head) = body.strip_suffix("%_Bs").map(str::trim) {
                if let Some(stop) = parse_gradient_stop(head) {
                    stops.push(stop);
                }
            }
        }
        if stops.len() >= 2 {
            stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
            out.insert(name, GradientDef { radial, stops });
        }
    }
    out
}

/// `colorSpec colorStyle midPoint rampPoint` — 사양 표 7·8.
fn parse_gradient_stop(head: &str) -> Option<GradientStop> {
    let (name, _) = split_ps_string(head).unzip();
    let toks: Vec<&str> = head
        .split_whitespace()
        .filter(|t| !t.starts_with('(') && !t.ends_with(')'))
        .collect();
    let nums: Vec<f64> = toks.iter().filter_map(|t| t.parse::<f64>().ok()).collect();
    if nums.len() < 4 {
        return None;
    }
    let ramp = nums[nums.len() - 1] / 100.0;
    let mid = nums[nums.len() - 2] / 100.0;
    let style = nums[nums.len() - 3];
    let spec = &nums[..nums.len() - 3];
    let _ = name;
    let color = match style as i32 {
        0 => Rgb::from_gray(*spec.first()?),
        1 if spec.len() >= 4 => Rgb::from_cmyk(spec[0], spec[1], spec[2], spec[3]),
        // RGB(2): `cyan magenta yellow black red green blue` — 뒤의 RGB 가 실제 색이다.
        2 if spec.len() >= 7 => Rgb::from_unit(spec[4], spec[5], spec[6]),
        // CMYK 별색(3): `cyan magenta yellow black (name) tint` — 문자열은 이미 걸러졌다.
        3 if spec.len() >= 5 => Rgb::from_cmyk_tint(spec[0], spec[1], spec[2], spec[3], spec[4]),
        // RGB 별색(4): `cyan magenta yellow black red green blue (name) tint type`
        4 if spec.len() >= 9 => Rgb::from_rgb_tint(spec[4], spec[5], spec[6], spec[7]),
        _ => Rgb::from_cmyk(
            *spec.first()?,
            spec.get(1).copied().unwrap_or(0.0),
            spec.get(2).copied().unwrap_or(0.0),
            spec.get(3).copied().unwrap_or(0.0),
        ),
    };
    Some(GradientStop {
        color,
        offset: ramp.clamp(0.0, 1.0),
        mid: if (0.0..=1.0).contains(&mid) { mid } else { 0.5 },
    })
}

/// 맨 앞 `(...)` 문자열과 나머지를 나눈다.
fn split_ps_string(s: &str) -> Option<(String, &str)> {
    let start = s.find('(')?;
    let mut depth = 0;
    let bytes = s.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((s[start + 1..i].to_string(), &s[i + 1..]));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// 무늬 정의 (설정부)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct PatternDef {
    bbox: [f64; 4],
    /// 타일 그림의 원본 조각 — 쓰일 때 하위 해석기로 그린다.
    tile: Vec<u8>,
}

/// `%AI3_BeginPattern: (name)` 블록.
///
/// 타일 그림은 `(...) @` (칠 스타일)와 `(...) &` (그림) **문자열 안에** 들어 있다. 문자열을
/// 이어 붙이면 그대로 아트워크 조각이 되므로, 쓰일 때 같은 해석기로 한 번 더 돌린다.
fn parse_patterns(setup: &[u8], _encoding: Encoding) -> HashMap<String, PatternDef> {
    let mut out = HashMap::new();
    let mut lex = Lexer::new(setup);
    let mut pending: Vec<Tok> = Vec::new();
    while let Some(tok) = lex.next() {
        match &tok {
            Tok::Name(n) if n == "E" => {
                // `(name) llx lly urx ury [ (style) @ (art) & … ] E`
                let mut name = None;
                let mut nums = Vec::new();
                let mut tile = Vec::new();
                for t in &pending {
                    match t {
                        Tok::Str(s) => {
                            if name.is_none() && nums.is_empty() {
                                name = Some(String::from_utf8_lossy(s).into_owned());
                            } else {
                                tile.extend_from_slice(s);
                                tile.push(b'\n');
                            }
                        }
                        Tok::Num(v) => {
                            if name.is_some() && nums.len() < 4 {
                                nums.push(*v);
                            }
                        }
                        _ => {}
                    }
                }
                if let (Some(name), true) = (name, nums.len() == 4) {
                    if !tile.is_empty() {
                        out.insert(
                            name,
                            PatternDef {
                                bbox: [nums[0], nums[1], nums[2], nums[3]],
                                tile,
                            },
                        );
                    }
                }
                pending.clear();
            }
            // 그러데이션 정의가 섞여 있으므로 무늬 시작 표시에서 모은 것을 버린다.
            Tok::Name(n) if n == "Bd" || n == "BD" => pending.clear(),
            _ => {
                pending.push(tok);
                if pending.len() > 4096 {
                    pending.drain(..2048);
                }
            }
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// 해석기
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
enum Source {
    Color(Rgb),
    Gradient(String),
    Pattern(String),
}

impl Source {
    fn color(&self) -> Rgb {
        match self {
            Source::Color(c) => *c,
            _ => Rgb::black(),
        }
    }
}

#[derive(Clone, Debug)]
struct GState {
    fill: Source,
    stroke: Source,
    line_width: f64,
    cap: u8,
    join: u8,
    miter: f64,
    dash: Option<String>,
    even_odd: bool,
    /// 무늬 인스턴스 변환 (`p`/`P` 의 좌표 변환).
    fill_pattern_matrix: Mat,
}

impl GState {
    fn new() -> Self {
        Self {
            fill: Source::Color(Rgb::black()),
            stroke: Source::Color(Rgb::black()),
            line_width: 1.0,
            cap: 0,
            join: 0,
            miter: 4.0,
            dash: None,
            even_odd: false,
            fill_pattern_matrix: Mat::identity(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct PaintKind {
    fill: bool,
    stroke: bool,
}

/// 그러데이션 인스턴스 기하 (`Bg`/`Bh`).
#[derive(Clone, Debug)]
struct GradientInstance {
    name: String,
    origin: (f64, f64),
    angle: f64,
    length: f64,
    matrix: Mat,
    hilight_angle: f64,
    hilight_length: f64,
}

#[derive(Clone, Debug)]
struct TextState {
    matrix: Mat,
    font: String,
    size: f64,
    render: i32,
    align: i32,
    leading: f64,
    rise: f64,
    hscale: f64,
    tracking: f64,
    char_space: f64,
    word_space: f64,
}

impl TextState {
    fn new() -> Self {
        Self {
            matrix: Mat::identity(),
            font: String::new(),
            size: 12.0,
            render: 0,
            align: 0,
            leading: 0.0,
            rise: 0.0,
            hscale: 100.0,
            tracking: 0.0,
            char_space: 0.0,
            word_space: 0.0,
        }
    }
}

struct Interp {
    bbox: [f64; 4],
    encoding: Encoding,
    gradients: HashMap<String, GradientDef>,
    patterns: HashMap<String, PatternDef>,

    defs: String,
    out: String,
    stack: Vec<Tok>,
    array: Option<Vec<f64>>,

    gs: GState,
    gs_stack: Vec<GState>,

    path: String,
    cur: (f64, f64),
    start: (f64, f64),

    compound: i32,
    pending_paint: Option<PaintKind>,
    clip_pending: bool,
    open_groups: usize,
    group_stack: Vec<usize>,
    def_seq: u32,

    gradient_instance: Option<GradientInstance>,
    in_gradient: bool,
    gradient_paint: Option<PaintKind>,
    gradient_path: String,

    text: TextState,
    text_stack: Vec<TextState>,

    skip_layer_depth: i32,
    painted: usize,
    tokens: usize,
    /// 무늬 타일을 그리는 하위 해석 중인가 — 재귀를 한 겹으로 막는다.
    nested: bool,
}

impl Interp {
    fn new(
        bbox: [f64; 4],
        encoding: Encoding,
        gradients: HashMap<String, GradientDef>,
        patterns: HashMap<String, PatternDef>,
    ) -> Self {
        Self {
            bbox,
            encoding,
            gradients,
            patterns,
            defs: String::new(),
            out: String::new(),
            stack: Vec::new(),
            array: None,
            gs: GState::new(),
            gs_stack: Vec::new(),
            path: String::new(),
            cur: (0.0, 0.0),
            start: (0.0, 0.0),
            compound: 0,
            pending_paint: None,
            clip_pending: false,
            open_groups: 0,
            group_stack: Vec::new(),
            def_seq: 0,
            gradient_instance: None,
            in_gradient: false,
            gradient_paint: None,
            gradient_path: String::new(),
            text: TextState::new(),
            text_stack: Vec::new(),
            skip_layer_depth: 0,
            painted: 0,
            tokens: 0,
            nested: false,
        }
    }

    fn run(&mut self, artwork: &[u8]) {
        let mut lex = Lexer::new(artwork);
        while let Some(tok) = lex.next() {
            self.tokens += 1;
            if self.tokens > MAX_TOKENS || self.out.len() + self.defs.len() > MAX_SVG_BYTES {
                return;
            }
            match tok {
                Tok::Name(name) => self.op(&name, &mut lex),
                Tok::ArrayOpen => self.array = Some(Vec::new()),
                Tok::ArrayClose => {
                    let arr = self.array.take().unwrap_or_default();
                    self.push(Tok::Array(arr));
                }
                Tok::Num(v) if self.array.is_some() => {
                    if let Some(arr) = self.array.as_mut() {
                        arr.push(v);
                    }
                }
                other => self.push(other),
            }
        }
    }

    // ── 피연산자 읽기 ────────────────────────────────────────────────────
    //
    // 연산자 하나가 끝나면 스택을 통째로 비우므로(PostScript 스택을 흉내 내지 않는다),
    // 꺼내지 않고 **꼬리에서 읽기만** 한다. 뽑아 버리면 뒤에 오는 문자열 피연산자(색 이름·
    // 글꼴 이름)가 같이 사라진다.
    fn push(&mut self, tok: Tok) {
        self.stack.push(tok);
        if self.stack.len() > 512 {
            self.stack.drain(..256);
        }
    }

    fn nums(&self) -> Vec<f64> {
        self.stack
            .iter()
            .filter_map(|t| match t {
                Tok::Num(v) => Some(*v),
                _ => None,
            })
            .collect()
    }

    /// 마지막 n 개의 수 (모자라면 앞을 0 으로 채운다).
    fn tail(&self, n: usize) -> Vec<f64> {
        let nums = self.nums();
        if nums.len() >= n {
            nums[nums.len() - n..].to_vec()
        } else {
            let mut out = vec![0.0; n - nums.len()];
            out.extend(nums);
            out
        }
    }

    fn num(&self) -> f64 {
        self.tail(1)[0]
    }

    fn last_str(&self) -> Option<String> {
        self.stack.iter().rev().find_map(|t| match t {
            Tok::Str(s) => Some(String::from_utf8_lossy(s).into_owned()),
            Tok::Lit(s) => Some(s.clone()),
            _ => None,
        })
    }

    /// 글자 문자열은 플랫폼 코드페이지라 UTF-8 로 읽으면 안 된다 — 원시 바이트로 준다.
    fn last_str_bytes(&self) -> Option<Vec<u8>> {
        self.stack.iter().rev().find_map(|t| match t {
            Tok::Str(s) => Some(s.clone()),
            _ => None,
        })
    }

    fn last_array(&self) -> Option<Vec<f64>> {
        self.stack.iter().rev().find_map(|t| match t {
            Tok::Array(v) => Some(v.clone()),
            _ => None,
        })
    }

    // ── 연산자 ──────────────────────────────────────────────────────────
    fn op(&mut self, name: &str, lex: &mut Lexer<'_>) {
        if self.skip_layer_depth > 0 {
            match name {
                "Lb" => self.skip_layer_depth += 1,
                "LB" => self.skip_layer_depth -= 1,
                _ => {}
            }
            self.stack.clear();
            return;
        }
        match name {
            // ── 경로 ─────────────────────────────────────────────────────
            "m" => {
                let p = self.tail(2);
                self.move_to(p[0], p[1]);
            }
            "l" | "L" => {
                let p = self.tail(2);
                self.line_to(p[0], p[1]);
            }
            "c" | "C" => {
                let p = self.tail(6);
                self.curve_to(p[0], p[1], p[2], p[3], p[4], p[5]);
            }
            // `v`/`V`: 첫 제어점이 현재점.
            "v" | "V" => {
                let p = self.tail(4);
                let (cx, cy) = self.cur;
                self.curve_to(cx, cy, p[0], p[1], p[2], p[3]);
            }
            // `y`/`Y`: 둘째 제어점이 끝점.
            "y" | "Y" => {
                let p = self.tail(4);
                self.curve_to(p[0], p[1], p[2], p[3], p[2], p[3]);
            }
            // `h`/`H` 는 경로를 **소비하지 않는다**(마스크를 세울 때 쓴다).
            "h" => self.close_path(),
            "H" => {}

            // ── 칠하기 ───────────────────────────────────────────────────
            "F" => self.paint(PaintKind {
                fill: true,
                stroke: false,
            }),
            "f" => {
                self.close_path();
                self.paint(PaintKind {
                    fill: true,
                    stroke: false,
                });
            }
            "S" => self.paint(PaintKind {
                fill: false,
                stroke: true,
            }),
            "s" => {
                self.close_path();
                self.paint(PaintKind {
                    fill: false,
                    stroke: true,
                });
            }
            "B" => self.paint(PaintKind {
                fill: true,
                stroke: true,
            }),
            "b" => {
                self.close_path();
                self.paint(PaintKind {
                    fill: true,
                    stroke: true,
                });
            }
            "N" => self.paint(PaintKind {
                fill: false,
                stroke: false,
            }),
            "n" => {
                self.close_path();
                self.paint(PaintKind {
                    fill: false,
                    stroke: false,
                });
            }
            "W" => self.clip_pending = true,

            // ── 복합 경로·묶음·마스크 ────────────────────────────────────
            "*u" => self.compound += 1,
            "*U" => {
                self.compound = (self.compound - 1).max(0);
                if self.compound == 0 {
                    if let Some(paint) = self.pending_paint.take() {
                        self.emit_path(paint);
                    } else {
                        self.path.clear();
                    }
                }
            }
            "u" | "U" => {}
            // 마스크 묶음 — `W` 로 연 오려내기 그룹을 짝이 되는 `Q` 에서 닫는다.
            "q" | "Mb" => {
                self.group_stack.push(self.open_groups);
                self.gs_stack.push(self.gs.clone());
            }
            "Q" | "MB" => {
                if let Some(depth) = self.group_stack.pop() {
                    while self.open_groups > depth {
                        self.out.push_str("</g>\n");
                        self.open_groups -= 1;
                    }
                }
                if let Some(gs) = self.gs_stack.pop() {
                    self.gs = gs;
                }
            }
            "Md" => {}

            // 안내선 — `(N) * … (N) *`. 그리지 않고 경로를 버린다(사양 6.1).
            "*" => {
                self.path.clear();
                self.clip_pending = false;
            }

            // ── 색 ───────────────────────────────────────────────────────
            "g" => {
                let v = self.num();
                self.gs.fill = Source::Color(Rgb::from_gray(v));
            }
            "G" => {
                let v = self.num();
                self.gs.stroke = Source::Color(Rgb::from_gray(v));
            }
            "k" => {
                let p = self.tail(4);
                self.gs.fill = Source::Color(Rgb::from_cmyk(p[0], p[1], p[2], p[3]));
            }
            "K" => {
                let p = self.tail(4);
                self.gs.stroke = Source::Color(Rgb::from_cmyk(p[0], p[1], p[2], p[3]));
            }
            // 별색: `c m y k (name) gray x`. 마지막 피연산자는 **gray** 이고 농도는 `1-gray` 다.
            "x" | "X" => {
                let p = self.tail(5);
                let color = Rgb::from_cmyk_tint(p[0], p[1], p[2], p[3], 1.0 - p[4]);
                if name == "x" {
                    self.gs.fill = Source::Color(color);
                } else {
                    self.gs.stroke = Source::Color(color);
                }
            }
            "Xa" | "XA" => {
                let p = self.tail(3);
                let color = Rgb::from_unit(p[0], p[1], p[2]);
                if name == "Xa" {
                    self.gs.fill = Source::Color(color);
                } else {
                    self.gs.stroke = Source::Color(color);
                }
            }
            // 일반 사용자색: `comp1 … compn (name) tint type`. type 0=CMYK, 1=RGB.
            "Xx" | "XX" => {
                let all = self.nums();
                let t = self.tail(2);
                let (tint, kind) = (t[0], t[1]);
                let want = if kind == 1.0 { 3 } else { 4 };
                let comps = if all.len() >= want + 2 {
                    all[all.len() - want - 2..all.len() - 2].to_vec()
                } else {
                    vec![0.0; want]
                };
                let color = if kind == 1.0 {
                    Rgb::from_rgb_tint(comps[0], comps[1], comps[2], tint)
                } else {
                    Rgb::from_cmyk_tint(comps[0], comps[1], comps[2], comps[3], tint)
                };
                if name == "Xx" {
                    self.gs.fill = Source::Color(color);
                } else {
                    self.gs.stroke = Source::Color(color);
                }
            }
            // 무늬: `(name) px py sx sy angle rf r k ka [a b c d tx ty] p`
            "p" | "P" => {
                let matrix = self
                    .last_array()
                    .filter(|m| m.len() >= 6)
                    .map(|m| Mat::from_slice(&m[m.len() - 6..]))
                    .unwrap_or_else(Mat::identity);
                let pattern = self.last_str().unwrap_or_default();
                if self.patterns.contains_key(&pattern) {
                    if name == "p" {
                        self.gs.fill = Source::Pattern(pattern);
                        self.gs.fill_pattern_matrix = matrix;
                    } else {
                        self.gs.stroke = Source::Pattern(pattern);
                    }
                }
            }

            // ── 선 ───────────────────────────────────────────────────────
            "w" => self.gs.line_width = self.num().max(0.0),
            "J" => self.gs.cap = self.num().clamp(0.0, 2.0) as u8,
            "j" => self.gs.join = self.num().clamp(0.0, 2.0) as u8,
            "M" => self.gs.miter = self.num().max(1.0),
            "d" => {
                let pattern = self.last_array().unwrap_or_default();
                self.gs.dash = if pattern.iter().any(|v| *v > 0.0) {
                    Some(
                        pattern
                            .iter()
                            .map(|v| fmt(*v))
                            .collect::<Vec<_>>()
                            .join(" "),
                    )
                } else {
                    None
                };
            }
            "XR" => self.gs.even_odd = self.num() != 0.0,

            // ── 그러데이션 인스턴스 ──────────────────────────────────────
            "Bb" => {
                self.gs_stack.push(self.gs.clone());
                self.in_gradient = true;
                self.gradient_instance = None;
                self.gradient_paint = None;
                self.gradient_path.clear();
            }
            // `xHilight yHilight angle length Bh`
            "Bh" => {
                let p = self.tail(4);
                if let Some(inst) = self.gradient_instance.as_mut() {
                    inst.hilight_angle = p[2];
                    inst.hilight_length = p[3];
                } else {
                    // Bh 는 Bg 앞에 온다 — 값만 들고 있다가 Bg 에서 합친다.
                    self.gradient_instance = Some(GradientInstance {
                        name: String::new(),
                        origin: (0.0, 0.0),
                        angle: 0.0,
                        length: 0.0,
                        matrix: Mat::identity(),
                        hilight_angle: p[2],
                        hilight_length: p[3],
                    });
                }
            }
            // `flag (name) xOrigin yOrigin angle length a b c d tx ty Bg`
            "Bg" => {
                let nums = self.tail(11);
                let name = self.last_str().unwrap_or_default();
                let (hilight_angle, hilight_length) = self
                    .gradient_instance
                    .as_ref()
                    .map(|g| (g.hilight_angle, g.hilight_length))
                    .unwrap_or((0.0, 0.0));
                self.gs.fill = Source::Gradient(name.clone());
                self.gradient_instance = Some(GradientInstance {
                    name,
                    origin: (nums[1], nums[2]),
                    angle: nums[3],
                    length: nums[4],
                    matrix: Mat::from_slice(&nums[5..11]),
                    hilight_angle,
                    hilight_length,
                });
            }
            // 이미징 보조 — 램프를 실제로 그리는 값이라 SVG 에서는 필요 없다.
            "Bm" | "Bc" | "Xm" | "Bn" => {}
            "BB" => {
                let flag = self.num();
                self.in_gradient = false;
                let paint = self.gradient_paint.take();
                let path = std::mem::take(&mut self.gradient_path);
                if !path.is_empty() {
                    self.path = path;
                    if flag == 2.0 {
                        self.close_path();
                    }
                    let stroke = flag != 0.0;
                    self.emit_path(PaintKind {
                        fill: paint.map(|p| p.fill).unwrap_or(true),
                        stroke,
                    });
                }
                self.gradient_instance = None;
                if let Some(gs) = self.gs_stack.pop() {
                    self.gs = gs;
                }
            }

            // ── 래스터 ───────────────────────────────────────────────────
            "XI" => self.raster_image(lex),
            // 연결 이미지는 픽셀이 파일 밖에 있다 — 그릴 수 없다.
            "XF" | "XG" => {}

            // ── 글자 ─────────────────────────────────────────────────────
            "To" => self.text_stack.push(self.text.clone()),
            "TO" => {
                if let Some(prev) = self.text_stack.pop() {
                    self.text = prev;
                }
            }
            // `a b c d tx ty startPt Tp`
            "Tp" => {
                let p = self.tail(7);
                self.text.matrix = Mat::from_slice(&p[..6]);
            }
            "TP" => {}
            "Tm" => {
                let p = self.tail(6);
                self.text.matrix = Mat::from_slice(&p);
            }
            "Td" => {
                let p = self.tail(2);
                self.text.matrix = self
                    .text
                    .matrix
                    .concat(Mat::from_slice(&[1.0, 0.0, 0.0, 1.0, p[0], p[1]]));
            }
            "T*" => {
                let leading = self.text.leading;
                self.text.matrix = self
                    .text
                    .matrix
                    .concat(Mat::from_slice(&[1.0, 0.0, 0.0, 1.0, 0.0, -leading]));
            }
            "TR" => {}
            "Tr" => self.text.render = self.num() as i32,
            // `/_fontname size ascent descent Tf` — 7.0 이전에는 ascent/descent 가 없다.
            // 글꼴 리터럴 바로 뒤가 크기이므로 첫 수를 쓴다.
            "Tf" => {
                if let Some(size) = self.nums().first().copied().filter(|v| *v > 0.0) {
                    self.text.size = size;
                }
                if let Some(font) = self.last_str().filter(|f| !f.is_empty()) {
                    self.text.font = font;
                }
            }
            "Ta" => self.text.align = self.num() as i32,
            "Tl" => self.text.leading = self.tail(2)[0],
            "Ts" => self.text.rise = self.num(),
            "Tz" => self.text.hscale = self.num(),
            "Tt" => self.text.tracking = self.num(),
            "Tc" => self.text.char_space = self.num(),
            "Tw" => self.text.word_space = self.num(),
            "Tx" | "Tj" => {
                if let Some(s) = self.last_str_bytes() {
                    self.show_text(&s);
                }
            }
            // 넘친 글자는 화면에 나오지 않는다.
            "TX" => {}

            // ── 레이어 ───────────────────────────────────────────────────
            // `visible preview enabled printing dimmed hasMultiLayerMasks colorIndex r g b Lb`
            // AI 프롤로그는 printing·hasMultiLayerMasks 가 둘 다 0 이면 레이어 내용을 버린다.
            "Lb" => {
                let p = self.tail(10);
                if p[3] == 0.0 && p[5] == 0.0 {
                    self.skip_layer_depth = 1;
                }
            }
            "LB" | "Ln" => {}

            // 팔레트·비인쇄 구간은 그림이 아니다.
            "Pb" => self.skip_layer_depth = 1,
            "PB" => self.skip_layer_depth = 0,

            // 값만 버리는 연산자 — 프롤로그가 `pop`·무동작으로 정의한 것들
            // (`A` 잠금, `D` 감김 방향, `O`/`R` 겹쳐찍기, `Ap`, `Ar`, `i` 평탄도 …).
            _ => {}
        }
        self.stack.clear();
        self.array = None;
    }

    // ── 경로 만들기 ─────────────────────────────────────────────────────
    fn move_to(&mut self, x: f64, y: f64) {
        self.path
            .push_str(&format!("M{} {}", fmt(self.sx(x)), fmt(self.sy(y))));
        self.cur = (x, y);
        self.start = (x, y);
    }

    fn line_to(&mut self, x: f64, y: f64) {
        if self.path.is_empty() {
            self.move_to(x, y);
            return;
        }
        self.path
            .push_str(&format!("L{} {}", fmt(self.sx(x)), fmt(self.sy(y))));
        self.cur = (x, y);
    }

    fn curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
        if self.path.is_empty() {
            self.move_to(x1, y1);
        }
        self.path.push_str(&format!(
            "C{} {} {} {} {} {}",
            fmt(self.sx(x1)),
            fmt(self.sy(y1)),
            fmt(self.sx(x2)),
            fmt(self.sy(y2)),
            fmt(self.sx(x3)),
            fmt(self.sy(y3))
        ));
        self.cur = (x3, y3);
    }

    fn close_path(&mut self) {
        if !self.path.is_empty() {
            self.path.push('Z');
            self.cur = self.start;
        }
    }

    /// PostScript 좌표 → SVG 좌표. y 축은 뒤집히고 원점은 경계 상자 왼쪽 아래다.
    fn sx(&self, x: f64) -> f64 {
        x - self.bbox[0]
    }
    fn sy(&self, y: f64) -> f64 {
        self.bbox[3] - y
    }
    /// 같은 사상을 행렬로 — 그러데이션·무늬·글자·래스터가 실어 오는 PostScript 행렬과 합친다.
    fn flip(&self) -> Mat {
        Mat {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: -1.0,
            e: -self.bbox[0],
            f: self.bbox[3],
        }
    }

    fn paint(&mut self, paint: PaintKind) {
        if self.in_gradient {
            // 그러데이션 인스턴스는 `BB` 에서 한 번에 그린다 — 칠 연산자는 그 안에 있다.
            self.gradient_paint = Some(paint);
            self.gradient_path = std::mem::take(&mut self.path);
            return;
        }
        if self.compound > 0 {
            // 복합 경로는 `*U` 에서 한 번에 칠한다 — 부분경로마다 칠하면 구멍이 메워진다.
            if paint.fill || paint.stroke {
                self.pending_paint = Some(paint);
            }
            return;
        }
        self.emit_path(paint);
    }

    fn emit_path(&mut self, paint: PaintKind) {
        if self.path.is_empty() {
            self.clip_pending = false;
            return;
        }
        let d = std::mem::take(&mut self.path);
        if paint.fill || paint.stroke {
            let mut attrs = String::new();
            if paint.fill {
                let fill = self.fill_paint_attr();
                attrs.push_str(&format!(" fill=\"{}\"", fill));
                if self.gs.even_odd {
                    attrs.push_str(" fill-rule=\"evenodd\"");
                }
            } else {
                attrs.push_str(" fill=\"none\"");
            }
            if paint.stroke {
                attrs.push_str(&format!(
                    " stroke=\"{}\" stroke-width=\"{}\"",
                    self.gs.stroke.color().css(),
                    fmt(if self.gs.line_width > 0.0 {
                        self.gs.line_width
                    } else {
                        0.1
                    })
                ));
                if self.gs.cap != 0 {
                    attrs.push_str(if self.gs.cap == 1 {
                        " stroke-linecap=\"round\""
                    } else {
                        " stroke-linecap=\"square\""
                    });
                }
                if self.gs.join != 0 {
                    attrs.push_str(if self.gs.join == 1 {
                        " stroke-linejoin=\"round\""
                    } else {
                        " stroke-linejoin=\"bevel\""
                    });
                }
                if (self.gs.miter - 4.0).abs() > f64::EPSILON {
                    attrs.push_str(&format!(" stroke-miterlimit=\"{}\"", fmt(self.gs.miter)));
                }
                if let Some(dash) = self.gs.dash.clone() {
                    attrs.push_str(&format!(" stroke-dasharray=\"{}\"", dash));
                }
            }
            self.out
                .push_str(&format!("<path d=\"{}\"{}/>\n", d, attrs));
            self.painted += 1;
        }
        if self.clip_pending {
            self.clip_pending = false;
            self.def_seq += 1;
            let id = format!("aiclip{}", self.def_seq);
            self.defs.push_str(&format!(
                "<clipPath id=\"{}\"><path d=\"{}\"{}/></clipPath>\n",
                id,
                d,
                if self.gs.even_odd {
                    " clip-rule=\"evenodd\""
                } else {
                    ""
                },
            ));
            self.out
                .push_str(&format!("<g clip-path=\"url(#{})\">\n", id));
            self.open_groups += 1;
        }
    }

    /// 칠 속성값 — 단색이면 색, 그러데이션·무늬면 `url(#…)` 참조를 만들어 준다.
    fn fill_paint_attr(&mut self) -> String {
        match self.gs.fill.clone() {
            Source::Color(c) => c.css(),
            Source::Gradient(name) => self
                .emit_gradient_def(&name)
                .unwrap_or_else(|| Rgb::black().css()),
            Source::Pattern(name) => self
                .emit_pattern_def(&name)
                .unwrap_or_else(|| Rgb::black().css()),
        }
    }

    fn emit_gradient_def(&mut self, name: &str) -> Option<String> {
        let def = self.gradients.get(name)?.clone();
        let inst = self.gradient_instance.clone()?;
        self.def_seq += 1;
        let id = format!("aigrad{}", self.def_seq);
        let transform = self.flip().concat(inst.matrix);
        let mut body = String::new();
        for (i, stop) in def.stops.iter().enumerate() {
            body.push_str(&format!(
                "<stop offset=\"{}\" stop-color=\"{}\"/>",
                fmt(stop.offset * 100.0) + "%",
                stop.color.css()
            ));
            // 중간점은 SVG 에 없다 — 두 정지점 사이 `mid` 위치에 섞은 색을 하나 더 넣어 근사한다.
            if let Some(next) = def.stops.get(i + 1) {
                if (stop.mid - 0.5).abs() > 0.02 {
                    let at = stop.offset + (next.offset - stop.offset) * stop.mid;
                    body.push_str(&format!(
                        "<stop offset=\"{}\" stop-color=\"{}\"/>",
                        fmt(at * 100.0) + "%",
                        stop.color.mix(next.color).css()
                    ));
                }
            }
        }
        if def.radial {
            // 방사형: 원점이 중심, length 가 반지름. 하이라이트는 각도·길이(반지름 대비)로 준다.
            let (cx, cy) = inst.origin;
            let r = inst.length.abs().max(0.001);
            let rad = inst.hilight_angle.to_radians();
            let fx = cx + r * inst.hilight_length * rad.cos();
            let fy = cy + r * inst.hilight_length * rad.sin();
            self.defs.push_str(&format!(
                "<radialGradient id=\"{}\" gradientUnits=\"userSpaceOnUse\" cx=\"{}\" cy=\"{}\" r=\"{}\" fx=\"{}\" fy=\"{}\" gradientTransform=\"{}\">{}</radialGradient>\n",
                id, fmt(cx), fmt(cy), fmt(r), fmt(fx), fmt(fy), transform.svg(), body
            ));
        } else {
            let (x1, y1) = inst.origin;
            let rad = inst.angle.to_radians();
            let x2 = x1 + inst.length * rad.cos();
            let y2 = y1 + inst.length * rad.sin();
            self.defs.push_str(&format!(
                "<linearGradient id=\"{}\" gradientUnits=\"userSpaceOnUse\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" gradientTransform=\"{}\">{}</linearGradient>\n",
                id, fmt(x1), fmt(y1), fmt(x2), fmt(y2), transform.svg(), body
            ));
        }
        Some(format!("url(#{})", id))
    }

    fn emit_pattern_def(&mut self, name: &str) -> Option<String> {
        if self.nested {
            return None;
        }
        let def = self.patterns.get(name)?.clone();
        let w = def.bbox[2] - def.bbox[0];
        let h = def.bbox[3] - def.bbox[1];
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        // 타일 그림은 같은 해석기로 한 번 더 돌린다 — 무늬 안의 무늬는 막는다.
        let mut tile = Interp::new(
            self.bbox,
            self.encoding,
            self.gradients.clone(),
            HashMap::new(),
        );
        tile.nested = true;
        tile.run(&def.tile);
        if tile.painted == 0 {
            return None;
        }
        self.def_seq += 1;
        let id = format!("aipat{}", self.def_seq);
        let transform = self.flip().concat(self.gs.fill_pattern_matrix);
        self.defs.push_str(&format!(
            "<pattern id=\"{}\" patternUnits=\"userSpaceOnUse\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" patternTransform=\"{}\">{}{}</pattern>\n",
            id,
            fmt(self.sx(def.bbox[0])),
            fmt(self.sy(def.bbox[3])),
            fmt(w),
            fmt(h),
            transform.svg(),
            tile.defs,
            tile.out
        ));
        Some(format!("url(#{})", id))
    }

    // ── 래스터 ──────────────────────────────────────────────────────────
    /// `[a b c d tx ty] llx lly urx ury h w bits type alpha reserved binascii mask XI`
    fn raster_image(&mut self, lex: &mut Lexer<'_>) {
        let matrix = self
            .last_array()
            .filter(|m| m.len() >= 6)
            .map(|m| Mat::from_slice(&m[m.len() - 6..]))
            .unwrap_or_else(Mat::identity);
        let nums = self.nums();
        if nums.len() < 9 {
            return;
        }
        let n = nums.len();
        let (h, w) = (nums[n - 8] as i64, nums[n - 7] as i64);
        let bits = nums[n - 6] as u32;
        let kind = nums[n - 5] as i32;
        let binascii = nums[n - 2] as i32;
        let is_mask = nums[n - 1] != 0.0;
        let hex = lex.take_hex_comment_lines();
        if w <= 0 || h <= 0 || binascii != 0 || hex.is_empty() {
            return;
        }
        let (w, h) = (w as usize, h as usize);
        if w.saturating_mul(h) > MAX_IMAGE_PIXELS {
            return;
        }
        let bytes = hex_decode(&hex);
        let fill = self.gs.fill.color();
        let Some(png) = raster_to_png(&bytes, w, h, bits, kind, is_mask, fill) else {
            return;
        };
        use base64::Engine as _;
        let uri = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(&png)
        );
        // 이미지 행렬은 단위 정사각형을 그림 자리로 보낸다 — SVG 는 0..w, 0..h 이므로
        // 1/w · 1/h 로 줄인 뒤 행렬을 태우고, 마지막에 y 뒤집기를 합친다.
        let unit = Mat {
            a: 1.0 / w as f64,
            b: 0.0,
            c: 0.0,
            d: 1.0 / h as f64,
            e: 0.0,
            f: 0.0,
        };
        let transform = self.flip().concat(matrix).concat(unit);
        self.out.push_str(&format!(
            "<image width=\"{}\" height=\"{}\" transform=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/>\n",
            w, h, transform.svg(), uri
        ));
        self.painted += 1;
    }

    // ── 글자 ────────────────────────────────────────────────────────────
    fn show_text(&mut self, raw: &[u8]) {
        // 3 = 안 보이는 글자, 7 = 마스크 전용.
        if self.text.render == 3 || self.text.render == 7 {
            return;
        }
        let text = decode_text(raw, self.encoding);
        if text.trim().is_empty() {
            return;
        }
        let (family, weight, style) = font_family(&self.text.font);
        let anchor = match self.text.align {
            1 => " text-anchor=\"middle\"",
            2 => " text-anchor=\"end\"",
            _ => "",
        };
        let mut paint = String::new();
        match self.text.render {
            1 | 5 => paint.push_str(&format!(
                " fill=\"none\" stroke=\"{}\"",
                self.gs.stroke.color().css()
            )),
            2 | 6 => paint.push_str(&format!(
                " fill=\"{}\" stroke=\"{}\"",
                self.gs.fill.color().css(),
                self.gs.stroke.color().css()
            )),
            _ => paint.push_str(&format!(" fill=\"{}\"", self.gs.fill.color().css())),
        }
        // 글자는 위로 자라므로 뒤집힌 좌표계에서 다시 뒤집어야 바로 선다.
        let flip_text = Mat {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: -1.0,
            e: 0.0,
            f: 0.0,
        };
        let transform = self.flip().concat(self.text.matrix).concat(flip_text);
        let mut extra = String::new();
        if (self.text.hscale - 100.0).abs() > 0.01 && self.text.hscale > 0.0 {
            extra.push_str(&format!(
                " transform-origin=\"0 0\" style=\"scale:{} 1\"",
                fmt(self.text.hscale / 100.0)
            ));
        }
        let spacing = self.text.char_space + self.text.tracking * self.text.size / 1000.0;
        if spacing.abs() > 0.001 {
            extra.push_str(&format!(" letter-spacing=\"{}\"", fmt(spacing)));
        }
        if self.text.word_space.abs() > 0.001 {
            extra.push_str(&format!(" word-spacing=\"{}\"", fmt(self.text.word_space)));
        }
        self.out.push_str(&format!(
            "<text x=\"0\" y=\"{}\" transform=\"{}\" font-family=\"{}\" font-size=\"{}\"{}{}{}{}>{}</text>\n",
            fmt(-self.text.rise),
            transform.svg(),
            escape_xml(&family),
            fmt(self.text.size),
            weight,
            style,
            anchor,
            paint.clone() + &extra,
            escape_xml(&text)
        ));
        self.painted += 1;
    }

    fn finish(mut self) -> Option<Vec<u8>> {
        if self.painted == 0 || self.out.len() + self.defs.len() > MAX_SVG_BYTES {
            return None;
        }
        for _ in 0..self.open_groups {
            self.out.push_str("</g>\n");
        }
        let [llx, lly, urx, ury] = self.bbox;
        let w = urx - llx;
        let h = ury - lly;
        // 좌표는 방출할 때 이미 경계 상자 기준으로 옮기고 뒤집었다(`sx`/`sy`) —
        // 바깥 transform 을 또 걸면 두 번 옮겨진다.
        let mut svg = String::with_capacity(self.out.len() + self.defs.len() + 512);
        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" \
             width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" preserveAspectRatio=\"none\">\n",
            fmt(w),
            fmt(h),
            fmt(w),
            fmt(h),
        ));
        if !self.defs.is_empty() {
            svg.push_str("<defs>\n");
            svg.push_str(&self.defs);
            svg.push_str("</defs>\n");
        }
        svg.push_str(&self.out);
        svg.push_str("</svg>\n");
        Some(svg.into_bytes())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 래스터 픽셀 → PNG
// ─────────────────────────────────────────────────────────────────────────────

/// `XI` 픽셀을 PNG 로 굽는다. `kind` 는 1=회색/비트맵, 3=RGB, 4=CMYK (사양 8.1).
fn raster_to_png(
    data: &[u8],
    w: usize,
    h: usize,
    bits: u32,
    kind: i32,
    is_mask: bool,
    mask_color: Rgb,
) -> Option<Vec<u8>> {
    use image::{ImageFormat, Rgba, RgbaImage};

    let channels = match kind {
        3 => 3usize,
        4 => 4usize,
        _ => 1usize,
    };
    let mut img = RgbaImage::new(w as u32, h as u32);
    if bits == 1 && channels == 1 {
        let row_bytes = w.div_ceil(8);
        for y in 0..h {
            for x in 0..w {
                let idx = y * row_bytes + x / 8;
                let bit = data.get(idx).map(|b| (b >> (7 - (x % 8))) & 1).unwrap_or(1);
                let px = if is_mask {
                    // 이미지 마스크: 0 비트에 현재 칠 색을 얹고 1 비트는 비운다.
                    if bit == 0 {
                        Rgba([mask_color.0, mask_color.1, mask_color.2, 255])
                    } else {
                        Rgba([0, 0, 0, 0])
                    }
                } else {
                    let v = if bit == 0 { 0 } else { 255 };
                    Rgba([v, v, v, 255])
                };
                img.put_pixel(x as u32, y as u32, px);
            }
        }
    } else if bits == 8 {
        let stride = w * channels;
        for y in 0..h {
            for x in 0..w {
                let at = y * stride + x * channels;
                let px = match channels {
                    3 => Rgba([*data.get(at)?, *data.get(at + 1)?, *data.get(at + 2)?, 255]),
                    4 => {
                        let c = *data.get(at)? as f64 / 255.0;
                        let m = *data.get(at + 1)? as f64 / 255.0;
                        let yy = *data.get(at + 2)? as f64 / 255.0;
                        let k = *data.get(at + 3)? as f64 / 255.0;
                        let rgb = Rgb::from_cmyk(c, m, yy, k);
                        Rgba([rgb.0, rgb.1, rgb.2, 255])
                    }
                    _ => {
                        let v = *data.get(at)?;
                        Rgba([v, v, v, 255])
                    }
                };
                img.put_pixel(x as u32, y as u32, px);
            }
        }
    } else {
        return None;
    }
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

// ─────────────────────────────────────────────────────────────────────────────
// 글꼴·문자열
// ─────────────────────────────────────────────────────────────────────────────

/// AI 의 재인코딩 글꼴 이름(`_Helvetica-BoldOblique`)을 CSS 글꼴 속성으로 옮긴다.
fn font_family(name: &str) -> (String, &'static str, &'static str) {
    let base = name.trim_start_matches('_');
    let (family, suffix) = match base.split_once('-') {
        Some((f, s)) => (f, s.to_ascii_lowercase()),
        None => (base, String::new()),
    };
    let weight = if suffix.contains("bold") || suffix.contains("black") || suffix.contains("heavy")
    {
        " font-weight=\"bold\""
    } else if suffix.contains("light") || suffix.contains("thin") {
        " font-weight=\"300\""
    } else {
        ""
    };
    let style = if suffix.contains("oblique") || suffix.contains("italic") {
        " font-style=\"italic\""
    } else {
        ""
    };
    let generic = match family.to_ascii_lowercase().as_str() {
        "times" | "timesnewroman" | "garamond" | "minion" | "batang" => "serif",
        "courier" | "couriernew" | "monaco" => "monospace",
        _ => "sans-serif",
    };
    let family = if family.is_empty() {
        generic.to_string()
    } else {
        format!("{}, {}", family, generic)
    };
    (family, weight, style)
}

/// 문자열 바이트를 플랫폼 코드페이지로 읽는다.
fn decode_text(raw: &[u8], encoding: Encoding) -> String {
    raw.iter()
        .map(|&b| match encoding {
            Encoding::WinAnsi => win_ansi_char(b),
            Encoding::MacRoman => mac_roman_char(b),
        })
        .collect()
}

fn win_ansi_char(b: u8) -> char {
    // CP1252 는 0x80~0x9F 만 Latin-1 과 다르다.
    const HIGH: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž',
        '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}',
        'ž', 'Ÿ',
    ];
    if (0x80..0xA0).contains(&b) {
        HIGH[(b - 0x80) as usize]
    } else {
        b as char
    }
}

fn mac_roman_char(b: u8) -> char {
    const HIGH: [char; 128] = [
        'Ä', 'Å', 'Ç', 'É', 'Ñ', 'Ö', 'Ü', 'á', 'à', 'â', 'ä', 'ã', 'å', 'ç', 'é', 'è', 'ê', 'ë',
        'í', 'ì', 'î', 'ï', 'ñ', 'ó', 'ò', 'ô', 'ö', 'õ', 'ú', 'ù', 'û', 'ü', '†', '°', '¢', '£',
        '§', '•', '¶', 'ß', '®', '©', '™', '´', '¨', '≠', 'Æ', 'Ø', '∞', '±', '≤', '≥', '¥', 'µ',
        '∂', '∑', '∏', 'π', '∫', 'ª', 'º', 'Ω', 'æ', 'ø', '¿', '¡', '¬', '√', 'ƒ', '≈', '∆', '«',
        '»', '…', '\u{a0}', 'À', 'Ã', 'Õ', 'Œ', 'œ', '–', '—', '“', '”', '‘', '’', '÷', '◊', 'ÿ',
        'Ÿ', '⁄', '€', '‹', '›', 'ﬁ', 'ﬂ', '‡', '·', '‚', '„', '‰', 'Â', 'Ê', 'Á', 'Ë', 'È', 'Í',
        'Î', 'Ï', 'Ì', 'Ó', 'Ô', '\u{f8ff}', 'Ò', 'Ú', 'Û', 'Ù', 'ı', 'ˆ', '˜', '¯', '˘', '˙', '˚',
        '¸', '˝', '˛', 'ˇ',
    ];
    if b < 0x80 {
        b as char
    } else {
        HIGH[(b - 0x80) as usize]
    }
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\r' | '\n' => out.push(' '),
            c if (c as u32) < 0x20 => {}
            c => out.push(c),
        }
    }
    out
}

fn fmt(v: f64) -> String {
    if v == 0.0 || !v.is_finite() {
        return "0".to_string();
    }
    let s = format!("{:.4}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eps_with(setup: &str, body: &str) -> Vec<u8> {
        format!(
            "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 50\n%%BeginSetup\n{setup}\n\
             %%EndSetup\n{body}\n%%Trailer\n"
        )
        .into_bytes()
    }

    fn eps(body: &str) -> Vec<u8> {
        eps_with("", body)
    }

    fn svg_of(body: &str) -> String {
        String::from_utf8(convert_ai_artwork_to_svg(&eps(body)).expect("변환")).unwrap()
    }

    #[test]
    fn fills_a_closed_path_and_flips_the_y_axis() {
        let svg = svg_of("0 0 0 1 k\n10 10 m\n90 10 L\n90 40 L\n10 40 L\nf");
        assert!(svg.contains("viewBox=\"0 0 100 50\""), "{svg}");
        assert!(svg.contains("M10 40L90 40L90 10L10 10Z"), "{svg}");
        assert!(svg.contains("fill=\"#000000\""), "{svg}");
    }

    #[test]
    fn cmyk_uses_the_ai_prolog_formula() {
        // 사양 5.4.1: red = 1 - min(1, cyan + black). 곱셈식이면 #2c... 가 나온다.
        let svg = svg_of("1 0.72 0 0.38 k\n0 0 m 10 0 L 10 10 L f");
        assert!(svg.contains("fill=\"#00009e\""), "{svg}");
    }

    #[test]
    fn custom_color_tint_is_one_minus_gray() {
        let full = svg_of("1 1 0 0 (PANTONE X) 0 x\n0 0 m 10 0 L 10 10 L f");
        assert!(full.contains("fill=\"#0000ff\""), "{full}");
        let none = svg_of("1 1 0 0 (PANTONE X) 1 x\n0 0 m 10 0 L 10 10 L f");
        assert!(none.contains("fill=\"#ffffff\""), "{none}");
    }

    #[test]
    fn rgb_and_generic_custom_color_operators() {
        let rgb = svg_of("1 0.5 0 Xa\n0 0 m 10 0 L 10 10 L f");
        assert!(rgb.contains("fill=\"#ff8000\""), "{rgb}");
        // `red green blue (name) tint 1 Xx` — RGB 별색.
        let spot = svg_of("1 0 0 (Spot) 0.5 1 Xx\n0 0 m 10 0 L 10 10 L f");
        assert!(spot.contains("fill=\"#ff8080\""), "{spot}");
    }

    #[test]
    fn compound_path_paints_once_so_holes_survive() {
        let svg =
            svg_of("*u\n0 0 m 40 0 L 40 40 L 0 40 L h\n10 10 m 30 10 L 30 30 L 10 30 L h\nf\n*U");
        assert_eq!(svg.matches("<path").count(), 1, "{svg}");
        assert_eq!(svg.matches('M').count(), 2, "{svg}");
    }

    #[test]
    fn even_odd_flag_selects_fill_rule() {
        let svg = svg_of("1 XR\n0 0 m 10 0 L 10 10 L f");
        assert!(svg.contains("fill-rule=\"evenodd\""), "{svg}");
    }

    #[test]
    fn stroke_carries_width_dash_and_caps() {
        let svg = svg_of("0 G\n2 w\n1 J 1 j\n[3 2]0 d\n0 0 m 10 10 L S");
        assert!(svg.contains("stroke-width=\"2\""), "{svg}");
        assert!(svg.contains("stroke-dasharray=\"3 2\""), "{svg}");
        assert!(svg.contains("stroke-linecap=\"round\""), "{svg}");
        assert!(svg.contains("stroke-linejoin=\"round\""), "{svg}");
    }

    #[test]
    fn clip_path_scope_closes_at_matching_q() {
        let svg = svg_of(
            "q\n0 0 m 10 0 L 10 10 L h W n\n0 0 m 5 0 L 5 5 L f\nQ\n20 20 m 30 20 L 30 30 L f",
        );
        assert!(svg.contains("<clipPath"), "{svg}");
        // 오려내기 그룹이 `Q` 에서 닫혀야 뒤 도형이 남의 오려내기에 걸리지 않는다.
        let after_close = svg.rsplit("</g>").next().unwrap_or("");
        assert!(after_close.contains("M20 30"), "{svg}");
    }

    #[test]
    fn guide_paths_are_discarded_not_merged() {
        let svg = svg_of("(N) *\n0 0 m 1000 0 L 1000 1000 L (N) *\n10 10 m 20 10 L 20 20 L f");
        assert_eq!(svg.matches("<path").count(), 1, "{svg}");
        assert!(!svg.contains("1000"), "{svg}");
    }

    #[test]
    fn non_printing_layer_is_skipped() {
        let visible = svg_of("1 1 1 1 0 0 0 0 0 0 Lb\n0 0 m 10 0 L 10 10 L f\nLB");
        assert_eq!(visible.matches("<path").count(), 1, "{visible}");
        let hidden =
            convert_ai_artwork_to_svg(&eps("1 1 1 0 0 0 0 0 0 0 Lb\n0 0 m 10 0 L 10 10 L f\nLB"));
        assert!(
            hidden.is_none(),
            "인쇄 안 하는 레이어만 있으면 그릴 것이 없다"
        );
    }

    #[test]
    fn linear_gradient_instance_becomes_svg_gradient() {
        let setup = "1 Bn\n%AI5_BeginGradient: (Red & Yellow)\n(Red & Yellow) 0 2 Bd\n[\n\
                     0 1 1 0 1 50 0 %_Bs\n0 0 1 0 1 50 100 %_Bs\nBD\n%AI5_EndGradient";
        let body = "0 0 m 100 0 L 100 50 L 0 50 L\nBb\n\
                    1 (Red & Yellow) 0 0 0 100 1 0 0 1 0 0 Bg\nf\n0 BB";
        let svg =
            String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
        assert!(svg.contains("<linearGradient"), "{svg}");
        assert!(svg.contains("stop-color=\"#ff0000\""), "{svg}");
        assert!(svg.contains("stop-color=\"#ffff00\""), "{svg}");
        assert!(svg.contains("fill=\"url(#aigrad"), "{svg}");
    }

    #[test]
    fn radial_gradient_uses_hilight_as_focus() {
        let setup = "1 Bn\n%AI5_BeginGradient: (Ball)\n(Ball) 1 2 Bd\n[\n\
                     0 0 0 0 1 50 0 %_Bs\n0 0 0 1 1 50 100 %_Bs\nBD\n%AI5_EndGradient";
        let body = "0 0 m 50 0 L 50 50 L\nBb\n0 0 0 0 Bh\n\
                    1 (Ball) 25 25 0 25 1 0 0 1 0 0 Bg\nf\n0 BB";
        let svg =
            String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
        assert!(svg.contains("<radialGradient"), "{svg}");
        assert!(svg.contains("r=\"25\""), "{svg}");
    }

    #[test]
    fn gradient_midpoint_adds_an_interpolated_stop() {
        let setup = "1 Bn\n%AI5_BeginGradient: (Mid)\n(Mid) 0 2 Bd\n[\n\
                     0 0 0 0 1 25 0 %_Bs\n0 0 0 1 1 50 100 %_Bs\nBD\n%AI5_EndGradient";
        let body = "0 0 m 10 0 L 10 10 L\nBb\n1 (Mid) 0 0 0 10 1 0 0 1 0 0 Bg\nf\n0 BB";
        let svg =
            String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
        assert_eq!(svg.matches("<stop").count(), 3, "{svg}");
        assert!(svg.contains("offset=\"25%\""), "{svg}");
    }

    #[test]
    fn text_is_placed_by_its_matrix_and_upright() {
        let svg =
            svg_of("0 To\n1 0 0 1 10 20 0 Tp\nTP\n0 Tr\n/_Helvetica-Bold 12 0 0 Tf\n(Hi) Tx\nTO");
        assert!(svg.contains("<text"), "{svg}");
        assert!(svg.contains(">Hi<"), "{svg}");
        assert!(svg.contains("font-size=\"12\""), "{svg}");
        assert!(svg.contains("font-weight=\"bold\""), "{svg}");
        // 뒤집힌 좌표계에서 글자는 다시 뒤집어야 바로 선다: d = -1 × -1 = 1.
        assert!(svg.contains("matrix(1 0 0 1 10 30)"), "{svg}");
    }

    #[test]
    fn invisible_text_render_modes_are_dropped() {
        let hidden = convert_ai_artwork_to_svg(&eps(
            "0 To\n1 0 0 1 0 0 0 Tp\nTP\n3 Tr\n/_Helvetica 12 0 0 Tf\n(Hi) Tx\nTO",
        ));
        assert!(hidden.is_none(), "render 3 은 안 보이는 글자다");
    }

    #[test]
    fn raster_image_becomes_an_embedded_png() {
        // 2×2 RGB, 16진 ASCII.
        let body = "[ 10 0 0 10 5 5 ] 0 0 2 2 2 2 8 3 0 0 0 0 XI\n\
                    %FF0000 00FF00\n%0000FF FFFFFF\n%AI5_EndRaster";
        let svg = svg_of(body);
        assert!(svg.contains("<image"), "{svg}");
        assert!(svg.contains("data:image/png;base64,"), "{svg}");
    }

    #[test]
    fn pattern_definition_becomes_an_svg_pattern() {
        let setup = "%AI3_BeginPattern: (dots)\n(dots) 0 0 10 10 [\n%AI3_Tile\n\
                     (0 O 0 R 1 0 0 0 k) @\n(0 0 m 10 0 L 10 10 L 0 10 L f) &\n] E\n\
                     %AI3_EndPattern";
        let body = "(dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n0 0 m 50 0 L 50 50 L 0 50 L f";
        let svg =
            String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
        assert!(svg.contains("<pattern"), "{svg}");
        assert!(svg.contains("fill=\"url(#aipat"), "{svg}");
    }

    #[test]
    fn prolog_is_not_interpreted() {
        // 프롤로그의 프로시저 정의 안에도 `m`·`f` 같은 이름이 널려 있다.
        let data = "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\n\
                    /f { closepath F } def\n0 0 m 5 5 L 5 0 L f\n%%EndSetup\n%%Trailer\n";
        assert!(convert_ai_artwork_to_svg(data.as_bytes()).is_none());
    }

    #[test]
    fn text_only_postscript_yields_nothing() {
        let data = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\n";
        assert!(convert_ai_artwork_to_svg(data).is_none());
    }

    #[test]
    fn missing_bounding_box_is_rejected() {
        let data = b"%!PS-Adobe-3.0 EPSF-3.0\n%%EndSetup\n0 0 m 10 10 L f\n";
        assert!(convert_ai_artwork_to_svg(data).is_none());
    }

    #[test]
    fn hires_bounding_box_wins() {
        let data = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 50\n\
                     %%HiResBoundingBox: 0 0 100.5 50.25\n%%EndSetup\n0 0 m 10 0 L 10 10 L f\n";
        let svg = String::from_utf8(convert_ai_artwork_to_svg(data).unwrap()).unwrap();
        assert!(svg.contains("viewBox=\"0 0 100.5 50.25\""), "{svg}");
    }

    /// 손상된 그러데이션 정지점은 그림 하나를 버리게 할 뿐 해석기를 죽이면 안 된다.
    ///
    /// `0 1 50 0 %_Bs` 는 색 종류가 CMYK(1) 인데 피연산자가 하나뿐이다 — 문서에서 온
    /// 바이트는 언제든 이럴 수 있으므로 종류별 피연산자 수를 믿고 첨자로 읽으면 안 된다.
    #[test]
    fn truncated_gradient_stop_does_not_panic() {
        let setup = "1 Bn\n%AI5_BeginGradient: (Broken)\n(Broken) 0 2 Bd\n[\n\
                     0 1 50 0 %_Bs\n0 0 1 0 1 50 100 %_Bs\nBD\n%AI5_EndGradient";
        let body = "0 0 m 100 0 L 100 50 L 0 50 L\nBb\n\
                    1 (Broken) 0 0 0 100 1 0 0 1 0 0 Bg\nf\n0 BB";
        let svg =
            String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
        assert!(svg.contains("<linearGradient"), "{svg}");
    }

    #[test]
    fn mac_and_windows_string_bytes_decode_differently() {
        assert_eq!(decode_text(&[0xD5], Encoding::MacRoman), "’");
        assert_eq!(decode_text(&[0xD5], Encoding::WinAnsi), "Õ");
    }
}

//! [Task #2264] PDF 변환 단계 메모리 프로파일링 하네스 (조사 전용, 배포 대상 아님)
//!
//! `export-pdf` 경로의 최대 RSS 를 폰트 / 이미지 / PDF chunk 로 분해한다.
//! 단계마다 별도 프로세스로 실행하고 `/usr/bin/time -l` 로 최대 RSS 를 잰다.
//!
//! 사용법:
//!   cargo run --release --example task2264_profile -- <page.svg> <stage> [ablation...]
//!
//!   stage:     read | fontdb | parse | chunk
//!   ablation:  no-images | no-text | no-sysfonts
//!
//! 예) /usr/bin/time -l cargo run ... -- page0.svg chunk no-images

fn strip_images(svg: &str) -> String {
    // <image .../> 요소 제거 (base64 페이로드 포함)
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<image") {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        match after.find("/>") {
            Some(end) => rest = &after[end + 2..],
            None => match after.find('>') {
                Some(end) => rest = &after[end + 1..],
                None => {
                    rest = "";
                    break;
                }
            },
        }
    }
    out.push_str(rest);
    out
}

fn strip_text(svg: &str) -> String {
    // <text ...>...</text> 요소 제거
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text") {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        match after.find("</text>") {
            Some(end) => rest = &after[end + "</text>".len()..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: task2264_profile <page.svg> <read|fontdb|parse|chunk> [no-images|no-text|no-sysfonts]");
        std::process::exit(2);
    }

    let path = &args[1];
    let stage = args[2].as_str();
    let ablations: Vec<&str> = args[3..].iter().map(|s| s.as_str()).collect();
    let has = |a: &str| ablations.contains(&a);

    // 1) SVG 로드 + ablation 적용
    let mut svg = std::fs::read_to_string(path).expect("SVG 읽기 실패");
    if has("no-images") {
        svg = strip_images(&svg);
    }
    if has("no-text") {
        svg = strip_text(&svg);
    }
    eprintln!("[svg] {:.2} MB", svg.len() as f64 / 1048576.0);
    if stage == "read" {
        std::hint::black_box(&svg);
        return;
    }

    // 2) fontdb 구성 (pdf.rs 의 create_fontdb 를 최소 재현)
    let mut fontdb = usvg::fontdb::Database::new();
    if !has("no-sysfonts") {
        fontdb.load_system_fonts();
    }
    for dir in &["ttfs", "ttfs/windows", "ttfs/hwp"] {
        if std::path::Path::new(dir).exists() {
            fontdb.load_fonts_dir(dir);
        }
    }
    fontdb.set_serif_family("AppleMyungjo");
    fontdb.set_sans_serif_family("Apple SD Gothic Neo");
    fontdb.set_monospace_family("Menlo");
    eprintln!("[fontdb] faces={}", fontdb.len());
    if stage == "fontdb" {
        std::hint::black_box(&fontdb);
        return;
    }

    // 3) usvg 파싱 (텍스트 shaping + 폰트 face 실체화 + 이미지 디코딩)
    let mut options = usvg::Options::default();
    options.fontdb = std::sync::Arc::new(fontdb);
    let tree = usvg::Tree::from_str(&svg, &options).expect("SVG 파싱 실패");
    eprintln!("[parse] done");
    if stage == "parse" {
        std::hint::black_box(&tree);
        return;
    }

    // 4) svg2pdf chunk 변환 (PDF XObject 생성, 이미지 재인코딩, 폰트 서브셋)
    let mut conv = svg2pdf::ConversionOptions::default();
    if has("no-embed-text") {
        // 텍스트를 PDF 폰트로 임베드하지 않고 path 로 변환한다.
        // 폰트 서브셋/임베딩 경로를 통째로 건너뛴다.
        conv.embed_text = false;
    }
    let (chunk, _r) = svg2pdf::to_chunk(&tree, conv).expect("chunk 변환 실패");
    eprintln!("[chunk] done");
    std::hint::black_box(&chunk);
}

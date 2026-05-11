//! PDF 렌더러 (Task #21)
//!
//! SVG 렌더러의 출력을 svg2pdf + pdf-writer로 PDF를 생성한다.
//! 단일/다중 페이지 모두 지원. 네이티브 전용 (WASM 미지원).

/// 폰트 데이터베이스를 초기화 (시스템 폰트 + 프로젝트 폰트 로드)
#[cfg(not(target_arch = "wasm32"))]
fn create_fontdb() -> usvg::fontdb::Database {
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    for dir in &["ttfs", "ttfs/windows", "ttfs/hwp"] {
        if std::path::Path::new(dir).exists() {
            fontdb.load_fonts_dir(dir);
        }
    }
    if std::path::Path::new("/mnt/c/Windows/Fonts").exists() {
        fontdb.load_fonts_dir("/mnt/c/Windows/Fonts");
    }
    fontdb.set_serif_family("바탕");
    fontdb.set_sans_serif_family("맑은 고딕");
    fontdb.set_monospace_family("D2Coding");
    fontdb
}

/// SVG에서 없는 한글 폰트명에 fallback 추가
#[cfg(not(target_arch = "wasm32"))]
fn add_font_fallbacks(svg: &str) -> String {
    svg.replace("font-family=\"휴먼명조\"", "font-family=\"휴먼명조, 바탕, serif\"")
       .replace("font-family=\"HCI Poppy\"", "font-family=\"HCI Poppy, 맑은 고딕, sans-serif\"")
}

/// SVG 콘텐츠에서 최대 X 좌표(x + width)를 스캔한다.
/// clipPath 등의 rect에서 뷰포트를 넘는 콘텐츠를 탐지.
#[cfg(not(target_arch = "wasm32"))]
fn scan_svg_max_x(svg: &str) -> f32 {
    let mut max_x: f32 = 0.0;

    // 루트 <svg> 태그 이후만 스캔
    let body = match svg.find('>') {
        Some(pos) => &svg[pos + 1..],
        None => return 0.0,
    };

    // <rect ... x="X" ... width="W" .../> 패턴에서 x + width 계산
    // rhwp의 clipPath rect가 뷰포트를 넘는 주요 원인
    let mut search_from = 0;
    while let Some(rect_pos) = body[search_from..].find("<rect ") {
        let abs_pos = search_from + rect_pos;
        let tag_end = match body[abs_pos..].find("/>") {
            Some(e) => abs_pos + e + 2,
            None => match body[abs_pos..].find('>') {
                Some(e) => abs_pos + e + 1,
                None => break,
            },
        };
        let tag = &body[abs_pos..tag_end];

        let mut rect_x: f32 = 0.0;
        let mut rect_w: f32 = 0.0;

        // x="..." 추출
        if let Some(xi) = tag.find(" x=\"") {
            let val_start = xi + 4;
            if let Some(val_end) = tag[val_start..].find('"') {
                rect_x = tag[val_start..val_start + val_end].parse().unwrap_or(0.0);
            }
        }
        // width="..." 추출
        if let Some(wi) = tag.find(" width=\"") {
            let val_start = wi + 8;
            if let Some(val_end) = tag[val_start..].find('"') {
                rect_w = tag[val_start..val_start + val_end].parse().unwrap_or(0.0);
            }
        }

        max_x = max_x.max(rect_x + rect_w);
        search_from = tag_end;
    }

    max_x
}

/// SVG 루트 요소의 width, height, viewBox를 확장하여
/// 콘텐츠가 BBox에 의해 잘리지 않도록 한다.
#[cfg(not(target_arch = "wasm32"))]
fn expand_svg_viewport(svg: &str, new_w: f32, new_h: f32) -> String {
    use std::fmt::Write;

    let mut result = String::with_capacity(svg.len() + 64);
    if let Some(svg_tag_end) = svg.find('>') {
        let tag = &svg[..=svg_tag_end];
        let rest = &svg[svg_tag_end + 1..];

        // width, height, viewBox 속성을 새 값으로 교체
        let mut new_tag = String::with_capacity(tag.len() + 64);
        let mut i = 0;
        let bytes = tag.as_bytes();
        while i < bytes.len() {
            if tag[i..].starts_with("width=\"") {
                new_tag.push_str("width=\"");
                let _ = write!(new_tag, "{}", new_w);
                new_tag.push('"');
                // 기존 값 건너뛰기
                i += 7; // skip `width="`
                while i < bytes.len() && bytes[i] != b'"' { i += 1; }
                i += 1; // skip closing `"`
            } else if tag[i..].starts_with("height=\"") {
                new_tag.push_str("height=\"");
                let _ = write!(new_tag, "{}", new_h);
                new_tag.push('"');
                i += 8;
                while i < bytes.len() && bytes[i] != b'"' { i += 1; }
                i += 1;
            } else if tag[i..].starts_with("viewBox=\"") {
                new_tag.push_str("viewBox=\"");
                let _ = write!(new_tag, "0 0 {} {}", new_w, new_h);
                new_tag.push('"');
                i += 9;
                while i < bytes.len() && bytes[i] != b'"' { i += 1; }
                i += 1;
            } else {
                new_tag.push(bytes[i] as char);
                i += 1;
            }
        }
        result.push_str(&new_tag);
        result.push_str(rest);
    } else {
        return svg.to_string();
    }
    result
}

/// 단일 SVG를 PDF로 변환
#[cfg(not(target_arch = "wasm32"))]
pub fn svg_to_pdf(svg_content: &str) -> Result<Vec<u8>, String> {
    let fontdb = create_fontdb();
    let mut options = usvg::Options::default();
    options.fontdb = std::sync::Arc::new(fontdb);
    let svg_with_fallback = add_font_fallbacks(svg_content);
    let tree = usvg::Tree::from_str(&svg_with_fallback, &options)
        .map_err(|e| format!("SVG 파싱 실패: {}", e))?;
    let pdf = svg2pdf::to_pdf(&tree, svg2pdf::ConversionOptions::default(), svg2pdf::PageOptions::default())
        .map_err(|e| format!("PDF 변환 실패: {:?}", e))?;
    Ok(pdf)
}

/// 여러 SVG 페이지를 단일 다중 페이지 PDF로 생성
#[cfg(not(target_arch = "wasm32"))]
pub fn svgs_to_pdf(svg_pages: &[String]) -> Result<Vec<u8>, String> {
    if svg_pages.is_empty() {
        return Err("페이지가 없습니다".to_string());
    }
    if svg_pages.len() == 1 {
        return svg_to_pdf(&svg_pages[0]);
    }

    use pdf_writer::{Pdf, Ref, Finish};
    use std::collections::HashMap;

    let fontdb = create_fontdb();
    let mut options = usvg::Options::default();
    options.fontdb = std::sync::Arc::new(fontdb);

    let mut alloc = Ref::new(1);
    let catalog_ref = alloc.bump();
    let page_tree_ref = alloc.bump();

    // 각 페이지의 SVG를 파싱하여 chunk + page 정보 수집
    struct PageData {
        chunk: pdf_writer::Chunk,
        svg_ref: Ref,
        width: f32,
        height: f32,
    }

    let mut page_datas: Vec<PageData> = Vec::new();

    for svg in svg_pages {
        let svg_with_fallback = add_font_fallbacks(svg);
        let tree = usvg::Tree::from_str(&svg_with_fallback, &options)
            .map_err(|e| format!("SVG 파싱 실패: {}", e))?;

        let orig_w = tree.size().width();
        let orig_h = tree.size().height();

        // Fix: SVG 콘텐츠 좌표가 뷰포트를 넘는지 확인하여 BBox 클리핑 방지
        // usvg의 bounding_box()는 뷰포트에 클리핑되므로 SVG 원본을 직접 스캔
        let max_x = scan_svg_max_x(&svg_with_fallback);

        let tree_for_chunk = if max_x > orig_w * 1.01 {
            let expanded = expand_svg_viewport(&svg_with_fallback, max_x, orig_h);
            usvg::Tree::from_str(&expanded, &options)
                .map_err(|e| format!("SVG 재파싱 실패: {}", e))?
        } else {
            tree
        };

        let (chunk, svg_ref) = svg2pdf::to_chunk(&tree_for_chunk, svg2pdf::ConversionOptions::default())
            .map_err(|e| format!("SVG→chunk 변환 실패: {:?}", e))?;

        // 페이지 크기는 원본 뷰포트 기준 (A4 유지)
        let dpi_ratio = 72.0 / 96.0; // 96 DPI → 72 pt
        let w = orig_w * dpi_ratio;
        let h = orig_h * dpi_ratio;

        page_datas.push(PageData { chunk, svg_ref, width: w, height: h });
    }

    // 각 chunk를 재번호화하고 페이지 참조 수집
    let mut page_refs: Vec<Ref> = Vec::new();
    let mut renumbered_chunks: Vec<pdf_writer::Chunk> = Vec::new();
    let mut svg_refs_remapped: Vec<Ref> = Vec::new();

    for pd in &page_datas {
        let page_ref = alloc.bump();
        let content_ref = alloc.bump();
        page_refs.push(page_ref);

        // chunk 재번호화
        let mut map = HashMap::new();
        let renumbered = pd.chunk.renumber(|old| {
            *map.entry(old).or_insert_with(|| alloc.bump())
        });

        let remapped_svg_ref = map.get(&pd.svg_ref).copied().unwrap_or(pd.svg_ref);
        svg_refs_remapped.push(remapped_svg_ref);
        renumbered_chunks.push(renumbered);
    }

    // PDF 생성
    let mut pdf = Pdf::new();
    pdf.catalog(catalog_ref).pages(page_tree_ref);
    pdf.pages(page_tree_ref)
        .count(page_refs.len() as i32)
        .kids(page_refs.iter().copied());

    // 각 페이지 생성
    let svg_name = pdf_writer::Name(b"S1");

    for (i, pd) in page_datas.iter().enumerate() {
        let page_ref = page_refs[i];
        let content_ref = alloc.bump();
        let svg_ref = svg_refs_remapped[i];

        let mut page = pdf.page(page_ref);
        page.media_box(pdf_writer::Rect::new(0.0, 0.0, pd.width, pd.height));
        page.parent(page_tree_ref);
        page.contents(content_ref);

        let mut resources = page.resources();
        resources.x_objects().pair(svg_name, svg_ref);
        resources.finish();
        page.finish();

        // 컨텐츠 스트림: SVG XObject를 페이지 크기에 맞게 배치
        let mut content = pdf_writer::Content::new();
        content.transform([pd.width, 0.0, 0.0, pd.height, 0.0, 0.0]);
        content.x_object(svg_name);

        pdf.stream(content_ref, &content.finish());
    }

    // 모든 chunk를 PDF에 추가
    for chunk in &renumbered_chunks {
        pdf.extend(chunk);
    }

    // 문서 정보
    let info_ref = alloc.bump();
    pdf.document_info(info_ref).producer(pdf_writer::TextStr("rhwp"));

    Ok(pdf.finish())
}

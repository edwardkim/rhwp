//! Task #554 회귀 테스트
//!
//! HWP3 → HWP5/HWPX 변환본 페이지네이션 정합 검증.
//!
//! ## 본질
//!
//! 한컴이 HWP3 → HWP5/HWPX 변환 시 한글97의 "마지막 줄 tolerance" (1600 HU)
//! 동작이 누락되어 페이지 수가 +1 ~ +4 증가했다. Task #554 에서 변환본
//! 식별 휴리스틱 + 조건부 `margin_bottom -= 1600` 보정으로 정정.
//!
//! ## 식별 휴리스틱
//!
//! - HWPX: `<hh:head version="1.4">`
//! - HWP5: `(ParaShape/Paragraph < 0.05) AND (CharShape/Paragraph < 0.15) AND (Paragraph > 50)`
//!
//! Task #554 Stage 1 진단 결과 (27 fixture 100% 정확).

use std::fs;
use std::path::Path;

fn page_count(rel_path: &str) -> usize {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"));
    doc.page_count() as usize
}

// ───────── HWP3 변환본 (Task #554 핵심 대상): 정답 정합 ─────────

#[test]
fn hwp3_sample4_hwp5_36p() {
    // HWP3 변환본 (36페이지 정답): 한컴 정답 일치
    assert_eq!(page_count("samples/hwp3-sample4-hwp5.hwp"), 36);
}

#[test]
fn hwp3_sample5_hwp5_64p() {
    // HWP3 변환본 (64페이지 정답)
    assert_eq!(page_count("samples/hwp3-sample5-hwp5.hwp"), 64);
}

#[test]
fn hwp3_sample5_hwpx_64p() {
    // HWPX 변환본 (hwpml=1.4)
    assert_eq!(page_count("samples/hwp3-sample5-hwpx.hwpx"), 64);
}

// ───────── 알려진 잔존 -1 over-correct 해소 확인 ─────────
//
// #949 lineSegArray vertpos 보존 후 sample 변환본도 한컴 정답 16p에 도달했다.
// 이전에는 단일 -1600 HU 보정의 한계로 15p가 나왔고, 이 값을 known-limit 가드로
// 고정해 두었다. 이제는 정답 페이지 수를 회귀 가드로 검증한다.

#[test]
fn hwp3_sample_hwp5_16p() {
    use rhwp::{document_core::DocumentCore, model::control::Control};
    use serde_json::Value;
    use std::process::Command;

    fn nodes<'a>(node: &'a Value, result: &mut Vec<&'a Value>) {
        result.push(node);
        for child in node["children"].as_array().into_iter().flatten() {
            nodes(child, result);
        }
    }
    fn top(node: &Value) -> f64 {
        node["bbox"]["y"].as_f64().expect("노드 상단")
    }
    fn bottom(node: &Value) -> f64 {
        top(node) + node["bbox"]["h"].as_f64().expect("노드 높이")
    }
    struct OwnedOutput(std::path::PathBuf);
    impl Drop for OwnedOutput {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("samples/hwp3-sample-hwp5.hwp");
    let bytes = fs::read(&source).expect("원본 HWP3 변환 문서");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(core.page_count(), 16, "독립 한컴 Print와 같은 전체 쪽 소속");
    let paragraphs = &core.document().sections[0].paragraphs;
    let host = paragraphs
        .iter()
        .position(|para| {
            para.controls.iter().any(|control| {
                let Control::Picture(picture) = control else {
                    return false;
                };
                picture.caption.as_ref().is_some_and(|caption| {
                    caption
                        .paragraphs
                        .iter()
                        .any(|p| p.text.contains("NAT에 의한 가상 서버의 예"))
                })
            })
        })
        .expect("원본 캡션이 가리키는 그림 호스트");
    let [Control::Picture(picture)] = paragraphs[host].controls.as_slice() else {
        panic!("원본 호스트 그림 한 개")
    };
    assert!(
        paragraphs[host].text.trim().is_empty(),
        "빈 호스트도 저장 줄 공간을 소유"
    );
    let previous = &paragraphs[host - 1];
    let last = previous.line_segs.last().expect("그림 앞 저장 줄");
    let offset = i64::from(picture.common.vertical_offset as i32)
        + i64::from(picture.common.margin.top)
        + i64::from(last.line_spacing);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let out = OwnedOutput(
        root.join("output/regression")
            .join(format!("converted16-{}-{stamp}", std::process::id())),
    );
    fs::create_dir_all(&out.0).expect("검사 소유 output 디렉터리");
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(bin)
        .arg("export-render-tree")
        .arg(&source)
        .args(["--compat", "2022"])
        .arg("-o")
        .arg(&out.0)
        .output()
        .expect("실제 CLI 배치");
    assert!(
        result.status.success(),
        "배치 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut files: Vec<_> = fs::read_dir(&out.0)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|s| s == "json"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 16, "CLI 전체 쪽과 누락 없음");
    let trees: Vec<Value> = files
        .iter()
        .map(|p| serde_json::from_slice(&fs::read(p).unwrap()).unwrap())
        .collect();
    let mut flattened = Vec::new();
    nodes(&trees[7], &mut flattened);
    let images: Vec<_> = flattened
        .iter()
        .copied()
        .filter(|n| n["type"] == "Image" && n["pi"].as_u64() == Some(host as u64))
        .collect();
    assert_eq!(images.len(), 1, "독립 Print 8쪽의 그림 소유와 중복 없음");
    let captions: Vec<_> = flattened
        .iter()
        .copied()
        .filter(|n| {
            n["type"] == "TextLine" && n["captionOwner"]["paraIdx"].as_u64() == Some(host as u64)
        })
        .collect();
    assert_eq!(captions.len(), 1, "같은 그림의 한 줄 캡션");
    let lines = |pi: usize| {
        flattened
            .iter()
            .copied()
            .filter(|n| {
                n["type"] == "TextLine"
                    && n["pi"].as_u64() == Some(pi as u64)
                    && n["captionOwner"].is_null()
            })
            .collect::<Vec<_>>()
    };
    let previous_lines = lines(host - 1);
    assert_eq!(
        previous_lines.len(),
        previous.line_segs.len(),
        "선행 원본 줄 보존"
    );
    // RenderTree JSON의 0.1px 직렬화 반올림 두 번만 허용한다. 문서 절대 좌표 핀이 아니다.
    let serialization_error = 2.0 * 0.1;
    let expected_gap = rhwp::renderer::hwpunit_to_px(offset as i32, 96.0);
    assert!(
        (top(images[0]) - bottom(previous_lines.last().unwrap()) - expected_gap).abs()
            <= serialization_error,
        "그림 앞 저장 줄 진행·개체 offset·여백을 한 번 소비"
    );
    assert!(top(captions[0]) >= bottom(images[0]), "캡션은 그림 아래");
    let next = lines(host + 1);
    assert_eq!(next.len(), 1, "그림 뒤 본문은 같은 쪽에 한 번");
    assert!(top(next[0]) >= bottom(captions[0]), "본문은 캡션 아래");
    let tables: Vec<_> = flattened
        .iter()
        .copied()
        .filter(|n| n["type"] == "Table")
        .collect();
    assert_eq!(tables.len(), 5, "독립 Print의 규칙 표와 네 패킷 표");
    let source_owners: Vec<_> = paragraphs
        .iter()
        .enumerate()
        .skip(host + 1)
        .take_while(|(_, p)| {
            p.line_segs
                .first()
                .is_some_and(|line| line.vertical_pos > 0)
        })
        .filter(|(_, p)| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .map(|(pi, _)| pi)
        .collect();
    let rendered_owners: Vec<_> = tables
        .iter()
        .map(|n| n["pi"].as_u64().expect("표 문단 소유") as usize)
        .collect();
    assert_eq!(
        rendered_owners, source_owners,
        "쪽 안의 원본 표 순서와 누락/중복 없음"
    );
    for table in tables {
        let pi = table["pi"].as_u64().unwrap() as usize;
        let before = lines(pi - 1);
        assert!(
            !before.is_empty() && top(table) >= bottom(before.last().unwrap()),
            "표가 앞 본문 위로 되감기지 않음: pi={pi}"
        );
        let after = lines(pi + 1);
        if pi != *source_owners.last().unwrap() {
            assert!(
                !after.is_empty() && top(after[0]) >= bottom(table),
                "표 뒤 본문이 같은 쪽에 있고 표를 침범하지 않음: pi={pi}"
            );
        }
    }
    let split_owner = paragraphs
        .iter()
        .position(|p| p.text.starts_with("NAT에 의한 가상 서버의 작업 흐름"))
        .expect("독립 Print 6~7쪽을 잇는 원본 문단");
    let source_lines = &paragraphs[split_owner].line_segs;
    let cut = source_lines
        .windows(2)
        .position(|pair| pair[0].vertical_pos > 0 && pair[1].vertical_pos == 0)
        .expect("원본의 물리 쪽 경계")
        + 1;
    let mut rendered_text = String::new();
    for (page_index, range) in [(5, 0..cut), (6, cut..source_lines.len())] {
        let mut page_nodes = Vec::new();
        nodes(&trees[page_index], &mut page_nodes);
        let owned: Vec<_> = page_nodes
            .iter()
            .copied()
            .filter(|n| {
                n["type"] == "TextLine"
                    && n["pi"].as_u64() == Some(split_owner as u64)
                    && n["captionOwner"].is_null()
            })
            .collect();
        assert_eq!(
            owned.len(),
            range.len(),
            "원본 저장 경계에 따른 줄의 쪽 소유: {}쪽",
            page_index + 1
        );
        let body = page_nodes
            .iter()
            .copied()
            .find(|n| n["type"] == "Body")
            .expect("본문 상자");
        assert!(
            owned
                .iter()
                .all(|n| top(n) >= top(body) && bottom(n) <= bottom(body)),
            "분할 본문은 쪽 본문 안에 포함"
        );
        let images: Vec<_> = page_nodes
            .iter()
            .copied()
            .filter(|n| n["type"] == "Image" && n["pi"].as_u64() == Some(split_owner as u64))
            .collect();
        assert_eq!(
            images.len(),
            usize::from(page_index == 6),
            "문단 그림은 뒤 조각에 한 번"
        );
        if let Some(image) = images.first() {
            assert!(
                top(image) >= bottom(owned.last().unwrap()) && bottom(image) <= bottom(body),
                "뒤 그림은 이어받은 본문 아래에서 쪽 안에 포함"
            );
        }
        for line in owned {
            for run in line["children"].as_array().expect("줄의 원본 내용") {
                rendered_text.push_str(run["text"].as_str().expect("본문 글자"));
            }
        }
    }
    let compact = |text: &str| {
        text.chars()
            .filter(|c| !c.is_whitespace() && *c > '\u{001f}' && *c != '\u{fffc}')
            .collect::<String>()
    };
    assert_eq!(
        compact(&rendered_text),
        compact(&paragraphs[split_owner].text),
        "양쪽 조각의 원문을 누락·중복 없이 한 번 표시"
    );
}

#[test]
fn hwp3_sample_hwpx_16p() {
    // HWPX 변환본: 한컴 정답 16p
    assert_eq!(page_count("samples/hwp3-sample-hwpx.hwpx"), 16);
}

// ───────── HWP3 원본 회귀 0 (Task #460 보정과 충돌 없음) ─────────

#[test]
fn hwp3_sample_hwp3_16p() {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn nodes<'a>(node: &'a RenderNode, pred: &impl Fn(&RenderNode) -> bool) -> Vec<&'a RenderNode> {
        let mut result = Vec::new();
        if pred(node) {
            result.push(node);
        }
        for child in &node.children {
            result.extend(nodes(child, pred));
        }
        result
    }
    fn body_line(node: &RenderNode, pi: usize) -> bool {
        matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(pi) && line.caption_owner.is_none())
    }
    fn gap_hu(gap: f64) -> i32 {
        (gap / rhwp::renderer::hwpunit_to_px(1, 96.0)).round() as i32
    }

    // issue_265.hwp와 같은 바이트의 원본이다. 전쪽 Native/fresh 및 독립 Print 16쪽을 대조한다.
    let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample.hwp"))
        .expect("HWP3 원본 읽기");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWP3 원본 열기");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 16, "원본의 전체 쪽 소속");
    let paragraphs = &core.document().sections[0].paragraphs;

    let host = paragraphs
        .iter()
        .position(|para| {
            para.controls.iter().any(|control| {
                let Control::Picture(picture) = control else {
                    return false;
                };
                picture.caption.as_ref().is_some_and(|caption| {
                    caption
                        .paragraphs
                        .iter()
                        .any(|paragraph| paragraph.text.contains("가상 서버의 일반적인 구조"))
                })
            })
        })
        .expect("그림 캡션 원본 소유자");
    let (ci, picture) = paragraphs[host]
        .controls
        .iter()
        .enumerate()
        .find_map(|(ci, control)| {
            if let Control::Picture(pic) = control {
                Some((ci, pic))
            } else {
                None
            }
        })
        .expect("그림 개체");
    assert!(
        picture.caption.as_ref().expect("캡션").spacing > 0,
        "원본 그림 정보의 세로 캡션 띠에는 텍스트 외 간격이 있음"
    );
    let page3 = core
        .build_page_render_tree(2)
        .expect("그림과 뒤 본문이 있는 3쪽");
    let images = nodes(&page3.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Image(image)
        if image.para_index == Some(host) && image.control_index == Some(ci))
    });
    assert_eq!(images.len(), 1, "그림의 같은 쪽 소속과 중복 없음");
    let captions = nodes(&page3.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.caption_owner.is_some_and(|owner| owner.para_idx == host && owner.control_idx == ci))
    });
    assert_eq!(captions.len(), 1, "독립 Print와 같은 한 줄 캡션");
    assert!(
        captions[0].bbox.y > images[0].bbox.y + images[0].bbox.height,
        "캡션 간격을 지워 그림 하단에 붙이지 않음"
    );
    let blank = nodes(&page3.root, &|node| body_line(node, host + 1));
    assert_eq!(blank.len(), 1, "그림 뒤 빈 문단의 줄 점유");
    let host_line = &paragraphs[host].line_segs[0];
    let occupied = gap_hu(blank[0].bbox.y - captions[0].bbox.y - captions[0].bbox.height)
        - i32::from(picture.common.margin.bottom);
    assert!(
        occupied >= host_line.line_height + host_line.line_spacing,
        "그림 호스트의 저장 줄 진행은 캡션과 별도로 보존: {occupied}"
    );
    let after = nodes(&page3.root, &|node| body_line(node, host + 2));
    assert!(
        !after.is_empty() && after[0].bbox.y >= blank[0].bbox.y + blank[0].bbox.height,
        "뒤 본문은 빈 문단 뒤에서 시작"
    );

    let paragraph = paragraphs
        .iter()
        .position(|para| para.text.starts_with("NAT에 의한 가상 서버의 작업 흐름"))
        .expect("쪽을 잇는 원본 문단");
    let source = &paragraphs[paragraph].line_segs;
    let cut = source
        .windows(2)
        .position(|lines| lines[0].vertical_pos > 0 && lines[1].vertical_pos == 0)
        .expect("원본의 물리 쪽 경계")
        + 1;
    let mut rendered = Vec::new();
    for (page_index, range) in [(5, 0..cut), (6, cut..source.len())] {
        let page = core
            .build_page_render_tree(page_index)
            .expect("이어지는 본문 쪽");
        let lines = nodes(&page.root, &|node| body_line(node, paragraph));
        let indexes: Vec<usize> = lines
            .iter()
            .map(|node| {
                let RenderNodeType::TextLine(line) = &node.node_type else {
                    unreachable!()
                };
                line.line_index.expect("원본 줄 소유") as usize
            })
            .collect();
        assert_eq!(
            indexes,
            range.collect::<Vec<_>>(),
            "문단의 저장 쪽 경계와 누락/중복 없음"
        );
        rendered.extend(indexes);
    }
    assert_eq!(
        rendered.len(),
        source.len(),
        "이어진 문단의 모든 줄을 한 번씩 표시"
    );

    let page8 = core
        .build_page_render_tree(7)
        .expect("규칙 표와 뒤 본문의 8쪽");
    let tables = nodes(&page8.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(_))
    });
    assert_eq!(tables.len(), 5, "독립 Print의 규칙 표와 네 패킷 표");
    let mut checked = 0;
    for table_node in &tables {
        let RenderNodeType::Table(table) = &table_node.node_type else {
            unreachable!()
        };
        let pi = table.para_index.expect("본문 표 소유");
        let ci = table.control_index.expect("표 컨트롤 소유");
        let Control::Table(original) = &paragraphs[pi].controls[ci] else {
            panic!("원본 표")
        };
        let next = nodes(&page8.root, &|node| body_line(node, pi + 1));
        if next.is_empty() {
            continue;
        }
        let spacing = paragraphs[pi].line_segs[0].line_spacing;
        assert!(
            spacing > 0,
            "백분율 문단의 호스트 글꼴 후행 간격을 표 높이와 구분"
        );
        let gap = gap_hu(next[0].bbox.y - table_node.bbox.y - table_node.bbox.height)
            - i32::from(original.outer_margin_bottom);
        assert!(
            gap >= spacing,
            "표 뒤 본문이 호스트 후행 띠를 침범하지 않음: {gap} < {spacing}"
        );
        checked += 1;
    }
    assert_eq!(checked, 4, "같은 쪽의 표 뒤 네 본문 간격을 확인");
}

#[test]
fn hwp3_sample5_hwp3_64p() {
    // HWP3 원본
    assert_eq!(page_count("samples/hwp3-sample5.hwp"), 64);
}

// ───────── 광범위 회귀 0 (휴리스틱 false positive 없음) ─────────

#[test]
fn task554_no_regression_2022_kuglip() {
    // 2022년 국립국어원: 단순 -1600 적용 시 -5 회귀였던 케이스
    // 휴리스틱 (PS/CS 비율) 로 변환본이 아니라 정확히 분류 → 보정 미적용
    // [Task #643] 페이지 분할 드리프트 정정 + Task #404 vpos_end 트레일링 ls 제외:
    //   pi=80 (page 6) 트레일링 ls 정정 + pi=39 (page 3) heading-orphan 가드 정정
    //   → 후속 페이지 압축 → 40 → 35 페이지 (HWP 원본 정합 회복)
    assert_eq!(page_count("samples/2022년 국립국어원 업무계획.hwp"), 35);
}

#[test]
fn task554_no_regression_aift() {
    // aift: 단순 -1600 적용 시 -1 회귀였던 케이스
    // Task #874 #1~#8 누적 정합 결과 한컴 PDF (mydocs/pr/assets/issue7445/aift74/aift-2022.pdf) 와 동일한 74p.
    assert_eq!(page_count("mydocs/pr/assets/issue7445/aift74/aift.hwp"), 74);
}

#[test]
fn task554_no_regression_2025_donations_hwpx() {
    // 2025년 기부·답례품 HWPX: hwpml=1.5 (직접 작성) — 휴리스틱 미적용
    assert_eq!(
        page_count("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx"),
        30
    );
}

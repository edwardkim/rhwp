//! [#5566] 어울림(Square) 비-TAC 표의 문단 기준 세로 오프셋(vertOffset) 렌더 계약.
//!
//! horz_rel_to=Para 인 Square 표는 x 만 인라인 경로에서 계산되고 y 는 흐름
//! 시작점에 직결돼, vertOffset(PARA) 이 소실되면 표가 앵커 줄 첫 텍스트 위에
//! 얹힌다(20180108093532000 8쪽: 제목 줄과 1×7 절차 표 겹침 — 가로 오프셋만
//! 적용되고 세로는 0 고정, 10k 코퍼스 영향 64문서).
//!
//! 픽스처: 기존 표 샘플의 첫 표를 Square·Para·오프셋으로 변조해 HWPX 로
//! 내보낸다(HWP5 재저장은 새 배치 속성을 raw_ctrl_data 경로에서 유실하고,
//! create_empty 합성 문서는 HWPX 직렬화 전제 스타일 등록이 없다 — 실샘플
//! 변조 + HWPX 왕복이 이 계약을 태우는 최소 경로다). 판정은 CLI
//! `dump-extents` 의 첫 최상위 표 y — 오프셋 0 대비 1904HU(≈25.4px)만큼
//! 내려가야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::Control;
use rhwp::model::shape::{HorzRelTo, TextWrap, VertRelTo};
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-5566-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture(vert_offset: u32) -> PathBuf {
    let bytes = std::fs::read(repo_path("samples/hwp_table_test-m.hwp")).expect("샘플 읽기");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("샘플 파싱");

    // 첫 표의 위치를 찾는다.
    let mut loc = None;
    'find: for (si, section) in doc.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            if para.controls.iter().any(|c| matches!(c, Control::Table(_))) {
                loc = Some((si, pi));
                break 'find;
            }
        }
    }
    let (si, pi) = loc.expect("샘플에 표가 없다");

    // 이 계약은 **가시 텍스트 host** 에 앵커된 표가 대상이다(빈 host 는 별도
    // lane 경로가 정합 처리 — issue_4090 계약). host 문단에 앵커 텍스트를 넣는다.
    doc.insert_text(si as u32, pi as u32, 0, "앵커 제목 줄")
        .expect("host 텍스트 삽입");

    let para = &mut doc.document_mut().sections[si].paragraphs[pi];
    let table = para
        .controls
        .iter_mut()
        .find_map(|c| match c {
            Control::Table(t) => Some(t),
            _ => None,
        })
        .expect("표");
    table.common.treat_as_char = false;
    table.common.text_wrap = TextWrap::Square;
    table.common.vert_rel_to = VertRelTo::Para;
    table.common.horz_rel_to = HorzRelTo::Para;
    table.common.vertical_offset = vert_offset;

    let out = temp(&format!("voff{vert_offset}"));
    std::fs::write(&out, doc.export_hwpx_native().expect("HWPX 내보내기")).unwrap();
    out
}

/// 전 페이지 dump-extents 에서 첫 최상위 표의 y 시작과, 그 표 뒤 첫 최상위
/// 본문 TextLine 의 y 시작을 파싱한다(셀 내부 줄은 들여쓰기가 더 깊다).
fn first_table_and_next_line_y(path: &PathBuf) -> (f64, f64) {
    let out = Command::new(rhwp_bin())
        .args(["dump-extents"])
        .arg(path)
        .output()
        .expect("dump-extents 실행");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let mut table_y = None;
    let mut next_line_y = None;
    for line in text.lines() {
        let t = line.trim_start();
        if table_y.is_none() {
            if t.starts_with("Table ") {
                table_y = parse_y(t);
            }
            continue;
        }
        if next_line_y.is_none()
            && line.starts_with("      TextLine ")
            && !line.starts_with("       ")
        {
            next_line_y = parse_y(t);
            break;
        }
    }
    (
        table_y.unwrap_or_else(|| panic!("Table 노드 없음:\n{text}")),
        next_line_y.unwrap_or(f64::NAN),
    )
}

fn parse_y(line: &str) -> Option<f64> {
    let yp = line.find("y=")?;
    let rest = &line[yp + 2..];
    let end = rest.find("..")?;
    rest[..end].trim().parse::<f64>().ok()
}

#[test]
fn square_para_table_respects_positive_vert_offset() {
    let f0 = fixture(0);
    let f1 = fixture(1904); // 1904 HU ≈ 25.39px @96dpi

    // 배치 속성이 HWPX 왕복에서 살아남는지 선확인.
    let rp = HwpDocument::from_bytes(&std::fs::read(&f1).unwrap()).expect("재파싱");
    let rp_ok = rp.document().sections.iter().any(|s| {
        s.paragraphs.iter().any(|p| {
            p.controls.iter().any(|c| {
                matches!(
                    c,
                    Control::Table(t)
                        if !t.common.treat_as_char
                            && matches!(t.common.text_wrap, TextWrap::Square)
                            && matches!(t.common.vert_rel_to, VertRelTo::Para)
                            && t.common.vertical_offset == 1904
                )
            })
        })
    });
    assert!(rp_ok, "HWPX 왕복에서 배치 속성 유실");

    let (t0, n0) = first_table_and_next_line_y(&f0);
    let (t1, n1) = first_table_and_next_line_y(&f1);
    let _ = std::fs::remove_file(&f0);
    let _ = std::fs::remove_file(&f1);

    let delta = t1 - t0;
    let expected = 1904.0 * 96.0 / 7200.0;
    assert!(
        (delta - expected).abs() < 1.5,
        "Square·Para 표의 vertOffset 이 렌더 y 에 반영되지 않았다: delta={delta:.2}px \
         (기대 {expected:.2}px) — offset0 table_y={t0:.1}, offset1904 table_y={t1:.1}"
    );
    // 후속 흐름은 프로파일에 따라 저장 vpos(불변) 또는 흐름 전진(오프셋 동반,
    // 20180108093532000 실측)으로 갈린다 — 여기서는 후속 줄이 표 위로 끌려
    // 올라가지 않았다는 최소 불변식만 고정한다.
    if n0.is_finite() && n1.is_finite() {
        assert!(
            n1 >= n0 - 0.5,
            "표 뒤 본문 줄이 위로 끌려갔다: {n0:.2} -> {n1:.2}"
        );
    }
}

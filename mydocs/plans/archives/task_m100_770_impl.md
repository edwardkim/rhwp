# Task #770 구현 계획서

**Issue**: [#770](https://github.com/edwardkim/rhwp/issues/770)
**브랜치**: `local/task770` (stream/devel 베이스)
**수행 계획서**: [`task_m100_770.md`](task_m100_770.md)
**작성일**: 2026-05-10

---

## 1. TDD 전략

### 1.1 RED 테스트 (Stage 1)

**파일**: `tests/issue_770.rs` (신규)

**의도**: 페이지 2 의 본문 첫 paragraph (pi=37, "새 문서") 가 PDF 정합 y 좌표에 등장.

```rust
//! Issue #770: shortcut.hwp 페이지 2~7 헤더 TAC 표 spacing 누락
//!
//! 페이지 2 헤더 ('파일') ~ 본문 ('새 문서') 거리가 PDF 권위(한글 2022)
//! 대비 약 40 px 압축.
//!
//! 정합 동작: 페이지 2 의 pi=37 ('새 문서') 가 본문 영역 상단으로부터
//! 약 60 px 아래 (= 헤더 표 + spacing 합계) 에 등장.

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/basic/shortcut.hwp";
const TARGET_PI: usize = 37;
const TARGET_PAGE: u32 = 1; // 페이지 2 (0-based)
// PDF 페이지 2: '새 문서' y ≈ 142 px (100 dpi 기준), body_area.y=56.7, 즉 body 시작 후 ~85 px
// rhwp 측정: body_area.y=56.7, body 시작 후 paragraph y 측정.
// 헤더 표 (1x1) + 표 후속 spacing = 47.1 px (hwp_used). PDF 정합은 약 ?
const EXPECTED_BODY_OFFSET_MIN: f64 = 40.0; // 표 본체 23.5 px + 후속 ~17 px

fn find_first_textline_y(node: &RenderNode, target_pi: usize) -> Option<f64> {
    if let RenderNodeType::TextLine(tl) = &node.node_type {
        if tl.para_index == Some(target_pi) {
            return Some(node.bbox.y);
        }
    }
    for child in &node.children {
        if let Some(y) = find_first_textline_y(child, target_pi) {
            return Some(y);
        }
    }
    None
}

fn find_body_y(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y);
    }
    for child in &node.children {
        if let Some(y) = find_body_y(child) {
            return Some(y);
        }
    }
    None
}

#[test]
fn issue_770_page2_body_paragraph_below_header_zone() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap();
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap();
    let tree = doc.build_page_render_tree(TARGET_PAGE).unwrap();

    let body_y = find_body_y(&tree.root).expect("Body 노드 누락");
    let pi37_y = find_first_textline_y(&tree.root, TARGET_PI)
        .unwrap_or_else(|| panic!("pi={} 가 페이지 {} 에 없음", TARGET_PI, TARGET_PAGE));

    let offset = pi37_y - body_y;
    eprintln!(
        "[issue_770] page {} body_y={:.2} pi={}_y={:.2} offset={:.2} (expected min={})",
        TARGET_PAGE, body_y, TARGET_PI, pi37_y, offset, EXPECTED_BODY_OFFSET_MIN,
    );

    assert!(
        offset >= EXPECTED_BODY_OFFSET_MIN,
        "페이지 2 의 pi={} (본문 첫 paragraph) 가 body 상단으로부터 {:.2} px 위치. \
         PDF 권위 정합 최소값 {} px 미달 — 헤더 zone 압축 결함.",
        TARGET_PI, offset, EXPECTED_BODY_OFFSET_MIN,
    );
}
```

> `EXPECTED_BODY_OFFSET_MIN = 40` 은 보수적 경계. PDF 정합 정확값은 Stage 2 측정 후 확정.

### 1.2 GREEN 단계 (Stage 3)

가설 H1/H2/H3 중 Stage 2 결과로 결정. 우선 H1 (PartialParagraph y advance) 검증:

```rust
// paragraph_layout.rs::layout_partial_paragraph 또는 layout_composed_paragraph
// 의 line 1..2 처리에서 lh + ls 가산 확인
```

Stage 3 에서 instrument 결과로 정확한 정정 위치 확정.

---

## 2. 분석 도구 (Stage 2)

### 2.1 디버그 인스트루먼트

**환경변수**: `RHWP_TASK770_DEBUG=1`

**추가 위치**:

1. `layout.rs::PageItem::Table` (TAC 표 처리) — pi/ci/y_in/y_out/table_height/spacing
2. `layout.rs::PageItem::PartialParagraph` (line 1..2 처리) — pi/start_line/end_line/y_in/y_out/lh/ls
3. `paragraph_layout.rs:line advance` — pi/line_idx/y_before/y_after/lh/ls

**출력 포맷 예시**:
```
TASK770_TBL: pi=36 ci=1 y_in=56.70 y_out=80.20 table_h=23.50 outer_margin=0.00
TASK770_PP:  pi=36 lines=1..2 y_in=80.20 y_out=84.04 line_advance=3.83
TASK770_PP_LINE: pi=36 line_idx=1 y_before=80.20 y_after=84.04 lh=3.83 ls=0.00
```

GREEN 후 instrument 제거.

### 2.2 가설 검증 절차

1. RHWP_TASK770_DEBUG=1 로 페이지 2 trace 수집
2. 표 + PartialParagraph 의 y advance 합 = 27.3 px 확인
3. line 1 의 lh (2332 HU = 31.1 px) 가 어디서 잘리는지 식별
4. 가설 확정 후 Stage 3 정정

---

## 3. 단계별 산출물

| Stage | 파일 / 변경 | 검증 |
|-------|-----------|------|
| 0 | 수행 + 구현 계획서 | 작성 + 커밋 |
| 1 (RED) | `tests/issue_770.rs` 신규 | `cargo test --test issue_770` FAIL |
| 2 (분석) | `RHWP_TASK770_DEBUG` instrument | 트레이스 수집 + 가설 확정 |
| 3 (GREEN) | layout 또는 paragraph_layout 정정 | RED PASS, hwp_used 정합 |
| 4 (회귀) | `cargo test --release` + 골든 SVG | 회귀 0 |
| 5 (광범위) | 205 샘플 페이지 수 + 시각 검증 | 의도된 변경만 |
| 6 (최종) | 최종 결과 보고서 + close #770 + PR | `report/task_m100_770_report.md` |

---

## 4. 위험 완화 매트릭스

| 위험 | 단계 | 완화 |
|------|------|------|
| TAC 표 layout 변경으로 다른 샘플 회귀 | 4, 5 | 횡단 검증, 1x1 TAC 표 보유 샘플 직접 시각 검증 |
| pi=0 페이지 1 헤더 paragraph 영향 | 3 | 1x1 TAC 표 + ColumnDef 동반 케이스만 정정 (가드) |
| Task #9 fix_overlay 와 상호작용 | 3 | fix_overlay 분기 변경 안 함, 별도 spacing 산출 경로 |
| ColumnDef 처리 분기 회귀 | 3 | controls 가 ColumnDef + Table 둘 다 보유 시만 정정 |

## 5. 비범위

- 페이지 1 헤더 paragraph (pi=0) 처리 변경
- 1x1 TAC 표 가 아닌 표 처리 변경
- Task #768 column-break wrap-around 영역
- HWPX 별도 검증 — IR 변환 후 동일 경로

---

## 6. 환경 / 명령어

```bash
cargo build --release --bin rhwp

# 재현
cargo run --release --bin rhwp -- dump-pages samples/basic/shortcut.hwp -p 1
cargo run --release --bin rhwp -- export-svg samples/basic/shortcut.hwp -p 1 -o /tmp/x

# Stage 2 디버그
RHWP_TASK770_DEBUG=1 cargo run --release --bin rhwp -- dump-pages samples/basic/shortcut.hwp -p 1

# 회귀 테스트
cargo test --test issue_770 -- --nocapture
cargo test --release
```

# Task #768 구현 계획서

**Issue**: [#768](https://github.com/edwardkim/rhwp/issues/768)
**브랜치**: `local/task768` (stream/devel 베이스)
**수행 계획서**: [`task_m100_768.md`](task_m100_768.md)
**작성일**: 2026-05-10

---

## 1. TDD 전략

### 1.1 RED 테스트 (Stage 1)

**파일**: `tests/issue_768.rs` (신규)

**의도**: pi=94 ("<편집 화면 분할에서>") 가 등장하는 페이지가 **페이지 2 (page_index=2, 3쪽)** 임을 단언. 또는 페이지 3 의 다단 영역 안에 pi=94/pi=95 가 포함됨을 단언.

```rust
//! Issue #768: shortcut.hwp 페이지 3 끝 column-break 행이 페이지 4 첫 줄로 밀림
//!
//! PDF 권위 (한글 2022): pi=94 ("<편집 화면 분할에서>"), pi=95 ("화면 이동 Ctrl+W,N")
//! 가 페이지 3 의 다단 영역 안 (좌단 7행, 우단 7행) 으로 등장.
//!
//! 현재 결함: pi=94/95 가 페이지 4 의 첫 zone 으로 밀림.

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/basic/shortcut.hwp";
const TARGET_PI: usize = 94;

/// para_index == target_pi 인 TextLine 노드를 가진 페이지 인덱스 탐색.
fn find_para_page(doc: &rhwp::wasm_api::HwpDocument, target_pi: usize) -> Option<u32> {
    for pn in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(pn).expect("build_page_render_tree");
        if has_para(&tree.root, target_pi) {
            return Some(pn);
        }
    }
    None
}

fn has_para(node: &RenderNode, target_pi: usize) -> bool {
    if let RenderNodeType::TextLine(tl) = &node.node_type {
        if tl.para_index == Some(target_pi) {
            return true;
        }
    }
    node.children.iter().any(|c| has_para(c, target_pi))
}

#[test]
fn issue_768_pi94_appears_on_page3_not_page4() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));

    let page_idx = find_para_page(&doc, TARGET_PI)
        .unwrap_or_else(|| panic!("pi={} 가 어떤 페이지에도 등장하지 않음", TARGET_PI));

    eprintln!("[issue_768] pi={} 등장 페이지 = {} (page_count={})",
              TARGET_PI, page_idx, doc.page_count());

    // PDF 권위 (한글 2022) 정합: pi=94 는 페이지 인덱스 2 (3쪽) 에 등장해야 함.
    assert_eq!(
        page_idx, 2,
        "pi={} 가 page_index={} 에 등장. PDF 권위(2)와 불일치",
        TARGET_PI, page_idx,
    );
}
```

> RenderNodeType::TextLine 의 `para_index` 필드 확인 후 이름이 다르면 정정. (찾기: `grep "TextLineNode" src/renderer/render_tree.rs`)

### 1.2 GREEN 단계 — H1 적용 (Stage 3)

**파일**: `src/renderer/typeset.rs:261-273` (`advance_column_or_new_page`) 또는 `typeset.rs:417-424` (column-break 분기)

**가설 H1 — column-break wrap-around (마지막 단에서 col 0 으로 회귀)**:

```rust
// typeset.rs:417-424 영역 정정안
if para.column_type == ColumnBreakType::Column {
    if has_diff_col_def {
        self.process_multicolumn_break(&mut st, para_idx, paragraphs, page_def);
    } else if !st.current_items.is_empty() {
        // [Task #768] 다단 영역 안에서 마지막 단의 column-break:
        // PDF 권위(한글 2022) 동작은 col 0 으로 wrap-around (같은 다단 영역에 행 추가).
        // 페이지 잔여 공간 부족 시는 push_new_page (기존 동작).
        let is_last_col_in_multi = st.col_count > 1
            && st.current_column + 1 >= st.col_count;
        if is_last_col_in_multi {
            self.column_break_wrap_or_new_page(&mut st);
        } else {
            st.advance_column_or_new_page();
        }
    }
}
```

**보조 함수 신규** (`column_break_wrap_or_new_page`):

```rust
/// [Task #768] column-break 가 다단 영역 마지막 단에서 발생한 경우:
/// 페이지 잔여 공간이 충분하면 col 0 으로 wrap-around, 부족하면 새 페이지.
fn column_break_wrap_or_new_page(&mut self, st: &mut TypesetState) {
    self.flush_column();
    // 페이지 내 모든 column 의 used height 중 최댓값 산출
    // (= 다음 paragraph 가 col 0 시작 시 진입 가능한 y 위치)
    let used_max = st.pages.last()
        .map(|p| p.column_contents.iter()
            .map(|cc| cc.used_height)
            .fold(0.0_f64, f64::max))
        .unwrap_or(0.0);
    let zone_avail = st.layout.column_height - used_max;
    // 다음 paragraph 의 최소 height (1 line) 보다 잔여 공간이 크면 wrap-around
    const MIN_WRAP_HEIGHT_PX: f64 = 13.3; // 약 1 line height (1000 HU = 13.33 px @ 96 DPI)
    if zone_avail >= MIN_WRAP_HEIGHT_PX {
        // col 0 으로 wrap, current_height = used_max (다음 행 시작 위치)
        st.current_column = 0;
        st.current_height = used_max;
    } else {
        st.push_new_page();
    }
}
```

> 위 코드는 추정 (TypesetState/PageContent/ColumnContent 의 정확한 필드명 확인 후 정정).

**또는 더 단순한 안 (H1-simple)**:

```rust
fn advance_column_or_new_page(&mut self) {
    self.flush_column();
    if self.current_column + 1 < self.col_count {
        self.current_column += 1;
        self.current_height = self.pending_body_wide_top_reserve;
    } else if self.col_count > 1 {
        // [Task #768] 다단 영역 마지막 단 → col 0 wrap-around
        // (잔여 공간 검증은 후속 paragraph 의 fit 검사에 위임)
        self.current_column = 0;
        // current_height 유지 (col 0 의 used_height 기준 다음 행 시작)
        self.current_height = /* col 0 used_height */;
    } else {
        self.push_new_page();
    }
}
```

`flush_column` 의 동작 확인 후 결정.

---

## 2. 분석 도구 (Stage 2)

### 2.1 디버그 인스트루먼트

**환경변수**: `RHWP_TASK768_DEBUG=1`

**추가 위치**:

1. `typeset.rs:417` (column-break 진입) — `pi`, `column_type`, `has_diff_col_def`, `col_count`, `current_column`, `current_height`
2. `typeset.rs:261-273` (`advance_column_or_new_page`) — 이전/이후 `current_column`, `current_height`, action(advance/push_new_page)
3. `typeset.rs:255` `push_new_page` 진입점 — 호출 위치 stack trace

**출력 포맷 예시**:
```
TASK768_CB: pi=94 column_type=Column has_diff_col_def=false col_count=2 current_column=1 current_height=80.0
TASK768_ADV: action=push_new_page (last col, fallback) before=(col=1, h=80.0) after=(col=0, h=0.0, page+1)
TASK768_PAGE: pushed new page idx=3 from pi=94 column-break
```

GREEN 후 instrument 제거.

### 2.2 가설 검증 절차

1. RHWP_TASK768_DEBUG=1 로 page 0..3 trace 수집
2. pi=94 진입 시 col_count=2, current_column=1 확인
3. advance_column_or_new_page 가 push_new_page 호출 확인
4. H1 적용 후 wrap-around 동작 검증:
   - col 0 으로 회귀
   - current_height 가 col 0 의 마지막 paragraph 끝 위치
   - pi=94 → col 0 (좌단 7행), pi=95 → col 1 (우단 7행)
5. 페이지 4 시작은 pi=96 [다단나누기]

---

## 3. 단계별 산출물

| Stage | 파일 / 변경 | 검증 |
|-------|-----------|------|
| 0 | 수행 + 구현 계획서 | 작성 + 커밋 |
| 1 (RED) | `tests/issue_768.rs` 신규 | `cargo test --test issue_768` FAIL |
| 2 (분석) | `RHWP_TASK768_DEBUG` instrument | 트레이스 수집 + H1 확정 |
| 3 (GREEN) | `typeset.rs` column-break 분기 정정 | RED PASS, pi=94 페이지 인덱스=2 |
| 4 (회귀) | `cargo test --release` + 골든 SVG | 회귀 0 |
| 5 (광범위) | 169 샘플 페이지 수 + 다단 영역 횡단 | 의도된 변경만 |
| 6 (보고) | 최종 결과 보고서 + close #768 + PR | `report/task_m100_768_report.md` |

---

## 4. Stage 별 상세

### Stage 1 (RED)

1. `tests/issue_768.rs` 작성 — pi=94 페이지 인덱스 2 단언
2. `cargo test --test issue_768 -- --nocapture` 실행 → FAIL (현재 page_index=3)
3. `mydocs/working/task_m100_768_stage1.md` 보고서
4. 커밋

### Stage 2 (분석)

1. RHWP_TASK768_DEBUG instrument 추가 (3 위치)
2. trace 수집 → column-break 분기 결정 흐름 검증
3. wrap-around 적용 시 동작 시뮬레이션
4. (Stage 3 통합 커밋 — instrument 는 Stage 3 종료 시 제거)

### Stage 3 (GREEN)

1. `typeset.rs:417-424` 또는 `advance_column_or_new_page` 정정
2. `column_break_wrap_or_new_page` 보조 함수 추가 (또는 inline)
3. instrument 제거
4. RED PASS 확인 + 페이지 3 dump 확인 (단 1 = 7항목, 단 2 = 7항목)
5. 보고서 + 커밋

### Stage 4 (회귀)

1. `cargo test --release` 전체
2. 골든 SVG 7개 회귀 0
3. shortcut.hwp 페이지 3/4 SVG 시각 점검
4. 보고서 + 커밋

### Stage 5 (광범위)

1. 169 샘플 페이지 수 비교 (before / after)
2. 다단 영역 보유 샘플 식별 후 시각 검증:
   - `samples/basic/shortcut.hwp` (본 결함)
   - `samples/2022년 국립국어원 업무계획.hwp` (다단 영역 일부)
   - `samples/exam_*.hwp` (다단 시험지 등)
3. 보고서

### Stage 6 (최종)

1. 최종 결과 보고서 작성
2. closes #768 커밋
3. plans/archives/ 이동
4. (작업지시자 승인 후) `pr-task768` 브랜치 생성, origin push, PR 생성

---

## 5. 위험 완화 매트릭스

| 위험 | 단계 | 완화 |
|------|------|------|
| wrap-around 영구 루프 | 3 | 잔여 공간 ≤ MIN_WRAP_HEIGHT 시 push_new_page fallback |
| 다단 분배 알고리즘 회귀 | 4, 5 | 단단 (col_count=1) 케이스 영향 없음. col_count > 1 가드 |
| Distribute 다단 영역 회귀 | 5 | `samples/exam_korean.hwp` 등 Distribute 케이스 별도 시각 검증 |
| col 0 wrap 후 col 1 추가 진입 안됨 | 3 | wrap 후 다음 paragraph 분배 검증 — 다단 분배 자동 동작 유지 |
| process_multicolumn_break 영향 | 3 | has_diff_col_def 분기 변경 안 함 (zone 재정의 분리) |

## 6. 비범위

- `process_multicolumn_break` 동작 변경 (zone 재정의 케이스)
- 첫 단(col=0)에서의 column-break 동작 (기존 advance to col 1 유지)
- ColumnBreakType::MultiColumn / Page / Section 처리
- HWPX 별도 검증 — IR 변환 후 동일 경로
- pagination/engine.rs 의 동일 분기 (fallback 경로) — 본 task active 경로(typeset.rs)만

---

## 7. 환경 / 명령어

```bash
# 빌드
cargo build --release --bin rhwp

# 재현
cargo run --release --bin rhwp -- dump-pages samples/basic/shortcut.hwp -p 2
cargo run --release --bin rhwp -- dump-pages samples/basic/shortcut.hwp -p 3
cargo run --release --bin rhwp -- export-svg samples/basic/shortcut.hwp -p 2 -o /tmp/shortcut-p3
# 페이지 3 SVG 시각 확인

# PDF 권위 자료
pdftotext -layout -f 3 -l 4 pdf/basic/shortcut-2022.pdf -

# Stage 2 디버그
RHWP_TASK768_DEBUG=1 cargo run --release --bin rhwp -- dump-pages samples/basic/shortcut.hwp -p 2

# 회귀 테스트
cargo test --test issue_768 -- --nocapture
cargo test --release
```

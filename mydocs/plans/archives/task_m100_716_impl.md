# Task #716 구현 계획서

**Issue**: [#716](https://github.com/edwardkim/rhwp/issues/716)
**브랜치**: `local/task716` (integration/3pr-stack 베이스)
**수행 계획서**: [`task_m100_716.md`](task_m100_716.md)
**작성일**: 2026-05-08

---

## 1. TDD 전략

### 1.1 RED 테스트 (Stage 1)

**파일**: `tests/issue_716.rs` (신규)

**의도**: `samples/20250130-hongbo.hwp` page 0 (1쪽) 의 마지막 텍스트 줄(pi=15 line 2 추정) 이 컬럼 하단(≈1028.0 px) 이내에 배치됨을 단언.

**전략**: page 0 RenderTree 에서 모든 TextLine 노드를 수집한 뒤 `bbox.y + bbox.height` 의 최댓값이 컬럼 하단 이내인지 단언. pi 인덱스 하드코딩은 pagination 변경에 취약하므로 단순 "page 1 의 어떤 텍스트 줄도 컬럼 하단을 초과하지 않음" 형식.

```rust
//! Issue #716: hongbo page 1 마지막 줄 LAYOUT_OVERFLOW_DRAW
//!
//! 결함: pi=15 line 2 가 컬럼 하단 (1028.0 px) 을 +20.1 px 초과 → cropping.
//! 본질: 음수 line_spacing(ls<0) 미반영으로 layout y_offset drift 누적.

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/20250130-hongbo.hwp";

fn collect_text_line_bboxes(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if let RenderNodeType::TextLine(_) = &node.node_type {
        out.push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        collect_text_line_bboxes(child, out);
    }
}

fn find_column_bottom(node: &RenderNode) -> Option<f64> {
    // ColumnArea / Body 노드 등 컬럼 하단을 식별 — 실제 RenderNodeType 확인 후 결정.
    // 차선: 페이지 RenderTree 의 root.bbox.y + root.bbox.height - bottom_margin 추정.
    todo!("Stage 1 에서 page render tree 구조 확인 후 결정")
}

#[test]
fn issue_716_page1_last_text_line_within_column() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));

    let tree = doc.build_page_render_tree(0).expect("build_page_render_tree");

    let mut bboxes = Vec::new();
    collect_text_line_bboxes(&tree.root, &mut bboxes);
    assert!(!bboxes.is_empty(), "page 0 의 텍스트 줄 0건");

    let max_bottom = bboxes.iter().map(|(_, b)| *b).fold(f64::MIN, f64::max);
    let col_bottom = 1028.0_f64; // 297mm − 15mm − 10mm = 272mm × 96/25.4 = 1028.0 px (top+header+col_h)

    eprintln!(
        "[issue_716] page 0: text lines={} max_bottom={:.2} col_bottom={:.2}",
        bboxes.len(), max_bottom, col_bottom,
    );

    assert!(
        max_bottom <= col_bottom + 0.5,
        "page 0 의 텍스트 줄이 컬럼 하단 초과: max_bottom={:.2}, col_bottom={:.2}, overflow={:.2}",
        max_bottom, col_bottom, max_bottom - col_bottom,
    );
}
```

> Stage 1 작업 시 RenderTree API 의 정확한 col_bottom 산출 메서드 확인. col_area 가 노드 메타로 노출되어 있으면 그것을 사용, 아니면 col_bottom = 1028.0 하드코딩 (스펙 산출 결과).

### 1.2 GREEN 단계 — 하이브리드 C 점진 적용 (Stage 3)

**적용 순서** (영향 범위 좁은 것부터):

#### Step 3-A: H2-1 (lazy_base.max(0))

**파일**: `src/renderer/layout.rs:1488-1498`

**현재**:
```rust
let lazy_base = prev_vpos_end - y_delta_hu;
// lazy_base가 음수이면 자리차지 표 등으로 y_offset이
// vpos 누적보다 크게 밀린 것 → 역산 무효
if lazy_base < 0 {
    // 보정 건너뛰기: base를 vpos_end로 설정하여
    // end_y = col_area.y + 0 → 검증 실패 → 보정 미적용
    (prev_vpos_end, false)
} else {
    vpos_lazy_base = Some(lazy_base);
    (lazy_base, false)
}
```

**변경**:
```rust
let raw_lazy_base = prev_vpos_end - y_delta_hu;
// [Task #716] y_offset 이 vpos 인코딩 예측보다 forward drift 한 경우
// (raw_lazy_base < 0) 절대 vpos 좌표(lazy_base=0)로 회복 시도.
// 검증 단(MAX_BACKWARD_PX) 에서 과도한 backward 점프는 그대로 차단.
let lazy_base = raw_lazy_base.max(0);
vpos_lazy_base = Some(lazy_base);
(lazy_base, false)
```

**효과**: `lazy_base = 0` 이면 `end_y = col_area.y + vpos_end * scale`. 즉 절대 vpos 좌표로 회복. 단, MAX_BACKWARD_PX(=8.0) 검증으로 과도한 backward 점프는 여전히 차단.

#### Step 3-B: H2-2 (음수 trailing_ls 허용)

**파일**: `src/renderer/layout.rs:1482-1485`

**현재**:
```rust
let trailing_ls_hu = paragraphs.get(prev_pi)
    .and_then(|p| p.line_segs.last())
    .map(|s| s.line_spacing.max(0))
    .unwrap_or(0);
```

**변경**:
```rust
// [Task #716] 음수 line_spacing 도 trailing 보정에 반영.
// 한컴 스펙은 line_spacing 음수(Percent < 100% 등)를 허용하며, layout
// paragraph_layout 도 마지막 line 의 line_spacing 을 그대로 advance 한다.
// max(0) 으로 음수를 0 으로 잘라내면 lazy_base 가 |ls| HWPUNIT 만큼
// 어긋나 forward drift 의 기반이 된다.
let trailing_ls_hu = paragraphs.get(prev_pi)
    .and_then(|p| p.line_segs.last())
    .map(|s| s.line_spacing)
    .unwrap_or(0);
```

**효과**: `y_delta_hu` 계산 시 음수 ls 가 반영되어 `lazy_base` 가 정확히 산출.

#### Step 3-C: H1 (TAC 표 호스트 advance 정정 — 음수 ls 가산)

**대상**: `src/renderer/layout.rs:2235-2252` 영역 (TAC 표 호스트의 `outer_margin_top + table_h + outer_margin_bottom` advance) 또는 `paragraph_layout.rs` 의 host paragraph 처리.

**Stage 2 측정 결과**에 따라 정정 위치 확정. 후보:

(a) `layout.rs:2249-2251` outer_margin 가산 시점에 host paragraph 의 ls 를 차감:
```rust
if outer_margin_top_px > 0.0 {
    y_offset += outer_margin_top_px;
}
// [Task #716] TAC 호스트 paragraph 의 음수 line_spacing 을 advance 에 반영.
// HWP 인코딩: 다음 paragraph vpos = host vpos + lh + ls. layout 은 outer_margin
// + table_h + outer_margin 으로 advance 하여 ls 를 무시 → forward drift.
let host_ls_px = paragraphs.get(para_index)
    .and_then(|p| p.line_segs.first())
    .map(|s| hwpunit_to_px(s.line_spacing, self.dpi))
    .unwrap_or(0.0);
if host_ls_px < 0.0 {
    y_offset += host_ls_px; // 음수 → backward
}
```

(b) 표 layout 종료 시점에 ls 가산 (테이블 layout 함수 반환 직전):
- 정확한 위치는 Stage 2 결과로 결정.

> **유의**: `outer_margin_top_px` 가 1mm(=283 HU=3.78 px) 이고 host_ls 가 -8 px 인 경우, 둘이 거의 상쇄. 하지만 host_ls 의 음수 절댓값이 outer_margin 보다 커서 advance 가 음수가 되는 케이스는 비정상. 가드 추가:
```rust
if host_ls_px < 0.0 {
    // backward 가산은 outer_margin 가산분을 넘지 않도록 제한
    let max_backward = outer_margin_top_px;
    y_offset += host_ls_px.max(-max_backward);
}
```

#### Step 3-D (조건부): H2-3 가드 약화

Stage 4 회귀 결과에서 RED FAIL 잔존 시:
- `if !shape_jumped && !prev_tac_seg_applied` 가드를 `prev_tac_seg_applied` 단독 분리 후 H2-1+H2-2 적용 대상에서만 진입 허용.
- 또는 MAX_BACKWARD_PX 를 prev_tac_seg_applied=true 일 때 동적으로 확장.

> 단, **회귀 검증에서 H2-1+H2-2+H1 만으로 RED PASS 시 Step 3-D 는 적용하지 않는다** (소스 정정으로 drift 누적 차단되면 backward 보정 폭이 작아 자연스럽게 회수).

---

## 2. 분석 도구 (Stage 2)

### 2.1 디버그 인스트루먼트

**환경변수**: `RHWP_TASK716_DEBUG=1`

**추가 위치**:
1. `layout.rs:1383 영역` (column item 루프 진입 전후) — 각 item 진입 시 `pi`, `y_offset`, expected `col_y + vpos*scale`, drift
2. `layout.rs:2245-2252 영역` (TAC 표 호스트 outer_margin 추가 시점) — `pi`, `outer_margin_top`, `outer_margin_bottom`, `host_ls`, advance
3. `paragraph_layout.rs:2657 영역` (line advance) — `pi`, `line_idx`, `line_height`, `line_spacing`, `y_after`

**출력 포맷 예시**:
```
TASK716_ADV: pi=0 type=Table_TAC y_in=94.47 advance=47.46 y_out=141.93 hwp_delta_px=39.50 drift_step=+8.00 drift_acc=+8.00
TASK716_ADV: pi=1 type=Para_empty y_in=141.93 advance=20.00 y_out=161.93 hwp_delta_px=12.00 drift_step=+8.00 drift_acc=+16.00
...
```

GREEN 후 모두 제거 또는 환경변수 가드만 유지(컴파일 시 무비용).

### 2.2 가설 검증 절차

1. RHWP_TASK716_DEBUG=1 로 page 0 export → drift 측정
2. drift 발생 지점 확인:
   - 빈 문단(pi=1) drift 가 +8.0 px (= -ls_px) 면 paragraph_layout 이 음수 ls 를 어떻게든 무시하는 경로 있음 → 추가 추적
   - TAC 호스트(pi=0,2) drift 가 +8.0/+12.0 px (= -ls_px) 면 outer_margin+table_h+margin advance 가 host ls 무시 → H1 적용
3. H2-1+H2-2 만 적용 후 재측정 → drift_acc 변화 확인
4. H1 추가 적용 후 drift_acc 0 근방 수렴 확인
5. RED test PASS 확인

**Step 2-A — 빈 문단 음수 ls 미반영 원인 추적**:

`paragraph_layout.rs:2657` 는 명시적으로 `y += line_height + line_spacing_px` 이지만 실측에서 빈 문단 advance 가 +8 px drift 한다. 가능 원인:
- composer 가 빈 paragraph 의 ComposedLine 을 생성하지 않아 `composed.lines.is_empty()` 분기(line 2703 근방) 의 `default_height = 400 HU = 5.33 px` 사용
- 호출자(`layout.rs:layout_column_item`)가 paragraph_layout 을 호출하지 않고 host paragraph 를 다른 경로로 처리
- spacing_after 가 ls 와 합산되어 가산
- LineSpacingType::Percent 일 때 어딘가에서 음수가 0 으로 잘림

Stage 2 에서 정확한 원인을 instrument 로 추적 후 정정 위치 확정.

---

## 3. 단계별 산출물

| Stage | 파일 / 변경 | 검증 |
|-------|-----------|------|
| 0 | 수행 + 구현 계획서 | 작성 + 커밋 |
| 1 (RED) | `tests/issue_716.rs` 신규 | `cargo test --test issue_716` FAIL |
| 2 (분석) | `RHWP_TASK716_DEBUG` instrument 일시 추가, drift 위치 식별 | 분석 보고서 (working/) |
| 3 (GREEN) | layout.rs (Step 3-A, 3-B), 추가 위치(Step 3-C) | RED PASS, 페이지 1 SVG `LAYOUT_OVERFLOW_*` 0건 |
| 4 (회귀) | `cargo test --release` + 골든 SVG | 회귀 0 |
| 5 (광범위) | 181 샘플 페이지 수 + 음수 ls 보유 샘플 횡단 | 의도되지 않은 변경 0 |
| 6 (보고) | 최종 결과 보고서 + close #716 + pr-task716 PR | `mydocs/report/task_m100_716_report.md` |

---

## 4. Stage 별 상세

### Stage 1 (RED)

1. `tests/issue_716.rs` 작성 — page 0 의 모든 TextLine bbox bottom ≤ col_bottom 단언
2. `cargo test --test issue_716 -- --nocapture` 실행 → FAIL (max_bottom ≈ 1048.2)
3. `mydocs/working/task_m100_716_stage1.md` 보고서 작성
4. 타스크 브랜치 커밋

### Stage 2 (분석)

1. RHWP_TASK716_DEBUG 환경변수 가드 instrument 추가 (3 위치)
2. `RHWP_TASK716_DEBUG=1 cargo run --release --bin rhwp -- export-svg samples/20250130-hongbo.hwp -p 0 -o /tmp/x` 트레이스 수집
3. drift 발생 단계 분석: pi 별 drift_step / drift_acc
4. 빈 문단 drift +8 px 의 발생 경로 식별 (paragraph_layout.rs vs layout.rs vs composer.rs)
5. H1 정확한 적용 위치 확정 (Step 3-C 후보 a/b 결정)
6. `mydocs/working/task_m100_716_stage2.md` 분석 보고서

### Stage 3 (GREEN)

**Stage 3-A** (H2-1 lazy_base.max(0)):
1. layout.rs:1488-1498 변경
2. `cargo test --test issue_716` 실행 → 부분 개선 측정 (PASS 또는 FAIL 잔존)
3. SVG 출력 확인

**Stage 3-B** (H2-2 음수 trailing_ls):
1. layout.rs:1482-1485 변경
2. RED 재측정

**Stage 3-C** (H1 TAC 호스트 ls 가산):
1. Stage 2 결과의 정정 위치에 host_ls 가산 로직 추가
2. RED 재측정 → PASS

**Stage 3-D** (조건부 가드 약화):
- Stage 3-A/B/C 후에도 RED FAIL 잔존 시에만 진행

각 sub-step 후 `mydocs/working/task_m100_716_stage3.md` 에 변경 누적 기록. instrument 는 stage 3 종료 시 제거.

### Stage 4 (회귀)

1. `cargo test --release` 전체 통과 확인
2. 페이지 1 SVG 시각 점검 (`output/svg/`)
3. `RHWP_VPOS_DEBUG=1` 출력 확인 — `applied=true` 가 적절히 발동
4. 골든 SVG 회귀 (회귀 테스트 셋트 실행)
5. `mydocs/working/task_m100_716_stage4.md`

### Stage 5 (광범위)

1. 181 샘플 페이지 수 비교 (before / after)
2. 음수 ls 보유 샘플 (특히 TAC 표 다수) 식별 후 시각 검증:
   - `samples/20250130-hongbo.hwp` (본 결함)
   - `samples/2022년 국립국어원 업무계획.hwp` (TAC 표 다수)
   - `samples/2024_대학혁신지원사업.hwp` 등
3. PDF 권위 자료(있는 경우) 와 시각 정합 확인
4. `mydocs/working/task_m100_716_stage5.md`

### Stage 6 (최종)

1. 최종 결과 보고서 작성 (`mydocs/report/task_m100_716_report.md`)
2. closes #716 커밋
3. plans/archives/ 로 계획서 이동
4. (작업지시자 승인 후) `pr-task716` 브랜치 생성 (stream/devel 베이스), origin push, PR 생성

---

## 5. 위험 완화 매트릭스

| 위험 | 단계 | 완화 |
|------|------|------|
| H2-1 backward 점프 | 3-A | MAX_BACKWARD_PX(8.0) 검증 유지. 8 px 초과 점프는 applied=false |
| H2-2 양수 ls 케이스 회귀 | 3-B | 양수 ls 의 경우 max(0) == ls 이므로 동작 동일. 음수에서만 변화 |
| H1 outer_margin 음수화 | 3-C | `host_ls_px.max(-outer_margin_top_px)` 가드로 advance 음수 방지 |
| TAC 표 다수 보유 다른 샘플 회귀 | 4, 5 | 횡단 검증, PDF 권위 자료 비교 |
| LineSpacingType 별 동작 차이 | 2, 5 | Percent / Fixed / SpaceOnly / Minimum 모두 검증 (현재 hongbo 는 Percent<100%) |

## 6. 비범위

- `compute_line_spacing_hwp` (line_breaking.rs:807) `.max(0.0)` clamp 변경 — reflow 경로(편집기 시 텍스트 변경) 만 영향, 본 결함과 무관
- VPOS_CORR `!shape_jumped` 가드 변경 — 본 결함 외 (overlay shape 영역)
- `pagination/engine.rs` 의 페이지 분할 산출 변경 — drift 누적이 줄어 자연스럽게 페이지 수 변동 가능하나 의도된 변동
- HWPX 파일 별도 검증 — 본 결함은 HWP5 샘플이며 IR 변환 후 동일 layout 경로이므로 자연스럽게 적용
- 한컴 편집기 시각 출력 비교 (Linux 환경 미접근) — PDF 권위 자료(`pdf/`, `pdf-2020/`) 가 있는 샘플로 한정

---

## 7. 환경 / 명령어

```bash
# 빌드
cargo build --release --bin rhwp

# 재현
cargo run --release --bin rhwp -- export-svg samples/20250130-hongbo.hwp -p 0 -o /tmp/hongbo-page1
# (stderr 에 LAYOUT_OVERFLOW_DRAW)

# VPOS_CORR 디버그
RHWP_VPOS_DEBUG=1 cargo run --release --bin rhwp -- export-svg samples/20250130-hongbo.hwp -p 0 -o /tmp/x

# 본 task 디버그 (Stage 2 추가)
RHWP_TASK716_DEBUG=1 cargo run --release --bin rhwp -- export-svg samples/20250130-hongbo.hwp -p 0 -o /tmp/x

# 회귀 테스트 (Stage 1 RED, Stage 3-D GREEN)
cargo test --test issue_716 -- --nocapture

# 광범위 회귀 (Stage 4)
cargo test --release

# 페이지 배치 검사
cargo run --release --bin rhwp -- dump-pages samples/20250130-hongbo.hwp -p 0
```

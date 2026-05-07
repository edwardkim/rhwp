# Task #671 Stage 2 단계별 보고서 — 본질 정정

## 1. 정정 위치 (Stage 1 진단 결과 반영)

**옵션 B (별도 fallback 함수)** 으로 진행. 회귀 위험 좁힘 원칙 준수.

| 파일 | 변경 종류 | 변경 영역 |
|------|----------|----------|
| `src/renderer/composer.rs` | 신규 함수 추가 | `recompose_for_cell_width` + `split_composed_line_by_width` |
| `src/renderer/layout/table_layout.rs` | 호출 위치 추가 | 셀 paragraph 재구성 (`inner_width` 결정 후) |

## 2. composer.rs 신규 함수

### 2.1 `recompose_for_cell_width`

```rust
pub fn recompose_for_cell_width(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
)
```

**동작 영역 가드 (3 중)**:

1. `para.line_segs.is_empty()` — 한컴 인코딩 부재 케이스만
2. `composed.lines.len() == 1` — `compose_lines` fallback 단일 ComposedLine 결과만
3. 측정 폭 > `cell_inner_width_px` — 너비 안에 들어가면 분할 불필요

3 중 가드 미충족 시 `composed` 무변경 → **회귀 0 보장**.

### 2.2 `split_composed_line_by_width`

분할 전략:
- **단어 경계 (공백) 우선** — 공백 ' '/'\t' 위치에서 break
- **단어 단위 break** 가 max_width 초과 시 → **글자 단위 break** (CJK 텍스트 안전)
- 각 분할 줄의 메타데이터 (line_height/baseline/segment_width 등) **원본 보존**
- char_style 변경 (CharShapeRef 경계) 자동 반영

## 3. table_layout.rs 호출 위치

**위치**: `src/renderer/layout/table_layout.rs:1226-1234` — `inner_width` 결정 직후, AutoNumber(Page) 치환 직전.

```rust
// [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine 압축
// 결과를 셀 가용 너비 (inner_width) 에 맞춰 다중 ComposedLine 으로 재분할.
for (cpi, para) in cell.paragraphs.iter().enumerate() {
    if let Some(comp) = composed_paras.get_mut(cpi) {
        crate::renderer::composer::recompose_for_cell_width(
            comp, para, inner_width, styles,
        );
    }
}
```

**호출 시점**: 패딩 (`shrink_cell_padding_for_overflow`) 이후 `inner_width` 가 확정된 직후 → 정확한 가용 너비 기준 분할.

## 4. 결정적 검증

| 검증 영역 | 결과 |
|----------|------|
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ **6/6** |
| `cargo test --release --test issue_546` | ✅ **1/1** |
| `cargo test --release --test issue_554` | ✅ **12/12** |
| `cargo clippy --release` | ✅ 신규 경고 0 |
| `cargo build --release` | ✅ |

**결정적 검증 통과**: 회귀 0 + 신규 경고 0.

## 5. SVG 출력 변화 (계획서.hwp p1)

| 영역 | BEFORE (baseline) | AFTER (정정) |
|------|------------------|-------------|
| 파일 크기 | 238,314 bytes | 237,990 bytes (-324) |
| 신규 y 좌표 | (없음) | 385.4 / 406.7 / 428.0 등 (간격 21.3 px 일정) |

신규 y 좌표 등장 + 21.3px 간격 누적 = **다중 줄로 분할되어 vpos 정상 누적** 입증.

## 6. 회귀 위험 영역 좁힘

### 6.1 가드 명시

- `line_segs.is_empty()` — 정상 인코딩 paragraph 무영향
- `lines.len() == 1` — 다중 ComposedLine 정상 paragraph 무영향
- 측정 폭 ≤ width — 한 줄에 들어가는 텍스트 무영향

### 6.2 영향 영역

- **영향**: 셀 paragraph 중 line_segs 비어 있고 텍스트가 셀 너비 초과
  → `samples/계획서.hwp` 권위 영역
- **무영향**: 본문 paragraph (호출 자체 없음), HWPX 영역, 정상 line_segs 인코딩 셀

### 6.3 본문 paragraph 보호

`compose_paragraph` (기존 함수) 는 변경 없음. 본문 paragraph 처리 경로 (`compose_section`, `layout.rs` 의 `compose_paragraph` 호출) 무영향.

## 7. Stage 2 보완 (시각 판정 결과 반영)

작업지시자 시각 판정 결과 — 본질 정정 (줄겹침 해소) 은 통과 ✅. 다만 일부 셀 마지막 줄/paragraph 클립 발견. 측정/렌더링 일관성 차원에서 모든 측정 호출 위치 처리.

### 7.1 추가 정정 위치

| 파일:줄 | 용도 | 정정 |
|---------|------|------|
| `table_layout.rs:614/678` | `resolve_row_heights` measured 분기 fallback caller | inner_width 계산 + 함수 시그니처 변경 |
| `table_layout.rs:700` (callee) | `calc_cell_paragraphs_content_height` | recompose 호출 추가 |
| `table_partial.rs:94` | 분할 표 시작 행 측정 | inner_width 계산 + recompose 호출 |
| `table_partial.rs:358` | 분할 표 셀 layout | inner_width 결정 후 recompose 호출 |
| **`height_measurer.rs:527`** | **MeasuredTable 핵심 측정 (가로쓰기)** | **recompose 호출 추가** |
| **`height_measurer.rs:712`** | **MeasuredTable 병합 셀 측정** | **recompose 호출 추가** |

### 7.2 잔존 결함 — 별도 Issue 분리

본 task #671 정정 후 잔존 결함 (셀 [21] 3번째 줄 / 셀 [52] 3번째 paragraph 클립) 의 본질 진단:

- **본질**: `height_measurer.rs:822-830` TAC 표 비례 축소 메커니즘
- **결함**: 측정 row_heights 합 > common.height 시 비례 축소 → 셀 클립
- **별개 결함 영역**: Task #671 (line_segs 부재) 와 다른 본질 (TAC 표 권위 영역 불일치)

→ **Issue #672** 로 별도 등록 ([링크](https://github.com/edwardkim/rhwp/issues/672)).

`feedback_hancom_compat_specific_over_general` + 회귀 위험 좁힘 영역 정합 — 본 task #671 의 본질 영역 (line_segs 부재 → 줄겹침) 을 명시적으로 좁히고, 다른 본질 (TAC 표 비례 축소) 은 별도 task 영역으로 분리.

## 8. Stage 3 진행 영역

### 8.1 광범위 페이지네이션 회귀 sweep

`samples/` 폴더 전체 fixture (167+ HWP/HWPX) 페이지 수 차이 0 검증.

### 8.2 시각 판정 게이트웨이

`samples/계획서.hwp` 1페이지 시각 판정 (작업지시자):
- 셀 [13]/[21] 줄겹침 해소 확인
- 다른 셀 영역 회귀 0 확인

생성된 SVG: `output/task671_after/계획서.svg`

비교용 baseline: `output/task671_baseline/계획서.svg`

### 8.3 측정/렌더링 일관성 검토

광범위 sweep 결과에 따라:
- 페이지 수 차이 0 → 측정/렌더링 일관성 OK, 추가 처리 없이 Stage 3 완료
- 차이 발견 → table_layout.rs:709 / table_partial.rs 측정 위치 동기화

## 9. Stage 3 진행 승인 요청

본 Stage 2 정정 결과 + 결정적 검증 통과 + SVG y 좌표 누적 확인 후 **시각 판정 게이트웨이** + **광범위 회귀 sweep** Stage 3 진행 승인 요청.

작업지시자 시각 판정 → 통과 시 Stage 3 진행, 미통과 시 Stage 2 보완 영역 분석.

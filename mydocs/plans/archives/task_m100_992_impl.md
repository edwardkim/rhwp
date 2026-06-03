# 구현계획서 — task992: 페이지 영역 밖 콘텐츠 제거

- 타스크: 로컬 task992 (GitHub 이슈 미발행 — 비공개 문서 검증 사안)
- 브랜치: `local/task992` (`local/devel` `f7c31f3a`에서 분기)
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-19
- 선행 문서: `mydocs/plans/task_m100_992.md` (수행계획서, 승인 완료)

## 1. 조사 결과 (코드 분석)

`src/renderer/height_measurer.rs`의 `measure_table_impl`에서, 셀 콘텐츠 높이는 두 경로로 측정된다.

### 경로 A — `row_heights[r]` (행 높이)

`measure_table_impl` 2단계(약 line 572~757). 중첩 표 포함 셀은(line 712~727):

```rust
let content_height = if has_nested_table_in_cell {
    // 마지막 문단의 마지막 LINE_SEG의 vpos + line_height
    let last_seg_end = ... max(vpos + line_height) ...;
    hwpunit_to_px(last_seg_end, dpi).max(text_height)
} else { text_height + non_inline_h };
```

vpos 점프가 중첩 표 높이를 반영한다는 전제로 `last_seg_end`를 쓴다. 이 값이 `row_heights[r]` → `remaining_content_for_row`의 `max_content` 캡으로 쓰인다.

### 경로 B — `MeasuredCell.total_content_height` (셀 콘텐츠 높이)

`measured_cells` 빌드(약 line 1000~1153)에서 `total_content_height = line_sum`(전 문단 줄 높이 합). 그 직후 중첩 표 보정(line 1155~1181):

```rust
for mc in &mut measured_cells {
    if mc.has_nested_table {
        let nested_h = /* 셀 내 모든 Control::Table 의 total_height 합 */;
        mc.total_content_height = nested_h.max(mc.total_content_height); // ← 문제
    }
}
```

**핵심 결함**: 보정이 `max(중첩표 합, 텍스트 줄 합)`이다. 셀에 *텍스트 문단과 중첩 표가 모두* 있으면 두 콘텐츠는 세로로 **쌓이므로** 합산해야 하는데, 더 큰 쪽 하나만 채택해 작은 쪽(약 500px)을 통째로 누락한다.

`remaining_content_for_row`(line 1396~)는 `has_nested_table` 셀에 대해 `capped = total_content_height`를 그대로 쓰고, content_offset이 있으면 `effective_total = capped.min(max_content.max(line_sum))` 후 `effective_total - content_offset`을 잔량으로 반환한다.

### 재현 데이터 (비공개 샘플 181쪽)

`dump-pages`로 확인한 페이지 143 (`pi=308`, 2행×1열 분할 표 연속분):

```
=== 페이지 143 ===  body_area h=913.3
  PartialTable pi=308 ci=0 rows=1..2 cont=true 2x1 split_start=1642.5 split_end=0.0
  단 0 used=527.6px
```

행 1(문단 51개 + 다중 중첩표) `total_content_height ≈ 1780~1791px`로 측정 → content_offset 1642.5 이후 잔량 ≈137px → "현재 페이지에 들어감" 판정 → 추가 분할 없음. 그러나 렌더러는 같은 연속분을 약 660px로 그려 텍스트가 y≈1166(페이지 1122 초과)까지 내려간다. 실제 행 1 총 높이 ≈ 1642.5 + 660 ≈ 2300px → **측정이 약 510px 과소**.

`max(nested_h, line_sum)`가 한 성분(텍스트 줄 합 또는 중첩표 합)을 누락한 결과와 정확히 일치한다.

## 2. 수정 방침

경로 B의 `max`를 **합산**으로 교체하는 것이 핵심 수정이다.

- 중첩 표 호스트 문단의 LINE_SEG는 표 높이를 미포함(앵커 줄만, line 712~713 주석). 따라서 `line_sum`(텍스트 + 작은 앵커 줄들)과 `nested_h`(중첩 표 실제 높이)는 서로 겹치지 않는다 → 합산이 이중 가산을 일으키지 않는다.
- 셀이 중첩 표만 가진 경우(1x1 래퍼는 line 526~552에서 별도 unwrap) `line_sum`은 앵커 줄 수준으로 작아 `line_sum + nested_h ≈ nested_h` → 기존 `max` 결과와 사실상 동일(회귀 위험 낮음).
- 앵커 줄 높이만큼 미세 과대 측정이 남지만, 이는 페이지 경계를 **넘지 않는 방향**의 오차이므로 안전하다(수행계획서 §35 방어 취지 부합).

경로 A는 stage 1에서 `last_seg_end`가 실제 행 높이를 반영하는지 계측으로 확인한다. 경로 B를 합산으로 고쳐도 `remaining_content_for_row`의 `effective_total = capped.min(max_content.max(line_sum))`에서 `max_content`(경로 A 유래)가 작으면 다시 캡되므로, 경로 A도 과소면 함께 정정한다.

**가드 도입 여부**: 렌더러측 "페이지 경계 강제 분할" 가드는 분할 표 연속분 좌표계와 얽혀 회귀 위험이 크다. 측정 정정만으로 stage 3에서 오버플로 0에 도달하면 도입하지 않는다. 미달 시 재검토하여 별도 stage로 추가한다.

## 3. 단계 구성

### Stage 1 — 계측·확정

- `measure_table_impl`에 **비커밋 임시 디버그 출력**을 넣어 `pi=308` 행 1 셀의 `line_sum`, `nested_h`, `max` 결과, `row_heights[1]`, `last_seg_end` 기반 `content_height`를 출력.
- 페이지 171 분할 표(pi 확인 후)도 동일 계측.
- 확인 항목:
  1. 경로 B `total_content_height`가 `max`로 약 500px 과소인지 확정.
  2. 경로 A `row_heights[r]`(=`max_content`)가 실제 행 높이를 반영하는지 — 과소면 경로 A도 수정 범위에 포함.
- 임시 디버그 출력은 제거하고, 결과를 `working/task_m100_992_stage1.md`에 기록(소스 비커밋, stage1 보고서만 커밋).

### Stage 2 — 측정 정정

- 경로 B(line 1155~1181): `mc.total_content_height = nested_h.max(...)` → 텍스트 줄 합과 중첩 표 높이의 **합산**으로 교체. `has_nested_table` 셀의 `total_content_height = line_sum + nested_h`.
- Stage 1에서 경로 A 과소가 확인되면 `content_height`(line 718~727)의 중첩 표 분기도 정정.
- `remaining_content_for_row`의 `has_nested_table` 캡 로직(line 1419~1442) 주석을 새 의미(`line_sum + nested_h`)에 맞게 갱신. #362 외부 행 캡은 유지.
- 단위 테스트 추가: 텍스트 문단 + 중첩 표를 함께 가진 셀의 `total_content_height`가 두 성분의 합인지 검증(공개 합성 픽스처 또는 인메모리 `Table` 구성).
- `cargo test` 해당 모듈 + `cargo clippy` 무경고 확인 후 커밋.

### Stage 3 — 전체 검증 + 최종 보고서

- 비공개 샘플 181쪽 `export-svg` 후 전 페이지 `<text>` y좌표 > viewBox 높이 스캔 → **오버플로 0** 확인(현재 2건).
- `dump-pages`로 pi=308·페이지 171 표의 측정값이 렌더 높이와 일치하는지 재확인.
- `cargo test --release` 전체 통과 + 골든 SVG 회귀 0 + `cargo clippy --release` 무경고.
- 다른 공개 분할 표 샘플(예: `aift.hwp` 등) 교차 회귀 + 페이지 수 회귀 확인.
- WASM 영향(렌더러 수정) — 릴리즈 시 Docker 재빌드 필요 명시.
- `report/task_m100_992_report.md` 작성.

## 4. 영향 범위 / 리스크

- 수정 파일: `src/renderer/height_measurer.rs` (단일 파일 예상).
- `remaining_content_for_row`/`effective_row_height`는 `typeset.rs`·`pagination/engine.rs`가 호출하나, `pagination/engine.rs`의 `paginate_table_control`은 사문(死文)이라 실질 영향은 `typeset.rs` 라이브 페이지네이터.
- 회귀 위험: 중첩 표를 가진 모든 표 셀의 측정이 커진다. 중첩 표 전용 셀은 기존과 사실상 동일, 텍스트+표 혼재 셀만 증가 → 골든 SVG 테스트로 검출. 측정 증가는 분할을 더 보수적으로 만들어 페이지 수 +쪽 드리프트 가능 → stage 3에서 확인.
- 제외(수행계획서 §4): 페이지 수 누적 드리프트, HWP3, 파서.

## 5. 비공개 문서 처리

재현용 HWPX/PDF는 커밋하지 않는다. stage 1 임시 디버그 코드도 커밋하지 않는다. 회귀 테스트는 공개 합성 픽스처 우선, 불가 시 비커밋 처리 후 보고서에 명시한다(`orders/` 파일은 생성·수정하지 않음).

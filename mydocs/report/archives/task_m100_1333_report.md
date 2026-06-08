# 최종 결과보고서 — 단 구분선 이중 렌더 수정 (M100 #1333)

- 이슈: edwardkim/rhwp#1333
- 브랜치: `local/task1333`
- 관련 문서: 수행계획서 `plans/task_m100_1333.md`, 구현계획서 `plans/task_m100_1333_impl.md`,
  Stage 1 `working/task_m100_1333_stage1.md`, Stage 2 `working/task_m100_1333_stage2.md`

## 1. 문제

`3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` 의 가운데 단 구분선이 모든
페이지에서 **세로선 2개**로 중복 렌더되었다.

| 선 | y 범위 | 출처 |
|----|--------|------|
| ① | 90.7 → 1092.3 (body 전체 높이) | `build_column_separators` (page-level) |
| ② | 90.7 → 변동(콘텐츠 높이) | `build_columns` → `emit_zone_column_separators` (zone emit) |

PDF 정답지(한글 2022) 기준 ① (전체 높이 단일 실선) 이 정답.

## 2. 근본 원인

`src/renderer/layout.rs` 페이지 빌더가 단 구분선 경로 두 개를 모두 실행:

- `layout.rs:1116` `build_columns` → 내부 zone emit
- `layout.rs:1140` `build_column_separators` (page-level) — `!has_zone_specific_layout` 게이트

`build_columns` 의 zone emit 은 `zone_layout = col_content.zone_layout.unwrap_or(layout)`
(layout.rs:2531) 로 **zone-specific layout 이 없으면 page layout 을 폴백 사용**한다.
본 문서는 구역 전체가 처음부터 2단(`문단 0.0` 단정의 2단, 단나누기/구역전환 없음)이라
전 페이지 `zone_layout = None`:

- `has_zone_specific_layout = false` → page-level 이 그림 (정상)
- 그러나 build_columns 도 폴백 layout(2단, 구분선) 으로 emit → **이중 렌더**

즉 **페이지 전체가 단일 다단(zone-specific layout 부재)인 문서는 항상 이중 렌더**되었다.
(Issue #874 Case 4 가드가 zone emit 경로의 `unwrap_or(layout)` 폴백까지는 막지 못함)

## 3. 수정

`build_columns` 의 zone emit 을 **페이지 단위 `has_zone_specific_layout`** 술어로 게이트하여
page-level 가드(layout.rs:1139)와 정확히 **배타** 관계로 만들었다. (`src/renderer/layout.rs`)

```rust
// 함수 진입부
let has_zone_specific_layout = page_content
    .column_contents.iter().any(|cc| cc.zone_layout.is_some());
...
// emit 기록 조건
if has_zone_specific_layout
    && zone_layout.column_areas.len() >= 2
    && zone_layout.separator_type > 0
{ prev_zone_layout_for_sep = Some(zone_layout.clone()); ... }
```

| 페이지 유형 | has_zone_specific | page-level | build_columns | 결과 |
|------------|-------------------|------------|---------------|------|
| 단일 다단 (대상 문서·연속 페이지) | false | 그림(전체높이) | skip | 단일 ✓ |
| zone 혼재 (shortcut.hwp) | true | skip | 그림(zone별) | 정상 ✓ |

cc 단위 `is_some()` 대신 페이지 술어를 쓴 이유: zone 혼재 페이지에서 page-level 이
전역 skip 될 때, 그 페이지의 `zone_layout=None` 다단 cc 도 build_columns 가 그려
누락을 방지하기 위함 (혼재 케이스 무손실).

## 4. 검증

### 4.1 대상 문서
- HWP 23쪽: 단 구분선(x=396.9) **1개** (이전 2개), y=90.7→1092.3 전체높이 = PDF 정답 일치.
- HWPX 쌍: 동일 정상화.
- 시각 확인: 페이지 1 렌더가 PDF 와 일치.

### 4.2 회귀
- **shortcut.hwp (#874)**: 9개 구분선(1,1,2,1,1,2,1) 전수 보존, 영향 없음.
- 추가 다단 샘플 회귀 스캔 (단 구분선 중복/누락):
  exam_kor / exam_math / exam_social / hwp-multi-001 / interview / sungeo → **중복 0건**.
  k-water-rfp 의 17쪽 중복은 **1단 문서의 표/테두리(점선+실선 겹침)** 로 본 수정과 무관
  (수정은 `column_areas.len()>=2` 다단에만 작용, 1단 문서엔 영향 없음).

### 4.3 테스트 / 정적분석
- `cargo test --release`: **2107 passed, 0 failed** (issue_874 포함).
- `cargo clippy --release`: 경고/에러 없음.

## 5. v2 정정 (중요)

Stage 2 는 이중 렌더 두 후보선 중 **page-level(body 전체높이 고정)** 을 남겼으나,
작업지시자 지적("3 페이지 구분선이 짧아야 하는데 다른 페이지와 같음") 후 PDF 픽셀
측정 결과 **정답은 zone emit(콘텐츠 높이)** 임이 확인되었다.

| 페이지 | PDF end% | zone emit(콘텐츠)% | page-level(고정)% |
|--------|----------|-------------------|-------------------|
| 1 | 90.3 | 89.4 ✓ | 97.3 |
| 3 | 74.2 | 73.2 ✓ | 97.3 ✗✗ |
| 8 | 64.8 | 63.9 ✓ | 97.3 ✗✗ |
| 23 | 52.8 | 53.5 ✓ | 97.3 ✗✗ |

**v2 수정** (`task_m100_1333_v2.md`, `working/task_m100_1333_v2_stage1.md`):

1. Stage 2 게이트 되돌림 — zone emit 이 단일 다단 페이지도 콘텐츠 높이로 그림.
2. page-level `build_column_separators` 호출 + 함수 제거 (전체높이 고정 = 오답).
3. `emit_zone_column_separators` y_end 를 body 영역 하단으로 캡 — 꽉 찬 페이지에서
   prev_zone_y_end 가 body 를 초과해 구분선이 페이지 밖까지 그려지던 결함(p22 105%) 정정.

검증: 23쪽 단일 구분선 + 콘텐츠 높이(PDF ±1.5%, 페이지 3 짧음 재현, 페이지 밖 초과 없음),
shortcut.hwp 9개 보존, 추가 샘플 중복 0, `cargo test` 2107 passed, clippy 무경고.

## 6. 결론

단 구분선을 **zone emit 단일 경로(콘텐츠 높이, body 하단 캡)** 로 통일하여 이중 렌더를
제거하고, 부분 페이지의 짧은 구분선을 PDF 정답과 일치시켰다. 전체 테스트 회귀 없음.

## 7. 변경 파일

- `src/renderer/layout.rs`
  - (Stage 2, 되돌림) `build_columns` zone emit 게이트
  - (v2) page-level `build_column_separators` 호출 + 함수 삭제
  - (v2) `emit_zone_column_separators` y_end body 하단 캡
- 문서: 계획서 3(v2 포함), Stage 보고서 3(v2 포함), 최종 보고서 1

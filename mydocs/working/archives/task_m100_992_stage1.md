# Stage 1 완료보고서 — task992: 페이지밖 콘텐츠 조사

- 타스크: 로컬 task992 / 브랜치 `local/task992`
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-19
- 단계: Stage 1 — 조사 (`height_measurer` 과소 계산 지점 확정)

## 1. 방법

`measure_table_impl`에 비커밋 임시 디버그 출력(`RHWP_DEBUG_MEASURE`)을 넣어 분할 표 셀의 측정 성분(`line_sum`/`nested_h`/`row_heights`/`total_height`)을 덤프하고, `export-svg`로 실제 렌더 좌표를 측정해 대조했다. 디버그 코드는 조사 후 전량 제거(`git diff` 무변경 확인).

## 2. 페이지 143 — 확정: 중첩 표 셀 per-cell `max` 결함

분할 표 `pi=308`(2행×1열, 행 1 = 문단 51개 + 다중 중첩표). `dump-pages` 페이지 143:

```
PartialTable pi=308 ci=0 rows=1..2 cont=true 2x1 split_start=1642.5 split_end=0.0
```

계측 결과(행 1 셀):

| 항목 | 값 |
|------|-----|
| `line_sum` (텍스트 줄 높이 합) | 1774.4px |
| `nested_h` (중첩 표 높이 합) | 646.2px |
| `total_content_height` = `max(line_sum, nested_h)` (현재) | **1774.4px** |
| `line_sum + nested_h` (성분 합) | 2420.7px |
| `row_heights[1]` (경로 A) | 1791.0px |
| 실제 렌더 연속분 (SVG `<text>` y 113.4→1165.9) | ≈1052px → 셀 전체 ≈ content_offset 1642.5 + 1052 ≈ **2695px** |

페이지 143 SVG: `<text>` 최대 y = **1165.9** (viewBox 1122.5, body 하단 1026.7) → 페이지 밖 ~43px, 본문 밖 ~139px 오버플로 확인.

### 원인

`measure_table_impl`이 *텍스트 문단 + 중첩 표를 함께 가진 셀*을 **per-cell `max`** 로 측정한다 — 두 곳:

- **경로 B** `total_content_height` (line 1179): `nested_h.max(line_sum)`.
- **경로 A** `row_heights`의 `content_height` (line 718~727, 중첩 표 분기): `last_seg_end.max(text_height)` — `last_seg_end`(1791px)가 중첩 표 vpos 점프를 반영하지 못해 `line_sum`과 사실상 동일.

렌더러(`table_partial.rs` · `calc_cell_remaining_content_height`)는 **per-paragraph** 로 처리한다 — 문단별로 `중첩표면 max(nested, line) / 아니면 line`을 계산해 **합산**한다(`table_layout.rs:3040~3055`). 텍스트와 중첩 표는 세로로 쌓이므로 합산이 정답. 페이지네이터의 per-cell `max`는 둘 중 작은 성분(646px)을 통째로 누락한다.

추가 발견 — `max`를 성분 합(2420.7)으로 고쳐도 실제 렌더(≈2695)에 ~275px 못 미친다. `nested_h`(646.2) 자체가 중첩 표의 `measure_table_impl().total_height`이고, `total_height`는 경로 A `row_heights`에서 나오므로, **중첩 표들도 같은 per-cell `max` 결함으로 재귀적으로 과소 측정**된다. 즉 경로 A·B 양쪽을 per-paragraph 합산으로 고치면 재귀적으로 모두 정정된다.

또한 `remaining_content_for_row`의 줄 단위 스냅 경로(line 1444~1466)는 `line_heights`만으로 잔량을 계산하는데 `line_heights`에는 중첩 표 높이가 없다 → 중첩 표 셀은 이 경로를 쓰면 안 되고, 경로 A(`max_content`) 캡도 정정된 값 아래로 깎으면 안 된다.

→ **Stage 2 수정 범위**: 경로 A·B의 per-cell `max`를 per-paragraph 합산으로 교체(재귀 포함) + `remaining_content_for_row`의 중첩 표 셀 분기 정정. 단일 파일 `height_measurer.rs`.

## 3. 페이지 171 — 별개 결함: 페이지보다 큰 부동(floating) 표

페이지 171 SVG: `<text>` 최대 y = **1180.1**, 표 테두리 rect 하단 **1491.6** → 본문 밖 대규모 오버플로.

`dump-pages`는 `Shape pi=533`으로 표기하나, `--debug-overlay`·`dump`로 확인하니 **`pi=533` = 32행×7열 표**다(`wrap=글앞으로` 부동 개체, `treat_as_char=false`, 선언 크기 47913×53096 HU ≈ 169×187mm ≈ **708px**). 렌더러는 이 표를 ≈1378px로 그려 선언 높이의 약 2배가 된다(셀을 실제 텍스트 콘텐츠 높이로 렌더 → 행별 선언 높이 초과).

이는 페이지 143과 **근본 원인이 다르다**:

- 페이지 143: 분할 표 연속분의 *중첩 표 셀* 측정 결함 (`height_measurer` per-cell `max`).
- 페이지 171: *부동 표*(쪽 분할 불가)의 **렌더 높이 ≠ 선언 높이** 불일치. 중첩 표 무관, `height_measurer`의 중첩 표 분기와 무관.

수행계획서 §3은 두 건을 동일 원인으로 인계했으나, 조사 결과 페이지 171은 별개 결함이다. (PDF 권위 자료 대조로 "렌더러 과대 vs 한컴도 오버플로"를 추가 판정해야 함 — Stage 1 범위 밖.)

## 4. 검증

- `cargo build --release` 정상, `git diff` 무변경(임시 디버그 전량 제거).

## 5. 다음 단계 — 작업지시자 결정 요청

페이지 171이 별개 결함으로 확인되어 task992 범위를 어떻게 둘지 결정이 필요하다(상세는 승인 요청 메시지 참조).

- 안 A: task992는 페이지 143(중첩 표 측정)만 수정 → Stage 2 진행. 페이지 171은 별도 타스크.
- 안 B: task992 범위를 확대해 페이지 171(부동 표 렌더 높이)도 포함.

# 최종 결과보고서 — task992: 페이지 영역 밖 콘텐츠 제거

- 타스크: 로컬 task992 (GitHub 이슈 미발행 — 비공개 문서 검증 사안)
- 브랜치: `local/task992` (`local/devel`에서 분기)
- 마일스톤: M100 (v1.0.0)
- 기간: 2026-05-19
- 결과: **Stage 3 완료·커밋(페이지 171 해결). 페이지 123·144·176은 렌더러 구조적 결함으로 확정·인계.**

## 1. 배경 및 목표

task991에서 비공개 샘플 HWPX를 SVG로 내보낼 때 콘텐츠가 페이지 본문 영역을 넘어 그려지는 현상이 남아, 본 타스크로 분리했다. 작업지시자 지시: **페이지 영역을 넘어 그려지는 경우가 없도록(페이지밖 콘텐츠 0)**.

시작 시점 오버플로(전수 스캔, `<text>` y > viewBox 높이): **4건** — 페이지 122·143·171·173. (수행계획서는 143·171 2건으로 봤으나, 0.5px 임계 전수 스캔에서 122·173의 ~1px 초과도 포함된다.)

## 2. 완료 — Stage 3: 페이지보다 큰 부동 표 데코레이션 오분류 (`a4c7e823`)

페이지 171의 `pi=533`은 HWPX 원본 속성이 `textWrap="IN_FRONT_OF_TEXT" pageBreak="CELL" repeatHeader="1"`인 32행×7열 표다. `typeset.rs`가 `InFrontOfText`/`BehindText` 표를 **무조건 데코레이션**(`PageItem::Shape`)으로 분류해 페이지 분할을 하지 않아, 페이지보다 큰 이 표가 한 페이지에 통째로(≈1378px, 본문 913px 초과) 그려졌다. 한컴 2022 PDF는 이 표를 쪽 분할+머리행 반복으로 렌더한다.

수정: 데코레이션 단축 분기에서 **본문보다 큰 다행(多行) 표를 제외**(`row_count > 1 && measured.total_height > base_available_height`)해 정상 페이지네이션을 타게 했다. 판별자로 `page_break`·`repeat_header`는 부적합 — 글뒤로 1×1 래퍼 데코레이션(`calendar_year.hwp`)도 그 비트를 갖기 때문이며, Issue #703 회귀로 확인했다.

검증: `cargo test --release` **1301 passed, 0 failed**(`issue_703` 포함), `cargo clippy` 무경고. 오버플로 4건 → 3건(페이지 171 제거).

## 3. 미해결 — 페이지 123·144·176: 렌더러 분할 표 위치 누적 불일치

남은 3건(`pi=308`·`pi=108`·`pi=111` 등 분할 표 연속분)은 모두 동일 근본 원인이며, 페이지네이터가 아니라 **렌더러 내부 결함**임을 단계적으로 확정했다.

### 진단 경과

- **Stage 2 (v1·v2)**: `height_measurer`의 중첩 표 셀 측정 보정 시도 → 비-TAC 중첩 표 이중 가산으로 `test_634` 회귀, 되돌림.
- **Stage 4 (v3)**: 페이지네이터 셀 높이 측정을 렌더러 `calc_cell_remaining_content_height`로 통일 → 72쪽 이동했으나 오버플로 불변, 되돌림.
- **Stage 5 (v4)**: 페이지네이터 분할 위치 측정을 렌더러 `compute_cell_line_ranges`+`calc_visible_content_height_from_ranges`로 통일 → 오버플로 불변, 되돌림.
- **Stage 6 (v5)**: `table_partial`의 중첩 표 가시성을 줄 범위로 통일 시도 → 해당 분기 미발현(no-op). 이어 `calc_visible_content_height_from_ranges`에 `compute_cell_line_ranges`와 동일한 `[Task #700]` vpos 동기화(+vpos 리셋 처리)를 이식 → 그 함수는 ≈546px → ≈1011px로 정정(렌더 실제 ≈1018px와 일치). 그러나 페이지네이터 측 헬퍼를 함께 적용하니 **무한 루프 발생** → 전량 되돌림.

### 확정된 근본 원인

렌더러의 분할 표 렌더 경로에 **상호 불일치하는 누적 위치 계산이 최소 3종** 공존한다.

1. `compute_cell_line_ranges` — `cum` 누적. `[Task #700]` LINE_SEG.vpos 절대 동기화 + vpos 리셋(셀 내 페이지 분할) 처리 포함.
2. `calc_visible_content_height_from_ranges_with_offset` — `cum_pos` 누적. vpos 동기화·리셋 처리 **없음** → 가시 중첩 표를 offset 이전으로 오판해 0 으로 계산.
3. `table_partial`의 실제 셀 렌더 — `content_y_accum`/`para_y` 누적. 중첩 표 호스트 문단을 앵커 줄 높이로만 전진.

`compute_cell_line_ranges`로 줄 범위를 정하고 `calc_visible_content_height_from_ranges`로 그 높이를 재면 같은 값이어야 하나, vpos 동기화 유무로 어긋난다. 계측 결과 `pi=308` 페이지 144 연속분: 줄 범위 함수 ≈546px, 실제 렌더 ≈1018px. `calc_visible`에 vpos 동기화를 이식하면 ≈1011px로 일치하나(2번 정정), 그 함수가 정확해져도 페이지네이터의 분할 모델 자체가 깨진다(아래).

### 더 깊은 결함 — 높이 기반 `content_offset` 모델과 vpos 리셋의 비양립

`pi=308` 셀은 HWP가 셀 내부 페이지 분할을 **LINE_SEG.vpos 리셋**으로 인코딩한 케이스다. `compute_cell_line_ranges`는 리셋 이후 콘텐츠를, 페이지네이터가 준 `content_offset`(px 높이)이 아무리 커도 **항상 visible 로 표시**한다(리셋이 hard cap). 즉 "`content_offset`를 키우면 연속분이 줄어든다"는 페이지네이터의 단조(monotonic) 가정이 성립하지 않는다.

그 결과 페이지네이터가 연속분을 렌더러 함수로 정확히 측정하더라도, 리셋 이후 콘텐츠를 분할하려 하면 `content_offset`가 무한히 커지며 연속분이 줄지 않아 **무한 루프**에 빠진다(v5 후속 시도에서 실측). 페이지네이터의 분할 표 모델은 *연속적 px offset* 기반이고, 렌더러의 줄 범위 모델은 *이산 + vpos 리셋 hard break* 기반이라 — 두 모델이 근본적으로 다르다.

### 해결 방향 (후속 타스크 인계)

두 가지가 함께 필요하다.

1. **렌더러 누적 위치 단일 기준화**: `compute_cell_line_ranges`·`calc_visible_content_height_from_ranges`·`table_partial`의 실제 셀 렌더가 같은 누적 규칙(vpos 동기화·리셋 처리 포함)을 공유. (`calc_visible`에 vpos 동기화 이식은 본 타스크에서 검증 완료 — 그 함수는 정정 가능.)
2. **페이지네이터 분할 표 모델 교체**: 연속적 px `content_offset` 기반을 버리고, 렌더러와 동일한 *줄 범위(line-range) + vpos 리셋 hard break* 기반으로 분할 위치를 결정. vpos 리셋을 자연 분할점으로 인식해야 무한 루프·과소 측정이 없다.

이는 렌더러 최복잡 파일(`table_partial.rs`)·`table_layout.rs`·페이지네이터(`typeset.rs`)에 걸친 구조 정비이며, 분할 표 문서의 골든 SVG가 광범위하게 이동하므로 페이지별 한컴 PDF 대조 검증이 수반된다. 별도 전용 타스크 권장.

## 4. 검증

- `cargo test --release` 전체 1301 passed, 0 failed. `cargo clippy --release` 무경고. (Stage 3 커밋 기준)
- 비공개 샘플 184쪽 전수 오버플로 스캔: 4건 → **3건**.

## 5. WASM

Stage 3 수정은 `typeset.rs`(페이지네이터)에 있어 WASM 빌드에 영향. 릴리즈 시 Docker로 WASM 재빌드 필요.

## 6. 비공개 문서

재현용 HWPX/PDF는 비공개 문서로 커밋하지 않았다. 테스트는 공개 골든 SVG·공개 샘플(`calendar_year.hwp`, `aift.hwp` 등)로 검증했고, 비공개 픽스처 기반 테스트는 추가하지 않았다.

## 7. 남은 사안 (신규 타스크 인계)

- **렌더러 분할 표 누적 위치 통일**: §3의 3종 누적 계산을 단일 기준으로 통일 → 페이지 123·144·176 오버플로 해소. 렌더러 정비 + 골든 회귀 대조.
- **페이지 수 드리프트** (SVG 184 vs PDF 179): 문서 전반의 누적 측정 정밀도 차. 별도 사안(수행계획서 §4 제외 항목).

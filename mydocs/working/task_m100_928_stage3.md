# Task #928 Stage 3: Fix 구현 + 단위 회귀 차단

## 1. 적용한 변경

**파일**: `src/renderer/layout/table_layout.rs`

**위치**: `layout_table_cells` 의 Shape 분기 (1812-1963 → 1812-1990, ~30 라인 증가)

**변경 요약**:

1. Shape 분기 진입 직후 (라인 1814 다음) `will_render_inline` 가드 계산 추가 — Picture 분기 (라인 1698) 와 동일 패턴.
2. 기존 `text_before` 추출/발행 블록 (1858-1949) 을 `if !will_render_inline { ... }` 로 감쌈.
3. 도형 렌더링 좌표 결정 시 `will_render_inline=true` 면 `tree.get_inline_shape_position()` 으로 paragraph_layout 이 등록한 좌표 사용, false 면 기존 `inline_x, tac_img_y` 사용.
4. `layout_cell_shape` 호출 자체는 양쪽 경로에서 동일 — 도형 자체 사라짐 회귀 차단.

트레일링 텍스트 블록 (라인 2158-2231) 은 별도 수정 없음 — Shape 분기에서 `prev_tac_text_pos` 갱신을 가드 안에 두었기 때문에, 단일 inline Shape 케이스에서는 `prev_tac_text_pos = 0` 유지되어 트레일링 블록이 진입하지 않는다. 다중 shape 혼재 케이스는 Stage 4 에서 검사.

## 2. 검증 결과

### 2.1 빌드

```
cargo build --release --bin rhwp
    Finished `release` profile [optimized] target(s) in 1m 24s
```

clean 빌드 (warning 0건 추가).

### 2.2 회귀 시각 검증

`samples/exam_kor.hwp` 5쪽 다이어그램 행 SVG 비교:

| 좌표 | Before (회귀) | After (Fix) |
|------|--------------|-------------|
| y=421.73 | (가) 246, ⇨ 279, ⇨ 386, (나) 412 | (가) 246, ⇨ 279, ⇨ 386, (나) 412 |
| y=423.12 | **(가) 299, ⇨ 332, ⇨ 439, (나) 465** (중복) | ❌ 없음 |
| y=420.63 (Before) / y=427.30 (After) | "A 단계" 379-393 (font 6.88) | "A 단계" 326-341 (font 6.88) |

- 다이어그램 행 paragraph 텍스트: **단일 baseline** (`y=421.73`) 에 `(가) ⇨ ⇨ (나)` 만 emit
- 사각형 안 "A 단계" inner text: 위치 변경 (379-393 → 326-341), 단 두 ⇨ 사이 (279.88, 386.32) 의 gap 안에 정상 배치
- 사각형 geometry 자체: 정상 렌더 (사라짐 회귀 없음)

### 2.3 자동 테스트

```
cargo test --release
test result: ok. 1275 passed; 0 failed; 2 ignored
test result: ok. 8 passed; 0 failed (svg_snapshot)
test result: ok. 1 passed; 0 failed (tab_cross_run)
... 전체 통과
```

`tests/svg_snapshot.rs::issue_617_exam_kor_page5` (exam_kor.hwp 6페이지 검증) 통과. 본 fix 는 5페이지 변경이지만 6페이지 골든도 영향 없음 확인.

## 3. 사각형 위치 검토

Before / After 의 사각형 inner text "A 단계" x 좌표:

- Before: 379-393 (두 ⇨ 사이의 두 번째 baseline 패턴 가운데)
- After: 326-341 (두 ⇨ 사이 [279.88, 386.32] 의 gap 중 좌측 ~17% 위치)

Gap 폭 = 386.32 - 279.88 - (⇨ 폭 약 15.5) ≈ 91 px. "A 단계" inner text 폭 ≈ 15 px. paragraph_layout 의 `set_inline_shape_position` 이 gap 좌측에 도형 origin 을 두는 것으로 보임. 한컴 정답 비교는 Stage 4 에서.

## 4. 위험 점검

| 위험 | 검증 결과 |
|------|----------|
| `get_inline_shape_position` 반환 None → 도형 사라짐 | fallback (`inline_x, tac_img_y`) 으로 처리, 본 케이스에서는 None 반환 미발생 |
| 텍스트 발행 스킵으로 trailing text 누락 | layout_composed_paragraph 가 run_tacs split 으로 양쪽 텍스트 모두 emit — 누락 없음 |
| 다른 표 셀 inline shape 케이스에서 회귀 | Stage 4 시각 회귀 검사 예정 |
| `prev_tac_text_pos` 미갱신으로 인한 다중 shape 케이스 트레일링 회귀 | 단일 shape 케이스 (회귀 대상) 에서는 영향 없음, 다중 shape 케이스 Stage 4 검사 |

## 5. Stage 4 진입 조건

- ✅ exam_kor.hwp 5쪽 다이어그램 행 단일 baseline 출력 확인
- ✅ cargo test 전체 통과 (snapshot 회귀 0건)
- ⏳ Stage 4: 다른 샘플 시각 회귀 검사 + 한컴 2022 PDF 정합 비교 + 다중 shape 혼재 케이스 검토

## 6. 변경 통계

```
src/renderer/layout/table_layout.rs | ~30 라인 증가 (가드 + 좌표 분기)
```

기능 변경 없는 회귀 차단 패턴 (Picture 분기 패턴을 Shape 분기에 정합).

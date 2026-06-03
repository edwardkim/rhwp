---
타스크: #290 cross-run 탭 감지가 inline_tabs 무시
단계: 2 / 4 — 구현 + 단위 테스트
브랜치: local/task290
작성일: 2026-04-24
---

# Stage 2 완료 보고서

## 1. 목표

구현계획서 Stage 2 의 변경 3 항목을 모두 적용 + 회귀 검증:

1. `resolve_last_tab_pending` 헬퍼 신규 추가
2. est 측 / render 측 cross-run 블록 2 곳을 헬퍼 호출로 교체 + `inline_tab_cursor` 도입
3. 단위 테스트 5 건 추가

## 2. 변경 요약

### 2.1 diff 통계

```
src/renderer/layout/paragraph_layout.rs | 110 +++++++++++++++++++++++++-------
src/renderer/layout/tests.rs            |  83 ++++++++++++++++++++++++
2 files changed, 169 insertions(+), 24 deletions(-)
```

### 2.2 변경 파일

#### `src/renderer/layout/paragraph_layout.rs`

- **import 추가** (`:12`): `TabStop` 심볼 가져오기 (헬퍼 시그니처용)
- **헬퍼 신규** (`resolve_last_tab_pending`, `ensure_min_baseline` 바로 아래):
  - inline_tabs 가 `last_inline_idx` 를 커버하면 `ext[2] >> 8` 고바이트로 탭 종류 판정
    - `0 | 1` (LEFT 또는 unspecified) → `None` (pending 없음, 본 수정의 핵심)
    - `2 | 3` (RIGHT / CENTER) → TabDef `find_next_tab_stop` 경로로 폴스루
    - 기타 (4=DECIMAL 등 미지) → 보수적 `None`
  - inline 이 없거나 `\t` 인덱스를 초과하면 기존 `find_next_tab_stop` 경로 유지
- **est 측 루프** (`:840-930`):
  - `inline_tab_cursor_est: usize = 0` 변수 도입 (루프 진입 전)
  - 기존 cross-run 블록 (`:905-918`) 을 헬퍼 호출로 교체
  - 루프 말미(`:929`) 와 char_overlap `continue` 직전 각각에서 `cursor += run.text.chars().filter(|c| *c == '\t').count()` 증가
- **render 측 루프** (`:1198-1811`):
  - `inline_tab_cursor_render: usize = 0` 변수 도입 (루프 진입 전)
  - 기존 cross-run 블록 (`:1270-1283`) 을 헬퍼 호출로 교체
  - 루프 말미(`:1810` 직전) 에서 cursor 증가

#### `src/renderer/layout/tests.rs`

- **import 추가**: `use crate::renderer::{TextStyle, TabStop};`
- **신규 테스트 5 건** (모두 `task290_` 프리픽스):
  1. `task290_inline_left_returns_none` — LEFT inline (ext[2]=0x0100) → None ✓ 핵심 수정
  2. `task290_inline_right_uses_tabdef` — RIGHT inline (ext[2]=0x0203) → TabDef 위치로 Some((tp, 1))
  3. `task290_inline_center_uses_tabdef` — CENTER inline (ext[2]=0x0300) → TabDef 위치로 Some((tp, 2))
  4. `task290_no_inline_fallback_to_tabdef` — inline_tabs 비었을 때 TabDef RIGHT stop 사용 (기존 동작)
  5. `task290_no_inline_auto_tab_right_fallthrough` — inline 없음 + TabDef 소진 + auto_tab_right → 우측 끝 RIGHT (기존 동작)

## 3. 시각 확인 (Stage 3 에서 전면 검증 예정, 이번엔 스폿 체크)

### 3.1 exam_math.hwp 페이지 7 item 18 (원래 버그)

`output/svg/exam_math/exam_math_007.svg` 에서 "수" 글리프 위치:

| | "수" translate.x |
|---|---|
| Stage 1 전 (버그) | `290.9` |
| Stage 2 적용 후 | **`109.8`** ✓ |

col_area.x(71.8) + ~38 px = ~109.8 — 예상 정상 위치와 일치.

### 3.2 hwp-3.0-HWPML.hwp 페이지 3 저작권\t1 (RIGHT 회귀 체크)

| | "저" translate.x | "1" translate.x |
|---|---|---|
| Stage 2 적용 후 | `102.67` | `663.04` |

"1" 이 페이지 우측 끝 근처 (col width ~720 px) 에 정렬 → RIGHT 탭 동작 유지. 회귀 없음.

## 4. 회귀 테스트

### 4.1 `cargo test --lib`

```
test result: FAILED. 955 passed; 14 failed; 1 ignored
```

- **955 passed**: 기존 950 + 신규 task290_* 5 건
- **14 failed**: 모두 `serializer::cfb_writer` · `wasm_api::tests` — Task #283 오늘할일(`mydocs/orders/20260424.md:128`) 에 "950 pass / 14 fail (기존 CFB writer)" 로 기록된 **선존재 실패**. 본 변경 회귀 0.

신규 테스트만 실행:
```
$ cargo test --lib task290
running 5 tests
test renderer::layout::tests::task290_inline_left_returns_none ... ok
test renderer::layout::tests::task290_no_inline_fallback_to_tabdef ... ok
test renderer::layout::tests::task290_inline_center_uses_tabdef ... ok
test renderer::layout::tests::task290_inline_right_uses_tabdef ... ok
test renderer::layout::tests::task290_no_inline_auto_tab_right_fallthrough ... ok

test result: ok. 5 passed; 0 failed
```

### 4.2 `cargo test --test svg_snapshot`

```
test result: ok. 3 passed; 0 failed
```

### 4.3 `cargo clippy --lib -- -D warnings`

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.63s
```

Clean. `--tests --bins` 범위의 clippy 경고는 기존 integration test 코드 (`tests/svg_snapshot.rs`, `wasm_api/tests.rs`) 에 선존재하는 것들로 본 변경과 무관.

## 5. 다음 단계 예고

Stage 3 에서:
- `samples/exam_math.hwp` 20 페이지 전체 + `biz_plan.hwp` + `exam_eng.hwp` + `exam_kor.hwp` + `hwp-3.0-HWPML.hwp` 회귀 PNG 비교
- exam_math p.7 item 18 통합 테스트 신규 (`tests/tab_cross_run.rs` 또는 기존 test 파일에 추가)
- before/after/PDF 3 면 시각 비교 PNG 저장

## 6. 승인 요청

Stage 2 결론:
- 헬퍼 함수 도입 ✓
- cross-run 블록 2 곳 교체 + cursor 도입 ✓
- 단위 테스트 5 건 추가 + 모두 pass ✓
- 전체 회귀 0 (955/955 중 기존 14 선존재 실패 제외) ✓
- clippy `--lib` clean ✓
- 스폿 시각 검증: 원래 버그 수정 확인 + RIGHT 회귀 없음 ✓

Stage 3 (시각 회귀 검증) 진행 승인 요청.

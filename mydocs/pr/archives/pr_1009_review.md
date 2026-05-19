# PR #1009 검토 — Task #1007: HWP5 변환본 페이지 강제 나눔 한컴 정합

- 작성일: 2026-05-19
- 컨트리뷰터: [@jangster77](https://github.com/jangster77) (Taesup Jang)
- PR: https://github.com/edwardkim/rhwp/pull/1009
- base/head: `devel` ← `jangster77:local/task1007`
- 연결 이슈: closes #1007 (HWP5 변환본 페이지 강제 나눔 정밀화 — #1005 잔존 후속)
- 규모: +942 / -7, 16 files (소스 9, 문서 7), mergeable CONFLICTING
- 본질 커밋: 단일 `71054c51` (작성자 Taesup Jang)

## 1. 컨트리뷰터 사이클 / 시리즈 위치

@jangster77 24+ 사이클. #997→#999→#1005→**#1009**→#1011 연속. #1005 잔존 "페이지 강제 나눔 정밀화 — 한컴 page-fill 휴리스틱 미상"의 직접 후속. sample16-hwp5 67→**64** (한컴 정답지 정합).

## 2. 변경 내용

| 파일 | 변경 |
|------|------|
| `pagination.rs` | `PaginationOpts::is_hwp3_variant` 필드 |
| `pagination/engine.rs` | variant vpos reset 감지 + walk-back/forward + aux trigger (+84) |
| `typeset.rs` | `typeset_section_with_variant` 동일 로직 (+136) |
| `composer.rs` | CHARS_PER_LINE 45→50 (variant CharShape spacing −12% 보정) |
| `cfb_reader.rs` / `model/document.rs` / `parser/mod.rs` / `hwpx/mod.rs` / `cfb_writer/tests.rs` | variant 식별 인프라 (**#1005와 중복 — cherry-pick 충돌 예상**) |
| `document_core/queries/rendering.rs` | `is_hwp3_variant` 전달 |

**핵심 메커니즘**: 변환본 encoder의 `line_segs.vertical_pos` page-reset 신호를 paginator가 직접 인식. 조건:
- `is_hwp3_variant` (#1005 4중 AND 가드 식별)
- `prev_end_vpos > body_height_hu × 0.85` (페이지 거의 끝)
- `curr_first_vpos < 1500 HU` (page-reset 시 작은 값, empty line_segs 시 <4000)
- empty paragraph walk-back/forward + aux_trigger (empty bridge ≥2 + prev_end > body/2)

## 3. 검토 의견

### 강점

1. **variant 가드 한정** — 모든 신규 로직이 `is_hwp3_variant` (#1005 4중 AND 가드) 하에서만 동작. variant=false 문서 완전 무영향. `feedback_hancom_compat_specific_over_general` 정합 (구조 가드 기반).
2. **#1005 직접 후속, 명확 분리** — #1005 잔존(페이지 강제나눔)을 별도 PR로 분리. encoder vpos reset 신호 직접 인식이라는 명확한 root cause.
3. **HWP3 전용 분기 아님** — variant 런타임 식별 플래그 조건. CLAUDE.md 규칙 위반 아님.
4. **검증 + 잔존 투명** — cargo test 1303, clippy 0, WASM, 페이지 정합 63/64(98.4%). 잔존(59 paragraph PARA_LINE_SEG 누락 micro-drift 9~45px overflow) 별도 follow-up 명시.

### ⚠️ 쟁점

#### (A) cherry-pick 충돌 — #1005 variant 식별 인프라 중복

cfb_reader.rs/model/document.rs/parser/mod.rs/hwpx/mod.rs/cfb_writer/tests.rs는 #1005(`8b7fba3b`)에서 이미 devel 머지. PR #1009가 #1005 이전 base 분기로 동일 인프라 재포함 → cherry-pick 충돌 예상. **해소: 충돌 파일은 devel(#1005 머지본) 우선, #1009 고유 증분(pagination/engine.rs vpos reset + typeset.rs + composer.rs 50)만 적용**.

#### (B) THRESHOLD 매직넘버 (0.85 / 1500 HU / 4000 HU / bridge≥2)

페이지 split 감지 다중 임계값 하드코딩. variant 한정이라 일반 문서 무영향이나, variant 문서 간 sample16-hwp5 외 다른 변환본 일반화 위험. PR 본문이 sample16-hwp5 단일 타깃 + 잔존 1건(페이지33 empty "(빈)" h=13.3px 시각 무영향) 명시.

#### (C) CHARS_PER_LINE 45→50 — #999/#1005 연쇄 휴리스틱 조정

#997(35) → #999(45) → #1009(50). variant CharShape spacing −12% 보정 근거이나 측정 무관 고정값 누적 조정. 임시 휴리스틱(향후 reflow_line_segs 대체)의 연장.

### 확인 필요 (검증 단계)

1. cherry-pick `71054c51` — 충돌 파일 devel(#1005) 우선 해소, #1009 고유 증분만
2. cargo test --release --lib + clippy -D + fmt 0
3. sweep — variant=false 일반 HWP5(exam/aift/biz_plan) 무회귀 + HWP3 sample16 무회귀 + 타깃 sample16-hwp5 67→64
4. WASM 빌드 + 작업지시자 시각 판정 — sample16-hwp5 64페이지 한컴 정합

## 4. 처리 옵션

- **옵션 A (수용)**: sweep variant=false 무회귀 + 작업지시자 시각 판정(sample16-hwp5 64 한컴 정합) 통과 시. 작업지시자 단축 지시("체리픽 + WASM 빌드 후 시각 검증")로 진행.
- **옵션 B (수정 요청)**: variant 문서 회귀 또는 충돌 해소 불가 시.
- **옵션 C (close)**: 본질 결함 시. 해당 낮음.

## 5. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @jangster77 #997~#1011 연속, #1009 순차
- `feedback_hancom_compat_specific_over_general` — vpos reset 로직 variant 가드 한정 (구조 가드). THRESHOLD/CHARS_PER_LINE 매직넘버는 임시 명시로 완화
- `feedback_fix_scope_check_two_paths` — pagination/engine.rs + typeset.rs 동일 로직 양쪽 적용 (두 경로 정합)
- `feedback_self_verification_not_hancom` — sample16-hwp5 64 = 한컴 정답지 정합, 작업지시자 시각 판정 게이트
- `feedback_visual_judgment_authority` — sample16-hwp5 64페이지 + 일반 HWP5 무회귀 작업지시자 시각 판정 최종 게이트
- `project_output_folder_structure` — sweep 산출물 output/poc/pr1009 배치

## 6. 권고

**옵션 A** — 작업지시자 단축 지시 수령(체리픽 + WASM + 시각 검증). 충돌 #1005 우선 해소, 검증(test/clippy/fmt + sweep variant=false 무회귀) + 작업지시자 시각 판정 통과 시 cherry-pick no-ff merge. 잔존(59 paragraph PARA_LINE_SEG micro-drift)은 PR 본문대로 별도 follow-up issue 분리.

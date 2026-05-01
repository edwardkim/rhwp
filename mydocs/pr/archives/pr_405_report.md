# PR #405 처리 보고서 — `Paragraph::control_text_positions` 추가 (옵션 A, #390)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#405](https://github.com/edwardkim/rhwp/pull/405) |
| 작성자 | [@DanMeon](https://github.com/DanMeon) — 이슈 #390 자체 보고자 |
| 이슈 | [#390](https://github.com/edwardkim/rhwp/issues/390) |
| 처리 결정 | **옵션 A (cherry-pick 머지)** |
| 처리 일자 | 2026-04-29 |
| 정황 | 작업지시자 사전 PR 권유 사안 — 의향 일치 |

## 처리 절차

### Stage 1: cherry-pick

`local/pr405` 브랜치 (`local/devel` 분기) 에서 PR head 까지 3 commit cherry-pick — 작성자 attribution 보존:

| commit | 작성자 | 내용 |
|--------|--------|------|
| `2c6727f` | @DanMeon | Stage 1: Paragraph::control_text_positions 메서드 + helpers wrapper 전환 |
| `eeb1971` | @DanMeon | Stage 2: 단위 테스트 6건 추가 |
| `13ffd0a` | @DanMeon | 최종 보고서 closes #390 |

cherry-pick 결과: 충돌 없이 자동 머지.

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1037 passed** (1031 → +6 신규 paragraph 테스트) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 22s, 4,111,447 bytes |

## 변경 요약

### 본질 — 위치 이동 리팩토링

| 변경 | 위치 |
|------|------|
| 알고리즘 본체 (~60 줄) | `document_core::helpers::find_control_text_positions` → **`model::Paragraph::control_text_positions`** (인스턴스 메서드, `pub`) |
| `helpers::find_control_text_positions` | thin wrapper (`para.control_text_positions()`) 로 교체 + `pub(crate)` 유지 |
| 26 caller (cursor / nav / 렌더러 / 책갈피 / 명령 / WASM) | **변경 없음** — wrapper 가 그대로 동작 |

### 옵션 A 선택 (작성자 분석)

> 외부 API surface 를 좁게 유지 — `Paragraph` 메서드 한 개만 노출. helpers 모듈은 내부 구현으로 자유롭게 진화. 의미 응집 — paragraph 자체에 컨트롤 위치 질의 메서드. 옵션 B 는 helpers 의 다른 비공개 함수까지 함께 노출되어 향후 SemVer 부담.

→ long-term API 안정성 측면에서 우수.

### 의존성 방향 정합

`model` ← `parser` ← `document_core` ← `renderer` ← `wasm_api` 단방향 의존 보존:
- 알고리즘 본체는 model 레이어 (`Paragraph` 자체 데이터만 사용)
- helpers (document_core 레이어) 가 model 메서드 호출 — 정방향 의존

### 단위 테스트 6 개

| 테스트 | 검증 |
|--------|------|
| `test_control_text_positions_empty` | controls 없음 → 빈 vec |
| `test_control_text_positions_no_offsets_inline_sequential` | char_offsets 없음 + Table 2개 → [0, 1] |
| `test_control_text_positions_gap_between_chars` | "AB" + char_offsets [0, 9] + Table 1개 → [1] |
| `test_control_text_positions_gap_before` | "A" + char_offsets [8] + Table 1개 → [0] |
| `test_control_text_positions_surrogate_pair_char_width` | "🎉A" surrogate pair (UTF-16 width=2) 처리 |
| `test_control_text_positions_no_offsets_non_inline_skipped` | Bookmark (비인라인) 은 pos 증가 안 함 |

## 변경 파일

| 파일 | 변경 |
|------|------|
| `src/model/paragraph.rs` | `pub fn control_text_positions(&self) -> Vec<usize>` 추가 (+86) |
| `src/document_core/helpers.rs` | 본체 (~60 줄) → thin wrapper 1 줄 (-57 / +4) |
| `src/model/paragraph/tests.rs` | 단위 테스트 6 개 추가 (+89) |
| `mydocs/plans/task_m100_390{,_impl}.md`, `mydocs/report/task_m100_390_report.md` | 작성자 작업 문서 |

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test --lib 1037 passed + clippy 0 + WASM 빌드 통과 |
| PR 댓글 톤 — 과도한 표현 자제 | ✅ |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr405` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 시각 판정 정황

본 PR 은 **API 노출 / 알고리즘 변경 없음** → 시각 판정 불필요. 단위 테스트 + cargo test 1037 passed 로 검증 게이트 통과.

PR #401 과 다름:
- PR #401: 페이지 분할 알고리즘 변경 → 다른 표 샘플 회귀 가능성 → 시각 판정 필요
- PR #405: 알고리즘 동일, 외부 가시성만 변경 → 회귀 위험 없음, 시각 판정 의미 없음

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr405` → `local/devel` → `devel` 머지 + push
3. PR #405 close + 작성자 댓글
4. 이슈 #390 자동 close (PR 본문에 `closes #390` 명시)

## 참고

- 검토 문서: `mydocs/pr/pr_405_review.md`
- PR: [#405](https://github.com/edwardkim/rhwp/pull/405)
- 이슈: [#390](https://github.com/edwardkim/rhwp/issues/390) (assignee @DanMeon, 본 PR 작성자)
- 외부 binding 사용처: `rhwp-python` (작성자 작업 중)

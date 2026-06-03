# Task #484: 최종 결과보고서

## 완료 사항

이슈 #484 — `utf16_pos_to_char_idx` 외부 crate 노출. 옵션 A (`Paragraph::utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize` 인스턴스 메서드 캡슐화) 로 구현 완료.

## 변경 내역

### 코드

- `src/model/paragraph.rs` — `pub fn utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize` 추가 (impl Paragraph 블록 내, `control_text_positions` 다음). 본체 1 줄 (`self.char_offsets.iter().position(|&off| off >= utf16_pos).unwrap_or(self.char_offsets.len())`) 자체 보유.
- `src/document_core/helpers.rs` — 변경 없음. free function 시그니처 (`(char_offsets: &[u32], utf16_pos: u32)`) 가 `Paragraph` 인스턴스를 받지 않아 PR #405 패턴 (본체 이동 + thin wrapper) 적용 불가. caller (cursor_nav, clipboard 5+ 라인) 변경 회피로 본 PR scope 외.

### 테스트

- `src/model/paragraph/tests.rs` — 신규 단위 테스트 6 건:
  - `test_utf16_pos_to_char_idx_empty_offsets` — `char_offsets.is_empty()` → `unwrap_or(0)` 분기
  - `test_utf16_pos_to_char_idx_zero_returns_first` — `utf16_pos = 0` → 첫 entry 인덱스
  - `test_utf16_pos_to_char_idx_exact_match` — offsets 정확값 매칭
  - `test_utf16_pos_to_char_idx_between_offsets` — offsets 사이 값 (SMP `"A🎉"`, offsets `[0, 1, 3]`)
  - `test_utf16_pos_to_char_idx_beyond_end_returns_len` — `utf16_pos > 모든 entry` → `char_offsets.len()` fallback
  - `test_utf16_pos_to_char_idx_surrogate_pair_midpoint` — surrogate pair low half 위치를 다음 codepoint 시작 위치로 정규화

### 문서 (`mydocs/`)

- `plans/task_m100_484.md` — 수행 계획서 (배경, 옵션 결정, 의존성 고려, 범위/비범위)
- `plans/task_m100_484_impl.md` — 구현 계획서 (3 단계)
- `report/task_m100_484_report.md` — 본 문서

## 검증 결과

| 검증 항목 | 결과 |
|---|---|
| `cargo build --tests` | 성공 |
| `cargo clippy --all-targets` (변경 파일 한정) | warning 0 건 |
| `cargo test --lib` | 1081 passed / 0 failed / 1 ignored (baseline `upstream/devel @ 109bb04` = 1075, +6 신규 테스트로 일치) |
| `Paragraph::utf16_pos_to_char_idx` 신규 단위 테스트 | 6 / 6 통과 |
| helpers 의 기존 caller (cursor_nav, clipboard) 회귀 | 없음 (helpers free function 미변경) |

## 동작 보장

- helpers 의 `utf16_pos_to_char_idx` free function 알고리즘과 동치 (`iter().position(...).unwrap_or(...)` 1줄)
- 외부 가시성: 새 `pub fn Paragraph::utf16_pos_to_char_idx` 만 추가. helpers / free function 가시성 변경 없음 (`pub(crate)` 유지)
- semver: MINOR (신규 public API 추가, 기존 contract 보존)

## 검증 범위

- 변경 파일 한정 rustfmt / clippy 통과
- 저장소 전체 fmt / clippy 는 본 PR 범위 외 — base branch `upstream/devel @ 109bb04` 의 기존 lint warning 43 건 (`emf/tests.rs`, `serializer/hwpx/field.rs`, `wasm_api/tests.rs`) 별도 cleanup 필요

## 다음 단계

- 메인테이너 리뷰 후 PR 머지 (대상 브랜치: `devel`)
- PR 본문에 `closes #484` 명시
- 머지 후 외부 binding (`rhwp-python` 등) 에서 `Paragraph::utf16_pos_to_char_idx()` 직접 호출 가능 → 외부의 자체 알고리즘 복사 (`utf16_to_cp` 등) 제거 가능

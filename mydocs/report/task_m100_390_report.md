# Task #390: 최종 결과보고서

## 완료 사항

이슈 #390 — `find_control_text_positions` 외부 crate 노출. 옵션 A (`Paragraph::control_text_positions()` 인스턴스 메서드 캡슐화) 로 구현 완료.

## 변경 내역

### 코드

- `src/model/paragraph.rs` — `pub fn control_text_positions(&self) -> Vec<usize>` 추가 (impl Paragraph 블록 내, `char_shape_id_at` 다음). 알고리즘 본체 (~60 줄, char_offsets 갭 분석) 를 helpers 에서 이동.
- `src/document_core/helpers.rs` — `find_control_text_positions` 본체를 `para.control_text_positions()` 한 줄 thin wrapper 로 교체. `pub(crate)` 가시성 유지.

### 테스트

- `src/model/paragraph/tests.rs` — 신규 단위 테스트 4 건:
  - `test_control_text_positions_empty` — `controls.is_empty()` 분기
  - `test_control_text_positions_no_offsets_inline_sequential` — `char_offsets.is_empty()` + 인라인 컨트롤 순차 배치
  - `test_control_text_positions_gap_between_chars` — 일반 분기, 'AB' 사이 8 unit 갭
  - `test_control_text_positions_gap_before` — 첫 문자 이전 갭, 'A' 앞 8 unit

### 문서 (`mydocs/`)

- `plans/task_m100_390.md` — 수행 계획서 (배경, 옵션 결정, 의존성 고려, 범위/비범위)
- `plans/task_m100_390_impl.md` — 구현 계획서 (4 단계)
- `report/task_m100_390_report.md` — 본 문서

## 검증 결과

| 검증 항목 | 결과 |
|---|---|
| `cargo build` | 성공 |
| `cargo clippy --all-targets` (변경 파일 한정) | warning 0 건 |
| `cargo test --lib` | 1022 passed / 0 failed / 1 ignored (baseline `upstream/devel @ 4828937` = 1016, +6 신규 테스트로 일치) |
| `cargo test --tests` | 통합 테스트 회귀 없음 |
| `Paragraph::control_text_positions` 신규 단위 테스트 | 6 / 6 통과 (empty / no-offsets-inline / no-offsets-non-inline-skipped / gap-between / gap-before / surrogate-pair-width) |
| 기존 26 caller (cursor / nav / 렌더러 / 책갈피 / 명령 / WASM) 회귀 | 없음 (helpers wrapper 가 위임) |

## 동작 보장

- 알고리즘 본체 byte-identical 이동 (조건문 분기, gap 계산식, char_width 판정 모두 원본 그대로 보존)
- `find_control_text_positions(para)` 호출 결과 = `para.control_text_positions()` 호출 결과 (thin wrapper)
- 외부 가시성: 새 `pub fn Paragraph::control_text_positions` 만 추가, helpers / `find_control_text_positions` 가시성 변경 없음
- semver: MINOR (신규 public API 추가, 기존 contract 보존)

## 검증 범위

- 변경 파일 한정 rustfmt / clippy 통과 (저장소 전체 fmt / clippy 는 본 PR 범위 외)

## 다음 단계

- 메인테이너 리뷰 후 PR 생성 (대상 브랜치: `devel`)

- PR 본문에 `closes #390` 명시
- 머지 후 외부 binding (rhwp-python 등) 에서 `Paragraph::control_text_positions()` 직접 호출 가능

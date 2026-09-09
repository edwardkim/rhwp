# Task M100 #2185 Stage 1 완료보고서 — 한글 줄 나눔 의미 복구

## 목표

HWP/HWPX 문단 모양 `attr1 bit 7`의 내부 계약을 `0=어절`, `1=글자`로 고정하고,
공통 라인브레이커가 이를 반대로 소비하던 두 조건을 최소 범위로 바로잡는다. 저장 포맷의
파서·직렬화 매핑과 raw bit는 변경하지 않는다.

## 구현 결과

- `src/renderer/composer/line_breaking.rs`
  - 한글 어절 토큰 분기를 `korean_break_unit == 0`으로 복구했다.
  - 단일 한글 문자를 줄바꿈 가능 지점으로 취급하는 조건을
    `korean_break_unit == 1`로 복구했다.
  - HWPX 속성 이름과 renderer 의미를 혼동하던 주석을 내부 계약 기준으로 정정했다.
- `src/renderer/style_resolver.rs`
  - `korean_break_unit` 문서 주석을 `0=어절, 1=글자`로 정정했다.
  - bit 추출이나 기본값은 변경하지 않았다.
- `src/renderer/composer/tests.rs`
  - 어절 토큰화 테스트는 bit0, 글자 토큰화 테스트는 bit1을 사용하도록 바로잡았다.
  - 동일한 `"가나 다라"` 문단과 60px 폭에서 실제 줄 시작점을 비교하는 회귀 테스트를
    추가했다.

## 회귀 핀

16px 테스트 스타일에서 `"가나 다라"`는 `가나` 32px, 공백 8px, `다` 16px까지
60px 안에 들어가고 다음 글자까지는 넘친다. 따라서 두 모드의 결과가 결정적으로 갈린다.

| `korean_break_unit` | 의미 | `LINE_SEG.text_start` |
|---------------------|------|-----------------------|
| `0` | 어절 | `[0, 3]` |
| `1` | 글자 | `[0, 4]` |

기존 반전 구현에서는 위 두 결과가 서로 뒤바뀌므로, 새 테스트는 소비 의미가 다시 반전되는
회귀를 직접 검출한다.

## 검증

- `cargo fmt --check`
  - 통과
- `git diff --check`
  - 통과
- `cargo test --lib renderer::composer::tests::test_tokenize_korean -- --nocapture`
  - 통과, 2 passed
- `cargo test --lib renderer::composer::tests::test_reflow_korean -- --nocapture`
  - 통과, 2 passed
- `cargo test --lib parser::hwpx::header::tests::test_parse_hwpx_para_shape_break_non_latin_word_bit -- --exact`
  - 통과, 1 passed
- `cargo test --lib serializer::hwpx::header::tests::write_para_pr_emits_align_and_break_from_preserved_bits -- --exact`
  - 통과, 1 passed

테스트 중 macOS SDK 탐색의 기존 `xcrun` 환경 경고가 출력됐으나 컴파일과 테스트 결과에는
영향이 없었다.

## 범위 확인

- HWP5/HWPX 파서와 직렬화기의 bit 7 매핑은 수정하지 않았다.
- 모델·Studio UI의 `0=어절`, `1=글자` 계약도 수정하지 않았다.
- #1949 공개 샘플의 실제 편집·저장·재로드 회귀 핀은 승인 후 Stage 2에서 추가한다.
- 거대 셀 입력 지연과 전체 pagination 성능은 이번 정확성 수정과 분리한 조사 대상으로
  유지한다.

## 상태

Stage 1 구현과 표적 검증 완료. 작업지시자 승인 전에는 Stage 2를 진행하지 않는다.

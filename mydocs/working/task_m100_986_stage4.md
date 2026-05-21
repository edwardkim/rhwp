# Stage 4 완료 보고서 — Task #986

## 범위

debug build 에서 제보 샘플을 처리할 때 `compose_lines` 의 line range 계산이
역전되어 `text_end - text_start` underflow panic 이 발생하는 문제를 방어했다.

## 재현

수정 전 debug build:

```bash
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo run --bin rhwp -- dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
```

결과:

```text
thread 'main' panicked at src/renderer/composer.rs:540:19:
attempt to subtract with overflow
```

## 변경 파일

- `src/renderer/composer.rs`
  - `utf16_range_to_text_range` 결과에서 `text_end < text_start` 인 경우
    해당 LineSeg 를 빈 텍스트 range 로 clamp
  - 원본 LineSeg 순서나 `text_start` 값을 재정렬하지 않고, 비정상 range 만
    안전하게 비워서 debug/release 동작 차이를 제거
- `src/renderer/composer/tests.rs`
  - `LineSeg.text_start` 가 감소하는 synthetic paragraph 단위 테스트 추가

## 검증

```bash
cargo fmt --all --check
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --lib composer::tests::test_compose_decreasing_lineseg_text_start_uses_empty_range
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo run --bin rhwp -- dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --lib renderer::composer::tests
```

결과:

- `cargo fmt --all --check`: 통과
- 신규 단위 테스트: 통과
- debug `dump-pages`: panic 없이 완료
- `renderer::composer::tests`: 36개 테스트 통과

## 제보 샘플 결과

debug build 에서도 Stage 3 과 같은 pagination 결과가 유지된다.

- 전체 2페이지
- page 1: `ci=2..8` 전체 표가 `Table`
- `PartialTable` 없음
- page 2: 빈 문단 `pi=2` 1개

## 남은 문제

최종 page count 는 아직 1 이 아니라 2이다. 다음 Stage 5 에서는 회귀 테스트를
추가하면서 trailing 빈 문단 처리 범위를 함께 확정해야 한다.

## 다음 단계

Stage 5 에서 이슈 #986 회귀 테스트를 추가한다. fixture 포함 여부와 최종 page count
단언을 계획서 기준으로 다시 확정한 뒤 진행한다.

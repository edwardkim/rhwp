# Task #1145 Stage 1 완료 보고서

## 1. 목표

`RowBreak` 표가 페이지 하단에서 작은 rowspan 제목/라벨 블록 중간으로 분할되어
첫 행 또는 제목 셀이 다음 페이지에 중복 출력되는 문제를 수정한다.

## 2. 재현 샘플

- `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx`

수정 전 관찰:

```text
page 1: PartialTable pi=22 rows=0..1
page 2: PartialTable pi=22 rows=1..3
```

문제 표의 제목 셀은 `row=0`, `row_span=2`이므로 `end_row=1` 분할은 rowspan 블록 내부 절단이다.

## 3. 수정 내용

수정 파일:

- `src/renderer/typeset.rs`

변경 요지:

```text
RowBreak 표라도 작은 rowspan 블록(2~3행)이면서 내부 hard break가 없으면
일반 rowspan 보호 블록처럼 atomic 처리한다.
```

구체적으로 다음 두 지점에 동일 정책을 적용했다.

1. 첫 fragment를 현재 페이지에 남길지 판단하는 `first_block_protected`
2. 실제 row walk 중 `protected || rowbreak_rowspan_block` 분기

`RowBreak` 표의 기존 목적이었던 일반 행 경계 분할은 유지한다.
내부 hard break가 있는 RowBreak rowspan 블록은 기존 `rowbreak_rowspan_block` 경로로 남겨
large block cut 동작을 보존한다.

## 4. 추가 테스트

신규 테스트:

- `tests/issue_1145.rs`

검증 내용:

```text
page 1:
  PartialTable pi=22 없음

page 2:
  Table pi=22 ci=0 3x3 로 전체 표 시작
  PartialTable pi=22 없음
```

## 5. 확인 결과

수정 후 dump:

```text
page 1:
  pi=22 미출력

page 2:
  Table pi=22 ci=0 3x3
```

SVG 산출물:

- `output/poc/issue1145_rowbreak_rowspan_fix/2025년 기부·답례품 실적 지자체 보고서_양식_001.svg`
- `output/poc/issue1145_rowbreak_rowspan_fix/2025년 기부·답례품 실적 지자체 보고서_양식_002.svg`

`s0:pi=22` debug label은 2페이지 SVG에서만 확인된다.

## 6. 검증

```text
cargo fmt --all -- --check
cargo test --test issue_1145 -- --nocapture
cargo test --test issue_1086 -- --nocapture
cargo test --test issue_1105 -- --nocapture
cargo test --lib
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
issue_1145: 1 passed
issue_1086: 4 passed
issue_1105: 14 passed
cargo test --lib: 1405 passed, 0 failed, 6 ignored
WASM build: success
```

`cargo test --lib`에는 기존 경고 6건이 출력되었으나 실패는 없다.
WASM 산출물은 `pkg/`에 생성되었다.

## 7. 시각 판정

작업지시자 시각 판정:

```text
통과
```

## 8. 다음 단계

최종 보고서 승인 후 커밋, `local/devel` 병합, 원격 반영, 이슈 close 절차로 진행한다.

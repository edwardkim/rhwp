# Task #1145 최종 보고서

## 1. 이슈

- GitHub Issue: [#1145](https://github.com/edwardkim/rhwp/issues/1145)
- 제목: HWP/HWPX RowBreak 표 rowspan 분할 시 첫 행 중복 출력
- 브랜치: `local/task1145`
- 마일스톤: M100 / v1.0.0

## 2. 문제

HWP/HWPX 공통 조판 경로에서 `RowBreak` 표가 페이지 하단에 걸릴 때,
rowspan 제목 셀의 중간 행 경계에서 `PartialTable` 분할이 발생했다.

문제 샘플:

- `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx`

수정 전:

```text
page 1: PartialTable pi=22 rows=0..1
page 2: PartialTable pi=22 rows=1..3
```

`pi=22` 표의 제목 셀은 `row=0`, `row_span=2`이므로 `end_row=1`은 rowspan 블록 내부 절단이다.
이 때문에 1페이지 말미와 2페이지 시작에 제목 셀이 중복 출력되었다.

## 3. 수정

수정 파일:

- `src/renderer/typeset.rs`

수정 내용:

- `RowBreak` 표라도 작은 rowspan 블록이면서 내부 hard break가 없는 경우에는 atomic 블록으로 처리한다.
- 첫 fragment 배치 판단과 row walk 분할 판단에 같은 보호 조건을 적용했다.
- 내부 hard break가 있는 `RowBreak` rowspan 블록은 기존 block cut 경로를 유지하여 #1086 계열 회귀를 피했다.

## 4. 회귀 테스트

추가 파일:

- `tests/issue_1145.rs`

검증 내용:

```text
page 1:
  PartialTable pi=22 없음

page 2:
  Table pi=22 ci=0 3x3 로 전체 표 시작
  PartialTable pi=22 없음
```

## 5. 검증

실행한 검증:

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

`cargo test --lib`에서 기존 경고 6건이 출력되었으나 실패는 없다.

SVG 확인 산출물:

- `output/poc/issue1145_rowbreak_rowspan_fix/2025년 기부·답례품 실적 지자체 보고서_양식_001.svg`
- `output/poc/issue1145_rowbreak_rowspan_fix/2025년 기부·답례품 실적 지자체 보고서_양식_002.svg`

## 6. 시각 판정

작업지시자 시각 판정:

```text
통과
```

## 7. 결론

`RowBreak` 표의 일반 행 경계 분할 정책은 유지하면서,
작은 rowspan 제목/라벨 블록 내부 절단만 차단했다.

문제 샘플에서 1페이지 말미에 표 제목 셀이 반쪽 출력되는 현상이 제거되었고,
2페이지에서 `pi=22` 표가 전체 표로 시작하도록 정정되었다.

## 8. 다음 절차

보고서 승인 후 다음 절차를 진행한다.

```text
1. 커밋
2. local/devel 병합
3. devel 검증
4. 원격 devel push
5. 이슈 #1145 close
```

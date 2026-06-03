# Task M100-1249 완료 보고서

- **이슈**: [#1249](https://github.com/edwardkim/rhwp/issues/1249) — HWPX 글상자 세로쓰기 렌더링 구현
- **처리일**: 2026-06-02
- **브랜치**: `local/task1249`
- **상태**: 구현 불필요, 기존 #1028 처리로 해결 확인

## 1. 결론

#1249에서 요청한 HWPX 글상자 세로쓰기 렌더링은 현재 `devel` 코드 기준 이미 구현되어 있다.

동일 증상은 기존 #1028에서 처리되었으며, 현재 코드에도 다음 구현이 유지되어 있다.

- `src/parser/hwpx/section.rs`
  - `<hp:subList textDirection="VERTICAL" | "VERTICALALL">` 파싱
  - `text_box.list_attr` bit 0~2에 세로쓰기 플래그 반영
- `src/renderer/layout/shape_layout.rs`
  - `text_box.list_attr & 0x07` 기반 글상자 세로쓰기 분기
  - `layout_vertical_textbox_text_with_paras` 경로 사용
- `tests/issue_1028_hwpx_textbox_vertical.rs`
  - HWPX 세로쓰기 fixture와 가로 글상자 무회귀 fixture 검증

따라서 이번 이슈는 신규 소스 수정 없이 **기존 해결 항목 재확인 후 종료**하는 것이 적절하다.

## 2. 검증 결과

### 2.1 회귀 테스트

```text
cargo test --test issue_1028_hwpx_textbox_vertical -- --nocapture
```

결과:

```text
running 2 tests
test issue_1028_hwpx_textbox_vertical_direction_parsed ... ok
test issue_1028_hwpx_horizontal_textbox_unchanged ... ok

test result: ok. 2 passed; 0 failed
```

### 2.2 SVG 산출

HWPX:

```text
target/debug/rhwp export-svg samples/hwpx/tbox-v-flow-01.hwpx -o output/poc/task1249/current/hwpx
```

산출물:

```text
output/poc/task1249/current/hwpx/tbox-v-flow-01.svg
```

HWP5 변환본:

```text
target/debug/rhwp export-svg samples/hwpx/hancom-hwp/tbox-v-flow-01.hwp -o output/poc/task1249/current/hwp5
```

산출물:

```text
output/poc/task1249/current/hwp5/tbox-v-flow-01.svg
```

### 2.3 rotate transform 정합

```text
rg -c "rotate\\(" output/poc/task1249/current/hwpx/tbox-v-flow-01.svg output/poc/task1249/current/hwp5/tbox-v-flow-01.svg
```

결과:

```text
output/poc/task1249/current/hwp5/tbox-v-flow-01.svg:4
output/poc/task1249/current/hwpx/tbox-v-flow-01.svg:4
```

HWPX와 HWP5 변환본 모두 세로쓰기 회전 transform이 4건으로 동일하다.

## 3. 기존 #1028과의 관계

기존 #1028의 제목은 다음과 같다.

```text
HWPX 글상자 세로 쓰기 미구현 (HWP5는 지원, HWPX 미적용)
```

해당 이슈는 2026-05-20에 `completed` 상태로 종료되었고, `task_m100_1028_report.md`에 구현 및 시각 판정 결과가 기록되어 있다.

#1249의 문제 정의는 #1028과 동일한 fixture 및 동일한 구현 축을 가리킨다.

## 4. 소스 수정 여부

이번 작업에서는 소스 수정이 필요하지 않았다.

변경 사항은 다음 문서 기록에 한정된다.

- `mydocs/orders/20260602.md`
- `mydocs/plans/task_m100_1249.md`
- `mydocs/report/task_m100_1249_report.md`

## 5. 권장 후속 절차

작업지시자 승인 후 #1249 이슈에 다음 취지로 코멘트하고 close 처리한다.

```text
현재 devel 기준 확인 결과, 이 이슈는 기존 #1028 처리로 이미 해결된 상태입니다.

- `cargo test --test issue_1028_hwpx_textbox_vertical -- --nocapture`: 2 passed
- `samples/hwpx/tbox-v-flow-01.hwpx` SVG: rotate transform 4건
- `samples/hwpx/hancom-hwp/tbox-v-flow-01.hwp` SVG: rotate transform 4건

따라서 신규 구현 없이 #1028 완료 항목으로 연결하여 종료합니다.
```

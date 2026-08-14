# Task M100 #3820 Stage 212 - HWP5-origin 원형 제목 RowBreak 소유 보존

## 회귀

`tests/issue_1939.rs`에서 HWP5 바이너리 fixture `76076_regulatory_analysis.hwpx`의 HWPX
왕복 strict render-diff가 614.53px로 실패했다. 원본과 재파스 모두 82쪽이지만 55쪽과
70쪽에서 후속 제목 문단이 이전 쪽에 잘못 남았다.

## 증거와 원인

`RHWP_FLOW_DBG`와 `RHWP_DIAG_FLOW`로 비교하면 표 host 직전까지의 누적 높이는 두 경로가
동일하다. 원본 HWP5는 `pi=550`을 조판하기 전에 다음 쪽으로 넘기지만 HWPX 재파스는 넘기지
않는다.

원인은 Stage 176의 구조 보정이었다. 이 보정은 원형 제목, 빈 carrier 한 줄, non-TAC 1x1
TopAndBottom RowBreak 표의 세 문단 구조를 인식해 제목을 표와 같은 물리 쪽에 둔다. 하지만
조건이 `native_hwp5_layout()`에만 묶여 있어, 동일한 저장 pagination 계약을 가진 HWP5-origin
marker HWPX에서는 적용되지 않았다.

## 보정

구조 보정의 profile 조건을 `hwp5_stored_pagination_layout()`으로 바꾼다. 순수 HWPX에는 적용하지
않고, native HWP5와 HWP5-origin marker HWPX만 원형 제목과 RowBreak 표의 쪽 소유를 공유한다.

## 검증 대상

- `cargo test --profile release-test --test issue_1939 -- --nocapture`
- `cargo test --profile release-test --test issue_1891 --test issue_1695 --test issue_1733 --test issue_3820_rowbreak_rowspan_band -- --nocapture`
- 전체 `cargo test --profile release-test --lib` 및 `cargo test --profile release-test --tests`

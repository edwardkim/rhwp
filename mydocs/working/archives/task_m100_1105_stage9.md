# Task #1105 Stage 9 완료 보고서 — 정상 HWP5 lineseg reflow no-op 보장

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 브랜치: `local/task1105`
- 기준: 한컴오피스 정답지 / `hwp3-sample16-hwp5-2024.hwp`

## 1. 추가 피드백

`hwp3-sample16-hwp5-2024.hwp`만 브라우저에서 23쪽 내용이 한컴오피스 정답지와 다르게 보였다.
화면에서는 상태 메시지의 문서 총 페이지는 64쪽인데, 현재 쪽 표시는 63쪽 기준으로 움직이며
23쪽이 한컴 기준보다 한 페이지 뒤의 내용처럼 보였다.

## 2. 원인 분석

현재 원본 로드 경로는 정상이다.

- native CLI: 64쪽
- `rhwp-studio` `WasmBridge.loadDocument`: 64쪽
- `CanvasView.loadDocument`: 64/64쪽
- 23쪽 텍스트 레이아웃: `pi=450`부터 시작하고 `pi=460`은 `lines=0..3`만 포함

문제는 `reflowLinesegs()`를 명시 호출했을 때 재현됐다.

`hwp3-sample16-hwp5-2024.hwp`는 검증 경고가 0건인데도 기존 `reflow_linesegs_on_demand()`가
긴 텍스트 + lineseg 1개 패턴을 broad reflow 대상으로 다시 계산했다. 그 결과 110개 문단이
불필요하게 재계산되고 총 페이지가 64쪽에서 63쪽으로 줄며, 23쪽 내용이 한컴 정답지와 달라졌다.

## 3. 구현

`src/document_core/commands/document.rs`의 `reflow_linesegs_on_demand()` 진입부에
검증 리포트가 비어 있으면 즉시 `0`을 반환하는 guard를 추가했다.

의도:

- 검증 경고가 없는 정상 HWP3/HWP5 문서는 사용자 명시 reflow도 no-op
- 비표준 HWPX처럼 `validation_report`가 있는 문서만 기존 보정 경로 사용
- 한컴 정답지와 맞는 기존 `LINE_SEG`를 임의 재생성하지 않음

## 4. 회귀 테스트

`tests/issue_1105.rs`에 다음 검사를 추가했다.

- `hwp3-sample16-hwp5-2024.hwp`의 validation warning count가 0
- `reflow_linesegs()`가 0 반환
- reflow 호출 후에도 총 페이지가 64쪽
- 23쪽은 계속 `pi=450`에서 시작하고 `pi=460 lines=0..3`만 포함
- `pi=461`은 23쪽에 들어오지 않음


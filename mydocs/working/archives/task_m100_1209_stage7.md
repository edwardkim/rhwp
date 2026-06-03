# Task M100 #1209 Stage 7

## 목적

Stage6 커밋 이후 남은 편집 UI 동작 문제를 별도 단계로 분리한다.
문단 모양, 글자 모양 등 모달 대화상자의 타이틀 바 드래그 이동을 개체 속성 대화상자와 동일하게 동작하도록 보정한다.

## 시작 기준

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- 작업 브랜치: `local/task_m100_1209`
- 선행 커밋: `e0151e37` (`task 1209: Stage6 미주 VPOS 되감김 보정`, `upstream/devel` rebase 후)
- 요청 일시: 2026-06-01

## 작업지시자 관찰

1. 문단 모양, 글자 모양 팝업이 아직 이동되지 않는다.
2. 개체 속성 대화상자는 타이틀 바 드래그 이동이 가능하므로 같은 방식으로 공통화가 필요하다.
3. 다른 모달성 대화상자도 개별 구현으로 흩어져 있어 동일 문제가 반복될 수 있다.

## 확인 질문

1. 문단/글자 모양 대화상자의 기존 드래그 코드가 왜 실제 위치 이동으로 이어지지 않는지 확인한다.
2. 개체 속성 대화상자와 같은 `position: fixed` 좌표 전환이 필요한지 확인한다.
3. 공통 `dialog-drag` 유틸로 모달 드래그 동작을 묶을 수 있는지 확인한다.

## 진행 계획

1. 문단 모양/글자 모양 대화상자의 title bar 드래그 경로를 확인한다.
2. `left/top` 변경이 적용되지 않는 원인을 찾아 개체 속성 방식과 맞춘다.
3. 공통 유틸을 추가하고 기존 중복 드래그 구현을 치환한다.
4. TypeScript 빌드와 작업지시자 시각 확인을 거친다.

## 현재 상태

- 2026-06-01: Stage6 변경분을 커밋한 뒤 새 스테이지 문서를 생성했다.
- 2026-06-01: 문단 모양/글자 모양 대화상자는 드래그 핸들러가 있었지만, 이동 시작 시 `position: fixed`로 전환하지 않아 `left/top` 변경이 실제 배치에 반영되지 않는 문제를 확인했다.
- 2026-06-01: `dialog-drag.ts` 공통 유틸을 추가하고 문단 모양, 글자 모양, 개체 속성, 수식 속성, 수식 편집, 문자표, 책갈피, 셀 나누기, 표 만들기, 검증 모달에 같은 드래그 로직을 적용했다.
- 2026-06-02: `upstream/devel` 동기화 후 rebase를 완료했고, Stage7 미커밋 변경분을 재적용했다.

## 검증

- 2026-06-01: `npm run build` 통과.
- 2026-06-01: `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(41 passed, 0 failed).
- 2026-06-02: 수식 관련 closed PR 회귀 점검으로 `cargo test --lib renderer::equation -- --nocapture`, `cargo test --test issue_1219_equation_line_hangul_advance -- --nocapture`, `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
- 2026-06-02: rebase 후 로컬 WASM `pkg` 타입 정의가 낡아 `npm run build`가 1차 실패했으나, `wasm-pack build --target web --out-dir pkg` 실행 후 `npm run build` 재실행 통과.
- 2026-06-02: 작업지시자가 커밋 후 PR 준비를 지시했다.

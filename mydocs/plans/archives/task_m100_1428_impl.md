# Task 1428 구현 계획서

## 구현 범위

이번 작업은 rhwp-studio UI 동작 정합화가 중심이다. `비율 유지`는 파일 포맷 저장 속성이 아니므로 HWP5/HWPX IR 필드 추가나 serializer 변경은 하지 않는다. 선택값은 다크모드처럼 `rhwp-settings`에 저장되는 사용자 UI 설정으로만 다룬다.

## Stage 1: 개체 속성 기본 탭 비율 유지 옵션

- `rhwp-studio/src/ui/picture-props-dialog.ts`에 기본 탭 전용 `keepRatioCheck`를 추가한다.
- 한컴 UI처럼 `크기 고정(S)` 옆 또는 아래에 `비율 유지` 체크박스를 배치한다.
- 기본값은 기존 rhwp-studio 동작을 깨지 않도록 ON으로 시작하되, 사용자가 OFF하면 `rhwp-settings`에 저장해 다음 대화상자 열기에서도 독립 입력을 유지한다.
- `rhwp-studio/src/core/user-settings.ts`에 대화상자 UI 설정 섹션을 추가해 다크모드와 같은 저장 키(`rhwp-settings`)를 사용한다.
- `widthInput`/`heightInput`의 상호 갱신은 체크 ON일 때만 수행한다.
- 이벤트 루프를 막기 위해 programmatic update 중 재진입 방지 플래그를 둔다.

## Stage 2: 모달 외부 클릭 닫힘 차단

- `ModalDialog`의 기본값을 외부 클릭 닫힘 금지로 변경하거나, 외부 클릭 close 경로를 제거한다.
- 개체 속성의 자체 overlay `mousedown` close 처리를 제거한다.
- 설명 입력 같은 보조 모달도 외부 클릭으로 닫히지 않게 정리한다.
- command palette나 드롭다운성 UI처럼 모달이 아닌 transient UI는 범위에서 분리한다.

## Stage 3: 검증

- rhwp-studio E2E에 개체 속성 기본 탭 검증을 추가한다.
  - OFF: 너비 변경 후 높이 유지
  - ON: 너비 변경 후 높이 자동 갱신
  - overlay 클릭 후 모달 유지
- 기존 개체 속성 저장 동작이 유지되는지 확인한다.
- 필요 시 `npm` 기반 타입 검사 또는 E2E 스크립트를 수행한다.

## Stage 4: 보고와 커밋

- `mydocs/working/task_m100_1428_stage1.md`에 구현/검증 결과를 기록한다.
- 소스 수정과 stage 문서를 함께 커밋한다.
- PR 준비 시 로컬 필수 검증과 `cargo clippy --all-targets -- -D warnings` 수행 여부를 별도 기록한다.

## 주의사항

- `비율 유지`를 HWP/HWPX 저장 속성으로 모델링하지 않는다.
- 크기 고정(`sizeFixed`)과 비율 유지(`keepRatio`)는 서로 다른 UI 옵션이다.
- 비율 유지 선택값은 문서가 아니라 사용자 환경설정이며, 파일 저장/로드 결과를 바꾸지 않는다.
- 외부 클릭 닫힘 차단은 모달/팝업 대화상자에 한정하고, 사용자 기대상 바깥 클릭으로 닫히는 메뉴성 UI와 혼동하지 않는다.

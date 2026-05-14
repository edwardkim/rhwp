# Task #886 Stage 1 완료 보고서

## 단계

- Stage 1 — dirty state 기반 추가

## 작업 요약

rhwp-studio에 저장되지 않은 변경사항 상태를 관리하는 `DocumentDirtyState` 서비스를 추가했다. 이 서비스는 dirty/clean 상태를 단일 위치에서 관리하고, dirty 상태일 때만 브라우저 기본 `beforeunload` 확인창이 동작하도록 한다.

## 변경 파일

| 파일 | 내용 |
|------|------|
| `rhwp-studio/src/core/document-dirty-state.ts` | dirty state 서비스 신규 추가, `beforeunload` 핸들러 설치/해제 구현 |
| `rhwp-studio/src/command/types.ts` | `EditorContext.isDirty`, `CommandServices.documentState` 추가 |
| `rhwp-studio/src/main.ts` | dirty state 생성/주입, DEV 전역 노출, 문서 초기화 후 clean/dirty 처리 |
| `rhwp-studio/src/engine/input-handler.ts` | 실제 편집 후 `document-mutated` 이벤트 발행 |

## 주요 결정

처음에는 기존 `document-changed` 이벤트를 dirty 전환 기준으로 사용할 수 있을 것으로 보였으나, 확인 결과 이 이벤트는 보기 옵션 변경 후 재렌더에도 사용된다.

예:
- 조판 부호 표시/숨김
- 문단 부호 표시/숨김
- 투명 선 표시/숨김
- 잘림 보기 토글

이 이벤트 전체를 dirty로 처리하면 저장 대상이 아닌 보기 변경만으로도 닫기 확인창이 뜬다. 따라서 Stage 1에서는 실제 문서 내용 변경을 나타내는 `document-mutated` 이벤트를 별도로 추가했다.

현재 Stage 1에서 dirty로 잡히는 경로:
- `InputHandler.executeOperation()` 이후 `afterEdit()`를 통과하는 일반 편집 경로
- 비표준 lineseg 자동 보정이 실제로 수행된 문서 로드 경로

Stage 2에서 직접 WASM 변경을 수행하는 파일/삽입/표/쪽/서식 명령 경로를 추가 점검하고 누락 경로를 `document-mutated`로 확장한다.

## 동작

- 문서 로드 또는 새 문서 생성 완료 후 clean 처리한다.
- 편집 작업 후 dirty 처리한다.
- validation 자동 보정이 수행된 경우에는 로드 직후라도 dirty 처리한다.
- dirty 상태에서만 `beforeunload`가 `preventDefault()`와 `returnValue`를 설정한다.
- dirty 상태 변경 시 `document-dirty-changed` 이벤트를 발행하고 커맨드 상태 갱신을 요청한다.

## 검증

```bash
cd rhwp-studio
npm run build
```

결과:
- `tsc` 통과
- `vite build` 통과
- 기존 chunk size warning만 표시됨

## 남은 작업

- Stage 2에서 `저장`, `저장 안 함`, `취소` 3선택 모달을 추가한다.
- Stage 2에서 `file:new-doc`, `file:open`, 파일 input/drop, `open-document-bytes` 문서 교체 경로를 보호한다.
- Stage 2에서 저장 성공 시 clean 처리 계약을 정리한다.
- Stage 3에서 dirty state 단위 테스트와 E2E 검증을 추가한다.

## 승인 요청

Stage 1 구현 및 검증 완료. 다음 단계(Stage 2 — 저장 확인 모달과 문서 교체 보호 추가) 진행 승인을 요청한다.

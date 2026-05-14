# Task #886 Stage 2 완료 보고서

## 단계

- Stage 2 — 저장 확인 모달과 문서 교체 보호 추가

## 작업 요약

dirty 상태에서 rhwp-studio 내부 문서 교체 동작을 실행할 때 `저장`, `저장 안 함`, `취소`를 선택할 수 있는 저장 확인 모달을 추가했다. 저장 로직은 결과값을 반환하도록 분리했고, 저장 성공 시 dirty 상태를 clean으로 되돌리도록 했다.

## 변경 파일

| 파일 | 내용 |
|------|------|
| `rhwp-studio/src/ui/unsaved-changes-dialog.ts` | `저장`, `저장 안 함`, `취소` 3선택 모달 신규 추가 |
| `rhwp-studio/src/command/commands/file.ts` | `saveCurrentDocument()`, `confirmSaveBeforeReplacingDocument()` 추가, 저장 성공 clean 처리 |
| `rhwp-studio/src/main.ts` | 새 문서/열기/파일 input/drop/`open-document-bytes`/postMessage 문서 로드 보호 |
| `rhwp-studio/src/command/commands/view.ts` | 보기 전용 재렌더 이벤트를 `document-view-changed`로 분리 |
| `rhwp-studio/src/view/canvas-view.ts` | `document-view-changed` 렌더 갱신 연결 |
| `rhwp-studio/src/view/ruler.ts` | `document-view-changed` 눈금자 갱신 연결 |

## 주요 결정

### 저장 결과 계약

기존 `file:save` 내부 저장 로직을 `saveCurrentDocument()`로 추출했다.

반환값:

| 값 | 의미 |
|----|------|
| `saved` | 저장 완료, dirty clean 처리 |
| `cancelled` | 사용자가 저장 대화상자 취소 |
| `failed` | 저장 실패 |
| `unsupported` | HWPX 출처 문서처럼 현재 저장 불가 |

문서 교체는 `saved` 또는 `discard` 선택일 때만 진행한다.

### 보기 이벤트 분리

Stage 1에서 `document-mutated`만 dirty 기준으로 삼았으나, 직접 WASM 변경 명령들은 기존 `document-changed`를 넓게 사용하고 있었다. 대신 보기 메뉴의 재렌더 이벤트를 `document-view-changed`로 분리하고, 나머지 `document-changed`는 dirty 전환 기준에 포함했다.

이로써 직접 편집 명령과 다이얼로그 적용 경로의 dirty 누락 위험을 줄였다.

## 보호된 문서 교체 경로

- `file:new-doc`
- `file:open`
- 숨김 파일 input fallback
- 드래그 앤 드롭 파일 열기
- `open-document-bytes` 이벤트
- `hwpctl-load` postMessage
- `rhwp-request/loadFile` postMessage

`file:open`에서 File System Access API가 없는 환경은 숨김 파일 input으로 fallback한다. 이 경우 이미 저장 확인을 거쳤으므로 `data-skip-unsaved-guard` 플래그로 이중 모달을 방지했다.

## HWPX 정책

HWPX 출처 문서는 기존 정책대로 직접 저장을 비활성화한다. dirty 상태에서 문서 교체 모달을 표시할 때는 `저장` 버튼을 비활성화하고, 사용자는 `저장 안 함` 또는 `취소`를 선택할 수 있다.

## 검증

### 빌드

```bash
cd rhwp-studio
npm run build
```

결과:
- `tsc` 통과
- `vite build` 통과
- 기존 chunk size warning만 표시됨

### 브라우저 수동 검증

대상:
- `http://127.0.0.1:7700/`
- Browser plugin / in-app browser

흐름:
1. rhwp-studio 로드
2. `Alt+N`으로 새 문서 생성
3. 본문에 `저장 확인 테스트` 입력
4. `새로 만들기` 실행
5. 저장 확인 모달 표시 확인
6. `취소` 선택 시 모달이 닫히고 기존 문서가 유지되는지 확인
7. 다시 `새로 만들기` 실행
8. `저장 안 함` 선택 시 새 문서로 전환되는지 확인

확인 결과:
- 저장 확인 모달 표시됨
- 버튼 3개 표시됨: `저장`, `저장 안 함`, `취소`
- `취소` 선택 시 문서 교체 중단
- `저장 안 함` 선택 시 새 문서 전환
- 브라우저 콘솔 warn/error 0건

## 남은 작업

- Stage 3에서 `DocumentDirtyState` 단위 테스트를 추가한다.
- Stage 3에서 내부 모달 흐름 E2E를 자동화한다.
- Stage 3에서 `beforeunload` prevent 동작을 테스트 가능한 범위에서 검증한다.
- Stage 3에서 저장 성공 후 dirty clean 상태를 테스트한다.

## 승인 요청

Stage 2 구현 및 검증 완료. 다음 단계(Stage 3 — beforeunload 보호와 회귀 테스트 추가) 진행 승인을 요청한다.

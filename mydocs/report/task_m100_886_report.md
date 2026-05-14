# Task #886 최종 결과 보고서

## 이슈

- 이슈 번호: #886
- 제목: rhwp-studio(extension): 웹 창 닫기 전 저장 여부 확인
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task886`

## 목표

rhwp-studio에서 저장되지 않은 문서 변경사항이 있을 때, 사용자가 문서를 닫거나 다른 문서로 전환하는 과정에서 변경사항을 잃지 않도록 보호 흐름을 추가한다.

## 최종 구현 요약

1. dirty state 관리 서비스 추가
   - `DocumentDirtyState`를 추가해 dirty/clean 상태를 단일 위치에서 관리한다.
   - dirty 변경 시 `document-dirty-changed` 이벤트를 발행한다.

2. 브라우저 이탈 보호 추가
   - dirty 상태에서만 `beforeunload`가 `preventDefault()`와 `returnValue = ''`를 설정한다.
   - 브라우저 창/탭 닫기, 새로고침, 페이지 이탈은 브라우저 기본 확인창으로 보호한다.

3. 앱 내부 문서 교체 보호 추가
   - dirty 상태에서 새 문서/열기/문서 로드 이벤트를 실행하면 커스텀 저장 확인 모달이 표시된다.
   - 버튼은 `저장`, `저장 안 함`, `취소`로 구성했다.
   - `취소`는 문서 교체를 중단한다.
   - `저장 안 함`은 기존 변경사항을 버리고 문서 교체를 진행한다.
   - `저장`은 저장 성공 시에만 문서 교체를 진행한다.

4. 저장 결과 계약 정리
   - `saveCurrentDocument()`를 추가해 저장 결과를 `saved`, `cancelled`, `failed`, `unsupported`로 구분한다.
   - 저장 성공 시 dirty 상태를 clean으로 되돌린다.

5. 보기 전용 재렌더 분리
   - `view:*` 표시 옵션은 저장 대상 변경이 아니므로 `document-view-changed`로 분리했다.
   - 기존 `document-changed`는 실제 문서 변경 경로의 dirty 전환 기준으로 사용한다.

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `rhwp-studio/src/core/document-dirty-state.ts` | dirty state와 `beforeunload` 처리 |
| `rhwp-studio/src/command/types.ts` | `EditorContext.isDirty`, `CommandServices.documentState` 추가 |
| `rhwp-studio/src/main.ts` | dirty state wiring, 문서 교체 보호, postMessage 로드 보호 |
| `rhwp-studio/src/command/commands/file.ts` | 저장 결과 계약, 저장 확인 보호 유틸 |
| `rhwp-studio/src/ui/unsaved-changes-dialog.ts` | `저장 / 저장 안 함 / 취소` 모달 |
| `rhwp-studio/src/command/commands/view.ts` | 보기 전용 이벤트 분리 |
| `rhwp-studio/src/view/canvas-view.ts` | `document-view-changed` 렌더 갱신 연결 |
| `rhwp-studio/src/view/ruler.ts` | `document-view-changed` 눈금자 갱신 연결 |
| `rhwp-studio/tests/document-dirty-state.test.ts` | dirty state 단위 테스트 |
| `rhwp-studio/e2e/unsaved-changes-guard.test.mjs` | 저장 확인 모달 E2E |
| `rhwp-studio/package.json` | `test`, `e2e:unsaved-guard` 스크립트 추가 |

## 보호 대상 경로

- `file:new-doc`
- `file:open`
- 숨김 파일 input fallback
- 드래그 앤 드롭 파일 열기
- `open-document-bytes` 이벤트
- `hwpctl-load` postMessage
- `rhwp-request/loadFile` postMessage
- 브라우저 창/탭 닫기
- 브라우저 새로고침
- 페이지 이탈

## 브라우저 이탈 동작 기준

브라우저 창/탭 닫기, 새로고침, 페이지 이탈에서는 rhwp-studio의 커스텀 HTML 모달을 띄우지 않는다.

이는 브라우저 보안 정책상 정상이며, 해당 경로는 브라우저 기본 `beforeunload` 확인창이 표시되는 것이 수용 기준이다. 작업지시자도 실제 브라우저에서 기본 확인창 표시를 확인했다.

## 검증

```bash
cd rhwp-studio
npm run test
npm run build
CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' VITE_URL='http://127.0.0.1:7700' npm run e2e:unsaved-guard -- --mode=headless
```

| 검증 | 결과 |
|------|------|
| `npm run test` | 통과 — 7 passed / 0 failed |
| `npm run build` | 통과 — 기존 chunk size warning만 표시 |
| `npm run e2e:unsaved-guard -- --mode=headless` | 통과 |

## E2E 검증 항목

- dirty 상태에서 저장 확인 모달 표시
- `저장`, `저장 안 함`, `취소` 버튼 표시
- `취소` 후 모달 닫힘
- `취소` 후 기존 문서 내용 유지
- `저장 안 함` 후 모달 닫힘
- `저장 안 함` 후 새 문서 전환

## 커밋

- `b13f5db6` — `Task #886 Stage 1: dirty state 기반 추가`
- `7f9d64d3` — `Task #886 Stage 2: 저장 확인 모달과 문서 교체 보호 추가`
- `06900cc1` — `Task #886 Stage 3: beforeunload 보호와 회귀 테스트 추가`

## 남은 제한사항

- 브라우저 기본 `beforeunload` 확인창 문구는 브라우저가 제어하므로 rhwp-studio에서 커스터마이징할 수 없다.
- 브라우저 창/탭 닫기에서 `저장`, `저장 안 함`, `취소` 커스텀 모달을 표시하는 것은 지원하지 않는다.
- HWPX 출처 문서는 기존 정책대로 직접 저장이 비활성화되어 있으며, 저장 확인 모달에서도 `저장` 버튼을 비활성화한다.

## 결론

Task #886의 목표를 충족했다. 저장되지 않은 변경사항이 있는 상태에서 브라우저 이탈은 기본 확인창으로 보호되고, 앱 내부 문서 교체는 커스텀 저장 확인 모달로 보호된다.

최종 승인 후 이슈 종료 또는 병합 절차를 진행할 수 있다.

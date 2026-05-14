# Task #886 Stage 3 완료 보고서

## 단계

- Stage 3 — beforeunload 보호와 회귀 테스트 추가

## 작업 요약

`DocumentDirtyState`의 dirty/clean 전환과 `beforeunload` 동작을 단위 테스트로 고정했다. 또한 저장되지 않은 변경사항 보호 모달의 핵심 내부 전환 흐름을 E2E 테스트로 추가했다.

## 변경 파일

| 파일 | 내용 |
|------|------|
| `rhwp-studio/tests/document-dirty-state.test.ts` | dirty/clean 이벤트, dirty 상태 beforeunload prevent, 핸들러 해제 검증 |
| `rhwp-studio/e2e/unsaved-changes-guard.test.mjs` | dirty 상태에서 새 문서 시도 시 저장 확인 모달, 취소, 저장 안 함 흐름 검증 |
| `rhwp-studio/package.json` | `npm run test`, `npm run e2e:unsaved-guard` 스크립트 추가 |
| `rhwp-studio/src/core/document-dirty-state.ts` | Node TypeScript strip-only 테스트 호환을 위해 생성자 parameter property 제거 |

## 단위 테스트

추가 테스트:

1. dirty/clean 전환 시 `document-dirty-changed` 이벤트가 상태 변경 시점에만 발행된다.
2. clean 상태의 `beforeunload`는 페이지 이탈을 막지 않는다.
3. dirty 상태의 `beforeunload`는 `preventDefault()`와 `returnValue = ''`를 설정한다.
4. 저장 후 clean 상태에서는 다시 이탈을 막지 않는다.
5. `installBeforeUnload()`가 반환한 해제 함수는 설치한 핸들러를 제거한다.

## E2E 테스트

추가 테스트:

1. 새 문서 생성 후 텍스트 입력으로 dirty 상태를 만든다.
2. 새 문서 요청 시 저장 확인 모달이 표시되는지 확인한다.
3. 모달에 `저장`, `저장 안 함`, `취소` 버튼이 표시되는지 확인한다.
4. `취소` 선택 시 모달이 닫히고 기존 문서 내용이 유지되는지 확인한다.
5. `저장 안 함` 선택 시 모달이 닫히고 새 문서로 전환되는지 확인한다.

테스트에서 사용자 단축키 대신 `window.__eventBus.emit('create-new-document')`를 사용했다. headless Chrome에서 모달 취소 후 포커스가 캔버스/문서 쪽으로 돌아가지 않아 두 번째 `Alt+N` 입력이 불안정했기 때문이다. 이 이벤트는 실제 `file:new-doc` 명령이 호출하는 동일한 문서 교체 경로를 통과한다.

## 검증 명령

```bash
cd rhwp-studio
npm run test
npm run build
CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' VITE_URL='http://127.0.0.1:7700' npm run e2e:unsaved-guard -- --mode=headless
```

## 검증 결과

| 검증 | 결과 |
|------|------|
| `npm run test` | 통과 — 7 passed / 0 failed |
| `npm run build` | 통과 — 기존 chunk size warning만 표시 |
| `npm run e2e:unsaved-guard -- --mode=headless` | 통과 |

E2E 통과 항목:

- dirty 상태에서 저장 확인 모달 표시
- `저장`, `저장 안 함`, `취소` 버튼 표시
- `취소` 후 모달 닫힘
- `취소` 후 기존 문서 내용 유지
- `저장 안 함` 후 모달 닫힘
- `저장 안 함` 후 새 문서 전환

## 참고

첫 E2E 실행은 sandbox 내 headless Chrome 실행이 차단되어 실패했다. 권한 승격 후 Chrome 실행은 가능했다. 이후 첫 번째 테스트 버전은 두 번째 `Alt+N` 입력 포커스 문제로 실패했고, 동일 문서 교체 경로를 직접 호출하도록 테스트를 수정해 통과시켰다.

## 남은 작업

- Stage 4에서 전체 검증 결과를 정리하고 최종 보고서를 작성한다.
- 최종 보고서에서 브라우저 창/탭 닫기 시 커스텀 모달이 아니라 브라우저 기본 확인창이 뜨는 제한사항을 명확히 기록한다.

## 승인 요청

Stage 3 구현 및 검증 완료. 다음 단계(Stage 4 — 최종 검증과 보고서) 진행 승인을 요청한다.

# Task #886 구현 계획서

수행 계획서: [`task_m100_886.md`](./task_m100_886.md)

## 구현 원칙

브라우저 창/탭 닫기와 새로고침은 브라우저가 커스텀 모달을 허용하지 않으므로 `beforeunload` 기본 확인창으로 처리한다. rhwp-studio 내부 동작은 기존 대화상자 체계와 맞춰 `저장`, `저장 안 함`, `취소` 3선택 모달로 처리한다.

dirty state는 `document-changed` 이벤트를 기준으로 dirty로 전환하고, 문서 로드/새 문서 생성/저장 성공 시 clean으로 전환한다. 저장 보호 흐름은 문서 교체 직전에만 실행한다.

## 옵션 비교

### 옵션 A — `main.ts` 로컬 상태로 최소 구현

- `main.ts`에 `let isDirty = false`와 `beforeunload` 핸들러를 둔다.
- 파일 명령과 문서 로드 경로에서 `main.ts` 함수를 직접 참조하거나 이벤트로 통신한다.

장점:
- 변경 파일 수가 적다.
- 초기 구현 속도가 빠르다.

단점:
- `file:open`, `file:new-doc`, `file:save`가 있는 `commands/file.ts`와 상태 소유권이 갈라진다.
- 테스트가 어렵고, 후속 문서 상태 표시 확장에 불리하다.

### 옵션 B — `DocumentDirtyState` 서비스 분리

- `rhwp-studio/src/core/document-dirty-state.ts`를 추가한다.
- `CommandServices`에 dirty state 서비스를 주입한다.
- `main.ts`에서 `document-changed` 이벤트, 문서 로드/생성 완료, `beforeunload`를 연결한다.
- `commands/file.ts`에서 저장 보호 유틸과 저장 성공 clean 처리를 수행한다.

장점:
- 상태 소유권이 명확하고 테스트가 쉽다.
- 명령, 이벤트, UI 보호 흐름이 같은 서비스를 공유한다.
- 이후 상태 표시줄의 `*` 표시, 자동 저장, 이력 저장과 연결하기 쉽다.

단점:
- 타입과 서비스 주입 변경이 필요하다.

### 옵션 C — WASM 스냅샷 비교 기반 dirty 판정

- 저장 시점 또는 로드 시점의 HWP export 결과와 현재 export 결과를 비교한다.

장점:
- 실제 저장 결과 기준 판정이 가능하다.

단점:
- 대용량 문서에서 비용이 크다.
- HWPX 저장 비활성화 정책과 충돌한다.
- 문서 변경 이벤트가 이미 있으므로 과하다.

## 권장안

**옵션 B**를 채택한다.

현재 rhwp-studio는 `document-changed` 이벤트가 이미 넓게 발행되고 있으며, 파일 명령은 `CommandServices`를 통해 필요한 서비스에 접근하는 구조다. 따라서 dirty state를 별도 서비스로 분리하고 `CommandServices`에 주입하는 방식이 기존 구조와 가장 잘 맞는다.

## 단계별 진행

### Stage 1 — dirty state 기반 추가

목적: 저장되지 않은 변경사항 상태를 단일 서비스로 관리한다.

변경 예상:
- `rhwp-studio/src/core/document-dirty-state.ts` 신규
  - `isDirty()`
  - `markDirty(reason?: string)`
  - `markClean(reason?: string)`
  - `setBeforeUnloadEnabled(enabled: boolean)` 또는 `installBeforeUnload(window)`
  - dirty 변경 시 `document-dirty-changed` 이벤트 발행
- `rhwp-studio/src/command/types.ts`
  - `CommandServices.documentState` 추가
  - 필요 시 `EditorContext.isDirty` 추가
- `rhwp-studio/src/main.ts`
  - `DocumentDirtyState` 생성 및 `commandServices` 주입
  - `eventBus.on('document-changed')`에서 dirty 표시
  - 문서 로드/새 문서 생성 완료 후 clean 표시

검증:
- 새 문서 생성 직후 clean
- 텍스트 입력/서식 변경 후 dirty
- 문서 로드 직후 clean

산출물:
- `mydocs/working/task_m100_886_stage1.md`

### Stage 2 — 저장 결과 계약과 내부 문서 교체 보호

목적: dirty 상태에서 새 문서/열기/외부 로드가 저장 여부를 묻도록 한다.

변경 예상:
- `rhwp-studio/src/ui/unsaved-changes-dialog.ts` 신규 또는 `confirm-dialog.ts` 확장
  - 반환값: `'save' | 'discard' | 'cancel'`
  - 버튼: `저장`, `저장 안 함`, `취소`
  - HWPX 저장 비활성화 문서는 `저장` 선택 시 저장 불가 안내 후 중단하거나 버튼 비활성 처리
- `rhwp-studio/src/command/commands/file.ts`
  - 기존 저장 로직을 `saveCurrentDocument(services): Promise<'saved' | 'cancelled' | 'failed'>`로 추출
  - `file:save`는 추출 함수를 호출
  - 저장 성공 시 `documentState.markClean('save')`
  - `file:new-doc`, `file:open` 전에 `confirmSaveBeforeReplacingDocument(services)` 호출
- `rhwp-studio/src/main.ts`
  - 파일 input/drop, `open-document-bytes`, postMessage `loadFile` 등 명령을 우회하는 문서 교체 경로에 동일 보호 적용
  - 내부 테스트/자동 로드가 필요한 경우에만 명시적 `skipUnsavedGuard` 플래그 검토

검증:
- dirty 상태에서 새 문서 선택 → 모달 표시
- `취소` → 기존 문서 유지
- `저장 안 함` → 새 문서/열기 진행
- `저장` → 저장 성공 후 새 문서/열기 진행
- clean 상태에서는 모달 없이 진행

산출물:
- `mydocs/working/task_m100_886_stage2.md`

### Stage 3 — 브라우저 이탈 보호와 테스트

목적: 탭/창 닫기, 새로고침, 페이지 이탈에서 저장되지 않은 변경사항 유실을 막는다.

변경 예상:
- `DocumentDirtyState` 또는 `main.ts`
  - dirty 상태일 때만 `beforeunload`에서 `event.preventDefault()`와 `event.returnValue = ''` 설정
  - clean 상태에서는 이탈 확인이 발생하지 않도록 처리
- `rhwp-studio/tests/document-dirty-state.test.ts` 신규
  - dirty/clean 전환
  - beforeunload 이벤트 prevent 여부
- `rhwp-studio/e2e/unsaved-changes-guard.test.mjs` 신규 또는 기존 E2E 확장
  - 내부 모달 버튼 동작 검증
  - 브라우저 기본 이탈창은 자동화 제약이 있으므로 가능한 경우 `page.on('dialog')` 또는 페이지 evaluate 기반으로 검증

검증 명령:

```bash
cd rhwp-studio
npm run build
node --test tests/*.test.ts
node e2e/unsaved-changes-guard.test.mjs --mode=headless
```

브라우저 이탈 확인 수동 검증:
1. rhwp-studio 실행
2. 새 문서 생성
3. 텍스트 입력
4. 브라우저 새로고침 또는 탭 닫기
5. 브라우저 기본 확인창 표시 확인
6. 문서 저장
7. 다시 새로고침 또는 탭 닫기 시 확인창 미표시 확인

산출물:
- `mydocs/working/task_m100_886_stage3.md`

### Stage 4 — 최종 검증과 보고서

목적: 구현 결과를 정리하고 최종 승인 요청 상태로 만든다.

검증:
- `npm run build`
- `node --test tests/*.test.ts`
- 추가 E2E 실행 결과 기록
- HWPX 문서에서 저장 비활성화 정책 유지 확인
- Chrome/Edge/Safari 확장 보안 제약상 커스텀 beforeunload 모달이 불가하다는 제한사항 재확인

문서:
- `mydocs/working/task_m100_886_stage4.md`
- `mydocs/report/task_m100_886_report.md`
- `mydocs/orders/20260514.md` 상태 갱신

산출물:
- 단계별 보고서
- 최종 결과 보고서
- 작업지시자 최종 승인 요청

## 변경 파일 예상

| 파일 | 변경 종류 | 내용 |
|------|----------|------|
| `rhwp-studio/src/core/document-dirty-state.ts` | 신규 | dirty state와 beforeunload 처리 |
| `rhwp-studio/src/command/types.ts` | 수정 | `CommandServices`/`EditorContext`에 dirty state 노출 |
| `rhwp-studio/src/main.ts` | 수정 | dirty state wiring, 문서 교체 경로 보호 |
| `rhwp-studio/src/command/commands/file.ts` | 수정 | 저장 결과 계약, 새 문서/열기 보호 |
| `rhwp-studio/src/ui/unsaved-changes-dialog.ts` | 신규 | 3선택 저장 확인 모달 |
| `rhwp-studio/tests/document-dirty-state.test.ts` | 신규 | 단위 테스트 |
| `rhwp-studio/e2e/unsaved-changes-guard.test.mjs` | 신규 | 내부 보호 흐름 E2E |
| `mydocs/working/task_m100_886_stage{1..4}.md` | 신규 | 단계별 완료 보고서 |
| `mydocs/report/task_m100_886_report.md` | 신규 | 최종 보고서 |
| `mydocs/orders/20260514.md` | 수정 | 진행 상태 갱신 |

## 단계별 커밋 전략

| Stage | 커밋 메시지 | 포함 범위 |
|------|-------------|----------|
| Stage 1 | `Task #886 Stage 1: dirty state 기반 추가` | dirty state 서비스, main wiring, Stage 1 보고서 |
| Stage 2 | `Task #886 Stage 2: 저장 확인 모달과 문서 교체 보호 추가` | 저장 결과 계약, 내부 보호 모달, Stage 2 보고서 |
| Stage 3 | `Task #886 Stage 3: beforeunload 보호와 회귀 테스트 추가` | beforeunload, 단위/E2E 테스트, Stage 3 보고서 |
| Stage 4 | `Task #886: 최종 보고서와 검증 정리` | 최종 보고서, orders 갱신, Stage 4 보고서 |

## 위험 영역과 가드

| 위험 | 가드 |
|------|------|
| `file:save`가 비동기인데 dispatcher가 await하지 않음 | 저장 보호 흐름은 dispatcher 우회 없이 추출한 async 저장 함수를 직접 await |
| 저장 취소 후 문서 교체가 계속 진행됨 | 저장 함수 반환값을 `saved/cancelled/failed`로 분리하고 `saved`일 때만 진행 |
| HWPX 문서에서 저장 선택이 혼란을 줌 | 저장 비활성화 정책을 유지하고 저장 불가 시 문서 교체 중단 |
| 테스트용 `eventBus.emit('create-new-document')`가 보호 흐름을 우회함 | 사용자 경로와 테스트/내부 경로를 구분하고 필요한 경우 명시적 `skipUnsavedGuard`만 허용 |
| 브라우저별 `beforeunload` 동작 차이 | 수용 기준을 기본 확인창 표시 여부로 제한하고 문구 커스텀은 제외 |

## 진행 조건

본 구현 계획서 승인 후 Stage 1 소스 수정과 Stage 1 완료보고서 작성을 시작한다.

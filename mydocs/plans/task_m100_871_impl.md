# Task #871 구현 계획서

**선행**: [task_m100_871.md](task_m100_871.md) 수행계획서 승인

**이슈**: [edwardkim/rhwp#871](https://github.com/edwardkim/rhwp/issues/871)

**브랜치**: `local/task871`

## 구현 목표

rhwp-studio 내부 클립보드가 남아 있어도 외부 앱에서 새로 복사한 시스템 클립보드 값을 정상적으로 붙여넣도록 한다. 동시에 rhwp-studio 내부 복사 직후 붙여넣기는 기존처럼 내부 WASM 클립보드 경로를 사용하여 서식과 컨트롤을 보존한다.

## 핵심 설계

현재 `onPaste()`는 `wasm.hasInternalClipboard()`를 먼저 검사한다. 이를 다음 우선순위로 변경한다.

1. paste 이벤트의 `e.clipboardData`를 먼저 읽는다.
2. 시스템 클립보드가 rhwp-studio 내부 복사 결과인지 marker로 판별한다.
3. marker가 현재 내부 클립보드와 매칭되면 `pasteInternal` 또는 `pasteControl`을 사용한다.
4. marker가 없으면 외부 클립보드로 보고 이미지, HTML, plain text 순서로 처리한다.

marker는 `text/html` 안에 rhwp-studio 전용 attribute 또는 comment 형태로 삽입한다. 일반 외부 HTML 붙여넣기와 충돌하지 않도록 `data-rhwp-clipboard-id` 같은 충분히 구체적인 식별자를 사용한다.

## 단계별 진행

### Stage 1 — 붙여넣기 경로 실측 및 marker 설계 확정

**목적**: 현재 copy/paste 경로별 데이터 형태를 확인하고 marker 삽입 위치를 확정한다.

**조사 영역**:

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
  - 텍스트 선택 copy/cut 이벤트
  - 그림/도형/표 객체 선택 상태의 Ctrl+C/Ctrl+X 직접 처리
  - `onPaste()` 내부/외부 클립보드 우선순위
- `rhwp-studio/src/engine/input-handler.ts`
  - 메뉴/도구 상자 `performCopy`, `performCut`
- `rhwp-studio/src/core/wasm-bridge.ts`
  - `clearClipboard()` 노출 필요 여부 확인

**판정 기준**:

- 텍스트 선택 복사는 `text/html` marker 삽입 가능 여부 확인
- 표/그림/도형 객체 복사는 `exportControlHtml()` 또는 `ClipboardItem` 경로로 marker 포함 가능 여부 확인
- plain text fallback만 가능한 경로는 내부 클립보드 오판을 피할 보수적 처리 방침 확정

**산출물**:

- `mydocs/working/task_m100_871_stage1.md`
  - copy/paste 경로별 현행 동작
  - marker 포맷
  - Stage 2 구현 범위 확정

### Stage 2 — 내부 클립보드 marker 및 붙여넣기 우선순위 정정

**목적**: stale 내부 클립보드가 외부 시스템 클립보드를 가리는 결함을 정정한다.

**수정 후보**:

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
  - 내부 복사 HTML에 rhwp marker 삽입 헬퍼 추가
  - `onCopy()`의 텍스트 선택 HTML 설정 시 marker 포함
  - 그림/도형 객체 직접 복사 경로의 HTML marker 포함
  - 표 객체 직접 복사 경로도 가능하면 `exportControlHtml()` + marker 포함 경로로 정리
  - `onPaste()`에서 `e.clipboardData`를 먼저 읽고 marker 판별 후 내부/외부 경로 선택
- `rhwp-studio/src/core/wasm-bridge.ts`
  - 필요 시 `clearClipboard()` 래퍼 추가

**정정 후 흐름**:

```text
onPaste(e)
  -> clipboardData 수집
  -> rhwp marker 존재 + 내부 클립보드 존재
       -> pasteInternal / pasteControl
  -> 이미지 파일 존재
       -> pasteImageFile
  -> 외부 text/html 존재
       -> pasteHtml / pasteHtmlInCell
  -> 외부 text/plain 존재
       -> InsertTextCommand / SplitParagraphCommand
```

**중요 가드**:

- 외부 HTML에 rhwp marker가 없으면 내부 클립보드를 사용하지 않는다.
- marker가 있더라도 내부 클립보드가 없으면 외부 HTML/plain text fallback으로 처리한다.
- 이미지 파일은 외부 클립보드 경로에서 계속 우선 처리한다.
- 내부 표/그림 복사 후 바로 붙여넣기는 컨트롤 보존 경로가 유지되어야 한다.

**산출물**:

- 코드 수정
- `mydocs/working/task_m100_871_stage2.md`
  - 변경 파일
  - 내부/외부 우선순위 변경 설명
  - 수동 검증 결과

### Stage 3 — 메뉴/도구 상자 붙이기 경로 및 회귀 테스트 정리

**목적**: 키보드 붙여넣기와 메뉴/도구 상자 붙이기 동작의 차이를 확인하고, 자동 회귀 테스트를 추가한다.

**수정 후보**:

- `rhwp-studio/src/command/commands/edit.ts`
  - `document.execCommand('paste')` 의존 한계 확인
  - 브라우저 보안 정책상 직접 paste 이벤트 생성이 불가능하면 현행 유지 + 문서화
  - 가능한 경우 input handler 경로와 일관된 호출 방식으로 정리
- `rhwp-studio/e2e/copy-paste.test.mjs` 또는 신규 e2e
  - 내부 복사 후 내부 붙여넣기 기존 시나리오 유지
  - 내부 복사 후 외부 텍스트를 시스템 클립보드에 주입하고 Cmd+V 결과 확인
  - 외부 plain text 다중 줄 붙여넣기 확인

**검증 시나리오**:

1. `abcdefg` 내부 복사 후 내부 붙여넣기 → `abcdefgabcdefg`
2. 내부 복사 후 외부 텍스트 `OUTSIDE` 복사 후 붙여넣기 → `OUTSIDE`
3. 외부 HTML 붙여넣기 → `pasteHtml` 경로 유지
4. 표 객체 내부 복사 후 붙여넣기 → `pasteControl` 경로 유지

**산출물**:

- e2e 테스트 수정 또는 추가
- `mydocs/working/task_m100_871_stage3.md`
  - 자동 검증 결과
  - 메뉴/도구 상자 붙이기 판정

### Stage 4 — 최종 검증 및 보고

**목적**: 코드, 테스트, 문서 상태를 정리하고 작업지시자 최종 판정을 받을 수 있게 한다.

**검증 명령 후보**:

- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && node e2e/copy-paste.test.mjs`
- 필요 시 `cargo test --lib` 또는 클립보드 관련 Rust 테스트 범위 실행

**마무리 문서**:

- `mydocs/report/task_m100_871_report.md`
- `mydocs/orders/20260513.md` 상태 갱신

**산출물**:

- 최종 보고서
- orders 갱신
- 작업지시자 시각 판정 요청

## 단계별 commit 전략

| Stage | commit | 영역 |
|-------|--------|------|
| Stage 1 | `Task #871 Stage 1: 클립보드 경로 진단 보고` | 진단 보고서 |
| Stage 2 | `Task #871 Stage 2: 외부 클립보드 우선순위 정정` | marker + paste routing 코드 |
| Stage 3 | `Task #871 Stage 3: 붙여넣기 회귀 테스트 추가` | e2e/메뉴 경로 정리 |
| Stage 4 | `Task #871: 최종 보고서 및 orders 갱신` | 최종 보고서 + orders |

## 위험 요소와 대응

| 위험 | 내용 | 대응 |
|------|------|------|
| 내부 서식 보존 회귀 | marker 판별 실패 시 내부 복사도 외부 HTML로 붙을 수 있음 | 내부 복사 직후 marker 매칭 테스트 추가 |
| 표/그림 컨트롤 회귀 | 객체 복사 경로 일부가 `writeText`만 사용 | 객체 경로도 HTML marker 또는 보수적 fallback 정리 |
| 외부 HTML 회귀 | 외부 HTML을 내부 클립보드로 오판 가능 | marker exact match만 내부 경로 허용 |
| 브라우저 보안 제한 | 메뉴 붙이기의 `execCommand('paste')`가 제한될 수 있음 | 키보드 paste 경로를 기준으로 검증하고 메뉴 경로는 가능한 범위 명시 |
| Clipboard API 호환성 | `ClipboardItem` 지원 차이 | 기존 `writeText` fallback 유지, fallback 시 외부 값 우선 정책 유지 |

## 승인 지점

각 Stage 완료 후 `mydocs/working/task_m100_871_stage{N}.md`를 작성하고 작업지시자 승인 후 다음 Stage로 진행한다. 소스 수정은 Stage 2부터 시작하며, Stage 1 완료 보고 승인 전에는 소스 코드를 수정하지 않는다.

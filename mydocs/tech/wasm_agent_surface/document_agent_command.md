---
kind: canonical
status: active
canonical: mydocs/tech/wasm_agent_surface/document_agent_command.md
last_verified: 2026-08-19
---

# 문서 에이전트 exact command 계약

서버가 HWP/HWPX 원문에서 만든 문단 교체 후보를 `@rhwp/editor`가 현재 열린 문서에 적용하는
v1 계약이다. 서버는 **후보 생성 권위**, Studio는 **현재 문서 상태와 쓰기 권위**를 가진다.
명령은 자유 형식 편집 지시가 아니라 상태·원문·서식·인접 문맥을 모두 결속한 exact command다.

구현 정본은 다음 네 곳이다.

- 공개 SDK와 strict 응답 검증: `npm/editor/index.js`, `npm/editor/document-agent-contract.js`
- iframe capability/session 경계: `npm/editor/transport.js`, `rhwp-studio/src/embed/`
- 명령 트랜잭션과 postcondition: `rhwp-studio/src/document-agent/controller.ts`
- 기존 편집기 undo/selection 경로: `rhwp-studio/src/engine/input-handler.ts`

## 1. 공개 표면

```ts
const state = await editor.getDocumentState();
const selection = await editor.getSelectionContext();
const receipt = await editor.applyTextCommand(command);
const reverted = await editor.revertTextCommand(revertCommand);
await editor.focusTarget(target);

const off = editor.onDocumentChanged((event) => {
  // reason: agent_apply | agent_revert
});
off();
```

Studio가 해당 capability를 광고하지 않으면 SDK는 요청을 보내지 않고
`CAPABILITY_UNSUPPORTED`로 실패한다. v1 DTO는 알 수 없는 필드를 허용하지 않는다.

| 메서드 | capability | 역할 |
| --- | --- | --- |
| `getDocumentState` | `document-state-v1` | epoch, change sequence, 페이지 수, 현재 직렬화 바이트 SHA |
| `getSelectionContext` | `selection-context-v1` | 현재 body paragraph exact target과 선택 SHA |
| `applyTextCommand` / `revertTextCommand` | `document-agent-command-v1` | 검증된 단일 문단 교체·역패치 |
| `focusTarget` | `target-navigation-v1` | mutation 없이 exact 문단 선택·중앙 이동 |
| `onDocumentChanged` | `document-change-events-v1` | 성공적으로 commit된 agent apply/revert 통지 |

## 2. 지원 target과 제한

v1은 본문 문단 전체만 지원한다.

```json
{
  "kind": "body_paragraph",
  "section": 0,
  "paragraph": 12,
  "charOffset": 0,
  "length": 31
}
```

- `charOffset`은 항상 `0`이다.
- `length`와 모든 offset은 UTF-16 code unit가 아니라 Unicode code point 수다.
- 대상 문단은 최대 4,000 code point다.
- replacement는 한 문단의 plain text이며 U+0000..U+001F, U+007F control 문자를 허용하지 않는다.
- 대상에 control, field, 혼합 `charShapeId`가 있으면 `TARGET_FORMAT_MISMATCH`다.
- HWP/HWPX만 지원한다. 표 셀, 머리말, 각주, 글상자는 v1 대상이 아니다.
- 인접 문단에는 control이나 혼합 서식이 있어도 되며, 그 상태 전체가 context SHA에 들어간다.

## 3. SHA-256 정규형

모든 digest는 **정규화하지 않은 원문 문자열의 UTF-8 바이트** 또는 아래에 정의한 공백 없는
JSON UTF-8 바이트에 SHA-256을 적용한 소문자 64자리 hex다. JSON key 순서는 이 문서의 순서로
고정한다. 런타임 `stable_id`는 세션 계보 값이므로 어떤 외부 digest에도 넣지 않는다.

### 3.1 원문 SHA

```text
expectedBeforeSha256 = sha256(utf8(targetParagraphText))
```

예제 `기존 문단`의 값은
`73b107c1b8b366ea082beba6cf29568890d48845fa212849f74b516209a6378e`다.

### 3.2 서식 SHA

```json
{"schemaVersion":1,"charShapeId":4,"paraShapeId":2,"styleId":3}
```

위 바이트의 SHA는
`5479c59c8c99dde1c9bb45c70a4689eab28233b28dcf31976db9ab8e4b74ec71`다.
대상 문단의 모든 code point가 같은 `charShapeId`일 때만 이 값을 만든다. 빈 문단은 offset 0의
기본 글자 모양을 사용한다.

### 3.3 인접 문맥 SHA

바로 앞·뒤 문단을 다음 고정 key 순서로 직렬화한다. 구역 경계 밖이면 `null`이다.

```ts
type AdjacentParagraphV1 = {
  section: number;
  paragraph: number;
  length: number;
  textSha256: string;
  paraShapeId: number;
  styleId: number;
  charShapeIds: number[]; // 각 code point, 빈 문단은 기본값 1개
  controls: number[];     // control text position, 원래 순서
};

JSON.stringify({ schemaVersion: 1, previous, next })
```

`앞 문단` / target `기존 문단` / `뒤 문단`이 모두 `charShapeId=4`, `paraShapeId=2`,
`styleId=3`, control 없음일 때 SHA는
`17026195f4004b9995060c1dd27a14fb8619c9afc502a371cbdebf94025b7568`다.

### 3.4 문서 SHA

`documentSha256`은 현재 IR을 원래 source format으로 직렬화한 전체 바이트의 SHA다.
HWPX writer는 ZIP mtime을 1980-01-01로 고정하므로 같은 상태의 출력은 결정적이다. 명령 작성자는
원본 업로드 파일의 SHA를 추측해 넣지 말고, 적용 직전 `getDocumentState()`가 반환한 값을 사용한다.

## 4. 적용 순서

1. `getDocumentState()`와 서버 후보의 document binding을 비교한다.
2. `expectedDocumentEpoch`, `expectedChangeSeq`, `expectedDocumentSha256`, target의 세 SHA를 넣어
   `applyTextCommand()`를 호출한다.
3. Studio는 mutation 전에 모든 fence를 검사한다.
4. 기존 snapshot undo 경로 안에서 문단 전체를 교체하고 원래 글자·문단 모양을 복원한다.
5. target 밖 semantic manifest, 인접 문맥, 페이지 수, 시간 예산(3초)을 검사한다.
6. 하나라도 다르면 같은 snapshot transaction을 rollback한다. 성공하면 `changeSeq`가 정확히 1
   증가한 receipt와 `documentChanged` 이벤트를 반환한다.

같은 `commandId`와 완전히 같은 binding의 apply/revert 재전송은 저장된 receipt를 돌려준다.
같은 ID에 다른 binding을 쓰면 `COMMAND_REPLAY_MISMATCH`다.

## 5. 되돌리기와 일반 undo

`revertTextCommand()`는 가장 최근에 성공한 agent command만 exact inverse patch로 되돌린다.
apply 뒤 사용자 입력이나 다른 mutation이 한 번이라도 있으면 `COMMAND_NOT_LATEST`로 거부한다.
apply와 revert는 각각 기존 편집기의 snapshot transaction 하나이므로 일반 Ctrl/Cmd+Z history에도
한 단계씩 기록된다. agent revert와 일반 undo를 동시에 성공했다고 간주해서는 안 된다.

## 6. 실패와 증거 경계

대표 오류는 epoch/sequence/document SHA 불일치, target 원문·서식·문맥 불일치, target 밖 변경,
페이지 수 변경, replay 충돌, 시간 초과, transaction 실패다. 모든 실패는 error code로 분기하고
메시지 문자열을 판정 권위로 쓰지 않는다. `TRANSACTION_FAILED`와 `RENDER_FAILED` Error에는
`recovered: boolean`이 붙는다. `false`이면 화면 복구까지 실패한 fatal 상태이므로 host는 추가
mutation을 막고 현재 문서 다운로드 또는 신뢰 가능한 server head 재로드만 제공해야 한다.

단위·계약 테스트와 TypeScript·Vite·WASM 빌드는 경계 로직과 컴파일 가능성을 증명한다. 실제 HWP와
HWPX 파일의 적용·revert·사용자 입력·undo·시각 보존은 실행 중인 Studio를 대상으로 한 브라우저
게이트가 별도로 필요하다.

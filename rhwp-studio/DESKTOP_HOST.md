# Studio 데스크톱 호스트

네이티브 앱에서 전체 Studio 편집기를 사용할 때, 신뢰하는 Studio 최상위 페이지에 모듈 실행 전
`window.rhwpDesktopHost`를 설치합니다. 호스트는 파일 선택기와 디스크 저장을 담당합니다.
Studio는 문서 편집, 내용 손실 검사, 암호 문서 처리, IndexedDB 복구를 담당합니다.

```ts
interface DesktopTransport {
  version: 1;
  postMessage(message: object): void;
  subscribe(listener: (message: unknown) => void): () => void;
}
```

호스트는 신뢰하는 Studio 최상위 프레임에서 발신된 메시지만 처리하고, 외부 링크는 시스템 브라우저에서
엽니다. 호스트에서 문서 경로를 보관하고 Studio에는 파일 이름과 바이트만 보냅니다.

## 메시지

모든 메시지에는 `version: 1`이 포함됩니다.

- Studio는 시작 문서와 복구 처리가 끝나면 `{ kind: 'event', event: 'ready', value }`를 보냅니다.
  초기화 오류에는 `event: 'failed'`, 문서 상태 변경에는 `event: 'changed'`를 보냅니다.
  호스트는 복구 대화상자를 표시할 수 있게 처음부터 Studio를 노출하고, `ready` 이후 파일 요청을 보냅니다.
- 호스트 요청은 `{ kind: 'request', id, method, params }` 형식입니다. Studio는 같은 ID로
  `{ kind: 'response', id, ok: true, result }` 또는
  `{ kind: 'response', id, ok: false, error }`를 보냅니다.
- Studio 메뉴는 `{ kind: 'command', id, command, format?, suggestedName? }`를 보냅니다.
  `command`는 `open`, `save`, `saveAs`입니다. 호스트는
  `{ kind: 'commandResult', id, ok: true, result }`로 완료를 알립니다.
  `result`는 `completed`, `saved`, `cancelled`, `failed` 중 하나입니다.

문서 상태에는 `hasDocument`, `fileName`, `format`(`hwp`, `hwpx`, 또는 `null`), `dirty`,
`canNativeSave`, `documentEpoch`, `changeSeq`가 포함됩니다. 호스트는 서로 다른 문서 세대를
구분하고, 저장하거나 교체하려는 변경 순서를 확인하는 데 이 값을 사용합니다.

| 요청 | 처리 |
| --- | --- |
| `state {}` | 현재 문서 상태를 반환합니다. |
| `load.begin { fileName, byteLength, documentEpoch, changeSeq }` | 사용자에게 교체 확인을 받은 뒤 전송을 시작합니다. 크기 한도는 128 MiB입니다. |
| `load.chunk { offset, bytes }` | 최대 768 KiB의 Base64 조각을 순서대로 보냅니다. |
| `load.commit {}` | 전체 바이트를 검증하고 Studio에서 문서를 연 뒤 상태를 반환합니다. |
| `export { format }` | HWP/HWPX를 직렬화하고 `byteLength`, `format`, `documentEpoch`, `changeSeq`를 반환합니다. |
| `export.chunk { offset, count }` | 최대 768 KiB의 내보낸 데이터를 Base64 형태의 `bytes`로 반환합니다. |
| `saved { fileName, documentEpoch, changeSeq }` | 호스트가 저장을 마친 뒤 이름과 복구 상태를 갱신합니다. |
| `discard { documentEpoch, changeSeq }` | 사용자가 버리기를 선택한 뒤 해당 변경의 복구본을 정리합니다. |
| `release {}` | 진행 중인 전송을 해제합니다. 호스트는 성공·오류·취소 경로에서 호출합니다. |
| `theme { theme }` | `light`, `dark`, `highContrast` 테마를 적용합니다. |

현재 상태가 바뀌면 Studio가 기존 버전에 대한 저장 완료와 복구본 폐기를 거부합니다.
`canNativeSave`가 꺼진 경우 보호 문서 및 특수 형식의 저장은 기존 Studio 파일 메뉴에서 처리합니다.

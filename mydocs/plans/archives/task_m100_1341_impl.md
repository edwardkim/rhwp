# 구현 계획서 — Task M100-1341

## 1. 구현 방향

PWA 파일 연결은 manifest 선언과 런타임 수신을 분리해 구현한다.

- manifest: `rhwp-studio/vite.config.ts`의 `VitePWA({ manifest })`에
  `.hwp/.hwpx` 전용 `file_handlers`를 추가한다.
- runtime: `window.launchQueue`를 feature detection 한 뒤, launch 파일 핸들을
  기존 `open-document-bytes` 이벤트로 전달한다.

기존 로딩/저장 경로를 재사용해 보안 표면을 늘리지 않는다.

## 2. 세부 단계

### Stage 1: Manifest 파일 핸들러 선언

`rhwp-studio/vite.config.ts`:

- `manifest.file_handlers` 추가
- `action`은 현 배포 scope와 같은 `/rhwp/` 사용
- `accept`는 좁은 MIME과 확장자만 사용

후보:

```ts
file_handlers: [
  {
    action: '/rhwp/',
    accept: {
      'application/x-hwp': ['.hwp'],
      'application/hwp+zip': ['.hwpx'],
    },
  },
],
```

금지:

- `application/octet-stream`
- `*/*`
- `.hwp/.hwpx` 외 확장자

### Stage 2: 파일 launch 타입과 helper 추가

`rhwp-studio/src/command/file-system-access.ts` 또는 별도 작은 모듈:

- `isSupportedDocumentFileName(name: string): boolean`
- `readFileFromHandle(handle)` 재사용
- launch 파일 수신용 타입 최소 정의

TypeScript DOM lib에 `launchQueue` 타입이 없을 수 있으므로 앱 내부에서만 쓰는
좁은 interface를 둔다.

예상 타입:

```ts
interface LaunchQueueLike {
  setConsumer(consumer: (params: { files?: FileSystemFileHandleLike[] }) => void): void;
}

interface FileHandlingWindowLike {
  launchQueue?: LaunchQueueLike;
}
```

### Stage 3: launchQueue consumer 연결

`rhwp-studio/src/main.ts`:

- `initialize()` 이후에도 launch queue가 누락되지 않도록 앱 초기화 초기에 consumer를
  설치한다.
- consumer 내부에서는 비동기 함수를 호출해 각 파일 핸들을 처리한다.
- 현재 앱은 한 번에 하나의 주 문서를 다루므로 우선 첫 파일만 열고, 다중 파일은
  사용자 알림 또는 console warning으로 처리한다.
- 처리 흐름:
  1. 파일 핸들 존재 확인
  2. 파일명 `.hwp/.hwpx` 검사
  3. `readFileFromHandle(handle)`
  4. `eventBus.emit('open-document-bytes', { bytes, fileName, fileHandle, skipUnsavedGuard: false })`

주의:

- 로드 성공 전 임의로 `wasm.currentFileHandle`을 설정하지 않는다.
- `open-document-bytes` 기존 catch가 `showLoadError()`를 호출하므로 오류 UI는 재사용한다.
- 기존 파일 input과 drag/drop 동작은 변경하지 않는다.

### Stage 4: 오류 처리와 상태 초기화 검증

다음 케이스를 가드한다.

- `.txt` 등 미지원 확장자는 WASM 로드 전에 기존 사용자 알림 방식으로 거부한다.
- HWP/HWPX 확장자지만 실제 내용이 잘못된 파일은 기존 로드 실패 토스트를 보여준다.
- 실패 후 `currentFileHandle`이 남지 않아 다음 정상 문서 열기/저장이 오염되지 않는다.
- 기존 한컴 설치 사용자의 기본 연결 변경을 앱 코드에서 유도하지 않는다.

필요하면 `open-document-bytes` payload에 `requestId`를 붙여 테스트에서 완료를 기다리는
방식을 사용한다. 이미 비교 UI가 같은 이벤트 완료 응답을 쓰고 있으므로 새 패턴을 만들지
않는다.

### Stage 5: 테스트

자동 테스트 후보:

- `rhwp-studio/tests/file-system-access.test.ts`
  - `.hwp/.hwpx` 파일명 허용
  - 기타 확장자 거부
  - launch handle에서 파일명과 bytes 읽기
- 신규 `pwa-file-handling` 테스트
  - `launchQueue.setConsumer()` 등록 여부
  - files가 비어 있으면 no-op
  - 미지원 확장자는 로드 이벤트를 내지 않음
  - 지원 확장자는 `open-document-bytes` payload를 생성

빌드 검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node --experimental-strip-types --test tests/*.test.ts
```

권장 검증:

```bash
docker compose --env-file .env.docker run --rm wasm
```

### Stage 6: 수동 OS 파일 연결 검증

수동 검증 항목:

- Chrome/Edge에서 설치된 PWA만 파일 핸들러 후보로 등록되는지 확인
- 한컴오피스/한컴뷰어가 설치된 macOS/Windows에서 기존 기본 연결이 유지되는지 확인
- 한컴 미설치 macOS/Windows에서 rhwp-studio 후보/기본 승격 동작 확인
- `.hwp`와 `.hwpx` 각각 더블클릭 또는 "다음으로 열기"로 정상 로드 확인
- 권한 거부 시 앱이 안전하게 중단되는지 확인

## 3. 리스크와 완화

| 리스크 | 완화 |
|---|---|
| 기존 한컴 기본 앱 연결을 방해 | 앱에서 기본 앱 설정 유도 금지, 한컴 설치 환경 수동 검증 필수 |
| 브라우저/OS별 지원 차이 | 설치된 데스크톱 Chromium 계열 중심으로 범위 명시 |
| 넓은 파일 연결 등록 | `.hwp/.hwpx` 전용 MIME과 확장자만 선언 |
| 악성/오인 파일 로드 | 확장자 사전 검사 + 기존 WASM 파서 오류 UI 재사용 |
| 실패한 파일 핸들이 저장 경로에 남음 | 로드 성공 후에만 `currentFileHandle` 설정하는 기존 이벤트 흐름 사용 |
| 자동 덮어쓰기 우려 | 저장은 기존 사용자 명시 명령에서만 수행 |

## 4. 승인 후 산출물

- PWA manifest 파일 핸들러 선언
- launchQueue consumer와 테스트
- 오늘할일 갱신
- Stage 보고서 `mydocs/working/task_m100_1341_stage1.md`
- 최종 보고서 `mydocs/report/task_m100_1341_report.md`

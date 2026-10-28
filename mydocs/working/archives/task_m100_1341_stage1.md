# Stage 1 완료 보고서 — Task M100-1341

- 이슈: https://github.com/edwardkim/rhwp/issues/1341
- 브랜치: `local/task_m100_1341`
- 작성일: 2026-06-09

## 1. 완료 범위

rhwp-studio PWA가 OS 파일 연결 후보로 등록될 수 있도록 manifest와 런타임 launch
처리 경로를 추가했다.

변경 파일:

| 파일 | 내용 |
|---|---|
| `rhwp-studio/vite.config.ts` | PWA manifest `file_handlers` 추가 |
| `rhwp-studio/src/command/file-system-access.ts` | HWP/HWPX accept 상수, 파일명 지원 여부 helper 추가 |
| `rhwp-studio/src/command/pwa-file-handling.ts` | `launchQueue` 파일 핸들 처리 유틸 추가 |
| `rhwp-studio/src/main.ts` | launch 파일을 기존 `open-document-bytes` 이벤트로 연결 |
| `rhwp-studio/tests/file-system-access.test.ts` | 확장자/MIME 가드 테스트 추가 |
| `rhwp-studio/tests/pwa-file-handling.test.ts` | launchQueue 처리 단위 테스트 추가 |

## 2. 구현 상세

### 2.1 Manifest 선언

`file_handlers`는 `.hwp`, `.hwpx`만 선언했다.

```json
{
  "action": "/rhwp/",
  "accept": {
    "application/x-hwp": [".hwp"],
    "application/hwp+zip": [".hwpx"]
  }
}
```

보안상 `application/octet-stream`, `*/*`, `application/*` 같은 넓은 MIME은
등록하지 않았다.

### 2.2 Runtime launch 처리

`installPwaFileHandling()`은 `window.launchQueue`가 있는 환경에서만 consumer를
등록한다. 브라우저가 API를 지원하지 않으면 false를 반환하고 기존 파일 열기 흐름은
그대로 유지된다.

launch 파일 처리 흐름:

1. launch 파일 핸들 목록 확인
2. 다중 파일이면 첫 파일만 처리하고 console warning 기록
3. 파일명 `.hwp/.hwpx` 검사
4. `FileSystemFileHandle.getFile()`로 bytes 읽기
5. 기존 `open-document-bytes` 이벤트 emit

`open-document-bytes`는 기존 코드가 이미 다음을 담당한다.

- 미저장 문서 교체 확인
- WASM 문서 로드
- 로드 성공 후 `currentFileHandle` 설정
- 로드 실패 시 `showLoadError()` 토스트 표시

따라서 이번 구현은 파일 핸들을 로드 성공 전에 저장 경로에 직접 주입하지 않는다.

## 3. 보안/UX 가드

- 설치된 PWA에서만 OS 파일 연결 기능이 의미를 가진다.
- `.hwp/.hwpx` 외 확장자는 WASM 로드 전에 거부한다.
- 잘못된 파일 내용은 기존 WASM 로드 오류 UI로 처리한다.
- 파일 핸들은 IndexedDB/localStorage/Service Worker cache에 저장하지 않는다.
- 자동 저장/자동 덮어쓰기는 추가하지 않았다.
- 앱 내부에서 "기본 앱으로 설정"을 유도하지 않는다.

## 4. 검증 결과

### 4.1 rhwp-studio 단위 테스트

```bash
cd rhwp-studio && node --experimental-strip-types --test tests/*.test.ts
```

결과:

- 63 passed
- 0 failed

신규 테스트 주요 항목:

- HWP/HWPX 확장자만 허용
- 넓은 binary MIME 미등록
- `launchQueue.setConsumer()` 등록
- 빈 launch no-op
- 미지원 확장자는 로드 이벤트 미발행
- HWP/HWPX launch handle은 `open-document-bytes` payload 생성
- 파일 읽기 실패는 `notifyError`로 전달

### 4.2 rhwp-studio production build

```bash
cd rhwp-studio && npm run build
```

결과: 통과.

기존 Vite 경고:

- `canvaskit-wasm`의 `fs` / `path` browser externalized warning
- 500 kB 초과 chunk warning

이번 변경과 무관한 기존 경고로 판단한다.

### 4.3 Manifest 산출물 확인

`rhwp-studio/dist/manifest.webmanifest`에 다음 항목이 생성됨을 확인했다.

```json
"file_handlers":[{"action":"/rhwp/","accept":{"application/x-hwp":[".hwp"],"application/hwp+zip":[".hwpx"]}}]
```

### 4.4 Whitespace 검사

```bash
git diff --check
```

결과: 통과.

## 5. 남은 수동 판정

PWA OS 파일 연결은 실제 설치된 PWA와 OS 파일 연결 정책이 관여하므로 자동 테스트만으로
완료 판정할 수 없다.

작업지시자 판정 항목:

- Chrome/Edge에서 rhwp-studio PWA 설치 후 `.hwp` 파일이 "다음으로 열기" 후보로
  rhwp-studio를 표시하는지 확인
- `.hwpx`도 동일하게 확인
- 파일 열기 권한 허용 후 기존 문서 로딩 화면으로 정상 진입하는지 확인
- 한컴오피스/한컴뷰어가 설치된 환경에서 기존 기본 파일 연결이 임의 변경되지 않는지 확인
- 권한 거부 또는 미지원 확장자 파일에서 안전하게 오류 UI로 종료되는지 확인

결과:

- 작업지시자 동작 판정 통과.

## 6. 다음 단계

최종 보고서를 작성하고 커밋/후속 git workflow로 진행한다.

# 최종 보고서 — Task M100-1341

- 이슈: https://github.com/edwardkim/rhwp/issues/1341
- 제목: PWA 가 OS 파일연결을 지원하게 해주십시오
- 브랜치: `local/task_m100_1341`
- 작성일: 2026-06-09

## 1. 결과 요약

rhwp-studio PWA가 OS 파일 연결 후보로 등록될 수 있도록 Web App Manifest
`file_handlers`와 runtime `launchQueue` 처리 경로를 추가했다.

사용자는 설치된 PWA 환경에서 `.hwp` / `.hwpx` 파일을 OS "다음으로 열기" 또는
더블클릭 흐름으로 rhwp-studio에 전달할 수 있다. 전달된 파일은 기존
`open-document-bytes` 로딩 흐름을 재사용하므로 미저장 문서 확인, 로드 실패 토스트,
로드 성공 후 파일 핸들 저장 동작이 기존 파일 열기와 같은 정책을 따른다.

작업지시자 동작 판정은 통과했다.

## 2. 변경 내역

| 파일 | 변경 |
|---|---|
| `rhwp-studio/vite.config.ts` | PWA manifest `file_handlers` 추가 |
| `rhwp-studio/src/command/file-system-access.ts` | HWP/HWPX accept 상수와 파일명 helper 추가 |
| `rhwp-studio/src/command/pwa-file-handling.ts` | PWA launch 파일 처리 유틸 신규 추가 |
| `rhwp-studio/src/main.ts` | `window.launchQueue` consumer를 기존 문서 로딩 이벤트로 연결 |
| `rhwp-studio/tests/file-system-access.test.ts` | 확장자/MIME 보안 가드 테스트 추가 |
| `rhwp-studio/tests/pwa-file-handling.test.ts` | PWA launch 파일 처리 단위 테스트 추가 |
| `mydocs/plans/task_m100_1341.md` | 수행 계획서 |
| `mydocs/plans/task_m100_1341_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_1341_stage1.md` | Stage 1 보고서 |

## 3. 보안/UX 조치

- `.hwp`, `.hwpx`만 파일 핸들러 대상으로 등록했다.
- `application/octet-stream`, `*/*`, `application/*` 같은 넓은 MIME은 등록하지 않았다.
- OS launch 파일도 파일명 확장자 검사를 통과해야 한다.
- 로드 성공 전에는 `wasm.currentFileHandle`을 직접 설정하지 않는다.
- 파일 핸들은 IndexedDB/localStorage/Service Worker cache에 저장하지 않는다.
- 자동 저장/자동 덮어쓰기는 추가하지 않았다.
- 기존 한컴오피스/한컴뷰어 사용자의 기본 앱 변경을 유도하는 UI는 추가하지 않았다.

## 4. 검증

### 4.1 단위 테스트

```bash
cd rhwp-studio && node --experimental-strip-types --test tests/*.test.ts
```

결과:

- 63 passed
- 0 failed

### 4.2 Production build

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

### 4.5 작업지시자 동작 판정

- 동작 판정 통과.

## 5. 남은 사항

브라우저/OS별 PWA File Handling API 지원 범위는 다르다. 따라서 배포 후 실제 사용자
환경에서는 설치된 데스크톱 Chromium 계열 PWA 중심으로 동작한다는 점을 유지한다.

Safari/Firefox, 미설치 일반 브라우저 탭, 네이티브 OS registry 직접 제어는 이번
범위가 아니다.

## 6. 후속 절차 결과

- task 브랜치 커밋: `ac31cb99`
- `local/devel` no-ff 병합: `5b9e08cc`
- `devel` no-ff 병합 및 push: `91531bb7`
- GitHub Issue #1341 close: completed, closedAt `2026-06-09T03:18:48Z`

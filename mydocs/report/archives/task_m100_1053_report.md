# Task M100-1053 최종 보고서

## 1. 요약

#1053에서 미지원 파일(HWPML 2.1 등)을 열 때 저수준 CFB 오류 대신 안정적인
오류코드와 사용자용 안내가 반환되도록 정정했다.

또한 rhwp-studio에서 미지원 문서 로딩 실패 후 내부 WASM 문서 상태를 초기화하여
다음 정상 문서를 다시 열 수 있도록 했다. 사용자 알림은 기존 `showLoadError()`
흐름인 상태 표시줄 + 토스트 UI를 그대로 사용한다.

## 2. 주요 변경

| 영역 | 결과 |
|------|------|
| 포맷 감지 | `FileFormat::LegacyHwpml` 추가 |
| HWPML 오류 | `UNSUPPORTED_HWPML` + `HWPML 2.1` 표시 |
| Unknown 오류 | `UNSUPPORTED_FILE_FORMAT` 반환, CFB 오류 누출 방지 |
| Studio 로드 실패 | `WasmBridge.loadDocument()` 실패 시 `doc`/파일 핸들/파일명 초기화 |
| E2E | 미지원 HWPML 오류 알림 후 정상 HWP 로드 회귀 가드 추가 |

## 3. 사용자 표시 예

```text
파일 로드 실패: 유효하지 않은 파일: 지원하지 않는 포맷입니다: HWPML 2.1. 오류코드: UNSUPPORTED_HWPML. 현재 rhwp는 HWP 5.0, HWPX, 일부 HWP 3.0 문서만 지원합니다. 한컴오피스에서 HWP 5.0 또는 HWPX로 다시 저장한 뒤 열어주세요.
```

## 4. 검증 결과

- `cargo fmt` 통과
- `cargo test --release --lib` — 1335 passed, 0 failed, 6 ignored
- `cd rhwp-studio && node --experimental-strip-types --test tests/*.test.ts` — 26/26 통과
- `docker compose --env-file .env.docker run --rm wasm` 통과
- `cd rhwp-studio && npm run build` 통과
- `unsupported-format-error.test.mjs --mode=headless` 통과

E2E는 다음을 확인했다.

- 기존 파일 입력 흐름으로 미지원 HWPML 입력
- 상태 표시줄에 `UNSUPPORTED_HWPML`과 `HWPML 2.1` 표시
- 토스트에 동일 오류 표시
- 로드 실패 후 `WasmBridge.hasLoadedDocument() === false`
- 이후 정상 `field-01.hwp` 로드 성공

## 5. 작업지시자 시각 판정

작업지시자 시각 판정 통과.

## 6. Git 반영

- 작업 브랜치 커밋: `729c98f7`
- `local/devel` 병합 커밋: `8ed437c4`
- `devel` 병합 커밋: `3b820806`
- `origin/devel` push 완료
- GitHub Issue #1053 close 완료: `2026-05-22T06:37:28Z`

## 7. 후속 후보

- 실제 HWPML 2.1 fixture 확보 시 실파일 회귀 테스트 추가
- JS/WASM 경계의 구조화 오류 객체 API는 별도 task로 분리 가능

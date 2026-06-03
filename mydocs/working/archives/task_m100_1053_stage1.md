# Task M100-1053 Stage 1 완료 보고서

## 1. 범위

이슈 #1053 "미지원 파일(HWPML 2.1 등)에 대해 적절한 오류코드 반환"의
Stage 1 구현을 완료했다.

이번 stage는 legacy HWPML 파서 구현이 아니라, 지원하지 않는 입력을 명확히
식별하고 기존 사용자 알림 UI로 안내한 뒤 다음 정상 문서 로드가 가능하도록
상태를 정리하는 작업이다.

## 2. 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/parser/mod.rs` | `FileFormat::LegacyHwpml` 추가, HWPML 2.1 감지, `UNSUPPORTED_HWPML`/`UNSUPPORTED_FILE_FORMAT` 오류코드 반환 |
| `src/error.rs` | `UnsupportedFormat` 오류코드가 `HwpError` 표시 문자열까지 전달되는지 테스트 보강 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `loadDocument()` 실패 시 `doc`, 파일 핸들, 파일명을 안전 초기화 |
| `rhwp-studio/e2e/unsupported-format-error.test.mjs` | 미지원 HWPML 오류 알림 후 정상 HWP 재로드 E2E 추가 |
| `mydocs/plans/task_m100_1053.md` | 수행 계획서 |
| `mydocs/plans/task_m100_1053_impl.md` | 구현 계획서 |

## 3. 구현 상세

### 3.1 parser 오류코드

`detect_format()`이 raw XML HWPML을 감지하면 `FileFormat::LegacyHwpml`을
반환한다. HWPML 2.1 입력은 다음 오류로 차단된다.

```text
지원하지 않는 포맷입니다: HWPML 2.1. 오류코드: UNSUPPORTED_HWPML. 현재 rhwp는 HWP 5.0, HWPX, 일부 HWP 3.0 문서만 지원합니다. 한컴오피스에서 HWP 5.0 또는 HWPX로 다시 저장한 뒤 열어주세요.
```

magic/signature로 식별할 수 없는 일반 unknown 입력은 HWP 파서로 보내지 않고
`UNSUPPORTED_FILE_FORMAT`으로 반환한다. 손상된 HWP 5.0 CFB 파일은 여전히 HWP
파서 내부 오류로 남겨 지원 포맷 내부 오류와 미지원 포맷을 구분했다.

### 3.2 rhwp-studio 로드 실패 초기화

`WasmBridge.loadDocument()`는 기존 문서를 해제한 뒤 새 `HwpDocument(data)` 생성
또는 `convertToEditable()` 과정에서 실패할 수 있다. 실패 시 내부 문서 참조가
남지 않도록 다음을 보장했다.

- 기존 문서는 `releaseDocument()`로 먼저 해제
- 새 문서는 성공적으로 초기화되는 동안에만 `this.doc`에 연결
- 실패 시 생성 중인 문서 `free()`
- `this.doc = null`
- `_currentFileHandle = null`
- `_fileName = 'document.hwp'`
- 오류는 다시 throw하여 기존 `showLoadError()` 흐름이 처리

따라서 사용자 알림 UI는 기존 상태 표시줄 + 토스트 방식을 그대로 사용한다.

## 4. 검증

| 검증 | 결과 |
|------|------|
| `cargo fmt` | 통과 |
| `cargo test --release --lib` | 1335 passed, 0 failed, 6 ignored |
| `cd rhwp-studio && node --experimental-strip-types --test tests/*.test.ts` | 26/26 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과, `pkg/` WASM 재빌드 |
| `cd rhwp-studio && npm run build` | 통과 |
| `CHROME_PATH=/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome node e2e/unsupported-format-error.test.mjs --mode=headless` | 통과 |

참고: E2E 최초 실행 시 Rust WASM이 갱신되지 않아 기존 CFB 오류가 표시됐다.
WASM 재빌드 후 동일 E2E가 `UNSUPPORTED_HWPML` 표시와 정상 문서 재로드를 모두
통과했다.

## 5. 작업지시자 시각 판정

작업지시자 시각 판정 통과.

## 6. 남은 사항

- 실제 HWPML 2.1 파일 fixture가 제공되면 합성 XML 외 실파일 회귀 가드를 추가할 수 있다.
- 구조화된 JS 오류 객체 API는 이번 범위 밖이다. 현재 contract는 안정 오류코드 문자열 표시다.

---
타스크: #265 HWP 3.0 파일 감지 + 친절한 에러 메시지
단계: Stage 3 — WASM 재빌드 + 프론트엔드 확인
브랜치: local/task265
작성일: 2026-04-24
---

# Stage 3 완료 보고서

## 1. 목적

Stage 1~2 에서 수정된 Rust 코드가 WASM 경계를 통해 프론트엔드 (rhwp-studio) 에 사용자 친화적 에러 메시지로 전달되는지 end-to-end 확인.

## 2. WASM 재빌드

```
docker compose --env-file .env.docker run --rm wasm
```

| 항목 | Before | After | 증가 |
|---|---|---|---|
| `pkg/rhwp_bg.wasm` | 4,076,166 B | **4,081,046 B** | +4,880 B |

증가분은 `Hwp3` variant · `UnsupportedFormat` variant · 한국어 힌트 문자열 · 관련 테스트 제외 (release 빌드) · `From` impl Display 전환 반영분.

## 3. 프론트엔드 검증 (rhwp-studio 실측)

### 3.1 환경

- `rhwp-studio` 로컬 vite dev 서버
- 대상 파일: `samples/issue_265.hwp` (이슈 제보자 파일)
- 브라우저: Chrome

### 3.2 실측 로그

```
client:827 [vite] connecting...
font-loader.ts:139 [FontLoader] OS 폰트 감지: 맑은 고딕, ...
font-loader.ts:238 [FontLoader] 폰트 로드 완료: 2개 성공, 0개 실패
wasm-bridge.ts:58 [WasmBridge] WASM 초기화 완료 (rhwp 0.7.3)
client:931 [vite] connected.
main.ts:465 [main] 파일 로드 실패: 유효하지 않은 파일: 지원하지 않는 포맷입니다:
  HWP 3.0. 현재 rhwp 는 HWP 5.0 과 HWPX 만 지원합니다. 한컴오피스 또는 LibreOffice
  에서 파일을 연 뒤 HWP 5.0 포맷으로 다시 저장하여 시도해주세요.
```

### 3.3 Before / After 대조

**Before (Stage 2 수정 이전 로그)**:
```
[main] 파일 로드 실패: 유효하지 않은 파일: CFB 오류: CFB 열기 실패:
Invalid CFB file (wrong magic number): [48, 57, 50, 20, 44, 6f, 63, 75]
```

**After**:
```
[main] 파일 로드 실패: 유효하지 않은 파일: 지원하지 않는 포맷입니다: HWP 3.0.
현재 rhwp 는 HWP 5.0 과 HWPX 만 지원합니다. 한컴오피스 또는 LibreOffice 에서
파일을 연 뒤 HWP 5.0 포맷으로 다시 저장하여 시도해주세요.
```

### 3.4 확인 항목

- [x] 포맷 감지: `HWP 3.0` 명시
- [x] 해결 방법 안내: 한컴오피스 / LibreOffice 언급
- [x] 내부 구현 세부 (CFB 매직 바이트 등) 누출 0
- [x] `rhwp-studio/src/main.ts:463` 의 `파일 로드 실패: ${error}` interpolation 정상 작동
- [x] WASM 0.7.3 초기화 성공

### 3.5 Chrome ext / github.io 영향

- `rhwp-chrome`: `build.mjs` 가 rhwp-studio 빌드 결과를 그대로 포함. 다음 확장 배포 시 자동 반영
- `rhwp-firefox`: 동일 구조
- `github.io`: rhwp-studio 의 배포 빌드. CI 반영 후 자동 적용

별도 프론트엔드 수정 **불필요** (구현계획서 §1.1 사전 조사대로).

## 4. 다음 단계

Stage 4 — 회귀 검증. 기존 HWP 5.0 · HWPX 샘플이 정상 로드되는지 + `cargo test --lib` / svg_snapshot / clippy 모두 그린 확인.

## 5. 산출물

- `pkg/rhwp_bg.wasm` 재빌드 (+4.88 KB)
- 본 문서 (`mydocs/working/task_m100_265_stage3.md`)

Stage 3 완료.

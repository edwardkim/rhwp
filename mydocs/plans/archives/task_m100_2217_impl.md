# 구현계획서 - Task M100 #2217: 로컬 글꼴 레코드와 CanvasKit 연결

## 변경 계약

### 로컬 글꼴 레코드

`LocalFontSnapshot`은 v2에서 글꼴 family 문자열 목록 대신 다음 메타데이터를 보존한다.

- UI와 CSS에 사용할 canonical family
- family, full name, PostScript name, family + style 및 OpenType name table에서 찾은 지역화 이름 별칭
- 감지 출처와 시각

저장소에는 이름 메타데이터만 기록한다. `FontData.blob()`의 원본 바이트는 현재 문서 렌더링 세션에만
연결한다.

### 이름 해석

공통 resolver는 Unicode NFC, 공백 정리, 대소문자 무시 비교를 수행하되, 서로 다른 글꼴을 넓게 추측해
합치지 않는다. HWP 원본 글꼴명으로 local record를 찾으면 CSS에는 record의 canonical family를 전달한다.

`08서울한강체 M` / `08SeoulHangang M` / `SeoulHangangM`은 같은 M face record를 반환해야 한다.
`08SeoulHangang`처럼 style이 빠진 family는 설치된 face가 하나일 때만 해석하고, L/M처럼 여러 face가
있으면 임의로 하나를 고르지 않는다.

### CanvasKit

CanvasKit renderer는 기본 Noto fallback을 유지한다. local record에 현재 세션의 SFNT 바이트가 있으면
문서 text run의 해석 결과에 맞는 Typeface와 FontMgr를 선택한다. 등록 실패는 렌더 전체 실패가 아니라
기존 fallback으로 처리하고 진단으로 남긴다.

## 변경 후보

| 파일 | 변경 |
|---|---|
| `rhwp-studio/src/core/local-fonts.ts` | snapshot v2, 별칭 record, FontData blob 타입과 메모리 registry |
| `rhwp-studio/src/core/document-font-status.ts` | resolver 기반 local availability 판정 |
| `rhwp-studio/src/core/font-substitution.ts` | canonical CSS family를 선행하는 display chain |
| `rhwp-studio/src/ui/toolbar.ts` | display name과 canonical family를 구분한 로컬 목록 |
| `rhwp-studio/src/view/canvaskit-renderer.ts` | per-family Typeface/FontMgr 선택 |
| `rhwp-studio/tests/*.test.ts` | v1 migration, 다국어 별칭, CSS chain, CanvasKit registry 회귀 |

## 검증

- Stage 1: local-fonts, document-font-status, font-substitution focused tests
- Stage 2: CanvasKit renderer contract 및 font coverage focused tests
- Stage 3: `wasm-pack build --target web --out-dir pkg`, Studio build, `20200830.hwp` 브라우저 확인

전체 CI 성격 cargo test/clippy는 PR 직전 별도 승인을 받은 뒤 수행한다.

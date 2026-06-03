# Task #67 — 3단계 완료보고서

## 프런트엔드 폰트 로더 수정 ✅

### 수정 파일

- `rhwp-studio/src/core/font-substitution.ts` — `fontFamilyWithFallback()` 전 플랫폼 체인
- `web/editor.html` — 폰트 목록 동기화 (font-loader.ts와 동일)

### 변경 내용

#### font-substitution.ts
- `fontFamilyWithFallback()`에 Monospace 판별 추가
- Serif/Sans-serif/Monospace 각 카테고리에 전 플랫폼 CSS 체인 적용
  - Serif: `바탕 → AppleMyungjo → Noto Serif KR → serif`
  - Sans: `Malgun Gothic → Apple SD Gothic Neo → Noto Sans KR → Pretendard → sans-serif`
  - Mono: `GulimChe → D2Coding → Noto Sans Mono → monospace`

#### web/editor.html
- 폰트 목록을 font-loader.ts와 완전 동기화
- 함초롬체 CDN 참조, HY/MS 폰트 → 오픈소스 대체
- `format` 필드 지원 (woff/woff2 구분)
- @font-face 및 FontFace API에서 format 반영

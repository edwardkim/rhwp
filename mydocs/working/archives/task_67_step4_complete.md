# Task #67 — 4단계 완료보고서

## Canvas 폰트 감지 + OS 폰트 우선 ✅

### 수정 파일

- `rhwp-studio/src/core/font-loader.ts` — OS 폰트 감지 + 로딩 최적화

### 변경 내용

1. **`detectOSFonts()`**: @font-face 등록 전에 `document.fonts.check()` 기반으로 OS 설치 폰트 감지
   - Windows: 맑은 고딕, 바탕, 돋움, 굴림, 굴림체, 바탕체, 궁서
   - macOS/iOS: Apple SD Gothic Neo, AppleMyungjo, AppleGothic
   - Android: Noto Sans KR, Noto Serif KR

2. **로딩 최적화**: OS에 설치된 폰트는 웹폰트 로딩 건너뜀
   - 예: Windows에서 "맑은 고딕"이 감지되면 Pretendard woff2 로딩 불필요
   - 네트워크 요청 절감 + 초기 로딩 시간 단축

3. **`getDetectedOSFonts()`**: 외부 모듈에서 감지 결과 참조 가능

### 검증

- TypeScript 타입 체크: 에러 없음

# Task #88 — 완료보고서

## macOS Safari 확장 HWP 썸네일 기능 추가 ✅

### 수정 파일

- `rhwp-safari/src/background.js` — 썸네일 추출 코드 + extract-thumbnail 핸들러
- `rhwp-safari/src/content-script.js` — 호버 카드 썸네일 연동

### 구현 내용

1. **background.js**: Chrome의 `thumbnail-extractor.js`를 인라인 통합
   - HWP(CFB): FAT 체인 파싱 → PrvImage 스트림 추출
   - HWPX(ZIP): Central Directory → Preview/PrvImage → DecompressionStream
   - URL 기반 LRU 캐시 (100건)
   - `extract-thumbnail` 메시지 핸들러 (sender 검증 포함)

2. **content-script.js**: 호버 카드 썸네일 비동기 로드
   - `data-hwp-thumbnail` 속성 우선
   - 없으면 background에 `extract-thumbnail` 요청
   - URL 기반 캐시 (중복 요청 방지)
   - `insertThumbnail()`: DOM API로 안전하게 이미지 삽입

### 검증 결과

- macOS Safari: 호버 카드 썸네일 표시 ✅
- 기존 기능 회귀 없음 ✅

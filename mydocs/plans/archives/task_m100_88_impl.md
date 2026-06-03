# Task #88: macOS Safari 확장 HWP 썸네일 — 구현 계획서 (사후 작성)

## Chrome 확장 구현 분석 (Task #86)

### 파일 구조
- `sw/thumbnail-extractor.js`: 순수 JS로 HWP/HWPX에서 PrvImage 추출 (WASM 불필요)
- `sw/message-router.js`: `extract-thumbnail` 메시지 핸들러
- `content-script.js`: 호버 카드 썸네일 표시 + background 추출 요청 + 캐시

### 기술 상세
- HWP(CFB): 헤더 → 디렉토리 엔트리에서 "PrvImage" 검색 → FAT 체인 → 스트림 데이터
- HWPX(ZIP): End of Central Directory → Central Directory → Preview/PrvImage.* → DecompressionStream(deflate)
- 이미지: PNG/BMP/GIF 매직 넘버 감지 → base64 dataUri
- 캐시: URL 기반 LRU 100건

### Safari 호환성
- DecompressionStream: Safari 16.4+ 지원
- 순수 JS 구현이므로 브라우저 API 차이 없음
- Safari background는 ES module 아닌 단일 파일 → 인라인 통합

---

## 구현 단계 (3단계)

### 1단계: background.js에 썸네일 추출 통합

**대상 파일:** `rhwp-safari/src/background.js`

**작업 내용:**
1. Chrome `thumbnail-extractor.js`의 함수를 Safari background.js에 인라인 포팅
   - `extractThumbnailFromUrl(url)` — URL fetch + 포맷 감지 + 추출
   - `extractPrvImageFromCFB(data)` — HWP(OLE2) 파싱
   - `extractPrvImageFromZip(data)` — HWPX(ZIP) 파싱 + DecompressionStream
   - `_parseImage(data)` — PNG/BMP/GIF 감지 + base64 변환
   - 바이너리 헬퍼: `_u16()`, `_u32()`
2. `extract-thumbnail` 메시지 핸들러 추가
3. sender 검증 (content script만 허용)
4. URL 검증 (기존 validateUrl 재사용)

---

### 2단계: content-script.js 썸네일 연동

**대상 파일:** `rhwp-safari/src/content-script.js`

**작업 내용:**
1. `thumbnailCache` (Map) 추가 — URL → 썸네일 데이터 캐시
2. `insertThumbnail(container, src)` — DOM API로 안전하게 이미지 삽입
3. `showHoverCard()` 수정:
   - `data-hwp-thumbnail` 속성 우선 사용
   - 없으면 캐시 확인
   - 캐시 없으면 `extract-thumbnail` 메시지로 background에 비동기 요청
   - 응답 받으면 카드에 썸네일 삽입 + 캐시 저장
4. title 없는 기본 카드에서도 썸네일 표시

---

### 3단계: 검증

- macOS Safari에서 HWP 링크 호버 → 썸네일 표시 확인
- HWP(CFB) 파일 썸네일 추출 확인
- HWPX(ZIP) 파일 썸네일 추출 확인
- 썸네일 없는 파일 → 카드 정상 표시 (썸네일 영역 숨김)
- 기존 기능 회귀 테스트

---

## 리스크

| 리스크 | 대응 |
|--------|------|
| DecompressionStream 미지원 (Safari 16.4 미만) | try-catch로 graceful 실패, 썸네일 없이 카드 표시 |
| 대용량 HWP fetch 시 지연 | 비동기 처리, 카드 먼저 표시 후 썸네일 후속 삽입 |
| CORS 제한으로 fetch 실패 | background에서 fetch (확장 권한), 실패 시 캐시에 null 저장 |

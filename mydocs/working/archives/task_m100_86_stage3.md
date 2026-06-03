# Task #86 단계3 완료 보고서: Chrome 확장 썸네일 자동 추출 연동

## 수행 내용

Chrome 확장에서 HWP 링크 호버 시 썸네일을 자동 추출하여 미리보기를 표시한다.

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `rhwp-chrome/sw/thumbnail-extractor.js` | 신규 — CFB PrvImage 경량 JS 파서 + 캐싱 |
| `rhwp-chrome/sw/message-router.js` | `extract-thumbnail` 메시지 핸들러 추가 |
| `rhwp-chrome/content-script.js` | 호버 카드에 자동 썸네일 추출 로직 |

## 아키텍처

```
사용자가 HWP 링크에 마우스 호버
  ↓
content-script: data-hwp-thumbnail 있음? → 해당 이미지 표시
  ↓ 없음
content-script → Service Worker: extract-thumbnail 메시지
  ↓
Service Worker: fetch(url) → CFB 파싱 → PrvImage 추출
  ↓
Service Worker → content-script: dataUri 반환
  ↓
content-script: 로딩 스피너 → 썸네일 이미지 교체
```

## 설계 결정

- **WASM 미사용**: Service Worker에서 WASM 로딩 없이 CFB 바이너리를 직접 JS로 파싱
- **FAT 체인 추적**: CFB 디렉토리 엔트리에서 "PrvImage" 이름 탐색 → FAT 체인으로 스트림 데이터 읽기
- **캐싱**: URL 기반 LRU 캐시 (최대 100개), 동일 URL 재요청 방지
- **호버 카드 개선**: data-hwp-title 없는 링크도 파일명으로 호버 카드 표시

## 웹 관리자 사용 시나리오

```html
<!-- 최소: 확장이 자동 추출 -->
<a href="doc.hwp" data-hwp="true">문서.hwp</a>

<!-- 사전 지정: 더 빠름 -->
<a href="doc.hwp" data-hwp="true" data-hwp-thumbnail="/thumbs/doc.webp">문서.hwp</a>
```

## 커밋

`220adb3` Task #86 단계3: Chrome 확장 썸네일 자동 추출 연동

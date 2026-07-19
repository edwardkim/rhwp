---
kind: reference
status: active
canonical: mydocs/tech/browser_decompression_stream.md
last_verified: 2026-07-17
---

# 브라우저 네이티브 DecompressionStream API

## 개요

Chrome 80+, Edge 80+, Firefox 113+, Safari 16.4+에서 지원하는 Web API 표준으로, 별도 라이브러리 없이 deflate/gzip 압축을 해제할 수 있다.

## 사용법

```javascript
// import 없이 바로 사용 (전역 객체)
const ds = new DecompressionStream('raw');    // raw deflate (ZIP 방식)
const writer = ds.writable.getWriter();
writer.write(compressedData);                 // Uint8Array 입력
writer.close();

const reader = ds.readable.getReader();
const chunks = [];
while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  chunks.push(value);
}
// chunks를 합쳐서 사용
```

## 지원 포맷

| 포맷 | 설명 | 용도 |
|------|------|------|
| `'raw'` | Raw deflate (RFC 1951) | ZIP 파일 내부 엔트리 |
| `'deflate'` | zlib wrapper + deflate | HTTP Content-Encoding |
| `'gzip'` | gzip wrapper + deflate | .gz 파일, HTTP |

## 압축도 가능

```javascript
const cs = new CompressionStream('gzip');
```

## 성능

- **네이티브 C++ zlib** 구현으로 JS 라이브러리 대비 수십 배 빠름
- JS 인터프리터 오버헤드 없음, GC 부담 없음
- Service Worker에서도 동작
- Transform Stream 패턴으로 대용량 데이터 스트리밍 처리 가능

## rhwp에서의 활용

### 현재: Chrome 확장 HWPX 썸네일 추출

`rhwp-chrome/sw/thumbnail-extractor.js`에서 HWPX(ZIP) 파일의 `Preview/PrvImage.png`를 추출할 때 사용. DevTools 프로파일링 결과 deflate 구간이 거의 순식간에 처리됨.

### 향후: HWPX 브라우저 파싱

HWPX의 section XML도 ZIP deflate로 압축되어 있으므로, 브라우저에서 HWPX를 직접 파싱할 때 동일하게 활용 가능.

## 이전 방식과 비교

| 이전 (JS 라이브러리) | 현재 (네이티브 API) |
|---------------------|-------------------|
| pako.js, fflate, jszip | `DecompressionStream` |
| import/script 로딩 필요 | 전역 객체, import 불필요 |
| JS 인터프리터 실행 | 네이티브 C++ 실행 |
| 동기 처리 | 비동기 (async/await) |
| 번들 크기 증가 | 0 bytes |

## 발견일

2026-04-09, Task #86 Chrome 확장 HWPX 썸네일 추출 구현 중 발견.

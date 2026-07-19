---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# 한컴 webhwp vs rhwp: HWP 파싱 아키텍처 비교 브리핑

> 분석 대상: `/webhwp/js/hwpApp.*.chunk.js` (minified 5.17MB, webpack bundle)
> 분석 일자: 2026-02-09

## 핵심 결론

**한컴 webhwp는 클라이언트에서 HWP를 파싱하지 않는다.**

서버가 HWP 파일을 파싱하여 JSON 문서 모델로 변환한 후 클라이언트에 전달한다. 클라이언트는 이 JSON을 받아 렌더링과 편집만 수행한다.

## 1. HWP 파싱 위치 비교

```
한컴 webhwp:
┌──────────┐    documentJson    ┌──────────────────┐
│  서버      │ ──────────────→  │  브라우저 (JS)     │
│  HWP 파싱  │                  │  렌더링 + 편집만   │
│  JSON 변환 │  ←──────────────  │  OT 편집 명령 전송 │
└──────────┘    revision/OT     └──────────────────┘

rhwp:
┌────────────────────────────────────┐
│          브라우저 (WASM + JS)       │
│  HWP 파싱 + 렌더링 + 편집 + 저장    │
│  서버 불필요                        │
└────────────────────────────────────┘
```

## 2. 증거: 클라이언트에 HWP 파서가 없음

webhwp JS 번들에서 다음 키워드 검색 결과 **모두 0건**:

| 키워드 | 의미 | 검색 결과 |
|--------|------|-----------|
| `CompoundFile`, `CFB`, `CFBReader` | OLE 복합파일 파서 | 0건 |
| `BodyText`, `DocInfo`, `BinData` | HWP 스트림 이름 | 0건 |
| `HWPTAG`, `tagId`, `recordHeader` | HWP 레코드 태그 | 0건 |
| `WebAssembly`, `.wasm` | WASM 모듈 | 0건 |
| `Section0`, `Section1` | HWP 본문 스트림 | 0건 |

## 3. 서버 → 클라이언트 데이터 흐름

### 3.1 문서 로딩

```javascript
// 서버에서 문서 JSON을 받아 앱에 전달
E.loadDocument(function(t) {
    // t.documentJson.content = 서버가 HWP를 파싱한 결과
    c.loadHwpApp(t.documentJson);
});

// 앱 내부에서 JSON 데이터를 엔진에 전달
window.HwpApp.TKs(content.bi, function() {
    window.HwpApp.document.open(data, "");
    // "Hwp Document Data Load 실패 (Engine)!" 에러 메시지
});
```

### 3.2 서버 의존 기능 (RPC 호출)

| RPC 메서드 | 용도 |
|-----------|------|
| `getFontWidthFromServer(font, char, callback)` | 클라이언트에 없는 폰트의 글자 폭 서버 조회 |
| `_getData(uniqueId, fileName, type, mime)` | 서버에서 변환 데이터(PDF 등) 가져오기 |
| `printDocument(id, url, options, callback)` | 서버에서 인쇄용 변환 |
| `insertFileByFileBlob(blob, ...)` | 파일 삽입 시 서버 처리 |
| `insertFileByUrl(url, ...)` | URL 파일 삽입 시 서버 처리 |

### 3.3 협업 편집 (OT)

```javascript
// Operational Transform 기반 실시간 협업
revision: n.revision + 1,
connectOtEngine()    // OT 엔진 연결
unloadDocument()     // 문서 언로드
// OT 에러 코드: OT1(100) ~ OT8(106), OT_OFFLINE(107-110)
```

## 4. 클라이언트 역할 비교

| 기능 | 한컴 webhwp 클라이언트 | rhwp 클라이언트 |
|------|----------------------|----------------|
| **HWP 바이너리 파싱** | X (서버가 수행) | O (WASM) |
| **CFB/OLE 읽기** | X | O (Rust) |
| **문서 모델 구축** | 서버 JSON 수신 | WASM에서 직접 파싱 |
| **레이아웃/줄바꿈** | JS (클라이언트) | Rust→WASM (클라이언트) |
| **텍스트 측정** | JS Canvas `measureText()` | Rust→WASM `measureText()` 콜백 |
| **Canvas 렌더링** | JS (문자 단위 `fillText`) | JS (run 단위 `fillText`) |
| **텍스트 편집** | JS (OT 명령 → 서버 동기화) | JS (직접 Document IR 수정) |
| **HWP 저장** | 서버에서 HWP 재생성 | WASM에서 직접 직렬화 |
| **오프라인 동작** | 불가 (서버 필수) | 가능 |

## 5. 기술 스택 비교

| 항목 | 한컴 webhwp | rhwp |
|------|-----------|------|
| **HWP 파서 언어** | 서버 (언어 미상, Java 또는 C++) | Rust |
| **클라이언트 엔진** | 순수 JavaScript (5.17MB) | Rust→WASM + JavaScript |
| **번들 방식** | Webpack code-splitting (9 chunks) | wasm-pack + 수동 JS |
| **폰트 메트릭** | JS 내장 318개 폰트 정의 + 서버 폴백 | 시스템 폰트 의존 |
| **문서 모델** | 서버 JSON 기반 | Rust Document IR |
| **다국어** | 22개 언어 지원 | 한국어 중심 |
| **UI 프레임워크** | React (추정) | 순수 JS |

## 6. 폰트 처리 차이

### 한컴 webhwp
- **318개 폰트 메트릭 내장**: `{fontname, height, width, charset, iYt, nYt}` 형태로 JS에 하드코딩
- **`iYt` (advance width base)**: 폰트별 기준값 (대부분 1024, 2048)
- **서버 폴백**: 내장 메트릭에 없는 폰트는 `getFontWidthFromServer()` RPC
- **웹폰트 번들**: 12+ woff2 파일 (19.4MB) — 한글/영문/특수 폰트

### rhwp
- **시스템 폰트 의존**: OS에 설치된 폰트 사용
- **`measureText()` 직접 측정**: 1000px 고정밀 Canvas 측정
- **한글 등폭 가정**: 모든 한글 음절 = '가' 측정값 (한컴과 동일 전략)
- **웹폰트**: 7개 woff2 파일 (7.7MB)

## 7. 아키텍처 시사점

### 한컴의 선택: 서버 파싱 + 클라이언트 렌더링

**장점:**
- HWP 파싱 로직을 서버에 집중 → 기존 데스크톱 엔진 재사용 가능
- 클라이언트 코드가 HWP 포맷에 독립적 → JSON만 이해하면 됨
- OT 기반 실시간 협업 편집 가능
- 폰트 폭 서버 폴백으로 정밀도 보장

**제약:**
- 서버 없이 동작 불가 (오프라인 미지원)
- 네트워크 지연 발생 (문서 열기, 저장, 폰트 조회)
- 서버 인프라 운영 비용

### rhwp의 선택: 클라이언트 완전 자립

**장점:**
- 서버 완전 불필요 → 제로 인프라 비용
- 오프라인 동작 가능
- 네트워크 지연 없음
- HWP 바이너리 직접 제어 (파싱 + 저장)

**제약:**
- HWP 포맷 자체 구현 필요 (높은 기술 장벽)
- 시스템 폰트 의존 (폰트 없을 시 폴백 불확실)
- 협업 편집 시 별도 OT 인프라 필요

## 8. 요약

| 비교 항목 | 한컴 webhwp | rhwp |
|-----------|-----------|------|
| **HWP 파싱 위치** | 서버 | 클라이언트 (WASM) |
| **클라이언트 역할** | 렌더링 + 편집 UI | 파싱 + 렌더링 + 편집 + 저장 |
| **서버 역할** | HWP 파싱/변환/저장/폰트 | 없음 (정적 호스팅만) |
| **데이터 형식** | 서버 JSON ↔ 클라이언트 | HWP 바이너리 ↔ Document IR |
| **오프라인** | 불가 | 가능 |
| **협업 편집** | OT 기반 지원 | 미구현 |

---

*작성일: 2026-02-09*

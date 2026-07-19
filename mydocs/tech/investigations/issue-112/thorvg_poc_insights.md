---
kind: investigation
status: historical
canonical: mydocs/tech/thorvg_decision.md
last_verified: 2026-07-17
---

# ThorVG POC 인사이트 보고서

## 개요

타스크 112~115에 걸쳐 ThorVG를 HWP 웹 에디터(rhwp-studio)의 대안 렌더링 백엔드로
검증하는 POC를 수행했다. 본 보고서는 POC를 통해 얻은 기술적 인사이트를 정리한다.

### POC 진행 경과

| 타스크 | 내용 | 핵심 성과 |
|--------|------|-----------|
| 112 | Rust FFI 바인딩 + 네이티브 렌더링 | ThorVG C API → Rust FFI 30개 함수, HWP→PNG 출력 성공 |
| 113 | Emscripten WASM 빌드 + WebGL 렌더링 | Docker 빌드 파이프라인, JS 브릿지, WebGL 2.0 GPU 직접 렌더링 |
| 114 | rhwp-studio 통합 + 편집 기능 검증 | 렌더러 전환 UI, 캐럿/선택/IME 모두 GL 위에서 동작 확인 |
| 115 | TTF 폰트 메트릭 + 글자별 개별 배치 | ttf-parser 기반 글리프 측정, charPositions 배열, 장평 변환 |

---

## 1. 검증된 것: rhwp 아키텍처의 강점

### 1.1 렌더러 독립적 편집 인프라

POC 시작 전 가설:
> 편집 인프라(캐럿, 히트테스트, IME)는 WASM의 문서 모델 좌표계를 사용하므로
> 렌더링 백엔드만 교체하면 편집 기능이 그대로 동작해야 한다.

**결과: 가설 확인됨.**

| 편집 기능 | 동작 여부 | 이유 |
|-----------|-----------|------|
| 캐럿 위치 표시 | O | DOM 오버레이 (`position:absolute` div) — 렌더러와 독립 |
| 마우스 클릭 → 캐럿 이동 | O | WASM `hitTest()` API — 렌더러와 독립 |
| 텍스트 입력 (한글 IME) | O | 숨겨진 textarea — 렌더러와 독립 |
| 텍스트 선택 (드래그) | O | DOM 오버레이 — 렌더러와 독립 |
| 삭제 (Backspace/Delete) | O | WASM 문서 모델 조작 — 렌더러와 독립 |

이 결과는 rhwp의 아키텍처가 Google Docs의 "Annotated Canvas" 패턴과 동일한
구조(Canvas 렌더링 + DOM 오버레이 + 숨겨진 textarea)를 취하고 있기 때문이다.
렌더링 백엔드를 교체해도 편집 인프라가 영향받지 않는다는 것은
아키텍처 설계가 올바르게 분리되어 있음을 의미한다.

### 1.2 렌더 트리 기반 추상화

Rust 파서/레이아웃 엔진이 **렌더 트리(JSON)**를 생성하고,
이를 각 렌더링 백엔드가 소비하는 구조가 효과적으로 동작했다:

```
HWP 문서 → Rust 파서 → 레이아웃 엔진 → 렌더 트리 (JSON)
                                              ↓
                               ┌──────────────┴──────────────┐
                               ↓                             ↓
                        Canvas 2D 렌더링               ThorVG 렌더링
                        (웹 편집기용)                  (내보내기/프리뷰용)
```

렌더 트리가 중간 표현(IR)으로서 렌더러 교체를 가능하게 한다.

---

## 2. 발견된 한계: ThorVG의 웹 에디터 부적합 영역

### 2.1 편집 후 재렌더링 지연

**증상**: 한글 입력 시 화면 반영까지 체감 가능한 지연 발생

**원인**: Canvas 2D는 동기 렌더링이지만, ThorVG GL은 비동기 파이프라인을 거침

```
[Canvas 2D 편집 루프 — 동기]
키 입력 → WASM insertText → renderPageToCanvas() → 즉시 화면 반영

[ThorVG GL 편집 루프 — 비동기]
키 입력 → WASM insertText → canvasPool.releaseAll() → (빈 화면)
  → exportRenderTree() → JSON.parse()
  → setupCanvas() → ThorVG 캔버스 파괴/재생성
  → preloadFonts() → 렌더 트리 순회
  → renderNode() → 전체 트리 순회 + ThorVG 오브젝트 생성
  → canvas_update/draw/sync → GL 렌더링
  → drawImage(GL→2D) → 화면 반영
```

Canvas 2D의 `fillText()`는 브라우저 네이티브 호출 한 번이지만,
ThorVG GL은 JSON 직렬화/파싱 + WASM 호출 + GL 렌더링 + 캔버스 복사까지
전체 파이프라인을 매번 거쳐야 한다.

**구조적 한계**: 이 지연은 최적화로 줄일 수 있지만 근본적으로 제거할 수 없다.
Canvas 2D의 `fillText()`가 브라우저 내부에서 GPU 가속으로 처리되는 것에 비해,
ThorVG는 WASM ↔ JS ↔ WebGL의 다층 경계를 넘어야 하기 때문이다.

### 2.2 동시 렌더링 레이스 컨디션

**증상**: 초기 렌더링 시 1페이지가 빈 화면, 스크롤 후 복귀하면 정상 표시

**원인**: ThorVG는 단일 GL 캔버스를 공유하는데, 여러 페이지가 동시에
비동기 렌더링을 시작하면 `setupCanvas()`가 이전 페이지의 ThorVG 캔버스를 파괴

```
페이지 0: setupCanvas() → await preloadFonts() (폰트 fetch 대기)
    ↓ (yield)
페이지 1: setupCanvas() → 페이지 0의 ThorVG 캔버스 파괴!
    ↓
페이지 0: renderNode() → 이미 파괴된 캔버스에 그림 → 빈 화면
```

**해결**: `renderToCanvas()`에 직렬화 큐(Promise chain)를 추가하여 순차 실행 보장.
그러나 이 직렬화 자체가 다중 페이지 렌더링 시 추가 지연의 원인이 된다.

### 2.3 폰트 처리의 비효율성

**Canvas 2D의 폰트 처리**:
- CSS `@font-face` 선언만 하면 브라우저가 로딩/캐싱/폴백을 자동 처리
- 새 폰트를 만나도 렌더링이 블로킹되지 않음 (FOUT/FOIT 브라우저 정책)
- 시스템 폰트, 웹폰트 모두 투명하게 지원

**ThorVG의 폰트 처리**:
- TTF 바이너리를 직접 fetch → WASM 힙에 복사 → `tvg_font_load_data()` 등록
- 매 렌더링마다 렌더 트리를 순회하여 폰트명 수집 → 미등록 폰트 fetch 대기
- 폰트 로딩이 **렌더링 핫 패스**에 위치하여 블로킹 발생
- 폰트 폴백 체인을 직접 구현해야 함

| 비교 항목 | Canvas 2D | ThorVG |
|-----------|-----------|--------|
| 폰트 로딩 | 브라우저 백그라운드 | 직접 fetch + WASM 로드 |
| 새 폰트 발견 시 | 투명 처리 | 렌더링 블로킹 |
| 폰트 폴백 | 브라우저 내장 체인 | 직접 구현 |
| 메모리 관리 | 브라우저 관리 | WASM 힙에 상주 |
| 적합 시나리오 | 동적/다양한 폰트 | 고정된 소수 폰트 |

ThorVG는 Tizen TV, Lottie 플레이어처럼 **폰트가 1~2개로 고정된 임베디드 환경**에
최적화되어 있다. 워드프로세서처럼 **문서마다 다른 폰트를 실시간 바인딩**하는
시나리오는 ThorVG의 설계 목표에 포함되지 않는다.

---

## 3. Google Docs 아키텍처와의 비교

`mydocs/feedback/font-metrics.md` 문서 검토를 통해
Google Docs의 Canvas 기반 렌더링 아키텍처와 비교 분석했다.

### 3.1 레이아웃 vs 페인팅 분리

| 항목 | Google Docs | rhwp |
|------|-------------|------|
| 레이아웃 엔진 | 자체 WASM 엔진 | Rust WASM 엔진 |
| 페인팅 | Canvas 2D `fillText()` (브라우저 위임) | Canvas 2D `fillText()` (동일) |
| 렌더링 일관성 | TTF 서버 분석 + WASM 파싱 | TTF 클라이언트 파싱 (`ttf-parser`) |

Google Docs는 **"계산은 자체, 그리기는 브라우저"** 전략을 취한다.
rhwp도 동일한 전략이며, 이것이 정답임을 POC가 재확인해주었다.

### 3.2 HarfBuzz (텍스트 셰이핑)

| 항목 | Google Docs | rhwp |
|------|-------------|------|
| 셰이핑 엔진 | HarfBuzz WASM 포팅 (추정) | 미사용 |
| Canvas 2D 경로 | 브라우저 내장 HarfBuzz 활용 | 브라우저 내장 HarfBuzz 간접 활용 |
| ThorVG 경로 | N/A | TTF raw advance width (셰이핑 없음) |

한글(완성형 음절)과 라틴 알파벳은 복잡한 셰이핑이 불필요하므로
HarfBuzz 없이도 TTF advance width 기반 배치로 충분하다.
아랍어, 힌디어 등 복잡한 스크립트를 지원해야 할 때만 HarfBuzz가 필요하다.

### 3.3 Annotated Canvas (접근성)

| 항목 | Google Docs | rhwp |
|------|-------------|------|
| 렌더링 표면 | Canvas per page | Canvas per page (동일) |
| 입력 처리 | 숨겨진 textarea | 숨겨진 textarea (동일) |
| 캐럿/선택 | DOM 오버레이 | DOM 오버레이 (동일) |
| 접근성 DOM 트리 | 투명 병렬 DOM (`<p>`,`<span>` + ARIA) | **미구현** |
| 스크린 리더 | `aria-live` 알림 | 미지원 |

rhwp는 Google Docs와 동일한 시각적/입력 인프라를 갖추고 있으나,
접근성 어노테이션 레이어는 미구현 상태이다.
HWP 뷰어/에디터의 접근성 요구가 본격화될 때 추가 구현이 필요하다.

---

## 4. 결론: ThorVG의 적합 영역

### 4.1 역할 분담

| 용도 | 적합 렌더러 | 이유 |
|------|-------------|------|
| **실시간 편집** | Canvas 2D | 동기 렌더링, 브라우저 폰트 시스템, 즉시 화면 반영 |
| **읽기 전용 프리뷰** | ThorVG GL | GPU 가속, 고품질 벡터 렌더링, 폰트 사전 로드 가능 |
| **PNG/PDF 내보내기** | ThorVG Native | 서버사이드 렌더링, 브라우저 불필요, 픽셀 일관성 |
| **인쇄 미리보기** | ThorVG GL | 배치 렌더링, 사전 준비 시간 허용 |

### 4.2 핵심 인사이트

1. **브라우저 기능을 이기려 하지 말 것**
   - Canvas 2D `fillText()`는 브라우저 내부에서 HarfBuzz + GPU 가속으로 처리된다
   - WASM→JS→WebGL 다층 경계를 넘는 ThorVG는 실시간 편집에서 이 속도를 이길 수 없다
   - Google Docs도 동일한 이유로 페인팅을 브라우저에 위임한다

2. **ThorVG의 강점은 "오프라인 렌더링"**
   - 폰트를 사전 로드하고, 시간 제약 없이 렌더링할 때 진가를 발휘한다
   - 내보내기, 썸네일 생성, 서버사이드 렌더링에 적합하다
   - 200~500KB의 경량 바이너리로 Skia(2~5MB) 대비 배포 부담이 적다

3. **아키텍처 분리가 핵심 자산**
   - 렌더 트리(JSON) 추상화 덕분에 렌더링 백엔드 교체가 가능하다
   - 편집 인프라(DOM 오버레이)가 렌더러와 독립적으로 동작한다
   - 이 분리 구조가 있기에 용도별 최적 렌더러를 선택할 수 있다

### 4.3 향후 방향

```
┌─────────────────────────────────────────────────────────────┐
│                     rhwp 렌더링 전략                         │
│                                                              │
│  [편집 모드]                      [뷰어/내보내기 모드]        │
│  Canvas 2D                       ThorVG                      │
│  ├ 브라우저 fillText()           ├ GL (웹 프리뷰)            │
│  ├ 브라우저 폰트 시스템           ├ Native (PNG/PDF 내보내기) │
│  ├ 동기 렌더링                   ├ 폰트 사전 로드            │
│  └ 즉시 화면 반영                └ 배치 렌더링               │
│                                                              │
│  [공유 인프라]                                               │
│  ├ Rust 파서/레이아웃 엔진 (WASM)                            │
│  ├ 렌더 트리 (JSON)                                          │
│  ├ TTF 폰트 메트릭 (ttf-parser)                              │
│  └ DOM 오버레이 (캐럿/선택/IME)                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 부록: POC에서 발견/수정한 버그

| 버그 | 원인 | 수정 |
|------|------|------|
| TTF 메트릭 적용 후 페이지 넘김 오류 | `estimate_text_width()`에 TTF 경로 추가 → Canvas 2D 레이아웃과 불일치 | TTF 경로를 layout.rs에서 제거, render_tree.rs charPositions에만 적용 |
| ThorVG PNG 내보내기 빈 페이지 | TTF 내부 family name 사용 → ThorVG는 file stem 사용 | `Path::file_stem()` 기반으로 변경 |
| "맑은 고딕" → "함초롬돋움" 치환 | `resolve_ttf_font()` 하드코딩 | 매핑 제거 (font-loader.ts에 이미 등록됨) |
| ThorVG GL 초기 렌더링 시 1페이지 누락 | 다중 페이지 동시 비동기 렌더링 → GL 캔버스 공유 충돌 | `renderToCanvas()` 직렬화 큐 추가 |
| ThorVG GL 비동기 렌더링 시 CSS zoom 미적용 | canvas.width가 비동기 완료 전에 CSS 적용 | zoom을 renderPageThorvg에 전달, 렌더링 완료 후 적용 |

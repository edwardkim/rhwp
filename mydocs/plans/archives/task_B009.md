# B-009: 인쇄 엔진 개발계획서

> **문서 유형**: 개발계획서
> **작성일**: 2026-02-23
> **상태**: 초안
> **우선순위**: P2 (핵심 차별화 기능)
> **선행 조건**: Hexagonal Architecture (타스크 149 완료), Renderer Trait (기존)

---

## 1. 개요

### 1.1 목표

rhwp Core Engine에서 **PDF/PostScript를 직접 생성**하고, Windows Localhost Agent를 통해 **브라우저 내에서 데스크톱 수준의 인쇄 경험**을 제공한다.

### 1.2 경쟁사 대비 차별점

| | 경쟁사 (한컴 웹기안기 등) | rhwp |
|---|---|---|
| 인쇄 방식 | PDF 다운로드 → 사용자가 별도 인쇄 | **브라우저 내 직접 인쇄** |
| 출력 품질 | PDF 뷰어 + 드라이버 의존 | **Core Engine이 보장** |
| 보안 워터마크 | 별도 솔루션 구매 | **내장** |
| 드라이버 문제 | 사용자/담당자 해결 | **PS RAW로 드라이버 우회** |
| 사용자 경험 | 8~10단계 (앱 전환 2회) | **3단계 (앱 전환 0회)** |

### 1.3 핵심 전략

```
PDF Renderer = 기본 (다운로드 + 일반 인쇄)
PostScript Renderer = 고급 (공공기관 행망 프린터 직접 출력)
Localhost Agent = 전송 (ActiveX 대체 패턴, 한국 IT 검증 완료)
```

---

## 2. 아키텍처

### 2.1 전체 구조

```
┌─────────────────────────────────────────────────────────┐
│  Core Domain (Rust, WASM)                               │
│                                                         │
│  Document → Layout → PageRenderTree                     │
│                           │                             │
│                     Renderer Trait                       │
│                     (7개 메서드)                          │
│                           │                             │
│              ┌────────────┼────────────┐                │
│              │            │            │                │
│         SvgRenderer  PdfRenderer  PostScriptRenderer    │
│         (기존)       (신규 P1)    (신규 P2)              │
│              │            │            │                │
│              ▼            ▼            ▼                │
│         화면 표시     Vec<u8>       Vec<u8>              │
│                      (PDF 바이트)  (PS 바이트)            │
└─────────────┬────────────┬────────────┬─────────────────┘
              │            │            │
              │     ┌──────┴──────┐     │
              │     │  WASM API   │     │
              │     │  (Adapter)  │     │
              │     └──────┬──────┘     │
              │            │            │
         ┌────┴────┐  ┌───┴───┐  ┌─────┴─────┐
         │ PDF     │  │ Print │  │ PS RAW    │
         │다운로드  │  │ Agent │  │ Agent     │
         │(Blob)   │  │ (PDF) │  │ (Spooler) │
         └─────────┘  └───┬───┘  └─────┬─────┘
                          │            │
                    ┌─────┴────────────┴─────┐
                    │   rhwp-print-service    │
                    │   (Windows Service)     │
                    │   localhost:9443 HTTPS  │
                    └────────────┬────────────┘
                                │
                          Win32 Spooler API
                                │
                            프린터 출력
```

### 2.2 Renderer Trait (기존, 변경 없음)

```rust
// src/renderer/mod.rs — 7개 메서드
pub trait Renderer {
    fn begin_page(&mut self, width: f64, height: f64);
    fn end_page(&mut self);
    fn draw_text(&mut self, text: &str, x: f64, y: f64, style: &TextStyle);
    fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64, corner_radius: f64, style: &ShapeStyle);
    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: &LineStyle);
    fn draw_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, style: &ShapeStyle);
    fn draw_image(&mut self, data: &[u8], x: f64, y: f64, w: f64, h: f64);
    fn draw_path(&mut self, commands: &[PathCommand], style: &ShapeStyle);
}
```

### 2.3 Localhost Agent 패턴

한국에서 ActiveX 대체로 이미 검증된 아키텍처를 그대로 채용한다.

```
[브라우저 rhwp-studio]              [rhwp-print-service]
       │                                    │
       │  GET /check                        │
       │───────────────────────────────→    │ 설치/버전 확인
       │  ← { version, status }             │
       │                                    │
       │  GET /printers                     │
       │───────────────────────────────→    │ EnumPrintersW()
       │  ← [{ name, driver, status }, ..]  │
       │                                    │
       │  POST /print                       │
       │  { printer, data(base64), type }   │
       │───────────────────────────────→    │ WTSQueryUserToken()
       │                                    │ ImpersonateLoggedOnUser()
       │                                    │ OpenPrinterW()
       │                                    │ StartDocPrinterW()
       │                                    │ WritePrinter()
       │  ← { jobId, status }              │
       │                                    │
       │  GET /job/{id}                     │
       │───────────────────────────────→    │ 상태 조회
       │  ← { status, progress }            │
```

참고 사례: Yessign(공동인증서), TouchEn nxKey(키보드보안), AhnLab Safe Transaction 등 동일 패턴.

---

## 3. 구성 요소별 상세 설계

### 3.1 PdfRenderer (Core, Rust)

**위치**: `src/renderer/pdf.rs`

PDF 1.7 표준에 따른 최소 구현. 외부 크레이트 의존 없이 직접 바이너리 생성.

#### PDF 구조

```
%PDF-1.7
1 0 obj  << /Type /Catalog /Pages 2 0 R >>
2 0 obj  << /Type /Pages /Kids [...] /Count N >>
3 0 obj  << /Type /Page /MediaBox [0 0 595 842] /Contents 4 0 R /Resources ... >>
4 0 obj  << /Length ... >>
stream
  BT /F1 12 Tf 72 720 Td (Hello) Tj ET   ← draw_text()
  0.5 0 0 RG 72 700 200 1 re S            ← draw_line()
  q ... cm ... Do Q                        ← draw_image()
endstream
...
xref
trailer
%%EOF
```

#### Renderer Trait 매핑

| Trait 메서드 | PDF 연산자 |
|-------------|-----------|
| `begin_page(w, h)` | Page 객체 생성, MediaBox 설정 |
| `end_page()` | Content stream 닫기 |
| `draw_text(text, x, y, style)` | `BT /Font size Tf x y Td (text) Tj ET` |
| `draw_rect(x, y, w, h, r, style)` | `x y w h re` (S/f/B) |
| `draw_line(x1, y1, x2, y2, style)` | `x1 y1 m x2 y2 l S` |
| `draw_ellipse(cx, cy, rx, ry, style)` | 4개 Bézier 곡선 근사 |
| `draw_image(data, x, y, w, h)` | XObject Image + `Do` |
| `draw_path(commands, style)` | `m`/`l`/`c`/`h` + S/f/B |

#### 좌표계 변환

```
rhwp (px, 96 DPI, 원점=좌상단)  →  PDF (pt, 72 DPI, 원점=좌하단)

pdf_x = px_x * 72.0 / 96.0
pdf_y = page_height_pt - (px_y * 72.0 / 96.0)
```

#### 폰트 전략

| 단계 | 접근 | 설명 |
|------|------|------|
| Phase 1 | PDF 기본 14폰트 매핑 | 한글은 CIDFont /KR 사용 |
| Phase 2 | TrueType 임베딩 | WASM 내장 폰트 → PDF /FontFile2 |
| Phase 3 | 서브세팅 | 사용된 글리프만 추출하여 파일 크기 최소화 |

### 3.2 PostScriptRenderer (Core, Rust)

**위치**: `src/renderer/postscript.rs`

PostScript Level 2 기반. PDF와 동일한 벡터 명령어를 PS 문법으로 출력.

#### Renderer Trait 매핑

| Trait 메서드 | PostScript 명령어 |
|-------------|------------------|
| `begin_page(w, h)` | `%%Page: n n`, `%%PageBoundingBox: 0 0 w h` |
| `end_page()` | `showpage` |
| `draw_text(text, x, y, style)` | `/Font size selectfont x y moveto (text) show` |
| `draw_rect(x, y, w, h, r, style)` | `x y w h rectfill` 또는 `rectstroke` |
| `draw_line(x1, y1, x2, y2, style)` | `x1 y1 moveto x2 y2 lineto stroke` |
| `draw_ellipse(cx, cy, rx, ry, style)` | `gsave translate scale arc grestore` |
| `draw_image(data, x, y, w, h)` | `image` 또는 `colorimage` 연산자 |
| `draw_path(commands, style)` | `moveto`/`lineto`/`curveto`/`closepath` |

#### PS 출력 예시

```postscript
%!PS-Adobe-3.0
%%Pages: 3
%%DocumentFonts: NanumGothic

%%Page: 1 1
%%PageBoundingBox: 0 0 595 842

% draw_text("안녕하세요", 72, 100, style)
/NanumGothic 12 selectfont
0 0 0 setrgbcolor
72 742 moveto
<C548B155D558C138C694> show

% draw_rect(72, 200, 200, 50, 0, style)
0.8 0.8 0.8 setrgbcolor
72 592 200 50 rectfill

% draw_line(72, 300, 272, 300, style)
0 0 0 setrgbcolor
1 setlinewidth
72 542 moveto 272 542 lineto stroke

showpage
```

#### PDF 대비 장점 (공공기관)

```
PS RAW → Spooler → 프린터
  ✓ 드라이버 완전 우회 → 드라이버 호환 문제 제거
  ✓ Core Engine이 최종 출력 100% 통제
  ✓ 보안 워터마크를 PS 레벨에서 삽입 → 변조 불가
  ✓ 행망 등록 프린터 = 전부 PS 지원 (업무용 레이저 복합기)
```

### 3.3 rhwp-print-service (Windows Service)

**별도 프로젝트**: `rhwp-print-service/`

#### 기술 스택

| 구성 | 기술 |
|------|------|
| 언어 | Rust |
| HTTP 서버 | hyper + rustls (TLS) |
| Windows API | windows-rs 크레이트 |
| 서비스 등록 | windows-service 크레이트 |
| 인증서 | rcgen (자체 서명, 설치 시 Root CA 등록) |
| 설치 패키지 | WiX Toolset (MSI) |

#### API 명세

| Method | Path | 설명 | 요청 | 응답 |
|--------|------|------|------|------|
| GET | `/check` | 서비스 상태 확인 | - | `{ version, status }` |
| GET | `/printers` | 프린터 목록 조회 | - | `[{ name, driver, port, status }]` |
| POST | `/print` | 인쇄 작업 전송 | `{ printer, docName, data, dataType }` | `{ jobId, status }` |
| GET | `/job/{id}` | 작업 상태 조회 | - | `{ status, progress }` |

#### dataType 옵션

| 값 | Spooler 데이터 타입 | 용도 |
|----|---------------------|------|
| `"raw"` | `RAW` | PS/PCL 직접 전송 (드라이버 우회) |
| `"pdf"` | `NT EMF 1.008` | PDF → EMF → 드라이버 경유 |
| `"xps"` | `XPS_PASS` | XPS 파이프라인 경유 |

#### 보안 설계

```
1. 바인딩: 127.0.0.1만 (외부 접근 원천 차단)
2. TLS: 자체 서명 인증서 (설치 시 Root CA 등록)
3. CORS: rhwp-studio 오리진만 허용
4. Origin 검증: 요청마다 Origin 헤더 확인
5. Rate Limiting: 과다 요청 차단
6. 사용자 격리: WTSQueryUserToken → Impersonation (사용자 프린터만 접근)
```

### 3.4 브라우저 측 (rhwp-studio)

**위치**: `rhwp-studio/src/print/`

#### 인쇄 흐름

```
사용자 Ctrl+P
    │
    ├─ Agent 설치 확인 (GET /check, 1.5초 타임아웃)
    │   ├─ 미설치 → 설치 안내 대화상자
    │   └─ 설치됨 → 계속
    │
    ├─ 프린터 목록 조회 (GET /printers)
    │
    ├─ 인쇄 대화상자 표시
    │   ├─ 프린터 선택 (드롭다운)
    │   ├─ 페이지 범위 (전체/현재/지정)
    │   ├─ 매수
    │   ├─ 양면 인쇄
    │   ├─ 출력 방식: PS 직접(권장) / PDF 경유
    │   └─ 보안 옵션: 워터마크, 보안 등급
    │
    ├─ WASM에서 PDF 또는 PS 생성
    │   └─ DocumentCore → PdfRenderer/PostScriptRenderer → Vec<u8>
    │
    ├─ Agent로 전송 (POST /print)
    │
    └─ 진행 상태 표시 (GET /job/{id} 폴링)
```

#### Agent 미설치 시 폴백

```
Agent 미설치 / macOS / Linux
    │
    ├─ PDF Blob 다운로드 (기존 방식)
    └─ 또는 window.print() (브라우저 인쇄)
```

---

## 4. 단계별 로드맵

### Phase 1: PDF Renderer (기반 구축)

**목표**: Renderer trait 구현체로 PDF를 생성하여 다운로드 기능 제공

| 단계 | 내용 | 산출물 |
|------|------|--------|
| 1-1 | PDF 바이너리 기본 구조 (헤더, xref, trailer) | `src/renderer/pdf.rs` |
| 1-2 | draw_text: 기본 14폰트 + CIDFont/KR 한글 | 텍스트 출력 검증 |
| 1-3 | draw_rect, draw_line, draw_ellipse, draw_path | 도형 출력 검증 |
| 1-4 | draw_image: JPEG/PNG 임베딩 | 이미지 출력 검증 |
| 1-5 | 다중 페이지 + 좌표계 변환 검증 | 샘플 HWP → PDF 비교 |
| 1-6 | WASM API 바인딩 (`export_pdf_native`) | JS에서 PDF Blob 다운로드 |

**검증**: 샘플 HWP 10종 → PDF 변환 → 조판 일치성 확인

### Phase 2: PostScript Renderer (공공기관 차별화)

**목표**: PDF Renderer와 동일한 벡터 경로를 PS 문법으로 출력

| 단계 | 내용 | 산출물 |
|------|------|--------|
| 2-1 | PS Level 2 기본 프레임 (DSC 헤더, 페이지 구조) | `src/renderer/postscript.rs` |
| 2-2 | draw_text: PS 폰트 선택 + 한글 CID 처리 | 텍스트 출력 검증 |
| 2-3 | draw_rect, draw_line, draw_ellipse, draw_path | 도형 출력 검증 |
| 2-4 | draw_image: PS image/colorimage 연산자 | 이미지 출력 검증 |
| 2-5 | 보안 워터마크 PS 레벨 삽입 | 워터마크 출력 검증 |
| 2-6 | 샘플 HWP → PS → GhostScript 렌더링 비교 | 조판 일치성 확인 |

**검증**: GhostScript로 PS → PNG 변환 후 SVG 출력과 픽셀 비교

### Phase 3: Windows Print Service (전송 계층)

**목표**: Localhost HTTPS Agent로 브라우저 → Spooler 인쇄 경로 확보

| 단계 | 내용 | 산출물 |
|------|------|--------|
| 3-1 | Rust Windows Service 스켈레톤 (SCM 등록/시작/중지) | `rhwp-print-service/` |
| 3-2 | Hyper + Rustls HTTPS 서버 (localhost 바인딩) | TLS 통신 검증 |
| 3-3 | 자체 서명 인증서 생성 + Root CA 등록 | 브라우저 경고 없이 통신 |
| 3-4 | `/check`, `/printers` API (EnumPrintersW) | 프린터 목록 표시 |
| 3-5 | `/print` API (OpenPrinterW → WritePrinter) | RAW/PDF 인쇄 |
| 3-6 | 사용자 토큰 위임 (WTSQueryUserToken + Impersonation) | 네트워크 프린터 접근 |
| 3-7 | `/job/{id}` 상태 조회 | 진행률 표시 |

**검증**: 실제 프린터 출력 테스트 (로컬 + 네트워크)

### Phase 4: 브라우저 통합 (최종 UX)

**목표**: Ctrl+P → 인쇄 대화상자 → 출력 완료 (데스크톱 한글과 동일)

| 단계 | 내용 | 산출물 |
|------|------|--------|
| 4-1 | print-client.ts (Agent 통신 모듈) | `rhwp-studio/src/print/` |
| 4-2 | Agent 설치 감지 + 미설치 안내 UI | 설치 유도 대화상자 |
| 4-3 | 인쇄 대화상자 UI (프린터 선택, 옵션) | 한컴 한글 스타일 UI |
| 4-4 | WASM export_pdf/export_ps → Agent 전송 | End-to-End 인쇄 |
| 4-5 | 진행 상태 표시 + 오류 처리 | 사용자 피드백 |
| 4-6 | Agent 미설치 폴백 (PDF 다운로드) | 범용 호환성 |

**검증**: 실사용 시나리오 테스트 (공공기관 환경 시뮬레이션)

### Phase 5: 보안 인쇄 + 고급 기능

**목표**: 공공기관 보안 요구사항 충족

| 단계 | 내용 | 산출물 |
|------|------|--------|
| 5-1 | 보안 워터마크 (사용자명, 일시, 등급) | PDF/PS 양쪽 지원 |
| 5-2 | Banner Page (표지 정보 자동 생성) | 보안 규정 준수 |
| 5-3 | Job Ticket 메타데이터 (사용자 ID, 보안 등급) | 감사 추적 지원 |
| 5-4 | TrueType 폰트 서브세팅 (PDF/PS) | 파일 크기 최적화 |
| 5-5 | MSI 설치 패키지 + GPO 배포 지원 | 관공서 대규모 배포 |

---

## 5. 기술적 고려사항

### 5.1 좌표계

```
                 rhwp 내부          PDF              PostScript
원점             좌상단              좌하단            좌하단
단위             px (96 DPI)        pt (72 DPI)      pt (72 DPI)
A4 크기          793.7 × 1122.5 px  595 × 842 pt     595 × 842 pt
Y축 방향         ↓ (아래로 증가)     ↑ (위로 증가)     ↑ (위로 증가)
```

변환 공식:
```
pt_x = px_x × (72 / dpi)
pt_y = page_height_pt - px_y × (72 / dpi)
```

### 5.2 한글 폰트 처리

| 단계 | PDF | PostScript |
|------|-----|------------|
| 초기 | CIDFont + Adobe-Korea1-2 인코딩 | CIDFont + CMap |
| 중기 | TrueType 임베딩 (/FontFile2) | Type 42 (TrueType in PS) |
| 최종 | 글리프 서브세팅 | 글리프 서브세팅 |

### 5.3 이미지 처리

| 형식 | PDF | PostScript |
|------|-----|------------|
| JPEG | `/Filter /DCTDecode` (원본 스트림 그대로) | `<< /Filter /DCTDecode >>` |
| PNG | `/Filter /FlateDecode` + alpha 분리 | `image` + Decode/DataSource |
| BMP | Raw → Flate 압축 | `image` 연산자 |

### 5.4 크레이트 의존성 최소화 원칙

PDF/PS 생성은 **외부 크레이트 없이** 직접 바이너리/텍스트를 구성한다.

```
허용:
  - flate2 (Deflate 압축, PDF 스트림 용)
  - WASM 내장 폰트 데이터 (기존)

불허:
  - printpdf, lopdf 등 PDF 라이브러리 (제어권 상실)
  - cairo, skia 등 렌더링 엔진 (WASM 비호환)
```

이유: WASM 환경에서 동작해야 하며, 바이트 수준 통제가 보안 워터마크 삽입의 전제 조건이다.

---

## 6. 프로젝트 구조

### 6.1 Core (기존 rhwp에 추가)

```
src/renderer/
├── mod.rs              ← Renderer trait (기존, 변경 없음)
├── svg.rs              ← SvgRenderer (기존)
├── canvas.rs           ← CanvasRenderer (기존)
├── html.rs             ← HtmlRenderer (기존)
├── pdf.rs              ← PdfRenderer (신규, Phase 1)
├── pdf/
│   ├── mod.rs          ← PdfRenderer 구조체 + Renderer impl
│   ├── objects.rs      ← PDF 객체 (Catalog, Page, Font, XObject)
│   ├── stream.rs       ← Content Stream 생성 (그리기 연산자)
│   ├── font.rs         ← 폰트 임베딩 + CIDFont 처리
│   └── image.rs        ← 이미지 임베딩 (JPEG/PNG)
├── postscript.rs       ← PostScriptRenderer (신규, Phase 2)
└── postscript/
    ├── mod.rs          ← PostScriptRenderer 구조체 + Renderer impl
    ├── dsc.rs          ← DSC 헤더/푸터 (%%Page, %%EOF 등)
    ├── font.rs         ← PS 폰트 정의 (Type 42, CIDFont)
    └── image.rs        ← PS 이미지 처리 (image 연산자)
```

### 6.2 Windows Print Service (별도 프로젝트)

```
rhwp-print-service/
├── Cargo.toml
├── src/
│   ├── main.rs              ← Windows Service 진입점
│   ├── service.rs           ← SCM 등록/시작/중지
│   ├── tls_server.rs        ← Hyper + Rustls HTTPS 서버
│   ├── cert.rs              ← 자체 서명 인증서 생성/관리
│   ├── routes.rs            ← REST API 핸들러
│   ├── print_spooler.rs     ← Win32 Spooler API (OpenPrinter/WritePrinter)
│   └── security.rs          ← Origin 검증, Rate Limiting, Token 위임
├── installer/
│   ├── rhwp-print.wxs       ← WiX MSI 정의
│   └── install.ps1          ← PowerShell 설치 스크립트
└── tests/
    └── integration_test.rs  ← 스풀러 연동 테스트
```

### 6.3 브라우저 측 (rhwp-studio에 추가)

```
rhwp-studio/src/
├── print/
│   ├── print-client.ts      ← Agent 통신 (fetch localhost:9443)
│   ├── print-dialog.ts      ← 인쇄 대화상자 UI
│   ├── print-dialog.css     ← 대화상자 스타일
│   └── print-fallback.ts    ← Agent 미설치 시 PDF 다운로드 폴백
```

---

## 7. 대상 환경

### 7.1 인쇄 경로별 대상

| 대상 환경 | 인쇄 경로 | 비고 |
|-----------|----------|------|
| 공공기관 (행망) | PS RAW → Agent → Spooler | **최적 경로**. 행망 프린터 전부 PS 지원 |
| 일반 기업 | PDF → Agent → Spooler (드라이버 경유) | 범용 호환 |
| 개인 사용자 | PDF 다운로드 | Agent 미설치 시 폴백 |
| macOS / Linux | PDF 다운로드 또는 브라우저 인쇄 | 향후 CUPS Agent 확장 가능 |

### 7.2 Windows 버전 지원

| OS | 지원 | 비고 |
|----|------|------|
| Windows 11 | O | 주 대상 |
| Windows 10 | O | 공공기관 다수 |
| Windows Server 2019+ | O | 터미널 서비스 환경 |

---

## 8. 리스크 및 완화

| 리스크 | 영향 | 완화 방안 |
|--------|------|----------|
| 한글 폰트 PDF 임베딩 복잡도 | Phase 1 지연 | CIDFont 우선, 임베딩은 후속 |
| 자체 서명 인증서 브라우저 호환 | Agent 통신 실패 | 설치 시 Root CA 등록 자동화 |
| 기업 방화벽 localhost 차단 | Agent 통신 불가 | 포트 변경 옵션 + PDF 폴백 |
| PS 출력 프린터별 호환성 차이 | 출력 오류 | PS Level 2 (최대 호환) + 테스트 매트릭스 |
| Service 계정 프린터 접근 권한 | 네트워크 프린터 실패 | WTSQueryUserToken Impersonation |
| WASM 환경 메모리 제한 | 대용량 문서 PDF 생성 실패 | 페이지별 스트리밍 생성 |

---

## 9. 성공 기준

| 항목 | 기준 |
|------|------|
| PDF 조판 일치 | SVG 렌더링과 픽셀 단위 비교 시 오차 < 1px |
| PS 조판 일치 | GhostScript 렌더링 결과와 오차 < 1px |
| 인쇄 단계 수 | Ctrl+P → 출력 완료까지 3단계 이하 |
| Agent 응답 시간 | `/printers` 조회 < 500ms |
| 인쇄 전송 시간 | 10페이지 문서 < 3초 |
| 테스트 커버리지 | PDF/PS Renderer 70% 이상 |

---

> [!IMPORTANT]
> 본 계획의 핵심 가치는 **"웹인데 데스크톱보다 낫다"**를 인쇄에서 실현하는 것이다.
> PDF Renderer가 기반이고, PostScript는 공공기관 차별화, Localhost Agent는 한국 IT 생태계에서 검증된 전송 수단이다.
> 이 세 요소의 조합이 경쟁사와의 결정적 제품 차이를 만든다.

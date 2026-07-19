---
kind: snapshot
status: historical
canonical: mydocs/tech/archive/README.md
last_verified: 2026-07-16
---

# rhwp 개발 로드맵

## 현황 요약 (2026년 2월 10일 기준)

### 완성된 핵심 엔진

| 영역 | 상태 | 세부 |
|------|------|------|
| HWP 파싱 | 완료 | OLE Compound File, DocInfo/BodyText/BinData 전체 해석 |
| HWP 직렬화 | 완료 | 수정된 문서를 HWP 바이너리로 저장, 한컴오피스 호환 확인 |
| SVG 렌더링 | 완료 | 텍스트, 표, 이미지, 도형, 배경 |
| WASM 빌드 | 완료 | 브라우저 동작 확인, 78개 WASM API |
| 텍스트 편집 | 완료 | 삽입/삭제/분리/병합, 서식 적용, 리플로우 |
| 표 편집 | 완료 | 행/열 추가, 셀 병합/분할, 셀 텍스트 편집 |
| 웹 에디터 | 완료 | 캐럿, 선택, 서식 툴바, 언어별 폰트 분기 |
| 단위 테스트 | 414개 | 전수 통과 |

### 누적 타스크 이력 (1~33)

33개 타스크를 통해 파서 → 렌더러 → 편집기 → 직렬화 → 서식 도구까지 단계적으로 구축.

### 미구현 영역 (제품화에 필요)

| 구성요소 | 현재 상태 | 제품화 필수 여부 |
|----------|----------|----------------|
| HTML → HWP 변환 | 미구현 | **필수** (Phase 1 핵심) |
| HWP → PDF 변환 | 미구현 | **필수** (Phase 1 핵심) |
| Python 바인딩 (PyO3) | 미구현 | **필수** (배포 채널) |
| PyPI 패키지 | 미구현 | **필수** (배포) |
| MCP 서버 | 미구현 | **필수** (Phase 2 핵심) |
| npm 패키지 정비 | 부분 (pkg/ 존재) | 권장 |
| 머리글/꼬리글 렌더링 | 미구현 | 권장 |
| 다단 레이아웃 | 미구현 | 선택 |
| 수식/차트 렌더링 | 미구현 | 선택 |
| Undo/Redo | 미구현 | 선택 (에디터 전용) |
| 복사/붙여넣기 | 미구현 | 선택 (에디터 전용) |

---

## 전체 타임라인

```
2026년
──────────────────────────────────────────────────────────
2월 10일~    Phase 1: 제품화 핵심 기능        ← 현재
             ├─ HTML → HWP 변환
             ├─ HWP → PDF 변환 (hwp2pdf)
             └─ 품질 보강 (머리글/꼬리글 등)

3월 초~      Phase 2: 배포 채널 구축
             ├─ PyO3 Python 바인딩
             ├─ PyPI 패키지 배포
             └─ npm 패키지 정비

3월 중순~    Phase 3: AI Agent 통합
             ├─ MCP 서버 구현
             ├─ Claude 스킬 문서
             └─ 템플릿 시스템

3월 말       ▶ v1.0 릴리스 (제품 완성)

4월~6월      Phase 4: 시장 침투 (오픈소스 공개)
             ├─ GitHub 공개 (MIT/Apache 2.0)
             ├─ 기술 블로그, 데모 사이트
             ├─ 커뮤니티 구축
             └─ 초기 사용자 피드백 반영

7월~         Phase 5: 수익화 전환
             ├─ 듀얼 라이선싱 또는 Open Core 전환
             ├─ 엔터프라이즈 모듈 (암호화, 대량 배치)
             └─ B2B/B2G 기술지원 계약
──────────────────────────────────────────────────────────
```

---

## Phase 1: 제품화 핵심 기능 (2월 중순~3월 초)

**목표**: AI Agent가 실제로 사용할 수 있는 두 가지 핵심 변환 기능 구현

### 1-1. HTML → HWP 변환

AI Agent가 생성한 HTML을 HWP 바이너리로 변환하는 핵심 파이프라인.

**기술 설계**:
```
HTML 문자열
  → HTML 파서 (html5ever 또는 경량 파서)
    → 중간 표현 (IR) 매핑
      → HWP 문서 모델 (Paragraph, Table, CharShape, ParaShape)
        → HWP 직렬화 (기존 serializer 활용)
          → .hwp 파일 출력
```

**지원 HTML 요소 (우선순위)**:

| 우선순위 | HTML 요소 | HWP 매핑 |
|----------|----------|----------|
| P0 (필수) | `<h1>`~`<h6>` | 문단 + 글자크기/굵게 |
| P0 | `<p>` | 문단 |
| P0 | `<strong>`, `<em>`, `<u>` | CharShape (굵게/기울임/밑줄) |
| P0 | `<table>`, `<tr>`, `<td>` | Table 컨트롤 + Cell |
| P0 | `<br>` | 줄바꿈 |
| P1 (중요) | `<ul>`, `<ol>`, `<li>` | 문단 + 번호 매기기 |
| P1 | `<img>` | Picture 컨트롤 (BinData) |
| P1 | `<a>` | 하이퍼링크 |
| P1 | `style` 속성 | CharShape/ParaShape 매핑 |
| P2 (선택) | `<div>` + CSS | 레이아웃 힌트 |
| P2 | `<code>`, `<pre>` | 고정폭 폰트 문단 |

**필요 의존성**:
- `html5ever` 또는 `scraper` (HTML 파싱)
- 기존 `model/` + `serializer/` 활용

**산출물**: `html_to_hwp(html: &str, output_path: &str)` 함수

**예상 작업량**: 타스크 3~4개 (HTML 파싱 → 텍스트/서식 매핑 → 표 매핑 → 통합 테스트)

### 1-2. HWP → PDF 변환 (hwp2pdf)

기존 SVG 렌더링 엔진을 활용한 PDF 변환.

**기술 설계**:
```
HWP 파일
  → rhwp 파싱 (기존)
    → 페이지 레이아웃 (기존)
      → SVG 렌더링 (기존)
        → SVG → PDF 변환 (신규)
          → .pdf 파일 출력
```

**구현 방안 비교**:

| 방안 | 라이브러리 | 장점 | 단점 |
|------|-----------|------|------|
| A. SVG→PDF | `svg2pdf` + `printpdf` | 기존 SVG 엔진 재활용, 구현 최소 | 폰트 임베딩 처리 필요 |
| B. 직접 PDF 생성 | `printpdf` | SVG 중간 단계 불필요, 세밀 제어 | 레이아웃 로직 재구현 필요 |
| C. 외부 도구 연동 | `resvg` + `cairo` | 렌더링 품질 보장 | 외부 의존성 증가, WASM 불가 |

**권장: 방안 A (SVG→PDF)**
- 이미 검증된 SVG 렌더링 결과물을 그대로 PDF로 변환
- `printpdf` crate 사용, 폰트 임베딩은 시스템 폰트 또는 번들 폰트 활용

**산출물**: `hwp_to_pdf(hwp_path: &str, pdf_path: &str)` 함수 + CLI `rhwp export-pdf`

**예상 작업량**: 타스크 2~3개 (SVG→PDF 변환기 → 폰트 임베딩 → 다중 페이지)

### 1-3. 품질 보강 (선택)

| 기능 | 설명 | 제품 영향 |
|------|------|----------|
| 머리글/꼬리글 | 공문서에 필수적인 요소 | PDF 변환 품질 향상 |
| 페이지 번호 | 자동 페이지 번호 삽입 | 공문서 요구사항 |
| 빈 문서 생성 개선 | 기본 스타일이 포함된 깨끗한 빈 HWP | 템플릿 시스템 기반 |

---

## Phase 2: 배포 채널 구축 (3월 초~중순)

**목표**: `pip install rhwp`로 누구나 즉시 사용할 수 있는 상태

### 2-1. PyO3 Python 바인딩

**공개 API 설계**:

```python
import rhwp

# === 변환 API (Phase 1 핵심) ===
rhwp.html_to_hwp(html_string, output_path)         # HTML → HWP
rhwp.hwp_to_pdf(hwp_path, pdf_path)                # HWP → PDF

# === 읽기/수정 API ===
doc = rhwp.open(hwp_path)                           # HWP 열기
text = doc.extract_text()                           # 전체 텍스트 추출
doc.insert_text(section, para, offset, text)        # 텍스트 삽입
doc.delete_text(section, para, start, end)          # 텍스트 삭제
doc.save(output_path)                               # HWP 저장

# === 구조화 생성 API ===
doc = rhwp.create()                                 # 빈 문서 생성
doc.add_paragraph(text, style="heading1")           # 문단 추가
doc.add_table(rows, cols, data)                     # 표 추가
doc.save(output_path)

# === 정보 API ===
info = rhwp.info(hwp_path)                          # 문서 메타정보
pages = rhwp.render_svg(hwp_path)                   # SVG 렌더링
```

**필요 작업**:
- `Cargo.toml`에 PyO3 의존성 추가 + feature flag (`python` feature)
- `src/python/` 모듈 신설 → PyO3 래퍼 함수 작성
- Python 타입 변환 (bytes ↔ Vec<u8>, str ↔ String, dict ↔ JSON)

**예상 작업량**: 타스크 2~3개

### 2-2. PyPI 패키지 배포

**패키지 구조**:
```
rhwp/
├── pyproject.toml          # maturin 빌드 설정
├── Cargo.toml              # python feature flag
├── src/
│   └── python/             # PyO3 바인딩
├── python/
│   └── rhwp/
│       ├── __init__.py     # Python 패키지 진입점
│       └── py.typed        # 타입 힌트 마커
└── README.md
```

**빌드 도구**: `maturin` (PyO3 공식 빌드 도구)
- `maturin develop` → 로컬 설치
- `maturin build --release` → wheel 생성
- `maturin publish` → PyPI 업로드

**지원 플랫폼** (maturin 크로스 컴파일):
- Linux x86_64 (주력)
- macOS x86_64 / ARM64
- Windows x86_64

**예상 작업량**: 타스크 1~2개

### 2-3. npm 패키지 정비

현재 `pkg/`에 WASM 빌드 결과물이 있으나, 공개 배포를 위한 정비 필요.

- `package.json` 메타데이터 완성 (description, keywords, license, repository)
- README.md 작성 (사용 예시, API 문서)
- `npm publish` 테스트

**예상 작업량**: 타스크 1개

---

## Phase 3: AI Agent 통합 (3월 중순~말)

**목표**: Claude Code/Cowork에서 HWP 문서를 직접 조작할 수 있는 MCP 도구 제공

### 3-1. MCP 서버 구현

**MCP 도구 정의**:

| 도구 이름 | 설명 | 입력 | 출력 |
|-----------|------|------|------|
| `html_to_hwp` | HTML → HWP 변환 | html: string, path: string | 성공/실패 |
| `hwp_to_pdf` | HWP → PDF 변환 | hwp_path: string, pdf_path: string | 성공/실패 |
| `create_hwp` | 구조화된 데이터로 HWP 생성 | sections: JSON, path: string | 성공/실패 |
| `read_hwp` | HWP 텍스트/구조 추출 | path: string | JSON (텍스트, 표, 메타데이터) |
| `modify_hwp` | 기존 HWP 수정 | path: string, ops: JSON | 성공/실패 |
| `render_hwp` | HWP → SVG/PDF 렌더링 | path: string, format: string | 파일 경로 |

**아키텍처**:
```
Claude Code / Cowork
    │
    │  MCP (JSON-RPC over stdio)
    ▼
┌─────────────────────────────┐
│ rhwp-mcp (Rust 단일 바이너리) │
│                             │
│  initialize()               │
│  tools/list → 6개 도구       │
│  tools/call → 도구 실행      │
│                             │
│  내부: rhwp 핵심 엔진 직접 호출 │
└─────────────────────────────┘
    │
    ▼
  .hwp / .pdf 파일
```

**구현 방식**:
- 별도 바이너리: `rhwp-mcp` (Cargo workspace member 또는 feature flag)
- `stdin/stdout`으로 JSON-RPC 메시지 처리
- MCP SDK 사용 또는 직접 프로토콜 구현 (경량)

**예상 작업량**: 타스크 2~3개

### 3-2. Claude 스킬 문서

AI Agent가 rhwp를 올바르게 사용할 수 있도록 스킬 문서를 제공.

```markdown
# HWP 문서 도구 (rhwp)

## 사용 가능한 도구
- html_to_hwp: HTML 문자열을 HWP 파일로 변환
- hwp_to_pdf: HWP 파일을 PDF로 변환
- read_hwp: HWP 파일의 텍스트/구조 추출
- modify_hwp: 기존 HWP 파일 수정
- create_hwp: 구조화 데이터로 HWP 생성

## 사용 지침
- 한국어 공문서/보고서가 필요하면 html_to_hwp 사용
- HTML은 <h1>~<h6>, <p>, <table>, <strong> 등 표준 태그 사용
- PDF 제출이 필요하면 hwp_to_pdf로 추가 변환
```

### 3-3. 템플릿 시스템

공문서/기관 양식에 데이터를 채워넣는 고급 기능.

**워크플로우**:
```
템플릿 HWP + 매니페스트 YAML
    │
    ├─ AI Agent가 read_hwp()로 구조 파악
    ├─ 매니페스트에서 필드 위치/형식 확인
    ├─ modify_hwp()로 데이터 삽입
    └─ 완성된 HWP 저장
```

**예상 작업량**: 타스크 1~2개 (기존 읽기/수정 API 활용)

---

## Phase 4: 시장 침투 (4월~6월)

**전략**: "일단 깔리게 하라" — 최대한 많은 AI Agent가 rhwp를 사용하게 만든다.

### 4-1. 오픈소스 공개

**라이선스 전략**: MIT 또는 Apache 2.0 (완전 무료)
- 진입 장벽 최소화: 기업도 법적 부담 없이 즉시 채용 가능
- 경쟁사가 따라올 의지를 꺾는 선제 공개

**공개 항목**:
- GitHub 리포지토리 (코어 엔진 + WASM + Python 바인딩)
- PyPI 패키지 (`pip install rhwp`)
- npm 패키지 (`npm install rhwp`)
- Crates.io (`cargo add rhwp`)
- MCP 서버 바이너리 (GitHub Releases)

### 4-2. 기술 마케팅

| 채널 | 내용 | 목표 |
|------|------|------|
| GitHub README | 설치 → 5분 퀵스타트 → 데모 GIF | 첫인상 확보 |
| 데모 사이트 | 브라우저에서 HWP 뷰어/에디터 체험 | 기술력 증명 |
| 기술 블로그 | "HWP 바이너리 포맷 역공학기", "Rust로 OLE 파싱하기" | 개발자 관심 |
| Hacker News / Reddit | 영문 소개: "First open-source HWP writer" | 글로벌 노출 |
| 한국 커뮤니티 | GeekNews, 디스코드, 개발자 커뮤니티 | 한국 개발자 확보 |

### 4-3. 핵심 지표 (KPI)

| 지표 | 1개월 목표 (4월) | 3개월 목표 (6월) |
|------|-----------------|-----------------|
| GitHub Stars | 100+ | 500+ |
| PyPI 주간 다운로드 | 50+ | 500+ |
| npm 주간 다운로드 | 30+ | 200+ |
| MCP 서버 설치 수 | 20+ | 100+ |
| 이슈/PR 참여자 | 5+ | 20+ |

### 4-4. 초기 사용자 확보 경로

| 대상 | 접근 방식 | 가치 제안 |
|------|----------|----------|
| AI 스타트업 (뤼튼 등) | 직접 컨택, 기술 데모 | "한국어 AI 비서에 HWP 출력 추가" |
| SI 기업 (삼성SDS, LG CNS) | 기술 세미나, PoC 제안 | "공공 SI에서 HWP 자동 생성" |
| 개인 개발자 | 오픈소스 커뮤니티, 블로그 | "토이 프로젝트에서 무료로 HWP 생성" |
| 공공기관 IT 담당 | 기관 내 AI 도입 시 연동 제안 | "기안문/보고서 AI 자동 작성" |

---

## Phase 5: 수익화 전환 (7월~)

### 5-1. 비즈니스 모델 선택

피드백 문서(`bz_model.md`) 분석 결과, 3가지 모델을 단계적으로 적용한다.

**1차: 듀얼 라이선싱 (iText 모델)** — 가장 강력 추천

```
코어 라이브러리 (MIT/Apache)          엔터프라이즈 (상용 라이선스)
├─ html_to_hwp()                    ├─ rhwp-pdf (PDF 변환 모듈)
├─ read_hwp()                       ├─ rhwp-crypto (문서 암호화/DRM)
├─ modify_hwp()                     ├─ rhwp-batch (대량 배치 처리)
├─ create_hwp()                     ├─ rhwp-template-builder (GUI)
└─ MCP 서버 (기본)                   └─ SLA 기술지원 (24h 패치)
```

전환 시나리오:
- 코어 라이브러리는 MIT/Apache 유지 → 시장 점유 지속
- 기업이 꼭 필요로 하는 기능을 유료 모듈로 분리
- GPL이 아닌 MIT/Apache + 유료 애드온 방식 (Open Core에 더 가까움)

**2차: 기술지원 구독 (Red Hat 모델)** — B2G/B2B

```
구독 등급:
├─ Community (무료): GitHub Issues, 커뮤니티 포럼
├─ Professional (월 50만원~): 48h SLA, 이메일 지원, 분기별 컨설팅
└─ Enterprise (월 200만원~): 24h SLA, 전담 엔지니어, 커스텀 개발
```

### 5-2. 유료 모듈 후보

| 모듈 | 대상 고객 | 예상 가격 |
|------|----------|----------|
| `rhwp-pdf` | 모든 기업 (PDF 출력 필수) | 연 100만원~ |
| `rhwp-crypto` | 금융/공공 (문서 보안) | 연 200만원~ |
| `rhwp-batch` | 대량 발송 시스템 (통지서 등) | 연 300만원~ |
| `rhwp-template-builder` | SI 기업 (양식 매핑 도구) | 연 500만원~ |
| 기술지원 SLA | B2G SI 프로젝트 | 프로젝트당 1000만원~ |

### 5-3. 수익화 전환 조건

다음 조건이 **모두** 충족되면 유료 모듈 도입:
1. PyPI 주간 다운로드 500건 이상 (시장 인지도 확보)
2. 기업에서의 사용 사례 3건 이상 확인
3. GitHub Stars 500개 이상 (커뮤니티 신뢰)
4. 기업으로부터 기술지원/유료화 문의 수신

조건 미충족 시: 무료 공개 지속, 시장 침투에 집중.

---

## 기술 의존성 추가 계획

### Phase 1에서 추가

```toml
[dependencies]
# HTML 파싱 (html_to_hwp용)
html5ever = "0.29"        # HTML5 표준 파서
markup5ever = "0.14"      # 공통 마크업 파서 유틸리티

# PDF 변환 (hwp2pdf용)
printpdf = "0.7"          # PDF 생성
svg2pdf = "0.12"          # SVG → PDF 변환
```

### Phase 2에서 추가

```toml
[dependencies]
# Python 바인딩 (feature = "python")
pyo3 = { version = "0.22", features = ["extension-module"], optional = true }

[build-dependencies]
maturin = "1.7"           # Python 패키지 빌드 도구
```

### Phase 3에서 추가

```toml
# MCP 서버 (별도 바이너리 또는 feature)
serde_json = "1.0"        # JSON-RPC 메시지 처리
tokio = { version = "1", features = ["io-std", "macros"] }  # async I/O
```

---

## 주간 마일스톤 (상세)

### 2월 3주차 (2/10~16): HTML→HWP 기반

- [ ] HTML 파서 통합 (html5ever + 태그→HWP IR 매핑 설계)
- [ ] 기본 텍스트 변환 (h1~h6, p, br, strong, em, u)
- [ ] 단위 테스트 (HTML 텍스트 → HWP 문단 검증)

### 2월 4주차 (2/17~23): HTML→HWP 표 + hwp2pdf 시작

- [ ] 표 변환 (table, tr, td → HWP Table 컨트롤)
- [ ] 이미지 변환 (img → Picture 컨트롤)
- [ ] hwp2pdf: SVG→PDF 변환기 프로토타입 (단일 페이지)

### 3월 1주차 (2/24~3/2): hwp2pdf 완성 + CLI

- [ ] hwp2pdf: 다중 페이지 PDF 생성
- [ ] hwp2pdf: 폰트 임베딩 처리
- [ ] CLI 통합: `rhwp export-pdf`, `rhwp html-to-hwp`
- [ ] 통합 테스트 (실제 HWP 파일 → PDF 변환 검증)

### 3월 2주차 (3/3~9): PyO3 + PyPI

- [ ] PyO3 바인딩 기본 구조 (html_to_hwp, hwp_to_pdf, open, create)
- [ ] maturin 빌드 설정 (pyproject.toml)
- [ ] 로컬 테스트 (pip install → Python에서 HWP 생성)
- [ ] PyPI 테스트 배포 (TestPyPI)

### 3월 3주차 (3/10~16): MCP 서버 + 배포

- [ ] MCP 서버 구현 (JSON-RPC over stdio, 6개 도구)
- [ ] PyPI 정식 배포
- [ ] npm 패키지 정비 + 배포
- [ ] Claude Code 연동 테스트

### 3월 4주차 (3/17~23): 템플릿 + 문서화

- [ ] 템플릿 매니페스트 시스템
- [ ] 스킬 문서 작성
- [ ] API 문서 (Python/JS/Rust)
- [ ] README + 퀵스타트 가이드

### 3월 마지막 주 (3/24~31): v1.0 릴리스 준비

- [ ] 전체 통합 테스트 (HTML→HWP→PDF 파이프라인)
- [ ] 성능 벤치마크 (변환 속도, 메모리 사용량)
- [ ] 릴리스 노트 작성
- [ ] v1.0 태그 + GitHub Releases

---

## 리스크 및 대응

| 리스크 | 영향 | 대응 방안 |
|--------|------|----------|
| HTML→HWP 매핑 복잡도 예상 초과 | Phase 1 지연 | P0 태그만 우선 구현, P1/P2는 후속 |
| PDF 폰트 임베딩 이슈 | hwp2pdf 품질 저하 | 기본 폰트 번들, 시스템 폰트 폴백 |
| PyO3 크로스 빌드 실패 | 배포 플랫폼 제한 | maturin CI/CD로 멀티 플랫폼 자동 빌드 |
| MCP 프로토콜 변경 | 서버 호환성 깨짐 | 프로토콜 버전 고정, 어댑터 패턴 |
| 한컴의 API 공개 대응 | 경쟁 심화 | 이미 6개월 선점 + 오픈소스 커뮤니티 효과 |

---

## 성공 기준

### 기술적 성공 (3월 말)

- [ ] `pip install rhwp && python -c "import rhwp; rhwp.html_to_hwp('<h1>테스트</h1>', 'test.hwp')"` 동작
- [ ] `rhwp export-pdf sample.hwp output.pdf` 동작
- [ ] Claude Code에서 MCP 도구로 HWP 생성 성공
- [ ] 500개 이상 단위 테스트 통과
- [ ] 한컴오피스에서 생성된 HWP 정상 표시

### 사업적 성공 (6월)

- [ ] PyPI 주간 다운로드 500건 이상
- [ ] GitHub Stars 500개 이상
- [ ] 기업 사용 사례 3건 이상
- [ ] AI 스타트업/SI 기업과 1건 이상 기술 협의

### 장기 성공 (12개월)

- [ ] "HWP 생성은 rhwp"가 한국 개발자 커뮤니티의 상식이 되는 것
- [ ] 연 매출 1억원 이상 (기술지원 + 유료 모듈)

---

*작성: 2026-02-10*
*기반 문서: project_vision.md, bz_model.md*

# rhwp 온보딩 가이드

이 프로젝트는 **Claude Code(AI 에이전트)와 작업지시자(Human)가 협업**하여 개발합니다.
전통적인 개발 워크플로우와 완전히 다른 점이 많으므로, 이 문서를 반드시 숙지하세요.

## 1. 프로젝트 개요

HWP(한글 워드프로세서) 파일을 Rust로 파싱하고 렌더링하는 프로젝트입니다.

```
HWP/HWPX 파일 → Parser → IR(Model) → Paginator → Layout → SVG/Canvas
```

- **네이티브**: CLI로 SVG 내보내기, IR 덤프
- **웹**: WASM 빌드 → rhwp-studio(웹 에디터)에서 문서 보기/편집

## 2. 개발 환경 설정

### 2.1 필수 도구

```bash
# Rust (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Docker (WASM 빌드용)
# → Docker Desktop 또는 Docker Engine 설치

# Node.js (rhwp-studio 웹 에디터)
# → Node.js 18+ 설치
```

### 2.2 빌드 확인

```bash
cargo build              # 네이티브 빌드
cargo test               # 테스트 실행
```

### 2.3 WASM 빌드

```bash
docker compose --env-file .env.docker run --rm wasm
```

### 2.4 웹 에디터 실행

```bash
cd rhwp-studio
npm install
npx vite --host 0.0.0.0 --port 7700
# 브라우저에서 http://localhost:7700
```

## 3. 작업지시자-에이전트 협업 모델

### 3.1 역할

| 역할 | 담당 | 책임 |
|------|------|------|
| **작업지시자** (Human) | 프로젝트 오너 | 타스크 지정, 한컴 기준 검증, 승인/반려, 방향 수정 |
| **에이전트** (Claude Code) | AI 개발자 | 코드 분석, 구현, 테스트, 문서 작성, 디버깅 |

### 3.2 타스크 진행 절차

```
1. 작업지시자: 타스크 지정 (mydocs/orders/에 등록)
2. 에이전트:  수행계획서 작성 → 승인 요청
3. 작업지시자: 승인 또는 수정 지시
4. 에이전트:  구현 계획서 작성 (3~6단계) → 승인 요청
5. 작업지시자: 승인
6. 에이전트:  단계별 구현 → 각 단계 완료보고서 → 승인 요청
7. 작업지시자: 검증 후 승인 또는 피드백
8. 반복...
9. 에이전트:  최종 결과보고서 → 커밋 → devel merge
```

### 3.3 핵심 원칙

- **작업지시자의 지시를 따른다**: 에이전트가 "더 나은 방법"을 제안할 수 있지만, 최종 결정은 작업지시자
- **즉각적 방향 수정**: "원복하세요", "엉뚱한 걸 고쳤네요" → 즉시 원복 후 재분석
- **승인 기반 진행**: 단계별 승인 없이 다음 단계로 넘어가지 않음
- **검증은 작업지시자**: 한컴 워드프로세서와 비교하여 렌더링 정확도 검증

## 4. 디버깅 프로토콜

이 프로젝트의 가장 차별화된 부분입니다. 작업지시자와 에이전트가 **동일한 언어로 문제를 지정**할 수 있는 도구 체계를 갖추고 있습니다.

### 4.1 3단계 디버깅 워크플로우

```
Step 1: export-svg --debug-overlay  → SVG에서 문단/표 식별
Step 2: dump-pages -p N             → 해당 페이지의 배치 목록과 높이 확인
Step 3: dump -s N -p M              → 특정 문단의 IR 상세 조사
```

**코드 수정 없이** 전 과정을 수행할 수 있습니다.

### 4.2 디버그 오버레이 (`--debug-overlay`)

```bash
rhwp export-svg sample.hwp --debug-overlay --output output/
```

SVG에 문단/표의 경계와 인덱스를 시각적으로 표시합니다:

- **문단**: 색상 교대 점선 경계 + `s{섹션}:pi={인덱스} y={좌표}` 라벨 (좌측 상단)
- **표**: 빨간 점선 경계 + `s{섹션}:pi={인덱스} ci={컨트롤} {행}x{열} y={좌표}` 라벨 (우측 상단)

#### 커뮤니케이션 예시

```
작업지시자: "s2:pi=44 문단이 끝난 후, s2:pi=45 표까지의 간격이 너무 큽니다"
에이전트:   "dump 확인 결과, pi=44 vpos_end=14860, pi=45 vpos=15360,
            gap=500 HU(6.7px)이 정상이나 레이아웃에서 172.3px로 출력됩니다.
            shape_reserved가 이중 적용된 것이 원인입니다."
```

양측이 `s2:pi=44`, `y=243.0` 같은 **정량적 식별자**로 소통하므로 모호함이 없습니다.

### 4.3 페이지네이션 결과 덤프 (`dump-pages`)

```bash
rhwp dump-pages sample.hwp -p 15
```

```
=== 페이지 16 (global_idx=15, section=2, page_num=6) ===
  body_area: x=96.0 y=103.6 w=601.7 h=930.5
  단 0 (items=7)
    FullParagraph  pi=41  h=37.3 (sb=16.0 lines=21.3 sa=0.0)  "자료형 설명"
    Table          pi=45 ci=0  16x4  492.2x278.7px  wrap=TopAndBottom tac=false
```

각 문단/표의 **측정된 높이(sb/lines/sa)**를 확인하여 페이지 넘김 오류를 즉시 진단합니다.

### 4.4 IR 덤프 (`dump`)

```bash
rhwp dump sample.hwp -s 2 -p 45
```

```
--- 문단 2.45 --- cc=9, text_len=0, controls=1
  [PS] ps_id=32 align=Justify spacing: before=1000 after=0 line=160/Percent
  [0] 표: 16행×4열
  [0]   [common] treat_as_char=false, wrap=위아래, vert=문단(0=0.0mm)
  [0]   [outer_margin] left=1.0mm top=2.0mm right=1.0mm bottom=7.0mm
```

ParaShape, LINE_SEG, 표 속성을 코드 수정 없이 확인합니다.

### 4.5 시각적 검증 (작업지시자)

작업지시자는 한컴 워드프로세서에서 다음과 같은 방법으로 검증합니다:

1. **글자색 변경**: 머리말/본문/바탕쪽 텍스트에 각각 다른 색을 입혀 렌더링 출처 식별
2. **조판부호 보이기**: 한컴에서 탭, 공백, 줄바꿈 등의 위치 확인
3. **스크린샷 비교**: 한컴 렌더링과 rhwp 렌더링을 나란히 비교
4. **속성 확인**: 한컴에서 개체 속성 → 위치/크기/여백 값을 정확히 전달

#### 검증 커뮤니케이션 예시

```
작업지시자: [스크린샷 첨부]
           "홀수쪽 머리글도 색을 입혔습니다. 다시 1페이지 SVG 내보내기 해보세요"

에이전트:   [SVG 내보내기 후]
           "y=149.3의 수학영역이 #2d629c(파란색)로 변경되었습니다.
            이것은 홀수쪽 머리말에서 온 것입니다. 1페이지에 머리말이 여전히 렌더링됩니다."

작업지시자: "머리말은 pi=41에 정의되어 있고, 2페이지부터 적용되어야 합니다."

에이전트:   [원인 분석 후]
           "pagination 엔진이 머리말을 전체 스캔으로 미리 등록하여
            정의 이전 페이지에도 적용한 것이 원인입니다. 수정합니다."
```

## 5. 피드백 체계

### 5.1 피드백 유형

| 유형 | 예시 | 에이전트 대응 |
|------|------|-------------|
| **즉시 수정** | "원복하세요" | 변경 사항 즉시 `git checkout` |
| **방향 전환** | "이 방식은 해결책이 아닙니다" | 현재 접근 폐기, 새 분석 시작 |
| **정보 제공** | "한컴에서 표 높이: 39.01mm" | 정확한 수치로 비교 검증 |
| **버그 보고** | "s2:pi=365에서 간격 오류" | debug-overlay + dump로 진단 |
| **백로그 등록** | "근본적 해결은 백로그에" | B-XXX로 등록, 현재 타스크 범위 한정 |

### 5.2 피드백 문서

- `mydocs/feedback/` — 피드백 기록
- `mydocs/feedback/r-code-review-*.md` — 1차~4차 코드 리뷰 보고서
- `mydocs/orders/` 백로그 섹션 — 미해결 이슈 추적 (B-001~B-008)

### 5.3 검증 도구별 용도

| 도구 | 누가 사용 | 용도 |
|------|----------|------|
| `export-svg --debug-overlay` | 양측 | 문단/표 위치 식별 |
| `dump-pages -p N` | 에이전트 | 페이지네이션 결과 확인 |
| `dump -s N -p M` | 에이전트 | IR 상세 조사 |
| 한컴 렌더링 비교 | 작업지시자 | 정답 기준 제시 |
| WASM 빌드 + 웹 확인 | 작업지시자 | 최종 렌더링 검증 |

## 6. 코드 구조 이해

### 6.1 핵심 모듈

```
src/
├── model/           # 순수 데이터 구조 (다른 모듈에 의존 없음)
├── parser/          # HWP/HWPX → model 변환
├── document_core/   # 도메인 코어 (CQRS 패턴)
│   ├── commands/    # 상태 변경 (편집)
│   └── queries/     # 상태 조회 (렌더링, 커서)
├── renderer/        # 렌더링 엔진
│   ├── pagination/  # 페이지네이션 (9.5/10 — 모범 코드)
│   ├── layout/      # 레이아웃 (문단, 표, 도형)
│   ├── equation/    # 수식 파서/렌더러
│   └── svg.rs       # SVG 출력
└── wasm_api.rs      # WASM thin wrapper
```

### 6.2 의존성 방향 (반드시 준수)

```
parser → model ← document_core ← renderer
                ↑                ↑
              wasm_api          main.rs
```

- **model은 아무것도 import하지 않는다** (순수 데이터)
- **parser는 model만 안다**
- **renderer는 model과 document_core를 안다**
- **wasm_api/main.rs는 모든 것을 안다** (adapter)

### 6.3 렌더링 파이프라인

```
Model → StyleResolver → Composer → HeightMeasurer → Paginator → LayoutEngine → SVG/Canvas
```

각 단계가 이전 단계의 출력만 사용합니다. 역방향 의존 없음.

## 7. 핵심 도메인 지식

### 7.1 HWPUNIT

```
1인치 = 7200 HWPUNIT
1인치 = 25.4 mm
1 HWPUNIT = 25.4 / 7200 mm ≈ 0.00353 mm
```

코드에서 `hwpunit_to_px(hu, dpi)` 함수로 변환합니다.

### 7.2 주요 속성

| 속성 | 설명 | 조합 |
|------|------|------|
| `treat_as_char` (TAC) | 글자처럼 취급 — 인라인 배치 | 텍스트 흐름 안에 위치 |
| `text_wrap` | TopAndBottom, Square, InFrontOfText, BehindText | 본문과의 배치 관계 |
| `vert_rel_to` | Paper, Page, Para — 세로 기준 | 절대/상대 위치 |
| `horz_rel_to` | Paper, Page, Column, Para — 가로 기준 | 절대/상대 위치 |

### 7.3 HWP 스펙 문서

- `mydocs/tech/hwp_spec_5.0.md` — HWP 5.0 파일 포맷 (주의: 스펙 오류 있음)
- `mydocs/tech/hwp_spec_equation.md` — 수식 스펙
- `mydocs/tech/table_layout_rules.md` — 표 레이아웃 규칙
- `mydocs/tech/equation_support_status.md` — 수식 지원 현황

> **중요**: HWP 스펙 문서에 오류가 있습니다. 항상 실제 바이너리 데이터로 검증하세요.
> 알려진 오류는 CLAUDE.md의 "Known Spec Errors" 섹션 참조.

## 8. E2E 테스트

### 8.1 실행

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700 &   # Vite dev server
node e2e/text-flow.test.mjs              # 개별 테스트 실행
```

### 8.2 모드

| 모드 | 용도 | 실행 환경 |
|------|------|----------|
| `--mode=headless` | CI 자동화 | 로컬/컨테이너 내부 헤드리스 Chrome (Puppeteer 번들) |
| `--mode=host` | 시각 확인 | 호스트 브라우저 (Windows Chrome / macOS Chrome / Linux Chromium) CDP 연결 |

### 8.3 새 E2E 테스트 작성

```javascript
import { launchBrowser, closeBrowser, createPage, loadApp, screenshot, assert } from './helpers.mjs';

const browser = await launchBrowser();
const page = await createPage(browser);
await loadApp(page, 'sample.hwp');

// 검증
const pageCount = await page.evaluate(() => window.hwpDoc?.pageCount());
assert(pageCount >= 1, `페이지 수 1 이상 (${pageCount})`);

await screenshot(page, 'my-test');
await closeBrowser(browser);
```

## 9. Git 워크플로우

### 브랜치 구조

```
main              ← 릴리즈 (태그: v0.5.0 등)
devel             ← 개발 통합
local/task{N}     ← GitHub Issue 번호 기반 타스크 브랜치
```

### 타스크 번호 관리

- **GitHub Issues**로 타스크 번호 자동 채번 (수동 번호 할당 금지)
- **GitHub Milestones**로 타스크 그룹화
- 마일스톤 표기: `M{버전}` (예: M100=v1.0.0, M05x=v0.5.x)

### 진행 절차

```bash
# 1. GitHub Issue 등록
gh issue create --repo edwardkim/rhwp --title "제목" --body "설명" --milestone "v1.0.0"

# 2. 타스크 브랜치 생성
git checkout -b local/task1 devel

# 3. 구현 + 커밋
git commit -m "Task #1: 내용"

# 4. devel에 merge + Issue 종료
git checkout devel && git merge local/task1
git push origin devel    # closes #1 포함 시 자동 종료
# 또는 수동: gh issue close 1

# 5. 릴리즈 (작업지시자 요청 시만)
git checkout main && git merge devel && git push origin main
```

### 오늘할일 표기

`mydocs/orders/yyyymmdd.md`에서 마일스톤+이슈 형식으로 참조:

```markdown
## M100 — 조판 엔진 체계화

| Issue | 타스크 | 상태 |
|-------|--------|------|
| [#1](https://github.com/edwardkim/rhwp/issues/1) | baseline 역공학 | 완료 |
| [#2](https://github.com/edwardkim/rhwp/issues/2) | 편집 갭 수정 | 대기 |
```

## 10. 기여 시 주의사항

1. **model 순수성 유지**: `src/model/`에서 다른 모듈을 import하지 마세요
2. **enum→bool 단순화 금지**: CharShape의 UnderlineType 등을 bool로 바꾸지 마세요
3. **HWP 스펙 신뢰하지 말기**: 실제 바이너리로 반드시 검증
4. **unwrap() 자제**: 특히 parser/renderer에서는 Option/Result 사용
5. **디버그 출력 정리**: eprintln! 등은 커밋 전 제거
6. **한국어 문서**: 모든 문서는 한국어로 작성

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 프로젝트 개요

**목표**: Rust 언어로 HWP 파일 뷰어/에디터 개발
- Rust로 HWP 파일 파서 및 렌더러 구현
- WebAssembly(WASM)로 빌드하여 웹브라우저에서 HWP 문서를 볼 수 있도록 함
- 한컴 웹기안기의 오픈소스 대안

### 파일 포맷별 파서 구조

공통 문서 모델은 `src/model/document.rs`의 `Document` 구조체이다. 모든 포맷 파서는 이 하나의 `Document` IR로 변환하여 반환한다.

| 포맷 | 파서 위치 | 출력 IR |
|------|----------|---------|
| HWPX (ZIP+XML) | `src/parser/hwpx/` | `Document` |
| HWP5 (OLE 복합) | `src/parser/hwp5/` | `Document` |
| HWP3 (고전 바이너리) | `src/parser/hwp3/` | `Document` |

> 역사적으로 `Document` 모델은 HWP5 형식의 구조를 기반으로 설계되었으며, HWPX는 같은 의미의 XML 포맷이다. HWP3는 고전 포맷이지만 동일한 `Document` IR로 변환한다.

**HWP3 파서 규칙**: `src/parser/hwp3/` 내부에서 HWP3 바이너리를 읽어 `Document` IR로 변환하여 반환한다. HWP3 전용 로직은 **반드시 `src/parser/hwp3/` 안에서만** 구현한다. 렌더러(`src/renderer/`), 레이아웃(`src/renderer/layout.rs`), 문서 코어(`src/document_core/`) 등 공통 모듈에 HWP3 전용 분기를 추가하지 않는다.

## 클로드 코드 사용 시 주의사항

이 프로젝트는 **하이퍼-워터폴** 방법론을 적용한다. 클로드 코드의 기본 동작(빠른 실행, 자율 수정)과 충돌이 발생할 수 있으므로 반드시 숙지한다.

상세 내용: [`mydocs/troubleshootings/claude_code_hyperfall_rule_conflict.md`](mydocs/troubleshootings/claude_code_hyperfall_rule_conflict.md)

**핵심 규칙 요약**:
- 소스 수정 전 반드시 작업지시자 승인 요청
- 이슈→브랜치→할일→계획서→구현 순서 절대 생략 금지
- 각 단계 완료 후 승인 없이 다음 단계 진행 금지
- 이슈 클로즈는 작업지시자 승인 후에만 수행

---

## 문서 생성 규칙

모든 문서는 한국어로 작성한다.

문서 폴더 구조 (`mydocs/` 하위):
- `orders/` - 오늘 할일 문서 (yyyymmdd.md)
- `plans/` - 수행 계획서, 구현 계획서
- `plans/archives/` - 완료된 계획서 보관
- `working/` - 단계별 완료 보고서
- `report/` - 기본 보고서
- `feedback/` - 피드백 저장
- `tech/` - 기술 사항 정리 문서
- `manual/` - 매뉴얼, 가이드 문서
- `troubleshootings/` - 트러블슈팅 관련 문서
- `pr/` - **외부 기여자 PR 검토 기록** (내부 타스크와 분리)
- `pr/archives/` - 처리 완료된 PR 보관

### 필수 참조 문서

- `mydocs/manual/browser_extension_dev_guide.md` — 브라우저 확장 개발 가이드 (Safari/Chrome/Edge 보안, UX, 빌드 규칙)
- `mydocs/tech/font_fallback_strategy.md` — 폰트 폴백 전략 (오픈소스 대체, 라이선스)
- `mydocs/report/browser_extension_security_audit.md` — 보안 감사 보고서

### 문서 파일명 규칙

**표준 형식** (`plans/`, `working/`, `report/`):

- 수행 계획서: `task_{milestone}_{이슈번호}.md` (예: `task_m100_71.md`)
- 구현 계획서: `task_{milestone}_{이슈번호}_impl.md` (예: `task_m100_71_impl.md`)
- 단계별 완료 보고서: `task_{milestone}_{이슈번호}_stage{N}.md` (예: `task_m100_71_stage1.md`)
- 최종 보고서: `task_{milestone}_{이슈번호}_report.md` (예: `task_m100_71_report.md`)

**접두어·접미어 규칙:**

- `task_` 접두어는 **필수**. `task_bug_`, `task_feat_` 등 성격별 접두어는 사용하지 않는다.
- 마일스톤은 항상 `m{숫자}` 형식(예: `m100`, `m200`). `m` 없이 숫자만 적거나 생략하지 않는다.
- 한 이슈에 후속 수정이 필요한 경우 `_v2`, `_v3` 등 버전 접미어를 사용한다 (예: `task_m100_71_v2.md`). `_fix`, `_hotfix` 등 의미가 모호한 접미어는 쓰지 않는다.

**폴더 역할 (엄격 준수):**

| 폴더 | 용도 | 비고 |
|------|------|------|
| `orders/` | 오늘 할일 | `yyyymmdd.md`만 허용. 이슈 상세 조사는 `troubleshootings/` 또는 `tech/`로 |
| `plans/` | 수행·구현 계획서 | `_stage{N}`, `_report`는 여기 두지 않는다 |
| `plans/archives/` | 완료된 계획서 보관 | merge 후 정리 시 사용 |
| `working/` | 단계별 완료 보고서 (`_stage{N}.md`) | 최종 보고서는 여기 두지 않는다 |
| `report/` | 최종 결과보고서 (`_report.md`) + 기타 장기 보관 보고서 | **최종 보고서는 반드시 이 폴더** |
| `feedback/` | 작업지시자 피드백, 코드 리뷰 의견 | |
| `tech/` | 기술 조사·분석 | 스펙 정오표, 라이브러리 발견 등 |
| `manual/` | 매뉴얼, 가이드 | 사용자/개발자 문서 |
| `troubleshootings/` | 트러블슈팅 | 재발 방지용 해결 기록 |
| `pr/` | 외부 기여자 PR 검토 기록 | 내부 타스크와 분리 |
| `pr/archives/` | 처리 완료된 PR 보관 | |

### PR 처리 규칙 (`pr/`)

외부 기여자 PR은 내부 타스크와 **다른 본질**을 가지므로 별도 절차와 폴더를 사용한다.

**파일명 형식:**

- 검토 문서: `pr_{번호}_review.md` (수행 계획 + 검토 항목 통합)
- 구현 계획서: `pr_{번호}_review_impl.md` (필요 시)
- 최종 보고서: `pr_{번호}_report.md` (merge/수정요청/close 결정 + 사유)

**PR 처리 절차 (간소화 4단계):**

1. PR 정보 확인 (이슈 연결, base/head, mergeable, CI 상태)
2. `pr_{번호}_review.md` 작성 → 승인 요청
3. (필요 시) `pr_{번호}_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy) + 판단 → `pr_{번호}_report.md` 작성

내부 타스크의 "수행 → 구현 → 단계별 보고 → 최종 보고" 절차는 적용하지 않는다. PR 검토는 본질적으로 **타인 코드를 검증하고 피드백하는 과정**이므로 단계별 보고서(stage)가 불필요하다.

**처리 완료 PR:** `pr/archives/` 로 이동.

## 빌드 및 실행

### 로컬 빌드

```bash
cargo build                    # 네이티브 빌드
cargo test                     # 테스트 실행
cargo build --release          # 릴리즈 빌드
```

네이티브 빌드·테스트·SVG 내보내기는 **항상 로컬 cargo**를 사용한다.

### Docker 빌드 (WASM 전용)

```bash
cp .env.docker.example .env.docker   # 최초 1회: 환경변수 설정
docker compose --env-file .env.docker run --rm wasm    # WASM 빌드 (→ pkg/)
```

Docker는 **WASM 빌드 전용**으로만 사용한다. 네이티브 빌드/테스트에는 사용하지 않는다.

### SVG 내보내기

```bash
rhwp export-svg sample.hwp                         # output/ 폴더에 SVG 출력
rhwp export-svg sample.hwp -o my_dir/              # 지정 폴더에 출력
rhwp export-svg sample.hwp -p 0                    # 특정 페이지만 출력 (0부터)
rhwp export-svg sample.hwp --show-para-marks       # 문단부호(↵/↓) 표시
rhwp export-svg sample.hwp --show-control-codes    # 조판부호 표시 (문단부호+개체마커)
rhwp export-svg sample.hwp --debug-overlay         # 디버그 오버레이 (문단/표 경계+인덱스)
rhwp export-svg sample.hwp --font-style            # @font-face local() 참조 삽입
rhwp export-svg sample.hwp --embed-fonts           # 폰트 서브셋 임베딩 (사용 글자만)
rhwp export-svg sample.hwp --embed-fonts=full      # 폰트 전체 임베딩
rhwp export-svg sample.hwp --font-path ~/fonts     # 폰트 파일 탐색 경로 (여러 번 지정 가능)
```

#### 폰트 임베딩 옵션

| 옵션 | SVG 크기 | 오프라인 | 설명 |
|------|---------|---------|------|
| (없음) | 최소 | ❌ | CSS font-family 체인만 |
| `--font-style` | +수 KB | ❌ | `@font-face { src: local("폰트명") }` 참조 |
| `--embed-fonts` | +수십~수백 KB | ✅ | 사용 글자만 서브셋 추출 + base64 |
| `--embed-fonts=full` | +수 MB | ✅ | 전체 폰트 base64 |

`--font-path`로 TTF/OTF 파일 탐색 경로를 지정한다. 여러 번 지정 가능하며 기본 탐색 경로(`ttfs/`, 시스템 폰트)보다 우선한다.

#### 디버그 오버레이 (`--debug-overlay`)

문단/표의 경계와 인덱스를 SVG에 시각적으로 표시한다.

- **문단**: 색상 교대 점선 경계 + `s{섹션}:pi={인덱스} y={좌표}` 라벨 (좌측 상단)
- **표**: 빨간 점선 경계 + `s{섹션}:pi={인덱스} ci={컨트롤} {행}x{열} y={좌표}` 라벨 (우측 상단)
- 셀 내부 문단, 머리말/꼬리말/바탕쪽/각주 영역은 제외

#### 페이지네이션 결과 덤프 (`dump-pages`)

특정 페이지의 문단/표 배치 목록과 높이를 확인한다.

```bash
rhwp dump-pages sample.hwp -p 15    # 페이지 16 (0부터) 배치 결과
```

출력 예시:
```
=== 페이지 16 (global_idx=15, section=2, page_num=6) ===
  body_area: x=96.0 y=103.6 w=601.7 h=930.5
  단 0 (items=7)
    FullParagraph  pi=41  h=37.3 (sb=16.0 lines=21.3 sa=0.0)  "자료형 설명"
    Table          pi=45 ci=0  16x4  492.2x278.7px  wrap=TopAndBottom tac=false
```

### IR 덤프 (`dump`)

문서의 조판부호 구조를 덤프한다. 섹션/문단 필터를 지정하여 특정 문단의 ParaShape, LINE_SEG, 표 속성을 확인할 수 있다.

```bash
rhwp dump sample.hwp                  # 전체 구조 덤프
rhwp dump sample.hwp -s 2 -p 45      # 섹션 2, 문단 45만 덤프
```

출력 예시:
```
--- 문단 2.45 --- cc=9, text_len=0, controls=1
  [PS] ps_id=32 align=Justify spacing: before=1000 after=0 line=160/Percent
       margins: left=7000 right=4000 indent=0 border_fill_id=1
  ls[0]: vpos=15360, lh=1000, th=1000, bl=850, ls=600, cs=3500, sw=0
  [0] 표: 16행×4열
  [0]   [common] treat_as_char=false, wrap=위아래, vert=문단(0=0.0mm)
  [0]   [outer_margin] left=1.0mm top=2.0mm right=1.0mm bottom=7.0mm
```

### IR 비교 (`ir-diff`)

동일 문서의 HWPX와 HWP 파일을 파싱하여 IR 차이를 자동 검출한다.

```bash
rhwp ir-diff sample.hwpx sample.hwp                    # 전체 비교
rhwp ir-diff sample.hwpx sample.hwp -s 0 -p 810        # 특정 문단만 비교
rhwp ir-diff sample.hwpx sample.hwp 2>&1 | grep "\[PS " # ParaShape 차이만
rhwp ir-diff sample.hwpx sample.hwp 2>&1 | tail -1      # 차이 건수만
rhwp ir-diff sample.hwpx sample.hwp --summary           # 카테고리별 카운트
rhwp ir-diff sample.hwpx sample.hwp --max-lines 50      # 출력 50줄 제한
```

비교 항목: text, char_count, char_offsets, char_shapes, line_segs, controls(타입+속성), tab_extended, ParaShape(여백/줄간격/탭), TabDef(위치/종류/채움).
표: page_break, outer_margin, treat_as_char, wrap, size, v_offset/h_offset 비교.
그림/도형: treat_as_char, wrap, size, v_offset/h_offset, vert_rel/horz_rel 비교.

상세 매뉴얼: `mydocs/manual/ir_diff_command.md`

### 디버깅 워크플로우

레이아웃/간격 버그 디버깅 시 다음 순서로 진행한다:

1. `export-svg --debug-overlay` → SVG에서 문단/표 식별 (`s{섹션}:pi={인덱스} y={좌표}`)
2. `dump-pages -p N` → 해당 페이지의 문단 배치 목록과 높이 확인
3. `dump -s N -p M` → 특정 문단의 ParaShape, LINE_SEG, 표 속성 상세 조사

HWPX↔HWP 불일치 디버깅 시 추가 단계:

4. `ir-diff sample.hwpx sample.hwp` → IR 차이 자동 검출
5. HWPX XML 원본 확인 (header.xml / section0.xml)

코드 수정 없이 전 과정 수행 가능하다.

### HWPUNIT

- 1인치 = 7200 HWPUNIT
- 1인치 = 25.4 mm

### 예제 폴더

- `samples/` - 테스트용 HWP/HWPX 파일 (git tracked 영구 보존)
- `pdf/` - 한글 **2022** 편집기 PDF 변환본 (PR #670, 시각 정합성 비교 권위 자료, < 50 MB)
- `pdf-2020/` - (예정) 한글 2020 편집기 PDF 변환본 (< 50 MB)
- `pdf-2010/` - (예정) 한글 2010 편집기 PDF 변환본 (< 50 MB)
- `pdf-large/` - **대용량 PDF (≥ 50 MB, Git LFS 추적)** — GitHub 권장 50 MB 초과 PDF 영역 영역 격리 (PR #753, hwp3-sample10 영역)

### PDF 권위 자료 명명 규약

| 폴더 | 한컴 버전 | 명명 패턴 | 처리 |
|------|----------|----------|------|
| `pdf/` | 한글 2022 | `pdf/{원본 stem}-2022.pdf` | 일반 git |
| `pdf-2020/` | 한글 2020 | `pdf-2020/{원본 stem}-2020.pdf` | 일반 git |
| `pdf-2010/` | 한글 2010 | `pdf-2010/{원본 stem}-2010.pdf` | 일반 git |
| `pdf-large/` | 모든 버전 | `pdf-large/{원본 stem}-{버전}.pdf` | **Git LFS** |

원본 파일이 하위 폴더 (`samples/basic/` / `samples/hwpx/`) 에 있는 경우 PDF 도 동일 하위 폴더 구조 유지. 상세는 `pdf/README.md` / `pdf-large/README.md`.

50 MB 초과 PDF 는 반드시 `pdf-large/` 영역 영역 배치 — `.gitattributes` 의 `pdf-large/**/*.pdf filter=lfs` 패턴 영역 영역 자동 LFS 변환. Clone / Fork 시 LFS 미설치 환경 영역 영역 placeholder 만 진입 영역 영역, 실제 PDF 영역 영역 `git lfs install && git lfs pull` 영역 영역 받음.

### PDF 권위 등급 (컨트리뷰터 환경별)

본 프로젝트의 시각 판정 권위 영역은 컨트리뷰터 환경에 따라 다르다 (`reference_authoritative_hancom` 메모리 룰 정합):

**Windows + 한컴 편집기 환경**:
- 1차 정답지: 한글 2010 / 2020 / 2022 **편집기** 직접 출력 (시각 판정)
- 보조: `pdf/`, `pdf-2020/`, `pdf-2010/` 의 PDF

**macOS / Linux 환경 (한컴 편집기 미접근)**:
- 1차 정답지: `pdf/` (한글 2022) 또는 `pdf-2020/` (한글 2020) PDF
- 등급 미달: `pdf-2010/` (한글 2010 PDF) — 보조 자료, 정답지 등급 미달

**모든 환경 공통 — 정답지 아님**:
- 한컴 뷰어 출력
- macOS 인쇄 / 외부 변환
- HWP5 v2024 변환본 등 한컴 변환 산출물 (비교 보조 자료)

### 출력 폴더

`output/` 하위를 용도별 서브폴더로 분리한다. `.gitignore`에 등록되어 있으므로 Git에 포함되지 않음.

| 폴더 | 용도 |
|------|------|
| `output/re/` | 재현검증용 샘플 (`re_sample_gen.rs` 테스트 자동 생성) |
| `output/svg/` | SVG 내보내기 기본 출력 (`rhwp export-svg`) |
| `output/debug/` | 디버그 오버레이 HTML (`rhwp export-svg --debug-overlay`) |

### E2E 테스트

E2E 테스트는 Puppeteer (puppeteer-core) 기반이며, 두 가지 모드로 실행할 수 있다.

#### headless Chrome (자동화용)

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700 &   # Vite dev server
node e2e/text-flow.test.mjs              # 텍스트 플로우 테스트
```

#### 호스트 Chrome CDP (시각 확인용)

1. Chrome 실행 (원격 디버깅 활성화):
```
chrome --remote-debugging-port=9222 --remote-debugging-address=0.0.0.0 --remote-allow-origins=*
```

2. 테스트 실행:
```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700 &
node e2e/text-flow.test.mjs --mode=host
```

## rhwp-studio UI 명칭 규약

코드와 대화에서 혼동을 방지하기 위해, 아래 명칭을 통일하여 사용한다.

```
┌─────────────────────────────────────────────────┐
│  메뉴바 (#menu-bar)                              │
│  파일 | 편집 | 보기 | 입력 | 서식 | 쪽 | 표      │
├─────────────────────────────────────────────────┤
│  도구 상자 (#icon-toolbar)                        │
│  [오려두기][복사][붙이기] | [글자모양][문단모양] | … │
├─────────────────────────────────────────────────┤
│  서식 도구 모음 (#style-bar)                      │
│  [스타일▼][글꼴▼][크기] | 가가간가 | ◀ ≡ ▶ ≡≡ | ⇕  │
├─────────────────────────────────────────────────┤
│                                                 │
│  편집 영역 (#scroll-container)                    │
│                                                 │
├─────────────────────────────────────────────────┤
│  상태 표시줄 (#status-bar)                        │
│  1/1쪽 | 구역:1/1 | 삽입 |           100% [−][+] │
└─────────────────────────────────────────────────┘
```

| 한국어 명칭 | HTML id/class | 설명 |
|------------|---------------|------|
| 메뉴바 | `#menu-bar` | 최상단 드롭다운 메뉴 (파일/편집/보기/입력/서식/쪽/표) |
| 도구 상자 | `#icon-toolbar` | 아이콘+라벨 버튼 모음 (tb-btn, tb-group) |
| 서식 도구 모음 | `#style-bar` | 스타일/글꼴/크기/서식 버튼 (sb-btn, sb-combo) |
| 편집 영역 | `#scroll-container` | 문서 페이지 렌더링 + 스크롤 영역 |
| 상태 표시줄 | `#status-bar` | 하단 쪽/구역/모드/줌 표시 |

### CSS 접두어 규칙

| 접두어 | 대상 |
|--------|------|
| `tb-` | 도구 상자 (#icon-toolbar) 요소 |
| `sb-` | 서식 도구 모음 (#style-bar) 요소 |
| `stb-` | 상태 표시줄 (#status-bar) 요소 |
| `md-` | 메뉴바 드롭다운 (#menu-bar) 요소 |
| `dialog-` | 대화상자 공통 |
| `cs-` | 글자모양 대화상자 (char-shape) |
| `ps-` | 문단모양 대화상자 (para-shape) |

## 워크플로우

### 브랜치 관리

| 브랜치 | 용도 |
|--------|------|
| `main` | 최종 릴리즈. 태그(v0.5.0 등)로 안정 버전 보존 |
| `devel` | 개발 통합 |
| `local/devel` | devel 브랜치의 로컬 작업 브랜치. 작업 완료 후 devel에 merge |
| `local/task{num}` | 타스크별 작업 |

### Git 워크플로우

```
local/task{N}  ──커밋──커밋──┐
local/task{N+1}──커밋──커밋──┤
                              ├─→ local/devel merge (작업 단위)
                              │
                              ├─→ devel merge (로컬) + push
                              │
                              ├─→ main PR 생성 + 리뷰 + merge + 태그 (릴리즈 시점)
```

- **타스크 브랜치**: `local/task{N}`에서 잘게 커밋. 작업 단위마다 커밋.
- **local/devel 작업**: devel에서 직접 작업하지 않고 `local/devel` 브랜치에서 작업한다. 타스크 브랜치도 `local/devel`에서 분기하고 `local/devel`로 merge한다.
- **원격 push**: `devel`만 push. `local/devel`과 `local/task` 브랜치는 **로컬 유지 (원격 push 금지)**.
- **main merge (PR 기반)**: 릴리즈 시점에 `devel` → `main` PR 생성 → 리뷰(approve) → merge 후 태그 생성.

#### 메인테이너 워크플로우

```bash
# 1. local/devel → devel (로컬 merge + push)
git checkout devel
git merge local/devel --no-ff -m "Merge local/devel: 제목"
git push origin devel

# 2. devel → main PR (릴리즈 시)
gh pr create --base main --head devel --title "Release: 제목"
gh pr review --approve
gh pr merge --merge --delete-branch=false
```

#### 컨트리뷰터 워크플로우 (Fork 기반)

```bash
# 1. 원본 저장소 Fork (GitHub에서 1회)
# 2. Fork한 저장소에서 작업
git clone https://github.com/{contributor}/rhwp.git
git checkout -b feature/my-task
# ... 작업 + 커밋 ...
git push origin feature/my-task

# 3. 원본 저장소의 devel로 PR 생성
gh pr create --repo edwardkim/rhwp --base devel --head {contributor}:feature/my-task --title "제목"

# 4. 메인테이너가 리뷰 + merge
```

### 타스크 번호 관리

- **GitHub Issues**를 타스크 번호로 사용한다. 자동 채번으로 중복 방지.
- **마일스톤 표기**: `M{버전}` (예: M100=v1.0.0, M05x=v0.5.x)
- 새 타스크 등록: `gh issue create --repo edwardkim/rhwp --title "제목" --body "설명" --milestone "v1.0.0"`
- 브랜치명: `local/task{issue번호}` (예: `local/task1`)
- 커밋 메시지: `Task #1: 내용` (Issue 번호 참조)
- `mydocs/orders/`에서 `M100 #1` 형식으로 마일스톤+이슈 참조
- 타스크 완료 시: `gh issue close {번호}` 또는 커밋 메시지에 `closes #번호`

### 타스크 진행 절차

1. GitHub Issue에 타스크 등록 → 작업지시자가 지정한 타스크 수행
2. `local/task{issue번호}` 브랜치 생성 후 진행
3. 수행 전 수행계획서 작성 → 승인 요청
4. 구현 계획서 작성 (최소 3단계, 최대 6단계) → 승인 요청
5. 단계별 진행 시작
6. 각 단계 완료 후 단계별 완료보고서 작성 → 승인 요청
7. **단계별 완료보고서(`_stage{N}.md`)는 해당 단계 소스 커밋과 함께 타스크 브랜치에서 커밋한다.**
8. 승인 후 다음 단계 진행
9. 모든 단계 완료 시 최종 결과 보고서 작성 → 승인 요청
10. **최종 결과보고서(`_report.md`)와 오늘할일(`orders/`) 갱신도 타스크 브랜치에서 커밋한다. merge 전 반드시 `git status`로 미커밋 파일이 없는지 확인한다.**
11. 승인 요청 시 작업지시자가 피드백 문서를 `mydocs/feedback/`에 등록
12. 모든 테스트 통과 시 피드백 없음
13. 최종 결과보고서 작성 후 오늘할일 해당 타스크 상태 갱신

### 작업 규칙

- 작업 시간의 시작과 종료는 작업지시자가 결정한다. 클로드가 임의로 작업 종료를 제안하거나 시간을 한정하지 않는다.

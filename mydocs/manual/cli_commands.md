---
kind: canonical
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-07-29
---

# rhwp CLI 명령어 매뉴얼

`rhwp` 바이너리의 전체 명령을 정리한다. 권위 출처는 `src/main.rs` 의 명령 디스패치이며,
`rhwp --help` 와 본 문서를 함께 현행화한다.

```
rhwp <명령> [옵션]
rhwp --help        # 도움말
rhwp --version     # 버전
```

> 빌드: `cargo build --release` 후 `./target/release/rhwp`, 또는 개발 중 `cargo run --bin rhwp -- <명령>`.
> 네이티브 빌드/실행은 항상 로컬 cargo 사용(Docker 는 WASM 전용).

공통 옵션(다수 export 명령):
- `-o, --output <폴더>` — 출력 폴더 (기본 `output/`)
- `-p, --page <번호>` — 특정 페이지만 (0부터). 생략 시 전체
- `--profile <프로필>` — 출력 프로필: `screen` | `print` | `high-quality` | `fast-preview`
  (export-svg / export-png / export-pdf 지원, #2297)
- 옵션은 파일 앞뒤 어디에 와도 된다 (#3359 — export-svg/png/pdf/markdown/render-tree/
  doclang, export-structure/export-tables 와 동일 규약). 파일 positional 을 두 번 주면 exit 2.

**프로필 의미론** — 편집 시각 요소(#2225 그림 미지정 placeholder 등)의 표시 여부를 가른다:

| 프로필 | 편집 시각 요소 | 용도 |
|--------|---------------|------|
| `screen`, `fast-preview` | **표시** — 그림 미지정 placeholder 를 점선 테두리+아이콘으로 렌더 | 편집기/미리보기 등가 |
| `print`, `high-quality` | **억제** — 한컴 인쇄 동작과 동일하게 미출력 | 인쇄 등가 산출물 |

> 한컴은 그림 미지정 placeholder 를 편집기에서만 표시하고 인쇄(및 인쇄 등가
> 출력)에서는 미출력한다 — rhwp 의 인쇄 등가 프로필이 이 계약을 따른다.

## 비밀번호 보호 HWP

HWP5 FileHeader의 `encrypted` 플래그와 `EncryptVersion=4`가 설정된 문서, 또는
압축된 HWP3 암호 문서는 전역 비밀번호 옵션으로 연다. 일반 HWPX 읽기와
암호화 HWPX 복호화는 별도 기능이며, 현재 입력 지원 상태는 다음과 같다.

| 입력 형식 | 현재 상태 | CLI 동작 |
|-----------|-----------|----------|
| 암호화되지 않은 HWP5 | 지원 | 기존 명령 사용 |
| HWP5 비밀번호 암호화, EncryptVersion 4 | 읽기 지원 | `--password` 또는 `--password-stdin` 필요 |
| HWP3 비밀번호 암호화, 압축 본문 | 읽기 지원 | `--password` 또는 `--password-stdin` 필요 |
| HWP3 비밀번호 암호화, 비압축 본문 | 미지원 | 비밀번호를 시도해도 종료 코드 1 |
| HWP5 EncryptVersion 1~3 | 미지원 | 비밀번호를 시도하지 않고 종료 코드 1 |
| 암호화되지 않은 HWPX | 지원 | 기존 HWPX 파서 사용, 비밀번호 옵션 불필요 |
| 암호화 HWPX(ODF `encryption-data`, AES-256-CBC/PBKDF2) | 읽기 지원 | `--password` 또는 `--password-stdin` 필요 |
| 암호화 HWPX(그 외 ODF 암호화 계약) | 미지원 | 지원하지 않는 암호화 방식으로 종료 코드 1 |
| DRM(Fasoo/SoftCamp 등) | 미지원 | 비밀번호 암호화와 다른 보호 방식 |

```bash
# 권장: 비밀번호를 프로세스 인자에 남기지 않는다.
rhwp info protected.hwp --password-stdin < password.txt

# 간편 방식: 셸 기록이나 프로세스 목록에 값이 노출될 수 있다.
rhwp --password '문서비밀번호' export-text protected.hwp -o output/
```

- `--password <값>`과 `--password-stdin`은 명령 앞뒤 어느 위치든 한 번만 지정할 수 있다.
- 비밀번호가 없으면 종료 코드 2, 틀리면 종료 코드 1이다. 지원하지 않는 HWP5
  EncryptVersion 또는 비압축 HWP3 암호 본문은 종료 코드 1로 거부한다.
- 일반 열기·내보내기·변환 명령과 `dump-records`가 이 옵션을 사용한다.
  `export-doclang`의 보호 문서 거부 정책과 미리보기만 읽는 `thumbnail`에는 적용하지 않는다.
- `convert`와 `export-hwpx`는 `--output-password <값>` 또는 `--output-password-stdin`을 받으면
  각각 HWP5 EncryptVersion 4와 ODF AES-256-CBC/PBKDF2 HWPX로 저장한다. 입력 암호와 출력 암호는
  독립적이므로 복호화·재암호화가 가능하다.
- 두 stdin 옵션을 함께 쓰면 stdin 첫 줄은 입력 암호, 둘째 줄은 출력 암호다. `--output-password`는
  프로세스 목록과 shell history에 값이 남을 수 있으므로 `--output-password-stdin`을 권장한다.
- 출력 암호는 `convert`/`export-hwpx`의 `--verify`, `--verify-pages` 재열기에도 사용한다. PDF와
  기타 내보내기 명령에는 파일 형식 암호 저장을 제공하지 않는다.

## 종료 코드 (#2707)

스크립트·CI·에이전트가 성공 여부를 판정하는 계약이다.

| 코드 | 의미 | 예 |
|---:|---|---|
| 0 | 성공 | 요청한 페이지를 모두 내보냄 |
| 1 | 런타임 실패 — 읽기·파싱·렌더·쓰기 | 입력 파일 없음, 파싱 실패, 출력 저장 실패 |
| 2 | 사용법 오류 — 인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과 | `rhwp export-svg` (인자 없음), `--fontpath` 오타 |
| 3 | IR 차이 검출 | `convert` / `export-hwpx` 의 `--verify` (아래 §3), `ir-diff --json` (#3274) |
| 4 | `--verify-pages` 페이지 수 불일치 | `convert` / `export-hwpx` 전용 (아래 §3) |

- 알 수 없는 명령·옵션은 **경고 후 진행하지 않고** 즉시 2로 끝난다. 안내는 stderr 로 나간다.
- 페이지 단위 내보내기 명령의 "N개 … 완료" 메시지는 **실제로 저장에 성공한 개수**다.
  한 장이라도 실패하면 종료 코드는 1이다.
- `export-png` 는 `native-skia` feature 없이 빌드된 바이너리에서 2로 끝난다(기능 부재).

---

## 1. 내보내기 (Export)

### `export-svg <파일> [옵션]`
HWP/HWPX → SVG.
- `--json` (#3287): 산출물 **매니페스트**를 stdout 에 JSON 으로 출력한다(렌더 동작 무변경).
  `{"schemaVersion":"1.0","source","format":"svg","outputDir","pageCount","renderedCount","overflowCellLines","pages":[{"page","path","bytes","overflowCellLines"}]}`
  기본 출력(사람용 진행 메시지)은 무변경이며, `--json` 모드에서는 stdout 에 JSON 만 나간다.
  `search --json`(#3283)과 조합하면 **찾은 페이지만 렌더해 VLM 에 넘기는** 루프가 닫힌다.
  - `overflowCellLines` (#3668): 셀 안 줄의 윗변이 쪽 하단 밖에 그려져 **보이지 않는 줄 수**
    (`LAYOUT_OVERFLOW_CELL` 진단과 같은 조건). top-level 은 문서 합계, `pages[]` 항목은
    페이지별 카운트다. 0 이 아니면 그 페이지의 셀 콘텐츠 일부가 소실 렌더된 것이다 —
    #3236 계열(분할 대신 통짜 배치 후 clip) 조사의 1차 신호. 원장 게이트는
    [`local_validation.md` 4.3.1](pr_review/local_validation.md#431-새-hwphwpx-fixture의-baseline-등록--ir-sweep--overflow-cell-원장) 참조.
- `-o`, `-p` (공통)
- `--show-para-marks` — 문단부호(↵/↓)
- `--show-control-codes` — 조판부호(문단부호 + 개체 마커)
- `--debug-overlay` — 디버그 오버레이(문단/표 경계 + 인덱스 라벨)
- `--respect-vpos-reset` — LINE_SEG vpos=0 리셋을 단/페이지 강제 경계로 처리
- `--show-grid[=Nmm]` — 격자 오버레이(기본 1mm, 예 `--show-grid=3mm`)
- `--grid-origin=X,Y|auto` — 격자 종이 기준 위치(예 `--grid-origin=15mm,20mm`)
- `--font-style` — `@font-face local()` 참조 삽입(폰트 데이터 미포함)
- `--embed-fonts` — 폰트 서브셋 임베딩(사용 글자만 base64)
- `--embed-fonts=full` — 폰트 전체 임베딩
- `--font-path <경로>` — 폰트 탐색 경로(여러 번 지정 가능)
- `--profile <프로필>` — layer 출력 프로필(공통 옵션 참조). 생략 시 기존
  (legacy) 경로 — 인쇄 등가 억제 동작.
  **제약**: `--font-style`/`--embed-fonts` 와 함께 사용할 수 없다(오류 종료).

### `export-png <파일> [옵션]` *(native-skia feature 필요)*
HWP/HWPX → PNG(Skia raster, AI 파이프라인/VLM 연동). 상세: [export_png_command.md](export_png_command.md)
- `-o`, `-p`, `--font-path` (공통/폰트)
- `--scale <배율>` (기본 1.0), `--dpi <값>`(pHYs 메타 + scale 자동), `--max-dimension <픽셀>`(longest edge)
- `--vlm-target <프리셋>` — claude / gpt4v-low / gpt4v-high(gpt4v) / gemini / qwen-vl(qwen) / llava
- `--profile <프로필>` — 출력 프로필. **기본 `high-quality`(인쇄 등가)** —
  그림 미지정 placeholder 는 억제된다. 편집기식 표시가 필요하면
  `--profile screen` 을 명시한다 (#2297, #2225 계약).

### `export-pdf <파일> [옵션]`
HWP/HWPX → PDF (svg2pdf + pdf-writer).
- `--json` (#3596): 산출물 매니페스트를 stdout 에 JSON 으로 출력한다(렌더 동작 무변경).
  `{"schemaVersion":"1.0","source","format":"pdf","backend","output","bytes","pageCount","renderedCount"}`
  실패 경로의 stdout 은 비운다(export-svg 규약).
- `-o <파일>`, `--output <파일>` — 출력 PDF 파일(기본 `output/<입력명>.pdf`)
- `-p <번호>`, `--page <번호>` — 0-based 단일 페이지 선택. 생략하면 전체 문서를 다중 페이지 PDF로 내보낸다.
- `--font-path <경로>` — PDF 변환 fontdb에 추가할 폰트 탐색 경로(여러 번 지정 가능)
  - 환경변수 `RHWP_FONT_PATH` 로도 지정할 수 있다(#2864). 복수 경로는 OS 관례
    구분자로 나눈다(유닉스 `:`, Windows `;`). 백엔드에서 대량 변환할 때 호출마다
    `--font-path` 를 붙이는 대신 한 번만 설정하면 된다.
  - 조달 순서: `--font-path` → `RHWP_FONT_PATH` → 시스템 설치 폰트 →
    저장소 번들 `ttfs/opensource`(최후 폴백, 한국어 드롭 방지).
  - **폰트를 지정하지 않으면 산출물이 달라진다.** 문서가 쓰는 폰트(한컴 바탕/돋움,
    Windows 폰트 등)가 시스템에 없으면 번들 대체 폰트(Noto Sans/Serif KR)로 떨어져
    글꼴이 바뀐다. 서버·컨테이너에서 대량 변환할 때는 **필요한 폰트를 설치하고
    `--font-path` 또는 `RHWP_FONT_PATH` 로 명시**해야 정본과 같은 결과를 얻는다.
- `--backend <svg|direct>` — PDF backend(기본값: svg). `svg`는 기존 SVG-derived 경로,
  `direct`는 `PageLayerTree → PDF` direct/vector 경로. `direct`는 `native-skia` feature로
  빌드한 native CLI가 필요하며, 해당 feature 없이 빌드된 CLI에서 `--backend direct`를 쓰면
  종료코드 1과 함께 오류 메시지를 반환한다.
- `--raster-dpi <DPI>` — `direct` backend fallback raster DPI(기본값: 144). `direct` backend
  에서만 사용할 수 있다.
- `--fallback-serif <family>` — PDF serif generic fallback family
- `--fallback-sans <family>` — PDF sans-serif generic fallback family
- `--fallback-mono <family>` — PDF monospace generic fallback family
- `--equation-font <family>` — PDF 수식 SVG의 우선 font-family
- `--text-as-paths` — 텍스트를 폰트 임베드 대신 path 로 변환 (#2266).
  폰트 서브셋 경로를 건너뛰어 **메모리를 크게 절감**(실측 예: 124→78 MB)
  하는 대신 **PDF 의 텍스트 선택·검색 기능을 잃는다** (시각 출력 동일,
  파일 크기는 증가). 저메모리 환경(Quick Look 등)용 옵트아웃.
- `--profile <프로필>` — layer 출력 프로필(공통 옵션 참조). 생략 시 기존
  (legacy) 경로.
- `<파일>`, `<경로>`, `<family>`는 자리표시자이며 실제 입력에는 꺾쇠괄호를 쓰지 않는다.
- 공백이 없는 값은 그대로 입력한다. 예: `--font-path ./ttfs`
- 공백이 있는 경로/폰트명은 큰따옴표를 권장한다. 예:

```bash
rhwp export-pdf input.hwp -o out.pdf \
  --font-path "./My Fonts" \
  --fallback-serif "Noto Serif CJK KR" \
  --fallback-sans "Noto Sans CJK KR" \
  --fallback-mono "Noto Sans Mono CJK KR" \
  --equation-font "STIX Two Math"
```

- 작은따옴표(`'...'`)는 zsh/bash/PowerShell에서 변수 확장 없이 literal 값을 넘길 때만 사용한다.
  Windows `cmd.exe` 호환 예시는 큰따옴표(`"..."`)를 사용한다.
- `DocumentCore::render_page_pdf_native`, `render_pages_pdf_native`, `render_document_pdf_native`
  native API와 같은 SVG-derived PDF export 경로를 사용한다.
- fallback family 옵션 미지정 시 OS별 기본값을 사용한다.
  - Windows: `바탕` / `맑은 고딕` / `D2Coding`
  - Linux: `Noto Serif CJK KR` / `Noto Sans CJK KR` / `Noto Sans Mono CJK KR`
  - macOS: `AppleMyungjo` / `Apple SD Gothic Neo` / `Menlo`
- 선택한 fallback family 또는 수식 폰트가 fontdb에 없으면 warning을 출력한다.
- direct/vector `PageLayerTree → PDF` backend는 `--backend direct`로 이미 사용 가능하다
  (`native-skia` feature 빌드 필요, 위 옵션 설명 참고).

### `export-text <파일> [옵션]`
페이지별 텍스트 → TXT. `-o`, `-p`.
- `--json` (#3237): 파일 저장 대신 stdout 에 순수 JSON 하나를 출력. 진행 메시지 없음.
  `{"schemaVersion":"1.0","source","pageCount","truncated","omittedCount","pages":[{"page","text"}]}` —
  `schemaVersion` 이 계약이며 필드 추가는 허용, 변경·삭제는 `tests/cli_json_contract.rs` 가 잡는다.
  `page` 는 `-p` 와 같은 0 기준.
- `--max-chars <N>` (#3787 S7): 본문 문자 상한. **기본은 무제한**이고, `--json` 과
  함께 써야 한다(파일 저장 모드에는 절단 사실을 실을 봉투가 없어 exit 2 로 거부).
  거대 문서가 에이전트 컨텍스트를 밀어내는 것을 막는 용도다.
  - **조용히 자르지 않는다** — 최상위 `truncated:true` 와 `omittedCount`(생략 문자 수)를
    싣고, 잘린 쪽마다 `pages[].truncated`·`pages[].omittedCount` 를 붙인다.
  - **쪽 주소를 보존한다** — 예산이 떨어져도 `pages[]` 에서 항목을 빼지 않는다.
    빼면 `pageCount` 가 줄어 문서가 실제보다 짧아 보인다.
  - `0`·음수·비정수는 사용법 오류(exit 2)다. `0` 을 무제한으로 뭉개면 "아무것도 주지
    마라"는 요청이 "전부 달라"로 뒤집힌다.
  - 계약 근거는 [에이전트 경계 무결성 계약](../tech/agent_boundary_contract.md) S7.

```bash
# 처음 보는 대형 문서를 컨텍스트 예산 안에서 훑기
rhwp export-text 편람.hwp --json --max-chars 4000 | jq '{truncated, omittedCount}'
```
- 옵션은 파일 앞뒤 어디에 와도 된다 (#3349, export-structure/export-tables 와 동일 규약).
  파일 positional 을 두 번 주면 exit 2.

### `batch <export-text|info|export-structure|export-tables|fields|search|convert> --json [옵션]` (#3238, #3261, #3346, #3626)
stdin 의 파일 목록(한 줄당 경로 하나)을 **한 프로세스에서 파일 간 병렬**로 처리해
NDJSON(한 줄당 레코드 하나)을 stdin 입력 순서대로 스트림 출력한다.
- `batch export-text` 성공 레코드: `{"schemaVersion":"1.0","source","pageCount","text"}`
- `batch info` 성공 레코드: `info --json` 과 **같은 스키마** — 단건/배치를 같은 소비 코드로 읽는다
- `batch export-structure` 성공 레코드: `export-structure --json` 봉투와 같은 스키마.
  `--mode auto|outline|clause` 는 이 축 전용(기본 auto)
- `batch export-tables` 성공 레코드: `export-tables --json` 봉투와 같은 스키마
  (병합 `rowSpan`/`colSpan`·중첩 표 보존)
- `batch fields` 성공 레코드: `fields --json` 봉투와 같은 스키마
- `batch search` 성공 레코드: `search --json` 봉투와 같은 스키마.
  **`--query <검색어>` 는 이 축 전용이며 필수**다(없으면 사용법 오류 2).
  대량 코퍼스에서 스트림이 부풀지 않도록 **파일당 매치 1,000건 상한**을 둔다
  (단건 `search --limit` 과 같은 취지). 대소문자는 구분한다.
- `batch convert` 는 **CLI 전용 쓰기 축**이다. `--out-dir <폴더>`가 필수이고,
  `--verify`·`--verify-pages`를 선택할 수 있다. 입력마다 `<out-dir>/<입력이름>.hwp`를
  한 번만 쓴 뒤 단건 `convert --json`과 같은 봉투를 NDJSON으로 낸다. MCP `hwp_batch`에는
  쓰기 산출물 계약을 아직 노출하지 않는다.
- convert는 쓰기 전에 모든 산출 이름을 예약한다. 같은 이름뿐 아니라 대소문자만 다른
  이름도 충돌로 처리해 exit 2로 끝내며, 산출 파일을 하나도 쓰지 않는다. 이 보수적
  규약은 macOS/Windows 기본 파일시스템과 Linux 재실행에서 같은 결과를 보장한다.
- stdin은 **파일 경로 목록 전용**이다. `batch`는 전역 인증 옵션
  `--password`·`--password-stdin`·`--output-password`·`--output-password-stdin`을
  지원하지 않으며, 함께 주면 입력을 소비하지 않고 exit 2로 거부한다. 암호화 문서의
  batch 처리/암호화 산출물은 credential 전달·산출 형식 계약이 정의된 뒤 별도로 제공한다.
- `--out-dir`의 값은 다음 플래그가 될 수 없다. 이름이 `-`로 시작하는 실제 폴더를
  지정할 때는 `./-결과`처럼 명시한다.
- 실패 레코드(공통): `{"schemaVersion":"1.0","source","error","exitClass":"runtime"}`
- 건별 실패(읽기·파싱·추출·panic)는 레코드로 격리하고 스트림을 계속한다.
  하나라도 실패하면 최종 종료 코드 1 (#2707 계약).
- `--threads <N>` 기본값은 CPU 코어 수. 출력 순서는 병렬에서도 입력 순서를 보존한다.
- 요약(`batch: N건 중 …`)은 stderr 로 나간다 — stdout 은 NDJSON 뿐이다.

```bash
# 아카이브 파이프라인: 메타데이터 스윕 → 대상 선별 → 본문 추출
find docs/ -name '*.hwp' | rhwp batch info --json > meta.ndjson
find docs/ -name '*.hwp' | rhwp batch export-text --json > corpus.ndjson

# 아카이브 전역 검색 — 어느 문서 어느 쪽에 있는지 (#3346)
find docs/ -name '*.hwp' | rhwp batch search --query "위임전결" --json   | jq -c 'select(.matchCount > 0) | {source, pages:[.matches[].page]}'

# 코퍼스 표 수확 / 서식 템플릿 일괄 조사
find docs/ -name '*.hwp' | rhwp batch export-tables --json | jq -c '{source, tableCount}'
find forms/ -name '*.hwp' | rhwp batch fields --json | jq -c 'select(.fieldCount>0) | {source, fieldCount}'

# 편집 가능한 HWP5를 한 폴더에 만들고 저장본 검증까지 집계 (CLI 전용)
find inbox/ -name '*.hwp' | rhwp batch convert --out-dir converted --verify --verify-pages --json > converted.ndjson
```

검증된 에이전트·파이프라인 시나리오(선별→추출, RAG 청킹, 실패 처리)는
[CLI JSON 파이프라인 가이드](cli_json_pipeline_guide.md) 참조.

### `export-markdown <파일> [옵션]`
페이지별 텍스트 → Markdown(.md). `-o`, `-p`.
- `--json` (#3596): 산출물 매니페스트를 stdout 에 JSON 으로.
  `{"schemaVersion":"1.0","source","format":"markdown","outputDir","pageCount","renderedCount","imageCount","pages":[{"page","path","bytes"}]}`
  실패 경로(부분 저장)의 stdout 은 비운다.

### `export-tables <파일> [--json] [-o out.json]` (#3278)
표를 **격자 JSON** 으로 추출한다 (표 데이터의 기계 소비용). 파서/렌더 무변경 읽기 질의.
- 평문·Markdown 추출은 **병합을 잃는다** — `table_to_markdown` 은 앵커 위치에만 텍스트를
  찍어 3열 병합 헤더가 `| 5월 |  |  |` 로 나오고, 소비자는 빈 칸을 별개 열로 오독한다.
  본 명령은 `Table.cells`(앵커 셀 + span)를 직역해 병합을 보존한다.
- `--json` 봉투: `{"schemaVersion":"1.0","source","tableCount","tables":[…]}`
- 표: `{index,section,paragraph,rows,cols,cellCount,caption?,cells:[…]}` —
  `section`/`paragraph` 는 인용·역참조용 주소
- 셀: `{row,col,rowSpan,colSpan,isHeader,text,nested?}` — 병합 셀은 **앵커에 한 번만** 나오고
  덮인 칸은 출력하지 않는다. `nested` 는 셀 안의 표(재귀)
- **본문뿐 아니라 글상자·머리말/꼬리말·각주/미주 안의 표까지 재귀 수집**한다.
  (최상위 `controls` 만 훑는 `info` 의 표 열거는 이들을 놓친다 — 실측:
  `samples/basic/treatise sample.hwp` 는 info 기준 1개, 실제 3개)
- 기본 출력은 사람용 요약(표별 크기·병합·중첩 개수), `-o` 는 pretty JSON 파일 저장
- 한계: 셀 안 **자동번호**는 IR 텍스트에 값이 없어(렌더 단계 주입) 빈 자리로 나온다.
  1×1 래퍼 표(공문서 관용)도 그대로 하나의 표로 잡히므로 소비자가 걸러야 한다.

```bash
# 병합 헤더를 가진 표에서 헤더 셀만 추출
rhwp export-tables 별표.hwp --json | jq '.tables[].cells[] | select(.isHeader)'
```

### `export-render-tree <파일> [옵션]`
페이지별 render tree bbox JSON(레이아웃 시각 분석용). 출력 `render_tree_{NNN}.json`.
- `-o`, `-p`, `--show-para-marks`, `--show-control-codes`, `--respect-vpos-reset`
- JSON: `{type, bbox:{x,y,w,h}, children:[...]}` (Page → PageBg/Line/TextRun/Image/Table/Shape …)

### `export-structure <파일> [--mode auto|outline|clause] [-o out.json] [--json]`
문서 **개요/조문 계층**을 중첩 JSON 트리로 추출 (조문 DB화·목차 생성용). 파서/렌더 무변경 읽기 질의.
- `--json` (#3261): 계약 봉투를 씌운 **한 줄** JSON —
  `{"schemaVersion":"1.0","source","mode","nodeCount","structure":{...기존 트리...}}`.
  기본 출력(무봉투 pretty JSON·`-o` 저장)은 무변경. `batch export-structure` 레코드와 같은 스키마.
- `--mode outline`: IR 개요 수준(`ParaShape.para_level`/head_type) 기반.
- `--mode clause`: 법률 조문 텍스트 패턴(편·장·절·관·조 / 항①②③ / 호1. / 목가.) 기반.
- `--mode auto`(기본): 명시적 `Outline` head_type을 최우선한다. `Number`와 조문형 텍스트가
  충돌하면 목차 쪽번호 tail·조사형 상호참조가 아닌 `제N조` 제목만 clause 선택 증거로 인정한다.
  편·장·절·관은 일반 보고서에도 흔해 Number를 단독으로 뒤집지 않으며, confidence를 통과한 조 제목이
  없으면 Number 문서는 outline을 선택한다. Outline과 Number가 모두 없으면 기존처럼 clause로 폴백한다.
  항·호·목 모양도 일반 번호 목록과 구분할 수 없어 auto 선택 증거로 쓰지 않는다.
- JSON: `{mode, node_count, preamble, roots:[{level,kind,marker,heading,section,paragraph,body,children}]}`.
  비제목 문단은 직전 제목 노드의 `body` 에 귀속. `-o` 생략 시 stdout.

### `export-doclang <파일.hwp|.hwpx> [-o <출력.xml>] [--assets-dir <디렉터리>] [--json]`
HWP5 / HWPX 문서를 **DocLang v0.6** 의미 XML 로 내보낸다 (다운스트림 AI 파이프라인용).
문서를 의미 IR(SirDocument)로 낮춘 뒤 `<doclang version="0.6">` 루트의 XML 로 직렬화한다.
- 입력은 `.hwp`(HWP5) / `.hwpx` 만 받는다. HWP3·HML·DRM·빈 파일은 사용법 오류로 거부한다.
- `-o`, `--output <파일>` 생략 시 입력과 같은 폴더에 `<입력 stem>.dclg.xml`.
  입력==출력 경로면 원본 보호를 위해 거부한다.
- `--assets-dir <디렉터리>` — 그림 등 이진 자원을 이 디렉터리에 파일로 기록하고 XML 은
  해당 경로를 참조한다. 생략 시 자원은 base64 data URI 로 XML 에 인라인된다.
- DocLang v0.6 으로 표현할 수 없는 정보는 손실 보고 건수로 요약 출력한다(변환 자체는 성공).
- `--json` (#3696): 산출 봉투를 stdout 순수 JSON 으로 (변환 동작 무변경).
  `{"schemaVersion":"1.0","source","output","format":"doclang","doclangVersion":"0.6","bytes","assetsDir","assetCount","lossCount"}`
  — `assetsDir` 는 `--assets-dir` 를 준 경우에만 문자열, 아니면 `null`. `lossCount` 는
  사람용 "손실 보고 N건"의 기계 필드. 실패 경로의 stdout 은 비운다(#3596 규약).

---

## 2. 구조 덤프·진단 (Debug)

### `dump <파일> [--section <N>] [--para <N>]` (별칭 `-s`/`-p`)
문서 조판부호 구조 덤프. ParaShape/LINE_SEG/표·도형 속성. 상세: [dump_command.md](dump_command.md)

### `dump-pages <파일> [-p <N>] [--respect-vpos-reset]`
페이지네이션 결과(페이지별 문단/표 배치 목록 + 높이).

### `dump-note-shape <파일.hwp|파일.hwpx>`
구역별 각주/미주 모양 raw 값과 한컴 UI 의미값을 JSON으로 덤프.

### `dump-endnote-lines <파일.hwp> <section> <para> <control> [note-para]`
특정 미주 원본 문단의 line_seg, TextRun, TAC 수식 위치를 함께 덤프.

### `dump-records <파일>`
HWP5 raw record 덤프(DocInfo/BodyText 레코드 트리).

### `diag <파일>`
문서 구조 진단(번호/글머리표/개요 분석).

### `capabilities` (#3263)
도구 자기서술 JSON 을 stdout 으로 출력한다 — 에이전트가 첫 호출 1회로 명령·플래그·
JSON 계약·종료 코드를 파악하는 입구.
`{"schemaVersion":"1.0","tool","version","formats","exitCodes","jsonContract","batch","commands":[{name,category,summary,...}]}`
- `--json` 계약 명령(info/export-text/export-structure/batch)은 `json:true`·`recordFields` 로 상세 서술
- feature 게이트 명령(export-png)은 `requiresFeature`·`available` 을 항상 방출한다 (#3357) —
  값은 빌드 실측과 일치하며(`available:false` 빌드에서만 기능 부재 오류), 매니페스트만 보고
  호출을 생성하는 에이전트가 사전에 걸러낼 수 있다
- `--help`(사람용)와 함께 현행화한다 — help 에만 추가된 명령은 드리프트 가드 테스트가 잡는다
- 편집 명령(`edit`)도 등재된다 — MCP 도구로는 `hwp_fill_fields` 로 노출된다 (#3329)

#### `capabilities --mcp` — MCP 도구 정의 생성
MCP 서버(및 함수 호출 클라이언트)가 **그대로 등록할 수 있는** 도구 정의를 낸다.
`{"schemaVersion":"1.0","protocol":"mcp","server":{…},"invocation":{…},"tools":[{name,description,inputSchema,cli,outputFields}]}`
- 각 도구는 MCP 필수 3종(`name`·`description`·`inputSchema`)에 더해 **실행 배선**(`cli.command`/`cli.args`)을 갖는다.
  `cli.args` 의 `{path}`·`{a}`·`{b}`·`{subcommand}` 자리표시자를 `inputSchema` 의 같은 이름 값으로 치환해 실행한다.
- `hwp_batch` 는 파일 목록을 stdin 으로 받는다(`invocation.stdinTools` 로 명시).
- 로드맵상 MCP 서버 자체는 별도 저장소(#227)다. 서버가 도구 목록을 **손으로 베껴 쓰면 rhwp 가
  바뀔 때 조용히 낡으므로**, 원천을 도구 자신이 낸다. `--json` 계약 명령이 늘었는데 MCP 에서
  빠지면 드리프트 가드(`capabilities_mcp_covers_every_json_command`)가 잡는다.

```bash
# MCP 서버 도구 목록을 자동 생성
rhwp capabilities --mcp | jq '.tools[] | {name, description}'
```

### `mcp-serve` — MCP 서버 (#3140)
rhwp 를 **실제 MCP 서버**로 실행한다. 전송은 MCP 표준 stdio(줄 단위 JSON-RPC 2.0)이며,
`initialize` → `tools/list` → `tools/call` 을 직접 받는다. Claude Code 등 MCP 호스트에는
명령 한 줄로 등록한다:

```jsonc
// MCP 호스트 설정 예 (예: .mcp.json)
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

- **도구 목록은 `capabilities --mcp` 와 단일 출처**(`mcp_tool_definitions`)다 — 선언과 서버가
  어긋날 수 없고, 드리프트 가드(`tools_list_matches_capabilities_manifest`)가 이를 고정한다.
- 무상태 도구 13종(`hwp_info`·`hwp_search`·`hwp_fill_fields` 등)은 선언의 `cli.args` 배선을
  그대로 해석해 자기 자신을 서브프로세스로 실행한다 — #2707 종료 코드·stdout 순수성 등
  검증된 CLI 계약을 문자 그대로 재사용한다. stdout 이 JSON 이면 `structuredContent` 로도 준다.
- **세션 도구 3종**(서버 전용): `hwp_open`(파싱 1회 → `docId` 핸들) → `hwp_doc_text`(재파싱
  없이 페이지 텍스트 반복 조회) → `hwp_close`(해제). #3140 이 짚은 "상태 유지 세션" 공백을
  채운다 — 대형 문서를 여러 번 조회할 때 프로세스별 재파싱 비용이 사라진다.
- 도구 실행 실패(없는 파일 등)는 MCP 규약대로 프로토콜 오류가 아니라 `isError:true` 도구
  결과로 돌아온다. 알 수 없는 메서드는 JSON-RPC `-32601`.
- 의존성 추가 없음 — 프로토콜 표면이 좁아 serde_json 만으로 구현했고, WASM 대상에는
  포함되지 않는다.

### `info <파일> [--json]`
HWP 파일 정보 표시(버전/구역 수/암호화 등).
- `--json` (#3237): stdout 에 순수 JSON 하나 —
  `{"schemaVersion":"1.0","source","format":"hwp5|hwpx|hwp3|hml","sizeBytes","version","sections","pageCount","paraCount","fonts"}`.
  `version` 은 HML 이면 null. 스키마 계약은 `export-text --json` 항목과 동일 규칙.

### `digest <파일> [--max-chars N] [--json]` (#3633)
초소형 모델용 매크로 1호 — "info 로 훑고 → export-structure 로 개요를 얻고 →
export-text 로 첫 장을 읽는" 3단 파이프라인을 **한 번 호출**로 수행한다. 도구
체이닝을 못 하는 로컬 소형 모델(4B급)이 1차 소비자다. 설계 결정은
[초소형 모델용 매크로 도구 축 설계 결정](../tech/tiny_model_macro_tools.md).
- 기계 전용 명령: `--json` 유무와 무관하게 항상 봉투 **한 줄 JSON** 을 낸다 —
  `{"schemaVersion":"1.0","source","format","pageCount","paraCount","outline":[최상위 노드 제목 최대 20개],"excerpt","truncated","nextStep"}`
- `excerpt` 는 페이지 0~2 텍스트를 `--max-chars`(기본 2000) **문자 수**에서 절단한
  발췌. 절단되면 `truncated:true`.
- `nextStep` 은 고정 문자열 계약(다음 행동 유도문) — 문구 변경은
  `tests/digest_macro_contract.rs` 가 잡는다.
- 실패 시 stdout 0바이트, 종료 코드는 #2707 계약(0/1/2).

```bash
# 처음 보는 문서를 한 번 호출로 파악
rhwp digest 편람.hwp --json --max-chars 500
```

### `search <파일> [--json] [--ignore-case] [--limit N] [--] <검색어>` (#3283)
문서를 검색해 매치마다 **구역·문단·페이지·문자 오프셋**을 함께 돌려준다.
평문을 뽑아 외부에서 찾으면 주소가 소멸해 근거 제시가 불가능한데, rhwp 는 조판 엔진이
있어 "몇 쪽"에 답할 수 있다. 파서/렌더 무변경 읽기 질의.
- `--json` 봉투: `{"schemaVersion":"1.0","source","query","caseSensitive","matchCount","totalMatchCount","truncated","omittedCount","matches":[…]}`
- 매치: `{section,paragraph,page?,charOffset,length,text,context,cell?}`
  - `page` 는 0부터 시작하는 글로벌 페이지. 조판에 배치되지 않은 문단이면 생략된다.
  - `cell` 은 표 셀 안의 매치일 때 `{control,cell,paragraph}` 좌표
  - `context` 는 매치 앞뒤 발췌(각 40자)
- 검색 범위는 본문 + 표 셀 + 글상자 (`search_query::search_all` 과 동일)
- **매치 0건은 오류가 아니다** — `matchCount:0`, 종료 코드 0 (1은 런타임 실패 전용)
- `--max-matches N`(= `--limit N`, #3353)은 대형 문서에서 컨텍스트를 아끼기 위한 상한.
  **기본은 무제한**이다. 절단돼도 `totalMatchCount`(문서 전체 매치 수)·`truncated:true`·
  `omittedCount`(생략 매치 수)로 총량이 보인다 — `matchCount` 는 종전대로 반환된 매치
  수(= `matches` 길이)다. 두 이름은 같은 축이며 봉투가 완전히 같다(`--max-matches` 가
  `export-text --max-chars` 와 어휘를 맞춘 이름, #3787 S7). `0` 은 사용법 오류(exit 2).
- **검색어가 `-` 로 시작하면 `--` 뒤에 둔다.** 그러지 않으면 옵션으로 파싱돼
  `알 수 없는 옵션` exit 2 가 난다. `--` 이후는 전부 위치 인자로 읽는다.

  ```bash
  rhwp search 문서.hwp --json -- "-회계"   # '-회계' 를 검색어로
  ```

  MCP `hwp_search` 는 이 구분자를 이미 배선에 넣어 두었으므로 `query` 를 그대로
  주면 된다. `batch search --query` 와 세션 `hwp_doc_search` 는 위치 인자가
  아니라서 원래부터 영향이 없다.
- 성능: 페이지 매핑 비용은 0이다(로드 시 조판 완료). `(구역,문단)→페이지` 인덱스를
  한 번만 만들어 재사용한다. 실측 393쪽·10MB 문서에서 19건 검색 **215ms**(파싱 포함).

```bash
# 근거를 댈 수 있는 답변: 어느 쪽 어느 문단인지
rhwp search 편람.hwp "위임전결" --json | jq -r '.matches[] | "\(.page+1)쪽: \(.context)"'
# 찾은 페이지를 이미지로 렌더해 눈으로 확인
rhwp export-png 편람.hwp -p "$(rhwp search 편람.hwp "위임전결" --json | jq '.matches[0].page')"
```

### `extract-data <파일> [--kind date|amount|number|all] [--limit N] [--json]` (#3719)
행정문서의 **날짜·금액·수량**을 값마다 **구역·문단·페이지·문자 오프셋**과 함께 뽑는다.
`search` 가 검색어에 대해 한 일을 데이터 값에 대해 한다 — 평문을 뽑아 밖에서 정규식을
돌리면 값은 얻어도 주소가 소멸해 근거 제시가 불가능하다. 파서/렌더 무변경 읽기 질의.
- `--json` 봉투: `{"schemaVersion":"1.0","source","kind","itemCount","totalItemCount","truncated","counts","items":[…]}`
- 항목: `{kind,raw,normalized,currency?,unit?,section,paragraph,page?,charOffset,length,cell?,textbox?}`
  - `raw` 는 문서에 적힌 그대로, `normalized` 는 기계용 값이다
  - `counts` 는 **요청한 종류의 문서 전체 건수**(`--limit` 절단 전). 요청하지 않은
    종류의 키는 넣지 않는다 — `"amount":0` 은 "금액이 없다"로 오독되기 때문이다
- 인식 규칙 (실물 표기 기준)
  - 날짜: `2026년 8월 2일` · `2026년 8월 2일(월)` · `2026. 8. 2.` · `2026-08-02` ·
    `2026/8/2` · `'26.8.2`. 연·월만 있는 표기(`2026. 1.` · `2025년 12월`)도 인식한다
  - 금액: `1,234,567원` · `금113,560원`(접두 `금`·`일금`) · `₩1,234,567` ·
    `3,180백만원`·`21,345천원`(단위 배수 반영) · `금 1,234,567원정`. `currency:"KRW"`
  - 수량: `12개` · `3.5%` · `1,000명` — 단위는 `unit` 으로 분리한다. **단위가 없는 맨
    숫자는 항목이 아니다**(표 하나가 수백 건의 잡음이 된다). 한글 단위는 붙여 쓴 것만
    인정하고(`표 3 개요` 의 `개` 를 삼키지 않는다), 기호·라틴 단위는 공백 하나를 허용한다.
    `제3조`·`제137호` 같은 서수는 수량이 아니다
- **정규화 규약 — 모르는 것은 모른다고 한다**
  - `normalized` 는 날짜면 ISO-8601 문자열, 금액·수량이면 숫자다. 일(日)이 없는 표기는
    **부분 날짜**(`2026-01`)로 둔다 — 없는 날을 1일로 채우면 조용히 틀린 값이 된다
  - 정규화할 수 없으면 `normalized: null` 이고 `raw` 만 믿을 수 있다. 두 자리 연도
    (`'26.8.2`)는 세기를 추정하지 않고, 한글 수사 금액(`일금 백이십삼만원`)은 v1 범위 밖이다
  - 금액은 정수 연산으로만 배수를 반영한다(`1.5억원` → `150000000`). 정수로 떨어지지
    않으면 추정하지 않고 `null` 이다
- 추출 범위는 본문 + 표 셀 + 글상자. 표 셀·글상자 값에는 `cell`/`textbox` 좌표가 붙고,
  분할 표는 그 행이 실제로 렌더되는 쪽을 쓴다(#3403). 페이지 인덱스는 `search` 와 같은
  `build_paragraph_page_index` 를 재사용한다
- **0건은 오류가 아니다** — `itemCount:0`, 종료 코드 0 (1은 런타임 실패 전용)
- `--limit N` 은 컨텍스트 상한이다. 절단돼도 `totalItemCount`·`truncated:true` 로 총량이 보인다
- 정규식을 쓰지 않는다 — 왼쪽에서 오른쪽으로 한 번 훑고 인식한 구간을 건너뛰므로
  되추적이 없고(ReDoS 불가), 항목끼리 겹치지 않는다

```bash
# 문서의 금액을 쪽 주소와 함께 (근거를 댈 수 있는 집계)
rhwp extract-data 편람.hwp --kind amount --json \
  | jq -r '.items[] | "\(.page+1)쪽 \(.raw) → \(.normalized)"'
# 정규화하지 못한 표기만 골라 사람이 확인
rhwp extract-data 보고서.hwpx --json | jq '.items[] | select(.normalized == null)'
```

### `thumbnail <파일> [옵션]`
HWP 내장 썸네일(PrvImage) 추출.
- `-o, --output <파일>` (기본 `입력명_thumb.png`)
- `--base64` — base64 문자열 stdout
- `--data-uri` — `data:image/...` URI stdout

### `fields <파일> [--json]` (#3281)
누름틀/필드를 **읽기 전용**으로 조사한다 — 서식이 무엇을 요구하는지 기계가 읽는 입구.
rhwp 는 이미 필드에 값을 쓸 수 있지만(`set_field_value_by_name`) 조회 API 가 WASM/스튜디오
경로에만 있어 CLI 소비자는 접근할 수 없었다. 기존 `collect_all_fields()` 를 그대로 노출한다.
- `--json` 봉투: `{"schemaVersion":"1.0","source","fieldCount","fields":[…]}`
- 필드: `{fieldId,fieldType,name,guide,memo,command,value,editableInForm,location}`
  - `guide` 는 누름틀 안내문, `memo` 는 HelpState 지시문("어떻게 쓰라"는 사람용 설명)
  - `location`: `{section,paragraph,nested:[{kind:"tableCell"|"textBox",…}]}` — 표 셀·글상자
    안의 필드는 `nested` 로 좌표를 준다
- 필드가 없는 문서는 오류가 아니라 `fieldCount:0` 이다 (파이프라인이 멈추지 않는다)
- 기본 출력은 사람용 요약, 종료 코드는 §종료 코드 계약(없는 파일 1·인자 없음 2)
- **범위 한계**: `collect_fields_from_paragraph` 의 재귀는 표 셀·글상자 두 갈래다.
  머리말/꼬리말·각주/미주 안의 필드는 잡히지 않는다(실재하는 사각지대 — 재귀 확장은
  편집 API 좌표계와 함께 봐야 하므로 별도 이슈).

```bash
# 서식이 요구하는 항목과 지시문 확인
rhwp fields 신청서.hwp --json | jq -r '.fields[] | "\(.name): \(.memo // .guide)"'
```

### `edit fill-fields <파일> --data <JSON|@파일> [옵션]` (#3329)
누름틀에 값을 채운다 — 서식 자동 작성/메일머지. 검증된 코어 경로
(`set_field_value_by_name`)를 재사용하므로 새 편집 로직이 없고, **필드 값만 바꾸므로
레이아웃·구조는 불변**이다.
- `--data <JSON|@파일>` — `{"필드이름":"값"}` 형식. `@경로` 면 파일에서 읽는다
  (대량 메일머지에서 셸 인용을 피한다. **UTF-8 이어야 한다** — CP949 로 저장하면
  `stream did not contain valid UTF-8` 로 exit 1). 값이 문자열이 아니면 JSON 표현으로 넣는다.
- **반복 항목 지목(#3476)** — 같은 이름이 여러 번 나오는 서식(규제영향분석서의
  `피규제집단명` ×14 등)은 키에 0 기준 순번을 붙여 N 번째를 지목한다:
  `{"피규제집단명[0]":"…","피규제집단명[13]":"…"}`. 순번은 `fields --json` 목록 순서와 같다.
  순번 없는 키는 **종전대로 첫 매치**를 채우고, 여러 곳에 해당하면 `ambiguous` 로 보고한다.
  범위를 벗어난 순번은 `notFound` 에 그대로 실린다.
- `-o, --output <파일>` — 출력 파일 (기본 `<입력명>_filled.<입력과 같은 확장자>`, §edit 산출 형식)
- `--dry-run` — **파일을 쓰지 않고** 변경 예정 내역만 보고. 에이전트의 사전 확인 장치.
- `--json` 봉투: `{"schemaVersion":"1.0","source","dryRun","filledCount","filled":[{name,occurrence,value}],"notFound":[…],"ambiguous":[…],"output"?,"outputFormat"?}`
  - `notFound` — 문서에 없는 필드 이름(또는 범위를 벗어난 순번). 조용히 무시하지 않으므로 오타를 즉시 안다.
  - `ambiguous` — 순번 없이 준 이름이 **여러 곳에 해당**할 때 `{name, matched, total}` 로 보고한다.
    이 신호가 없으면 소비자가 "14개 중 1개만 채운 문서"를 완성본으로 오판한다.
  - `output`/`outputFormat` 은 실제 저장했을 때만 실린다(`--dry-run` 이면 없음).
- **실패 시 원본 불변**: 필드 설정이 하나라도 실패하면 출력 파일을 쓰지 않고 종료 코드 1.
- 종료 코드는 §종료 코드 계약 (없는 파일·직렬화/쓰기 실패 1 · 인자/JSON 오류 2)

```bash
# 서식 조사 → 값 채우기 → 산출물 재확인 (전 과정 CLI)
rhwp fields 신청서.hwp --json | jq -r '.fields[].name'
rhwp edit fill-fields 신청서.hwp --data @row.json -o out.hwp --json
rhwp fields out.hwp --json | jq -c '[.fields[]|select(.value!="")|{name,value}]'
```

### `edit replace-text <파일> --find <문자열> --replace <문자열> [옵션]` (#3373)
문서 전체 일괄 치환(본문+표 셀) — 기관명 변경·연도 갱신·용어 정비. 검증된 코어 경로
(`replace_all` — 역순 치환으로 오프셋 안전)를 재사용하므로 새 편집 로직이 없다.
- `--find <문자열>` — 찾을 문자열 (빈 문자열은 exit 2)
- `--replace <문자열>` — 바꿀 문자열 (`""` 이면 삭제)
- `--ignore-case` — 대소문자 무시
- `-o, --output <파일>` — 출력 파일 (기본 `<입력명>_replaced.<입력과 같은 확장자>`, §edit 산출 형식)
- `--dry-run` — **파일을 쓰지 않고** 읽기 전용 검색으로 치환 예정 건수만 보고
- `--json` 봉투: `{"schemaVersion":"1.0","source","find","replace","caseSensitive","dryRun","replacedCount","output"?,"outputFormat"?}`
  - `output`/`outputFormat` 은 실제 저장했을 때만 실린다 — **치환 0건이면 출력 파일을 만들지 않는다**
    (무변경 산출물 금지, dry-run 과 동일하게 파일 경로를 타지 않음).
- **실패 시 원본 불변**: 치환·직렬화·쓰기 실패 시 출력 파일 없이 종료 코드 1.

```bash
# 치환 → 산출물 재독 대조 (전 과정 CLI)
rhwp edit replace-text 공문.hwp --find "2025년" --replace "2026년" -o 개정본.hwp --json
rhwp search 개정본.hwp "2025년" --json | jq .matchCount     # → 0 이어야 함
```

### `edit set-cell <파일> --table <번호> --row <행> --col <열> --text <문자열> [옵션]` (#3381)
표 격자 좌표로 셀 값을 바꾼다. `export-tables`의 `index`/`row`/`col`과 같은 좌표계를 써서
누름틀 없는 표 양식도 발견 → 기록 → 재독 검증을 하나의 주소로 닫는다.

- `--table`/`--row`/`--col` — 본문 최상위 표의 0-based 격자 좌표
- `--text <문자열>` — 셀에 넣을 값. 빈 문자열은 비우기이며 줄바꿈·탭은 v1에서 허용하지 않는다.
- `--keep-style` — 셀 안내문의 글자 모양을 유지한다. 기본은 검정·비이탤릭·비진하게로 기록해,
  제출용 양식의 파란 안내문 스타일을 실값이 상속하지 않게 한다 (#3391).
- `-o, --output <파일>` — 출력 파일 (기본 `<입력명>_cell.<입력과 같은 확장자>`, §edit 산출 형식)
- `--dry-run` — 파일을 쓰지 않고 `oldText` → `newText` 변경 예정만 보고한다.
- `--json` 봉투: `{"schemaVersion":"1.0","source","table","row","col","oldText","newText","dryRun","keepStyle","overflow":[…],"output"?,"outputFormat"?}`
- **맞춤 검사(#3480)** — 넣은 값이 칸 폭을 넘치면 `overflow` 로 알린다:
  `[{"target":"table0[2,3]","text":"…","cellWidthPx":214.63,"textWidthPx":440.0,"lines":3}]`
  - 에이전트는 렌더 결과를 보지 않으므로, 신호가 없으면 표 경계를 벗어난 문서를
    완성본으로 판단한다. 이 검사는 **조판 엔진이 있어야** 가능하다.
  - **채우기를 막지 않는다** — 여러 줄이 정상인 칸(주소·사유)도 있으므로 판단은 소비자 몫이다.
  - `--dry-run` 에서도 검사하므로 **파일을 만들기 전에** 알 수 있다.
  - 칸 폭은 `Cell.width` − 안여백, 글자 폭은 첫 문단 `CharShape.base_size` 기준 한글 전각·
    ASCII 반각 **근사**다(넘침 판정용이며 정밀 조판이 아니다). 실측 사례:
    [편집 맞춤 검사](../report/edit_demo_fit_check/README.md).
- 병합으로 덮인 칸은 앵커 좌표를 안내하며 exit 2로 끝난다. 격자 밖 좌표도 exit 2다.
- 실패 시 원본은 불변이며, v1 범위는 본문 최상위 표와 셀 첫 문단이다.

```bash
# 발견 → 기록 → 재독 검증
rhwp export-tables 양식.hwpx --json | jq '.tables[0].cells[:4]'
rhwp edit set-cell 양식.hwpx --table 0 --row 2 --col 1 --text "1,234" -o 작성본.hwpx --json
rhwp export-tables 작성본.hwpx --json | jq '.tables[0].cells[] | select(.row==2 and .col==1).text'
```

### `edit redact <파일> [--kind …] [--mask <문자>] [--dry-run] [-o <출력>|--in-place]` (#3719 §6-11)
공개 전 개인정보 마스킹 — 주민등록번호·전화번호·이메일·카드번호를 찾아 **자릿수를 유지한 채**
가린다. 탐지는 읽기 전용 코어(`document_core::queries::pii_scan`)가 하고, 실제 변경은 검증된
치환 경로(`replace_all_native`)를 재사용하므로 새 편집 로직이 없다. 주소는 `grep` 재사용이라
매치마다 구역·문단·**쪽**·문자 오프셋이 따라온다.

- `--kind <목록>` — `ssn|phone|email|card|all` 을 쉼표로 나열 (기본 `all`)
- `--mask <문자>` — 마스킹 문자 **한 글자**. 영숫자는 거부한다(본문과 구별 불가). 두 글자
  이상이면 자릿수 보존이 깨지므로 조용히 자르지 않고 exit 2.
- `--dry-run` — **권장 첫 단계**. 파일을 만들지 않고 `findings[]` 만 보고한다.
- `-o, --output <파일>` / `--in-place` — 둘 중 하나가 **반드시** 필요하다(§원본 보호).
- `--verify` — 저장 직후 IR 자기검증(차이 시 exit 3, #3702)
- `--json` 봉투:
  `{"schemaVersion":"1.0","source","kinds","mask","dryRun","inPlace","findingCount",`
  `"findings":[{"kind","raw","masked","section","paragraph","page","charOffset"}],`
  `"redactedCount","changedPages","output"?,"outputFormat"?,"verify"?}`
  - `output`/`outputFormat`/`verify` 는 실제 저장했을 때만 실린다 — **탐지 0건이면 출력
    파일을 만들지 않는다**.

**탐지 규칙(보수적)** — 마스킹은 되돌릴 수 없고 오탐은 본문을 훼손하므로, 형태가 맞아도
검증을 통과하지 못하면 **탐지하지 않는다**.

| 종류 | 형태 | 추가 검증 |
| --- | --- | --- |
| `ssn` | `######-#######` | 생년월일 실재(윤년 포함) + 성별/세기 코드 1~8 + mod 11 검증 숫자 |
| `card` | `4-4-4-4`(`-`/공백), Amex `4-6-5`, 연속 15·16자리 | Luhn |
| `phone` | `01[016789]-3~4자리-4자리`, `02-3~4자리-4자리` | 하이픈 필수 |
| `email` | `지역부@라벨(.라벨)+` | 라벨 2개 이상 + 최상위 도메인 영문 2자 이상 |

- 앞뒤가 숫자면 더 긴 토큰의 일부로 보고 버린다(계좌번호 안의 16자리 부분열 오인 방지).
- **02 외 지역번호, 13·14·19자리 카드, 여권번호·계좌번호는 v1 범위 밖**이다 — 체크섬이
  없거나 문서번호와 구별할 근거가 없어 보수적 판정이 불가능하다. 근거와 확장 조건은
  [처리 기록](../report/task_m100_redact/README.md)에 있다.

**원본 보호** — `-o` 도 `--in-place` 도 없으면 **실행하지 않는다**(exit 2). 다른 `edit`
명령과 달리 `_redacted.hwp` 같은 기본 이름도 만들지 않는다. `-o` 가 원본 자신을 가리켜도
같은 이유로 거부한다. 쓰기는 원자적이라 `--in-place` 도중 실패해도 원본이 잘리지 않는다.

> `findings[].raw` 는 **원문 개인정보 그 자체**다. 감사를 위해 넣었지만 로그·이슈에 그대로
> 붙이면 유출 경로가 된다.

```bash
# 권장 흐름: 먼저 무엇이 지워질지 본다 → 확인 후 적용
rhwp edit redact 계약서.hwp --dry-run --json | jq '.findings[] | {kind, page, masked}'
rhwp edit redact 계약서.hwp -o 공개본.hwp --verify --json | jq '{redactedCount, changedPages}'
rhwp edit redact 계약서.hwp --kind ssn,card -o 공개본.hwp --json   # 종류를 좁힐 수도 있다
```

### `edit sanitize <파일> [--keep-preview] [-o <출력>] [--json]` (#3719 §6-11)
문서 메타데이터 제거 — 작성자·제목·주제·최종수정자·작성/수정 일시·미리보기.
**본문 내용은 건드리지 않는다**(`export-text` 결과가 전후 동일).

- `--keep-preview` — 미리보기 **이미지**를 남긴다(기본은 제거). 미리보기 텍스트는 언제나 대상.
- `-o, --output <파일>` — 출력 파일 (기본 `<입력명>_sanitized.<입력과 같은 확장자>`)
- `--json` 봉투:
  `{"schemaVersion":"1.0","source","keepPreview","removedCount","removed":[{"field","before"}],"output","outputFormat"}`

지우는 대상은 셋이다.

1. **OLE 요약 정보**(`\x05HwpSummaryInformation`) — `title`·`subject`·`author`·`keywords`·
   `comments`·`lastSavedBy`·`revisionNumber`·`dateString`(문자열)과 `createdAt`·
   `lastSavedAt`·`lastPrintedAt`(FILETIME → ISO 8601 로 보고). 속성 오프셋 표가 절대 위치를
   담고 있어 **바이트 길이를 바꾸지 않고** 비운다.
2. **HWPX 저작자 메타**(`Contents/content.hpf` 의 `<opf:metadata>`) — 직렬화기가 원본에서
   그대로 splice 하는 유일한 저작자 경로다. 중립 블록으로 교체한다.
3. **미리보기**(PrvText·PrvImage) — ZIP 엔트리와 HWP5 계약 스트림 양쪽.

`removed[]` 는 **거짓 보고를 하지 않는다**. HWP5 직렬화기는 PrvText 가 비면 본문 앞부분으로
다시 채우므로, 미리보기 텍스트는 **지금 본문과 다를 때만**(= 예전 판의 잔재일 때만) 지우고
보고한다. HWPX 원본의 `/HwpSummaryInformation` 은 파일에 없던 계약 fallback 상수라 HWPX 로
저장할 때는 손대지 않고, HWP5 로 변환할 때만 처리한다. 그래서 **두 번째 실행은
`removedCount: 0`** 이다 — 첫 실행이 실제로 지웠다는 증거다.

```bash
rhwp edit sanitize 보고서.hwp -o 배포본.hwp --json | jq '.removed[] | "\(.field): \(.before)"'
rhwp edit sanitize 배포본.hwp -o /tmp/재확인.hwp --json | jq .removedCount   # → 0
```

### `edit` 산출 형식 (#3383)
`edit` 5종(`fill-fields`/`replace-text`/`set-cell`/`redact`/`sanitize`)은
**입력 형식을 보존**한다.

- HWPX 입력 → HWPX 산출(`export_hwpx_native`), 기본 확장자도 `.hwpx`
- 그 밖의 입력(HWP5/HWP3) → HWP5 산출. 이때 직렬화는 **어댑터 경유**
  (`export_hwp_with_adapter`)라 HWPX→HWP IR 매핑(#178)을 건너뛰지 않는다.
- `--json` 봉투의 `outputFormat` 이 실제 저장 형식을 보고한다. 값은 `info --json` 의
  `format` 과 같은 어휘(`"hwp5"`/`"hwpx"`)라 두 봉투를 그대로 대조할 수 있다.
- 예외 하나: HWPX 입력에 `-o ….hwp` 를 **명시**하면 그 경로를 그대로 존중해 HWP5 로
  저장하되(기존 스크립트 호환), 형식이 바뀐다는 사실과 이미지·차트 유실 가능성을
  stderr 로 경고한다. 반대로 HWP 입력에 `-o ….hwpx` 를 줘도 형식은 바뀌지 않으며
  (경고만 출력) 형식 변환은 `export-hwpx` 가 담당한다.

종전에는 세 명령 모두 HWP5 를 강제 산출해 HWPX 양식이 조용히 `.hwp` 로 바뀌었고,
어댑터 없는 경로라 실물 정부 양식에서 차트·이미지 페이지가 유실됐다(#3383).

---

## 3. 변환·비교

### `convert <입력.hwp|.hwpx> <출력.hwp> [--verify] [--verify-pages] [--output-password-stdin]`
배포용(읽기전용) HWP → 편집 가능 HWP 변환. 출력은 항상 `.hwp`.
- `--verify` — 저장 후 산출물을 재파싱하여 어댑터 적용 후 IR과 재로딩 IR 차이를 검출한다.
  차이가 있으면 산출물은 남기고 종료 코드 3으로 실패한다.
- `--verify-pages` — 저장 전 문서 페이지 수와 저장 후 재로딩 페이지 수를 비교한다.
  불일치하면 산출물은 남기고 종료 코드 4로 실패한다.
- `--output-password <값>` / `--output-password-stdin` — 출력 HWP5에 비밀번호를 설정한다.
  암호 출력은 HWPX→HWP adapter를 적용한 뒤 저장한다.

### `extract-pages <입력> <출력.hwp> --from N --to M [--json]` (#3565)
지정한 쪽 범위만 남겨 저장한다. **대형 문서의 결함을 이분법으로 좁히기 위한 진단 도구**다
(387쪽 문서가 저장 후 한컴에서 열리지 않을 때 절반씩 잘라 재현 여부를 본다).
- **`--from`/`--to` 는 1 기준이다** (첫 쪽이 1). rhwp 의 다른 쪽 축(`-p`,
  `export-text` 의 `pages[].page`, `search` 의 `matches[].page`)은 0 기준이므로
  그대로 옮겨 쓰면 **오류 없이 한 쪽 밀린 문서**가 나온다. `search` 가 `page: 1` 을
  줬다면 여기서는 `--from 2 --to 2` 다.
- 쪽 단위로 자르되 **문단 단위로** 지운다. 여러 쪽에 걸친 문단은 한 쪽이라도 범위 안이면 남긴다.
- 결과 쪽수가 요청 범위와 정확히 같지 않을 수 있다(잘라 낸 뒤 레이아웃이 다시 흐른다).
  목적은 **재현 최소화**이지 정밀한 페이지 오려내기가 아니다.
- 구역·DocInfo·BinData 는 그대로 남는다. 그 축들을 떼어 내려면
  [`tools/hwp_open_bisect/`](../../tools/hwp_open_bisect/README.md) 를 쓴다.
- `--json` — 결과 요약(원본/추출 후 쪽수, 남긴·지운 문단 수)을 JSON 한 줄로 출력한다.

### `export-hwpx <입력.hwp|.hwpx> [출력.hwpx] [--verify] [--verify-pages] [--output-password-stdin]` (#1868, #1638)
HWP 문서를 HWPX(ZIP+XML)로 변환 저장. `convert`(배포용 해제)와 별개의 포맷 변환 명령.
- 입력 포맷 자동 감지(HWP5/HWP3/HWPX — HWPX 입력은 재직렬화).
- 출력 생략 시 입력과 같은 폴더에 `<입력 stem>.hwpx`. 입력==출력 경로면 거부(원본 보호).
- `--verify` — 변환 후 산출물을 재파싱하여 원본 IR과 산출물 IR 차이를 검출한다.
  차이가 있으면 산출물은 남기고 종료 코드 3으로 실패한다.
- `--verify-pages` — 변환 전/후 렌더 페이지 수를 비교한다.
  불일치하면 산출물은 남기고 종료 코드 4로 실패한다.
- `--output-password <값>` / `--output-password-stdin` — 출력 HWPX의 ODF encryption-data와
  AES-256-CBC/PBKDF2 보호를 설정한다.
- `--json` (#3596): 변환·검증 봉투를 stdout 순수 JSON 으로.
  `{"schemaVersion":"1.0","source","output","format":"hwpx","bytes","verify","verifyPages"}`
  — `verify`/`verifyPages` 는 해당 옵션을 준 경우에만 객체(`{identical,diffCount}` /
  `{before,after,identical}`), 아니면 `null`. **종료 코드 계약은 무변경**: 차이가
  검출되면 봉투를 낸 뒤 exit 3/4 로 끝난다(`ir-diff --json` 과 같은 "판정은 데이터" 규약).
  재파싱 실패는 판정 불가이므로 stdout 을 비우고 기존 코드로 끝난다.
  `convert` 는 이 옵션을 받지 않는다(구현 없는 침묵 수용 방지, exit 2).
- 더 넓은 시각 정합은 `tools/roundtrip_fidelity_harness.py` 또는 `render-diff`로 별도 대조한다.
  단, 한컴 기준 PDF가 있는 대형 문서는 `tools/fidelity_compare/fidelity_compare.py`의 direct pair
  `--source <HWP/HWPX> --reference-pdf <PDF> --label <ASCII>`를 사용한다. 먼저
  `--text-only --export-all-svg --layout-ledger`로 PDF text↔SVG text 및 render-tree 기하 후보(본문/각주,
  표/footer, frame, Square/Tight/Through 그림을 3행 이상 침범한 본문)를 전수 수집하고, 후보 페이지만
  pixel compare/visual sweep으로 확정한다. 이때 `text-owner-shift-candidates.tsv`의 인접 쪽
  reciprocal text difference와 `text-owner-sequence-candidates.tsv`의 NFC·공백 정규화 16자 이상
  순서 보존 text 이동은 각주·본문·caption이 한 쪽 이르게/늦게 놓인 physical owner 후보를 바로 묶어 준다.
  후자는 p52→p53 URL처럼 다른 본문과 문자 Counter가 상쇄되는 이동을 보완한다.
  `table-fragment-candidates.tsv`는 같은 source `(pi, ci)` Body 표가 인접 render-tree 쪽에 연속한 경우와
  표/footer·frame 신호, 또는 쪽 하단 표와 24자 이상 text delta를 rows/cols·bbox·쪽 신호와 함께 묶는다.
  이는 visual review 우선순위 후보일 뿐 **PDF table row owner나 표 분할 정답을 판정하지 않는다.**
  `page-count-ledger.tsv`는 PDF↔전체 SVG/render-tree page count drift를 별도로 드러낸다. 이 신호들은
  모두 candidate일 뿐 전역 page-break 보정의 근거는 아니다. 같은 문자만으로는 표 row geometry·same-page
  overlap을 판정할 수 없다.
  `render-diff`는 rhwp
  자기 roundtrip 비교이므로 한컴 PDF 기준 fidelity 후보를 대신하지 않는다.

### `export-hml <입력.hml> -o <출력.hml>`
HML 원본 문서를 의미 보존 HWPML 2.91 XML로 저장한다.
- `-o`, `--output <파일>`은 필수다.
- 입력과 출력이 같은 경로이면 원본 보호를 위해 거부한다.
- 이 명령은 HWP/HWPX 변환 명령이 아니며 입력은 `.hml`만 받는다.

### `ir-diff <파일A.hwpx> <파일B.hwp> [-s <구역>] [-p <문단>] [--summary] [--max-lines N] [--json]`
두 파일의 IR 비교(HWPX↔HWP 불일치 검출). 상세: [ir_diff_command.md](ir_diff_command.md)
- 비교: text, char_count/offsets/shapes, line_segs, controls, tab_extended, ParaShape, TabDef,
  표(page_break/outer_margin/treat_as_char/wrap/size/offset), 그림·도형(rel_to 등)
- `--json` (#3274): 판정 봉투 **한 줄** JSON 을 stdout 으로 —
  `{"schemaVersion":"1.0","a","b","identical","diffCount","categories":{카테고리:건수}}`.
  종료 코드 0=동일 / **3=차이 발견**(위 "종료 코드 (#2707)" 표의 "IR 차이 검출" 코드와 동일 의미) /
  1=읽기·파싱 실패(stdout 0바이트) / 2=사용법 오류 → 변환 파이프라인 게이트:
  `rhwp ir-diff 원본.hwp 변환본.hwpx --json || 격리처리`
- 종료 코드 정정(#3274): 기본(텍스트) 모드도 읽기·파싱 실패는 1, 인자 부족은 2 (#2707 정렬).
  **기본 모드의 정상 비교는 차이가 있어도 종전대로 0** — 기존 소비자 무변경.

### `build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>`
ingest JSON(시험문제 등) → HWPX 생성. (rhwp-exam-ingest 파이프라인)

- 이 명령은 PDF/HWP를 직접 분석하지 않는다. Vision/수동 분석/외부 도구가 만든
  `ingest.json` 중간 표현을 rhwp HWPX 문서로 조립한다.
- `-o`, `--output <out.hwpx>` 는 필수다.
- `--media-dir <dir>` 는 `ingest.json` 의 `media[].id` 와 이미지 `stem_blocks[].ref` 를
  해석할 기준 디렉터리다. 이미지가 없으면 생략한다.
- 최소 입력 필드: `version`, `page_size`, `default_font`, `questions[]`.
  각 문제는 `number`, `stem`, `passage_ref`, `stem_blocks`, `choices`, `media`, `auto_number` 를 사용할 수 있다.
  top-level optional 필드로 `passages`, `header_text`, `footer_text`, `form_label` 을 사용할 수 있다.
  `stem_blocks` 는 `text`, `image`, `boxed` 블록을 지원한다.
  자세한 스키마 모델은 `src/parser/ingest/schema.rs`, 예시는
  `tools/rhwp-ingest/schema/sample_minimal.json` 과
  `tools/rhwp-ingest/schema/sample_structured.json` 을 기준으로 확인한다.
- 시험지 e2e 검증은 생성만으로 끝내지 않고, 산출 HWPX를 다시 CLI로 확인한다.

```bash
rhwp build-from-ingest tools/rhwp-ingest/schema/sample_minimal.json \
  -o output/poc/ingest/sample_minimal.hwpx

rhwp build-from-ingest tools/rhwp-ingest/schema/sample_structured.json \
  -o output/poc/ingest/sample_structured.hwpx

rhwp export-text output/poc/ingest/sample_minimal.hwpx \
  -o output/poc/ingest/text

rhwp dump output/poc/ingest/sample_minimal.hwpx \
  > output/poc/ingest/sample_minimal.dump.txt

rhwp export-svg output/poc/ingest/sample_minimal.hwpx \
  -o output/poc/ingest/svg
```

- 텍스트 보존 검증은 `ingest.json` 의 문제/지문/선택지 텍스트와 `export-text` 결과를 비교한다.
- 구조 검증은 `dump` 로 ParaShape/CharShape/표·이미지 control 생성 여부를 확인한다.
- `export-svg` 는 산출 HWPX 가 렌더러에서 SVG 로 변환 가능한지 확인하는 smoke test 로
  사용할 수 있다. 이것만으로 원본 PDF 와 시각적으로 일치한다고 판정하지 않는다.
- 원본 PDF 와의 시각 검증이 필요하면 PDF 기준 비교를
  [visual_sweep_guide.md](verification/visual_sweep_guide.md)에 따라 별도로 수행한다.
- 수식/도형/손글씨처럼 PDF 텍스트 레이어가 의미 정보를 잃는 항목은 `build-from-ingest` 단독으로
  복원할 수 없다. 이 경우 ingest 단계에서 이미지/media 또는 전용 구조로 분류하고,
  결함 유형을 hotfix/follow-up 으로 나누어 기록한다.

### `hwpx-roundtrip <파일.hwpx | --batch 폴더> [-o <출력폴더>] [--lineseg-report]`
HWPX → IR → HWPX roundtrip 검증(**구조 보존 게이트**, #1315 baseline). 재조립 `.rt.hwpx` 와
`inventory.tsv` 산출(기본 `output/poc/task1315`). 하드 실패 존재 시 종료 코드 1.
`samples/hwpx/` 전수 회귀는 `cargo test --test hwpx_roundtrip_baseline`.
상세: [hwpx_roundtrip_baseline.md](hwpx_roundtrip_baseline.md)
- `--lineseg-report` — 문단별 lineseg diff를 `lineseg_diff.tsv` 로 산출(#1380).
- 주의: baseline 통과 = 뼈대 보존이며 시각 충실도 보장이 아니다(시각은 `render-diff`).

### `hwp5-roundtrip <파일.hwp | --batch 폴더> [-o <출력폴더>]`
HWP5 → IR → HWP5 roundtrip 무손실 검증(#1552). 재조립 `.rt.hwp` 와 `inventory.tsv` 산출
(기본 `output/poc/task1552`). 상세: [hwp5_roundtrip_baseline.md](hwp5_roundtrip_baseline.md)

### `render-diff <파일> [--via hwpx|hwp] [-p <페이지>] [--max-disp <px>]`
라운드트립 **시각 정합성 게이트** — 페이지별 `RenderNode` bbox 변위(px)를 정량화한다.
구조 보존만 보는 `hwpx-roundtrip` 과 달리, 라운드트립이 유발한 렌더 기하 변화(시각 회귀)를
검출한다(자기 roundtrip 통과 ≠ 한컴 충실도임에 유의 — 내부 회귀 방지용).
- `render-diff <파일>` — 자기 라운드트립(원본 IR vs 직렬화→재로드 IR). `--via hwpx`(기본)는
  hwp 레거시→hwpx 전환 시각 보존 검증, `--via hwp` 는 HWP 어댑터 경로.
- `render-diff <A> <B>` — 두 파일 직접 비교.
- `--batch <폴더> [-o 출력폴더]` — 폴더 전수 → `geom_inventory.tsv`(기본 `output/poc/render_diff`).
  컬럼: sample/status/pages_a/pages_b/max_disp/worst_page/struct_pages/over_pages/elapsed_ms/error/**struct_delta**.
- status: PASS / OVER(변위>임계) / STRUCT_MISMATCH(노드 삽입·삭제) / PAGE_MISMATCH(하드) / LOAD_FAIL.
- 종료 코드: `PASS`만 0, `OVER`/`STRUCT_MISMATCH`/`PAGE_MISMATCH`/`LOAD_FAIL`은 1.
- 매칭: 노드 타입 LCS 정렬(삽입/삭제 있어도 대응 노드 변위 측정). `--max-disp` 기본 1.0px.
- **구조 불일치 원인 국소화**: STRUCT_MISMATCH 시 노드 타입별 순증감을 출력한다(단일은 페이지별
  `Δ Line: 4→0 (-4)  RawSvg: 1→0 (-1)`, 배치는 콘솔/`struct_delta` 컬럼에 `Line:-4;RawSvg:-1`).
  음수=라운드트립 손실, 양수=추가. 손실 노드 타입으로 직렬화 누락 원인을 즉시 좁힌다.

### `bench <파일...> | --batch <폴더> [-n <반복수>] [--tsv <출력.tsv>]`
**단계별 처리 성능 계측** — parse / layout / render / serialize 를 워밍업 1회 후 N회(기본 3)
반복하여 median(ms)으로 보고한다.
- 단계: `parse`(바이트→IR, `parse_document`) · `layout`(=load−parse 근사) ·
  `render`(전 페이지 SVG) · `serialize`(`serialize_hwpx`, 저장 비용).
- 파일별 크기KB/쪽수 + 단계별 median + total 표, 다파일 시 합계·쪽당 평균.
- `--batch <폴더>` 재귀 전수(.hwp/.hwpx), `--tsv <경로>` 산출(부모 폴더 자동 생성).
- **주의**: 절대 수치는 측정 머신·빌드(release/debug) 의존. 동일 환경 **상대 비교·재현**
  지표로 해석(한컴 등 외부 기준 아님). release 빌드 권장.

---

## 4. HWPX→HWP 저장 계약 분석 (hwp5-* 진단 도구)

HWPX→HWP 직렬화(#178 어댑터) contract 분석·디버깅 전용. oracle(한컴 저장본)과 generated(rhwp 저장본)
record 를 축별로 비교한다.

| 명령 | 용도 |
|------|------|
| `hwp5-inventory <파일> [--format jsonl\|md] [--section N] [--out <path>]` | DocInfo/BodyText record inventory 생성 |
| `hwp5-inventory-diff <oracle> <generated> [--align index\|lcs] [--report …] [--focus …] [--window N] …` | inventory 비교 + contract 힌트/bundle |
| `hwp5-contract-analyze <source.hwpx> <oracle> <generated> --out-dir <폴더>` | record-control contract graph 보고서 |
| `hwp5-ctrl-data-trace <oracle> <generated> --out <path> [--section N] [--record-index N]` | CTRL_DATA ParameterSet 구조 추적 |
| `hwp5-contract-probe <oracle> <generated> --out-dir <폴더>` | MEMO_SHAPE/ID_MAPPINGS + 누락 CTRL_DATA 축 판정 probe |
| `hwp5-table-probe <oracle> <generated> --out-dir <폴더>` | TABLE/CTRL_HEADER(Table) field 축 판정 probe |
| `hwp5-cell-header-probe <oracle> <generated> --out-dir <폴더>` | 표 셀 LIST_HEADER/PARA_HEADER 계약 probe |
| `hwp5-mel-personnel-probe <oracle> <generated> --out-dir <폴더>` | mel-001 인원현황 표 축 판정 probe |
| `hwp5-borderfill-diagonal-probe <oracle> <generated> --out-dir <폴더>` | BORDER_FILL 대각선 attr/payload 축 판정 probe |
| `hwp5-first-para-control-probe <oracle> <generated> --out-dir <폴더>` | 첫 문단 control/PARA_TEXT/PARA_CHAR_SHAPE 계약 probe |
| `hwp5-anchor-trace <파일> --needle <텍스트> [--section N] [--window N] [--out <path>]` | 특정 텍스트 주변 raw HWP5 record 추적 |

---

## 5. 내부 개발·회귀 도구 (test-*, gen-*)

일반 사용자 대상 아님. 회귀 검증·픽스처 생성용.

| 명령 | 용도 |
|------|------|
| `test-caption <파일>` | 캡션 라운드트립 검증 |
| `test-field <파일>` | 필드 라운드트립 검증 |
| `test-shape <입력> <출력>` | 도형 라운드트립 검증 |
| `gen-table` | 표 테스트 HWP 생성 |
| `gen-pua` | PUA 문자 테스트 HWP 생성 |

---

## 6. 디버깅 워크플로우 (참고)

레이아웃/간격 버그 디버깅 권장 순서(상세 CLAUDE.md):

1. `export-svg --debug-overlay` → 문단/표 식별(`s{섹션}:pi={인덱스} y={좌표}`)
2. `dump-pages -p N` → 해당 페이지 배치 목록·높이
3. `dump -s N -p M` → ParaShape/LINE_SEG/표 속성 상세
4. (HWPX↔HWP 불일치) `ir-diff a.hwpx b.hwp`
5. (저장 계약) `hwp5-inventory-diff oracle.hwp generated.hwp`
6. (정밀 좌표) `export-render-tree -p N` → bbox JSON 직접 비교

---

## 단위 환산
- 1인치 = 7200 HWPUNIT = 25.4mm = 96px(DPI 96)
- 1mm ≈ 283.46 HWPUNIT, 1px = 75 HWPUNIT

## 비고
- 본 문서는 `src/main.rs` 명령 디스패치 기준. CLI 추가/변경 시 `--help` 문자열과 본 문서를 함께 갱신한다.
- 2026-07-04 현행화: dispatch 39개 명령 전수 등재 완료(§1~§5). 게이트·공용 명령은 정식 절,
  조사 프로브(§4)·개발 보조(§5)는 묶음 등재.

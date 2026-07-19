---
kind: canonical
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-07-16
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

**프로필 의미론** — 편집 시각 요소(#2225 그림 미지정 placeholder 등)의 표시 여부를 가른다:

| 프로필 | 편집 시각 요소 | 용도 |
|--------|---------------|------|
| `screen`, `fast-preview` | **표시** — 그림 미지정 placeholder 를 점선 테두리+아이콘으로 렌더 | 편집기/미리보기 등가 |
| `print`, `high-quality` | **억제** — 한컴 인쇄 동작과 동일하게 미출력 | 인쇄 등가 산출물 |

> 한컴은 그림 미지정 placeholder 를 편집기에서만 표시하고 인쇄(및 인쇄 등가
> 출력)에서는 미출력한다 — rhwp 의 인쇄 등가 프로필이 이 계약을 따른다.

---

## 1. 내보내기 (Export)

### `export-svg <파일> [옵션]`
HWP/HWPX → SVG.
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
- `-o <파일>`, `--output <파일>` — 출력 PDF 파일(기본 `output/<입력명>.pdf`)
- `-p <번호>`, `--page <번호>` — 0-based 단일 페이지 선택. 생략하면 전체 문서를 다중 페이지 PDF로 내보낸다.
- `--font-path <경로>` — PDF 변환 fontdb에 추가할 폰트 탐색 경로(여러 번 지정 가능)
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
- direct/vector `PageLayerTree → PDF` backend는 아직 후속 작업이다.

### `export-text <파일> [옵션]`
페이지별 텍스트 → TXT. `-o`, `-p`.

### `export-markdown <파일> [옵션]`
페이지별 텍스트 → Markdown(.md). `-o`, `-p`.

### `export-render-tree <파일> [옵션]`
페이지별 render tree bbox JSON(레이아웃 시각 분석용). 출력 `render_tree_{NNN}.json`.
- `-o`, `-p`, `--show-para-marks`, `--show-control-codes`, `--respect-vpos-reset`
- JSON: `{type, bbox:{x,y,w,h}, children:[...]}` (Page → PageBg/Line/TextRun/Image/Table/Shape …)

### `export-structure <파일> [--mode auto|outline|clause] [-o out.json]`
문서 **개요/조문 계층**을 중첩 JSON 트리로 추출 (조문 DB화·목차 생성용). 파서/렌더 무변경 읽기 질의.
- `--mode outline`: IR 개요 수준(`ParaShape.para_level`/head_type) 기반.
- `--mode clause`: 법률 조문 텍스트 패턴(편·장·절·관·조 / 항①②③ / 호1. / 목가.) 기반.
- `--mode auto`(기본): 개요 head_type 있으면 outline, 없으면 clause.
- JSON: `{mode, node_count, preamble, roots:[{level,kind,marker,heading,section,paragraph,body,children}]}`.
  비제목 문단은 직전 제목 노드의 `body` 에 귀속. `-o` 생략 시 stdout.

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

### `info <파일>`
HWP 파일 정보 표시(버전/구역 수/암호화 등).

### `thumbnail <파일> [옵션]`
HWP 내장 썸네일(PrvImage) 추출.
- `-o, --output <파일>` (기본 `입력명_thumb.png`)
- `--base64` — base64 문자열 stdout
- `--data-uri` — `data:image/...` URI stdout

---

## 3. 변환·비교

### `convert <입력.hwp|.hwpx> <출력.hwp> [--verify] [--verify-pages]`
배포용(읽기전용) HWP → 편집 가능 HWP 변환. 출력은 항상 `.hwp`.
- `--verify` — 저장 후 산출물을 재파싱하여 어댑터 적용 후 IR과 재로딩 IR 차이를 검출한다.
  차이가 있으면 산출물은 남기고 종료 코드 3으로 실패한다.
- `--verify-pages` — 저장 전 문서 페이지 수와 저장 후 재로딩 페이지 수를 비교한다.
  불일치하면 산출물은 남기고 종료 코드 4로 실패한다.

### `export-hwpx <입력.hwp|.hwpx> [출력.hwpx] [--verify] [--verify-pages]` (#1868, #1638)
HWP 문서를 HWPX(ZIP+XML)로 변환 저장. `convert`(배포용 해제)와 별개의 포맷 변환 명령.
- 입력 포맷 자동 감지(HWP5/HWP3/HWPX — HWPX 입력은 재직렬화).
- 출력 생략 시 입력과 같은 폴더에 `<입력 stem>.hwpx`. 입력==출력 경로면 거부(원본 보호).
- `--verify` — 변환 후 산출물을 재파싱하여 원본 IR과 산출물 IR 차이를 검출한다.
  차이가 있으면 산출물은 남기고 종료 코드 3으로 실패한다.
- `--verify-pages` — 변환 전/후 렌더 페이지 수를 비교한다.
  불일치하면 산출물은 남기고 종료 코드 4로 실패한다.
- 더 넓은 시각 정합은 `tools/roundtrip_fidelity_harness.py` 또는 `render-diff`로 별도 대조한다.

### `export-hml <입력.hml> -o <출력.hml>`
HML 원본 문서를 의미 보존 HWPML 2.91 XML로 저장한다.
- `-o`, `--output <파일>`은 필수다.
- 입력과 출력이 같은 경로이면 원본 보호를 위해 거부한다.
- 이 명령은 HWP/HWPX 변환 명령이 아니며 입력은 `.hml`만 받는다.

### `ir-diff <파일A.hwpx> <파일B.hwp> [-s <구역>] [-p <문단>] [--summary] [--max-lines N]`
두 파일의 IR 비교(HWPX↔HWP 불일치 검출). 상세: [ir_diff_command.md](ir_diff_command.md)
- 비교: text, char_count/offsets/shapes, line_segs, controls, tab_extended, ParaShape, TabDef,
  표(page_break/outer_margin/treat_as_char/wrap/size/offset), 그림·도형(rel_to 등)

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

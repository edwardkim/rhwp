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

### `export-png <파일> [옵션]` *(native-skia feature 필요)*
HWP/HWPX → PNG(Skia raster, AI 파이프라인/VLM 연동). 상세: [export_png_command.md](export_png_command.md)
- `-o`, `-p`, `--font-path` (공통/폰트)
- `--scale <배율>` (기본 1.0), `--dpi <값>`(pHYs 메타 + scale 자동), `--max-dimension <픽셀>`(longest edge)
- `--vlm-target <프리셋>` — claude / gpt4v-low / gpt4v-high(gpt4v) / gemini / qwen-vl(qwen) / llava

### `export-pdf <파일> [-o 출력.pdf] [-p 페이지]`
HWP/HWPX → PDF (svg2pdf + pdf-writer).
- `DocumentCore::render_page_pdf_native`, `render_pages_pdf_native`, `render_document_pdf_native`
  native API와 같은 SVG-derived PDF export 경로를 사용한다.
- `-p`는 0-based 단일 페이지 선택이며, 생략하면 전체 문서를 다중 페이지 PDF로 내보낸다.
- direct/vector `PageLayerTree → PDF` backend는 아직 후속 작업이다.

### `export-text <파일> [옵션]`
페이지별 텍스트 → TXT. `-o`, `-p`.

### `export-markdown <파일> [옵션]`
페이지별 텍스트 → Markdown(.md). `-o`, `-p`.

### `export-render-tree <파일> [옵션]`
페이지별 render tree bbox JSON(레이아웃 시각 분석용). 출력 `render_tree_{NNN}.json`.
- `-o`, `-p`, `--show-para-marks`, `--show-control-codes`, `--respect-vpos-reset`
- JSON: `{type, bbox:{x,y,w,h}, children:[...]}` (Page → PageBg/Line/TextRun/Image/Table/Shape …)

---

## 2. 구조 덤프·진단 (Debug)

### `dump <파일> [--section <N>] [--para <N>]` (별칭 `-s`/`-p`)
문서 조판부호 구조 덤프. ParaShape/LINE_SEG/표·도형 속성. 상세: [dump_command.md](dump_command.md)

### `dump-pages <파일> [-p <N>] [--respect-vpos-reset]`
페이지네이션 결과(페이지별 문단/표 배치 목록 + 높이).

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

### `convert <입력.hwp|.hwpx> <출력.hwp>`
배포용(읽기전용) HWP → 편집 가능 HWP 변환.

### `ir-diff <파일A.hwpx> <파일B.hwp> [-s <구역>] [-p <문단>] [--summary] [--max-lines N]`
두 파일의 IR 비교(HWPX↔HWP 불일치 검출). 상세: [ir_diff_command.md](ir_diff_command.md)
- 비교: text, char_count/offsets/shapes, line_segs, controls, tab_extended, ParaShape, TabDef,
  표(page_break/outer_margin/treat_as_char/wrap/size/offset), 그림·도형(rel_to 등)

### `build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>`
ingest JSON(시험문제 등) → HWPX 생성. (rhwp-exam-ingest 파이프라인)

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
- `--help` 에 일부 내부 도구(test-*/gen-*/export-pdf/dump-records/build-from-ingest)가 누락되어 있을 수
  있으나, 모두 실제 동작하는 명령이다(현행화 대상).

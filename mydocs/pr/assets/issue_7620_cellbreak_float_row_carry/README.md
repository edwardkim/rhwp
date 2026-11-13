# #7620 후속 3 — 비TAC 자리 차지 표의 CellBreak 행 이월 시각 증적

## 입력과 기준

| 입력 | 기준 PDF | 쪽수 |
|---|---|---|
| `samples/issue7620/e-cellbreak-row-moves-next-page.hwp` | `pdf/issue7620/e-cellbreak-row-moves-next-page-hancomdocs.pdf` | 2(rhwp 2) |
| `samples/issue7620/f-rowbreak-row-splits-at-line.hwp` | `pdf/issue7620/f-rowbreak-row-splits-at-line-hancomdocs.pdf` | 2(rhwp 2) |

입력은 `samples/issue7620/generate-cellbreak.mjs`로 만든 생성 대조군이며 쪽 경계 나눔 값만 다르다
(e: CellBreak 1, f: RowBreak 2). 기준 PDF는 같은 입력을 한컴독스 웹 한글에서 파일 → 인쇄한 뒤
Chrome 인쇄 미리보기에서 PDF로 저장한 것이다(Producer PDFium, 편집기 빌드 번호 미확인).
입력·PDF의 SHA-256은 `samples/issue7620/MANIFEST.json`에 있다.

## 측정 조건

- 2026-11-13 재측정(리뷰 반영 후 `upstream/devel` `41c889cfd` rebase head), `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`(대표 PNG는 `--silhouette-only` 없이).
- 지표: `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣). 기준 PDF는
  pypdfium2 5.13.0(`pdfium_smooth_rgb_exact_dpi_v2`)으로 래스터화했다. 색상 경계 대조 픽셀은 모든 쪽 0이다.
- source: `upstream/devel` `41c889cfd`와 `src/`가 같은 트리(`src` tree `1400a0587`). 이 PR은 조판 코드를 바꾸지 않는다.
  Native `rhwp.exe`(native-skia, release-test) SHA-256 `8fa00b33436f1791d826df85deac0a409f6d48ba79b7408504b84413d4c26368`,
  fresh WASM `rhwp_bg.wasm`(`wasm-pack --target web`) `a0b5e8010107118ddf0d748beb3e9f166edc70eb4132e3dcb28cc0b68919429d`.
  WASM 래스터는 Chrome 157.0.8103.51(`VISUAL_SWEEP_CHROME`)이다.
- 글꼴: visual sweep 기본 웹폰트 캐시 환경(`output/pr-review/native-webfont-cache`, 함초롬돋움→HCR Dotum, 함초롬바탕→HCR Batang).
- 입력: #8939 `/RhwpLineSegTags` 표지를 더한 e·f(`samples/issue7620/README.md`). 표지 없는 원래 생성본
  (`generated_sha256`)을 같은 head에서 재면 2쪽이 Native 94.91%(e)·93.44%(f), fresh WASM 96.00%(e)·94.81%(f)로
  떨어진다. 표와 셀 글줄은 같고 「표 뒤 문단」만 8px 아래로 내려간다(빈 문단 bit 31 을 한/글 저장 속성으로 읽음).
  2026-11-11 `4ab860651` 측정(표지 없는 입력, 전쪽 100.00%)과 이번 측정의 TSV는 바이트가 같다.

## 결과(전 쪽)

| 입력 | 경로 | p1 / p2 |
|---|---|---|
| e | Native | 100.00 / 100.00 |
| e | fresh WASM | 100.00 / 100.00 |
| f | Native | 100.00 / 100.00 |
| f | fresh WASM | 100.00 / 100.00 |

- e: 1쪽에는 1행(6줄)만 있고, 쪽 바닥에 걸린 2행 6줄 전부와 3행이 2쪽 표 조각에 있다. 「표 뒤 문단」은 2쪽 표 아래.
- f: 2행은 1–5줄이 1쪽, 6줄째가 2쪽에 이어지고 3행이 뒤따른다. 두 열이 같은 줄 경계에서 나뉜다.

review PNG에서 두 입력의 표 괘선·셀 글줄·뒤 문단 위치가 PDF와 같다.

TSV 원본은 `tsv/<native|wasm>_<입력명>_silhouette.tsv`다. producer가 쓴 원본은 CRLF
(네 파일 모두 `0ee8be1a…6fdb8df0`, 155바이트)이고, 저장소 공백 검사에 맞춰 줄 끝만 LF로 바꿨다(값 동일).

## 회귀 검사

`tests/cases/issue_7620_cellbreak_float_row_carry.rs` 3건. 고친 결함이 없으므로 수정 전 실패 증거는 없다.

- 셀 글줄 18×2개는 라벨(`가2-1` = 가 열·2행·1줄)에서 얻은 독립 기대 순서로 판정한다. 열마다 «쪽 순서 →
  같은 쪽 안 윗변 순서»로 늘어놓은 실제 라벨 열이 `1-1 … 3-6`과 같아야 한다(렌더 트리 순회 순서는 쓰지 않는다).
- 셀 포함: 글줄은 라벨의 행·열과 같은 `Cell` 상자 안, 셀은 표 조각 안, 같은 셀의 이웃 글줄은 겹치지 않고,
  두 열의 같은 행·줄 글줄은 같은 높이다.
- `issue_7620_cell_line_checks_reject_counterexamples`: e·f의 실제 배치는 통과하고, 그 배치에서 만든 반례
  — 같은 쪽 `가2-1`·`가2-2` 라벨 역전, `가2-1` 10000px 아래 이동, `가2-1` 소유 셀을 1행으로 변경 — 는 각각
  내용 순서·셀 포함 검사가 거부한다.
- 돌연변이(2026-11-13, rebase head): `Table::layout_page_break()`가 CellBreak를 RowBreak로 돌려주게 임시로
  바꾸면 e 검사가 `CellBreak 표는 쪽 바닥에 걸린 2행을 자르지 않고 통째로 2쪽에 둔다 (가2-1)`에서 실패하고
  (TAC #7288 검사 2건도 실패), f·반례·기존 #7620 검사는 통과한다. 소스는 원복했다(`git status -- src` 없음).

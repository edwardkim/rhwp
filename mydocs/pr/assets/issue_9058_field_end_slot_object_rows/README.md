# #9058 누름틀 끝 표식 뒤 글자처럼 취급 표의 줄 소속 — 입력·기준 PDF·Visual Sweep 증적

## 입력

세 문서 모두 한/글 2022(`12.0.0.535`, Windows)에서 직접 만든 저장본이다. 손으로 고친 메타데이터는 없다. 개인정보 없음. #9058 에 첨부한 원본을 메인테이너가 `mydocs/pr/assets/johndoekim-20261116/issue9058/` 에 보존했고, 값 1자 문서는 같은 바이트가 이미 `samples/issue9082/` 에 있어 그 경로를 쓴다. 이 PR 은 입력·기준 PDF 사본을 새로 넣지 않는다.

빈 문서에 글상자(169.4 × 204.2mm, 세로 가운데 정렬)를 그리고, 그 안 한 문단에 누름틀과 표 2개(3×4, 3×6, 각 165.4 × 13.6mm, 글자처럼 취급)를 Enter 없이 넣었다(#9076 재현물과 같은 구성).

| 문서 | 바이트 · SHA-256 | 만든 방법 | 글상자 문단 원시 PARA_TEXT | 저장 줄 `text_start` |
| --- | --- | --- | --- | --- |
| `mydocs/pr/assets/johndoekim-20261116/issue9058/field_end_slot_textbox_with_cursor.hwp` | 18,944 · `384c46a75ea7a7f80e7baf94cc0d446c7c18717aafb45b729c3571e81e4101f5` | 누름틀 안에 커서를 둔 채 저장(안내문 글자 없음) | 시작 0 · 끝 8 · 표1 16 · 표2 24 | `[0, 24]` |
| `mydocs/pr/assets/johndoekim-20261116/issue9058/field_end_slot_textbox_with_text.hwp` | 19,456 · `4fe0ecb6c7723f752097c43696557eea02d986649bce12c300926108aa42dea2` | 누름틀에 「가나다라」 입력 | 시작 0 · 가나다라 8–11 · 끝 12 · 표1 20 · 표2 28 | `[0, 20, 28]` |
| `samples/issue9082/textbox_filled_field_tac_tables.hwp` | 18,944 · `a4dd2bd52c02ca3d25414e63e8c8af7cefc96d53b00e49fa635ed6f1610cb069` | 누름틀에 「값」 입력 | 시작 0 · 값 8 · 끝 9 · 표1 17 · 표2 25 | `[0, 17, 25]` |

세 문서 모두 저장 줄 시작이 누름틀 끝 표식(FIELD_END) 8유닛을 센 값이다. 끝 표식은 `controls[]` 에 들지 않는다.

## 기준 PDF

각 저장본을 같은 한/글 2022(`12.0.0.535`)에서 열어 **인쇄 → Microsoft Print to PDF** 로 출력했다(`Producer: Microsoft: Print To PDF`, PDF 1.7, A4 1쪽). 빌드 번호는 출력한 PC 의 한/글 정보에서 확인했다(PDF 메타데이터에는 없음). PDF 제목은 출력 당시의 파일 이름이다.

| PDF | 바이트 · SHA-256 | 표1 · 표2 위 괘선 y (96dpi px) | 글자 줄 |
| --- | --- | --- | --- |
| `…/issue9058/field_end_slot_textbox_with_cursor-hwp2022-print.pdf` | 1,250 · `616af761237655c3ebfd793a36153f032cb884bf325b88e72340b40b8da66514` | 469.92 · 536.64 | 없음 |
| `…/issue9058/field_end_slot_textbox_with_text-hwp2022-print.pdf` | 45,058 · `9eef0a599a7042ca8e6e63ef51da1458df34e6b4cb09f8da159a1fed85c30b42` | 480.48 · 547.36 | 글자 시작 (121.28, 466.72) |
| `…/issue9058/field_end_slot_textboxWitnoneText-hwp2022-print.pdf` (값 1자 문서의 인쇄) | 44,925 · `c8007709140c9ee62e25d8a9ec218e0643caedcfc1a8c10513e278d1036f5c6d` | 480.48 · 547.36 | 글자 시작 (121.28, 466.72) |

좌표는 PDF 내용 스트림의 선분에서 읽었다. 세 PDF 모두 표 왼쪽 괘선 x 는 125.12, 오른쪽은 750.24 이다. 빈 누름틀 문서는 표 두 줄, 값이 든 두 문서는 `글자 줄 · 표1 · 표2` 세 줄이다.

## 측정

- 도구: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, 인쇄 프로필, PDF 래스터 PDFium. 판정값은 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 이웃 내용 실루엣, 방법 `threshold_boundary_color_support_v1`).
- 후보: 조판 변경 `579018a7fe`. Native(native-skia) 실행 파일 SHA-256 `4d8caef4ad8936bc894104d9c15a95a33d969dcbe54402eac47fe27b9d395fb7`, fresh WASM(`scripts/wasm-pack-locked.sh --target web`, 최적화 포함) `rhwp_bg.wasm` SHA-256 `77d26f32d03b3add319dba496d7f9940628e60fb02cc15af252487350da73b8d`.
- 기준: devel `443dafb081`(#9081·#9084 병합 뒤). Native 실행 파일 SHA-256 `4dff137a6ca65e85425d8e27246a78dbb4fbdffab047b3065fac20be3b92af89`, fresh WASM `rhwp_bg.wasm` SHA-256 `f64cf3b4e70650cf590aedda1f3d405d0e60e1f908b9e762513cba15d83da773`.
- 측정일: 2026-11-16 (macOS arm64). 글꼴은 `RHWP_FONT_PATH` 로 로컬 글꼴 디렉터리를 공급했다. 값이 든 두 문서의 글자(「가나다라」, 「값」)가 결과에 들어간다.
- 전체 쪽수: 모든 문서가 rhwp 1쪽 = PDF 1쪽. 따라서 아래 1쪽 값이 각 경로의 전쪽 최저값이다.

| 문서 · 경로 | 기준 devel `443dafb081` | 후보 `579018a7fe` |
| --- | --- | --- |
| 빈 누름틀 · Native | 64.87% (64.87046, 미대응 5,600px / 내용 15,941px) | **100.00%** (0 / 15,941) |
| 빈 누름틀 · fresh WASM | 64.87% (64.87046, 5,600 / 15,941) | **100.00%** (0 / 15,941) |
| 값 4자 · Native | 58.36% (58.36085, 7,565 / 18,168) | **100.00%** (0 / 17,424) |
| 값 4자 · fresh WASM | 58.42% (58.42493, 7,565 / 18,196) | **100.00%** (0 / 17,453) |
| 값 1자 · Native | 57.23% (57.23443, 7,806 / 18,253) | **100.00%** (0 / 17,280) |
| 값 1자 · fresh WASM | 57.24% (57.24380, 7,806 / 18,257) | **100.00%** (0 / 17,284) |

### 인접 문서(변화 없어야 하는 대조군)

같은 실행에서 함께 측정했다. 기준과 후보가 정밀값까지 같다.

| 문서 | Native 기준 → 후보 | fresh WASM 기준 → 후보 |
| --- | --- | --- |
| `samples/issue9076/textbox_unfilled_field_tac_tables.hwp` (#9076, 안내문 잔재) | 100.00 → 100.00 | 100.00 → 100.00 |
| `samples/issue9076/textbox_unfilled_field_tac_tables_nofield.hwp` | 100.00 → 100.00 | 100.00 → 100.00 |
| `samples/issue8641/inspection_request_stamp_a.hwpx` (같은 꼴: 누름틀 뒤 비-TAC 표) | 98.00072 → 98.00072 | 98.85534 → 98.85534 |
| `samples/issue8641/inspection_request_stamp_b.hwpx` | 97.98330 → 97.98330 | 98.75166 → 98.75166 |

## 회귀 확인

- **전 쪽 render tree 전수 비교**: 기준·후보 release-test 바이너리로 저장소의 `.hwp`·`.hwpx` 전 쪽 render tree(`rhwp export-render-tree`)를 떠서 해시를 비교했다.
  - `samples/` 1,784건(열리는 1,781건, 암호 문서 3건 제외): 바뀐 문서는 `issue9082/textbox_filled_field_tac_tables.hwp`(위 값 1자 문서) 하나다.
  - `samples/` 밖(`mydocs/`, `tests/`, `pdf/`, `template/`, `saved/`, `tools/`, `rhwp-studio/`, `bindings/`) 1,965건(열리는 1,951건, 일부러 깨뜨린 파일·암호 문서 14건 제외, fuzz 말뭉치 제외): 바뀐 문서는 위 `issue9058/` 두 문서다.
  - 두 바이너리가 열지 못하는 문서 목록은 같다.
- **편람 gate**: `scripts/visual_sweep.py --handbook-gate`(Native, 383쪽) 기준·후보 모두 `passed`, 최저 99.97, 100.00 인 쪽 367. 두 실행의 전 쪽 TSV 가 바이트 단위로 같다(SHA-256 `261cbaa3c053a992e7058701416c4b38fb504bef61e8df22e44e5c5e7fb9a590`).

## 렌더 트리의 표 위치 (px)

| 문서 | 한/글 인쇄 표1 · 표2 위 괘선 y | 기준 devel 표1 · 표2 (x, y) | 후보 표1 · 표2 (x, y) |
| --- | --- | --- | --- |
| 빈 누름틀 | 469.92 · 536.64 | (125.00, 469.80) · **(757.70, 469.80)** | (125.00, 469.80) · (125.00, 536.70) |
| 값 4자 | 480.48 · 547.36 | **(177.00, 459.20)** · **(125.00, 480.50)** | (125.00, 480.50) · (125.00, 547.30) |
| 값 1자 | 480.48 · 547.36 | **(138.00, 459.20)** · **(125.00, 480.50)** | (125.00, 480.50) · (125.00, 547.30) |

- 기준 devel 은 빈 누름틀 문서의 표2 를 표1 옆(글상자 오른쪽 끝 밖)에 놓는다. 값이 든 문서에서는 표1 을 값 글자의 오른쪽(글자 줄)에, 표2 를 표1 의 줄에 놓는다. 두 표가 한 줄씩 앞이다.
- 후보는 세 문서 모두 한/글 저장 줄대로 놓는다. 표 위치가 한/글 인쇄 괘선과 0.2px 안에서 만난다.

## 파일

문서마다 같은 이름 규칙이다. 접두어는 `with-cursor`(빈 누름틀), `with-text`(값 4자), `one-char`(값 1자).

- `<접두어>-native-review-001.png`, `<접두어>-native-overlay-001.png` — 후보 Native 1쪽 (rhwp · 한/글 인쇄 · 비교)
- `<접두어>-wasm-review-001.png`, `<접두어>-wasm-overlay-001.png` — 후보 fresh WASM 1쪽
- `<접두어>-base-native-review-001.png` — 기준 devel Native 1쪽
- `base/{native,wasm}/`, `head/{native,wasm}/` — 위 7개 문서의 전 쪽(1쪽) 실루엣 TSV 원본(파일 이름 = 입력 파일 이름 + `_silhouette.tsv`)

overlay 의 색은 엄격 픽셀 비교다(파랑 = rhwp 만, 빨강 = PDF 만, 주황 = 양쪽 차이, 회색 = 일치). 한/글 인쇄의 2px 괘선과 rhwp 의 1px 괘선이 겹치는 자리에서 색이 남으며, 판정값은 위 2px 이웃 실루엣이다.

# #9076 글상자 안 미기입 누름틀 문단의 글자처럼 취급 표 — 입력·기준 PDF·Visual Sweep 증적

## 입력

두 문서 모두 한/글 2022(`12.0.0.535`, Windows)에서 직접 만든 저장본이다. 손으로 고친 메타데이터는 없다. 개인정보 없음.

- `samples/issue9076/textbox_unfilled_field_tac_tables.hwp` (18,944 바이트, SHA-256 `cbab3774fedc4d9944cccf6b3e365ed147c8624375f6cffd3f301967f6f6570b`)
  - 빈 문서에 글상자(169.4 × 204.2mm, 세로 가운데 정렬)를 그리고, 그 안 한 문단에 누름틀(안내문 「.」)을 넣은 뒤 Enter 없이 표 2개(3×4, 3×6, 각 165.4 × 13.6mm, 글자처럼 취급)를 넣었다.
  - 글상자 문단의 원시 PARA_TEXT(34유닛): `누름틀 시작 0 · 안내문 「.」 8 · 누름틀 끝 9 · 표1 17 · 표2 25 · 문단 끝 33`
  - 저장 줄 3개: `text_start` 0·17·25, `vertical_pos` 0·1600·6612, `line_height` 1000·4412·4412 (HWPUNIT). 첫 줄은 안내문만 있는 글자 줄이다.
  - 적재 뒤(rhwp): 안내문은 `Field::guide_residue` 로 빠져 문단에 글자가 없고 `char_count` 는 33 이다. 저장 줄은 그대로 남는다.
- `samples/issue9076/textbox_unfilled_field_tac_tables_nofield.hwp` (18,432 바이트, SHA-256 `055629eea0baa5ffe5090cf682357fc6278b34304c02cd8daefa12bd160099f9`)
  - 위 문서에서 누름틀만 지워 다른 이름으로 저장했다.
  - 글상자 문단의 원시 PARA_TEXT(17유닛): `표1 0 · 표2 8 · 문단 끝 16`
  - 저장 줄 2개: `text_start` 0·8, `vertical_pos` 0·5012, `line_height` 4412·4412.

## 기준 PDF

각 저장본을 같은 한/글 2022(`12.0.0.535`)에서 열어 **인쇄 → Microsoft Print to PDF** 로 출력했다(`Producer: Microsoft: Print To PDF`, A4 1쪽). 빌드 번호는 출력한 PC 의 한/글 정보에서 확인했다(PDF 메타데이터에는 없음). PDF 제목은 출력 당시의 파일 이름이다.

- `pdf/issue9076/textbox_unfilled_field_tac_tables-2022.pdf` (1,226 바이트, SHA-256 `3bd2fa3656ec337120898eb46cfb5ce7cbb7214fadaa7939298a8dd485df21a8`)
- `pdf/issue9076/textbox_unfilled_field_tac_tables_nofield-2022.pdf` (1,254 바이트, SHA-256 `1da00c4cc3fc2849153e96c519beae26e5c49a6eeb5e4fce4c26e5b8ed2feed0`)

두 PDF 에는 글자가 없다(안내문은 인쇄되지 않는다). 내용 스트림의 표 괘선 선분 20개가 두 PDF 에서 완전히 같다. 한/글은 누름틀 유무와 상관없이 두 문서를 똑같이 인쇄한다.

PDF 내용 스트림에서 읽은 좌표(96dpi px): 글상자 x 117.44–757.76 · y 153.76–925.44, 표1 x 125.12–750.24 · y 469.92–521.12, 표2 x 125.12–750.24 · y 536.64–588.00.

## 측정

- 도구: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, 인쇄 프로필, PDF 래스터 PDFium. 판정값은 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 이웃 내용 실루엣, 방법 `threshold_boundary_color_support_v1`).
- 후보: 코드 `de5a45e09c`(조판 변경은 `a8e97854ec`). Native(native-skia) 실행 파일 SHA-256 `3b822e5e2c23f3ae3ea2123b13e99b2248e16c91b8a2e1462d527627cff05a04`, fresh WASM(`scripts/wasm-pack-locked.sh --target web`, 최적화 포함) `rhwp_bg.wasm` SHA-256 `27fe03967731289d7b32e1db1f2e70de50605c9484f1223920c998b630c1190b`.
- 기준: devel `53da4d2da2`. Native 실행 파일 SHA-256 `230b8e29fdbd408ece8da1fee447230432be670e6bc4f9197b6788b49741b0a7`, fresh WASM `rhwp_bg.wasm` SHA-256 `8020bdc28079ad3912a7d93c84c16a333470ca11896d3318ed7f9db1e4328eea`.
- 글꼴: 두 문서에는 인쇄되는 글자가 없어 글꼴이 결과에 관여하지 않는다(표 괘선과 글상자 테두리만 있다).
- 전체 쪽수: 두 문서 모두 rhwp 1쪽 = PDF 1쪽.

| 문서 · 경로 | 기준 devel `53da4d2da2` | 후보 `de5a45e09c` | TSV |
| --- | --- | --- | --- |
| 누름틀 있음 · Native 1쪽 | 29.43% (정밀값 29.42928%, 미대응 13,033px / 내용 18,468px) | **100.00%** (미대응 0px / 내용 15,941px) | `base/native/`, `head/native/` |
| 누름틀 있음 · fresh WASM 1쪽 | 29.43% (정밀값 29.42928%, 미대응 13,033px / 내용 18,468px) | **100.00%** (미대응 0px / 내용 15,941px) | `base/wasm/`, `head/wasm/` |
| 누름틀 없음 · Native 1쪽 | 100.00% | 100.00% (미대응 0px / 내용 15,941px) | 같음 |
| 누름틀 없음 · fresh WASM 1쪽 | 100.00% | 100.00% (미대응 0px / 내용 15,941px) | 같음 |

대표 이미지를 만든 일반 모드 실행(`--pages 1`)의 `pr_review_gate` 는 후보 Native·fresh WASM 두 문서 모두 `passed` 다.

## 괘선 직접 측정 (96dpi, px)

같은 실행의 rhwp PNG 와 PDF 래스터에서 괘선이 놓인 행·열을 읽었다(누름틀 있는 문서). 한/글 인쇄의 괘선은 2px 두께로 래스터된다.

| 괘선 | 한/글 인쇄 | 기준 devel | 후보 Native | 후보 fresh WASM |
| --- | --- | --- | --- | --- |
| 표1 가로선 y | 469–470 · 486–487 · 503–504 · 520–521 | 458 · 475 · 492 · 509 | 469 · 486 · 503 · 520 | 같음 |
| 표2 가로선 y | 536–537 · 553–554 · 570–571 · 587–588 | 보이지 않음 | 536 · 553 · 570 · 587 | 같음 |
| 표1 세로선 x | 124–125 · 280–281 · 437–438 · 593–594 · 749–750 | 124 · 281 · 437 · 593 · 749 | 124 · 281 · 437 · 593 · 749 | 같음 |
| 표2 세로선 x | 124–125 · 228–229 · 333 · 437–438 · 541–542 · 645–646 · 749–750 | 보이지 않음 | 124 · 229 · 333 · 437 · 541 · 645 · 749 | 같음 |

- 기준 devel 은 두 표를 안내문만 있던 첫 저장 줄에 나란히 놓는다. 표1 은 한/글보다 11px 위에 있고, 표2 는 글상자 오른쪽 끝(x 757.7)에서 시작해 글상자 밖이라 보이지 않는다(렌더 트리의 표2 `x=757.70 y=459.20`).
- 후보는 표1 을 첫 줄, 표2 를 둘째 줄에 놓는다. 렌더 트리의 표 위치는 표1 `x=125.00 y=469.80`, 표2 `x=125.00 y=536.70` 이고 누름틀 없는 문서와 같다. 모든 괘선이 한/글 인쇄와 1px 안에서 만난다.
- 누름틀 없는 문서는 기준·후보의 괘선이 위 「후보」 값과 같다.

## 파일

- `native-review-001.png`, `native-overlay-001.png` — 누름틀 있는 문서, 후보 Native 1쪽 (rhwp · 한/글 인쇄 · overlay)
- `wasm-review-001.png`, `wasm-overlay-001.png` — 누름틀 있는 문서, 후보 fresh WASM 1쪽
- `base-native-review-001.png` — 누름틀 있는 문서, 기준 devel Native 1쪽
- `nofield-native-review-001.png` — 누름틀 없는 문서, 후보 Native 1쪽
- `head/native/`, `head/wasm/`, `base/native/`, `base/wasm/` — 두 문서의 전 쪽(1쪽) 실루엣 TSV 원본

overlay 의 색은 엄격 픽셀 비교다(파랑 = rhwp 만, 빨강 = PDF 만, 주황 = 양쪽 차이, 회색 = 일치). 한/글 인쇄의 2px 괘선과 rhwp 의 1px 괘선이 겹치는 자리에서 색이 남으며, 판정값은 위 2px 이웃 실루엣이다.

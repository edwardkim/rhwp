# #9021 글자 없는 글상자 문단의 글자처럼 취급 표 — 입력·기준 PDF·Visual Sweep 증적

## 입력

- `samples/issue9021/textbox_tac_tables_two_lines.hwp` (17,920 바이트, SHA-256 `e0cd85dad3edbd8d00a4c37c2d3c78bc3912524f9527baac1cab6d23943df959`)
- 한/글 2022(`12.0.0.535`, Windows)에서 직접 만든 저장본이다. 빈 문서에 글상자 하나를 넣고, 그 안 한 문단에 Enter 없이 표 3개(표1·표2 는 좁게, 표3 은 상자 폭에 가깝게)를 넣었다. 손으로 고친 메타데이터는 없다. 개인정보 없음.
- 글상자 문단(본문 문단 1 의 사각형, 글상자 문단 0)
  - 글자 없음: `char_count` 25 = 표 3개 × 8 + 문단 끝. 조절 문자는 글자처럼 취급 표 3개뿐이다.
  - 저장 줄 2개: `text_start` 0·16, `vertical_pos` 0·2731, `line_height` 2131·1848 (HWPUNIT)
  - 표 폭 21126·23390·45464, 표 높이 1565·1282·1282, 위·아래 바깥여백 283 (HWPUNIT)
  - 첫 줄 높이 2131 = 표1 높이 1565 + 바깥여백 283 × 2 → 표1 이 첫 줄의 높이를 정한다.

## 기준 PDF

- `pdf/issue9021/textbox_tac_tables_two_lines-2022.pdf` (1,198 바이트, SHA-256 `93b58ab9db87459cb699b1c3da773db0b013fcb4c6122c979881d5a21b07a018`)
- 위 저장본을 같은 한/글 2022(`12.0.0.535`)에서 열어 **인쇄 → Microsoft Print to PDF** 로 출력했다(`Producer: Microsoft: Print To PDF`, A4 1쪽). PDF 제목 `3gridInOneTextBox.hwp` 는 출력 당시의 파일 이름이다(내용은 위 입력과 바이트 동일). 빌드 번호는 출력한 PC 의 한/글 정보에서 확인했다(PDF 메타데이터에는 없음).

## 측정

- 도구: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, 인쇄 프로필, PDF 래스터 PDFium. 판정값은 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 이웃 내용 실루엣, 이진화 232, 방법 `threshold_boundary_color_support_v1`).
- 후보: 코드 `c6f859254`. Native(native-skia) 실행 파일 SHA-256 `860ea26e4a5a697ace767084c58df07be5df80a43d36c6dc0fc7745e9eeb69c2`, fresh WASM(`scripts/wasm-pack-locked.sh --target web`, 최적화 포함) `rhwp_bg.wasm` SHA-256 `092632c6d9fef9caf782d1e3e20c4d38535b9b7e65dc354206ab3c786b8672b6`.
- 기준: devel `fa32a1633`. Native 실행 파일 SHA-256 `569a824be2d23d0dac3d4487ed262707853d4d87acddaa534141d0f57ee70da5`, fresh WASM `rhwp_bg.wasm` SHA-256 `68b270118fec07180e707250e8bfc15711d57d2b526cfe28be5a479b95abacb5`.
- 글꼴: 이 문서에는 글자가 없어 글꼴이 결과에 관여하지 않는다(표 괘선과 글상자 테두리만 있다).
- 전체 쪽수: rhwp 1쪽 = PDF 1쪽.

| 경로 | 기준 devel `fa32a1633` | 후보 `c6f859254` | TSV |
| --- | --- | --- | --- |
| Native 1쪽 | 61.61% (정밀값 61.61307%, 미대응 4,393px / 내용 11,444px) | **100.00%** (미대응 0px / 내용 10,788px) | `base/native/`, `head/native/` |
| fresh WASM 1쪽 | 61.61% (정밀값 61.61307%, 미대응 4,393px / 내용 11,444px) | **100.00%** (미대응 0px / 내용 10,788px) | `base/wasm/`, `head/wasm/` |

대표 이미지를 만든 일반 모드 실행(`--pages 1`)의 `pr_review_gate` 는 Native·fresh WASM 모두 `passed` 다.

## 괘선 직접 측정 (96dpi, px)

같은 실행의 rhwp PNG 와 PDF 래스터에서 표 괘선이 놓인 행·열을 읽었다. 한/글 인쇄의 괘선은 2px 두께로 래스터된다.

| 괘선 | 한/글 인쇄 | 기준 devel | 후보 Native | 후보 fresh WASM |
| --- | --- | --- | --- | --- |
| 표1 윗변 y | 540 | 539 | 539 | 539 |
| 표2 윗변 y | 543–544 | 539 | 543 | 543 |
| 표1·표2 아랫변 y | 560–561 | 표1 560 · 표2 556 | 560 | 560 |
| 표3 윗변 y | 576–577 | 539 (첫 줄) | 576 | 576 |
| 표3 아랫변 y | 593–594 | 556 (첫 줄) | 593 | 593 |
| 첫 줄 세로선 x | 130–131 · 270–271 · 411–412 · 419–420 · 575–576 · 731 | 130 · 271 · 411 · 419 · 575 · 731 · 738(표3) | 130 · 271 · 411 · 419 · 575 · 731 | 같음 |
| 둘째 줄 세로선 x | 130–131 · 433–434 · 736–737 | 없음 | 130 · 433 · 736 | 같음 |

- 기준 devel 은 표3 을 첫 줄 끝(x 738 부터)에 이어 붙여 글상자 오른쪽 끝(x 743–744) 밖으로 내보낸다.
- 후보는 표3 을 둘째 줄에 놓고, 낮은 표2 를 표1 과 같은 기준선에 앉힌다. 모든 괘선이 한/글 인쇄와 1px 안에서 만난다.

## 파일

- `native-review-001.png`, `native-overlay-001.png` — 후보 Native 1쪽 (rhwp · 한/글 인쇄 · overlay)
- `wasm-review-001.png`, `wasm-overlay-001.png` — 후보 fresh WASM 1쪽
- `base-native-review-001.png` — 기준 devel Native 1쪽
- `head/native/`, `head/wasm/`, `base/native/`, `base/wasm/` — 전 쪽(1쪽) 실루엣 TSV 원본

overlay 의 색은 엄격 픽셀 비교다(파랑 = rhwp 만, 빨강 = PDF 만, 주황 = 양쪽 차이, 회색 = 일치). 한/글 인쇄의 2px 괘선과 rhwp 의 1px 괘선이 겹치는 자리에서 색이 남으며, 판정값은 위 2px 이웃 실루엣이다.

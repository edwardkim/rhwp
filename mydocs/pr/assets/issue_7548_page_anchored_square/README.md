# 쪽 기준 어울림 표 — 공개 합성 입력 증적 (#7548 2단계)

**합성 입력 증적이며 비공개 원본 문서의 일치를 주장하지 않는다.**

| 항목 | 값 |
|---|---|
| 입력 | `samples/page_anchored_square/page_anchored_square.hwp` (sha256 `36571e1991a5fdb9f0646284ab92a47db2e9a697e70bd2ff15d1bce1b222936b`) |
| 생성 | `python3 samples/page_anchored_square/make_page_anchored_square.py <출력.hwpx>`(저장소 루트, 결정적 — 두 번 생성 해시 동일) → hwp2024Convert MCP engine 2020(Hancom 11.0.0.9136)으로 HWP 저장. 저장 LineSeg 는 한/글이 쓴 값 |
| 뼈대 | `samples/issue2527_empty_linesegs.hwpx` 의 secPr·colPr·header·패키지 파일. rhwp 출처 표식 없음 |
| 형상 | host 문단(본문 있음)에 3x4 어울림 표: treatAsChar=0, SQUARE, vertRelTo=PAGE vertOffset=13000, horzRelTo=PAGE CENTER, 폭 = 본문 폭(옆 공간 없음), outMargin 140/140/140/852 |
| 한/글 저장 결과 | host 뒤 pi=4~7 은 저장 vpos 6400~11200(표 위), 표 띠와 겹치는 pi=8 부터 19992(띠 아래) |
| 정본 PDF | `pdf/page_anchored_square/page_anchored_square-2020.pdf` (sha256 `736a7b58e5ba418b05bddbf0f154f683e3c4c16a0444a70059564084c82a0af7`) — engine 2020, Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, 내장 HCRBatang·Haansoft Batang, 1쪽 |
| 비교 가능성 | `scripts/oracle_comparability.py`: 비교가능_주의 — 용지 일치, fontScale 일치(0.9952, 26줄). 선언 face 를 "대체"로 셌으나 내장 HCRBatang 은 함초롬바탕의 영문 이름. 이 증적은 세로 위치를 본다 |

| 측정 (96dpi, 2px 관용 실루엣) | before (#7557 head 144271afb) | after (#7548 2단계) |
|---|---:|---:|
| Native | 38.47 | **99.81** |
| fresh WASM | — | **99.81** |

바이너리·WASM sha256: after native `57053dad…`, after WASM `2728e096…`.

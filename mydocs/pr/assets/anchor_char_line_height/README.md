# 비-글자취급 개체 기준 문자 줄 높이 — 공개 합성 입력 증적

**합성 입력 증적이며 원본(비공개) 문서의 일치를 주장하지 않는다.**

| 항목 | 값 |
|---|---|
| 입력 | `samples/anchor_char_line_height/anchor_char_height.hwpx` (sha256 `e33ebcbe83ac24768367d4e4ef3d1d06c9f315075283a87ecf2342f39bb7517e`) |
| 생성 | `python3 samples/anchor_char_line_height/make_anchor_char_height.py <출력>` (저장소 루트, 결정적 — 두 번 생성 해시 동일) |
| 뼈대 | `samples/issue2527_empty_linesegs.hwpx` 의 secPr·colPr·header·패키지 파일. rhwp 출처 표식(META-INF/rhwp-hwp5-origin)은 넣지 않음 |
| 저장 정보 | 모든 문단에 `linesegarray` 없음. 글자모양 id0=10pt, id1=12pt, id2=8pt. 표 2개: 2×3, treatAsChar=0, TOP_AND_BOTTOM, vertRelTo=PARA offset 0, outMargin 140/140/140/852. host 문단의 표 run 은 id1(12pt), 같은 줄 텍스트 run 은 id2(8pt) |
| 정본 PDF | `pdf/anchor_char_line_height/anchor_char_height-2020.pdf` (sha256 `c6c0d63a1cb7276ea7e70b9b651d8664e27c6d0c09469a3b84c9b582d2ae565a`) — hwp2024Convert MCP engine 2020 (Hancom 11.0.0.9136), Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, 내장 글꼴 HCRBatang·Haansoft Batang, 1쪽 |
| 비교 가능성 | `scripts/oracle_comparability.py`: 비교가능_주의 — 용지 일치, fontScale 일치(0.9952, 19줄). 선언 face `함초롬바탕` 을 "대체"로 셌으나 내장 글꼴은 그 영문 이름 HCRBatang 이다. 이 증적은 가로 폭이 아니라 세로 줄 위치를 본다 |

| 측정 (96dpi, 2px 관용 실루엣) | before (stream/devel e1ecaa248) | after (533466568) |
|---|---:|---:|
| Native | 46.58 (re_review_required) | **99.69** (passed) |
| fresh WASM | 46.58 (re_review_required) | **99.69** (passed) |

바이너리·WASM sha256: before native `9794123c…`, before WASM `1a7b78a8…`; after native `1834f45e…`, after WASM `4fb84c95…`.

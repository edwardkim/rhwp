# samples/issue9000 — 중첩 표 저장 쪽 프레임 경계의 큰 표 쪽 컷

[#9000](https://github.com/edwardkim/rhwp/issues/9000)의 정식 회귀 fixture다.

10k 공개 코퍼스의 한국주소포럼(KAF) 총회 회의자료(docs#881, HWP 5.0.2.2, 66쪽)에서 표 뒤 내용을
지운 **파생본**(44쪽)이다. 원본의 부록 쪽 48·50~52·54·64쪽은 표와 무관한 별개 결함
([#9379](https://github.com/edwardkim/rhwp/issues/9379) — 글 상자 칸 줄 간격, 그림 배율·자르기)으로
90% 미만이라, 표가 걸친 33~44쪽만 남긴 입력으로 검증한다.

| 항목 | 값 |
| --- | --- |
| 원본 | `1311000-201100031_D0150014-2-001_한국주소포럼(KAF) 총회 회의자료(한글)110421.hwp`, 2,900,992 bytes, SHA-256 `8014ae9b…f6c9e5`, 66쪽 |
| 생성 | `rhwp edit delete-paragraph --section 1 --para N` 을 N = 244 … 33 순서로 적용(표 뒤 45~66쪽 내용 삭제). 구역 1 마지막 빈 문단은 남는다 |
| 보존 | 1~44쪽 `dump-pages` 조판 항목은 원본과 같다(44쪽 끝에 남은 빈 문단만 추가) |
| 대상 표 | 구역 1 문단 32, 11×6 RowBreak, 33~44쪽. 칸 c5 의 중첩 표 행이 한/글이 저장한 쪽 프레임(vpos 0 재시작)에서 이어진다 |

- 회귀 계약: [MANIFEST.json](MANIFEST.json)의 `contract`, 검사는
  `tests/cases/issue_9000_nested_frame_row_cuts.rs`.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기.
- 기준 엔진: **2020** — 원본의 공개 기준 PDF 도 한/글 2018(11.0.0.1623) 계열 `Hancom PDF` 인쇄본이다.

기준 PDF: [`pdf/issue9000/kaf_table_section-2020.pdf`](../../pdf/issue9000/kaf_table_section-2020.pdf)
— 738,730 bytes, SHA-256 `3c526bf9968c9ec90b5174963492762c57662ab16c2c09c62ca182b55d9ed899`,
한/글 2020(11.0.0.1623) `Hancom PDF` Print(`hancom2020_pdf_driver_one_up`), 44쪽.

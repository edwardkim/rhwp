# samples/issue9379 — 글꼴에 어울리는 줄 높이 글 상자, 원래 크기 전체를 자른 WMF 그림

[#9379](https://github.com/edwardkim/rhwp/issues/9379)의 정식 회귀 fixture다.

10k 공개 코퍼스의 한국주소포럼(KAF) 총회 회의자료(docs#881, HWP 5.0.2.2, 66쪽)에서 33~44쪽의
큰 표 문단(구역 1 문단 32, [#9000](https://github.com/edwardkim/rhwp/issues/9000))만 지운
**파생본**(54쪽)이다. 그 표 쪽은 #9000 이 고치므로, 부록 쪽만 이 이슈의 범위로 검증한다.

| 항목 | 값 |
| --- | --- |
| 원본 | `1311000-201100031_D0150014-2-001_한국주소포럼(KAF) 총회 회의자료(한글)110421.hwp`, 2,900,992 bytes, SHA-256 `8014ae9b…f6c9e5`, 66쪽 |
| 생성 | `rhwp edit delete-paragraph --section 1 --para 32` (표 문단 하나 삭제) |
| 보존 | 원본 45~66쪽 내용이 33~54쪽이 된다 |
| 글 상자 | 36·40·42쪽(원본 48·52·54쪽) 1×1 글 상자, 문단 모양 bit 22(글꼴에 어울리는 줄 높이), 함초롬바탕 13pt, 192% |
| WMF 그림 | 38·39·52쪽(원본 50·51·64쪽), 표 116 기준 이미지 크기 없음, 자르기 = 메타파일 원래 크기 전체 |

- 회귀 계약: [MANIFEST.json](MANIFEST.json)의 `contract`, 검사는
  `tests/cases/issue_9379_kaf_appendix_textbox_wmf.rs`.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기.
- 기준 엔진: **2020** — 원본 저장 제품(한/글 2010 계열)에 맞춰 한/글 2020 으로 인쇄했다.

기준 PDF: [`pdf/issue9379/kaf_appendix-2020.pdf`](../../pdf/issue9379/kaf_appendix-2020.pdf)
— 812,384 bytes, SHA-256 `7787910c514b5947639fb5857e1a5847616000115f18210a76eff05765c07a2c`,
한/글 2020(11.0.0.1623) `Hancom PDF` Print(`hancom2020_pdf_driver_one_up`), 54쪽.

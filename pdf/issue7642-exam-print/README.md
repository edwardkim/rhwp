# 이슈 #7642 시험지 16개와 독립 한컴 Print PDF

사용자가 지정한 HWP 11개·HWPX 5개를 변경하지 않고 개별 MCP Print로 출력했다.
원본은 기존 `samples/`의 추적 파일이다. 이 폴더는 대응 PDF와 입력·출력·작업 출처를 보존한다.
새 회귀 fixture/golden이나 시각 개선 완료 판정이 아니다. 전체 페이지 TSV·로그·중간 JSON은
ignored `output/pr-review/pr7600/`에 보존하며 Git에 포함하지 않는다.

실행·보정 계획은 [확장 입력 검증 기록](../../mydocs/working/task_m100_7642_stage2.md)에 연결한다.

## source `302be66a1`의 Native·fresh WASM 최초 전체 비교

| ID | 원본 | Print PDF | 전체 비교 쪽수 | Native·fresh WASM 최저 | 90% 미만 쪽 |
| --- | --- | --- | ---: | ---: | --- |
| exam-01 | [`21_언어_기출_편집가능본.hwp`](../../samples/21_언어_기출_편집가능본.hwp) | [exam-01-21_언어_기출_편집가능본-hwp-print-2020.pdf](exam-01-21_언어_기출_편집가능본-hwp-print-2020.pdf) | 15 | 60.58931% | 1 |
| exam-02 | [`exam_eng.hwp`](../../samples/exam_eng.hwp) | [exam-02-exam_eng-hwp-print-2020.pdf](exam-02-exam_eng-hwp-print-2020.pdf) | 8 | 81.58687% | 1 |
| exam-03 | [`exam_kor.hwp`](../../samples/exam_kor.hwp) | [exam-03-exam_kor-hwp-print-2020.pdf](exam-03-exam_kor-hwp-print-2020.pdf) | 20 | 90.61634% | 없음 |
| exam-04 | [`exam_math.hwp`](../../samples/exam_math.hwp) | [exam-04-exam_math-hwp-print-2020.pdf](exam-04-exam_math-hwp-print-2020.pdf) | 20 | 68.13553% | 1, 2, 4, 6, 7, 8, 9, 11, 12, 13, 15, 16, 17, 18 |
| exam-05 | [`exam_math_8.hwp`](../../samples/exam_math_8.hwp) | [exam-05-exam_math_8-hwp-print-2020.pdf](exam-05-exam_math_8-hwp-print-2020.pdf) | 1 | 67.21742% | 1 |
| exam-06 | [`exam_math_no.hwp`](../../samples/exam_math_no.hwp) | [exam-06-exam_math_no-hwp-print-2020.pdf](exam-06-exam_math_no-hwp-print-2020.pdf) | 20 | 67.38112% | 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 15, 16, 17, 18 |
| exam-07 | [`exam_science.hwp`](../../samples/exam_science.hwp) | [exam-07-exam_science-hwp-print-2020.pdf](exam-07-exam_science-hwp-print-2020.pdf) | 4 | 89.91304% | 4 |
| exam-08 | [`exam_social.hwp`](../../samples/exam_social.hwp) | [exam-08-exam_social-hwp-print-2020.pdf](exam-08-exam_social-hwp-print-2020.pdf) | 4 | 96.42156% | 없음 |
| exam-09 | [`exam-kor-2p.hwp`](../../samples/exam-kor-2p.hwp) | [exam-09-exam-kor-2p-hwp-print-2020.pdf](exam-09-exam-kor-2p-hwp-print-2020.pdf) | 2 | 98.18705% | 없음 |
| exam-10 | [`exam-kor-3p.hwp`](../../samples/exam-kor-3p.hwp) | [exam-10-exam-kor-3p-hwp-print-2020.pdf](exam-10-exam-kor-3p-hwp-print-2020.pdf) | 3 | 97.25974% | 없음 |
| exam-11 | [`exam-kor-4p.hwp`](../../samples/exam-kor-4p.hwp) | [exam-11-exam-kor-4p-hwp-print-2020.pdf](exam-11-exam-kor-4p-hwp-print-2020.pdf) | 4 | 97.21023% | 없음 |
| exam-12 | [`exam_kor.hwpx`](../../samples/hwpx/exam_kor.hwpx) | [exam-12-exam_kor-hwpx-print-2020.pdf](exam-12-exam_kor-hwpx-print-2020.pdf) | 20 | 90.61634% | 없음 |
| exam-13 | [`exam_social.hwpx`](../../samples/hwpx/exam_social.hwpx) | [exam-13-exam_social-hwpx-print-2020.pdf](exam-13-exam_social-hwpx-print-2020.pdf) | 4 | 95.16241% | 없음 |
| exam-14 | [`exam-kor-2p.hwpx`](../../samples/hwpx/exam-kor-2p.hwpx) | [exam-14-exam-kor-2p-hwpx-print-2020.pdf](exam-14-exam-kor-2p-hwpx-print-2020.pdf) | 2 | 98.18705% | 없음 |
| exam-15 | [`exam-kor-3p.hwpx`](../../samples/hwpx/exam-kor-3p.hwpx) | [exam-15-exam-kor-3p-hwpx-print-2020.pdf](exam-15-exam-kor-3p-hwpx-print-2020.pdf) | 3 | 97.25974% | 없음 |
| exam-16 | [`exam-kor-4p.hwpx`](../../samples/hwpx/exam-kor-4p.hwpx) | [exam-16-exam-kor-4p-hwpx-print-2020.pdf](exam-16-exam-kor-4p-hwpx-print-2020.pdf) | 4 | 97.21023% | 없음 |

16개·134쪽을 비교했다. 10개 문서는 전체 페이지 수치가 90% 이상이고 6개는 미달이다.
원문/PDF 전체 쪽수는 각 입력에서 같다. 같은 source의 새 WASM 브라우저 실행도 전체 쪽을 비교했으며
모든 최저값과 미달 쪽이 Native와 같다. 직접 확대 판독은 보정 기록에서 별도로 추적한다.
90% 통과만으로 수식·간격·내용 소속의 정확성이나 PR 승인·회귀 변경 완료를 판정하지 않는다.

## 개별 입력과 Print 출처

### exam-01 — 21_언어_기출_편집가능본.hwp

- 원본: `samples/21_언어_기출_편집가능본.hwp`; SHA-256 `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15`.
- Print PDF SHA-256: `68aa48724b4d6f68ed04ef30d6be5d67dcd3bcf57c916aa131f7ad15c2d81ba8`.
- MCP job: `6cc99b2a-79f9-4572-834f-75f0f4f7cfca`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 15쪽.
- 원본 저장 메타데이터: `hancom-office-2020` / `11.0.0.2129`.

### exam-02 — exam_eng.hwp

- 원본: `samples/exam_eng.hwp`; SHA-256 `7a5755a2f773fce4d295cbfeb1c5d722edb02c7f920bb067fa56940e8cd6a05b`.
- Print PDF SHA-256: `08b34c32089b73b5535bd59c38171e698e0b11932c66c4172c54b01a796b2102`.
- MCP job: `275e6658-d16c-4eae-8d25-3d714b2a1e16`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 8쪽.
- 원본 저장 메타데이터: `hancom-office-2022` / `12.0.0.3650`.

### exam-03 — exam_kor.hwp

- 원본: `samples/exam_kor.hwp`; SHA-256 `0315576fb25dd29ad3b6b188ee2539d0e8d31c15b74847be801c2186a97aac69`.
- Print PDF SHA-256: `72341eae4b56ca18894129e244ad00170991d91e7262bd3983c00720cd872972`.
- MCP job: `93228b23-c8e3-4c1c-b36c-563af7556897`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 20쪽.
- 원본 저장 메타데이터: `hancom-office-2022` / `12.0.0.4204`.

### exam-04 — exam_math.hwp

- 원본: `samples/exam_math.hwp`; SHA-256 `e40e3d675373c8efb3a844fc71f209600d3b0db987a04b3808b8e74a6b1671fe`.
- Print PDF SHA-256: `21e1b6930f2fbd9a8be3c1d839f45be84396697b03d4b3b682efa23d765d836b`.
- MCP job: `e3ed2235-53ed-40c2-b823-80c935a935fe`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 20쪽.
- 원본 저장 메타데이터: `hancom-office-2022` / `12.0.0.3747`.

### exam-05 — exam_math_8.hwp

- 원본: `samples/exam_math_8.hwp`; SHA-256 `f29e0d92c45c51306a482c9304e70820a18f55ec8c44b6cdffc26db0e54645ff`.
- Print PDF SHA-256: `a4aab7757207e2cc4838afd739641d5ea1546f3b0a57944a5dbca3e7bf3e2d6f`.
- MCP job: `9db2d5f0-ac4b-4ff1-a5a1-e2db4aa554a4`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 1쪽.
- 원본 저장 메타데이터: `hancom-office-2022` / `12.0.0.4426`.

### exam-06 — exam_math_no.hwp

- 원본: `samples/exam_math_no.hwp`; SHA-256 `f4cb91a4deee10c7f2dd704fc96e500c9597c45cb430af9aa730014eb06bbeea`.
- Print PDF SHA-256: `c24467189b8e27ca780417865df3d388c3bbdda77d780acb409909c9566c8125`.
- MCP job: `22620f76-2080-4996-920a-bff4c266ee7d`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 20쪽.
- 원본 저장 메타데이터: `hancom-office-2010` / `8.0.0.466`.

### exam-07 — exam_science.hwp

- 원본: `samples/exam_science.hwp`; SHA-256 `22d29786a80d68a9b2ad9294c2dab4915e0eced941e790e37390b14312b8b6a8`.
- Print PDF SHA-256: `1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14`.
- MCP job: `d6a82fe2-4f2f-47ea-9a29-8012b8816fe4`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 4쪽.
- 원본 저장 메타데이터: `hancom-office-2022` / `12.0.0.3747`.

### exam-08 — exam_social.hwp

- 원본: `samples/exam_social.hwp`; SHA-256 `ef1f84e7dd213194e68c243a67b0b60022472d9fa81e4db8dcf9bd9be0825146`.
- Print PDF SHA-256: `9cf208b8d0c2e5b9b814f1c6def5d28370d38e2c120b984590fdd3326d1c8799`.
- MCP job: `09efb75a-3bc4-47f0-8b23-e1c64f594074`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 4쪽.
- 원본 저장 메타데이터: `hancom-office-2022` / `12.0.0.3747`.

### exam-09 — exam-kor-2p.hwp

- 원본: `samples/exam-kor-2p.hwp`; SHA-256 `bdf208426c35ec93e846c3f40651d26ae4b017d2cf82765a71bb03df38dc9326`.
- Print PDF SHA-256: `25392bf95b480343f31858570f051f955b35b96e20346ceeb4b5c4c1d030db1a`.
- MCP job: `15015549-c22b-4a90-83d2-2ad6111126be`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 2쪽.
- 원본 저장 메타데이터: `hancom-office-2020` / `11.0.0.8969`.

### exam-10 — exam-kor-3p.hwp

- 원본: `samples/exam-kor-3p.hwp`; SHA-256 `1352c06c35db6425dab62e5c5c9c0848d224d914401d8f17c7cd16a56e853a33`.
- Print PDF SHA-256: `84e068dade902500a5479ffaca6bac557d6ae1c9af8f123ffe246055dcc70a96`.
- MCP job: `fe76c45d-d4c1-4948-91cb-ca773f30b724`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 3쪽.
- 원본 저장 메타데이터: `hancom-office-2020` / `11.0.0.8969`.

### exam-11 — exam-kor-4p.hwp

- 원본: `samples/exam-kor-4p.hwp`; SHA-256 `530813c78612d78e2443faf9674df16b622ae3bde0d8cefc77fe120f784794e6`.
- Print PDF SHA-256: `859dc09d1340730a440ce3769cebe633af024eb9884dc98b74198e9798765c54`.
- MCP job: `489ca6d4-0037-42fb-92f3-c34dfcbd0a26`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 4쪽.
- 원본 저장 메타데이터: `hancom-office-2020` / `11.0.0.8969`.

### exam-12 — exam_kor.hwpx

- 원본: `samples/hwpx/exam_kor.hwpx`; SHA-256 `c71a8991d63d084c18db1c90d2e32865b2af69cae5762b20cb554ee53a539b7c`.
- Print PDF SHA-256: `3328a5c8696aa8884ef3f7035e5fc4813773e0f8a49e1496e7bedf2179b2221f`.
- MCP job: `a7a22742-5a9e-4c23-bd6f-70fa839c73ec`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 20쪽.
- 원본 저장 메타데이터: `hancom-office-2020` / `11.0.0.8969`.

### exam-13 — exam_social.hwpx

- 원본: `samples/hwpx/exam_social.hwpx`; SHA-256 `c11b117dfc55606bb3ea703f9ea5b0a14481f7e70e918bc09ce4e202a04c9fd0`.
- Print PDF SHA-256: `ef470c55d315f900220e6bcbd83df10f374a99d2cd010ffb29abd2e214eb7000`.
- MCP job: `d3fb8c00-a218-46b2-8fc2-a41d943c889f`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 4쪽.
- 원본 저장 메타데이터: `hancom-office-2020` / `11.0.0.8969`.

### exam-14 — exam-kor-2p.hwpx

- 원본: `samples/hwpx/exam-kor-2p.hwpx`; SHA-256 `c5c4599a35e17985e24c6899840808b9da0a26a359dd2fba7b8e332b65a53c82`.
- Print PDF SHA-256: `b6a48dc980113a3a4817cf192a8d267f636db78806cc091938db3b904bea76c6`.
- MCP job: `8d842977-5107-4916-bac4-acdc6ed3d87f`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 2쪽.
- 원본 저장 메타데이터: `hancom-office-2020` / `11.0.0.8969`.

### exam-15 — exam-kor-3p.hwpx

- 원본: `samples/hwpx/exam-kor-3p.hwpx`; SHA-256 `d62e4ec0798088858b41e9e28767614ecc46988e45150cf6fec046c297f88736`.
- Print PDF SHA-256: `382de8ac44858288b4c955d8e35d7e7e733a370822434879f390ce8a5cc3ab36`.
- MCP job: `b592e64d-abe5-41a7-8cff-e999d83231f8`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 3쪽.
- 원본 저장 메타데이터: `hancom-office-2020` / `11.0.0.8969`.

### exam-16 — exam-kor-4p.hwpx

- 원본: `samples/hwpx/exam-kor-4p.hwpx`; SHA-256 `2478e53999984e6aa925a9d50c17c853ace8273863986432fe285089729f13f6`.
- Print PDF SHA-256: `7bc46b216bd1b3fddbaddbb279e1b9e2b3b9eaf8f60aa26af77d8f0f55fe6a59`.
- MCP job: `c5f35706-7f0e-4de3-963c-d1cfd2478d08`; engine `2020`, 한컴 `11.0.0.9136`.
- `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 4쪽.
- 원본 저장 메타데이터: `hancom-office-2020` / `11.0.0.8969`.

# #8682 선언 모순 표의 마지막 행 회수 증거

- base `08963f6825`, 코드 커밋 `5c290c2b4d`. 조판 코드는 `src/renderer/height_measurer.rs` 한 파일이다.
- 측정: 2026-11-06, `scripts/visual_sweep.py --silhouette-only`, 96dpi, `--compat 2022`,
  `--font-path ttfs/opensource`. 판정값은 `silhouette_raw_match_percent` 이다.
- `native/`·`wasm/` 아래 `base/`·`head/` 에 전쪽 TSV 원본을 바이트 그대로 둔다.
- `png/` 에 review·overlay PNG 와 PDF·base Native·head Native·head WASM 을 같은 배율로 나란히 둔
  확대 판독 PNG 를 둔다.

| 문서 | 쪽 | base Native | base WASM | head Native | head WASM |
|---|---|---:|---:|---:|---:|
| rental_approval_form | 1 | 70.14 | 68.71 | 100.00 | 99.91 |
| artifact_loan_form | 1 | 71.31 | 72.04 | 96.91 | 100.00 |
| ai_procurement_appendix(반례) | 1 | 99.93 | 99.99 | 99.93 | 99.99 |
| | 2 | 99.59 | 99.62 | 99.59 | 99.62 |
| | 3 | 97.29 | 97.41 | 97.29 | 97.41 |
| | 4 | 99.91 | 100.00 | 99.91 | 100.00 |

기준 PDF 는 engine 2020(hancom 11.0.0.9136, `pdf_print_method` 0, `hancom2020_pdf_driver_one_up`)이다.

| 입력 | SHA-256 | job |
|---|---|---|
| samples/issue8682/rental_approval_form.hwp | `301d6f7e7df4129ee5090d174100b1e0dfcf1b2abfb8afdcf289a0bed2beef46` | |
| pdf/issue8682/rental_approval_form-2020.pdf | `4fb0322894a091b29b429344486bdb874feb1afa85edc18837618264cf5dad82` | b32923bb-237a-4af4-99f2-4dcc07bc9593 |
| samples/issue8682/artifact_loan_form.hwp | `9dc4fc28bd951a009932298c29263e24544e455726ba9e1cc641c9fdb0a9272b` | |
| pdf/issue8682/artifact_loan_form-2020.pdf | `72ae7ff498e2371c0f316739f72a09e12cd8fc28e921f446e8f8c8a034f13b9b` | 660dc899-158e-4015-abd3-ccded3670e5b |
| samples/issue8682/ai_procurement_appendix.hwp | `8f18a357c73ff2aae4b0e2a2df3f1fecf9679aab65d137e684c7791685c09b5a` | |
| pdf/issue8682/ai_procurement_appendix-2020.pdf | `a6fe15e195cda7b85c520b69bfa53bc8a0727821ac90c1822e956e227269c68b` | 96021634-3a44-4a48-a352-a7e143db0b77 |

확대 판독: 두 표의 앞 행 괘선은 PDF 와 맞고 마지막 행이 초과분을 담는다. base 는 아래 행으로 갈수록
괘선이 위로 어긋난다. `artifact_loan_form` Native 는 base·head 모두 고딕 대체 글꼴로 나와 본문 첫 줄
폭이 PDF 와 다르다. 표 행과 무관한 글꼴 공급 차이이며 WASM 은 PDF 와 맞는다.

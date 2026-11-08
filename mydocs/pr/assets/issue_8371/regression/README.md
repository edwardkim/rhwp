# 기존 회귀 입력의 재측정

2026-10-31, 코드 `8f3d099de5b8569ae44405858d8210588ed3898e`를 Native Skia와 fresh WASM Canvas2D로 출력하고 같은 한컴 2020 Print PDF와 96 DPI에서 비교했다. 표의 값은 색상 경계 보정 전 2px 내용 실루엣의 전쪽 최저값이다. 각 TSV에는 전체 페이지가 포함된다.

| 입력 | 쪽수 | Native 최저 | WASM 최저 | 대표 쪽 |
| --- | ---: | ---: | ---: | ---: |
| `samples/issue5699/20099369_yeongwol_forms.hwp` | 4 | 99.60% | 99.59% | 1 |
| `samples/task3307/issue3307_outline_number.hwpx` | 9 | 97.89% | 97.81% | 8 |
| `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | 17 | 94.96% | 96.32% | 3 |

대표 쪽은 해당 회귀가 검사하는 위치를 보여 준다. TSV의 최저값과 직접 판독은 별도 근거다. `*-review.png`는 RHWP·PDF·overlay를 함께 표시하고 `*-overlay.png`는 겹침을 별도로 제공한다.

| 입력 또는 기준 PDF | SHA-256 |
| --- | --- |
| `samples/issue5699/20099369_yeongwol_forms.hwp` | `6d723789148699479983fbcb6fa6368a252c1187fe0a7b3c922b583f6727ec09` |
| `pdf/issue7642-failed-inputs-followup-20261011/20099369_yeongwol_forms-2020.pdf` | `a04da624238d77da4832dd302017c7d2f559a01da02e4c1b4ae12cfd3f77b123` |
| `samples/task3307/issue3307_outline_number.hwpx` | `28a60bd05e152fd5292bdc8b75c6dbd980a04375c0d8890b752e32017fd1731d` |
| `pdf/task3307/issue3307_outline_number-2020.pdf` | `0b176d3020e89d6a896b4efcab87bb9683298ea10b8593ad799bf99009668acb` |
| `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626` |
| `pdf/review-integration-20261026/issue2007_nested_cell_pagination_42065-2020.pdf` | `bb486ef1215e79c2ca0c891b2b194bdf988691b0e91b8a07ebdbffc6d749473b` |

Native 실행 파일 SHA-256은 `08f1728c75d3eabeebe1518ca6dda25e9fb39c832654817c9c6fab9705dd3686`, WASM SHA-256은 `c00f4eb8881fb217338ebe2fe370e67fc4dbb3add8f008e3bd036a2e9a52a87e`다. 문서별 기존 글꼴 카탈로그와 `compat=2022`를 사용했으며 자세한 실행 기록은 작업 문서에 남겼다.

# 법령 문서 그림 문단과 줄 배치 검증

2026-10-30에 원본 `mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx`와 독립 한컴 2018 Print PDF `mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law_hancom2018_print.pdf`를 96 DPI로 비교했다. 입력 SHA-256은 `63c5669a5fbf99bed259bb6040b3a1ec17b218e7115b867c32bd545dbb623cc1`, PDF SHA-256은 `1ce35eb7e2d6f93e41834215d6cf7653e3a09ea2fc544856b3df3a89e6d24511`이다. PDFium의 동일 쪽 래스터와 색상 경계 보정 전 2px 내용 실루엣을 대조했다.

코드 기준은 `8f3d099de5b8569ae44405858d8210588ed3898e`다. Native 실행 파일 SHA-256은 `08f1728c75d3eabeebe1518ca6dda25e9fb39c832654817c9c6fab9705dd3686`, fresh WASM SHA-256은 `c00f4eb8881fb217338ebe2fe370e67fc4dbb3add8f008e3bd036a2e9a52a87e`다. 두 경로의 쪽수는 모두 독립 PDF와 같은 6쪽이다.

| 쪽 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| 1 | 99.97% | 99.90% |
| 2 | 98.86% | 98.19% |
| 3 | 99.80% | 99.82% |
| 4 | 99.99% | 99.99% |
| 5 | 99.99% | 100.00% |
| 6 | 99.98% | 100.00% |

[Native 전체 TSV](native.tsv)와 [fresh WASM 전체 TSV](wasm.tsv)는 실행 원본이다. 1쪽의 `<개정 ...>` 줄 소속과 양쪽 정렬을 Native [review](native-review-001.png)·[overlay](native-overlay-001.png), WASM [review](wasm-review-001.png)·[overlay](wasm-overlay-001.png)로 대조했다. 6쪽 빈 문단 뒤 부칙 위치도 Native [review](native-review-006.png)·[overlay](native-overlay-006.png), WASM [review](wasm-review-006.png)·[overlay](wasm-overlay-006.png)로 대조했다. 같은 폴더의 기존 Native 2쪽 및 WASM 3쪽 이미지는 이전 중간 측정의 증거다.

한컴 2020 저장 대조에서는 원본의 생략된 문단 모양 참조가 160% 기본 줄 간격으로 해석됐다. 같은 빈 문단에 `paraPrIDRef="0"`을 명시한 합성 대조군은 참조 0번의 130%로 저장됐다. 제품은 이 두 경우를 구분하며, 회귀 검사는 글꼴 설치와 Visual Sweep에 의존하지 않고 문단의 줄 소속과 상대적 배치를 확인한다.

리베이스 후 후보의 같은 입력 5쪽을 다시 대조한 증거는 Native [review](native-review-005.png)·[overlay](native-overlay-005.png)와 fresh WASM [review](wasm-review-005.png)·[overlay](wasm-overlay-005.png)다. 원본과 한컴 Print는 위와 같으며, 각각의 출력 PNG는 [리베이스 후 전쪽 정밀 대조](../rebase-55f40/README.md)에 기록한 실행에서 가져왔다. 5쪽 raw는 Native 99.99347%, WASM 100.00%다.

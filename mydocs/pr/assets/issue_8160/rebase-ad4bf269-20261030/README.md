# #8188 리베이스 후 sample16 전쪽 검증

2026-10-30, 후보 `2ea4a0ae45ac15d2ca000088b13b5f243cb7be39`, 새 기준
`ad4bf269ffef5adf2142b33004a9981fad7ce804`. 기존 독립 한컴 Print PDF를
96dpi PDFium(pypdfium2)으로 래스터화하고 Native Skia와 fresh WASM Canvas PNG를 같은 조건의
`scripts/visual_sweep.py --silhouette-only --png-pair`로 비교했다. 각 문서는 64쪽이다.
Native 실행 파일 SHA-256 `6774543ac5c6eaf976543d5d71a43d2b5a6663860fceed48b1f4ae514a95bf8c`,
최적화 WASM SHA-256 `c494ecce8324cfc4c2bcf4d067ee48829ceda38604c0e2fbc32533da558e28a4`.
글꼴은 로컬 시스템에 설치된 프로그램과 기존 검증 카탈로그를 사용했으며 글꼴 파일은 포함하지 않는다.
원본·PDF의 SHA와 글꼴 카탈로그 구성은 [기존 정식 증거](../rebase-8231/raw100-proof.json)와 같다.

| 원본 | 독립 PDF | Native 전쪽 TSV | fresh WASM 전쪽 TSV | 전쪽 최저 Native / WASM |
| --- | --- | --- | --- | --- |
| `samples/hwp3-sample16-hwp5.hwpx` | `pdf/hwp3-sample16-hwp5-hwpx-2024.pdf` | [TSV](hwpx-native.tsv) | [TSV](hwpx-wasm.tsv) | 99.99% / 99.99% |
| `samples/hwp3-sample16-hwp5-2024.hwp` | `pdf/hwp3-sample16-hwp5-2024-hwp-2024.pdf` | [TSV](hwp2024-native.tsv) | [TSV](hwp2024-wasm.tsv) | 99.99% / 99.99% |
| `samples/hwp3-sample16-hwp5-2022.hwp` | `pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf` | [TSV](hwp2022-native.tsv) | [TSV](hwp2022-wasm.tsv) | 100.00% / 100.00% |

HWPX와 2024 HWP는 18쪽이 Native·WASM 모두 99.99%, 35쪽이 99.99% / 100.00%다.
그 밖의 쪽과 2022 HWP 전쪽은 양쪽 100.00%다. 두 경로가 모두 100.00%인 쪽에만
독립 좌표 허용 오차 ±1px을 적용하고, 18·35쪽은 ±2px을 유지한다.
[HWPX 8쪽](hwpx-native-p008-overlay.png)·[18쪽](hwpx-native-p018-overlay.png)·
[2022 HWP 35쪽](hwp2022-native-p035-overlay.png)의 Native overlay와
[기존 전쪽·확대 판독 근거](../rebase-8231/README.md)를 함께 참조한다.

100.00%는 2px 이웃 실루엣 점수이며 모든 실수 좌표가 ±1px 이내라는 뜻은 아니다.
HWPX 8쪽은 PDF 괘선 중심과 논리 표 상자의 끝을 같은 값으로 취급하던 기존 핀을
원본의 47,204 HWPUNIT 표 높이와 실제 하단 괘선 소유 검사로 정정한다. 제목·표 시작의
독립 Print 좌표는 ±1px로 유지한다. 2022 HWP 35쪽 그림은 PDF에서 여러 조각으로
출력되어 기존 PDF 경계 추정치와 원본 이미지 프레임이 같은 대상이 아니다. 왼쪽·위쪽·오른쪽의
독립 좌표는 ±1px로 유지하고, 프레임 높이는 원본 HWPUNIT과 비교한다.

# #8153 — 디코드된 색 수와 JPEG Print 대조

## 생성 출처와 독립 기준

이 두 HWPX는 RHWP의 기존 합성 대조군
`pdf/issue7642-raster-print-codec-controls-20261017/raster-picture-format-controls.hwpx`를
수정한 진단 입력이다. 한컴이 저장한 정상 생성본이라고 주장하지 않는다.
원본 편람이나 그 기준 PDF를 수정한 자료도 아니다.

그림 바이트는 `samples/2025 행정업무운영 편람(최종).hwpx`의 `BinData/image365.jpg`다.
기존 다섯 슬롯 중 1 BMP·2 RGB PNG·4 RGBA PNG는 이 JPEG를 Pillow 12.3.0에서
디코드해 해당 형식으로 저장했고, 3 JPEG는 원본 바이트 그대로다.
5는 기존 256색 indexed PNG를 유지한다. 위치·크기·글자·페이지 설정은 유지한다.
`full`의 imgClip은 두 축 0..38400, `crop`은 두 축 0..19200이다.
MCP 입력 전처리는 두 경우 모두 `none`이다.

PDF는 각각 이 입력을 독립 한컴 2024 13.0.0.3901의 Print로 출력한 한 쪽짜리다.
`pdf_print_method=0`, `pdf_output_mode=frame_print_to_pdf_ex_one_up`이며
새 코드의 출력이나 원본 편람의 개선 수치로 사용하지 않는다.
기존 편람의 두 383쪽 Print PDF는 재출력하지 않고 그대로 재사용한다.

| 입력 | MCP job | PDF |
| --- | --- | --- |
| [full HWPX](photo-format-full.hwpx) | `cc68c5e8-6b67-4dc3-a4ab-85ce5e460971` | [full Print](photo-format-full-2024.pdf) |
| [crop HWPX](photo-format-crop.hwpx) | `1ef71c45-b45f-4172-b1b2-8e1643810e70` | [crop Print](photo-format-crop-2024.pdf) |

## 관측과 기존 가정의 재검토

기존 stage2025 형식 대조군은 BMP·RGB PNG·RGBA PNG가 예약 흑백 포함 257색인데,
**JPEG는 압축·디코드 뒤 256색**이었다. 따라서 그 자료의 JPEG 값이 유지됐다는 관측만으로
JPEG 형식 전체를 색 확장 대상에서 제외할 수 없다.

새 사진 대조군의 JPEG는 full에서 14,845색, crop에서 9,484색이다.
각 형식의 디코드된 원본과 독립 PDF에서 추출한 bitmap을 같은 범위로 대조했다.
crop 출력 bitmap은 정수 절삭된 597×720이며, full은 1195×1441이다.

| 입력 | 원본과 PDF의 평균 RGB 절대 오차 | `c+(c>>5)`와 PDF의 오차 |
| --- | --- | --- |
| full BMP/PNG | 1.40222 / 1.17513 / 1.11716 | 1.02985 / 0.84355 / 0.99107 |
| full JPEG | 1.40290 / 1.17492 / 1.11618 | 1.03040 / 0.84303 / 0.98974 |
| crop BMP/PNG | 2.75448 / 2.27606 / 2.26387 | 1.87227 / 1.50656 / 1.77411 |
| crop JPEG | 2.75346 / 2.27493 / 2.26443 | 1.87227 / 1.50529 / 1.77480 |

BMP/PNG와 JPEG가 같은 색 확장 경향을 보인다. JPEG 재압축·디코드 차이는 남아 있어
색 함수가 한컴 내부 구현과 바이트 단위로 같다고 주장하지 않는다.
256색 indexed PNG는 제외 조건의 대조군이며, crop의 가시 64색은 PDF 추출값과 정확히 같다.
기존 grayscale JPEG의 256색 대조군도 제외 조건을 유지해야 한다.

구현은 기존 공통 가시 팔레트 판정에 JPEG를 포함한다. 그림 효과·2020 출력의 기존 제외
조건과 원본 신원·출력 바이트 재조회 경로는 유지한다. 실제 Native/fresh WASM 및 원본 편람의
100% 달성은 별도 재출력으로 검증하며 이 표의 평균 오차로 대신하지 않는다.

## SHA-256

```text
176d4f6948b196680c39a45763499155afa8a8740024590ce5d9d6281a84082b  photo-format-full.hwpx
e4588c3feda9e5f61848d8cc28d36a74ecc1a668d3c3c7ded296d9da08a3b554  photo-format-full-2024.pdf
0600630f171fe8ed06ae425a037bf93c930e190df0b920a4c36f94178413b152  photo-format-crop.hwpx
5984b4aaf6b5a0662af0604701ddb098edc73b4a8dfbbd2d4de7e8073068d932  photo-format-crop-2024.pdf
```

생성·MCP start/status/download receipt와 bitmap 대조 JSON은
`output/pr-review/issue8153-handbook-raw100-20261022/jpeg-crop-controls/`에 있다.

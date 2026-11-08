# 저장 그림 컷의 원점·물리 프레임 복구

- 기존 범위: [원본 HWPX](../../../../../../../samples/issue5734/cell_float_stack_stored_vpos.hwpx) 전 2쪽.
- 독립 기준: [한컴 Print PDF](../../../../../../../pdf/cell_float_stack_stored_vpos-2020.pdf) 전 2쪽.
- 원본 SHA-256: `c1fb8e471f295848b8e5ac5043dac13313e633d39a51940e22b776d09a64723b`.
- PDF SHA-256: `04265b5080350c29246efb2c8107f30fc3af599f4fab37cc73077182a30885cc`.
- 조건: 96 DPI, compat 2022, Print, PDFium 153.0.7999.0 / smooth_rgb_v1, raw 2px.
- base `7bb143879dbde80a02fb4a63692556ac28fe46ac`, parent `0c1c7a377440000e24d9ca1615486bec13b4cb87`.
  후보는 이 제출의 생산 소스이며 아래 producer로 새로 산출했다.
- Native SHA-256: `c8d5fced146da49baecea31c2d5100c1b1404cbe0c686f0e515754eb7a88525d`.
- fresh WASM SHA-256: `2148ec4d9b58ad735c56cadaa4cf818182bc83c64d510e2a382fb52265a3a791`.
- JS SHA-256: `aea5f6639676442e0628b640b75d0f70eb300481421112d23b661a082f695a68`.
  root pkg / Studio public / 봉인 pkg가 같다. 실제 Studio 초기화·actual36 완료 근거는 아니다.
- Mac locked wrapper `--target web --out-dir pkg --no-opt` 빌드이며 Docker 최적화가 아니다.

## 전쪽 TSV

| 실행 | 1쪽 raw | 2쪽 raw | 전쪽 TSV |
| --- | ---: | ---: | --- |
| 이전 Stage65 Native | 34.71813% | 61.58705% | [원본](before-native.tsv) |
| 이전 Stage65 fresh WASM | 34.72789% | 61.58705% | [원본](before-wasm.tsv) |
| 수정 Native | 100.00000% | 100.00000% | [원본](after-native.tsv) |
| 수정 fresh WASM | 100.00000% | 100.00000% | [원본](after-wasm.tsv) |

수정 전은 같은 브랜치의 Stage65 봉인 출력이다. 최신 devel 대비 전쪽2%p 비교를 대체하지 않는다.
새 실행은 각 2/2쪽이며 누락·중복·90% 미달이 없다. 원본의14개 그림은 흰색 대체 데이터다.
실제 문서의 그림 내용 피델리티를 주장하지 않으며 높은 raw값으로 그림별 소속·좌표를 대신하지 않는다.

## 직접 판독한 review / overlay

| 출력·쪽 | 수정 전 | 수정 후 |
| --- | --- | --- |
| native 1쪽 | [review](before-native-p001-review.png) · [overlay](before-native-p001-overlay.png) | [review](after-native-p001-review.png) · [overlay](after-native-p001-overlay.png) |
| native 2쪽 | [review](before-native-p002-review.png) · [overlay](before-native-p002-overlay.png) | [review](after-native-p002-review.png) · [overlay](after-native-p002-overlay.png) |
| wasm 1쪽 | [review](before-wasm-p001-review.png) · [overlay](before-wasm-p001-overlay.png) | [review](after-wasm-p001-review.png) · [overlay](after-wasm-p001-overlay.png) |
| wasm 2쪽 | [review](before-wasm-p002-review.png) · [overlay](before-wasm-p002-overlay.png) | [review](after-wasm-p002-review.png) · [overlay](after-wasm-p002-overlay.png) |

Native와 fresh WASM 양쪽의 표 시작·중간 행·마지막 선과 라벨을 직접 판독했다.
RGB 동일성이 아니다. 엄격 내용 픽셀 값은 Native 32.58/10.43%, WASM 32.63/10.43%다.
선 두께·래스터 차이를 남기며 도구 라벨의 한글과 수치를 확인했다.

## 내용 소유와 Native 회귀

기존 함수는14개 그림의 BinData 소속, 누락·중복·순서·셀 내부 포함, 완전 행 높이 비,
본문 축소 예산의 이월 보존을 계속 검사한다. 기존 함수 본문 바이트와 기대값은 그대로다.

전체2쪽이 양쪽98% 이상이고 직접 판독을 마친 뒤 `native_export_keeps_print_picture_origins_and_vertical_bounds`를
같은 기존 case에 추가했다. 실제 Native CLI RenderTree에서 원본 쪽·셀·그림 순서로 선택하고
독립 PDF의 x0/y0/y1을 CSS 96 ±2px로 검사한다. 현재 출력 좌표로 대상을 선택하지 않는다.
raw 100%여도 그림이 잘못된 중간v4 출력기에서는1 FAIL, 수정v7 출력기에서는1 PASS다.
정식 자동 수집 경로에서도 기존 그림·신규 좌표·계산 호스트 3건 모두PASS다.

독립 PDF 14개 이미지의 시작점·세로 끝점 차이는 양쪽 모두 최대 0.903px 미만이다.
원 해결 415단계와 Native 14개 상자가 모두 같다. 2쪽 두 번째 그림의 오른쪽 끝에는
약 4.93px 차이가 남으며 원 해결도 동일하다. 원본 선언 폭22792HU와 PDF 이미지 폭은 다르다.
이 너비를 신규 핀으로 잠그거나 네 변 모두±2px라고 주장하지 않는다.

## 정상 대조와 잔여

- 기존 민원 서식2쪽을 같은 현재 Native/fresh WASM으로 새로 산출해 각각100/100%다.
  [Native TSV](control-civil-native.tsv) · [WASM TSV](control-civil-wasm.tsv).
  각PNG는 직전 검증된65단계의 해당 backend PNG와 바이트 동일하다.
- 정식 집중 101건은 93 PASS/기존 8 FAIL/4957선택제외, 실행8.391초다. 새 실패0건이다.
  중간v6에서 새로 실패했던 계산 호스트는v7에서 복구했다.
- 신규 함수 추가 뒤 manifest 준비·현재 suite derive로 별도3 PASS/0 FAIL을 확인했다.
  1736 source /7610 attrs /48 integration targets, unit-tier4101 tests /296 modules 검사PASS다.
- 최신 전체 nextest·전체 제출 범위의 양쪽 전쪽 비교·devel2%p 비교·실제 소비 경로는 아직 남는다.
  이 단계의 통과를 브랜치 전체PR 준비 완료로 쓰지 않는다.

전체 로그·빌드 원장·14개 PDF 좌표·중간 실패·기계 검증 JSON은 ignored
`output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage66-stored-picture-frame/v7/`에
보존한다. 원본·PDF·폰트·다른 작업장·공유 target을 삭제하거나 변경하지 않았다.

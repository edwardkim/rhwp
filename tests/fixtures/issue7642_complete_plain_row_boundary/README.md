# 완결 줄의 물리 예산 경계 대조군

Issue: #7642. 새로운 회귀 이름은
`issue_7642_complete_plain_row_boundary::complete_plain_row_keeps_print_ownership_at_exact_and_one_unit_budgets`다.
기존 시험지 검사 이름이나 golden을 변경하지 않는다.

## 입력 생성과 독립 기준

두 파일은 저장 LineSeg가 없는 합성 HWPX다. 6행·2열의 각 셀은 한 줄의
`ROW r CELL c`를 가지며 선언 최소 높이는 28,500HU다. 최소 경계 원본의
용지 폭은 29,100HU이고, `headroom-1`은 그 속성만 29,101HU로 바꾼다.
`landscape=NARROWLY`이므로 해당 폭은 출력 쪽의 세로 방향이다.
생성 ZIP과 MCP Print 입력은 바이트 그대로 보존했다. 한컴 저장 LineSeg 문서라는
주장을 하지 않으며, source HU 경계의 독립 Print 계약으로 사용한다.

한컴 MCP의 `engine=2020`, `PrintMethod=0`, 한 쪽씩 인쇄로 각 원본을 직접 출력했다.
실제 실행 모드는 다음과 같다. 비공개 접속 정보는 포함하지 않는다.

| 입력 | MCP job | 실제 한컴 버전 | Print 모드 |
| --- | --- | --- | --- |
| `minimum-28500` | `8d819e8f-e8a0-4c9b-af5d-ce660ec7061c` | `11.0.0.9136` | `hancom2020_pdf_driver_one_up` |
| `headroom-1` | `e8020db2-2fc9-49dc-8367-2e876739abaf` | `11.0.0.9136` | `hancom2020_pdf_driver_one_up` |

| 파일 | 바이트 | SHA-256 |
| --- | ---: | --- |
| `minimum-28500.hwpx` | 13159 | `79a9e06762f9ebeba2b69c945b574324f85627f9a1800d79cfa9f2f1243ed4ec` |
| `minimum-28500-2020.pdf` | 16251 | `4872b98c4b23728e16b680da8b8919a3c24dde190d07a0583495681f401fec8f` |
| `headroom-1.hwpx` | 13159 | `f3559f85c7ba0bbe3296c47c601f3e1c0ec9b5772f405285acaeeff871cc076a` |
| `headroom-1-2020.pdf` | 16250 | `7c16bc7e12950ff2cade21d527660f23759301a1bd84fcd2585e9ce4c958c03e` |

## 전쪽 자격과 직접 판독

런타임 소스 SHA: `7d685cc178009c291979aa8a2ea422bed5cad58b`.
Native 빌드 `stage3377`, fresh WASM 빌드 `stage3379`, 전쪽 측정 `stage3378`,
직접 판독용 대표 PNG `stage3413`을 사용했다. PDFium 153.0.7999.0 /
pypdfium2 5.13.0, 96dpi, print profile, compat 2022다.
측정 도구 6개의 해시가 현재 저장소 도구와 같음을 확인했다.

두 문서 각각 Native와 fresh WASM **전 8쪽 최저 raw 100%**, PDF 쪽수도 8쪽이다.
텍스트·행·셀의 누락·중복과 대표 쪽의 글자·간격을 직접 확인했다.
전체 TSV 원문과 양쪽 backend의 review/overlay는
[최종 증적 폴더](../../../mydocs/pr/assets/issue7642_complete_plain_row_boundary)에 있다.
Native와 WASM은 각 하위 폴더로 구분한다. 추가 테스트·문서만 변경한 뒤에는
기존 캡처의 소스 SHA와 런타임 소스 동일성 확인을 유지하며 새 캡처로 표시하지 않는다.

## 관계 검사와 허용된 좌표 검사

PDF 전쪽의 텍스트에서 관측한 행 소유는 다음과 같다(쪽은 1부터 시작).
각 행의 두 셀은 같은 쪽에 있으며 순서는 왼쪽 셀, 오른쪽 셀이다.

| 입력 | 행 0 | 행 1 | 행 2 | 행 3 | 행 4 | 행 5 | 전체 쪽수 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| minimum-28500 | 1 | 2 | 3 | 4 | 6 | 7 | 8 |
| headroom-1 | 1 | 2 | 3 | 4 | 5 | 7 | 8 |

일반 관계 검사는 모든 셀의 본문 포함, 글줄·글자의 셀 포함, 내용 순서,
정확히 한 번의 내용 보존, 쪽 소유를 함께 검사한다. CLI JSON의 소수 첫째 자리
좌표 반올림에만 0.11의 관계 비교 허용치를 둔다.

중요한 행 4의 첫 셀은 두 backend raw 100%인 페이지에서 독립 PDF의 글자 행렬로
기준선을 고정했다. PDFium `FPDFText_GetMatrix`의 `e`, `f`를 사용하며
PDF 아래쪽 원점에서 CSS 위쪽 원점으로 변환한다: `x=e*96/72`,
`baseline=(PDF 페이지 높이-f)*96/72`. 구현이 계산한 줄 높이나 baseline으로 기대값을 만들지 않았다.

| 입력·쪽 | PDF 행렬 e | PDF 행렬 f | PDF 높이(pt) | x(CSS px) | 기준선(CSS px) | 허용 |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| minimum-28500 p6 | 50.01289367675781 | 252.4957275390625 | 291 | 66.68385823567708 | 51.339029947916664 | ±2px |
| headroom-1 p5 | 50.01289367675781 | 32.62554931640625 | 291 | 66.68385823567708 | 344.499267578125 | ±2px |

`headroom-1` 용지와 Print MediaBox의 차이는 1HU(약 0.0133px)이며,
실제 내용과 위 전쪽 점수·직접 판독으로 함께 확인했다. 별도의 +170HU 대조군은
PDF 높이가 원본보다 2px 작아 확대 차이가 남으므로 이 회귀의 적격 자료에 포함하지 않았다.
그 원본·PDF와 진단 결과는 ignored output에 보존한다.

## 정식 회귀 실행

`e579f10ecd6fda96c14a7ef40394247ebe4efbb7`의 자동 수집 suite에서 새 검사가 통과했다.
같은 테스트 실행 파일의 CLI만 바꿔 아래를 확인했다.

| 실제 CLI 소스 | 결과 | 확인한 의미 |
| --- | --- | --- |
| `7bccb3d635e2e9d1c6b891719df0927232770e33` | FAIL, exit 101 | `minimum-28500`의 행·쪽 소유 불일치 |
| `e579f10ecd6fda96c14a7ef40394247ebe4efbb7` | PASS, exit 0 | 두 입력의 관계·독립 기준선 검사 |

52건 집중 실행은 50건 PASS, 다른 표 예산 검사 2건 FAIL이다. 전체 통과가 아니다.
첫 새 검사 실행은 표 뒤 본문까지 셀 내용이라고 가정해 실패했고, 원본의
`AFTER TABLE FOLLOWER`를 별도 본문으로 검사하도록 정정했다. 이 본문의 한 번 보존,
독립 Print의 8쪽 소유, 표의 끝 조각 뒤 배치도 현재 검사에 포함된다.

실행 원문은 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`의
`stage3444-batched-flow-native-build/nextest.log`와
`stage3448-new-contract-proof/{receipt.json,before.log,after.log}`에 있다.
이 실행 결과는 위 source `7d685cc178`의 전쪽 Native/fresh WASM 캡처를 새 head에서
재산출했다는 의미가 아니다. 최종 후보의 양쪽 전쪽 시각 증거는 별도로 갱신한다.

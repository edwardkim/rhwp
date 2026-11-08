# 양쪽 정렬의 줄 가장자리 공백 대조군

`leading-tracking-contract.hwpx`는 한컴 저장본이 아닌 수동 합성 입력이다. 10pt Times New Roman으로 `alpha beta gamma delta epsilon zeta eta theta `를 3회 반복하고, 선두 공백 0·1·3개 및 자간 0·−12%의 각 조건을 Left/Justify 두 문단에 적용했다. 매 조건 앞의 명시적 쪽 나눔으로 6쪽, 12문단이다.

## 독립 기준

- 입력 SHA-256: `cb7d4ae03e319c6f33b0432a118c2c25dbfee522a4bbb88bef8868aa480d899c`
- PDF: [원문 그대로 한컴 Print로 출력한 6쪽](../../../pdf/issue7642-leading-tracking-contract/leading-tracking-contract-2020.pdf)
- PDF SHA-256: `fd7ae62cb38455e2d114d233290babd48a595bb36c0c8b8582b65d90ddbb84a4`
- 한컴 MCP engine: `2020`, job `a2492590-4fec-4e6d-9cd9-45fdda8e328c`, target `pdf`. HWP/HWPX 재저장을 기준으로 사용하지 않았다.
- 검증 구현: `810654da2af51c727bf639c3c6ecec8595d3c5f2`, 96dpi, compat `2022`, Native Skia와 fresh WASM Canvas2D. 아래 회귀 변경은 생산 코드가 같은 이 구현의 출력을 근거로 한다.

전체 6쪽에 누락·미달이 없으며 독립 PDF와 양쪽 출력의 쪽수가 같다.

| 쪽 | Native 관용 실루엣(%) | fresh WASM 관용 실루엣(%) |
| --- | ---: | ---: |
| 1 | 97.56700 | 97.47330 |
| 2 | 98.71203 | 98.63130 |
| 3 | 98.70913 | 98.63265 |
| 4 | 99.55523 | 99.64115 |
| 5 | 99.55817 | 99.64227 |
| 6 | 99.55804 | 99.64259 |

전쪽 최저값은 Native 97.56700%, WASM 97.47330%다. 앞 공백·단어 사이 간격·자동 줄바꿈·가시 끝을 같은 영역에서 직접 판독했다. 문자 소유·순서·자연 전진 관계는 전체 6쪽에서 유지한다. 공개 JSON의 소수 첫째 자리 양자화 허용 오차는 배치의 px 고정 허용치와 구분한다.

## 완료 페이지의 좌표 검사 선정

선두 공백 다음 글자의 원점을 검증하는 후보 중 100% 페이지는 없다. 양쪽 점수의 최솟값이 99%대인 5·6쪽을 우선 선정한다. 두 쪽은 자간 −12%와 선두 공백 1·3개의 경계를 보호한다. 4쪽은 선두 공백이 없으므로 같은 경계의 대체 증거로 쓰지 않는다. 자간 0%의 독립 경계는 기존 2쪽을 함께 유지한다. 같은 조건의 3쪽은 전체 관계 검사로 보호한다.

| 쪽 | 양쪽 최솟값(%) | 독립 PDF의 첫 `a` 원점(pt) | 96dpi CSS px |
| --- | ---: | ---: | ---: |
| 5 | 99.55817 | 54.501792 | 72.669056 |
| 6 | 99.55804 | 63.265296 | 84.353728 |
| 2 | 98.63130 | 55.102032 | 73.469376 |

좌표는 PDF 내용 스트림의 CTM과 텍스트 행렬을 곱해 얻은 원점이며 글리프 잉크 bbox와 구분한다. 각 쪽의 Left/Justify에서 같은 원점을 확인했다. 선정한 쪽만 ±2px로 검사한다. 소유·순서·자연 전진 관계를 유지하며, 이후에도 양쪽 98% 이상과 확대 판독을 다시 확인한다. 1쪽에는 좌표 핀을 추가하지 않는다.

실행 원장은 ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`에 있다. Native/fresh WASM 전쪽 TSV는 각각 `stage1330-contract-tsv/1314/silhouette.tsv`, `stage1330-contract-tsv/1316/silhouette.tsv`다. `stage1314-native-boundaries/leading-all-review`와 `stage1316-native-boundaries/leading-all-review`에서 새로 출력한 전체 6쪽 PNG를 `--png-pair`로 측정했다. 원문·PDF·생산자 해시·전체 쪽수는 각 실행의 manifest와 `stage1330-contract-tsv/receipt.json`에 연결되어 있다. TSV만으로 원문 쪽수를 추정하지 않는다. 독립 PDF 원점은 `stage1317-independent-leading-print-origins.json`, 같은 좌표의 확대 판독은 `stage1318-native-leading-details`와 `stage1329-wasm-leading-details`다.

WASM 검증은 Google Chrome for Testing 152.0.7977.75 실행파일 SHA-256 `3c84cfdbadd0b5b9d1943568b67faea54cf818d20ca32a337fd8a68a0d61ac76`을 명시했다. `stage1313-fresh-wasm-build.json`의 새 release 패키지로 6쪽을 다시 렌더한 결과이며 이전 캡처와 혼합하지 않았다. 원본 review·standalone overlay도 해당 실행 아래 보존한다.

이 대조군의 통과는 원래 시험지·계약서 전체나 HML 수식의 완료 증거가 아니다. 생산 코드·입력·글꼴·backend가 바뀌면 관련 페이지를 다시 측정하고 직접 판독한다.

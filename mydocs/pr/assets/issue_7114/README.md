# #7114 중첩 표 호스트를 편집한 문서의 저장 검증

`samples/issue1949_giant_cell_nested_tables_perf.hwp`와 같은 이름의 HWPX를 열어
`insertTextInCell(0, 0, 2, 2, 2276, 0, "1".repeat(120))`을 실행했다. HWP와
HWPX 각각 편집 직후, 저장본 재열기, 재저장본 재열기의 115쪽 본문 소속과 순서가
같았다. 화면 배치에 접근하기 전 저장, 접근 후 저장, 지연 페이지네이션을 완료한
후 저장을 각각 실행했다. 두 형식 모두 같은 형식으로 재저장할 때의 바이트가
같았고, HWP에서 HWPX로 또는 HWPX에서 HWP로 교차 저장한 파일도 재저장 뒤
115쪽을 유지했다. 교차 저장은 102~105쪽의 본문과 그림 소속을 비교했다.

한컴 2024(13.0.0.3901)에서 실제 저장 바이트를 읽고 PDF 드라이버 한 쪽씩 인쇄로
독립 기준을 만들었다. 같은 저장 파일을 Native Skia 및 fresh WASM Canvas로
출력해 96 DPI에서 색상 경계 보정 전 2px 실루엣을 전쪽 대조했다. 출력에 사용한
생산 코드 커밋은 `b0f7b0c60c80d8a020d3c15ccbabdfc013b9f5fa`다.

| 저장 형식 | 한컴 기준 PDF | Native 결과 | fresh WASM 결과 |
| --- | --- | --- | --- |
| HWP | [115쪽 독립 Print](../../../../pdf/issue7114-nested-save-120/hwp-120-hancom2024.pdf) | [115쪽 TSV, 최저 98.46%](native-hwp-120.tsv) | [115쪽 TSV, 최저 99.31%](wasm-hwp-120.tsv) |
| HWPX | [115쪽 독립 Print](../../../../pdf/issue7114-nested-save-120/hwpx-120-hancom2024.pdf) | [115쪽 TSV, 최저 98.46%](native-hwpx-120.tsv) | [115쪽 TSV, 최저 99.31%](wasm-hwpx-120.tsv) |

| 경계 | 눈으로 대조한 내용 |
| --- | --- |
| 102·103쪽 | 첫 74자는 102쪽, 다음 46자는 103쪽 8행 표 아래에 놓인다. 103쪽의 새 줄과 표 하단 사이가 열린다. [Native HWP 비교](native-hwp-p103.png), [Native HWPX 출력](native-hwpx-p103.png) |
| 103·104쪽 | 14행 표의 터보 과급기 행은 103쪽 하단에서 시작하고 같은 행의 영문 설명이 104쪽 첫머리로 이어진다. 이어받은 표에서 앞 문단과 뒤 행이 중복되지 않는다. [WASM HWP 비교](wasm-hwp-p104.png) |
| 104·105쪽 | 삽화는 쪽마다 한 장씩 표 내부에 그려지고 105쪽의 다음 제목이 그림 아래에 놓인다. [WASM HWPX 비교](wasm-hwpx-p105.png) |

나란히 놓인 이미지의 왼쪽은 rhwp, 오른쪽은 한컴 Print다. 글꼴의 획 모양에
차이가 있으며, 변경 영역의 줄 순서·표 경계·그림 포함·제목 앞 간격을 직접
확인했다. 수정 전 `c198dc3ef1` WASM에서 HWP가 115쪽 화면에서 116쪽
저장본으로 바뀌었고, 변경 전 생산 코드 `0adeca439f1481e7245a344b7713d7d563df2ace`의
Native에서도 렌더 전에 저장하면 116쪽이었다.

| 형식 | rhwp 저장 파일 SHA-256 | 독립 PDF SHA-256 |
| --- | --- | --- |
| HWP | `667ed25e8943c471429eda16e6b5d57892a4e6e0a106e1a3e7846cb5f629f335` | `fd94af210f172b48d9b1690f067ccad5f7eb17002898f47f9ce3ef2091c09ae8` |
| HWPX | `9ea27e3c9cc78efc0eba5e79dd19adc5a4089bd659831def07b4ebe1c0af1eed` | `84953359b3b39d9bc5a2258ecff94b216e380d11264925ec9c951954429e4878` |

## 출력 경로와 보호 검사

fresh WASM을 연결한 Studio에서 실제 키 입력 후 파일 메뉴로 저장했다. HWP와
HWPX 각각 입력 직후 지연 페이지네이션이 남았고, 저장이 이를 한 번 완료했다.
재열기와 재저장 모두 115쪽이었고, 변경 영역의 숫자·그림·이어받기 소속이 같았다.
같은 HWP 입력의 인쇄 미리보기도 지연 페이지네이션을 한 번 완료하고 115쪽을 준비했다.

편람 HWPX Native 383쪽과 공통 보호 문서 4종의 Native·fresh WASM 전쪽에서
각 기준의 하락이 없었다. [편람 전쪽 TSV](handbook-native.tsv),
[공통 보호 결과·전쪽 TSV·대표 이미지 목록](contributor-gate/results.json)

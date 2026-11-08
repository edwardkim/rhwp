# Native OLE 포함 그림과 캡션의 독립 Print 대조

제품 커밋 `af16ab490b231d371ed5819021c631561087aa52`의 Native083·최적화 fresh034를
같은 독립 한컴 Print와 대조했다. 입력·PDF·TSV·PNG의 해시, 실제 렌더 명령과 Print
실행 정보는 [provenance.json](provenance.json)에 있다. 원본은 이미 추적하는
[HWP](../../../../samples/한셀OLE.hwp)·[HWPX](../../../../samples/한셀OLE.hwpx)를 재사용한다.

## 편집과 독립 기준

원본 두 형식에 공개 `set_shape_properties_native` API로 오른쪽·아래 맞춤 캡션을
추가했다. 모델의 번호1·접미사 `.`·너비8504HU·간격850HU·여백 포함은 원 검사와
같다. 각각 HWP/HWPX로 저장한 네 입력을 이 폴더에 보존했다. 같은 저장본을 실제 저장
제품에 맞는 engine2020/2024에서 PrintMethod0·one-up으로 출력했다. 편집하지 않은
원본 PDF를 편집 결과의 기준으로 쓰지 않았다. 내부 MCP 주소와 인증 정보는 포함하지 않는다.

| 저장 입력 | 독립 Print | Native TSV | fresh TSV |
| --- | --- | --- | --- |
| [HWP 원본→HWP](inputs/ole-caption-hwp-source-as-hwp.hwp) | [2020](pdf/ole-caption-hwp-source-as-hwp-hwp-2020.pdf) | [원문](native/ole-caption-hwp-source-as-hwp-hwp.tsv) | [원문](wasm/ole-caption-hwp-source-as-hwp-hwp.tsv) |
| [HWP 원본→HWPX](inputs/ole-caption-hwp-source-as-hwpx.hwpx) | [2020](pdf/ole-caption-hwp-source-as-hwpx-hwpx-2020.pdf) | [원문](native/ole-caption-hwp-source-as-hwpx-hwpx.tsv) | [원문](wasm/ole-caption-hwp-source-as-hwpx-hwpx.tsv) |
| [HWPX 원본→HWP](inputs/ole-caption-hwpx-source-as-hwp.hwp) | [2020](pdf/ole-caption-hwpx-source-as-hwp-hwp-2020.pdf) | [원문](native/ole-caption-hwpx-source-as-hwp-hwp.tsv) | [원문](wasm/ole-caption-hwpx-source-as-hwp-hwp.tsv) |
| [HWPX 원본→HWPX](inputs/ole-caption-hwpx-source-as-hwpx.hwpx) | [2024](pdf/ole-caption-hwpx-source-as-hwpx-hwpx-2024.pdf) | [원문](native/ole-caption-hwpx-source-as-hwpx-hwpx.tsv) | [원문](wasm/ole-caption-hwpx-source-as-hwpx-hwpx.tsv) |

네 입력 모두 양쪽 전체1쪽=PDF1쪽이며 raw 일치율은100%다.

## 원인·같은 영역 직접 판독

WMF의 원본 DIB와 공통 WMF→SVG 변환 결과에는 노란 바탕과 3행·5열의 숫자가 있었다.
Native의 SVG 조각 래퍼만 `xmlns:xlink`를 선언하지 않아 XML 파싱에서 실패했다.
실제 조각의 usvg 실행은 선언 전 `unknown namespace prefix 'xlink'`, 선언 뒤 정상
렌더링이었다. 같은 공통 Native 래퍼의 선언만 보완했으며 좌표·배율·정밀도·마스크·
외부 파일 참조 차단은 바꾸지 않았다. fresh의 기존 래퍼에는 선언이 있었다.

| 실제 출력 | review | standalone overlay |
| --- | --- | --- |
| Native083 | [review](native_review_001.png) | [overlay](native_overlay_001.png) |
| fresh034 | [review](wasm_review_001.png) | [overlay](wasm_overlay_001.png) |

동일 backend의 네 입력은 실제 PNG SHA가 같아 대표 한 장씩을 연결한다. 같은 배율에서
노란3행·5열의 숫자1~5·캡션 `그림1.`·외곽·주변 빈 공간을 직접 확인했다. Native의
이전 회색 상자도 raw100%였으므로 그 점수만으로 내용 보존을 인정하지 않았다. 현재
대표 Native의 엄격 내용 픽셀 일치율은5.96→93.87%, fresh034는95.02%다. 이 값은
raw100%와 별개다. 작은 숫자·선의 래스터 피복률과 글리프 가장자리 차이는 남는다.

## 정식 회귀와 기대값

새 [정식 회귀2함수](../../../../tests/cases/native_embedded_ole_preview_content.rs)는
원본별 live 캡션·HWP 저장·HWPX 저장의 실제 Print PNG를 검사한다. OLE 소유 틀에서
노란 바탕과15개 셀의 숫자 잉크가 남는지 확인한다.85% 색 면적 조건은 회색 대체 출력과
내용 누락을 검출하는 보조 계약이며90% 시각 gate를 대체하거나 완화하지 않는다.
숫자 글리프의 완전한 동일성은 이 존재 검사만으로 입증하지 않으며 위 직접 판독을 함께 적용한다.

네 독립 PDF의 실제 image XObject 좌표는 모두 같았다.72pt→96DPI로 환산한
[independent_coordinates.json](independent_coordinates.json)을 기대값으로 쓰며,
실제 모델/엔진 좌표를 복사하지 않았다. 해당 페이지의 Native/fresh raw100%와 직접
판독을 확인한 뒤에만 OLE 틀을 ±2px로 보호한다. 셀 영역 선택은 OLE 틀의 상대 행·열
관계이며 절대 페이지 px 대역이나 전체 SVG 해시를 고정하지 않는다.

최종 같은 소스의 strict 실행은 Native082에서2FAIL, 보정083에서2PASS다. 두 build는
모두 성공했으며 이전 실패는 노란 바탕 누락이다. OLE 기존 검사의 표시 문자열 수집도
모델 `text` 대신 공개 `display_or_text()` 계약을 소비하도록 공통화했다. 같은 Native083에서
소비값 보정 전2FAIL·보정 뒤 전체9PASS이며 원래 번호·속성·배치·제거 기대값은 유지했다.

새 source의 자동 수집·고정 devel manifest·unit-tier 검사는 모두 통과했다. 정식
`regression_suite_023`의 native-skia nextest도2PASS·0FAIL이다. 최신 lint는 이어서 실행한다. 이 폴더의 완료를 전체45개 PR의
최종 검증·CI·원격 승인으로 표현하지 않는다. 실행 로그·중간 JSON은 ignored output에 둔다.

# 인라인 표 문단의 언어별 글꼴 슬롯 대조군

`curly_quote_tac_native.hwpx`는 공개 원본
[`36494836_field_marker_tac_lines.hwpx`](../issue7344/36494836_field_marker_tac_lines.hwpx)의
진단 derivative를 Windows 한컴에서 실제 편집·저장한 대조군이다. 원본 전체의 동일성이나 개선을
이 대조군의 결과로 주장하지 않는다.

생성은 표 host의 텍스트를 `가‘나’`로 바꾸고 Latin face를 지정한 선행 진단에서 시작했다.
수동으로 남긴 LineSeg를 정상 저장본 증거로 사용하지 않고, 한컴에서 열어 실제 본문 선택을 확인한 뒤
Latin face를 Arial→Times New Roman으로 두 번 변경하고 readback 후 HWPX로 저장했다.
저장 제품은 한컴2020이며 대응 독립 출력은
[`curly_quote_tac_native-2020.pdf`](../../pdf/issue7622/curly_quote_tac_native-2020.pdf)다.
Print service의 engine2020·한컴11.0.0.9136·print method0·driver one-up 실행을 확인했다.

| 자료 | SHA-256 |
| --- | --- |
| HWPX | `2bcfeef3fcd53badf4d111e55ac435e74e60b8fcc727718f2c8476a4dedac489` |
| Print PDF | `72935a7cb8cda4dfa5b8fc923a8ba486b42165181b1c4b2d535cc77cefc089d6` |

실제 저장 글자모양의 Latin face와 독립 Print의 두 굽은 인용부호는 Times New Roman이다.
수정 전 product `711f7f92e`의 Native/fresh WASM은 굴림체로 paint했다. 보완 product `5538efd38`에서
두 출력 모두 Times New Roman을 소비하고, 같은 원문·독립 PDF의 전1쪽 Visual Sweep 최저는
Native/fresh WASM 각각99.01152%다. 비교·overlay·review를 직접 열어 표 외곽·뒤 본문·인용부호를
확인했다. 일부 획 농도·AA·괘선 두께 차이는 남는다.

실행·생성·원시 출력·TSV·수정 전후 증거와 미검증 범위는
[#7622 검토 기록](../../mydocs/pr/archives/pr_7622_review.md)에 연결한다.
회귀는 같은 char-shape ID 안의 실제 Latin face와 한글 대조군, 내용 순서·누락·중복을 검사한다.
절대 픽셀 좌표·전체 SVG 해시를 기대값으로 고정하지 않는다.

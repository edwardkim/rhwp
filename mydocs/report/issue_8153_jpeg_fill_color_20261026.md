# #8153: 전체 JPEG 채움의 RGB 전달

[계획](../plans/issue_8153_jpeg_fill_color_20261026.md)의 독립 자원과 공통 호출 경로를 따른다.
기존 8adc3a4010의 두 문서 전쪽 Native/fresh WASM 1532조건은 모두 90% 이상이고
383쪽씩 대응한다. 전쪽 raw100은 Native 683/766·WASM 681/766으로 아직 목표 미달이다.

## 원본과 수정 전 실제 실패

정식 원본 검사 `tests/cases/issue_8153_print_jpeg_fill_color.rs`의 SHA-256은
`53a4c81c391773d0ec83ffcb76436d828638cbf3be4944bcf6f80baafcdc0310`다.
고정 8adc3a4010 라이브러리에서 빌드 exit 0·실행 exit 101이며 두 형식 모두
독립 JPEG를 보내야 하는 채움이 PNG라는 지점에서 FAIL했다. 글꼴이나 컴파일 실패가 아니다.
독립 JPEG 2개는 기존 PDF의 바이트를 그대로 `tests/fixtures/issue8153/`에 보존했다.
해당 README에 PDF·자원 SHA와 양쪽 독립 출력의 같은 RGB 근거를 기록했다.

기존 decoder/encoder 소스를 소비한 진단은 source RGB와 최종 JPEG의 복호화 RGB가
각각 독립 자료와 전픽셀 동일하다. 실제 Native 261쪽의 원본 좌표·텍스트·소유·clip을
유지하고 원본에서 만든 자원만 교체한 진단에서는 미대응 **322→0픽셀**, raw **100%**다.
진단의 수정 전 PNG가 정식 전쪽 Native PNG와 RGB 전픽셀 같음도 확인했다.
독립 PDF 자원을 제품에 주입하거나 수치로 좌표를 움직인 결과가 아니다.
같은 영역의 독립 Print/직전/진단 PNG를 직접 열어 그림 내용·표·선·색을 대조했다.
이 결과는 제품 구현 뒤의 실제 재검증을 대신하지 않는다.

## 공통 구현 후보

이미 있는 전체 원본 채움 판정과 최종 paint 폭을 공통 Print 자원 진입점에서 소비한다.
JPEG FitToSize/Total은 기존 IJG RGB decoder·공통 품질/인코더로 배치별 JPEG를 발행한다.
Picture의 RGB555 계약과 다른 형식·효과·부분 창·반복 채움은 유지한다.
캐시는 같은 JPEG를 Picture와 채움으로 쓸 때도 두 색 계약이 섞이지 않게 구분한다.
decoder/encoder의 수학·폭 계산·기존 품질 경계를 새로 맞추지 않았다.
출력 payload는 OutputDevice이며 모든 backend가 같은 결과와 기존 공통 표본을 소비한다.
원본 BinData와 편집/저장 정보·배치·문단 소유는 바꾸지 않는다.

같은 정식 원문을 새 Native에서 재실행하고 관련 정상 대조·영향 페이지·전쪽 및 표준
최적화 fresh WASM을 확인한다. 아직 제품 변경의 수정 후 검증은 완료 전이며 전체 테스트와
PR 준비 완료로 기록하지 않는다. 작업 증적은 ignored `handbook-jpeg-fill-color-candidate/`와
`remaining-raw100-diagnostic/remaining-image-resource-audit/current8adc/`에 보존한다.
upstream 동기화와 원격 쓰기는 하지 않고 전쪽 100%·필수 테스트 후 PR 준비까지만 한다.

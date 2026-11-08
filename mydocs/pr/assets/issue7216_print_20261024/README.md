# #7216 원본의 독립 Print 대조

원본 `samples/issue7216/tall_table_before.hwpx`를 수정하지 않고 한컴 2020 MCP의
Print method 0으로 출력했다. 저장 제품은 한컴 2020, 11.0.0.3524다.
실제 실행 제품·Print 경로·입력/PDF SHA는 `pdf/issue7216/print-provenance-20261024.json`에 있다.
스크립트 등 입력 전처리는 없었다. 원본은 기존 samples 경로에 그대로 보존한다.

고정 devel과 후보의 Native/fresh WASM을 같은 PDF 전2쪽, compat2022, 96 DPI에서
비교했다. base 최저 raw는 Native 1.11835% / fresh WASM 1.08641%이고 후보는
양쪽 모든 쪽 raw100%다. 후보는 PDF와 쪽수도 같다. base의 exit1과 미달 값을
숨기지 않고 원문 TSV를 함께 보존한다.

현재 후보의 corpus 검사에서는 overflow-cell 21줄·off-canvas 1건 등의 증가를
검출했다. 이 증거만으로 baseline이나 assertion을 갱신하지 않는다. 실제 paint의
clip, 원본의 내용 소속·누락·중복과 독립 Print의 출력을 함께 판독해 진단 의미를
확인한다. 픽셀 일치만으로 해당 검사를 PASS로 바꾸지 않는다.

렌더 source는 4adb7faf48이다. 이후 보충 측정 병합 보정 5386de6e6a는 아직 이
자료의 실행 범위에 포함하지 않는다. 실행 로그와 중간 receipt는 output에 보존한다.

## 원본 쪽 나눔과 실제 잘림

원본 HWPX의 표는 `pageBreak="NONE"`, 61행·3열이다. 독립 Print 첫 쪽에는
번호1~53과 대응 항목·설명이 있고, 두 번째 쪽에는 `표 다음 문단`만 있다.
54~60행은 Print에서도 용지 아래로 잘린다. 후보 render tree는 이 행들을
원래 표 소유로 보존하며 첫 쪽 물리 하단 아래에 둔다. 따라서 21줄의
셀 넘침과 표1개의 off-canvas/body-overflow는 원본의 나누지 않음 계약과
함께 해석해야 한다. 이 문서는 새 scaffold의 정상 쪽 나눔 기본값을
검증하는 after 입력과 다른 before 입력이다. 기존 #7216 검사 주석도
독립 Print가 `설명 53`에서 끊기는 사실을 기록하고 있다.

이 관측은 진단 신호를 숨기거나 삭제할 근거가 아니다. 신호를 유지하면서
원본 계약과 의도된 출력 변화의 근거로 판정한다. baseline 갱신 여부와
최종 source의 재검증은 아직 완료하지 않았다.

## 최종 음성 대조 계약과 원장 보정

Native208와 최적화 fresh055에서 같은 원본·Print 전 2쪽을 다시 출력했다.
양쪽 모든 쪽의 raw는 100%이고 쪽수도 일치한다. 현재 TSV는 앞서 보존한
후보 원문과 바이트가 같아 같은 파일을 재사용한다. 새 source·producer와
review/overlay의 해시는 evidence.json의 current_verification에 구분했다.

첫 쪽에서 53행까지 보이고 둘째 쪽에는 후속 문단만 있다는 실제 Print와
표의 NONE 속성·전체 61행 소유를 함께 확인했다. 원본 모델·render tree에서
잘린 54~60행을 삭제하거나 다음 쪽에 중복하지 않는다. 새 관계형 회귀
issue_7216_print_clipped_unsplit_table은 이 소유·내용·잘림·후속 문단을 검사하고,
21개 셀 줄·표 1개의 off-canvas·body-overflow가 계속 진단되는지도 보호한다.

이 독립 근거에 따라 세 원장 중 이 before 입력의 값만 각각 21·1·1로 기록했다.
진단 검출 조건·전역 허용치·다른 입력의 기준값은 바꾸지 않았다. 새 음성 대조와
기존 scaffold의 정상 쪽 나눔·왕복·짧은 표 검사 총 4건이 모두 통과했다.
전체 corpus 및 전체 Rust의 최종 통과는 후속 필수 검증으로 남아 있다.

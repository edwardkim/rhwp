# #7642 Stage 77 — CASES의 실제 점유 끝과 후속 본문 전진

Issue: #7642

## 독립 근거

Stage 76에서 종단 TAC 단어와 공백 줄 소유를 보정했으며 그 Native
집중 검증은 진행 중이다. 전체 55개 원본·853쪽과 관련 회귀 범위는 유지한다.
수식 geometry 무조건 수용은 정상 대조에서 실패해 이미 제거했다.

원본과 같은 산식의 geometry 유지/비움·plain space 유지/정리 쌍
4개 뒤에 동일한 `[NEXT] 다음 본문`을 붙였다. 원본을 대체하지 않는
진단 입력이다. 한컴 2024 저장 job ce2f2da8-2e7b-4f50-badc-e551a0b92016,
Print job 832e3c71-65fc-4e73-84fe-593bd0d59d78이 실제 성공했다.
한컴 13.0.0.3901, PrintMethod 0, frame_print_to_pdf_ex_one_up이며
PDF SHA-256은 a3fa7bb192f5f9e0aea77c3cefec1369ec653b81b3e6b8deb7df50f2f72964fa다.

기존 저장 폭 8438HU와 새 저장 폭 11138HU가 달라도 Print의 후속
본문 위치는 같은 산식 앞 label 폭을 제외하면 같다. 따라서 양수
저장 폭의 보존도 Print 흐름 계약의 근거로 사용할 수 없다.
기존 Native 대조는 전체 1쪽 92.77681%지만 뒤 본문의 시작이 Print보다
약 4px 밀린다. 90% 통과로 이 미세 간격 차이를 면제하지 않는다.

## 원인과 공통 소비

layout_cases의 Row 자식은 x=0.3em에서 시작한다. 그러나 그 Row 폭을
max(child.width)+0.6em으로 발행해 실제 마지막 자식 끝보다 0.3em 크다.
Paren은 body.x=0.3em을 별도로 소비하므로 결과 폭은 실제 자식의 끝에
다시 0.3em이 붙는다. 9pt 대조에서는 3.6px 추가 예약이며, 이 폭은
flow_metrics_hwp → inline_box/composer → 후속 본문 원점과 줄 나눔에
전달된다. 실제 글리프 원점과 CASES 행 간격은 이 추가 폭을 사용하지 않는다.

Row 폭을 같은 자식 배치의 max(child.x+child.width)로 계산한다.
직접 점유 끝을 공유하며 추가 clamp·문서 ID·임의 폭 보정·저장 상수화를
사용하지 않는다. 명시적인 산식 공백은 자식 폭에 남기고 중괄호 예약은
기존 Paren 원점에서 한 번 소비한다. 뒤 delimiter와 첨자처럼 이 폭을
소비하는 경로도 같은 결과를 사용한다.

## 검증

paired CASES와 후속 본문 대조, 기존 CASES 32개 전체 두 쪽, 분수·한글
36/42개, 기존 시험지와 정상 그림 경계, 종단 TAC 4개와 실제 배포 원본을
같은 Native/fresh WASM source에서 대조한다. 원본 10월 HWPX 9~10쪽의
첫 소유 차이가 개선됐는지도 별도로 확인한다. 입력/출력 SHA와 명령은
ignored output에 기록한다. 전체 원본 TSV와 직접 판독 후 관계형 회귀를
변경하며, 아직 fixture·golden·기존 기대값을 변경하지 않는다.

## Native 후속 본문 대조 결과

source `2ee336fde3f372d5f9dc9f7cb5425feb2fbea3a4`의 Native 빌드,
fmt check와 Native Clippy는 exit 0이다. 후속 본문을 붙인 4개 산식의
독립 Print 1쪽 대조는 92.77681%에서 98.72170%로 개선됐다.
저장 geometry 유지/재생성 쌍은 수식 뒤 같은 본문 시작을 사용하며,
글리프 원점·행 높이를 움직이지 않고 실제 점유 끝의 폭만 공유했다.
확대 before / after / Print도 같은 원문 영역에서 새로 산출해 판독했다.

이 source의 paired 대조는 98.16677%다. 종단 수식의 별도 줄 소유를
함께 보정하는 Stage 78로 이어가면서 나머지 반복 집중 검증은 의도적으로
종료했다. 완료 기록과 이미지는 보존했으며, 해당 Native 부분 결과를
최신 head 전체 또는 fresh WASM 통과로 간주하지 않는다.

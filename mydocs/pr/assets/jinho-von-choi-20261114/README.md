# JinHo-von-Choi PR 3건 검토 원장

**로컬 판정: 메인테이너 보완 후 수용 가능.** #9070·#9071·#9072의 승인된 보완과 필수 검증을 완료했다.
사용자가 거절한 기존 차이 예외를 적용하지 않았으며 원격 단계의 별도 승인을 기다린다.
원 기여 head 3개와 fork branch를 보존한다.

최신 base `2d83fa4c8a`·제품 source `8ad1c5c08a`의 [최종 검증](final-source-8ad/README.md)에
전체 회귀 13,608건·Native 공식 4,073건·Native/fresh WASM 보호 각 1,771쪽과 전쪽 TSV를 기록했다.
공식값 하락은 0행이며 같은 최종 실행의 상승 19행을 [새 하한](../visual-protected-baseline/README.md)으로 반영했다.
원 사례·대조군은 양쪽 각 134쪽 모두 90% 이상이고, 최신 devel 추가 기능 403쪽씩도 보존했다.

## 승인된 보완과 실제 근거

- #9070: 일반 기호와 괘선 77개를 독립 한컴 저장값으로 복원하고 SFNT 측정·paint를 함께 연결했다.
- #9071: 종류·색 없음·속성·각주 길이의 저장 계약, Both 바탕쪽의 공통 물리 프레임,
  저장 TAC 줄 점유와 후행 간격의 공통 소비를 보완했다.
- #9072: occurrence 도움말, 다구역 전역 필드 주소와 본문 삽입, TextLine 밖 실제 텍스트 수집을 보존했다.
- 실제 대체 face의 공급 우선순위와 그림/소유 테두리의 공통 paint 순서를 바로잡았다.
  2022·2024 Print의 그림 pen이 공통 600dpi 장치를 소비하게 했다.
- 실제 정적 TTF 형식은 원 SFNT를 메트릭으로 축약하기 전에 보존한다.
  논리 원문 유형·전진은 유지하고 기존 공통 Print text 프레임을 실제 paint가 소비한다.
  CFF·가변·자연 잉크·PDF 소유 paint·Screen/HQ 대조를 실제 두 세대의 48상태에서 확인했다.

[전체 원인·보완 이력](protected-environment-review.md), [정적 TTF의 확대 판독·수정 전 FAIL/후 PASS](supplied-ttf-frame/README.md),
[최종 직접 판독](final-source-8ad/review/README.md), [입력 118개 보존](final-source-8ad/input-inventory.json).
과거 폴더의 수치는 각 영수증에 적힌 producer의 체크포인트다. 현재 완료 판정은 `final-source-8ad/`를 사용한다.

## 후속 범위

upstream 수용 PR의 해당 head CI·self-review와 merge는 별도 원격 승인 뒤 수행한다.
연결 이슈 9건의 완료 범위는 개별 review와 최종 제출안에 구분한다. 부모 #7445·#7620은 OPEN 유지다.

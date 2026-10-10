# 저장 앵커 실패의 보정과 관계 검사

- 브랜치: `fix/7445-completed-20261009`
- 고정 devel: `21ed64071f81f35f37618717693bd30f727392c9`
- 증적: [validation.json](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/validation.json)
- 전체 회귀의 27개 실패가 모두 해결된 상태는 아니다. 전체 재실행과 사용자 승인 뒤에만 PR을 진행한다.

## 구현 근거와 소비 경로

| 원본 | 실제 결함과 독립 근거 | 공유한 결과와 적용 경로 |
| --- | --- | --- |
| `samples/hwpx/hancom-hwp/issue_157.hwp` | Print 2쪽에 남는 마지막 날짜·주의 문구가 후보에서는 3쪽으로 밀렸다. 앞 float의 호스트 줄을 누적 흐름 원점에 재포함했다. | `stored_float_before_whitespace_line_frame`은 앞 본문 끝·표 높이·양쪽 여백·표 뒤 공백 줄·후속 줄의 원본 종료식을 확인한다. `whole_fit::query_original_control_table_frame`이 반환 프레임을 수용과 배치에 전달한다. |
| `samples/issue3837/stored_vpos_rewind_form.hwp` | Print 3쪽 마지막 표는 앞 빈 줄 상자·간격과 표 위여백 뒤에서 시작한다. 후보는 뒤 표의 원본 TAC 원점을 버렸다. | `TypesetState`는 쪽 첫 TAC의 실제 수용 배치와 원본 0 원점을 함께 확인한다. `stored_tac`는 확인된 단에서 후속 원본 객체 원점을 보존한다. |
| `samples/issue6123/3112461_railway_emc_criteria.hwpx` | 첫 보정은 이전 쪽의 호스트 뒤에 있고 후속 줄이 되감기는 닫힌 상자를 같은 쪽에 수용하여 글자 91쌍이 겹쳤다. 고정 devel·보정 전에는 0건이다. | `empty_float`의 같은 쪽 판정은 앞 문단뿐 아니라 후속 원본 줄이 같은 축에서 증가하는지도 확인한다. 새 쪽의 닫힌 상자를 현재 쪽 위에 놓지 않는다. V4 Native에서 14쪽·겹침/overflow 0건으로 복구했다. |

재계산된 누적 LineSeg 스냅샷과 원본 쪽 상대 줄을 구분하며, lane 이동은 후속 줄 원점에도 동일하게 적용한다. 문서 ID·절대 좌표에 맞춘 분기는 추가하지 않았다. 철도 문서는 한컴 2020 독립 인쇄 PDF를 새로 확보해 함께 보존했다.

## 시각 검증과 기존 검사

V3 Native/fresh WASM에서 주주총회 2쪽 최저 99.02249%, 응시원서 4쪽 최저 94.11853%다.
규제분석 17쪽·KTX 1쪽·low-slack 1쪽·TAC 3쪽·민원 서식 2쪽의 대조군도 각각 최저
94.31120%, 95.86468%, 95.86412%, 92.54967%, 98.64096%다. 전체 30쪽에서 미달 0이며,
두 backend PNG가 바이트 동일하다. V4 Native 30쪽 SVG도 V3와 모두 동일하다.
V4 fresh WASM 및 기존 검사 재실행은 이 기록 작성 시 진행 중이며 최종 검증으로 대체하지 않는다.

- 기존 검사 23개는 V3에서 모두 통과했다. 기존 함수 안에 보완한 관계 검사 3개도 통과했다.
- 쪽수 원장은 주주총회 소집공고·참석장·마지막 날짜·주의 문구의 쪽 소속과 정확히 한 번의 보존을 함께 검사한다.
- 응시원서는 마지막 표와 앞 빈 줄의 원본 줄 상자·간격·여백 관계 및 다음 쪽 표의 소속을 검사한다. 원본 계약을 px로 환산했지만 절대 y를 고정하지 않았다.
- 응시원서의 겹침 기준 3→4는 독립 Print에도 있는 사진 안내 `2매`와 `㊞` 도장의 겹침이다. 두 사진 칸의 원문과 각 칸 안 도장 하나를 검사하며, 다른 문서의 겹침을 면제하지 않는다.
- V3 겹침 partition 7은 철도 문서의 새 회귀 때문에 실패했다. 이 실패를 보존하고 V4 구현을 고쳤다. 전체 partition의 재실행 없이 통과로 보고하지 않는다.

대표 비교: [주주총회 2쪽](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/shareholder-review_002.png),
[응시원서 3쪽](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/application-review_003.png),
[사진 도장과 원본 Print](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/photo-stamp-base-current-print.png).

## 남은 판정

- 가스 표는 원본과 PDF를 보존하고 [#7803](https://github.com/edwardkim/rhwp/issues/7803)을 #7445 하위로 등록했다. 고정 devel과 후보 모두 32쪽이고 Print는 31쪽이다. 측정 불가를 90% 미만 점수로 취급하지 않았으며 검사도 아직 제거하지 않았다.
- HWP 컨트롤 API 105쪽의 고정 devel 전쪽 Native에는 23쪽 미달이 있다. 기존 #7731과 연결하되, 후보 16쪽의 추가 데이터 행은 Print와 devel에 없는 실제 회귀라 구현 보정 대상으로 남긴다.
- PrEP는 105쪽 연구비 표에서 첫 소속 차이가 발생해 140→141쪽이 된다. 원본 저장 프레임과 실제 whole-fit·행 컷·배치의 소비 차이를 조사 중이다.
- 위첨자 3쪽은 변환 서비스 저장 오류가 반복돼 독립 PDF를 기다리고 있다. 이 기준 없이 기대값을 갱신하지 않는다.

로그·TSV·중간 JSON은 `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/`에만 보존한다.

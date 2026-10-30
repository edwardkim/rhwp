# PR #8233 리뷰 — 그룹 자식 도형 CTRL_DATA 저장 보존

## 최종 판정

**머지 보류 — 검증 진행 중.** 실제 편집·저장·재열기의 자식 도형별 데이터 보존과 필수 회귀를 확인한 뒤 판정한다.

## 접수 정보

- 작성자: `chrisryugj`, CONTRIBUTOR, Open, non-Draft. Reviewer: `edwardkim`.
- 관련 이슈: #8225, 참조 범위. 원 head: `d9c8a71b642fad1ee8221f4036817a23dfe15f10`.
- 비교 기준: `upstream/devel` `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`.
- 검토 브랜치: `review/integration-20261031-chrisryugj-storage`.
- #8224 다음 충돌 없이 통합한 후보: `86f4879934d1fc903b8497d011fdd7db9dede38f`.
- 이번 기여자의 원격 push·새 PR·댓글·머지는 아직 승인받지 않았다.

## 변경과 검토 범위

`SHAPE_COMPONENT` 자식의 `CTRL_DATA`를 `ShapeComponentAttr.ctrl_data`로 읽고, 저장할 때 해당 도형의 하위 레벨에 쓴다. 최상위 컨트롤 수집은 자식 데이터까지 잘못 가져오지 않도록 깊이를 제한한다. 저장 위치와 payload 순서·바이트를 대조하며 전체 레코드 수만으로 소유 관계를 판정하지 않는다.

이 필드는 serde에서 제외되는 HWP 원본 보존 데이터이며 레이아웃·paint의 소비 지점은 발견되지 않았다. 조판 자체를 수정하는 PR과 구분하고 실제 편집 저장을 통해 도형과 본문 보존을 확인한다. 동일 통합 후보의 #8224는 별도의 시각 검증 대상이다.

## 검증 입력과 진행 기록

- `samples/basic/sungeo.hwp`: 그룹 자식 CTRL_DATA payload 24개 보존 대상.
- `samples/issue6542/156678235_mid_para_vpos_rewind.hwp`: 자식 데이터가 최상위에 중복 저장되지 않아야 하는 대조군.
- `samples/hwp-multi-001.hwp`: 여러 구역 보존 대조군.
- 원 PR의 3개 저장 계약 검사를 고정 devel과 누적 후보에서 순차 실행한다.
- base WASM에서 실제 글자 삽입·삭제 → HWP 저장 → 재열기를 실행했다. 텍스트는 보존되며 쪽수는 각각 94·7·9쪽이었다. 이 결과는 CTRL_DATA 보존 판정과 구분한다.
- 기록과 진단 산출물: `output/pr-review/chrisryugj-20261031/`.

원 PR은 `sungeo.hwp`의 별도 최상위 CTRL_DATA 중복 표현까지 모두 복구한다고 주장하지 않는다. 남은 데이터 차이는 자식 보존과 구분해 확인하고 #8225 전체 해결로 과장하지 않는다. 고정 devel에서 3개 검사 실패, 실제 재컴파일한 후보에서 3개 통과를 확인했다. base WASM의 편집 저장본에서도 자식 payload 수가 24→0, 1→0, 1→0으로 줄었다. 후보의 나머지 필수 검증은 진행 중이며 원 PR CI만으로 완료를 판정하지 않는다.

## 사양 근거

HWP 5.0 revision 1.3 §4.3.8·표 66은 CTRL_DATA를 컨트롤의 필드 이름·하이퍼링크 등을 담는 Parameter Set으로 정의한다. 실제 원본의 레코드 level과 부모 SHAPE_COMPONENT를 함께 추적한다. 이 불투명 payload를 실행하거나 조판 속성으로 재해석하지 않고 원 소유 도형 아래에 보존한다.

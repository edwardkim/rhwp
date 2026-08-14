# Stage 165: RowBreak 중간 쪽 declared-height trust 계약 분석

## 목적

RowBreak 표의 중간 fragment에서 선언 높이를 신뢰할 때 사용되는 `16/20/4/48px` 고정
tolerance를 폐기하고, stored object height·painted row height·현재 body bottom의 실제 차이로
판정할 수 있는 공통 계약을 찾는다.

## 분석 범위

- `MIDPAGE_ROWBREAK_DECLARED_TRUST_*`
- `MIDPAGE_ROWBREAK_NEAR_FIT_*`
- `whole_fit_table_total`, `declared_object_total`, `table_total`의 좌표계
- HWP/HWPX stored lineSeg, declared table height, RowBreak row fragment의 관계

## 원칙

- fixed pixel 또는 문서/profile별 초과량으로 declared height를 신뢰하지 않는다.
- declared geometry가 실제 paint fragment를 담는다는 source 근거가 있을 때만 whole-fit을
  허용한다.
- 저장 object bottom과 row/frame boundary가 서로 모순되면 일반 RowBreak cut을 우선한다.

## 완료 기준

- 각 초과량이 source/paint 좌표에서 무엇을 뜻하는지 분류한다.
- 공통 predicate로 대체할 수 있을 때만 코드와 결과 문서를 같은 Stage 커밋에 남긴다.
- 분석 문서만 커밋하지 않는다.

## 분석 결과

- 기존 `16/20/4/48px` 두 티어는 `whole_fit_table_total`의 body 초과량과
  `table_total - declared_object_total`의 차이를 관측값 구간으로 나눈 정책이었다.
  이 값들은 stored object, row geometry, paint frame 중 어느 좌표계에도 직접 대응하지
  않아 font metric drift와 실제 source fragment boundary를 구별할 수 없다.
- `saved_span`은 host의 저장 object bottom을 제공한다. declared height가 현재 flow body
  안에 들어가더라도, 중간 fragment에서는 이 source bottom도 같은 body 안에 있을 때만
  source가 whole object owner를 지지한다.
- 새 fragment 시작은 `current_height`의 근사값이 아니라 `current_items.is_empty()`로 판정할
  수 있다. 앞선 flow item이 없는 경우에는 기존 #2105의 fresh-fragment declared fit 의미를
  보존한다.

## 구현

- `MIDPAGE_ROWBREAK_DECLARED_TRUST_*`와 `MIDPAGE_ROWBREAK_NEAR_FIT_*` 네 상수를
  제거했다.
- RowBreak whole-fit의 범위는 다음 중 하나로 한정했다.
  - 현재 fragment에 선행 flow item이 없다.
  - 저장 host object bottom이 현재 body bottom 안에 있다.
- 기존의 painted-row footprint, strict fit, declared-excess 가드는 유지한다. 즉 source
  bottom만으로 실제 paint가 footer를 넘는 table을 whole-fit으로 승격하지 않는다.

## 결과

- 중간 쪽 declared-height trust가 fixed px bucket이 아니라 저장 object frame과 fragment
  구조로 결정된다.
- 전체 export 및 test는 이 Stage에서 실행하지 않았다. 다음 Stage에서 남은
  `declared_excess_within_drift`의 비율/상한 계약을 별도로 분석한다.

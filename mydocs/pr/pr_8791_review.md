# PR #8791 검토 — fix(parser/hwp3): 표 격자를 셀 왼쪽·위쪽 변으로 세운다

## 최종 판정

**머지 보류 — 실행으로 재현된 셀 내용 누락과 필수 시각 증거 부족.** 원 CI 성공은 확인했고 최신 base에서 원 문제·수정의 일반성과 실제 출력 영향을 확인한다. 구현 결함을 확인하기 전에는 회귀 검출로 표현하지 않는다.

## 접수 정보

- [원 PR #8791](https://github.com/edwardkim/rhwp/pull/8791), chrisryugj, CONTRIBUTOR, devel 대상 Ready.
- reviewer edwardkim을 지정·API 재확인한 뒤 상세 조사·fetch를 진행했다.
- 원 fork `chrisryugj/rhwp_fork`, branch `fix/hwp3-table-grid-left-edges`, 원 head `383f66acca8c966da9a198fd046c3da0ed94c3b6`. 원 head를 별도 ref로 보존했다.
- 기준 devel `b768bee1c4c0023087687f79a0a95f12108743e9`. 단독 merge-tree exit 0, 원 head check 36개에 실패·진행 항목이 없다. closing issue는 없다.
- [공통 검토 계획](chrisryugj_20261109_review_plan.md). 원격 push·게시·merge는 수행하지 않았다.

## 검증·판정 근거

[공통 계획](chrisryugj_20261109_review_plan.md)의 사양·원 해결 이력·실행 검증을 진행한다. 원문 parser contract, 실제 출력·시각 판정, 미검증 사항을 분리해 최종 결과를 기록한다.

## Merge 후 contributor PR comment 계획

최종 판정과 실제 source·CI·merge가 확정된 뒤 해당 원 PR의 입력·쪽에 맞는 review/overlay와 실제 수치·남은 범위를 기록한다. 현재는 댓글 게시 초안·원격 조치를 확정하지 않았다.

## 재현된 blocker

[실제 parser 반례와 제한 보정안](chrisryugj_20261109_review_impl.md)을 따른다. 2048 탐색 경계 이후 실제 오른쪽 이웃을 놓치고 뒤의 겹침 해소가 해당 셀의 문단을 버린다. 정상 2048행까지는 base와 같고 2049행부터 한 셀이 사라진다. 원 문서 네 표의 행열 개선은 확인했지만 이 새 누락을 원 head의 CI 성공으로 면제하지 않는다.

# PR #5189 검토 - HWP3 표 셀 겹침 해소

- PR: https://github.com/edwardkim/rhwp/pull/5189
- 작성자: `planet6897`
- base: `devel`
- 원 head: `03adcc66502a821e223b53dd3e3795e88ea6780d`
- 누적 검토 브랜치: `review/planet6897-hwp-contracts-20260818`
- 체리픽 커밋: `ab41e9c60`
- 선행 관계: PR 안의 격자 완성 커밋은 #5183과 논리적으로 동일하므로 중복 적용하지 않고,
  겹침 해소 커밋만 #5183 뒤에 적용했다.

## 결론

blocking finding 없음. 행 우선으로 셀을 정렬하고 점유 칸을 침범하는 span을 자르며,
완전 중복 셀을 제거한다. 이 정규화는 #5183의 빈 격자 보충보다 먼저 수행되어야 하며,
누적 적용 순서가 이를 보장한다.

## 검증

- 체리픽 충돌 없음
- focused: `issue_4367_hwp3_convert_fourth_contract` 10 passed
  - `hwp3_table_cells_do_not_overlap` 포함
- 누적 전체 Rust 회귀: 6,735 passed, 38 skipped, 3 slow
- 구조 확인: `git diff --check upstream/devel...HEAD` pass

## Fixture와 시각 증적

- 관련 fixture: `samples/hwp3-table-cell-overlap.hwp`
- 파서가 출력 가능한 정규 격자를 만들도록 보정하는 변경이다. renderer 비교나 PDF
  외관 주장을 하지 않았으므로 이번 검토에는 별도 PDF 증적이 없다.

## 리스크와 권고

겹침 해소는 손상된 표의 원래 span 정보를 축소할 수 있다. 그러나 격자 중복 제거라는
명시적 계약과 전체 회귀를 통과했으므로 누적 통합 PR 후보에 포함할 수 있다.

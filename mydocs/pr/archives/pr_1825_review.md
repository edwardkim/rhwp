# PR #1825 리뷰 — #1809 부분 수정과 조사 기록

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1825 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1809` |
| 작성 시점 참고 head | `204908bca947ea9bbaa041fb84731b40ed41a924` |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions 통과 |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- #1809 중 `admrul_1066`, `admrul_0296` 계열의 셀 측정 aim 정합 문제를 부분 수정한다.
- `hwpx_to_hwp` 변환과 `height_measurer` 의 표 측정 흐름을 보정한다.
- 문서에는 남은 케이스와 후속 분류가 함께 기록되어 있다.

## 로컬 검증

- 체리픽 커밋: `204908bca947` -> `52a496dab`
- 충돌: `mydocs/orders/20260702.md` 행 충돌 1건. #1807/#1808/#1809 행을 모두 보존해 해결.
- 누적 검증: focused Rust test, `svg_snapshot`, release-test integration, Clippy 통과.

## 판단

이 PR 은 #1809 전체 해결 PR 이 아니라 부분 수정과 조사 기록이다. 같은 누적 검토에서 #1837 이 `admrul_0556`,
`admrul_0072` 잔여 케이스를 추가로 처리하므로, #1825 는 #1837 과 순서대로 merge 되는 조건에서 merge 후보로
판단한다.

## 결론

merge 후보. 다만 #1809 close 는 #1837 merge 결과까지 함께 확인한 뒤 판단한다.

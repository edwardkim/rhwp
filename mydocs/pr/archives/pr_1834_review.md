# PR #1834 리뷰 — TextRun ±1 조성 노이즈 WARN_TEXTRUN 분리

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1834 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1773` |
| 작성 시점 참고 head | `4443f0505e66728af99d8f475433fb3cd7213c12` |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions 통과 |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- `src/diagnostics/render_geom_diff.rs` 에서 TextRun ±1 조성 노이즈를 `WARN_TEXTRUN` 등급으로 분리한다.
- render-diff 판정에서 실제 blocker 와 허용 가능한 조성 노이즈를 구분한다.
- #1773 의 근본 원 인코딩 보존 구현은 이 PR 의 범위가 아니다.

## 로컬 검증

- 체리픽 커밋: `4443f0505e66` -> `0c4ccaf0d`
- 충돌: 없음
- focused 검증: `env CARGO_INCREMENTAL=0 cargo test render_geom_diff::tests --lib` 통과.
- 누적 검증: release-test integration, Clippy 통과.

## 판단

diagnostics 등급 조정 PR 로서 테스트가 guard 한다. #1773 의 잔여 근본 과제는 #1840 문서 PR 로 이어지므로, 이
PR merge 만으로 #1773 을 닫지는 않는다.

## 결론

merge 후보. #1773 은 후속 기록과 함께 open 유지 여부를 확인한다.

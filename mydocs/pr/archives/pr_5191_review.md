# PR #5191 검토 - HWP3 곡선 점 IR 보존

- PR: https://github.com/edwardkim/rhwp/pull/5191
- 작성자: `planet6897`
- base: `devel`
- 원 head: `0e8f49c5636734278e7dbaff48dbc10ac017713d`
- 누적 검토 브랜치: `review/planet6897-hwp-contracts-20260818`
- 누적 통합 PR: https://github.com/edwardkim/rhwp/pull/5197
- 체리픽 커밋: `c6f9ab104`

## 결론

blocking finding 없음. HWP3 곡선의 원본 점 배열을 `CurveShape`에 전달하고, 세그먼트
타입을 점 수에 맞춰 생성한다. 기존의 빈 곡선 직렬화 경로를 피하도록 계약 테스트가
고정한다.

## 최종 처분

- 통합 PR #5197이 관리자 squash merge `6a62c399ca70178bff0c57fea36cf8e366d1e078`으로
  `devel`에 반영됐다.
- 이 원본 PR에는 통합 범위와 검증 근거를 댓글로 남긴 뒤 중복 원본 PR로 종료했다.

## 검증

- 체리픽 충돌 없음
- focused: `issue_4367_hwp3_convert_fourth_contract` 10 passed
  - `hwp3_curve_points_are_loaded` 포함
- 누적 전체 Rust 회귀: 6,735 passed, 38 skipped, 3 slow
- 구조 확인: `git diff --check upstream/devel...HEAD` pass

## Fixture와 시각 증적

- 관련 fixture: `samples/hwp3-curve.hwp`
- parser IR 보존 변경이며 PDF 외관 비교는 수행하지 않았다. 실제 한글 앱의 개방성은
필요 시 별도 Windows/MCP 검증에서 확인한다.

## 리스크와 권고

현재 계약은 곡선의 점 배열이 비어 있지 않음을 검증한다. 세부 곡선 기하의 완전한
한글 앱 동등성은 별도 시각·개방성 증적 범위다. 전체 회귀가 통과했으므로 누적 통합
PR 후보에 포함할 수 있다.

# PR #2155 검토 - Task #2150 21761835 잔여 팽창 분석 도구

- PR: https://github.com/edwardkim/rhwp/pull/2155
- 작성자: `planet6897`
- base: `devel`
- 원 head: `9c99b08c4de61f3eb49cd5e5b0edb75b183feef9`
- 체리픽 커밋: `f4519e25f`
- 포함 README: 없음

## 결론

문서/분석 도구 PR로, runtime renderer 동작을 직접 변경하지 않는다. #2146 이후에도 남는
21761835 전체 페이지 수 불일치의 원인을 분해하고, 한글 row height oracle 도구를 보강하는
내용이다.

## 변경 검토

- `mydocs/plans/task_m100_2150*.md`
  - fresh 줄높이 공식, 상쇄 coupling, 잔여 팽창 분석 기록
- `mydocs/report/task_m100_2150_report.md`
  - #2146 이후 남는 전체 6p/7p 차이를 후속 축으로 정리
- `tools/hangul_row_heights2.py`
- `tools/make_ls_ladder.py`
- `tools/probe_ls_ladder.py`

## 검증

- reviewer assign 완료
- 체리픽 충돌 없음
- Python compile:
  - `tools/hangul_row_heights2.py` pass
  - `tools/make_ls_ladder.py` pass
  - `tools/probe_ls_ladder.py` pass
- 누적 브랜치 검증:
  - `git diff --check upstream/devel...HEAD` pass
  - `cargo fmt --check` pass
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass

## 리스크

- 분석 도구는 compile만 확인했다. 실제 한글 COM oracle 실행은 이번 누적 PR 검토 범위 밖이다.
- #2146의 잔여 페이지 수 문제를 해결하지 않고, 해결 경로를 문서화하는 PR이다.

## 권고

누적 체리픽 PR에 포함 가능하다. #2149의 범위 제한을 보완하는 문서/도구 성격이다.

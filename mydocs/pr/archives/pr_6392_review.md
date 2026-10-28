---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6392
---

# PR #6392 review - 빈 table raw cache의 0 확장을 막는다

## 검토 판단

**수용 권고.** HWPX parse본의 빈 `raw_ctrl_data`를 0으로 확장해 common 합성 저장 경로를
끊던 결함을, 길이 가드와 common 원천 읽기로 제거한다. 사용자 추가 push 뒤 최신 source head를
다시 fetch해 동일 기능 patch를 통합했다.

## 라우팅과 근거

- 원 PR: https://github.com/edwardkim/rhwp/pull/6392
- 작성자 / reviewer: `lpaiu-cs` / `jangster77` review request 등록
- latest source head: `80e2bc972094169dfffaca554795d1039d1fd1cf`
- 최신 source CI는 `MERGEABLE/CLEAN`, required checks 통과 상태를 확인했다.
- `issue_6388_table_raw_zero_extension`: 6/6 통과.
- 통합 head의 full release-test, clippy, WASM library/package, Native Skia가 통과했다.

## 후속 코멘트

merge 후 원 PR에는 latest head 반영 여부와 빈 raw의 geometry/offset 보존 회귀 6건을 수용 근거로 남긴다.

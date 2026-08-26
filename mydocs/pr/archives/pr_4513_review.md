---
kind: pr-review
status: pending-review-only-fast-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4513 리뷰 - humdrum00001010 CI 성공 PR 11건 누적 통합

## 라우팅과 접수

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, visual_fixture_evidence.md,
  review_only_fast_pass.md, post_merge.md
```

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4513](https://github.com/edwardkim/rhwp/pull/4513) |
| 작성자 / 원 기여자 | @jangster77 / @humdrum00001010 |
| base / source | `devel` / `pr/devel-humdrum00001010-batch` |
| code·검증 기록 head | `4b7b817281bc494f8cd3f882067c4c0eb7cd6af5` |
| 규모 | 원 PR 11건, 메인터너 보정 1건, 원 PR별·누적 검토 기록 |
| 최초 CI | Full CI, Build & Test, CodeQL, Canvas visual diff, Native Skia 성공 |
| 검토 방식 | 작업지시자 승인 collaborator self-merge. 이 review·오늘할일 trailing commit은 code candidate를 바꾸지 않는다. |

## 변경 범위와 provenance

이 PR은 @humdrum00001010의 #4443, #4446, #4454, #4462, #4469, #4497,
#4500, #4501, #4502, #4503, #4504를 PR 번호순으로 누적한다. 각 원 commit은
`-x` 체리픽해 author와 provenance를 보존했고, contributor branch의 `devel` merge commit은 넣지 않았다.
Draft이면서 `CONFLICTING`인 #4315는 이 통합에서 제외했다.

원 PR별 수용 근거는 `mydocs/pr/archives/pr_<번호>_review.md`, 적용 순서·충돌·검증 전체는
`mydocs/pr/archives/pr_4443_4446_4454_4462_4469_4497_4500_4501_4502_4503_4504_review_impl.md`에 기록했다.

## 메인터너 보정

#4462가 공용 HWPX caption parser를 `parse_caption`으로 이름 변경한 뒤 #4503이 추가한
`shape_children` 경로가 과거 이름 `parse_table_caption`을 호출해 누적 컴파일이 실패했다. 두
원 PR을 독립적으로 적용할 때는 나타나지 않는 교차 결함이다.

`2d2f42524`는 이 호출 한 곳만 현재 공용 함수 이름으로 고쳤다. caption 데이터 모델, vertical
alignment 의미, parse error 처리, HWP5/HWPX 저장 형식은 바꾸지 않는다. #4500과 #4503의
`control/tests.rs` 충돌은 독립 테스트 모듈을 양쪽 모두 보존했다.

## 완료한 검증

- `cargo fmt --check` 통과.
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
  5,645 passed, 정책 skip 35.
- caption vertical alignment·field parameters focused test 6건 통과.
- 최신 Node WASM 기준 Studio 저장 관련 test 23건 통과.
- web WASM, `npm run build` 통과.
- `npm run e2e:issue-4430-content-loss`: 345/345 통과. HWP/HWPX 명시 저장, 암호 저장, picker fallback,
  content-loss artifact 수명주기를 실제 브라우저에서 검증했다.
- code candidate `4b7b81728`의 GitHub Full CI, Build & Test, CodeQL, Canvas visual diff, Native Skia가 통과했다.

## 위험과 후속 처리

- trailing 범위는 `mydocs/` 아래 review·오늘할일뿐이다. 현재 code candidate의 CI를 재사용할 수 있는지
  최신 head의 preflight와 Build & Test aggregate로 다시 확인한다.
- 통합 merge 뒤 원 PR 11건에는 통합 PR·검증·메인터너 보정의 범위를 알리고 close한다. 관련 issue는
  기본 branch 반영 뒤 상태를 재조회해 자동 close되지 않은 것만 수동 종료한다.

## 최종 권고

**최신 trailing review head의 fast-pass, mergeability, 작업지시자 merge 승인을 조건으로 merge 권고.**
누적 검토에서 발견한 유일한 교차 컴파일 결함은 최소 메인터너 보정으로 해소됐고, 전체 Rust·WASM·Studio·CI
검증이 모두 통과했다.

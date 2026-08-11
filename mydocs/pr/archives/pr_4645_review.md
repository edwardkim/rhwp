---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4645 self-review — SVG 폰트 파일 탐색 후보 경계

## 결론

**CI 확인 전 수용 보류.** [PR #4645](https://github.com/edwardkim/rhwp/pull/4645)는
SVG 폰트 파일 탐색 후보를 단일 일반 파일명으로 제한하고, 정상 파일명과 기존 별칭의
탐색 순서는 유지한다. 변경은 두 함수와 두 회귀 테스트에 한정됐으며 self-review에서
별도 blocking finding은 발견하지 않았다.

focused test는 통과했지만 로컬 release-test 전체 검증은 host 디스크 공간 부족으로
compile 단계에서 중단됐다. 최신 PR head의 GitHub Actions 성공, reviewer 확인과
작업지시자의 명시적 merge 승인을 최종 조건으로 둔다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
           visual_fixture_evidence.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, visual_fixture_evidence.md
devel base: 193e26b7ffb05adf5bb2c9e4cb752a9a707310dc
code candidate: 3d74dfd98d61f53a0e4390c6785d5bddef635ad0
trailing review head: 이 문서와 오늘할일을 포함할 후속 docs-only commit
```

변경이 작고 conflict 해결이나 보정 단계가 없으므로 `pr_4645_review_impl.md`는 추가하지
않는다.

## 메타데이터

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4645](https://github.com/edwardkim/rhwp/pull/4645) |
| 관련 이슈 | 별도 공개 이슈 없음 |
| 작성자 | `humdrum00001010` |
| reviewer | `edwardkim` 요청을 시도했으나 fork 계정 권한 부족으로 미지정 |
| base / head | `devel` / `humdrum00001010:renderer/font-lookup-candidate-boundary-33` |
| code candidate | `3d74dfd98d61f53a0e4390c6785d5bddef635ad0` |
| 규모 | 2 files, +50 / -0 |
| 상태 | Open, non-draft, MERGEABLE / BLOCKED; checks 생성 대기 |

원본 저장소 작업 branch push도 권한 부족으로 거부돼 같은 branch를 fork에 push하고
`devel` 대상 PR을 만들었다. reviewer request도 같은 권한 경계에서 거부됐으며, 이를
우회하는 GitHub 상태 변경은 수행하지 않았다.

## 변경 범위와 소유권

`src/renderer/svg.rs::find_font_file_with_weight`가 글꼴 별칭과 확장자로 만든 후보를
설정된 탐색 디렉터리에 결합한다. 따라서 후보가 파일명 하나인지 판정하는 책임도 이
결합 직전의 SVG 폰트 파일 탐색 경계에 둔다.

`is_plain_font_file_name`은 상대 `Component::Normal` 하나만 허용한다. 정상 파일명과
공백이 있는 파일명은 유지하고, 다중 경로 성분은 후보에서 제외한다. parser, document
model, 공통 폰트 조달 순서와 다른 renderer backend는 변경하지 않는다.

## 완료한 로컬 검증

| 게이트 | 결과 |
| --- | --- |
| focused Rust unit | `cargo test --lib renderer::svg::tests::font_ -- --nocapture`: 2 passed |
| formatting | `rustfmt --edition 2021 --check src/renderer/svg.rs src/renderer/svg/tests.rs`: 통과 |
| whitespace | `git diff --check`: 통과 |
| release-test 전체 | compile 중 host disk `No space left on device`로 중단, PASS로 기록하지 않음 |

검증은 다른 작업과 겹치지 않는 Cargo slot에서 task 전용 target을 사용했다. 실패 뒤 실행
중인 Cargo가 없음을 확인하고 task 전용 target만 정리해 약 1.4 GiB를 회수했으며, shared
target과 다른 작업 산출물은 삭제하지 않았다. 이후 Cargo는 재실행하지 않았다.

## 렌더·시각 영향

변경 파일은 `src/renderer` 아래지만 geometry, layout, paint, SVG 요소·속성 또는 정상
글꼴 파일의 출력 바이트를 바꾸지 않는다. 새 sample, golden, 기준 PDF도 없다. 따라서
별도 visual sweep과 review asset은 만들지 않았고, 정상 파일명 허용과 탐색 루트 직접
자식 조회를 focused test로 고정했다.

## 최종 권고

다음 조건이 모두 충족될 때 수용 후보로 다시 판정한다.

1. 이 review 문서와 오늘할일을 포함한 trailing docs-only commit을 같은 PR branch에 push한다.
2. 최신 PR head의 GitHub Actions와 required check가 성공한다.
3. 권한 있는 reviewer가 변경을 확인한다.
4. 작업지시자의 별도 merge 승인을 받는다.

---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4646 리뷰 - 썸네일 출력 상한 정렬

## 결론

**CI 대기 후 수용 권고.** [PR #4646](https://github.com/edwardkim/rhwp/pull/4646)은
HWPX 썸네일 추출의 출력 상한을 기존 브라우저 미리보기 정책과 같은 10 MiB로 정렬한다.
core와 세 브라우저 확장 경로가 선언 크기와 실제 stream 출력을 모두 확인하며, 브라우저
구현은 하나의 공통 collector를 사용한다.

renderer, layout, paint, sample과 fixture는 바꾸지 않으므로 시각 검증 대상이 아니다. 최신
head의 Full CI와 CodeQL이 성공하고 review 조건을 충족한 뒤 수용한다. merge는 이 기록의
범위가 아니다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md
devel base: 193e26b7ffb05adf5bb2c9e4cb752a9a707310dc
code candidate: da8930952420b9a4b2de15ad78b98431af9e85ab
trailing review head: 이 문서와 오늘할일을 포함할 후속 docs-only commit
```

## 메타데이터

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4646](https://github.com/edwardkim/rhwp/pull/4646) |
| 공개 issue 참조 | 없음 |
| 작성자 | `humdrum00001010` |
| base / head | `devel` / `humdrum00001010:parser/thumbnail-output-limit-20260812` |
| code candidate | `da8930952420b9a4b2de15ad78b98431af9e85ab` |
| 규모 | 10 files, +242 / -50 |
| 상태 | Open, non-draft, MERGEABLE / BLOCKED; checks 미보고 |
| reviewer request | `edwardkim` 요청은 현재 계정 권한 부족으로 반영되지 않음 |

원본 저장소 작업 branch push도 현재 계정 권한으로 403이어서 같은 commit을 fork branch에
push하고 `devel` 대상 PR을 만들었다. 이 권한 제약은 code candidate 내용과 base 정합에는
영향이 없다.

## 변경 판단

- Rust 경로는 ZIP central-directory의 비압축 크기를 먼저 확인하고, `Read::take(max + 1)`로
  실제 read 결과도 다시 제한한다.
- Chrome과 Firefox는 같은 shared script를 symlink로 소비한다. Safari는 background보다 먼저
  같은 script를 로드하고 build 산출물에도 복사한다.
- shared collector는 선언 크기가 상한 밖이면 reader를 열지 않고, stream이 선언 크기를
  초과하면 즉시 취소한다. 크기가 일치할 때만 최종 `Uint8Array`를 만든다.
- 기존 CFB 썸네일 경로의 10 MiB 정책도 같은 상수를 사용해 경로별 숫자 드리프트를 막는다.

제품 출력이나 조판은 바뀌지 않는다. 허용 범위 안의 정상 썸네일은 종전과 같은 이미지
파싱 경로로 전달되고, 상한 밖 또는 크기 불일치 입력만 썸네일 없음으로 처리한다.

## 완료한 검증

| 게이트 | 결과 |
| --- | --- |
| `cargo fmt --check` | 통과 |
| `cargo test --lib thumbnail_ -- --nocapture` | 4 passed, 3,509 filtered |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| service-worker 공식 test 명령 | 118 passed |
| JS 구문 검사 / `bash -n rhwp-safari/build.sh` | 통과 |
| `git diff --check` | 통과 |

디스크 제약과 다중 작업의 Cargo 순차 게이트에 따라 로컬 release-test 전체와 확장 package build는
중복 실행하지 않았다. 최신 PR head의 Full CI가 Rust 전체 회귀와 frontend package lane을 담당한다.
Safari Xcode 산출물은 변경하지 않았으며, shared script의 manifest 순서와 build 복사는 결정적
contract test와 구문 검사로 확인했다.

## 최종 권고

blocking code finding은 없다. 이 review 문서와 오늘할일만 추가한 trailing commit을 fork의 같은
PR branch에 push한 뒤, 최신 head의 required checks와 mergeability를 다시 확인한다. reviewer 지정은
권한 있는 maintainer가 수행해야 하며, 별도 승인 전에는 merge하지 않는다.

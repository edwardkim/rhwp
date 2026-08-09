---
kind: pr_review
status: merged-post-review
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4183 검토 - 배포 전 보안 점검 스윕 레시피

## 절차와 대상

~~~text
base route: maintainer_general
modifiers: intake_and_review, local_validation, multi_pr_update_branch,
  review_only_fast_pass
loaded documents: pr_review_workflow.md, pr_review/README.md,
  maintainer_general.md, intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, review_only_fast_pass.md
~~~

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4183](https://github.com/edwardkim/rhwp/pull/4183) / @kevin9327 |
| 관련 이슈 | [#4111](https://github.com/edwardkim/rhwp/issues/4111) |
| contributor 원 head | `1dc664c4c0cbcd3294b03972c412e96cd1cd224f` |
| current-base merge | `2182de665` / `devel` `dea285711` |
| 규모 | 원 변경 1개 파일, +129/-0, 1 commit |
| 가시성 검토 브랜치 | `review/kevin9327-4183-20260808` |
| 시각 검증 | 비대상. `mydocs/` 문서만 변경하고 renderer, fixture, Studio 출력 경로를 변경하지 않는다. |

원 변경은 송신 전 문서를 `inspect hidden-text`, `inspect injection`, `inspect unicode`로 읽기 전용
검사하고, `edit redact --dry-run --no-raw`, redact 적용, sanitize, 재검사로 닫는 레시피 10을
추가한다. inspect 신호를 종료 오류가 아닌 봉투 데이터로 읽는 원칙, `--no-raw`, 개인정보 3건,
`verify.identical`, metadata 10건 제거 설명은 현재 CLI 계약과 일치한다. 저장소의
`samples/field-01.hwp`, 레시피 3·4 상대 링크, `tests/security_corpus_regression.rs`도 존재한다.

## 발견 사항과 메인터너 보정

문서의 목표와 요약 카드는 네 가지 질문을 모두 재스윕한다고 명시했지만 4단계의 실제 명령과 출력은
개인정보와 hidden-text 두 축만 다시 검사했다. injection·unicode를 처리 뒤 다시 실행하지 않으면
“전부 0일 때만 내보낸다”는 게이트를 명령 그대로 자동화할 수 없다. 최종 명령·실측 출력·게이트에
두 inspect 축을 추가하고, 네 봉투의 판정 필드를 각각 명시했다.

새 레시피가 지식 지도에서 발견되도록 기존 레시피 색인에 레시피 10 행도 추가했다. PR이 오래된
`devel`에서 갈라져 먼저 최신 `devel`을 한 번 병합했으며 contributor 원 commit은 변경하지 않았다.
원격에는 이 merge와 보정·review 기록을 한 번에 push해 synchronize와 CI 실행을 한 번으로 제한한다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| reviewer 지정 | GitHub requested reviewer에 `edwardkim`이 반영됐다. |
| 실제 파이프라인 | `target/release/rhwp` v0.8.2와 가짜 주민번호·전화·이메일을 `/tmp/rhwp-pr4183.DwYwXY`에서 실행했다. |
| 초안 판정 | hidden 0, injection 0, unicode 0, redact 개인정보 3건을 확인했다. |
| 처리 판정 | redact 3건, `verify.identical=true`, sanitize `removedCount=10`을 확인했다. |
| 최종 네 축 | redact 0, hidden 0, injection 0, unicode 0이며 unicode `scannedChars=138`을 확인했다. |
| current-base 병합 | contributor head에 `upstream/devel`을 충돌 없이 병합한 `2182de665`를 만들었다. |
| Cargo·시각 검증 | 문서 전용 변경이므로 Cargo, WASM, Skia, 시각 검증을 생략했다. |

## GitHub Actions와 수용 판단

원 head `1dc664c4c`의 [CI 31220566762](https://github.com/edwardkim/rhwp/actions/runs/31220566762)와
[CodeQL 31220566524](https://github.com/edwardkim/rhwp/actions/runs/31220566524)은 동일 PR·branch·head의
`pull_request` event에 귀속되어 성공했다. PR 전체가 `mydocs/` 범위여서 preflight와 Build & Test
aggregate가 성공하고 heavy worker가 skip된 것은 review-only fast-pass B 경로의 정상 결과다.

**메인터너 보정 포함 수용.** current-base merge, 보정, review 기록을 포함한 head `d91ad8fae`의
[CI 31241512361](https://github.com/edwardkim/rhwp/actions/runs/31241512361)과
[CodeQL 31241512293](https://github.com/edwardkim/rhwp/actions/runs/31241512293)은 preflight와
Build & Test aggregate를 성공했고 heavy worker를 skip했다. 최신 head가 `MERGEABLE`·`CLEAN`임을
재확인하고 작업지시자 승인 뒤 admin merge했다. Rust·Skia·WASM broad CI는 중복 실행하지 않았다.

## Merge와 후속 상태

- merge commit: `01f4360acd719657e692284f3d38a25af654359b`
- merge 시각: 2026-08-08 14:23 KST
- `upstream/devel` 포함: merge commit을 fetch하고 로컬 `devel`을 fast-forward했다.
- 관련 이슈 #4111: GitHub Actions가 closing keyword를 확인해 2026-08-08 14:23 KST에 자동 close했다.
  운영 기록 반영 뒤 maintainer comment만 남긴다.
- contributor fork의 `task/4111-recipe10-security-sweep` branch는 삭제하지 않는다.

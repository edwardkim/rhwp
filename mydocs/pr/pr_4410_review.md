---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4410 검토 — N-부모 작업 계보 DAG 지평

## 라우팅

base route: `collaborator_external_pr.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `review_only_fast_pass.md`

loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `review_only_fast_pass.md`,
`codex/docs_and_git_workflow.md`

current head: `67e7c3bcdb29753532c4f4d500cbf2d5d003b6d0` (로컬 메인터너 보정,
remote push 전)

GitHub reviewer assign은 이 독립 검토 작업의 명시적 GitHub mutation 금지 때문에
수행하지 않았다.

## 메타데이터와 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4410](https://github.com/edwardkim/rhwp/pull/4410) / @kevin9327 |
| base | `devel` |
| contributor source | `cc0d4678e2141ad4f04cf904a4117c59cf18d2a3` |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4410` |
| 원 변경 규모 | 2파일, `+474/-0`, 커밋 1개 |
| 작성 시점 원격 상태 | open, mergeable, CLEAN, 문서-only Build & Test 성공 |
| 선행 관계 | PR #4406의 단일-parent lineage 후보를 설계 근거로 참조 |

원 변경은 `trend_merge_dag_2026h2.md`와 #4407 처리결과 문서만 추가한다. Rust,
workflow, fixture, renderer는 바꾸지 않으므로 Cargo와 visual sweep 대상이 아니다.
PR #4406 merge는 이 문서의 M1 **구현 착수 조건**이며, #4410 문서 자체가 #4406
commit 위에 적층돼야 한다는 뜻은 아니다.

## 발견한 차단 결함과 메인터너 보정

원 설계는 material parent에 capsule 파일 무결성만 적용하고, 그 부모 산출물이 자식의
구체적 재료로 실제 사용됐는지는 판정하지 않았다. 현행 replay plan에는 예시로 든
`csv-to-table`과 `insert-image` step도 없으므로 "plan 인자에 재료 바이트가 이미
고정된다"는 전제가 성립하지 않았다.

또한 deep replay 결과를 `planSha256`만으로 cache하면 같은 plan text를 서로 다른
입력 바이트에 적용한 캡슐의 결과를 오재사용한다. capsule bytes hash를 방문 키로
삼는 방식도 같은 bytes 복사본이 서로 다른 폴더에서 상대 parent를 다르게 해석하는
경우 한 계보를 건너뛴다. 합본 예시는 4노드라고 적고 분할 1 + 편집 3 + 합본 1의
5개 작업을 계산했다. C2PA를 모델 카드와 함께 전부 자기 신고로 분류한 설명도
서명된 manifest와 자산/ingredient를 암호학적으로 결속하는 실제 보장을 누락했다.

메인터너 문서 보정 `67e7c3bc`는 다음을 반영했다.

- 모든 parent edge가 부모 output digest, edge binding, 자식 input slot digest를
  3자 대조하도록 `receipt.inputs[]`/`outputs[]`와 slot mapping을 설계했다.
- role은 표현·정책 분류로만 쓰고 primary와 material의 검증 강도를 같게 했다.
- deep cache key를 plan, 정렬된 input slot digest, 실제 tool/execution profile로
  확장하고 완전한 키가 없으면 node별 재실행하도록 했다.
- 방문 키를 canonical 실파일/file identity로 바꾸고 file hash는 무결성과 동일
  내용 보고에만 쓰도록 분리했다. node·edge·queue 상한과 정확한 경계도 DoD에 넣었다.
- v1.0 단일 사슬은 기존 봉투 key/exit를 그대로 출력하고 v1.1 DAG만 새
  `nodes[]`/`edges[]` 계약을 쓰도록 하위호환을 확정했다.
- 합본 시나리오를 분할 s + 편집 a·b·c + 합본 d의 5노드로 바로잡고, 다중
  input/output 영수증이 선행되어야 함을 명시했다.
- C2PA의 무결성·서명 보장과 레시피 재현성을 구분하고 공식 spec을 연결했다.

contributor commit은 rewrite하지 않았고 보정은 원 head의 single-parent 후속
commit이다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| PR head API / fork `ls-remote` / local fetch 교차 대조 | 세 곳 모두 `cc0d4678e2141ad4f04cf904a4117c59cf18d2a3` |
| `python scripts/check_markdown_links.py --changed-from refs/remotes/origin/devel ...` | 통과, 변경 문서 2개 |
| `python scripts/check_document_metadata.py` | 통과, 522개 문서 이상 없음 |
| `python tools/roadmap_progress.py` | 통과, 100개 트랙 집계·결번 0·중복 0·README 일치 |
| `git diff --check` | 통과 |
| 원문 대조 | arXiv 4건의 제목·핵심 수치, C2PA 2.2 spec, Hugging Face `base_model` 공식 문서 확인 |
| Cargo / 시각 검증 | 생략. source·test·fixture·renderer 변경 없음 |

## 리스크와 권고

- task arithmetic, TIES, git re-basin, sparse upcycling, Mixtral 등 식별자를 싣지
  않은 참고 항목의 전량 원문 링크 검증은 문서가 정한 M1 DoD로 남는다.
- PR #4406이 merge 전에 계약을 다시 바꾸면 M1 착수 시 이 설계의 v1.0 정규화와
  회귀 기준을 최신 구현에 맞춰 다시 확인해야 한다.
- 보정은 문서-only지만 contributor source 뒤의 새 head이므로 push 후 최신 preflight와
  Build & Test aggregate를 확인해야 한다.

**메인터너 보정 head의 문서-only CI 통과 후 merge 권고. 실제 merge는 작업지시자의
별도 승인 대상이다.**

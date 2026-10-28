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

latest design correction: `58cf7d4c` (root external input 경계). trailing review 기록이
뒤따르며 remote push 전이다.

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
active asset hard binding과 ingredient 추가 시점 validation record라는 실제 보장을
누락했다.

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

후속 독립 재검토에서는 세 경계를 더 좁혔다. 첫째, file identity를 방문 주키에
사용하면 hardlink alias의 서로 다른 접근 base가 같은 inode라는 이유로 합쳐져 상대
parent 계보 하나를 건너뛸 수 있었다. 둘째, v1.1 "전체 예시"가 4 node·3 edge star로
남아 본문이 정의한 5 node 합본 DAG와 맞지 않았다. 셋째, C2PA 설명이 active asset의
현재 hard binding 검증과 ingredient 추가 시점의 validation record를 구분하지 않았다.

후속 문서 보정 `f42bf99f`는 다음을 반영했다.

- 방문 주키를 `(canonicalized access path, resolution base)`로 한정했다. file-id는
  symlink 보조 확인·보고에만 쓰고 hardlink dedup을 금지했으며, M2 DoD에 hardlink
  alias distinct-lineage 회귀를 추가했다.
- v1.1 봉투 예시를 D→{A,B,C}→S의 5 node·6 edge로 완성했다. node id, 각 edge의
  slot digest, `roots:[4]`, `nodeCount:5`, `edgeCount:6`, head=1 기준 node depth와
  `maxDepth:3`이 모두 일치한다.
- C2PA active asset은 hard binding으로 현재 asset/manifest 변조를 검출하지만,
  ingredient bytes가 보통 포함되지 않아 소비자가 ingredient hard binding을 같은
  방식으로 다시 검증하지 못한다는 2.2 §7.3.2 경계를 기록했다. ingredient 추가 당시
  hard binding·credential 유효성을 검사해 validation record를 남기는 보장과 replay
  재현을 분리했다.

마지막 독립 재검토에서는 root S의 source input에 parent edge가 없는데 T5와 M3가 모든
input slot에 parent binding을 요구해 5-node 예시를 자체 규약상 invalid로 만드는 모순을
확인했다. 후속 설계 보정 `58cf7d4c`는 `parents:[]` root의 입력을 계보 밖
`external` source digest로 명시하고, 모든 비-root input에만 정확히 한 parent binding을
요구한다. root digest는 실행 입력을 결속하지만 제3자 provenance anchor는 아니라는 한계도
기록했다. 이 review의 C2PA 요약도 active asset과 ingredient validation 경계에 맞췄다.

contributor commit은 rewrite하지 않았고 보정은 원 head의 single-parent 후속
commit이다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| PR head API / fork `ls-remote` / local fetch 교차 대조 | 세 곳 모두 `cc0d4678e2141ad4f04cf904a4117c59cf18d2a3` |
| `python scripts/check_markdown_links.py --changed-from refs/remotes/origin/devel ...` | 통과, 변경 문서 2개 |
| `python scripts/check_document_metadata.py` | 통과, 522개 문서 이상 없음 |
| `python tools/roadmap_progress.py` | 통과, 100개 트랙 집계·결번 0·중복 0·README 일치 |
| v1.1 예시 JSON parse·그래프 불변식 대조 | 통과, 5 node·6 edge·root 4·`maxDepth:3`, 모든 edge id 유효 |
| root/input binding 규약 대조 | root S input은 `external` digest, 비-root input 6개는 edge binding 6개와 일대일 |
| `git diff --check` | 통과 |
| 원문 대조 | arXiv 4건, C2PA 2.2 §7.3.2 active/ingredient 경계, Hugging Face `base_model` 공식 문서 확인 |
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

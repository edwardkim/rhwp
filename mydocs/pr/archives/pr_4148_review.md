---
kind: pr_review
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4148 검토 — Codex 프로젝트 메모리 덤프 활성화

## 결론

**문서 전용 Draft PR 생성 및 로컬 검증 통과.** 기존 redirect stub이던
`mydocs/manual/codex/MEMORY.md`를 작업지시자가 확정한 장기 운영 규칙의 활성 진입점으로 전환하고,
이전 redirect 내용은 historical archive로 보존했다. `AGENTS.md`, 루트 canonical manifest와 manual·Codex
문서 지도도 같은 역할과 로딩 순서를 가리킨다.

최종 diff에는 code, sample, PDF, `output/` 변경이 없다. 최신 PR head의 review-only fast-pass preflight와
최종 aggregate가 통과하고 작업지시자 승인을 유지하는 조건으로 Ready 전환 및 merge를 권고한다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
           review_only_fast_pass.md, post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, review_only_fast_pass.md, post_merge.md
base/head at review: devel @ a5698255e / docs/codex-memory-20260807 @ 7508ee061
```

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4148](https://github.com/edwardkim/rhwp/pull/4148) |
| 작성자 / assignee | `edwardkim` / `edwardkim` |
| milestone | `v1.0.0` |
| labels | `documentation` |
| 대상 / head | `devel` / `docs/codex-memory-20260807` |
| 작성 시점 상태 | Draft, OPEN, MERGEABLE, 6 files, +58/-26 |

상태와 head SHA는 작성 시점 참고값이다. merge 전 최신 head, mergeability와 required checks를 다시 확인한다.

## 변경 범위

- `mydocs/manual/codex/MEMORY.md`: 활성 memory 역할과 유지보수자 PR 1차 트리야지 규칙을 기록했다.
- `mydocs/manual/codex/archive/memory_redirect_20260717.md`: 대체된 redirect 내용을 historical snapshot으로
  보존했다.
- `AGENTS.md`: Codex가 프로젝트 메모리 덤프를 문서 로딩 초기에 읽도록 연결했다.
- `mydocs/README.md`, `mydocs/manual/README.md`, `mydocs/manual/codex/README.md`: 활성 memory의 역할,
  canonical manifest와 탐색 경로를 정렬했다.

원래 작업공간에 있던 tracked `output/poc` 삭제 커밋은 별도 브랜치에 보존했고 이 PR에서 제외했다.
정답지 PDF도 최신 `devel`과 SHA-256이 동일해 이 PR 변경에 포함하지 않았다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 변경분 링크·redirect 검사 | 513개 문서 중 변경 6개, redirect stub 30개 검사 통과 |
| 변경 문서 targeted link check | 6개 문서, 내부 상대 링크 이상 없음 |
| `git diff --check devel...HEAD` | 통과 |
| 최종 범위 | 문서 6개만 변경, worktree clean |
| 전체 metadata 검사 | 기존 `mydocs/tech` 2개 문서의 선행 오류 3건만 재현; 이번 변경 문서 오류 없음 |

문서 전용 변경이므로 Cargo, npm, WASM과 시각 검증은 생략했다. 기존 PDF, sample, fixture를 수정하지 않아
시각·fixture 증적 보조 경로도 최종 범위에서 제외했다.

## 위험과 최종 게이트

- 활성 memory는 canonical manual을 대체하지 않으며, 충돌하면 현재 작업지시자와 canonical 절차가 우선한다.
- 세션 상태나 종료 task를 활성 memory에 누적하지 않고 archive에 historical snapshot으로 보존한다.
- PR 전체가 review-only 허용 경로이므로 CI preflight가 B 경로 fast-pass를 선택하는지 확인한다.
- 최신 head의 preflight와 최종 `Build & Test` aggregate 성공을 확인한 뒤에만 Ready 전환 및 merge한다.

## 검토 중 기준선 갱신

Draft 생성 뒤 `devel`이 #4140 merge commit `0fdac31ba`까지 전진해 PR이 충돌 상태가 됐다. 최신
`upstream/devel`을 merge했으며, 충돌은 `mydocs/orders/20260807.md`의 EOF 추가 위치 한 곳뿐이었다.
#4140의 누적 PR 기록을 먼저 두고 #4148 기록을 이어 양쪽 섹션을 모두 보존했다.

merge tree를 최신 `upstream/devel`과 비교한 PR 고유 diff는 문서 8개뿐이며, #4140의 source·test 변경은
기준선 parent에서만 유입돼 PR 고유 변경에 포함되지 않는다. merge commit push 뒤 최신 head의 CI만 최종
판정에 사용한다.

## 최종 권고

로컬 문서 검증과 변경 범위 분리가 완료됐다. archive review와 오늘할일을 추가한 최신 head를 push한 뒤
review-only fast-pass 결과와 mergeable 상태를 확인해 merge한다.

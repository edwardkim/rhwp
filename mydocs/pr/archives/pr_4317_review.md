---
kind: pr_review
status: ci-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4317 검토 - #4314 후속 roadmap 사실과 완료 상태 보정

## 결론

**수용 권고.** [PR #4317](https://github.com/edwardkim/rhwp/pull/4317)은 통합 PR #4314가
`devel`에 반영된 뒤에도 남아 있던 기술 문서의 사실 오류와 진행 상태 드리프트를 바로잡는다.
변경은 `mydocs/` 아래 6개 문서의 사실·집계 보정으로 제한되며 blocking finding은 없다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
           review_only_fast_pass.md (B)
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, review_only_fast_pass.md
devel base: b64c42f374d7b3e84b272e97e6261f0b31a9d325
code candidate: 3936d6002b3a7424be7b1e15585e2ea6e30d8f59
```

별도 `pr_4317_review_impl.md`는 만들지 않는다. 단일 문서 보정 PR이고 추가 구현·충돌 해소·다단계
선택이 없어 이 문서의 검증 및 merge 조건으로 남은 절차가 충분히 명확하다.

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4317](https://github.com/edwardkim/rhwp/pull/4317) |
| 성격 | PR #4314 머지 후 문서 사실 보정, 별도 관련 이슈 없음 |
| 작성자 / assignee | `edwardkim` / `edwardkim` |
| reviewer | 작업지시자 승인 `edwardkim` maintainer self-review |
| GitHub review request | 작성자 자기 PR에는 reviewer 요청을 등록하지 않음; 빈 목록 재확인 |
| milestone / label | `v1.0.0` / `documentation` |
| base / source | `devel` / `docs/fix-pr-4314-doc-facts-20260809` |
| code candidate | `3936d6002b3a7424be7b1e15585e2ea6e30d8f59` |
| 변경 규모 | 1 commit, 6 files, +31 / -24 |
| 초기 상태 | open, non-draft, `MERGEABLE/CLEAN`; 최초 head CI 성공 |

GitHub 작성자 본인은 자기 PR을 `APPROVE`하거나 자신에게 review를 요청할 수 없다. 작업지시자가
self-review 경로를 선택했으므로 reviewer request를 시도하지 않고, 검토 결과는 최신 head CI 성공 뒤
`COMMENTED` review로 남긴다. 이는 독립 승인을 대체하지 않으며 merge는 별도 승인 게이트다.

## 변경 범위와 사실 검토

1. roadmap R40에서 #4110·#4111을 merge PR로 잘못 기록한 부분을 이슈 번호로 바로잡고, 실제
   recipe 09·10 merge PR #4182·#4183을 연결했다.
2. PR #4314로 이미 `devel`에 착지한 R84 OWPML 관찰 노트와 R93 assignee 세션 잠금을
   `[실측]`에서 `[완료]`로 승급하고 통합 PR 및 merge commit `b64c42f37`을 근거로 남겼다.
3. 단계 태그 변경 뒤 roadmap 생성기를 다시 실행해 전수 집계를 완료 35·실측 10·문서 7·이슈 3·
   가설 45, 합계 100으로 동기화했다.
4. HWPX의 `Scripts/headerScripts`·`Scripts/sourceScripts`가 HWP5
   `Scripts/DefaultJScript`로 변환되고, 원천이 없는 contract·누락 script·preview만 fallback으로
   보충된다는 OLE contract 경계를 명확히 했다.
5. HWPX 구현 현황 표가 실제로 7개 항목임을 반영하고, 함수 개수로 잘못 셌던 `shape.rs` 수치 81을
   제거했다. 28개 테스트와 `render_shape()` 근거 범위는 현재 source와 대조해 보정했다.

문서끼리의 상태·링크·집계가 서로 일치하며 제품 코드, public API, fixture, CI workflow와 렌더러는
변경하지 않는다.

## 로컬 검증

code candidate `3936d6002`에서 다음을 확인했다.

| 검증 | 결과 |
| --- | --- |
| `python3 tools/roadmap_progress.py` | PASS, 100개 단계·집계 일치 |
| `python3 scripts/check_markdown_links.py` | PASS, 526개 문서 |
| `python3 scripts/check_document_metadata.py` | PASS, 521개 문서 |
| `git diff --check` | PASS |
| 정정 전 stale 문구 재검색 | 0건 |
| `upstream/devel...HEAD` 변경 범위 | `mydocs/` 아래 6개 파일만 |

코드·WASM·렌더링·fixture 변경이 없는 문서 사실 보정이므로 Cargo, WASM과 시각 검증은 수행하지
않았다.

## GitHub Actions와 review-only 후속 commit

code candidate의 CI preflight, CodeQL preflight와 `Build & Test` aggregate가 성공했다. WASM,
Lint, Frontend, archive, Native Skia와 test shard의 skip은 `mydocs/` 전용 변경을 판정한
review-only fast-pass B의 정상 결과다.

이 review 문서와 오늘할일 기록은 code candidate 뒤의 single-parent trailing review-only
commit이다. push 뒤 최신 head에서도 preflight와 aggregate 성공, `MERGEABLE/CLEAN`을 다시 확인한다.

## 최종 권고

blocking finding은 없으며 다음 조건을 모두 충족하면 merge를 권고한다.

1. review-only 후속 commit을 포함한 최신 head의 fast-pass가 성공한다.
2. 최신 head가 `MERGEABLE/CLEAN`인지 다시 확인한다.
3. maintainer self-review 결과를 `COMMENTED` review로 게시한다.
4. 별도 작업지시자 승인 뒤 PR을 merge한다.

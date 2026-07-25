---
kind: canonical
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# PR 리뷰 · 통합 워크플로우 매뉴얼

**대상**: rhwp 메인테이너, collaborator, 외부 PR 처리 담당
**역할**: 공통 계약과 조건별 절차의 라우터

이 문서는 안정적인 canonical 진입 경로다. 세부 명령과 역할별 절차를 모두 반복하지 않는다.
PR review를 시작하는 에이전트는 이 문서와 [조건별 가이드 선택표](pr_review/README.md)를 먼저 읽고,
선택표가 가리키는 자식 문서를 **작업 전에** 모두 읽는다.

## 1. 공통 계약

rhwp의 PR 처리는 외부 contributor PR, collaborator self PR, collaborator가 매개하는 외부 PR을
구분한다. 권한과 변경 위치가 다르므로 한 경로의 예외를 다른 경로에 일반화하지 않는다.

- **maintainer**는 원본 저장소의 admin 또는 branch-protection bypass 권한을 가진 사람이다.
  코드 동작을 바꾸지 않는 merge 후 운영 기록만 devel에 직접 반영할 수 있다.
- **collaborator**는 write 권한이 있지만 branch protection을 우회하지 않으며, 원칙적으로 PR로
  변경을 반영한다.
- **contributor**는 외부 fork 또는 contributor branch에서 PR을 제출한 사람이다.

소스, 테스트, CI workflow, golden/baseline, 기존 샘플 변경은 maintainer라도 일반 PR과 최신 CI를
기본으로 한다. GitHub review, comment, push, ready 전환, merge, close는 각각 작업지시자의 명시 승인을
받은 뒤에만 수행한다.

## 2. 필수 라우팅

라우팅 판정에는 PR·이슈 metadata, diff, CI 상태를 읽는 행위만 사용한다. 이 판정 단계에서는 reviewer
assign, branch fetch, comment, push 같은 상태 변경을 하지 않는다.

1. 이 모 문서와 [조건별 가이드 선택표](pr_review/README.md)를 읽는다.
2. 아래 표에서 **기본 경로 하나**를 고른다.
3. 변경 범위와 현재 상태에 해당하는 **보조 경로를 모두** 고른다.
4. reviewer assign, local fetch, review 문서 작성, GitHub 변경처럼 상태를 바꾸기 전에 다음 형식으로
   상태 보고 또는 review 문서에 기록한다.

~~~text
base route: <문서명>
modifiers: <없음 또는 문서명 목록>
loaded documents: pr_review_workflow.md, pr_review/README.md, ...
current head: <작성 시점 참고 SHA 또는 재확인 필요>
~~~

| 기본 경로 | 적용 조건 | 반드시 읽을 문서 |
| --- | --- | --- |
| maintainer 일반 | maintainer가 외부 PR을 검토·통합·후속 처리 | [maintainer 일반 경로](pr_review/maintainer_general.md) |
| collaborator self-merge | collaborator가 본인 PR을 merge 후보로 준비 | [collaborator self-merge](pr_review/collaborator_self_merge.md) |
| collaborator 매개 외부 PR | collaborator가 contributor PR head에 검토 기록 또는 보정을 더함 | [collaborator 매개 외부 PR](pr_review/collaborator_external_pr.md) |

| 보조 경로 | 적용 조건 | 반드시 읽을 문서 |
| --- | --- | --- |
| 접수·리뷰 기록 | 모든 정식 PR review | [PR 접수와 리뷰 기록](pr_review/intake_and_review.md) |
| 로컬 검증 | fetch, merge simulation, Cargo, npm, fixture 검증을 수행 | [로컬 검증](pr_review/local_validation.md) |
| 시각·fixture 증적 | renderer/layout/paint, HWP/HWPX/PDF sample, 기준 PDF, 페이지·표·wrap·clipping 주장 | [시각·fixture 증적](pr_review/visual_fixture_evidence.md) |
| 다수 PR·update branch | 대량 유입, 누적 cherry-pick, stale SHA CI 취소, update branch 발생 | [다수 PR과 update branch](pr_review/multi_pr_update_branch.md) |
| review-only fast-pass | code PR 뒤 review 기록만 추가하거나 PR 전체가 문서·허용된 신규 기준 자료뿐임 | [review-only fast-pass](pr_review/review_only_fast_pass.md) |
| merge 후 | 원 코드 PR 또는 후속 기록 PR이 merge됨 | [merge 후속 처리](pr_review/post_merge.md) |
| 재작업·예외 | close/rework, Dependabot, 오래된 base, 대형 PR | [재작업과 예외](pr_review/rework_and_exceptions.md) |

기본 경로 또는 보조 경로가 애매하면, 상태 변경 없이 필요한 metadata를 더 확인한 뒤 작업지시자에게
판단을 요청한다. 편의상 모든 자식 문서를 읽는 방식은 금지한다. 기본 경로 하나와 실제 조건에 맞는 보조
문서만 읽어야 누락과 불필요한 절차를 함께 줄일 수 있다.

## 3. 병렬 실행과 순차 게이트

### 3.1 GitHub Actions의 실제 병렬 구조

[CI workflow](../../.github/workflows/ci.yml)는 다음 의존 그래프로 동작한다. PR review 담당자는 CI가
모든 job을 순차 실행한다고 가정하지 않는다.

~~~text
CI preflight
    ├─ Lint (fmt, clippy, WASM check) ─┐
    └─ Frontend package gates ─────────┴─ gate 충족
       (영향 있음: success, 영향 없음: skipped) │
                         ┌────────────────────┴───────────────────┐
                         │                                        │
                    Build test archive                     Native Skia tests
                         └────────────────────┬───────────────────┘
                                              │
                              Default-feature tests: 8 shards 병렬
                                              │
                                      Build & Test 집계
~~~

- Lint와 Frontend package gates는 preflight 뒤 병렬이다.
- Build test archive와 Native Skia tests는 Lint가 성공하고, Frontend가 필요한 경우 success이거나
  영향이 없어 skipped인 뒤 병렬이다.
- 8개 default-feature shard는 archive와 Native Skia가 모두 성공한 뒤 병렬이며, shard 실패 시
  fail-fast로 나머지를 취소할 수 있다.
- review-only fast-pass는 heavy job이 skipped일 수 있다. 이때도 preflight와 branch protection이
  요구하는 집계 상태를 최신 PR head 기준으로 확인한다.

CodeQL, Render Diff 등 별도 workflow의 결과도 같은 PR head 기준으로 관찰할 수 있다. 이전 head의 run은
현재 head의 required check로 취급하지 않으며, update branch가 있으면 해당 보조 경로의 stale-run 규칙을
따른다.

### 3.2 CI 대기 중 병렬로 가능한 일

아래 작업은 같은 최신 head를 기준으로 하고 결과를 다시 확인하는 조건에서 CI 대기와 병렬로 수행할 수 있다.

- PR/이슈 metadata, diff, 기존 보고서, 기존 CI log의 읽기 전용 조사
- review 문서의 사실·검증 계획 초안, 시각 증적의 출처와 SHA-256 목록 작성
- merge 후 사용할 issue/PR comment의 **초안** 작성
- 다른 PR의 접수 분류. 단, 각 PR의 reviewer assign과 최종 판단은 해당 PR별로 기록한다.

CI가 끝난 뒤 또는 contributor가 새 commit을 push한 뒤에는 head SHA, mergeable 상태, required check를
다시 읽는다. 초안·이전 CI 결과를 최신 head의 최종 판정으로 재사용하지 않는다.

### 3.3 순차로 유지할 일

공유 상태를 바꾸거나 선행 결과가 필요한 작업은 아래 순서를 지킨다.

- 하나의 checkout, target, Cargo cache를 공유하는 cargo test, cargo clippy, cargo build, wasm-pack은
  순차 실행한다. 로컬 검증을 CI처럼 병렬화하지 않는다.
- branch fetch 이후의 merge simulation, cherry-pick, conflict resolution, commit, push, update branch,
  merge와 stale run force-cancel은 대상 SHA를 확인한 뒤 순차로 실행한다.
- 실제 GitHub review/comment, issue close, PR close는 승인과 선행 조건이 갖춰진 뒤에만 게시한다.
- merge 후에는 merge SHA 확인 → 문서·asset의 devel 반영 → 최종 devel sync → issue 상태 확인 및
  comment → branch/worktree/검토 전용 target 정리 순서를 지킨다. raw image URL을 쓰는 comment는
  asset이 devel에 존재한 뒤에만 게시한다.

서로 다른 host, worktree, CARGO_TARGET_DIR, Cargo home이 실제로 분리된 경우에도 로컬 Cargo 병렬 실행은
이 매뉴얼의 기본 경로가 아니다. 필요하면 별도 작업 계획과 작업지시자 승인을 받아 lock·disk·결과 귀속을
명확히 한 뒤에만 사용한다.

## 4. review 산출물의 공통 규칙

PR review 문서는 merge 후에도 모순되지 않아야 한다. draft, mergeable, head SHA, CI 상태는
작성 시점 참고값으로만 적고, 최종 조건에는 항상 최신 head의 CI와 작업지시자 승인을 둔다.

review 문서 경로, review_impl 작성 조건, 사전 판단 report의 범위는 선택한 기본 경로와
[PR 접수와 리뷰 기록](pr_review/intake_and_review.md)을 따른다. 시각 검증을 실제 판단 근거로 사용하면
임시 output 경로만 남기지 말고, 대표 review PNG를 mydocs/pr/assets 아래 안정 경로에 보존한 뒤
그 경로를 review 문서와 실제 GitHub comment에 사용한다.

## 5. 기존 절 번호 대응

기존 링크 경로인 이 파일은 유지한다. 과거 review 문서와 보고서에 남은 절 번호를 해석할 때는 아래 표를
사용한다. 역사 문서를 이관만을 이유로 일괄 수정하지 않는다.

| 이전 절 | 현재 문서 |
| --- | --- |
| 2.0, 2.5, 4.2.1 | [다수 PR과 update branch](pr_review/multi_pr_update_branch.md) |
| 2.1–2.4, 3.1–3.4 | [PR 접수와 리뷰 기록](pr_review/intake_and_review.md) |
| 2.6, 3.5, 3.5.1 | [시각·fixture 증적](pr_review/visual_fixture_evidence.md) |
| 4.1, 4.1.1, 4.2, 4.3, 4.3.1, 4.3.2, 4.4 | [로컬 검증](pr_review/local_validation.md) |
| 5–6 | [maintainer 일반 경로](pr_review/maintainer_general.md) |
| 7, 7.1–7.8 | [merge 후속 처리](pr_review/post_merge.md) |
| 8, 8.2.1 | [collaborator self-merge](pr_review/collaborator_self_merge.md) |
| 9, 9.2.1, 9.3.1 | [collaborator 매개 외부 PR](pr_review/collaborator_external_pr.md) |
| 9.3.2 | [review-only fast-pass](pr_review/review_only_fast_pass.md) |
| 10–13 | [재작업과 예외](pr_review/rework_and_exceptions.md) |

## 6. 문서 유지 규칙

새 규칙은 공통 계약이면 이 문서에, 특정 역할·변경 범위·상태에서만 필요한 절차이면
pr_review/의 해당 자식 문서에 둔다. 같은 명령과 조건을 모 문서와 자식 문서에 복제하지 않는다.

새 자식 문서를 추가하거나 조건을 바꿀 때는 이 문서의 라우팅 표, pr_review/README.md,
mydocs 문서 지도와 AGENTS.md의 로딩 지침을 같은 변경에서 함께 갱신한다. 정보구조 변경이므로
문서 링크와 메타데이터 검사를 수행한다.

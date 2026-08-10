---
kind: canonical
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
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

### 1.1 PR 번호 채번과 review 기록

PR 번호는 PR을 생성할 때 채번된다. 따라서 collaborator self PR의 번호 기반 review 기록은
다음 순서로 같은 PR에 포함한다.

1. 구현과 로컬 검증이 끝난 후보 commit을 원격 작업 branch에 push한다.
2. 작업지시자의 PR 생성 승인 후 Open PR을 생성해 번호 `N`을 받는다. 완료된 후보에
   번호만 확보하려고 Draft를 생성하지 않는다.
3. reviewer assign 승인과 역할별 접수 절차를 수행한 뒤 `pr_N_review.md`와 필요한 오늘할일을
   작성해 같은 source branch에 review 기록 commit으로 push한다.
4. review 기록이 포함된 최신 head의 required check를 확인하고, 작업지시자 승인 후 merge한다.

외부 contributor PR처럼 PR이 이미 존재하는 경우에는 발급된 번호로 바로 review 접수를 시작한다.
Draft는 WIP 공유나 조기 검토가 필요하고 그 상태 변경을 작업지시자가 명시적으로 승인한 경우에만
사용한다. 정확한 생성 순서와 승인 게이트는 [문서와 Git 워크플로우](codex/docs_and_git_workflow.md#internal-task-pr-approval)를
따른다.

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

같은 checkout의 Cargo 검토 명령은 `target/pr-review`를 공유해 **순차 실행**한다. 긴 baseline은 별도
Cargo 명령을 동시에 띄우지 말고 `.config/nextest.toml`의 nextest priority로 같은 run 안에서 먼저
스케줄한다. 이는 고정 target cache의 재사용과 실행 중 산출물의 무결성을 함께 보장한다.

### 3.1 GitHub Actions의 실제 병렬 구조

[CI workflow](../../.github/workflows/ci.yml)는 다음 의존 그래프로 동작한다. PR review 담당자는 CI가
모든 job을 순차 실행한다고 가정하지 않는다.

~~~text
CI preflight
    ├─ Frontend unit/package gate (해당 mode만 실행)
    ├─ Lint (rust=true일 때만)
    ├─ Native Skia tests (native=true, lint 결과를 조건부 확인)
    └─ Rust builders (rust=true, lint success, frontend 예상 결과)
                         │
          Default-feature tests: slow + 일반 shard 1/3, 2/3, 3/3
          (자기 builder success, Native는 required→success / 미해당→skipped)
                         │
                 Build & Test 영향축 집계
~~~

- Frontend unit/package gate는 `frontend_mode`에 맞는 job 하나만 실행하고, Rust lint는
  `rust_required=true`일 때만 실행한다. 둘은 preflight 뒤 서로 독립적으로 시작할 수 있다.
- slow+`2` builder, regular `1` builder, regular `3` builder는 Rust가 필요하고 lint가 성공했으며,
  frontend job이 mode에 맞게 success 또는 skipped인 뒤 실행한다. 각 builder는 자기 Cargo test
  target만 빌드해 `slow`, `1`, `2`, `3` archive 중 맡은 항목만 upload한다.
- Native Skia job은 `native_skia_required=true`일 때만 실행한다. Rust lane이 같이 필요하면 lint success,
  Rust가 불필요하면 lint skipped를 요구하고 frontend mode의 예상 결과도 확인한다.
- `slow shard`와 일반 `2/3`은 slow+`2` builder, 일반 `1/3`은 regular `1` builder, 일반 `3/3`은
  regular `3` builder 성공 뒤 시작한다. Native가 필요하면 Native success, 필요하지 않으면
  skipped를 요구한다. 네 worker는 독립 job이며 집계 job이 각 영향축의 `success|skipped` 조합을 확인한다.
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
다시 읽는다. 초안·이전 CI 결과를 최신 head의 최종 판정으로 재사용하지 않는다. 새 head가 최신
`devel` 병합 또는 update branch를 포함하면 로컬 `devel`과 visibility review branch도 같은 기준선으로
갱신하고, PR 고유 diff와 재실행할 검증 범위를 다시 판정한다. 상세 절차는
[다수 PR과 update branch](pr_review/multi_pr_update_branch.md)의 "2.6 검토 중 기준선 갱신"을 따른다.
collaborator가 contributor PR head에 review-only 기록을 직접 push할 예정이면, 최종 archive review·오늘할일·
증적 commit은 새 head를 fetch하고 reviewer 기록을 그 위의 단순 trailing commit으로 정렬한 뒤에만 만든다.
문서 기록만 만들기 위해 최신 `devel`을 contributor branch에 merge하거나 rebase하지 않는다. branch가 최신
base 반영을 강제하지 않는 정책이면, 같은 PR·같은 source repository·같은 code candidate SHA의 녹색 aggregate를
재사용해 trailing review-only commit을 fast-pass할 수 있다. contributor가 source·test를 새로 push한 경우에는
그 새 code head의 CI를 먼저 통과시킨 뒤 review 기록을 한 번만 이어 붙인다.

#### 3.2.1 최신 `devel` 오늘할일을 보존하는 trailing 기록

contributor source branch의 `mydocs/orders/YYYYMMDD.md`가 최신 `upstream/devel`보다 오래될 수 있다.
이 경우 review 기록을 추가하려고 최신 `devel`의 오늘할일 전체를 source에 복사하거나 `devel`을 merge/rebase하면,
source에 없는 archive link를 도입하거나 today add/add 충돌과 불필요한 full CI를 만들 수 있다.

1. `git fetch upstream devel` 뒤 `git diff HEAD..upstream/devel -- mydocs/orders/YYYYMMDD.md`로 최신 base의
   변경 구간을 확인한다.
2. contributor source에 이미 있는 오늘할일은 보존하고, 현재 PR의 항목만 위 diff에서 변경되지 않은 section
   경계에 추가한다. 최신 `devel`의 다른 PR 기록을 source branch에 복사하지 않는다.
3. trailing 문서 commit을 만든 뒤 최신 `upstream/devel`에서 merge simulation을 수행한다. merge tree의
   `git diff --check`와 변경한 오늘할일·review 문서의 Markdown 링크 검사가 모두 통과해야 한다.

이 방식은 source history를 선형으로 유지하면서, 실제 merge tree에는 최신 `devel`의 기존 오늘 기록과
현재 PR 기록이 함께 남는지 확인한다. 변경되지 않은 경계를 찾을 수 없거나 simulation이 충돌하면 source에
`devel`을 억지로 병합하지 말고 작업지시자에게 보고한다. 불가피하게 current base를 병합해 오늘할일 충돌을
해소한 경우에는 [review-only fast-pass](pr_review/review_only_fast_pass.md)의 `mydocs/` 한정 bridge 검증을
따르며, source·test·workflow·증적 파일 충돌 해소에는 이 예외를 적용하지 않는다.

#### 3.2.2 녹색 GitHub code head의 중복 로컬 전체 회귀 생략

외부 또는 collaborator PR을 **검토**하는 단계에서, 정확한 code head가 이미 GitHub의 Full CI와
변경 범위에 맞는 별도 required check(CodeQL, Render Diff 등)를 모두 통과했고, 검토자가 그 뒤에
source·test·fixture·workflow 보정을 추가하지 않았다면 같은 전체 Rust 회귀를 로컬에서 다시 실행하지
않는다. 이 예외는 contributor의 PR 생성 전 사전 검증이나 maintainer 보정 뒤의 검증 의무를 줄이지 않는다.

다음 조건을 모두 확인해야 한다.

1. review 문서에 기록한 code candidate SHA와 GitHub 녹색 run의 head SHA가 정확히 같다.
2. candidate 뒤의 변경은 review·오늘할일 등 review-only 문서이거나, 검증한 current-base merge tree의
   `mydocs/` 한정 bridge다. 코드, test, fixture, baseline, workflow, PDF/asset 보정은 하나도 없다.
3. 현재 `upstream/devel`과의 merge simulation이 충돌 없이 통과했거나, 허용된 `mydocs/` bridge 검증을
   통과했다. 이때도 `git diff --check`와 변경 문서 링크 검사는 실행한다.
4. renderer/layout 계열이면 focused Rust test와 실제 WASM/브라우저 또는 동등한 시각 검증을 별도로
   실행해, GitHub 전체 회귀가 놓칠 수 있는 이번 검토의 핵심 경로를 확인한다.

이 조건에서는 [로컬 검증의 `release-test` 전체 nextest 회귀](pr_review/local_validation.md)와 Native Skia 전체 묶음처럼 이미 같은 code
candidate에서 성공한 광범위 로컬 회귀를 중복 실행하지 않는다. 이미 시작한 명령을 중지했다면 결과를
`PASS`로 기록하지 말고 중지 사실과 이유를 적는다. review 문서에는 candidate SHA, 재사용한 GitHub
run URL 또는 run 번호, 실행한 focused 검증, 생략한 전체 검증과 사유를 모두 남긴다. 최신 head의
fast-pass 또는 Full CI aggregate 성공은 여전히 merge 직전 다시 확인한다.

### 3.3 순차로 유지할 일

공유 상태를 바꾸거나 선행 결과가 필요한 작업은 아래 순서를 지킨다.

- 하나의 checkout, `target/pr-review`, Cargo cache를 공유하는 cargo test, cargo clippy, cargo build,
  wasm-pack은 순차 실행한다. 로컬 검증을 CI처럼 여러 Cargo 실행으로 병렬화하지 않는다. 장시간 테스트는
  `.config/nextest.toml`의 우선순위로 같은 nextest 실행 안에서 먼저 시작한다.
- branch fetch 이후의 merge simulation, cherry-pick, conflict resolution, commit, push, update branch,
  merge와 stale run force-cancel은 대상 SHA를 확인한 뒤 순차로 실행한다.
- 실제 GitHub review/comment, issue close, PR close는 승인과 선행 조건이 갖춰진 뒤에만 게시한다.
- merge 후에는 merge SHA 확인 → 문서·asset의 devel 반영 → 최종 devel sync → issue 상태 확인 및
  comment → branch/worktree/검토 전용 target 정리 순서를 지킨다. raw image URL을 쓰는 comment는
  asset이 devel에 존재한 뒤에만 게시한다.

서로 다른 host, worktree, CARGO_TARGET_DIR, Cargo home이 실제로 분리된 경우에도 로컬 Cargo 병렬 실행은
이 매뉴얼의 기본 경로가 아니다. 필요하면 별도 작업 계획과 작업지시자 승인을 받아 lock·disk·결과 귀속을
명확히 한 뒤에만 사용한다.

### 3.4 GitHub Markdown 본문 전송

여러 단락의 review·comment·issue comment에는 실제 LF 줄바꿈을 전송한다. 셸 큰따옴표 안의 `\n`은
줄바꿈이 아니라 문자 그대로이므로 `--body "...\n..."` 또는 `--comment "...\n..."`로 게시하지 않는다.

- PR review와 PR comment는 실제 줄바꿈을 담은 임시 Markdown 파일을 만들고 `--body-file`로 보낸다.
  `gh pr review N --repo edwardkim/rhwp --approve --body-file <review.md>`,
  `gh pr comment N --repo edwardkim/rhwp --body-file <comment.md>`처럼 실행한다.
- 여러 단락의 issue 후속 기록은 `gh issue close N`과 `gh issue comment N --body-file <comment.md>`로
  분리한다. `gh issue close --comment`는 한 줄의 짧은 기록에만 쓴다.
- `gh api --input <json>`을 쓸 때는 JSON의 `\n` escape가 실제 LF로 해석되는 유효 JSON인지 확인한다.
- 게시 직후 해당 review/comment API의 `body`를 읽어 literal `\\n`이 남지 않았는지 확인하고, 임시 본문
  파일은 정확한 경로만 정리한다.

## 4. review 산출물의 공통 규칙

PR review 문서는 merge 후에도 모순되지 않아야 한다. draft, mergeable, head SHA, CI 상태는
작성 시점 참고값으로만 적고, 최종 조건에는 항상 최신 head의 CI와 작업지시자 승인을 둔다.

### 4.1 완료한 검증의 시제

로컬 CI 성격의 검증(Cargo, npm, lint, fixture, 시각 검증)을 이미 실행해 종료 결과를 얻었다면,
review 문서에는 계획형이나 미래형으로 쓰지 않는다. 실행한 명령, 대상 head, 결과를 과거형 사실로
기록한다.

- 올바른 예: "로컬 검증의 `release-test` 전체 nextest 회귀를 실행해 통과했다."
- 잘못된 예: "PR 전에 전체 테스트를 실행할 예정이다."

아직 실행하지 않은 GitHub Actions, contributor의 새 push, 작업지시자 승인 뒤의 ready 전환·merge·
후속 정리는 미래 조건으로 분리해 적을 수 있다. 완료된 로컬 결과와 대기 중인 외부 조건을 한 문장에
섞어 검증 상태가 불명확해지지 않게 한다.

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

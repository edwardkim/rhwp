# 작업 기록 — task_m100_3790 Stage 5B

- **이슈**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **브랜치**: `issue-3790-stage5b-codeql-languages`
- **worktree**: `tmp/issue-3790-stage5b-codeql`
- **최초 기준**: `upstream/devel` `8ea92cdad120` (#4341 merge)
- **최신 동기화 기준**: `upstream/devel` `32ecfd113690` (#4512 merge)
- **상태**: Draft PR #4519 최신 devel 반영, reviewer·검토 기록 완료, 최종 full CI·CodeQL 확인 대기

## 선행 정리

- Stage 5A PR #4341은 2026-08-11 merge commit `8ea92cdad120d2db2c9097dc2ffd2df804939f74`로
  `devel`에 반영됐다.
- merge 뒤 Stage 5A 전용 worktree, 로컬 브랜치와 fork 원격 브랜치를 정리했다.
- Stage 2.6 controller prototype의 유일본인 `tmp/issue-3790-stage26`과 로컬 브랜치는 보존했다.

## required status 확인

2026-08-11 작업지시자의 WRITE collaborator 인증으로 GitHub API를 직접 조회했다.

- repository permission은 `WRITE`이며 `admin=false`, `maintain=false`다.
- 상세 `GET /branches/devel/protection`은 404이고 ruleset·GraphQL protection rule은 노출되지 않았다.
- 공개 `GET /repos/edwardkim/rhwp/branches/devel` 응답은 `protected=true`와 required context
  `Build & Test` 하나를 반환했다. app id는 GitHub Actions `15368`이다.
- 따라서 현재 `Analyze (javascript-typescript)`, `Analyze (python)`, `Analyze (rust)`, GHAS `CodeQL`은
  branch protection required check가 아니다. collaborator도 branch metadata로 이 값을 직접 확인할 수
  있으며, 상세 관리 구성 열람·변경은 admin 권한이 필요하다.
- 보호 규칙은 바뀔 수 있으므로 PR 생성 전 같은 branch metadata를 다시 조회한다.

## 설계 보정

- PR head가 자기 분석 언어를 줄이지 못하도록 `pull_request.base.sha`의 classifier만 sparse checkout해
  실행한다. PR 파일 목록은 API로 수집하며 credential은 checkout에 남기지 않는다.
- push·schedule·workflow_dispatch와 checkout·API·classifier·출력 검증 실패는
  `javascript-typescript,python,rust` full로 닫는다.
- matrix는 세 언어를 계속 생성해 `Analyze (...)` check identity를 보존한다. 선택되지 않은 언어 job은
  명시적 no-op success로 끝내 check 부재에 따른 영구 pending 가능성을 제거한다.
- checkout·CodeQL init·Rust toolchain·analysis는 선택된 언어에서만 실행한다.
- Stage 5A의 candidate-bound 재사용은 세 Analyze job과 GHAS `CodeQL`을 계속 독립 확인한다.
  `codeql_languages=none`에서 GHAS check가 없으면 후속 review-only 재사용은 fail-closed한다.

## 구현

- `.github/workflows/codeql.yml` preflight에 trusted classifier checkout, PR 파일 수집, classifier 실행과
  허용 언어 집합 finalizer를 추가했다.
- candidate-bound workflow run·job을 읽는 기존 preflight REST 호출의 의도를 토큰 권한에도 명시하도록
  read-only `actions: read`를 선언하고 계약 테스트로 고정한다.
- preflight output으로 `codeql_languages`, `classification_status`, `impact_reason`, `impact_authority`를
  노출하고 Job Summary에 판정 근거를 남긴다.
- 고정 세 언어 matrix의 선택되지 않은 lane에는 no-op step을 두고 실제 분석 step을 정확한 token
  membership 조건으로 묶었다.
- `.github/workflows/ci.yml`의 기존 classifier 설명을 Stage 5B 소비 관계와 일치시켰다.
- `scripts/tests/test_codeql_workflow.py`는 trusted-base·full fallback·허용 집합 검증·고정 job 이름과
  선택 step wiring을 장기 workflow 계약으로 고정한다.

## focused 검증

- TDD RED에서 trusted classifier/fail-closed wiring과 선택 언어/no-op wiring 부재를 검출하는 2건이
  예상대로 실패했다.
- `python3 -m unittest scripts/tests/test_codeql_workflow.py` — 11/11 통과.
- CI가 실행하는 연관 Python workflow 계약 10개 파일 — 90/90 통과.
- `node --test scripts/tests/ci-impact-classifier.test.cjs` — 28/28 통과.
- `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml` — 통과.
- `git diff --check` — 통과.

변경 범위가 workflow·정적 계약 테스트·문서뿐이므로 Cargo와 제품 테스트는 적용하지 않는다. 원격 push와
PR 생성은 별도 승인 뒤 진행한다.

## PR #4519 1차 CI와 리뷰 보정

- candidate `d14e29c307ca68393afdd5a2813c64c77fe19769`의 CI run `31410523565`, CodeQL run
  `31410523372`, 세 Analyze job과 GHAS `CodeQL`이 모두 성공했다. workflow 변경은 trusted classifier에서
  `fail-closed:workflow-contract`로 판정돼 세 언어 full 경로를 검증했다.
- 리뷰 [#5243938913](https://github.com/edwardkim/rhwp/pull/4519#issuecomment-5243938913)의 F1을
  수용했다. preflight가 cancel·runner 장애로 output을 발행하지 못하면 빈 언어 값으로 세 no-op success가
  되는 경계를 consumer-side `SELECTED_LANGUAGES`의 세 언어 fallback으로 닫는다.
- F2를 수용해 `actions: read`가 candidate-bound workflow 조회를 위한 최소 읽기 권한임을 기록하고 테스트로
  고정한다.
- F4를 수용해 fast-pass Summary의 언어·authority·status를 `n/a (fast-pass)`로 표시하고 fast-pass 사유를
  별도로 남긴다.
- F3에 따라 reviewer `edwardkim`을 지정하고
  [`pr_4519_review.md`](../pr/archives/pr_4519_review.md)를 추가했다.
- TDD RED에서 consumer fallback 부재와 fast-pass Summary 오표시 2건을 확인했다. 보정 뒤 CodeQL 계약
  12/12, classifier 28/28, `actionlint`, `git diff --check`가 통과했다.
- `python3 -m unittest discover -s scripts/tests -p 'test_*.py'`는 Homebrew Python 3.14에 Pillow가 없어
  `test_visual_sweep` import에서 중단됐다. 코드 실패가 아니므로 Pillow 12.2.0을 포함한 Codex 번들
  Python으로 같은 discover 범위를 재실행했고 188/188이 통과했다.

## 최신 devel 반영과 최종 검증 준비

- `upstream/devel` `32ecfd1136905c7b1bb26b16c47579a16143d305`를 merge commit
  `64175764120e97eb9af0f0a55be4da86c072cdc6`으로 충돌 없이 반영했다.
- current-base merge 뒤 CodeQL workflow 계약 12/12, 전체 Python 188/188, classifier 28/28,
  `actionlint`, `git diff --check`가 통과했다.
- 최종 문서 trailing commit을 push한 뒤 같은 head에서 수동 `workflow_dispatch` CI와 CodeQL full lane을
  시작한다. 완료 확인과 Draft 해제·merge는 작업지시자가 별도로 수행한다.

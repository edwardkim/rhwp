---
kind: pr_review
status: local-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4341 검토 — #3790 CodeQL 재사용 안전 경계와 Rust 중복 prebuild 제거

## 결론

**보정 head의 full CI·CodeQL 대기.** Rust CodeQL의 기본 build mode와 내부 autobuild를 유지하면서
중복 cargo cache·수동 prebuild를 제거한 원 변경은 같은 run A/B에서 raw SARIF 32건과 fingerprint,
유효 source extraction이 동등함을 확인했다. self-review가 발견한 GHAS 단일 check의 관측 범위 과장,
실제와 반대인 test timing, 죽은 시각 판정 코드, 최신 devel 충돌과 기록 누락을 code candidate
`07ab54c5d3981fb14f3e6e9e13904ef8d6038c80`에서 보정했다.

최신 `upstream/devel` 병합 뒤 workflow 실행 경로가 바뀌었으므로 이전 head의 녹색 check는 최종 merge
근거로 재사용하지 않는다. 최종 merge 조건은 보정 head의 GitHub Actions 통과, 실제 reviewer 확인과
작업지시자 승인이다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md(4.3 CI workflow),
           multi_pr_update_branch.md(2.6 기준선 갱신)
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, multi_pr_update_branch.md,
                  codex/docs_and_git_workflow.md
code candidate head: 07ab54c5d3981fb14f3e6e9e13904ef8d6038c80
current base: upstream/devel 0664e6568e9bc5a50ff6472db8f9eb5825d569c0
```

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4341](https://github.com/edwardkim/rhwp/pull/4341) |
| 작성자 | `postmelee` (collaborator self-merge) |
| 대상 / source | `devel` / `postmelee:issue-3790-stage5a-codeql-safety` |
| code candidate | `07ab54c5d` — 최신 devel 병합·focused 검증 완료, 원격 상태는 push 뒤 재확인 필요 |
| 규모 | review 문서 전 기준 9 files, +755 / -58 — workflow 2, 계약 테스트 2, 계획·기록 5 |
| 관련 issue | [#3790](https://github.com/edwardkim/rhwp/issues/3790), [#4080](https://github.com/edwardkim/rhwp/issues/4080) |
| review | [self-review #4898256773](https://github.com/edwardkim/rhwp/pull/4341#pullrequestreview-4898256773), 실제 reviewer `jangster77` 요청 상태 |

## 변경 범위와 안전 계약

- review-only fast-pass는 candidate에 연결된 세 `Analyze (...)` job 성공을 각각 요구한다.
- candidate SHA의 GHAS `CodeQL` check도 같은 workflow run attempt 이후에 시작되고 success여야 한다.
- 단일 GHAS check는 실측상 첫 언어 analysis에서 종결되므로 뒤에 도착한 언어의 policy 결과까지
  보증한다고 해석하지 않는다.
- check의 `started_at`이 없거나 workflow run attempt보다 이르면 현재 check가 없는 것으로 처리해 full
  CodeQL로 닫는다. 이전 attempt의 check를 재사용하지 않는다.
- Rust matrix의 check identity, 기본 build mode, stable toolchain, 정상 Code Scanning upload와 내부
  `autobuild.sh`는 유지한다. 수동 cache restore/save와 중복 `cargo build`만 제거한다.
- 제품 Rust 코드, renderer·layout·paint·pagination, HWP/HWPX fixture와 golden은 바꾸지 않는다.

## self-review 보정

| 항목 | 판정과 처리 |
| --- | --- |
| F1 GHAS 관측 범위 | candidate `c2674bd33`의 check는 Python analysis와 같은 `12:49:58Z`에 시작해 `12:50:00Z`에 끝났고, JavaScript/TypeScript·Rust analysis는 각각 `12:50:28Z`, `12:57:08Z`에 생성됐다. 단일 check에서 뒤의 언어 policy 결과를 추론하지 않도록 코드 주석과 문서를 좁혔다. |
| F2 반대 timing mock | GHAS check가 Python 뒤, JavaScript/TypeScript·Rust보다 먼저 끝나는 실제 순서로 fixture를 보정했다. |
| F3 죽은 코드 | check-run에 없는 `created_at`, 유한값으로 파싱되는 `Date.parse(0)`, 선행 filter 때문에 도달 불가능한 identity mismatch 분기를 제거했다. `started_at` 누락 test를 추가했다. |
| F4 `ci.yml` 충돌 | 최신 devel을 병합하고 PR의 CodeQL 테스트와 devel의 Docker·release installer·release package·setup 테스트를 모두 보존했다. |
| F5 review 문서 | 이 문서를 `mydocs/pr/archives/pr_4341_review.md`에 추가했다. |
| F6 기록 불일치 | Ready 상태와 최신 devel 기준을 계획·작업·오늘할일에 반영했다. #1667 계획에는 cache namespace 유지가 역사적 계약임을 명시했다. |
| 테스트 단계명 | `test_codeql_stage5a_workflow.py`를 장기 계약 이름 `test_codeql_workflow.py`로 바꾸고 CI·wiring 참조를 함께 갱신했다. |

## 검증

### 로컬 focused 검증

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts/tests/test_codeql_workflow.py` | 7/7 통과 |
| 연관 workflow 계약 테스트 10개 파일 결합 실행 | 86/86 통과 |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 28/28 통과 |
| `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml` | 통과 |
| `git diff --check` | 통과 |
| `git merge-base --is-ancestor upstream/devel HEAD` | 통과 — 최신 devel이 code candidate의 조상 |

Rust 제품 코드나 Cargo 동작을 변경하지 않으므로 Cargo 검증은 적용하지 않았다. workflow 조건과 장기
계약 test, 최신 GitHub Actions 결과가 이 변경 범위의 기본 검증이다.

### 원격 검증 참고값

- 이전 code candidate `c2674bd336a26448d1673f7f70389cb8fc2a0ce8`의
  [CodeQL run 31314188222](https://github.com/edwardkim/rhwp/actions/runs/31314188222)와
  [CI run 31314188326](https://github.com/edwardkim/rhwp/actions/runs/31314188326)은 성공했다.
- 같은 SHA A/B의 기본 build mode에서 no-prebuild가 59초(8.4%) 빨랐고 raw SARIF result 32건과
  fingerprint, 성공 추출 1,097파일이 같았다.
- 위 결과는 설계 근거로 보존하지만 최신 devel merge와 workflow 보정 뒤의 최종 required check로
  재사용하지 않는다. code candidate `07ab54c5d`의 full CI·CodeQL 결과를 새로 확인한다.

## 시각·fixture 판단

별도 시각 검증은 적용하지 않았다. PR 고유 변경은 CI workflow·계약 테스트·운영 기록이며 renderer 출력,
HWP/HWPX/PDF sample, golden과 시각 기준을 변경하지 않는다.

## 잔여 조건

- 이 review-only trailing commit을 push한 뒤 최신 PR head SHA와 `MERGEABLE / CLEAN` 회복을 확인한다.
- 최신 head의 full CI·CodeQL과 required checks를 확인한다.
- self-review `COMMENTED`는 approval이 아니므로 실제 reviewer와 작업지시자의 merge 승인을 별도로 받는다.
- merge 뒤에는 `post_merge.md`에 따라 merge SHA, devel 반영, issue #3790의 Stage 5A 상태를 확인한다.

## 최종 권고

code candidate는 최신 devel 위에서 self-review F1–F6을 보정하고 focused 검증을 통과했다. review 문서
commit을 push한 뒤 최신 head의 full CI·CodeQL을 확인하고, 실제 reviewer 검토와 작업지시자 승인을 받아
merge 후보로 진행한다.

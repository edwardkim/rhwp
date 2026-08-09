---
kind: pr_review
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4170 검토 — #4040 파일 게이트 native-skia test 3건 CI 회복

## 결론

**merge 후보.** 파일 게이트된 native-skia integration test 3건이 CI 의 어떤 job 에서도 실행되지 않던
검증 공백을 회복한다. 세 테스트를 로컬에서 `native-skia` 로 직접 실행해 통과를 확인했으므로, 죽은
테스트를 되살린 것이 아니라 **유효한 가드가 CI 에서 빠져 있던 것**이다.

Stage 4([PR #4032](https://github.com/edwardkim/rhwp/pull/4032))의 Rust·Native Skia 조건화 의미는
바꾸지 않는다. `CHANGES_REQUESTED` 리뷰의 두 P2를 보정했고 관련 계약 테스트 전건 무회귀를 확인했다.
최종 merge 판단은 보정 push 뒤 새 head의 전체 GitHub Actions와 재검토를 기다린다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md(4.3 CI workflow),
           multi_pr_update_branch.md(2.6 기준선 갱신)
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, multi_pr_update_branch.md, post_merge.md,
                  codex/docs_and_git_workflow.md
current base: upstream/devel 1ede9c7acf08cd836f5c19d6283083b59229c7a5
```

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4170](https://github.com/edwardkim/rhwp/pull/4170) |
| 작성자 | `postmelee` (collaborator self-merge) |
| 대상 / head | `devel` / `issue-4040-native-targets` (upstream branch) |
| 규모 | 보정·최신 devel 병합 뒤 작성 시점 11 files, +1003 / -1 — 기능 변경은 workflow 6줄 + classifier 7줄 |
| 관련 issue | [#4040](https://github.com/edwardkim/rhwp/issues/4040), [#4132](https://github.com/edwardkim/rhwp/issues/4132), [#2083](https://github.com/edwardkim/rhwp/issues/2083), [#2292](https://github.com/edwardkim/rhwp/issues/2292), [#2293](https://github.com/edwardkim/rhwp/issues/2293), [#3790](https://github.com/edwardkim/rhwp/issues/3790) |
| metadata | label·milestone·review request 없음 |

## 변경 범위와 안전 계약

- `Native Skia tests` job 에 `--test` 3개를 **release-test·release 두 경로 모두** 추가한다. job 의 조건·
  의존은 건드리지 않는다.
- classifier `NATIVE_SKIA_RUST_FILES` 에 세 경로를 추가한다. 판정이
  `rust=true, native=true, reason=classified:native-skia-rust` 로 바뀌며, 기존 `issue_2225`·`render_p37`
  과 동일한 계약이 된다.
- `render_required` 는 `false` 로 유지된다. 기존 두 파일과 같다.
- 다른 축의 판정은 변하지 않는다.

## 조사에서 계획 전제를 두 번 고쳤다

사후에 다듬지 않고 경위를 남긴다.

### 판별식이 좁았다

처음 쓴 정확 일치 패턴 `#!\[cfg\(feature\s*=\s*"native-skia"\)\]` 은
`render_p37_direct_pdf_export.rs` 의 중첩 게이트를 놓친다.

```rust
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
```

**"알려진 파일이 발견되는지" 를 단언하는 테스트를 함께 넣어둔 덕에 RED 재현에서 잡혔다.** 그 단언이
없었다면 발견 패턴이 조용히 절반만 훑는 상태로 merge 됐을 것이다.

### `issue_2225` 의 성격을 잘못 봤다

계획서는 `issue_2225`·`render_p37` 을 "파일 게이트 정상 사례" 로 적었으나, `issue_2225` 는 **함수
게이트**다. 정확한 구분은 파일 게이트 4건(3건 누락 + `render_p37` 정상)이고 `issue_2225` 는 별개 축이다.

이 정정은 #4132 의 전제에도 영향을 준다 — 아래 참조.

## 계약 테스트 설계

| 테스트 | 방향 |
| --- | --- |
| `test_every_file_gated_native_skia_test_is_wired` (신규) | 저장소 → job·classifier |
| `test_native_skia_integration_targets_are_classifier_inputs` (기존, 유지) | job → classifier |
| `test_discovery_finds_the_known_file_gated_native_skia_tests` (신규) | 발견 패턴 — 좁은 쪽 |
| `test_discovery_rejects_negated_gates_and_quoted_attributes` (신규) | 발견 패턴 — 넓은 쪽 |
| `test_native_skia_targets_run_in_both_profiles` (신규) | 두 프로파일 대칭 |

기존 단방향 테스트를 지우지 않고 남긴 이유는 역방향(job 에는 있는데 classifier 에 없음)을 계속
감시하기 위해서다. 둘이 합쳐 양방향이 된다.

`#4080` 의 `test_workflow_contract_wiring.py` 와 같은 방어를 적용했다 — 발견 패턴이 망가지면 부류
강제가 조용히 무의미해지므로 패턴 자체를 단언한다.

### 첫 보정 — 부정과 줄 주석

최초 판별식은 정규식 한 줄이라 `feature = "native-skia"` 가 **어떤 문맥에 있든** 매치했다. 좁은 쪽
오탐(중첩 게이트 놓침)은 RED 재현에서 잡혔지만 넓은 쪽은 남아 있었다.

| 입력 | 최초 | 반영 후 |
| --- | --- | --- |
| `#![cfg(not(feature = "native-skia"))]` | 매치 | 제외 |
| `//! \`#![cfg(feature = "native-skia")]\` 로 게이트한다` | 매치 | 제외 |

`not(...)` 로 게이트된 파일은 native-skia 빌드에서 **오히려 cfg-out** 되므로, 배선을 요구하면 0건짜리
target 이 생긴다. 인용은 애초에 게이트가 아니며, 이 저장소는 한국어 `//!` 문서에 cfg 속성을 자주
인용한다. 둘 다 저장소에 해당 파일이 생기기 전에는 드러나지 않으므로 합성 입력으로 고정했다.

첫 보정은 괄호 균형 파싱 + 부정 문맥 추적과 줄 주석 제거였다. 기존 RED 재현은 그대로 성립했지만,
이 방식도 cfg의 `all`·`any` 의미와 문자열·블록 주석을 다루지 못했다.

### 2026-08-08 `CHANGES_REQUESTED` 리뷰 보정

[리뷰](https://github.com/edwardkim/rhwp/pull/4170#pullrequestreview-4888431770)는 다음 두 P2를 지적했다.

1. `any(feature = "native-skia", target_os = "linux")`, raw string, 블록 주석을 실제 파일 게이트로
   오인한다.
2. #4040 classifier fixture가 세 경로를 한 입력에 묶어, 일부 경로가 목록에서 빠져도 한 경로만
   일치하면 통과한다.

판별기는 Rust 문자열·중첩 블록 주석·줄 주석을 같은 길이의 공백으로 가린 뒤 brace depth 0의 실제
crate inner attribute만 찾는다. cfg meta-item은 작은 재귀 하강 parser로 `all`·`any`·`not`을 읽고,
다른 atom을 미정으로 둔 3값 평가에서 **native-skia를 끄면 반드시 거짓이고 켜면 가능성이 생길 때만**
파일 게이트로 판정한다. 리뷰의 세 입력과 중첩 부정·중첩 블록 주석·블록 내부 inner attribute를
합성 회귀로 고정했다.

classifier 쪽은 기존 `Native Skia integration test changes` 단위 테스트에 세 경로를 추가했다. 각
경로를 단독 `files` 입력으로 실행해 `rust=true`, `native_skia=true`, `render=false`,
`reason=classified:native-skia-rust` 전체 결과를 독립적으로 단언한다. 실제 #4040 세 파일 묶음 fixture는
통합 시나리오로 유지한다.

### 최신 devel 기준선 병합

원 리뷰 head `1166e5946`을 확인한 뒤 최신 `upstream/devel` `1ede9c7ac`을 merge commit
`cd427c37e`로 반영했다. 충돌은 `mydocs/orders/20260807.md` 하나였고, PR 쪽 `#4080`·`#4040`·`#4132`
기록과 devel 쪽 `PR #4174` 기록을 모두 보존했다. 작업지시자는 보정 commit의 기존 PR branch push와
보정 완료 코멘트 게시를 승인했다.

## 검증

### 로컬 실행 — 세 테스트 실제 통과

`native-skia` 로컬 빌드가 가능해 원격 CI 를 기다리지 않고 직접 실행했다.

| target | 테스트 | 결과 |
| --- | --- | --- |
| `issue_2293_chart_png_text` | `chart_png_renders_text_labels` | ok (0.18s) |
| `issue_2292_chart_png_clip` | `chart_png_renders_full_bbox_not_top_left_fragment` | ok (0.09s) |
| `issue_2083_hide_fill_page_background` | `hide_fill_page_renders_opaque_white_not_transparent_black` | ok (0.08s) |

### 회귀

| 검증 | 결과 |
| --- | --- |
| workflow 계약 테스트 5개 파일 | 63 passed / 0 failed (보정 후 최신 devel) |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 28 passed / 0 failed; 세 경로 단독 입력 포함 |
| `actionlint .github/workflows/ci.yml` | 통과, 진단 없음 |
| `node --check scripts/ci-impact-classifier.cjs` | 통과 |
| `git diff --check` | 통과 |
| `python3 -m unittest discover -s scripts/tests` | 관련 포함 102건 통과 후 `test_visual_sweep.py` 1건 import 오류 — 로컬 Python에 Pillow 미설치 |

### RED 재현과 뮤테이션

수정 전 신규 계약 테스트가 정확히 세 파일을 지목하며 실패했다.

| 뮤테이션 | 결과 |
| --- | --- |
| job 에서 `issue_2293` 두 경로 제거 | 1건 실패 |
| classifier 에서 `issue_2292` 제거 | 2건 실패 |
| release 경로에서만 `issue_2083` 제거 | 1건 실패 |
| 발견 정규식을 중첩 미지원으로 축소 | 1건 실패 |

## 시각·fixture 판단

별도 시각 증적은 적용하지 않았다. PR 고유 변경은 CI workflow·classifier·계약 테스트이며 renderer·
layout·paint·pagination·golden 출력을 바꾸지 않는다. 회복시킨 세 테스트 자체가 PNG 잉크 존재를
단언하는 검증이므로, 그 통과가 시각 축의 증적을 대신한다.

## 절차 기록

`docs_and_git_workflow.md` §Issue Workflow 11단계를 따랐다. 다만 두 가지 이탈이 있었고 PR 생성 전에
보정했다.

- **#4132 등록 전 동일 증상 선행 검색 누락** — 사후에 세 검색어로 확인해 중복 없음을 확인했다.
- **오늘할일 미작성** — 작업지시자 지적으로 PR 생성 전에 드러나 `mydocs/orders/20260807.md` 를
  작성하고 이 PR diff 에 포함했다. §8.2.1 의 "최초 remote push 와 PR 생성 전에 포함" 조건을 지켰다.

## 잔여 위험과 후속

- **Native Skia job 소요시간 증가**를 원격 CI 로 실측한다. 현재 368~382초 기준이며 로컬 실행이 각
  0.08~0.18초였으므로 증가폭은 작을 것으로 보나 단정하지 않는다. 과다하면 범위를 재검토한다.
- #4132 는 이 PR 이 세우는 파일 게이트 규약 위에서 판단한다. 이 PR 에서 확인한 `issue_2225` 의 중복
  실행 선례를 그 이슈에 정정 코멘트로 남긴다.
- #4040 은 원격 CI 로그 확인 뒤 close 한다.

## 최종 권고

보정 commit을 기존 PR branch에 push하고 완료 코멘트를 게시한다. 그 새 head의 전체 GitHub Actions와
재검토를 확인한 뒤 별도 merge 승인을 받아 collaborator self-merge 한다. merge 뒤에는 `post_merge.md`에
따라 devel sync, branch·worktree 정리, Native Skia job 로그 확인을 수행한다.

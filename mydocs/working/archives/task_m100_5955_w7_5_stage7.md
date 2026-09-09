---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_5955.md
last_verified: 2026-08-25
---

# Task M100 #5955 — Stage W7.5-7 self-review·최종 보고서·PR 준비

## 1. 판정

Stage W7.5-7의 로컬 작업을 완료했다. 전체 diff self-review에서 blocker 2건을 발견해 정정했고, 최신
`upstream/devel@385e93b2c317d1f50d874fd655e88cf4b2a1ba07`과 충돌 없는 merge tree를 만들었다. 정정된
후보와 최신 base를 합친 review worktree의 변경 범위별 gate도 통과했다.

remote push와 PR 생성은 수행하지 않았다. PR 번호가 없으므로 번호를 예측한 `pr_N_review.md`와 오늘할일도
만들지 않았다.

## 2. self-review findings

### 2.1 같은 decision plane의 교차 projection successor

`validateSuccessorGraph()`는 successor의 `decisionPlane`만 비교했다. 그 결과 reducer가 금지하는
`canvas2d-webfont` retired rule과 `canvaskit-sfnt` active rule의 successor/predecessor 연결을 수기 registry
변조에서는 허용했다. 두 projection은 `supply` plane을 공유할 수 있지만 backend 책임은 다르므로 보호
불변식 위반이다.

validator가 predecessor와 successor의 projection ID도 같아야 한다고 강제하도록 정정했다. canonical v2를
변형해 같은 plane·다른 projection successor를 구성하는 negative test를 추가했고, 정정 전 오류 0건이던
재현이 정정 후 `cross-projection successor`로 실패한다.

### 2.2 Stage 6 보고서의 절대 host path

Stage W7.5-6 보고서는 host 식별 경로를 기록하지 않았다고 판정했지만 Cargo target 한 줄에 사용자 절대경로가
남아 있었다. 저장소 상대경로 `target/pr-review`로 바꿨고 #5955 계획·stage·조사·fixture 범위에서 POSIX·
Windows 절대경로가 더 남지 않았음을 검사했다.

## 3. self-review 후 focused 검증

| gate | 결과 |
| --- | --- |
| Node syntax 2개 | 통과 |
| v2 registry focused | 23/23 |
| v2 registry deterministic check | 통과 |
| projection generator check | 통과 |
| 전체 `font_rule_*.test.mjs` | 93/93 |
| `git diff --check` | 통과 |

Stage W7.5-6의 92/92는 당시 실제 실행 수치로 보존하고, 추가 negative contract를 포함한 최신 수치는
93/93으로 구분했다.

## 4. 최신 base merge simulation

`upstream/devel`은 최초 작업 기준선보다 40커밋 전진했다. merge base는
`8a880baea3be295477b62196958c2ada90e3f505`이고 최신 base와 #5955 양쪽이 함께 수정한 파일은 0개였다.
정정된 working candidate는 main index를 바꾸지 않는 임시 Git object로 만들었고, 최신 base와의
`git merge-tree --write-tree`가 clean tree를 생성했다.

해당 tree의 독립 review worktree 결과는 다음과 같다.

| gate | 결과 |
| --- | --- |
| merge tree diff check | 통과 |
| v1/v2 registry·projection·baseline | 통과 |
| font-rule Node | 93/93 |
| Rust unit-tier | 4,221 tests / 299 modules / drift 0 |
| prepared integration inventory | 911 source / 4,267 attrs / 32 suite + 9 exception / 41 of 48 target |
| W7 public projection focused Rust | 3/3 |
| `cargo fmt --all -- --check` | 통과 |

integration prepare 결과와 임시 worktree는 검증 뒤 제거했다. Stage W7.5-6 이후 source 정정은 JavaScript
validator의 negative guard뿐이고 latest base와 path overlap이 없으므로, 이미 통과한 release·Native Skia·
Docker WASM 전체 묶음을 로컬에서 반복하지 않았다. 최신 PR head의 GitHub Full CI는 재사용 대상이 아니며
PR 생성 뒤 반드시 새로 통과해야 한다.

## 5. 문서 현행화

- [최종 보고서](../../report/archives/task_m100_5955_report.md)를 추가했다.
- [canonical font fallback 전략](../../tech/font_fallback_strategy.md)을 schema 2.0 active-only authority와
  append-only change-set 운영으로 현행화했다.
- `mydocs/README.md` canonical manifest의 확인일을 맞췄다.
- [Stage W7.5-6 보고서](task_m100_5955_w7_5_stage6.md)의 식별 host path를 제거했다.

## 6. 제출 경계와 다음 게이트

- 실제 font mapping, metric, paint, supply, font asset은 변경하지 않았다.
- 새 HWP/HWPX/PDF sample, golden, baseline과 visual asset은 없다.
- private corpus, Hyper-V Oracle, font bytes와 식별 경로를 사용하거나 기록하지 않았다.
- generated integration suite·manifest는 제출하지 않는다.
- #4967의 rank 8 correction은 시작하거나 승인하지 않았다.

관련 변경 문서 7개의 내부 상대 링크, `cargo fmt --all -- --check`와 `git diff --check`는 통과했다. 전역
document metadata 검사는 #5955가 수정하지 않은 기존 문서 4개의 front matter 누락 16건을 보고했다.
이번에 추가·현행화한 장기 문서의 front matter와 canonical manifest는 일치하며, 기존 metadata 부채는
#5955 diff에 섞지 않았다.

다음은 Stage W7.5-7 결과 승인과 경계 커밋이다. remote push와 PR 생성은 이후 각각 별도 승인 대상이다.

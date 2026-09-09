# PR #6728 검토 기록

## 접수 정보

| 항목 | 내용 |
| --- | --- |
| PR | [#6728](https://github.com/edwardkim/rhwp/pull/6728) `시험(studio): 어울림 그림 이동 뒤 화면 되감김을 e2e 로 잠근다 (#6202 studio 축 - 결함 아님)` |
| 작성자 | `planet6897` (`planet6897/rhwp:test/6202-studio-e2e-guard`) |
| reviewer | `jangster77` (2026-09-04 사전 배정) |
| base | `devel` |
| 검토한 code head | `572b66372e558d4208d3875e71f131191ce617a4` |
| 규모 | 1 file, +99/-0: `rhwp-studio/e2e/issue-6202-picture-move-reflow.test.mjs` |
| 작성 시점 참고 상태 | `MERGEABLE`, `CLEAN`, non-draft. merge 직전에 최신 head와 required check를 다시 확인해야 한다. |

## 라우팅과 범위

- base route: `collaborator_external_pr.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, `pr_review/collaborator_external_pr.md`, `pr_review/intake_and_review.md`, `pr_review/local_validation.md`, `pr_review/review_only_fast_pass.md`, `pr_review/visual_fixture_evidence.md`

이 PR은 제품 renderer, layout, WASM, sample 또는 fixture를 바꾸지 않는다. studio에서 그림 드래그 중 이미 문서 변경과 다시 그리기가 일어난다는 현재 동작을 브라우저 E2E로 고정하는 test-only 변경이다. 따라서 기준 PDF 또는 별도 visual sweep asset을 수용 근거로 사용하지 않았다. 스크린샷 crop은 E2E의 관측 수단이며, 제품 출력 fidelity의 독립적인 visual sweep 통과를 뜻하지 않는다.

## 관련 이슈와 범위 제한

- [#6202](https://github.com/edwardkim/rhwp/issues/6202)는 `156483689_1-1_...hwp`에서 어울림 그림 이동 뒤 본문이 옛 배제 밴드를 유지한다는 현상을 기록한 OPEN 이슈다.
- 이 PR은 `samples/143E433F503322BD33.hwp`의 square 어울림 그림을 아래로 72px 움직이는 studio 드래그 경로만 검증한다. 원 이슈의 동일 문서, 저장 후 재열기, 모든 placement 조합을 재현하거나 #6202 전체를 닫는 변경이 아니다.
- PR 본문에는 closing keyword가 없으므로 #6202는 merge 뒤에도 OPEN으로 유지한다. 이 review는 원 이슈의 renderer/layout 결론을 변경하지 않는다.

## 검증

### GitHub code candidate

`572b66372e558d4208d3875e71f131191ce617a4`에서 다음 required aggregate와 실행 worker가 성공했다.

- CI [33868655617](https://github.com/edwardkim/rhwp/actions/runs/33868655617): `Build & Test`, `Frontend package gates` 성공. Rust/WASM/nextest worker는 이 test-only 영향 정책의 expected skip이다.
- CodeQL [33868655917](https://github.com/edwardkim/rhwp/actions/runs/33868655917): JavaScript/TypeScript, Python, Rust 분석 성공 및 aggregate neutral.
- Render Diff [33868655371](https://github.com/edwardkim/rhwp/actions/runs/33868655371): preflight와 Canvas visual diff 성공.
- Adapter inter-diff [33868655618](https://github.com/edwardkim/rhwp/actions/runs/33868655618), Proptest [33868655449](https://github.com/edwardkim/rhwp/actions/runs/33868655449) 성공.
- trusted CI impact policy [33869501519](https://github.com/edwardkim/rhwp/actions/runs/33869501519) 성공.

### focused browser E2E

검토 source head에서 다음을 실행해 성공했다.

```bash
cd rhwp-studio
node e2e/run-with-vite.mjs -- node e2e/issue-6202-picture-move-reflow.test.mjs --mode=headless
```

- 그림 y: `476.2 -> 548.2`
- 그림이 비켜간 자리의 글자 띠 puppeteer screenshot: `49456B -> 46692B`, `동일=false`
- 두 assertion 모두 통과했다. 드래그가 문서에 반영되고, 대상 글자 띠가 이전 픽셀 상태로 남지 않는 것을 확인했다.

`package.json`에 직접 실행 스크립트가 없는 것은 기존 수동 E2E 다수와 같은 운용 범위다. 이를 이 PR 단독의 CI 배선 누락이나 머지 차단 사유로 해석하지 않는다. 반대로 `devel`의 108개 전체 수동 E2E sweep은 이 PR candidate 검증이 아니며, 그 전체 결과를 이 PR의 전체 E2E green 증적으로 사용하지 않았다.

## 검토 결과

### 잔여 risk

- 테스트는 합성 mouse handler 호출과 한 개의 square-wrap 문서를 대상으로 한다. 실제 사용자 입력, 다른 wrap/anchor 조합, #6202 원본의 저장-재열기 경로는 별도 검증 범위다.
- 신규 E2E는 현재 frontend package gate가 직접 실행하지 않는 수동 E2E 집합에 속한다. 이 저장소의 현행 CI 분류 정책을 바꾸지 않는다.

## 최종 판정: 승인

- 검토 대상 head `572b66372e558d4208d3875e71f131191ce617a4`는 주장한 좁은 studio 드래그 회귀 계약을 focused browser E2E와 해당 head의 GitHub required check로 충족한다.
- 이 review와 오늘할일은 code candidate 뒤에 추가하는 review-only trailing commit이다. source, test, fixture, workflow 또는 PDF/asset을 추가로 바꾸지 않는다.
- merge 전 조건: trailing head가 같은 source repository/PR의 green code candidate를 trusted fast-pass로 재사용하는지, 최신 required aggregate가 success 또는 정책상 expected skip인지, `MERGEABLE`/`CLEAN`인지 재확인하고 작업지시자의 merge 승인을 받는다.
- merge 후 조건: `post_merge.md`에 따라 merge SHA의 devel CI를 확인한다. #6202를 자동 close하거나 현 시점에 contributor comment를 게시하지 않는다.

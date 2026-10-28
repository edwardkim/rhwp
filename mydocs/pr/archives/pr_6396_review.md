---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6396
issue: 6395
author: postmelee
---

# PR #6396 review — 쪽 나누기 후 새 쪽 캐럿 표시

## 라우팅과 metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#6396](https://github.com/edwardkim/rhwp/pull/6396) |
| 관련 issue | [#6395](https://github.com/edwardkim/rhwp/issues/6395), `Closes #6395` 연결 |
| 작성자·검토자 | `postmelee` self-review; collaborator 본인 PR이므로 reviewer request 없음 |
| base / head | `devel` / `codex/issue-6395-page-break-caret-reveal` |
| code candidate | `ca28099d4af8218ae58d3a894a4a2cf03fd44196` |
| 규모 | trailing 기록 전 11 files, `+488/-0`, 3 commits |
| 상태 | Open non-draft, `MERGEABLE`, `mergeStateStatus=BLOCKED`; required checks 시작 단계 |

- base route: `collaborator_self_merge.md`.
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_self_merge.md`,
  `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`.
- mergeability, head SHA와 CI는 작성 시점 참고값이며 merge 전 최신 PR head에서 다시 확인한다.
- 변경이 단일 원인·단일 구현 단계이고 추가 보정이나 선택 분기가 없어 별도 `pr_6396_review_impl.md`는
  만들지 않았다.

## 변경 범위와 원인

- WASM page break는 새 문단 cursor position을 즉시 반환하지만, mutation renderer 선택과
  `VirtualScroll` page offset 갱신은 비동기로 끝난다.
- 기존 첫 `updateCaret()`은 이전 page layout을 사용해 아직 없는 새 page index의 offset을 0으로 계산했고,
  layout 완료 뒤 새 좌표로 캐럿을 다시 계산해 reveal하는 경계가 없었다.
- current mutation revision의 `CanvasView.refreshPages()` 직후 내부 완료 이벤트를 발행하고, 쪽/단 나누기와
  그 history snapshot에서만 one-shot reveal을 소비하도록 정정했다.
- 일반 텍스트 mutation은 예약을 만들지 않으며, Rust/WASM 편집 명령과 document pagination 알고리즘은
  변경하지 않았다.

범위 밖 변경은 없다. 계획·stage·최종 보고 문서는 이슈 #6395의 Hyper-Waterfall 계보를 기록한다.

## 로컬 검증

- `cd rhwp-studio && npx tsc --noEmit`: 통과했다.
- `cd rhwp-studio && npm test`: 1,246건 중 1,245 통과, 1 skip, 실패 0으로 끝났다.
- `cd rhwp-studio && npm run build`: 통과했다. 기존 성격의 Vite chunk-size 경고만 발생했다.
- `CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
  npm run e2e:page-break-caret`: 실제 headless Chrome에서 통과했다.
- `python3 scripts/check_e2e_manifest.py`: tracked E2E 121개와 manifest 121행의 정합을 확인했다.
- `git diff upstream/devel...HEAD --check`: 통과했다.
- `python3 scripts/check_markdown_links.py mydocs/pr/archives/pr_6396_review.md
  mydocs/orders/20260830.md mydocs/report/task_m100_6395_report.md`: 변경 문서 3개의 내부 상대 링크가
  통과했다.
- `python3 scripts/check_document_metadata.py`: 저장소 전체 검사에는 이번 PR이 바꾸지 않은 기존 4개 문서의
  metadata 누락 16건이 남아 있어 실패했다. 이번 변경 문서 경로에는 새 진단이 없었다.

Rust source·test/baseline helper, npm/editor public API, HWP/HWPX fixture가 바뀌지 않아 Rust lint/Cargo 전체
회귀와 editor package 검증은 적용하지 않았다.

## 사용자-visible·시각 증적 판정

`CanvasView` 파일은 바뀌지만 document Canvas paint, 페이지 수, 조판 결과를 바꾸는 수정은 아니다. 따라서
HWP/PDF fixture나 기준 PDF를 이용한 PDF/SVG visual sweep은 이번 주장에 맞는 검증이 아니다.

대신 150% 배율의 실제 Chrome에서 `Meta+Enter`를 입력한 E2E가 다음 사용자-visible 계약을 직접 검증했다.

- cursor: section 0, paragraph 1, offset 0, page index 1
- 새 page offset: `1713.75`
- DOM 캐럿: `1912.2px`, 계산 기대값 `1912.2px`
- 편집 영역: `scrollTop=1214`
- viewport 안 캐럿: `698.2..718.15px / 738px`

이 수치는 문서 fidelity의 visual sweep 통과를 뜻하지 않고, PR이 변경하는 캐럿·viewport 추종 계약의 실제
브라우저 증적이다. HWP/HWPX/PDF fixture나 외부 첨부 asset은 없다.

## 발견한 위험과 후속 범위

- `columnBreak`와 쪽/단 나누기 undo·redo는 같은 history type allowlist를 unit test로 고정했지만 개별 browser
  E2E는 없다. 현 구현은 같은 mutation 완료 이벤트를 사용하므로 별도 결함이 재현될 때 후속 E2E를 분리한다.
- 한글 2024 동작은 사용자 제보를 기대 계약으로 사용했고 자동화로 직접 재측정하지 않았다.
- one-shot 예약은 current revision 확인 뒤 완료 이벤트에서만 소비하므로 stale mutation이 캐럿을 먼저
  이동할 위험을 제한한다.

## 최신 devel 충돌 해소 — 2026-08-30

- 이전 PR head `5481452f8`은 Frontend package gates, Canvas visual diff, Build & Test를 포함한 분류된 CI를
  모두 통과했다. 그 뒤 `devel`이 `d3b40a3d7`까지 이동하면서 PR은 `CONFLICTING/DIRTY`가 됐다.
- 최신 `upstream/devel@d3b40a3d7`을 merge하고 `mydocs/orders/20260830.md`와
  `rhwp-studio/src/engine/input-handler.ts`의 content conflict를 해소했다.
- 오늘할일은 최신 base 항목을 모두 보존한 뒤 #6395 표만 추가했다. `InputHandler`는 최신 HF selection·IME·
  submode snapshot 복원을 모두 보존하고 #6395의 15줄만 PR 고유 diff로 유지했다.
- 최신 WASM을 새로 만들고 TypeScript, Studio/editor 전체 test 1,313건, production build 245 modules,
  실제 Chrome E2E, E2E manifest 122/122를 통과했다. 상세 근거는
  [Stage 2](../../working/archives/task_m100_6395_stage2.md)에 기록했다.
- 이 merge commit이 push되면 이전 녹색 head를 최신 head의 최종 CI 근거로 재사용하지 않는다. 최신 head의
  required checks와 mergeability를 다시 확인한다.

## 최종 권고와 남은 조건

**조건부 수용.** 원인과 수정 경계가 좁고 TypeScript 전체 회귀 및 실제 Chrome E2E가 이슈 #6395의 수용
기준을 충족했다.

- 이 trailing 문서 commit을 포함한 최신 PR head의 required checks가 성공해야 한다.
- merge 전 최신 head SHA, `MERGEABLE/CLEAN` 상태를 다시 확인해야 한다.
- 작업지시자의 별도 merge 승인이 있어야 squash merge할 수 있다.

## self-review 보정 — 2026-08-30

추가 검토에서 one-shot 예약의 문서 수명 경계 누락을 확인해 merge 전에 보정했다. mutation renderer 선택이
문서 전환과 경합하면 `document-layout-refreshed`가 생략될 수 있는데, 기존 `InputHandler.deactivate()`는
`CaretLayoutReveal`의 pending 상태를 지우지 않아 다음 문서의 첫 full mutation이 이전 예약을 소비할 수
있었다.

- 보정 계획 commit: `dfd6e124e`
- 보정 code candidate: `a21bbc9a0`
- `CaretLayoutReveal.clear()`를 추가하고 문서 교체 공통 경계인 `deactivate()`에서 호출했다.
- 예약 뒤 초기화하면 다음 consume이 `false`인지 단위 테스트를 추가했다.
- focused unit 4/4, TypeScript, Studio/editor 전체 test 1,314건(1,313 pass·1 skip), production build 245
  modules, 실제 Chrome E2E, E2E manifest 122/122, `git diff --check`를 통과했다.
- Chrome E2E의 page offset `1713.75`, DOM 캐럿 `1912.2px`, `scrollTop=1214`, viewport
  `698.2..718.15px / 738px` 계약은 보정 뒤에도 유지됐다.

allowlist 일반화, `cursor.updateRect()` 정리, 첫 pass의 잠재적 flicker, E2E timeout 진단성은 #6395 범위의
정확성 결함으로 재현되지 않아 이번 code candidate에는 섞지 않았다. 최신 보정 head의 required checks와
`MERGEABLE/CLEAN`, 작업지시자의 별도 merge 승인은 계속 최종 조건이다.

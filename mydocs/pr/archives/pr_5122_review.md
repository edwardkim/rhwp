---
kind: pr-review
status: active
pr: 5122
issue: 4961
author: edwardkim
base: devel
head: task_m100_4961
last_verified: 2026-08-17
---

# PR #5122 자체검토 - 문자별 Font Decision Trace와 backend 선택 계보

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#5122](https://github.com/edwardkim/rhwp/pull/5122) |
| 작성자 | `edwardkim` (collaborator self-review) |
| base / head | `devel` / `task_m100_4961` |
| 검증 code candidate SHA | `da326d0dc8f8dd8696c886ec6ace1a008e530dce` |
| 변경 규모 | 51 files, +6,398 / -379 |
| draft | `false` |
| merge state | `CLEAN` |

collaborator 자체 PR이므로 reviewer를 지정하지 않았다. 최종 판단 전 최신 head와 GitHub Actions,
mergeability를 다시 확인한다.

## 변경 범위

- 기존 font 선택과 조판 결과를 바꾸지 않는 opt-in, versioned, bounded Font Decision Trace를 제공한다.
- 문서 face에서 `layoutMetric`과 backend paint 후보로 이어지는 문자별 결정 계보를 분리해 설명한다.
- W1 Font Rule Ledger identity를 trace와 결합하고 target-independent hash와 상한을 고정한다.
- native Skia, Canvas2D, CanvasKit의 관측 가능 범위와 실패 reason을 기능 탐지 방식으로 구분한다.
- 저장소에 추적된 공개 HWP/HWPX fixture만 회귀 근거로 사용하며 private corpus와 비배포 font bytes는
  commit·trace에서 제외한다.

## 자체검토에서 발견해 보정한 차단 결함

초기 PR head의 `DocumentCore::get_font_decision_trace_native`는 query마다 빈
`SkiaLayerRenderer::new()`를 만들었다. 실제 renderer가 준비한 custom·bundled font inventory가 없는데도
native 관측을 `complete`로 기록할 수 있어, trace가 실제 paint 후보 계보를 설명한다는 계약을 위반했다.

- renderer-bound query는 이미 준비된 `SkiaLayerRenderer` snapshot만 관측하도록 변경했다.
- standalone `DocumentCore` native query는 `nativeRendererSnapshotRequired`로 fail-closed한다.
- custom inventory가 실제로 `source=custom`과 `nativeGlyphCoverageObserved`를 남기는 회귀를 추가했다.
- 수정 commit은 `4c135b70718364f5b64beb3dc244c2133370fbdd`다.

최신 `devel`의 PR 원본-only test 정책도 적용했다. `tests/cases/issue_4961_font_decision_trace.rs`는
유지하되 generated suite와 manifest 변경은 candidate에서 제거했다. CI와 검토자는 `--prepare`로 파생
산출물을 만들고 검증한다.

## 검토 결과

| 항목 | 판정 |
| --- | --- |
| 기존 fallback target·layout metric·font asset 선택 변경 | 없음 |
| trace query의 font path read·font load 시작 | 없음 |
| renderer snapshot 없는 native 관측의 성공 승격 | 차단 후 보정 완료 |
| 출력 상한·결정론·portable hash | 통과 |
| private corpus·재배포 불가 font bytes 포함 | 없음 |
| 남은 차단 결함 | 발견하지 못함 |

Canvas2D는 브라우저가 실제 glyph face를 공개하지 않으므로 CSS supply까지만 관측한다. 활성화되지 않은
CanvasKit typeface와 외부 oracle도 관측 성공으로 승격하지 않는다. 이 한계는 capability와 reason으로
노출되며 기존 fallback 또는 렌더링 결과를 추정값으로 바꾸지 않는다.

## 로컬 검증

| gate | 결과 |
| --- | --- |
| 최종 candidate focused trace | default 4 / native Skia 6 passed |
| suite prepare/check | 570 sources / 2,500 static attrs, 통과 |
| unit tier | 4,225 tests / 298 modules, 통과 |
| manifest Node 계약 | 14 passed |
| archive workflow 계약 | 12 passed |
| CI impact policy / workflow | 31 / 27 passed |
| Markdown links / fmt / diff | 통과 |

제품 source가 같은 직전 candidate에서 release-test nextest 6,542 passed/38 skipped, Native Skia
58+2+4, Studio 957 passed/1 skipped, editor 24, fresh WASM 공개 E2E 3건을 통과했다. 이후 합성한
`upstream/devel@1fe3348af`는 CI·test routing과 문서만 변경했으며, 최종 candidate에서는 위 focused
회귀와 새 정책 계약을 재실행했다. 정확한 최종 head의 전체 Rust·Native·frontend·WASM 검증은 아래
GitHub Actions가 다시 수행했다.

## GitHub Actions 검증

| 검증 | 결과 |
| --- | --- |
| code candidate | `da326d0dc8f8dd8696c886ec6ace1a008e530dce` |
| CI | [run 32018062403](https://github.com/edwardkim/rhwp/actions/runs/32018062403) 성공 |
| CodeQL | [run 32018062020](https://github.com/edwardkim/rhwp/actions/runs/32018062020) 성공 |
| Render Diff | [run 32018061948](https://github.com/edwardkim/rhwp/actions/runs/32018061948) 성공 |
| CI 주요 lane | lint, frontend, Native Skia, archive, regular 3 shards, slow shard, aggregate 모두 성공 |
| CodeQL languages | JavaScript/TypeScript, Python, Rust 모두 성공 |

## 위험과 후속 조건

- API는 opt-in trace이므로 기존 rendering hot path의 기본 동작과 serialization을 바꾸지 않는다.
- trace는 관측 불가능한 backend 상태를 추측하지 않고 `unsupported` 또는 명시적 reason으로 닫는다.
- 실제 10k corpus coverage와 조판 위험 순위는 제품 선택을 바꾸지 않는 후속 #4962 범위다.
- 이 review·오늘할일 trailing commit은 제품 source와 test routing을 변경하지 않는다.

## 현재 권고

**병합 권고.** 자체검토에서 발견한 native snapshot 차단 결함은 보정됐고, 최신 `devel`을 포함한 정확한
code candidate에서 CI·CodeQL·Render Diff가 모두 성공했으며 PR은 `CLEAN/MERGEABLE`이다. trailing
review-only 문서 head의 Actions가 실패·대기 없이 완료되고 메인테이너가 병합을 별도로 승인하면 정상
merge 절차를 진행한다.

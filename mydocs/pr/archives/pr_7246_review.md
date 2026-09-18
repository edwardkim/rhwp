---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7246 검토

## 최종 판정

**승인** — 2026-09-18 통합 검토.

배치를 바꾸지 않고 누락된 검출을 추가한다. focused 3건과 통합 off-canvas 래칫 16 partition이 통과했다. 기존 정상 3011411 대조군은 신고가 늘지 않는다.

통합 최종 head CI 확인이 필요하다. corrected 86712의 제거된 옛 baseline 행을 되살리지 않았다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7246](https://github.com/edwardkim/rhwp/pull/7246) — 진단: off-canvas 가 컨테이너 자손의 그리는 개체도 본다 (#7190) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `aa4c9a291215e6707b9211e4d2d72ad7fda1dd44` |
| 규모 | 3 files, +255 / -15 |
| 조회 당시 mergeability | `CONFLICTING` / `DIRTY` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `aa4c9a291215e6707b9211e4d2d72ad7fda1dd44` | `1bc7c273d2f944a0ed478b1a4cb5a70975a848bf` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35268927139) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35268926710) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35268926036) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35268926689) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35268926857)

## 범위·조판 계약 검토

관련 이슈: [#7190](https://github.com/edwardkim/rhwp/issues/7190). off-canvas 진단의 컨테이너 가로 합집합에 실제 개체 자손을 포함한다.

deep_vertical_union_bbox의 수평 union 대상만 확장한다. visible/editor_only/양수 bbox 조건과 상위 컨테이너 단위 중복 억제를 유지한다. 새 신호와 렌더 회귀를 구분해 baseline +delta를 검토했다.

주요 소비 경로: [src/diagnostics/layout_anomaly.rs](../../../src/diagnostics/layout_anomaly.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 실행한 검증과 한계

- 통합 제품의 `issue_7190_off_canvas_object_descendant`: **3 tests run: 3 passed, 223 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

진단 전용 변경이므로 이 PR 자체의 Visual Sweep은 비해당이다. 실제 자손 좌표와 용지 우변의 차를 기대값으로 삼는 focused 검사를 실행했다.

통합 off-canvas baseline: 16/16 통과.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 이슈·다음 단계

#7190의 기존 배치 수정과 이번 검출 축을 함께 확인한 후 종료 판단. 이 검출 변경이 그림 배치를 고쳤다고 쓰지 않는다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.

---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7259 검토

## 최종 판정

**머지 보류** — 2026-09-18 통합 검토.

[P2] #7244와 중복 SVG 변경은 합쳤으나 crop을 가진 None 경로가 backend마다 다르다. Skia renderer는 None→Zoom으로 바꾸며 Zoom은 decoded 전체 비율로 crop_src를 그린다. SVG None/Zoom은 crop을 읽지 않는다. 실제 Skia raster와 CanvasKit 화면 비교는 아직 없다.

crop 유무를 포함해 실제 backend 출력의 source/destination 사각형을 일치시키고 Native Skia·Canvas2D·CanvasKit raster 증거를 남긴다. #7244의 쪽 배경 확대 범위도 별도 처리한다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7259](https://github.com/edwardkim/rhwp/pull/7259) — 수정(renderer): 칸 배경 그림 채우기 None 을 칸에 맞춰 축소·가운데로 그린다 (#7235) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `91ec20dc30992961ae5e4f3dc1b3332bdf73fac7` |
| 규모 | 11 files, +435 / -2 |
| 조회 당시 mergeability | `MERGEABLE` / `CLEAN` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `91ec20dc30992961ae5e4f3dc1b3332bdf73fac7` | `2182dcadb49705d5ed872e44d8114b01bf71caad` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35299914871) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35299915002) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35299914403) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35299914946) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35299914909) · [CI 6](https://github.com/edwardkim/rhwp/actions/runs/35299914602)

## 범위·조판 계약 검토

관련 이슈: [#7235](https://github.com/edwardkim/rhwp/issues/7235). 셀 그림 None/Zoom을 SVG·Skia·CanvasKit에서 contain으로 그리도록 보완한다.

ImageNode None → Skia Zoom 변환 → image_conv의 contain 사각형과 crop_src → paint. #7244의 별도 None 분기는 crop 비율을 사용하므로 통합 후 의미가 둘로 갈린다. 단순 이미지 사각형 4검사와 JS helper 검사는 crop 경계를 보호하지 않는다.

주요 소비 경로: [rhwp-studio/src/view/canvaskit-renderer.ts](../../../rhwp-studio/src/view/canvaskit-renderer.ts), [rhwp-studio/src/view/canvaskit/image-replay.ts](../../../rhwp-studio/src/view/canvaskit/image-replay.ts), [src/renderer/skia/image_conv.rs](../../../src/renderer/skia/image_conv.rs), [src/renderer/skia/renderer.rs](../../../src/renderer/skia/renderer.rs), [src/renderer/svg.rs](../../../src/renderer/svg.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 실행한 검증과 한계

- 통합 제품의 `issue_7235_cell_image_fill_contains`: **4 tests run: 4 passed, 187 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

원 PR에 없던 PDF를 커밋된 1쪽 발췌 HWP에서 engine 2020으로 새로 변환했다. 로고가 보이고 가운데에 놓이지만 크기·글꼴 차이는 남는다. 64개 Studio renderer 검사는 실제 CanvasKit raster의 PDF 일치 검사가 아니다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| logo | 1 | 0 / 31.06% | 0 / 31.06% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| logo p1 | [compare](../assets/pr7259_review/native_logo_compare_001.png) · [overlay](../assets/pr7259_review/native_logo_overlay_001.png) · [review](../assets/pr7259_review/native_logo_review_001.png) | [compare](../assets/pr7259_review/wasm_logo_compare_001.png) · [overlay](../assets/pr7259_review/wasm_logo_overlay_001.png) · [review](../assets/pr7259_review/wasm_logo_review_001.png) |

## Merge 후 contributor PR comment 계획

현재 게시·merge 승인으로 간주하지 않는다. 보류 해제와 최종 head CI 및 통합 merge 이후 실제 merge SHA·통합 PR·CI URL·수정 계약·실제 검증 범위·잔여 차이를 한국어로 설명하고 기여에 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.

- logo: 위 p1의 Native/WASM review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.

URL 형식 예: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7259_review/wasm_logo_overlay_001.png`. `<merge-commit-sha>`는 게시 전 실제 값으로 치환한다. UTF-8 파일과 `gh ... --body-file`로 게시하고 API 재조회로 한국어·실제 head·모든 이미지 URL을 확인한다.

## 이슈·다음 단계

#7235 종료는 해당 backend 검증 후 판단한다. 원본 6쪽 전체 검증으로 확대해 주장하지 않는다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.

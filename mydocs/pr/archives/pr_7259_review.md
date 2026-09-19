---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7259 검토

## 최종 판정

**메인터너 보정 후 수용 가능** — 통합 로컬 필수 검증 완료. 원 PR 직접 merge가 아닌 보정 통합 head를 수용한다.

`None/Zoom`의 source crop과 destination 비율을 SVG·WebCanvas·Skia·CanvasKit에서 일치시켰다. Skia 호출부의 None→Zoom 재해석을 제거하고 두 모드가 동일한 crop 기준 contain 계산을 소비한다. SVG는 맞춘 crop viewport에서 잘라낸 영역이 여백으로 새지 않도록 한다. 독립 한컴 근거가 없는 쪽 배경 None 확대는 철회해 기존 stretch를 유지했다.

잘린 정사각형의 아래 절반(2:1)이 정사각형 칸에서 높이 절반·가로 전체를 차지해야 한다는 독립 기하와 최종 픽셀로 검증했다. 수정 전 SVG/Skia 2개 FAIL → 수정 후 crop·쪽 배경 대조 3개 PASS. 기존 실물 focused 6개 PASS, 실제 Chrome CanvasKit None/Zoom/FitToSize/Total 4개 PASS, Studio renderer 64개 및 tsc PASS. 최종 전체 게이트 전 통합 merge-ready 판정은 아니다.

실물 로고가 잘리지 않고 가운데에 맞춰 보이는 개선을 직접 확인했다. 그림 외의 글꼴·굵기·셀 원점 차이는 남으며 PDF 전체 일치라고 쓰지 않는다. Native Skia 실물 본문의 일부 사각형 글리프는 별도 잔여 차이이고, crop 색상·영역 검사는 텍스트 없이 수행한다. 이번 보정은 텍스트 폰트 선택을 변경하지 않았다. 이 증거만으로 #7235의 모든 렌더링 차이가 해결됐다고 닫지 않는다.

<details>
<summary>최초 통합 검토의 보류 사유(보정 전)</summary>

**머지 보류** — 2026-09-18 통합 검토.

[P2] #7244와 중복 SVG 변경은 합쳤으나 crop을 가진 None 경로가 backend마다 다르다. Skia renderer는 None→Zoom으로 바꾸며 Zoom은 decoded 전체 비율로 crop_src를 그린다. SVG None/Zoom은 crop을 읽지 않는다. 실제 Skia raster와 CanvasKit 화면 비교는 아직 없다.

crop 유무를 포함해 실제 backend 출력의 source/destination 사각형을 일치시키고 Native Skia·Canvas2D·CanvasKit raster 증거를 남긴다. #7244의 쪽 배경 확대 범위도 별도 처리한다.

</details>

## 최종 통합 검증

보정 제품 코드 `88f2f00da8412c769f34ef6bc3b72bc13402557b`. contributor 원 head는 아래 provenance에 별도 기록했다.
[공동 최종 검증](pr_7244_review_impl.md#최종-통합-검증)의 전체 nextest, Native Skia 3종,
fmt·Clippy 3종·workspace build·정책 검사, Studio 타입·단위·production build를 통과했다.
원 PR head CI를 재사용한 통과 주장과 구분한다. 해당 입력은 최종 Native/fresh WASM으로
다시 Visual Sweep했고 compare·standalone overlay·review와 남은 차이를 확인했다.
렌더 변경이 없는 진단·scaffold ID·래칫 자체에는 별도 시각 통과를 주장하지 않는다.

작업지시자의 PR 생성·CI 모니터링·merge·후속 처리 승인을 받았다.
[통합 PR #7264](https://github.com/edwardkim/rhwp/pull/7264)의 code candidate
`8228249fcfdf04cb7c47af9b3f5d5447a645d3b9` CI를 모두 확인했다.
[원격 CI 증적](pr_7244_review_impl.md#통합-pr-7264-code-candidate-ci)의 동일 PR 실행이며,
같은 PR의 trailing review·오늘할일 head aggregate와 mergeability를 확인한 뒤 merge한다.

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
| 최초 검토 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
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

## 최초 실행한 검증과 한계

- 통합 제품의 `issue_7235_cell_image_fill_contains`: **4 tests run: 4 passed, 187 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 최초 검토 당시에는 전체 게이트가 미실행이었다. 아래 과거 기록을 최종 상태로 해석하지 않으며, 최신 결과는 최종 통합 검증 절과 공동 원장을 따른다.

원 PR에 없던 PDF를 커밋된 1쪽 발췌 HWP에서 engine 2020으로 새로 변환했다. 로고가 보이고 가운데에 놓이지만 크기·글꼴 차이는 남는다. 64개 Studio renderer 검사는 실제 CanvasKit raster의 PDF 일치 검사가 아니다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 최초 Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| logo | 1 | 0 / 31.06% | 0 / 31.06% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| logo p1 | [compare](../assets/pr7259_review/native_logo_compare_001.png) · [overlay](../assets/pr7259_review/native_logo_overlay_001.png) · [review](../assets/pr7259_review/native_logo_review_001.png) | [compare](../assets/pr7259_review/wasm_logo_compare_001.png) · [overlay](../assets/pr7259_review/wasm_logo_overlay_001.png) · [review](../assets/pr7259_review/wasm_logo_review_001.png) |

## 메인터너 보정 검증·증적

검증 제품 Native SHA-256 `8cedbbf0ea3000486ca086c0a877a775738f93528886f15e7c09619f8ef11e7b`, fresh WASM `09b66772e961aaa807e7823df99f4c1ae32fb92e742fd9ae67ab14f8c44a4941`, JS `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.

원본 HWP/PDF는 기존 커밋 경로 그대로다. 전쪽 fidelity text/layout ledger 후 Native/fresh WASM Visual Sweep을 재실행했다. `image_none` 14쪽 중 1·9쪽, `logo` 1쪽을 골랐고 Native 3쪽·WASM 3쪽 모두 최초 capture와 PNG bytes가 동일하다. 따라서 위 기존 compare/overlay/review를 중복 복사하지 않고 재사용한다. 자동 flag는 각 입력 0이며 승인 근거 그 자체가 아니다.

임시 원장은 `/private/tmp/rhwp-planet-repair-20260918/fidelity-image-none`, `fidelity-logo`; 새 sweep은 `sweep-native-image`, `sweep-native-logo`, `sweep-wasm-image`, `sweep-wasm-logo`다. 추가 실제 backend capture는 `image-backends`, `skia-logo`, `skia-image1`, `skia-image9`, 비교 산출물은 `backend-sweep`에 있다. Native Skia는 `export-png --profile screen`, Canvas2D는 fresh WASM `renderPageToCanvasFilteredWithProfile(..., all, screen)`, CanvasKit은 같은 문서의 `getPageLayerTreeWithProfile`를 Chrome software surface에서 재생했다. 문서를 바꿀 때 renderer의 document resource cache를 초기화했다.

추가 PNG도 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md)의 `make_compares`, `make_overlay_page`, `make_review_panels`로 PDF 대비 생성했고 실제 열어 확인했다. 픽셀/ink 지표는 전체 페이지 보조값으로, 이미지 채우기의 정확도 점수로 대체하지 않는다.

| backend·쪽 | pixel match / ink match (%) | 실제 비교 증적 |
| --- | --- | --- |
| skia p1 | 83.94 / 20.29 | [compare](../assets/pr7259_review/maintainer_20260918/skia_logo_compare_001.png) · [overlay](../assets/pr7259_review/maintainer_20260918/skia_logo_overlay_001.png) · [review](../assets/pr7259_review/maintainer_20260918/skia_logo_review_001.png) |
| canvas2d p1 | 87.25 / 22.72 | [compare](../assets/pr7259_review/maintainer_20260918/canvas2d_logo_compare_001.png) · [overlay](../assets/pr7259_review/maintainer_20260918/canvas2d_logo_overlay_001.png) · [review](../assets/pr7259_review/maintainer_20260918/canvas2d_logo_review_001.png) |
| canvaskit p1 | 87.06 / 23.69 | [compare](../assets/pr7259_review/maintainer_20260918/canvaskit_logo_compare_001.png) · [overlay](../assets/pr7259_review/maintainer_20260918/canvaskit_logo_overlay_001.png) · [review](../assets/pr7259_review/maintainer_20260918/canvaskit_logo_review_001.png) |

검사 재현: `node scripts/run-rust-test.mjs issue_7235_cropped_contain -- --cargo-profile release-test --target-dir target/planet-review-20260918 --features native-skia`; Studio에서 `node e2e/run-with-vite.mjs -- node e2e/canvaskit-cropped-contain.test.mjs`. 합성 그림은 테스트 코드에서 생성·소비하므로 중복 입력 파일은 없다.

## Merge 후 contributor PR comment 계획

승인된 통합 PR의 최종 head CI와 실제 merge를 확인한 뒤 원 PR·관련 이슈에 한국어로
merge SHA·CI URL·수정 계약·실제 검증 범위·남은 차이를 기록하고 기여에 감사한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.

- `mydocs/pr/assets/pr7259_review/`: `native_logo_review_001.png`, `native_logo_overlay_001.png`, `wasm_logo_review_001.png`, `wasm_logo_overlay_001.png`를 실제 이미지로 포함한다.
- `mydocs/pr/assets/pr7259_review/maintainer_20260918/`: `skia_logo_review_001.png`, `skia_logo_overlay_001.png`, `canvas2d_logo_review_001.png`, `canvas2d_logo_overlay_001.png`, `canvaskit_logo_review_001.png`, `canvaskit_logo_overlay_001.png`를 실제 이미지로 포함한다.

이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<실제-merge-SHA>/<위-경로>`로 표시한다.
UTF-8 파일과 `--body-file`로 게시하고 한국어·실제 head·이미지 URL을 API와 HTTP로 재확인한다.

## 이슈·다음 단계

#7235 종료는 해당 backend 검증 후 판단한다. 원본 6쪽 전체 검증으로 확대해 주장하지 않는다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.

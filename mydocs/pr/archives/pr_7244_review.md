---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7244 검토

## 최종 판정

**메인터너 보정 후 수용 가능 — 통합 최종 게이트 대기.** 2026-09-18.

`None/Zoom`의 source crop과 destination 비율을 SVG·WebCanvas·Skia·CanvasKit에서 일치시켰다. Skia 호출부의 None→Zoom 재해석을 제거하고 두 모드가 동일한 crop 기준 contain 계산을 소비한다. SVG는 맞춘 crop viewport에서 잘라낸 영역이 여백으로 새지 않도록 한다. 독립 한컴 근거가 없는 쪽 배경 None 확대는 철회해 기존 stretch를 유지했다.

잘린 정사각형의 아래 절반(2:1)이 정사각형 칸에서 높이 절반·가로 전체를 차지해야 한다는 독립 기하와 최종 픽셀로 검증했다. 수정 전 SVG/Skia 2개 FAIL → 수정 후 crop·쪽 배경 대조 3개 PASS. 기존 실물 focused 6개 PASS, 실제 Chrome CanvasKit None/Zoom/FitToSize/Total 4개 PASS, Studio renderer 64개 및 tsc PASS. 최종 전체 게이트 전 통합 merge-ready 판정은 아니다.

실물 로고가 잘리지 않고 가운데에 맞춰 보이는 개선을 직접 확인했다. 그림 외의 글꼴·굵기·셀 원점 차이는 남으며 PDF 전체 일치라고 쓰지 않는다. Native Skia 실물 본문의 일부 사각형 글리프는 별도 잔여 차이이고, crop 색상·영역 검사는 텍스트 없이 수행한다. 이번 보정은 텍스트 폰트 선택을 변경하지 않았다. 이 증거만으로 #7235의 모든 렌더링 차이가 해결됐다고 닫지 않는다.

<details>
<summary>최초 통합 검토의 보류 사유(보정 전)</summary>

**머지 보류** — 2026-09-18 통합 검토.

[P2] 셀·도형 로고의 실측으로 쪽 배경 None까지 동작을 바꿨지만 쪽 배경의 독립 기준과 대조군이 없다. 또한 SVG의 새 None/Zoom 분기는 img.crop을 읽지 않는 반면 Skia는 crop_src를 적용한다. crop이 있는 입력의 backend 동등성은 입증되지 않았다.

쪽 배경의 한컴 기준을 확보하거나 변경 범위에서 분리하고, crop 유무에 따른 SVG·Canvas·Skia의 실제 칠한 영역을 같은 입력으로 검사한다.

</details>

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7244](https://github.com/edwardkim/rhwp/pull/7244) — fix: 그림 채우기 유형 15(NONE)를 종횡비 맞춤으로 그린다 (#7235) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `3945f5e1defd499f772a170c9da2073739803ec3` |
| 규모 | 7 files, +148 / -9 |
| 조회 당시 mergeability | `MERGEABLE` / `CLEAN` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `3945f5e1defd499f772a170c9da2073739803ec3` | `a7e30431c115dcc373cf20fae555384987e43aa1` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35264130349) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35264130964) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35264129472) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35264130991) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35264130357) · [CI 6](https://github.com/edwardkim/rhwp/actions/runs/35264129485)

## 범위·조판 계약 검토

관련 이슈: [#7235](https://github.com/edwardkim/rhwp/issues/7235). 그림 채우기 None을 SVG·WebCanvas·Skia 및 쪽 배경에서 contain으로 해석한다.

paint 변경이다. ImageNode.fill_mode → SVG render_image_node / WebCanvas / Skia draw_image를 추적했다. 측정 bbox는 그대로이고 칠하는 사각형만 달라진다. 구분해야 할 셀·도형·쪽 배경 계약을 하나의 표본으로 승인하지 않는다.

주요 소비 경로: [src/renderer/skia/image_conv.rs](../../../src/renderer/skia/image_conv.rs), [src/renderer/svg.rs](../../../src/renderer/svg.rs), [src/renderer/web_canvas.rs](../../../src/renderer/web_canvas.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 최초 실행한 검증과 한계

- 통합 제품의 `issue_7235_none_image_fill_fits_area`: **2 tests run: 2 passed, 188 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

1·9쪽에서 잘리던 로고와 머리띠는 보인다. 글꼴·굵기·원점 차이가 남으며 전체 PDF 일치로 판정하지 않는다. Native Skia 실제 raster/PDF와 crop·쪽 배경 조합은 이번 실행 범위 밖이다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 최초 Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| image_none | 1, 9 | 0 / 32.39% | 0 / 32.39% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| image_none p1 | [compare](../assets/pr7244_review/native_image_none_compare_001.png) · [overlay](../assets/pr7244_review/native_image_none_overlay_001.png) · [review](../assets/pr7244_review/native_image_none_review_001.png) | [compare](../assets/pr7244_review/wasm_image_none_compare_001.png) · [overlay](../assets/pr7244_review/wasm_image_none_overlay_001.png) · [review](../assets/pr7244_review/wasm_image_none_review_001.png) |
| image_none p9 | [compare](../assets/pr7244_review/native_image_none_compare_009.png) · [overlay](../assets/pr7244_review/native_image_none_overlay_009.png) · [review](../assets/pr7244_review/native_image_none_review_009.png) | [compare](../assets/pr7244_review/wasm_image_none_compare_009.png) · [overlay](../assets/pr7244_review/wasm_image_none_overlay_009.png) · [review](../assets/pr7244_review/wasm_image_none_review_009.png) |

기존 devel 제품 대조: [base image_none p1](../assets/pr7244_review/base_image_none_review_001.png).

## 메인터너 보정 검증·증적

검증 제품 Native SHA-256 `8cedbbf0ea3000486ca086c0a877a775738f93528886f15e7c09619f8ef11e7b`, fresh WASM `09b66772e961aaa807e7823df99f4c1ae32fb92e742fd9ae67ab14f8c44a4941`, JS `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.

원본 HWP/PDF는 기존 커밋 경로 그대로다. 전쪽 fidelity text/layout ledger 후 Native/fresh WASM Visual Sweep을 재실행했다. `image_none` 14쪽 중 1·9쪽, `logo` 1쪽을 골랐고 Native 3쪽·WASM 3쪽 모두 최초 capture와 PNG bytes가 동일하다. 따라서 위 기존 compare/overlay/review를 중복 복사하지 않고 재사용한다. 자동 flag는 각 입력 0이며 승인 근거 그 자체가 아니다.

임시 원장은 `/private/tmp/rhwp-planet-repair-20260918/fidelity-image-none`, `fidelity-logo`; 새 sweep은 `sweep-native-image`, `sweep-native-logo`, `sweep-wasm-image`, `sweep-wasm-logo`다. 추가 실제 backend capture는 `image-backends`, `skia-logo`, `skia-image1`, `skia-image9`, 비교 산출물은 `backend-sweep`에 있다. Native Skia는 `export-png --profile screen`, Canvas2D는 fresh WASM `renderPageToCanvasFilteredWithProfile(..., all, screen)`, CanvasKit은 같은 문서의 `getPageLayerTreeWithProfile`를 Chrome software surface에서 재생했다. 문서를 바꿀 때 renderer의 document resource cache를 초기화했다.

추가 PNG도 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md)의 `make_compares`, `make_overlay_page`, `make_review_panels`로 PDF 대비 생성했고 실제 열어 확인했다. 픽셀/ink 지표는 전체 페이지 보조값으로, 이미지 채우기의 정확도 점수로 대체하지 않는다.

| backend·쪽 | pixel match / ink match (%) | 실제 비교 증적 |
| --- | --- | --- |
| skia p1 | 85.62 / 29.25 | [compare](../assets/pr7244_review/maintainer_20260918/skia_image_none_compare_001.png) · [overlay](../assets/pr7244_review/maintainer_20260918/skia_image_none_overlay_001.png) · [review](../assets/pr7244_review/maintainer_20260918/skia_image_none_review_001.png) |
| canvas2d p1 | 88.00 / 39.26 | [compare](../assets/pr7244_review/maintainer_20260918/canvas2d_image_none_compare_001.png) · [overlay](../assets/pr7244_review/maintainer_20260918/canvas2d_image_none_overlay_001.png) · [review](../assets/pr7244_review/maintainer_20260918/canvas2d_image_none_review_001.png) |
| canvaskit p1 | 89.62 / 43.06 | [compare](../assets/pr7244_review/maintainer_20260918/canvaskit_image_none_compare_001.png) · [overlay](../assets/pr7244_review/maintainer_20260918/canvaskit_image_none_overlay_001.png) · [review](../assets/pr7244_review/maintainer_20260918/canvaskit_image_none_review_001.png) |
| skia p9 | 92.68 / 8.83 | [compare](../assets/pr7244_review/maintainer_20260918/skia_image_none_compare_009.png) · [overlay](../assets/pr7244_review/maintainer_20260918/skia_image_none_overlay_009.png) · [review](../assets/pr7244_review/maintainer_20260918/skia_image_none_review_009.png) |
| canvas2d p9 | 94.22 / 10.13 | [compare](../assets/pr7244_review/maintainer_20260918/canvas2d_image_none_compare_009.png) · [overlay](../assets/pr7244_review/maintainer_20260918/canvas2d_image_none_overlay_009.png) · [review](../assets/pr7244_review/maintainer_20260918/canvas2d_image_none_review_009.png) |
| canvaskit p9 | 94.49 / 10.59 | [compare](../assets/pr7244_review/maintainer_20260918/canvaskit_image_none_compare_009.png) · [overlay](../assets/pr7244_review/maintainer_20260918/canvaskit_image_none_overlay_009.png) · [review](../assets/pr7244_review/maintainer_20260918/canvaskit_image_none_review_009.png) |

검사 재현: `node scripts/run-rust-test.mjs issue_7235_cropped_contain -- --cargo-profile release-test --target-dir target/planet-review-20260918 --features native-skia`; Studio에서 `node e2e/run-with-vite.mjs -- node e2e/canvaskit-cropped-contain.test.mjs`. 합성 그림은 테스트 코드에서 생성·소비하므로 중복 입력 파일은 없다.

## Merge 후 contributor PR comment 계획

현재 게시·merge 승인으로 간주하지 않는다. 보류 해제와 최종 head CI 및 통합 merge 이후 실제 merge SHA·통합 PR·CI URL·수정 계약·실제 검증 범위·잔여 차이를 한국어로 설명하고 기여에 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.

- image_none: 위 p1, p9의 Native/WASM 및 메인터너 보정 backend review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.

URL 형식 예: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7244_review/wasm_image_none_overlay_001.png`. `<merge-commit-sha>`는 게시 전 실제 값으로 치환한다. UTF-8 파일과 `gh ... --body-file`로 게시하고 API 재조회로 한국어·실제 head·모든 이미지 URL을 확인한다.

## 이슈·다음 단계

#7235 로고 축이 개선됐으나 backend 검증 전 종료하지 않는다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.

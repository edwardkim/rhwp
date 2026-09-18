---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7253 검토

## 최종 판정

**머지 보류** — 2026-09-18 통합 검토.

[P1] 통합 focused 5개 중 page5_starts_at_the_stored_page_frame 실패. 첫 직계 노드는 빈 TextLine(y=100.1), 그 뒤 실제 제목 Table(y=102.0)이다. #7255의 빈 슬롯 보존과 검사 가정이 충돌한다. 제목은 올바른 5쪽에 있지만 PDF 제목 원점과도 차이가 있다.

빈 줄을 제거해 테스트를 맞추지 말고 첫 가시 내용과 슬롯 점유를 각각 검증한다. 4→5쪽 제목·본문·표의 위치를 PDF와 일치시키고 저장 reset 경계 반례를 확인한다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7253](https://github.com/edwardkim/rhwp/pull/7253) — 수정: 감싼 1칸 표의 조각 컷이 저장 쪽 프레임을 지나치지 않는다 (#6923 부분) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `af407d86707f82c570fac054a85f64a69d3df5d5` |
| 규모 | 5 files, +250 / -3 |
| 조회 당시 mergeability | `CONFLICTING` / `DIRTY` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `af407d86707f82c570fac054a85f64a69d3df5d5` | `03fec50df2087daee6da840c8a70981cd875b914` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35291738641) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35291738684) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35291738248) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35291738556) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35291738633) · [CI 6](https://github.com/edwardkim/rhwp/actions/runs/35291738497)

## 범위·조판 계약 검토

관련 이슈: [#6923](https://github.com/edwardkim/rhwp/issues/6923). HWP5 1×1 감싼 표에서 저장 page-frame 재시작을 분할 경계로 수용한다.

저장 cross-paragraph reset → hwp5_page_scale_cross_para_reset(가용 높이 70% 조건) → cell_units break → 부분 표 배치. 기존 devel shared_empty_frame 조건은 보존했다. 페이지 크기의 70% 판별은 독립적인 계약·반례가 부족하며 빈 줄의 존재와 가시 내용을 구분해야 한다.

주요 소비 경로: [src/renderer/layout/table_layout.rs](../../../src/renderer/layout/table_layout.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 실행한 검증과 한계

- 통합 제품의 `issue_6923_wrapper_table_stored_page_frame`: **5 tests run: 4 passed, 1 failed, 201 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

3~6쪽을 대조했다. 4쪽의 용지 밖 내용과 5쪽 제목의 쪽 소속은 개선됐다. 4쪽 제목 뒤 큰 공백·본문/표 원점 차이는 남는다. #7062 9·10쪽도 대조군으로 캡처했다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| wrapper | 3, 4, 5, 6 | 0 / 21.71% | 0 / 21.71% |
| tac_control | 9, 10 | 0 / 26.56% | 0 / 26.56% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| wrapper p3 | [compare](../assets/pr7253_review/native_wrapper_compare_003.png) · [overlay](../assets/pr7253_review/native_wrapper_overlay_003.png) · [review](../assets/pr7253_review/native_wrapper_review_003.png) | [compare](../assets/pr7253_review/wasm_wrapper_compare_003.png) · [overlay](../assets/pr7253_review/wasm_wrapper_overlay_003.png) · [review](../assets/pr7253_review/wasm_wrapper_review_003.png) |
| wrapper p4 | [compare](../assets/pr7253_review/native_wrapper_compare_004.png) · [overlay](../assets/pr7253_review/native_wrapper_overlay_004.png) · [review](../assets/pr7253_review/native_wrapper_review_004.png) | [compare](../assets/pr7253_review/wasm_wrapper_compare_004.png) · [overlay](../assets/pr7253_review/wasm_wrapper_overlay_004.png) · [review](../assets/pr7253_review/wasm_wrapper_review_004.png) |
| wrapper p5 | [compare](../assets/pr7253_review/native_wrapper_compare_005.png) · [overlay](../assets/pr7253_review/native_wrapper_overlay_005.png) · [review](../assets/pr7253_review/native_wrapper_review_005.png) | [compare](../assets/pr7253_review/wasm_wrapper_compare_005.png) · [overlay](../assets/pr7253_review/wasm_wrapper_overlay_005.png) · [review](../assets/pr7253_review/wasm_wrapper_review_005.png) |
| wrapper p6 | [compare](../assets/pr7253_review/native_wrapper_compare_006.png) · [overlay](../assets/pr7253_review/native_wrapper_overlay_006.png) · [review](../assets/pr7253_review/native_wrapper_review_006.png) | [compare](../assets/pr7253_review/wasm_wrapper_compare_006.png) · [overlay](../assets/pr7253_review/wasm_wrapper_overlay_006.png) · [review](../assets/pr7253_review/wasm_wrapper_review_006.png) |
| tac_control p9 | [compare](../assets/pr7253_review/native_tac_control_compare_009.png) · [overlay](../assets/pr7253_review/native_tac_control_overlay_009.png) · [review](../assets/pr7253_review/native_tac_control_review_009.png) | [compare](../assets/pr7253_review/wasm_tac_control_compare_009.png) · [overlay](../assets/pr7253_review/wasm_tac_control_overlay_009.png) · [review](../assets/pr7253_review/wasm_tac_control_review_009.png) |
| tac_control p10 | [compare](../assets/pr7253_review/native_tac_control_compare_010.png) · [overlay](../assets/pr7253_review/native_tac_control_overlay_010.png) · [review](../assets/pr7253_review/native_tac_control_review_010.png) | [compare](../assets/pr7253_review/wasm_tac_control_compare_010.png) · [overlay](../assets/pr7253_review/wasm_tac_control_overlay_010.png) · [review](../assets/pr7253_review/wasm_tac_control_review_010.png) |

기존 devel 제품 대조: [base wrapper p4](../assets/pr7253_review/base_wrapper_review_004.png) · [base wrapper p5](../assets/pr7253_review/base_wrapper_review_005.png).

## Merge 후 contributor PR comment 계획

현재 게시·merge 승인으로 간주하지 않는다. 보류 해제와 최종 head CI 및 통합 merge 이후 실제 merge SHA·통합 PR·CI URL·수정 계약·실제 검증 범위·잔여 차이를 한국어로 설명하고 기여에 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.

- wrapper: 위 p3, p4, p5, p6의 Native/WASM review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.
- tac_control: 위 p9, p10의 Native/WASM review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.

URL 형식 예: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7253_review/wasm_wrapper_overlay_003.png`. `<merge-commit-sha>`는 게시 전 실제 값으로 치환한다. UTF-8 파일과 `gh ... --body-file`로 게시하고 API 재조회로 한국어·실제 head·모든 이미지 URL을 확인한다.

## 이슈·다음 단계

#6923은 다른 누락 줄·하위 표 문제를 포함하므로 OPEN 유지. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.

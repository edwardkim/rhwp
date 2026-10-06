---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7256 검토

## 최종 판정

**메인터너 보정 후 수용 가능** — 통합 로컬 필수 검증 완료. 원 PR 직접 merge가 아닌 보정 통합 head를 수용한다.

2026-09-18 `746b7881e` 이후 보정은 저장 컷의 source unit에서 프레임 원점을 읽고,
배치에서 그 원점을 끝까지 유지한다. 첫 빈 TextLine을 지우지 않고 첫 가시 내용과
1400HU 빈 슬롯의 점유를 각각 검사한다. 이어받는 셀 좌표와 표 자신의 문단 offset을
혼동하던 제한을 제거했다. 4쪽 제목 y=445.23→532.96px, 5쪽 y≈102→129.9px로 복원했다.
PDF 841pt를 원문 841.88pt로 정규화한 제목 원점과 **0.5px 이내**다.

중첩 TAC 표는 같은 `control_line_seg_index`가 가리키는 줄의 들여쓰기와 표 앞 텍스트만
소비한다. 뒤의 공백과 앞줄 텍스트를 전체 합산하던 가로 오프셋을 바로잡았다.
PDF의 바깥 표 대비 상대 x(제목 11.11px, 조치 표 19.03px, 법조문 표 22.55px)를
595→595.28pt로 정규화하여 0.5px 이내인지 정식 회귀 검사로 확인한다.

- 세로 반례: 수정 전 7개 중 1개 실패 → 7/7 통과. 가로 반례: 8개 중 1개 실패 → **8/8 통과**.
- #6013 빈 문단 프레임 **1/1**, #6653 소유 줄 **1/1**, #7095 조각 상자 **6/6 통과**.
- 원래 source의 70% reset 호환 판별을 더 넓히지 않았다. 겹친 줄 reset과 빈 슬롯은
  위 음성 대조 및 기존 페이지/본문 경계 검사로 구분했다. 이 비율을 일반 문서 사양으로 주장하지 않는다.
- Native CLI는 focused nextest의 동일 release-test/native-skia 빌드 산출물이다.
  fresh WASM `--no-opt` 및 영향 3~6쪽/대조 9~10쪽 재캡처 완료.
- 큰 빈 띠와 법조문 표 겹침은 해소됐다. 바깥 wrapper x 약 3.8px와 하단 p4 약 12px,
  p5 약 10px의 외곽선 차이 및 글꼴 차이는 남아 있다. 이 보정은 내부 저장 프레임·소유 줄
  계약의 승인이고 전체 PDF 픽셀 일치 또는 #6923 전체 해결 판정이 아니다.
- 전체 nextest·lint·Native Skia 최종 게이트도 통과했다. 실행 결과는 최종 통합 검증 절을 따른다.

소비 경로: `cell_units`의 시작 컷 → `stored_frame_origin_for_cut` →
`preserve_linear_single_cell_vpos`/`frag_vpos_origin` → paragraph vpos snap →
중첩 표의 `stored_nested_table_line_offset_px` 및 소유 줄 prefix → 최종 Table bbox.
측정/컷은 동일 유닛을 유지하며 빈 슬롯과 clip을 삭제·완화하지 않았다.

제품 SHA-256: Native `92e4cf72c32fd78fe1a9f12c16cd882e1916b4adc22ef75f1c4e17c8d533ed68`, WASM `7a0da590a0573ebeae9b2acaa97efa45b30d7d44ba0b055cc6aa96bfcb3c1cda`.
검증 명령은 [공동 기록](pr_7244_review_impl.md)의 focused/Sweep 형식이며
모듈 `issue_6923_wrapper_table_stored_page_frame`, target `target/planet-review-20260918`,
WASM package `/private/tmp/rhwp-planet-repair-20260918/wasm-wrapper-x`를 사용했다.

## 보정 후 Visual Sweep

| 입력·쪽 | Native SVG/Chrome | fresh WASM SVG/Chrome |
| --- | --- | --- |
| wrapper p3 | [compare](../assets/pr7253_review/maintainer_20260918/native_wrapper_compare_003.png) · [overlay](../assets/pr7253_review/maintainer_20260918/native_wrapper_overlay_003.png) · [review](../assets/pr7253_review/maintainer_20260918/native_wrapper_review_003.png) | [compare](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_compare_003.png) · [overlay](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_overlay_003.png) · [review](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_review_003.png) |
| wrapper p4 | [compare](../assets/pr7253_review/maintainer_20260918/native_wrapper_compare_004.png) · [overlay](../assets/pr7253_review/maintainer_20260918/native_wrapper_overlay_004.png) · [review](../assets/pr7253_review/maintainer_20260918/native_wrapper_review_004.png) | [compare](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_compare_004.png) · [overlay](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_overlay_004.png) · [review](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_review_004.png) |
| wrapper p5 | [compare](../assets/pr7253_review/maintainer_20260918/native_wrapper_compare_005.png) · [overlay](../assets/pr7253_review/maintainer_20260918/native_wrapper_overlay_005.png) · [review](../assets/pr7253_review/maintainer_20260918/native_wrapper_review_005.png) | [compare](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_compare_005.png) · [overlay](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_overlay_005.png) · [review](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_review_005.png) |
| wrapper p6 | [compare](../assets/pr7253_review/maintainer_20260918/native_wrapper_compare_006.png) · [overlay](../assets/pr7253_review/maintainer_20260918/native_wrapper_overlay_006.png) · [review](../assets/pr7253_review/maintainer_20260918/native_wrapper_review_006.png) | [compare](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_compare_006.png) · [overlay](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_overlay_006.png) · [review](../assets/pr7253_review/maintainer_20260918/wasm_wrapper_review_006.png) |
| tac_control p9 | [compare](../assets/pr7253_review/maintainer_20260918/native_tac_control_compare_009.png) · [overlay](../assets/pr7253_review/maintainer_20260918/native_tac_control_overlay_009.png) · [review](../assets/pr7253_review/maintainer_20260918/native_tac_control_review_009.png) | [compare](../assets/pr7253_review/maintainer_20260918/wasm_tac_control_compare_009.png) · [overlay](../assets/pr7253_review/maintainer_20260918/wasm_tac_control_overlay_009.png) · [review](../assets/pr7253_review/maintainer_20260918/wasm_tac_control_review_009.png) |
| tac_control p10 | [compare](../assets/pr7253_review/maintainer_20260918/native_tac_control_compare_010.png) · [overlay](../assets/pr7253_review/maintainer_20260918/native_tac_control_overlay_010.png) · [review](../assets/pr7253_review/maintainer_20260918/native_tac_control_review_010.png) | [compare](../assets/pr7253_review/maintainer_20260918/wasm_tac_control_compare_010.png) · [overlay](../assets/pr7253_review/maintainer_20260918/wasm_tac_control_overlay_010.png) · [review](../assets/pr7253_review/maintainer_20260918/wasm_tac_control_review_010.png) |

아래 Metadata 이후 최초 검토 결과는 **수정 전 기록**이다. merge 후 comment에는
이 절의 최신 review와 **각 standalone overlay**를 실제 이미지로 게시한다.
#7256은 같은 파일을 링크하여 증적을 중복 저장하지 않는다.

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
| 원 PR | [#7256](https://github.com/edwardkim/rhwp/pull/7256) — 수정: 칸 안 TAC 중첩 표를 저장이 말하는 자기 줄에 앉힌다 (#6923 둘째 축) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `ee3a903d855c1fbc0eea90d8fda2331ea56a4ca8` |
| 규모 | 6 files, +318 / -3 |
| 조회 당시 mergeability | `CONFLICTING` / `DIRTY` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 최초 검토 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `ee3a903d855c1fbc0eea90d8fda2331ea56a4ca8` | `3fac1a15932e1b0d40d478112471c89681719724` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35296498907) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35296498971) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35296498840) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35296498895) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35296498870) · [CI 6](https://github.com/edwardkim/rhwp/actions/runs/35296498823)

## 범위·조판 계약 검토

관련 이슈: [#6923](https://github.com/edwardkim/rhwp/issues/6923). 저장 wrapper 안 TAC 중첩 표가 자기 소유 LineSeg의 위치를 사용한다. #7253을 포함하는 스택이다.

표 host의 실제 줄 범위 → 두 저장 LineSeg의 delta → 중첩 표 유닛/배치. af407d867은 #7253에서 한 번만 적용하고 ee3a903d8만 추가 적용했다. synthetic LineSeg와 실제 저장 줄의 분기를 구분한다.

주요 소비 경로: [src/renderer/layout/table_layout.rs](../../../src/renderer/layout/table_layout.rs), [src/renderer/layout/table_partial.rs](../../../src/renderer/layout/table_partial.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 최초 검토 검증과 한계

- 통합 제품의 `issue_6923_wrapper_table_stored_page_frame`: **5 tests run: 4 passed, 1 failed, 201 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 최초 검토 당시에는 전체 게이트가 미실행이었다. 아래 과거 기록을 최종 상태로 해석하지 않으며, 최신 결과는 최종 통합 검증 절과 공동 원장을 따른다.

4쪽 법조 제목과 표의 겹침 제거는 Native에서 확인했다. PDF의 제목·본문·표 전체 위치와 큰 빈 영역은 아직 다르다. 3~6쪽 증적은 #7253과 같은 파일을 재사용한다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 최초 검토 Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| wrapper | 3, 4, 5, 6 | 0 / 21.71% | 0 / 21.71% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| wrapper p3 | [compare](../assets/pr7253_review/native_wrapper_compare_003.png) · [overlay](../assets/pr7253_review/native_wrapper_overlay_003.png) · [review](../assets/pr7253_review/native_wrapper_review_003.png) | [compare](../assets/pr7253_review/wasm_wrapper_compare_003.png) · [overlay](../assets/pr7253_review/wasm_wrapper_overlay_003.png) · [review](../assets/pr7253_review/wasm_wrapper_review_003.png) |
| wrapper p4 | [compare](../assets/pr7253_review/native_wrapper_compare_004.png) · [overlay](../assets/pr7253_review/native_wrapper_overlay_004.png) · [review](../assets/pr7253_review/native_wrapper_review_004.png) | [compare](../assets/pr7253_review/wasm_wrapper_compare_004.png) · [overlay](../assets/pr7253_review/wasm_wrapper_overlay_004.png) · [review](../assets/pr7253_review/wasm_wrapper_review_004.png) |
| wrapper p5 | [compare](../assets/pr7253_review/native_wrapper_compare_005.png) · [overlay](../assets/pr7253_review/native_wrapper_overlay_005.png) · [review](../assets/pr7253_review/native_wrapper_review_005.png) | [compare](../assets/pr7253_review/wasm_wrapper_compare_005.png) · [overlay](../assets/pr7253_review/wasm_wrapper_overlay_005.png) · [review](../assets/pr7253_review/wasm_wrapper_review_005.png) |
| wrapper p6 | [compare](../assets/pr7253_review/native_wrapper_compare_006.png) · [overlay](../assets/pr7253_review/native_wrapper_overlay_006.png) · [review](../assets/pr7253_review/native_wrapper_review_006.png) | [compare](../assets/pr7253_review/wasm_wrapper_compare_006.png) · [overlay](../assets/pr7253_review/wasm_wrapper_overlay_006.png) · [review](../assets/pr7253_review/wasm_wrapper_review_006.png) |

기존 devel 제품 대조: [base wrapper p4](../assets/pr7253_review/base_wrapper_review_004.png) · [base wrapper p5](../assets/pr7253_review/base_wrapper_review_005.png).

## Merge 후 contributor PR comment 계획

승인된 통합 PR의 최종 head CI와 실제 merge를 확인한 뒤 원 PR·관련 이슈에 한국어로
merge SHA·CI URL·수정 계약·실제 검증 범위·남은 차이를 기록하고 기여에 감사한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.

- `mydocs/pr/assets/pr7253_review/maintainer_20260918/`: `native_wrapper_review_003.png`, `native_wrapper_overlay_003.png`, `wasm_wrapper_review_003.png`, `wasm_wrapper_overlay_003.png`, `native_wrapper_review_004.png`, `native_wrapper_overlay_004.png`, `wasm_wrapper_review_004.png`, `wasm_wrapper_overlay_004.png`, `native_wrapper_review_005.png`, `native_wrapper_overlay_005.png`, `wasm_wrapper_review_005.png`, `wasm_wrapper_overlay_005.png`, `native_wrapper_review_006.png`, `native_wrapper_overlay_006.png`, `wasm_wrapper_review_006.png`, `wasm_wrapper_overlay_006.png`를 실제 이미지로 포함한다.

이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<실제-merge-SHA>/<위-경로>`로 표시한다.
UTF-8 파일과 `--body-file`로 게시하고 한국어·실제 head·이미지 URL을 API와 HTTP로 재확인한다.

## 이슈·다음 단계

#6923 OPEN 유지. source 스택의 중복 commit을 두 번 반영하지 않는다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.

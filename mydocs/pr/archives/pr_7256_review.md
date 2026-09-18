---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7256 검토

## 최종 판정

**머지 보류** — 2026-09-18 통합 검토.

[P1] 의존 #7253의 통합 focused 실패가 남는다. 4쪽 적용법조와 중첩 표의 겹침은 줄었지만, 위 제목과 뒤 표 사이의 큰 공백 및 PDF와의 절대 원점 차이가 남아 있다. 상대 간격만의 통과로 전체 소유·점유 계약을 승인하지 않는다.

#7253의 빈 슬롯/분할 계약 충돌 및 4쪽 공백·원점 차이를 해결하고 실제 전체 점유와 다음 내용을 함께 대조한다.

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
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
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

## 실행한 검증과 한계

- 통합 제품의 `issue_6923_wrapper_table_stored_page_frame`: **5 tests run: 4 passed, 1 failed, 201 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

4쪽 법조 제목과 표의 겹침 제거는 Native에서 확인했다. PDF의 제목·본문·표 전체 위치와 큰 빈 영역은 아직 다르다. 3~6쪽 증적은 #7253과 같은 파일을 재사용한다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## Visual Sweep 증적

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

현재 게시·merge 승인으로 간주하지 않는다. 보류 해제와 최종 head CI 및 통합 merge 이후 실제 merge SHA·통합 PR·CI URL·수정 계약·실제 검증 범위·잔여 차이를 한국어로 설명하고 기여에 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.

- wrapper: 위 p3, p4, p5, p6의 Native/WASM review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.

URL 형식 예: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7253_review/wasm_wrapper_overlay_003.png`. `<merge-commit-sha>`는 게시 전 실제 값으로 치환한다. UTF-8 파일과 `gh ... --body-file`로 게시하고 API 재조회로 한국어·실제 head·모든 이미지 URL을 확인한다.

## 이슈·다음 단계

#6923 OPEN 유지. source 스택의 중복 commit을 두 번 반영하지 않는다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.

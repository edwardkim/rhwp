---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7228_review.md
last_verified: 2026-09-17
---

# PR #7228 검토

## 최종 판정

**머지 보류** — 분할 컷에 적용한 trailing trim이 예약 높이와 실제 상자 높이에 끝까지 전달되지 않는다.

[원 PR #7228](https://github.com/edwardkim/rhwp/pull/7228): 수정: 저장 사다리 되감김 경계의 조각 컷이 마지막 줄 줄간격을 요구하지 않는다 (#7203 컷 갈래)
관련 [이슈 #7203](https://github.com/edwardkim/rhwp/issues/7203).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `a954fc95b769ae30d3aa631a1ba153d7ba28d2d6`, base `devel`. 검토자는 `jangster77`이다.
- source `a954fc95b769ae30d3aa631a1ba153d7ba28d2d6` → applied `ca404c82746dc70dd67c387fc74bdb7ec42aec87`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35186364674/job/105089178978), [CI](https://github.com/edwardkim/rhwp/actions/runs/35186364650/job/105089179144), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35186364408/job/105089178482), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35186364651/job/105089178966), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35186364755/job/105089178955), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35186364439/job/105089177962), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35187420390). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7228_review_impl.md).

## 코드 경로와 독립 실행 증거

table_layout.rs:12584 native_intra_para_saved_reset_trailing_trim → :13805 예산 실패 분기의 컷 h에서만 trim → unit 소유 컷은 앞당김. row_block_content_height 및 typeset의 누적 예약 높이는 같은 trim 결과를 소비하지 않는다.

통합 hwpctl p52 pi1274 (y,h)=(941.7,62.6), p53 continuation h=314.7, 뒤 pi1296 bottom=997.9로 본문 안에 들어왔다. 이동한 코드 줄과 후속 내용 보존은 5개 focused 검사로 확인. 그러나 p52 상자 bottom=1004.3은 PDF 바닥 약1000.51(stroke 환산을 달리한 원 PR 값1001.48)과 여전히 2.8–3.8px 차이다. 기준을 바꿔 이 잔차를 숨기지 않는다.

관련 실행: **issue_7203_stored_rewind_fragment_trailing_trim 5개 및 issue_6368 1개**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

코드 검토와 실행 좌표로 확인한 공통 높이 계약 미충족이다. declared_box와 ±0.5px 일치할 때만 분기를 적용하는 조건도 물리 쪽 경계를 일반적으로 보장하는 독립 증거가 더 필요하다. 5개 테스트의 통과는 컷·문자 보존을 검사하며 상자와 예약 높이의 동일성까지 검사하지 않는다.

trim이 포함된 조각 geometry/예약 높이를 공통 결과로 반환하고 cut·typeset·paint에서 재계산하지 않게 한다. 양쪽 컷, 종료, 같은 declared height지만 로컬 reset인 반례 및 실제 PDF 상자 높이를 정식 검사한다. #7221과 함께 재검토하고 #7203은 열린 상태로 유지한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 미검증 | 위 코드 경로·입력/대조군 참조. declared height에 의한 물리 경계 판별의 독립 반례는 #7228에서 미검증. |
| 측정·배치 일관성 | 미충족 | table_layout.rs:12584 native_intra_para_saved_reset_trailing_trim → :13805 예산 실패 분기의 컷 h에서만 trim → unit 소유 컷은 앞당김. row_block_content_height 및 typeset의 누적 예약 높이는 같은 trim 결과를 소비하지 않는다. |
| 분할·이어받기 계약 | 미충족 | 컷/예약/paint의 동일성 또는 새 페이지 경계 회귀가 해소되지 않았다. |
| 줄 소속과 점유 높이 | 미충족 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

## Visual Sweep 입력·직접 확인 범위

DPI 96, fresh WASM 및 Native. overlay 색상은 rhwp만 있는 차이 빨강 / PDF만 있는 차이 파랑 / 양쪽 내용의 색상 차이 주황 / 허용값 이내 회색이다. 자동 일치율은 보조값이며 승인 기준 자체가 아니다.

| 입력 | 기준 PDF | 직접 비교한 쪽 |
| --- | --- | --- |
| [samples/hwpctl_API_v2.4.hwp](../../../samples/hwpctl_API_v2.4.hwp) | [pdf/hwpctl_API_v2.4-hwp-2020.pdf](../../../pdf/hwpctl_API_v2.4-hwp-2020.pdf) | 12,26,52,53,57 |

입력·PDF는 이미 Git에 존재하는 경로를 재사용했고 새로 추가한 3입력/3PDF는 `6600d48b2`에서 추적한다.

### 보존한 비교 PNG

- [hwpctl_wasm_compare_012.png](../assets/pr7228_review/hwpctl_wasm_compare_012.png)
- [hwpctl_wasm_compare_026.png](../assets/pr7228_review/hwpctl_wasm_compare_026.png)
- [hwpctl_wasm_compare_052.png](../assets/pr7228_review/hwpctl_wasm_compare_052.png)
- [hwpctl_wasm_compare_053.png](../assets/pr7228_review/hwpctl_wasm_compare_053.png)
- [hwpctl_wasm_compare_057.png](../assets/pr7228_review/hwpctl_wasm_compare_057.png)
- [hwpctl_wasm_overlay_012.png](../assets/pr7228_review/hwpctl_wasm_overlay_012.png)
- [hwpctl_wasm_overlay_026.png](../assets/pr7228_review/hwpctl_wasm_overlay_026.png)
- [hwpctl_wasm_overlay_052.png](../assets/pr7228_review/hwpctl_wasm_overlay_052.png)
- [hwpctl_wasm_overlay_053.png](../assets/pr7228_review/hwpctl_wasm_overlay_053.png)
- [hwpctl_wasm_overlay_057.png](../assets/pr7228_review/hwpctl_wasm_overlay_057.png)
- [hwpctl_wasm_review_012.png](../assets/pr7228_review/hwpctl_wasm_review_012.png)
- [hwpctl_wasm_review_026.png](../assets/pr7228_review/hwpctl_wasm_review_026.png)
- [hwpctl_wasm_review_052.png](../assets/pr7228_review/hwpctl_wasm_review_052.png)
- [hwpctl_wasm_review_053.png](../assets/pr7228_review/hwpctl_wasm_review_053.png)
- [hwpctl_wasm_review_057.png](../assets/pr7228_review/hwpctl_wasm_review_057.png)
- [hwpctl_base_review_052.png](../assets/pr7228_review/hwpctl_base_review_052.png)
- [hwpctl_base_overlay_052.png](../assets/pr7228_review/hwpctl_base_overlay_052.png)
- [hwpctl_base_review_053.png](../assets/pr7228_review/hwpctl_base_review_053.png)
- [hwpctl_base_overlay_053.png](../assets/pr7228_review/hwpctl_base_overlay_053.png)
- [hwpctl_native_review_052.png](../assets/pr7228_review/hwpctl_native_review_052.png)
- [hwpctl_native_overlay_052.png](../assets/pr7228_review/hwpctl_native_overlay_052.png)
- [hwpctl_native_review_057.png](../assets/pr7228_review/hwpctl_native_review_057.png)
- [hwpctl_native_overlay_057.png](../assets/pr7228_review/hwpctl_native_overlay_057.png)

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.

---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7211_review.md
last_verified: 2026-09-17
---

# PR #7211 검토

## 최종 판정

**승인** — 가이드가 실제 셀 경계 구간만 덮도록 하는 변경을 수용한다.

[원 PR #7211](https://github.com/edwardkim/rhwp/pull/7211): 수정: 표 크기 조절 가이드선을 경계가 실제로 있는 구간에만 긋는다 (#7191)
관련 [이슈 #7191](https://github.com/edwardkim/rhwp/issues/7191).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `55ad9f1b79130d075c6710bac2c2014e5443f239`, base `devel`. 검토자는 `jangster77`이다.
- source `55ad9f1b79130d075c6710bac2c2014e5443f239` → applied `08e8b617208bdfa90c5754c76c5f31f161fcc62d`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35156271687/job/104996337270), [CI](https://github.com/edwardkim/rhwp/actions/runs/35156271664/job/104996337176), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35156271256/job/104996335710), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35156271519/job/104996336495), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35156271576/job/104996336724), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35156271275/job/104996334847), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35157027641). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7211_review_impl.md).

## 코드 경로와 독립 실행 증거

CellBbox의 행/열 span → computeBorderSpans → TableResizeRenderer의 구간별 marker. 전체 표 bbox는 가이드 길이의 대용으로 쓰지 않는다.

실제 3147199에서 x=374 경계는 y=645.1..695.5, 길이 50.4px다. 전체 표 높이 827.9px에 비해 DOM marker도 정확히 50.4px이며 cursor=col-resize였다. 한컴 PDF에도 해당 짧은 경계만 존재한다.

관련 실행: **Studio table-border-lines 구간 검사 및 3147199 실제 hover**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

폼의 글꼴·굵기·텍스트 배치에 남은 PDF 차이는 기존 문서 렌더링 문제다. 가이드 길이와 표 자체의 PDF 일치를 혼동하지 않는다.

최종 통합 CI와 다른 보류 사유 해소 후 merge한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일 ID 분기 없이 문서/셀/줄 속성을 사용한다. 적용 범위와 비적용 대조군을 위에 구분했다. |
| 측정·배치 일관성 | 비해당 | 문서 레이아웃을 바꾸지 않는 이벤트/가이드 변경이다. |
| 분할·이어받기 계약 | 비해당 | 이 PR은 분할 컷·continuation 소유 규칙을 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 비해당 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

## Visual Sweep 입력·직접 확인 범위

DPI 96, fresh WASM 및 Native. overlay 색상은 rhwp만 있는 차이 빨강 / PDF만 있는 차이 파랑 / 양쪽 내용의 색상 차이 주황 / 허용값 이내 회색이다. 자동 일치율은 보조값이며 승인 기준 자체가 아니다.

| 입력 | 기준 PDF | 직접 비교한 쪽 |
| --- | --- | --- |
| [tests/fixtures/planet_review_20260917/3147199_[별지 62] 소규모 주류 제조 면허증(주세사무처리규정).hwpx](../../../tests/fixtures/planet_review_20260917/3147199_%5B%EB%B3%84%EC%A7%80%2062%5D%20%EC%86%8C%EA%B7%9C%EB%AA%A8%20%EC%A3%BC%EB%A5%98%20%EC%A0%9C%EC%A1%B0%20%EB%A9%B4%ED%97%88%EC%A6%9D%28%EC%A3%BC%EC%84%B8%EC%82%AC%EB%AC%B4%EC%B2%98%EB%A6%AC%EA%B7%9C%EC%A0%95%29.hwpx) | [pdf/planet-review-20260917/3147199-2020.pdf](../../../pdf/planet-review-20260917/3147199-2020.pdf) | 1 |

입력·PDF는 이미 Git에 존재하는 경로를 재사용했고 새로 추가한 3입력/3PDF는 `6600d48b2`에서 추적한다.

### 보존한 비교 PNG

- [partial_border_wasm_compare_001.png](../assets/pr7211_review/partial_border_wasm_compare_001.png)
- [partial_border_wasm_overlay_001.png](../assets/pr7211_review/partial_border_wasm_overlay_001.png)
- [partial_border_wasm_review_001.png](../assets/pr7211_review/partial_border_wasm_review_001.png)
- [studio-partial-border.png](../assets/pr7211_review/studio-partial-border.png)

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.

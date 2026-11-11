검토 및 머지 완료했습니다. 기여해 주셔서 감사합니다.

- 원 PR: [#8945](https://github.com/edwardkim/rhwp/pull/8945), [merge `4ab860651b`](https://github.com/edwardkim/rhwp/commit/4ab860651b857bfe6d28b22b4565b2855bebb042).
- 최신 exact-head CI 38개와 개별 병합 시뮬레이션을 확인했습니다. 최소 글꼴 집중 회귀 **14 PASS**, 필수 HWPX Native 편람 **383쪽 PASS**입니다. 성공한 전체 CI는 재사용했습니다.
- 독립 메인테이너 검증: 새 b/c/d 6쪽과 기존 RowBreak·None 대조 2쪽의 **Native/fresh 최적화 WASM 각 8쪽 raw 100.00%**입니다. 표·뒤 본문의 쪽 소속과 분할·내용 보존을 독립 Print와 직접 대조했습니다.
- 보호 문서의 현재 devel/후보 PNG **3,542쌍은 모두 동일**합니다. 과거 보호값의 기존 차이 911행은 메인테이너의 이번 PR 한정 수용 결정으로 분리했고, 기준 자체는 변경하지 않았습니다.
- 문서 비교: [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment).
- [독립 검토 기록](https://github.com/edwardkim/rhwp/blob/{RECORD_SHA}/mydocs/pr/archives/pr_8945_review.md) · [전쪽 TSV·검증 원장](https://github.com/edwardkim/rhwp/blob/{RECORD_SHA}/mydocs/pr/assets/pr8945/README.md).

`raw`는 색상 경계 보정 전 2px 내용 실루엣 일치율이며, 아래 RGB `pixel match`와 `visual_accuracy_proxy_percent`는 별도의 진단 보조값입니다. 글꼴·획·서브픽셀 차이에 민감하며 사람의 판정 정확도나 문서 전체의 완전 동일을 뜻하지 않습니다. 이 보조값으로 직접 판독을 대신하지 않았습니다.

| 문서 | 경로 | 쪽 | raw 최저 | pixel match 최저 | visual_accuracy_proxy 최저 |
|---|---|---|---:|---:|---:|
| b-rowbreak-bottom-margin-overflow | native | 1–2 | 100.00% | 98.93379% | 19.26440% |
| c-rowbreak-table-moves-next-page | native | 1–2 | 100.00% | 98.93659% | 20.60440% |
| d-rowbreak-splits-at-row-boundary | native | 1–2 | 100.00% | 96.22189% | 13.58506% |
| a-rowbreak-float | native | 1–1 | 100.00% | 99.68037% | 42.69053% |
| control-float | native | 1–1 | 100.00% | 99.68037% | 42.69053% |
| b-rowbreak-bottom-margin-overflow | wasm | 1–2 | 100.00% | 98.86369% | 18.20980% |
| c-rowbreak-table-moves-next-page | wasm | 1–2 | 100.00% | 98.84586% | 18.72895% |
| d-rowbreak-splits-at-row-boundary | wasm | 1–2 | 100.00% | 96.12521% | 11.81050% |
| a-rowbreak-float | wasm | 1–1 | 100.00% | 99.67544% | 42.09684% |
| control-float | wasm | 1–1 | 100.00% | 99.67544% | 42.09684% |

아래는 **메인테이너가 독립 실행한 실제 출력**의 페이지별 review·overlay입니다. 각 입력·기준 PDF·출력 경로는 원장과 대응합니다.

<details>
<summary>b-rowbreak-bottom-margin-overflow — Native·WASM 전쪽 review / overlay</summary>

![b-rowbreak-bottom-margin-overflow native 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/b-rowbreak-bottom-margin-overflow/review/review_001.png)

![b-rowbreak-bottom-margin-overflow native 2쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/b-rowbreak-bottom-margin-overflow/review/review_002.png)

![b-rowbreak-bottom-margin-overflow native 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/b-rowbreak-bottom-margin-overflow/overlay/overlay_001.png)

![b-rowbreak-bottom-margin-overflow native 2쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/b-rowbreak-bottom-margin-overflow/overlay/overlay_002.png)

![b-rowbreak-bottom-margin-overflow wasm 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/b-rowbreak-bottom-margin-overflow/review/review_001.png)

![b-rowbreak-bottom-margin-overflow wasm 2쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/b-rowbreak-bottom-margin-overflow/review/review_002.png)

![b-rowbreak-bottom-margin-overflow wasm 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/b-rowbreak-bottom-margin-overflow/overlay/overlay_001.png)

![b-rowbreak-bottom-margin-overflow wasm 2쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/b-rowbreak-bottom-margin-overflow/overlay/overlay_002.png)

</details>

<details>
<summary>c-rowbreak-table-moves-next-page — Native·WASM 전쪽 review / overlay</summary>

![c-rowbreak-table-moves-next-page native 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/c-rowbreak-table-moves-next-page/review/review_001.png)

![c-rowbreak-table-moves-next-page native 2쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/c-rowbreak-table-moves-next-page/review/review_002.png)

![c-rowbreak-table-moves-next-page native 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/c-rowbreak-table-moves-next-page/overlay/overlay_001.png)

![c-rowbreak-table-moves-next-page native 2쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/c-rowbreak-table-moves-next-page/overlay/overlay_002.png)

![c-rowbreak-table-moves-next-page wasm 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/c-rowbreak-table-moves-next-page/review/review_001.png)

![c-rowbreak-table-moves-next-page wasm 2쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/c-rowbreak-table-moves-next-page/review/review_002.png)

![c-rowbreak-table-moves-next-page wasm 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/c-rowbreak-table-moves-next-page/overlay/overlay_001.png)

![c-rowbreak-table-moves-next-page wasm 2쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/c-rowbreak-table-moves-next-page/overlay/overlay_002.png)

</details>

<details>
<summary>d-rowbreak-splits-at-row-boundary — Native·WASM 전쪽 review / overlay</summary>

![d-rowbreak-splits-at-row-boundary native 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/d-rowbreak-splits-at-row-boundary/review/review_001.png)

![d-rowbreak-splits-at-row-boundary native 2쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/d-rowbreak-splits-at-row-boundary/review/review_002.png)

![d-rowbreak-splits-at-row-boundary native 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/d-rowbreak-splits-at-row-boundary/overlay/overlay_001.png)

![d-rowbreak-splits-at-row-boundary native 2쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/d-rowbreak-splits-at-row-boundary/overlay/overlay_002.png)

![d-rowbreak-splits-at-row-boundary wasm 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/d-rowbreak-splits-at-row-boundary/review/review_001.png)

![d-rowbreak-splits-at-row-boundary wasm 2쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/d-rowbreak-splits-at-row-boundary/review/review_002.png)

![d-rowbreak-splits-at-row-boundary wasm 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/d-rowbreak-splits-at-row-boundary/overlay/overlay_001.png)

![d-rowbreak-splits-at-row-boundary wasm 2쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/d-rowbreak-splits-at-row-boundary/overlay/overlay_002.png)

</details>

<details>
<summary>a-rowbreak-float — Native·WASM 전쪽 review / overlay</summary>

![a-rowbreak-float native 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/a-rowbreak-float/review/review_001.png)

![a-rowbreak-float native 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/a-rowbreak-float/overlay/overlay_001.png)

![a-rowbreak-float wasm 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/a-rowbreak-float/review/review_001.png)

![a-rowbreak-float wasm 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/a-rowbreak-float/overlay/overlay_001.png)

</details>

<details>
<summary>control-float — Native·WASM 전쪽 review / overlay</summary>

![control-float native 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/control-float/review/review_001.png)

![control-float native 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/native/control-float/overlay/overlay_001.png)

![control-float wasm 1쪽 review](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/control-float/review/review_001.png)

![control-float wasm 1쪽 overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{RECORD_SHA}/mydocs/pr/assets/pr8945/focus/complete-visual/wasm/control-float/overlay/overlay_001.png)

</details>

[#7620](https://github.com/edwardkim/rhwp/issues/7620)의 **바닥 예산 부분 개선**으로 반영했습니다. footer·CellBreak 등 남은 범위가 있어 이슈는 OPEN을 유지합니다.

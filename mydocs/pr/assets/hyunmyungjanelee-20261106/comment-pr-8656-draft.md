<!-- 로컬 초안. 원격 승인·실제 merge·운영 기록·devel sync 뒤 integration_pr/merge_sha를 확정해 게시한다. -->
기여해 주셔서 감사합니다. [#8656](https://github.com/edwardkim/rhwp/pull/8656)의 원 기여 commit을 보존하여 [통합 PR #{integration_pr}](https://github.com/edwardkim/rhwp/pull/{integration_pr})의 merge commit `{merge_sha}`로 devel에 반영했습니다.

soft-edge 반경 다음의 원본 크기·선택적 alpha를 해석하는 원 기여를 수용했습니다. 실제 Desktop Print와 비교하고 21/22바이트 합성 대조군·HWP/HWPX 저장을 확인했습니다. blur와 다른 그림 효과 전체가 구현됐다는 의미는 아닙니다.

같은 입력의 독립 한컴 Desktop Print와 전체 1쪽을 비교해 Native 최저 **99.91617%**, 최적화 fresh WASM 최저 **99.91621%**를 확인했습니다. 전체 회귀·Native 공식 검사·lint·새 sample 보안 및 최종 head CI의 실제 결과는 [검토 기록](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/archives/pr_8656_review.md)에 남겼습니다. 수치는 원 재현물의 전쪽 최저이며 추가 경계와 남은 차이는 별도로 구분했습니다.

문서 비교는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 따랐습니다. 아래 raw 실루엣은 96dpi·2px 이웃 기준의 내용 일치율 보조값이며 사람의 판정 정확도가 아닙니다. 독립 Print와 같은 영역을 직접 확대 판독했습니다.

입력 `picture_soft_edge`: [Native 전쪽 TSV](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/assets/hyunmyungjanelee-20261106/native-picture_soft_edge.tsv), [fresh WASM 전쪽 TSV](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/assets/hyunmyungjanelee-20261106/wasm-picture_soft_edge.tsv)

native p1 raw 99.91617%

![picture_soft_edge p1 native review](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/native/picture_soft_edge/review_001.png)

![picture_soft_edge p1 native overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/native/picture_soft_edge/overlay_001.png)

wasm p1 raw 99.91621%

![picture_soft_edge p1 wasm review](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/wasm/picture_soft_edge/review_001.png)

![picture_soft_edge p1 wasm overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/wasm/picture_soft_edge/overlay_001.png)

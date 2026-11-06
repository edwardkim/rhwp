<!-- 로컬 초안. 원격 승인·실제 merge·운영 기록·devel sync 뒤 integration_pr/merge_sha를 확정해 게시한다. -->
기여해 주셔서 감사합니다. [#8608](https://github.com/edwardkim/rhwp/pull/8608)의 원 기여 commit을 보존하여 [통합 PR #{integration_pr}](https://github.com/edwardkim/rhwp/pull/{integration_pr})의 merge commit `{merge_sha}`로 devel에 반영했습니다.

모순 열 경계를 측정·줄 나눔·실제 칸 배치가 함께 소비하도록 한 원 기여를 보존했습니다. 메인터너는 계산 중 할당을 공통 Storage 요청 예산에 연결하고 행×셀 반복을 제거했습니다. 낮은 예산 거절·새 요청 복구와 음성 대조도 확인했습니다.

같은 입력의 독립 한컴 Desktop Print와 전체 2개 입력 각 1쪽을 비교해 Native 최저 **99.99030%**, 최적화 fresh WASM 최저 **100.00000%**를 확인했습니다. 전체 회귀·Native 공식 검사·lint·새 sample 보안 및 최종 head CI의 실제 결과는 [검토 기록](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/archives/pr_8608_review.md)에 남겼습니다. 수치는 원 재현물의 전쪽 최저이며 추가 경계와 남은 차이는 별도로 구분했습니다.

문서 비교는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 따랐습니다. 아래 raw 실루엣은 96dpi·2px 이웃 기준의 내용 일치율 보조값이며 사람의 판정 정확도가 아닙니다. 독립 Print와 같은 영역을 직접 확대 판독했습니다.

입력 `short_row_grid_r21`: [Native 전쪽 TSV](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/assets/hyunmyungjanelee-20261106/native-short_row_grid_r21.tsv), [fresh WASM 전쪽 TSV](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/assets/hyunmyungjanelee-20261106/wasm-short_row_grid_r21.tsv)

native p1 raw 99.99030%

![short_row_grid_r21 p1 native review](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/native/short_row_grid_r21/review_001.png)

![short_row_grid_r21 p1 native overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/native/short_row_grid_r21/overlay_001.png)

wasm p1 raw 100.00000%

![short_row_grid_r21 p1 wasm review](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/wasm/short_row_grid_r21/review_001.png)

![short_row_grid_r21 p1 wasm overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/wasm/short_row_grid_r21/overlay_001.png)

입력 `short_row_grid_r4`: [Native 전쪽 TSV](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/assets/hyunmyungjanelee-20261106/native-short_row_grid_r4.tsv), [fresh WASM 전쪽 TSV](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/assets/hyunmyungjanelee-20261106/wasm-short_row_grid_r4.tsv)

native p1 raw 99.99207%

![short_row_grid_r4 p1 native review](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/native/short_row_grid_r4/review_001.png)

![short_row_grid_r4 p1 native overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/native/short_row_grid_r4/overlay_001.png)

wasm p1 raw 100.00000%

![short_row_grid_r4 p1 wasm review](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/wasm/short_row_grid_r4/review_001.png)

![short_row_grid_r4 p1 wasm overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/wasm/short_row_grid_r4/overlay_001.png)

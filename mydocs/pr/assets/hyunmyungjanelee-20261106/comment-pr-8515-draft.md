<!-- 로컬 초안. 원격 승인·실제 merge·운영 기록·devel sync 뒤 integration_pr/merge_sha를 확정해 게시한다. -->
기여해 주셔서 감사합니다. [#8515](https://github.com/edwardkim/rhwp/pull/8515)의 원 기여 commit을 보존하여 [통합 PR #{integration_pr}](https://github.com/edwardkim/rhwp/pull/{integration_pr})의 merge commit `{merge_sha}`로 devel에 반영했습니다.

떠 있는 표 오프셋을 복원하고 후속 인라인 표가 paint 하단 때문에 밀리지 않도록 실제 문단 원점을 공유했습니다. 큰 signed 오프셋과 바깥 여백의 정수 overflow도 보정했습니다.

같은 입력의 독립 한컴 Desktop Print와 전체 1쪽을 비교해 Native 최저 **100.00000%**, 최적화 fresh WASM 최저 **100.00000%**를 확인했습니다. 전체 회귀·Native 공식 검사·lint·새 sample 보안 및 최종 head CI의 실제 결과는 [검토 기록](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/archives/pr_8515_review.md)에 남겼습니다. 수치는 원 재현물의 전쪽 최저이며 추가 경계와 남은 차이는 별도로 구분했습니다.

문서 비교는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 따랐습니다. 아래 raw 실루엣은 96dpi·2px 이웃 기준의 내용 일치율 보조값이며 사람의 판정 정확도가 아닙니다. 독립 Print와 같은 영역을 직접 확대 판독했습니다.

입력 `textbox_floating_table_offset`: [Native 전쪽 TSV](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/assets/hyunmyungjanelee-20261106/native-textbox_floating_table_offset.tsv), [fresh WASM 전쪽 TSV](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/assets/hyunmyungjanelee-20261106/wasm-textbox_floating_table_offset.tsv)

native p1 raw 100.00000%

![textbox_floating_table_offset p1 native review](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/native/textbox_floating_table_offset/review_001.png)

![textbox_floating_table_offset p1 native overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/native/textbox_floating_table_offset/overlay_001.png)

wasm p1 raw 100.00000%

![textbox_floating_table_offset p1 wasm review](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/wasm/textbox_floating_table_offset/review_001.png)

![textbox_floating_table_offset p1 wasm overlay](https://raw.githubusercontent.com/edwardkim/rhwp/{merge_sha}/mydocs/pr/assets/hyunmyungjanelee-20261106/comment-visual/wasm/textbox_floating_table_offset/overlay_001.png)

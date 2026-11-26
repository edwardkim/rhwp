# PR #9416 검토 · Word 호환 문단의 줄 높이와 형식 변환

## 최종 판정

**메인테이너 보정 후 수용.** [통합 PR #9428](https://github.com/edwardkim/rhwp/pull/9428)에서 원 기여를 보존해 병합했다. [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9428#pullrequestreview-6070130915) · [병합 커밋 `79f17a994a6f`](https://github.com/edwardkim/rhwp/commit/79f17a994a6fb42b85cf3ad06672d537361ce9a8)

세 건 중 첫 번째로 원 head를 기준 `devel`에 병합했다(`1536bcad98`). Git의 충돌 없는 병합 결과와 실제 트리가 같았다. 메인테이너는 `2cf02c3e29`에서 글꼴·공백 배치를 보완하고 `02bc510e7d`와 `d034f6602e`에서 양방향 저장과 Word 줄 메트릭 보존을 보완했다.

hyunmyungJaneLee의 원 기여 `ae243b34c1744e68dfbe32ab2866148228b9a53b`와 메인테이너가 보완한 제품 소스 `9e601ad6a862a30b019ad9563a1d597f6a33222b`을 검토했다. 대조 기준은 `devel`의 `cbebdbeeca060efe670cb8702355e4d79cb7e70e`다. [원 PR](https://github.com/edwardkim/rhwp/pull/9416)

같은 `gulim.ttc`를 한글 이름과 영문 이름으로 지정한 한컴 인쇄본은 96 DPI에서 픽셀이 같았다. 세 원 기여만 병합한 Native에서는 한글 이름 63.37%, 영문 이름 99.54%였다. 보완본은 문서의 글꼴 슬롯과 이름이 가리키는 실제 글꼴을 따라 같은 줄 메트릭을 사용한다. 두 이름과 표 안 문단은 Native에서 각각 100.00%다. [글꼴과 인쇄본의 대조](../assets/hyunmyungjanelee-20261126/font-check.tsv)

HWP 5의 호환 대상·조판 플래그와 HWPX의 `compatibleDocument`를 양방향으로 보존한다. 표준 HWPX만으로 복원할 수 없는 플래그는 해당 호환 헤더에 결합해 보존하고, 저장 줄이 없는 비표준 Word 설정에서는 실제 배치한 본문 줄의 시작·높이·기준선·줄 간격을 HWPX에 기록한다. 외부 편집으로 호환 헤더를 바꾼 대조군도 검사했다. 한컴으로 저장·인쇄한 굴림 대조군에서 HWP 변환 뒤의 마지막 줄이 원래 위치로 돌아왔다. [재현·보완 근거](hyunmyungjanelee_20261126_review_impl.md) · [한컴 재인쇄의 쪽별 결과](../assets/hyunmyungjanelee-20261126/final/desktop-pixels.tsv)

#8582에는 Word 탭 위치의 문제가 남는다. 이 PR의 처리 범위는 줄 높이와 호환 설정 보존이다. [연결 이슈](https://github.com/edwardkim/rhwp/issues/8582) · [통합 검증](../assets/hyunmyungjanelee-20261126/final/README.md)

## Merge 후 contributor PR comment 계획

[원 PR](https://github.com/edwardkim/rhwp/pull/9416)의 첫 완료 댓글에서 기여에 감사를 전하고 Word 줄 높이를 보정한 원 기여와 굴림 이름·호환 설정 보존을 보완한 이유를 설명한다. [쪽별 검토표](../assets/hyunmyungjanelee-20261126/final/claim-comparisons.tsv) · [시각 검증 댓글 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)

Native 원본 20쪽 중 자동 확대 검토 후보는 0쪽이다. 전체 픽셀 일치율 최저는 98.65614%, 잉크 픽셀 기준 최저는 28.84874%다.

fresh WASM 원본 20쪽 중 자동 확대 검토 후보는 0쪽이다. 전체 픽셀 일치율 최저는 98.65233%, 잉크 픽셀 기준 최저는 28.63324%다.

위 검토표에서 이 원 PR의 각 입력·쪽·출력 경로에 대응하는 review와 overlay 80장을 [병합 커밋 `79f17a994a6f`](https://github.com/edwardkim/rhwp/commit/79f17a994a6fb42b85cf3ad06672d537361ce9a8)에 고정해 첫 댓글에 표시한다. 게시 전 공개 이미지와 Git 바이트를 비교하고 게시 후 실제 표시를 확인한다.

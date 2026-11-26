# PR #9420 검토 · 문단 안 인라인 표와 앞뒤 글자의 쪽 소속

## 최종 판정

**메인테이너 보정 후 수용.** [통합 PR #9428](https://github.com/edwardkim/rhwp/pull/9428)에서 원 기여를 보존해 병합했다. [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9428#pullrequestreview-6070130915) · [병합 커밋 `79f17a994a6f`](https://github.com/edwardkim/rhwp/commit/79f17a994a6fb42b85cf3ad06672d537361ce9a8)

#9416·#9418을 적용한 뒤 세 번째로 원 head를 병합했다(`3f09b6d71d`). 두 부모의 충돌 없는 병합 결과와 실제 트리가 같았다. 메인테이너는 `2b5f6775b4`에서 표 높이의 중복 계상을 고치고 `9e601ad6a8`에서 첫 줄에 누름틀이 있는 대조군의 표 간격을 복원했다.

hyunmyungJaneLee의 원 기여 `6485652352171c06446d5ce3ba2540675ce46250`은 저장 줄이 없는 문서에서 양수 세로 오프셋의 표가 종이 아래로 밀리는 회귀를 다룬다. 메인테이너는 표 앞과 뒤에 보이는 문자가 있는 입력까지 검토했다. [원 PR](https://github.com/edwardkim/rhwp/pull/9420) · [원 이슈](https://github.com/edwardkim/rhwp/issues/8463)

같은 원본에서 표 뒤의 공백 하나를 `X`로 바꾸면 한컴 인쇄는 1쪽이다. 원 기여와 이전 `devel`은 표 높이를 두 번 반영해 2쪽을 만들었다. 보완본은 재조판 시 문단의 문자·개체 순서에서 표를 소유한 줄을 찾고, 그 줄에 반영한 표 높이를 배치 단계에서 사용한다. 표 앞 글자, 뒤 글자, 공백과 양수·0 오프셋을 대조했다. [수정 전 두 쪽과 인쇄본](hyunmyungjanelee_20261126_review_impl.md)

양수 오프셋의 앞 글자·뒤 글자 대조는 보완본 Native에서 한컴과 같은 1쪽이 됐다. HWP 및 HWPX로 저장한 대조본 네 개를 한컴에서 직접 인쇄하니 각각 원본 인쇄와 픽셀이 일치했다. [재인쇄 결과](../assets/hyunmyungjanelee-20261126/final/desktop-pixels.tsv) · [통합 검증](../assets/hyunmyungjanelee-20261126/final/README.md)

전체 회귀에서 누름틀 시작 줄 다음의 두 표가 겹치는 중간 보완본을 발견했다. 글자가 없는 개체 문단에서는 이미 배치한 누름틀 줄을 다시 방출하지 않도록 보완했다. 한컴 인쇄본에서 두 표의 경계 사이에 보이는 간격을 되살렸고 활성 회귀의 기대값을 유지했다. [한컴 인쇄·devel·보완 전후의 표 경계](../assets/hyunmyungjanelee-20261126/final/field-over19-comparison.png)

## Merge 후 contributor PR comment 계획

[원 PR](https://github.com/edwardkim/rhwp/pull/9420)의 첫 완료 댓글에서 기여에 감사를 전하고 인라인 표 이월을 고친 원 기여와 표 앞뒤 글자·누름틀 줄의 중복 점유를 보완한 이유를 설명한다. [쪽별 검토표](../assets/hyunmyungjanelee-20261126/final/claim-comparisons.tsv) · [시각 검증 댓글 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)

Native 원본 1쪽 중 자동 확대 검토 후보는 0쪽이다. 전체 픽셀 일치율 최저는 97.51916%, 잉크 픽셀 기준 최저는 31.43362%다.

fresh WASM 원본 1쪽 중 자동 확대 검토 후보는 0쪽이다. 전체 픽셀 일치율 최저는 97.56597%, 잉크 픽셀 기준 최저는 30.55342%다.

위 검토표에서 이 원 PR의 각 입력·쪽·출력 경로에 대응하는 review와 overlay 4장을 [병합 커밋 `79f17a994a6f`](https://github.com/edwardkim/rhwp/commit/79f17a994a6fb42b85cf3ad06672d537361ce9a8)에 고정해 첫 댓글에 표시한다. 게시 전 공개 이미지와 Git 바이트를 비교하고 게시 후 실제 표시를 확인한다.

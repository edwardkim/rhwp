# PR #9418 검토 · 분할 표의 칸 안 그림과 뒤 문단

## 최종 판정

**메인테이너 보정 후 수용.** [통합 PR #9428](https://github.com/edwardkim/rhwp/pull/9428)에서 원 기여를 보존해 병합했다. [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9428#pullrequestreview-6070130915) · [병합 커밋 `79f17a994a6f`](https://github.com/edwardkim/rhwp/commit/79f17a994a6fb42b85cf3ad06672d537361ce9a8)

#9416을 적용한 뒤 두 번째로 원 head를 병합했다(`d9c2ed5345`). 두 부모의 충돌 없는 병합 결과와 실제 트리가 같았다. 메인테이너는 `4b6aedfe7a`에서 그림 양옆의 글 조각을 한 줄로 배치하도록 보완했다.

hyunmyungJaneLee의 원 기여 `0553e7137303efa88a5c93722fa835c4de32a4c4`과 메인테이너 보완본을 대조했다. #8683에서 수용한 개체 전용 문단에 이어, 그림 뒤에 본문이 있는 칸의 어울림 배치와 행 높이를 확인했다. [원 PR](https://github.com/edwardkim/rhwp/pull/9418) · [선행 수용 범위](https://github.com/edwardkim/rhwp/issues/8579)

공개 HWPX에서 그림을 120 HU 옮기면 한컴은 같은 기준선의 양옆에 글을 놓는다. 이 경계는 이전 `devel`에서도 재현됐다. 원 기여는 한쪽만 열린 줄을 처리했고, 보완본에서는 줄의 첫 조각·끝 조각과 같은 세로 위치를 사용해 양쪽을 한 줄로 측정하고 그린다. 공백 문단의 줄 점유도 유지한다. [그림과 두 글줄의 인쇄 좌표](../assets/hyunmyungjanelee-20261126/reference-text.tsv)

개체 전용 칸, 그림 뒤 글이 있는 원본, 양쪽에 글이 흐르는 문서, 오른쪽만 열린 경우와 양쪽이 좁은 경우를 비교했다. HWP와 HWPX로 각각 저장한 양쪽 흐름 문서는 한컴에서 두 쪽 모두 원본과 같은 픽셀로 인쇄됐다. [재인쇄 결과](../assets/hyunmyungjanelee-20261126/final/desktop-pixels.tsv) · [통합 검증](../assets/hyunmyungjanelee-20261126/final/README.md)

기여자가 대조군으로 든 #8695·#8576 폴더의 공개 원본 10개도 전부 확인했다. 두 출력 경로 각각 11쪽이 한컴 대비 100.00%였고, 기준 소스와 보완본의 PNG 22쌍도 모두 같았다. [대조 원본·인쇄본·쪽별 수치와 해시](../assets/hyunmyungjanelee-20261126/final/picture-compatibility.tsv)

## Merge 후 contributor PR comment 계획

[원 PR](https://github.com/edwardkim/rhwp/pull/9418)의 첫 완료 댓글에서 기여에 감사를 전하고 셀 그림 뒤 본문을 복원한 원 기여와 그림 양옆에 배치되는 글을 한 줄로 측정한 이유를 설명한다. [쪽별 검토표](../assets/hyunmyungjanelee-20261126/final/claim-comparisons.tsv) · [시각 검증 댓글 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)

Native 원본 4쪽 중 자동 확대 검토 후보는 0쪽이다. 전체 픽셀 일치율 최저는 98.39157%, 잉크 픽셀 기준 최저는 41.25477%다.

fresh WASM 원본 4쪽 중 자동 확대 검토 후보는 0쪽이다. 전체 픽셀 일치율 최저는 98.41043%, 잉크 픽셀 기준 최저는 41.50765%다.

위 검토표에서 이 원 PR의 각 입력·쪽·출력 경로에 대응하는 review와 overlay 16장을 [병합 커밋 `79f17a994a6f`](https://github.com/edwardkim/rhwp/commit/79f17a994a6fb42b85cf3ad06672d537361ce9a8)에 고정해 첫 댓글에 표시한다. 게시 전 공개 이미지와 Git 바이트를 비교하고 게시 후 실제 표시를 확인한다.

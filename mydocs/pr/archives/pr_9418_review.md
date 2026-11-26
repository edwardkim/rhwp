# PR #9418 검토 · 분할 표의 칸 안 그림과 뒤 문단

#9416을 적용한 뒤 두 번째로 원 head를 병합했다(`d9c2ed5345`). 두 부모의 충돌 없는 병합 결과와 실제 트리가 같았다. 메인테이너는 `4b6aedfe7a`에서 그림 양옆의 글 조각을 한 줄로 배치하도록 보완했다.

hyunmyungJaneLee의 원 기여 `0553e7137303efa88a5c93722fa835c4de32a4c4`과 메인테이너 보완본을 대조했다. #8683에서 수용한 개체 전용 문단에 이어, 그림 뒤에 본문이 있는 칸의 어울림 배치와 행 높이를 확인했다. [원 PR](https://github.com/edwardkim/rhwp/pull/9418) · [선행 수용 범위](https://github.com/edwardkim/rhwp/issues/8579)

공개 HWPX에서 그림을 120 HU 옮기면 한컴은 같은 기준선의 양옆에 글을 놓는다. 이 경계는 이전 `devel`에서도 재현됐다. 원 기여는 한쪽만 열린 줄을 처리했고, 보완본에서는 줄의 첫 조각·끝 조각과 같은 세로 위치를 사용해 양쪽을 한 줄로 측정하고 그린다. 공백 문단의 줄 점유도 유지한다. [그림과 두 글줄의 인쇄 좌표](../assets/hyunmyungjanelee-20261126/reference-text.tsv)

개체 전용 칸, 그림 뒤 글이 있는 원본, 양쪽에 글이 흐르는 문서, 오른쪽만 열린 경우와 양쪽이 좁은 경우를 비교했다. HWP와 HWPX로 각각 저장한 양쪽 흐름 문서는 한컴에서 두 쪽 모두 원본과 같은 픽셀로 인쇄됐다. [재인쇄 결과](../assets/hyunmyungjanelee-20261126/final/desktop-pixels.tsv) · [통합 검증](../assets/hyunmyungjanelee-20261126/final/README.md)

기여자가 대조군으로 든 #8695·#8576 폴더의 공개 원본 10개도 전부 확인했다. 두 출력 경로 각각 11쪽이 한컴 대비 100.00%였고, 기준 소스와 보완본의 PNG 22쌍도 모두 같았다. [대조 원본·인쇄본·쪽별 수치와 해시](../assets/hyunmyungjanelee-20261126/final/picture-compatibility.tsv)

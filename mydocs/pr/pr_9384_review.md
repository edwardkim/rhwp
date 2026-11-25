# PR #9384 검토 · 줄 정보가 없는 칸의 사진 배치

원 기여자는 hyunmyungJaneLee이고 원 head는 `c3c76e6c2d2b07389714dbafbb40505747b47ec6`이다. #9383의 가로 나란한 사진, 단일 사진, 가로로 겹쳐 세로로 쌓이는 사진을 검토한다. 선행 #9352를 포함하며 #9407과는 병렬이다. [원 PR](https://github.com/edwardkim/rhwp/pull/9384) · [연결 이슈](https://github.com/edwardkim/rhwp/issues/9383)

최신 devel `50324d194d`에 #9352와 이 원 head를 반영한 로컬 커밋은 `c9b3a8ed9d`이다. 세 원 PR의 누적 검토 소스는 `f11b8624fd`이다. 한컴 2020 원본 인쇄 영수증은 제출된 HWP 및 PDF와 일치한다. [기여자의 인쇄 기록](assets/side_by_side_photo_cell/hancom-print-provenance.json)

창에서 좌우 기준을 바꿔 같은 위치를 입력하는 편집을 재현하기 위해 마지막 사진 한 장의 가로 정렬과 오프셋만 바꿨다. 한컴 2020에서 원본(왼쪽, 4,000 HU)과 대조본(오른쪽, 5,718 HU)의 96 DPI 전쪽 색상 픽셀이 동일했다. 두 문서 모두 마지막 사진을 그 앞 사진 바로 아래에 배치했다. 용지 높이만 줄인 대조본은 한컴에서 같은 쪽 아래에서 일부 사진이 잘리므로 분할 행 쌓기의 정답으로 삼기 어렵다. [이번 검토 계획](hyunmyungjanelee_20261125_review_plan.md)

선행 #9365의 보정 `de678c8325`는 어울림(Square) 그림에서 각 그림의 가로 정렬과 겹침 허용을 반영했다. 이번 원 PR의 자리차지(TopAndBottom) 경로는 `unstored_cell_no_overlap_topbottom_picture`에서 왼쪽 정렬로 대상을 제한한다. 물리적으로 같은 위치인 대조본에서 한컴처럼 세로로 쌓이는지를 별도로 확인한다. [선행 보정 기록](pr_9339_review.md)

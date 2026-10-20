# 그림 배치 모드의 한컴 교차 저장 관측

이 파일들은 픽셀 baseline이나 조판의 정답지가 아니다. 공통 IR의 서로 다른 배치 모드를
두 파일 형식에서 어떤 enum 값으로 기록하는지 확인한 독립 저장 결과다.

- `hancom-original-save.hwpx`: 저장소 `156086935_none_image_fill.hwp`를 한컴 2020으로
  무편집 저장. 원본 HWP5 값 15(None)의 글상자·두 BorderFill이 모두 `ZOOM`이다.
- `hancom-zoom-back.hwp`: 위 HWPX를 한컴 2020으로 다시 HWP 저장. 세 곳 모두 값 15다.
- `maintainer-total.hwpx`: 사용자가 글상자 그림을 ‘크기에 맞게’로 변경한 대조군.
  글상자는 `TOTAL`, 셀의 원래 `TILE`은 그대로다. 최종 보정 문서로 대체하지 않는다.
- `hancom-total-back.hwp`: 위 대조군을 한컴 2020으로 HWP 저장. 해당 글상자는 값 5다.

생성 job·관측과 수용 범위는 `mydocs/pr/pr_7661_review.md`에 기록한다.
전체 문서의 시각 판정이나 저장소 모든 배치 모드의 지원을 이 파일들로 주장하지 않는다.

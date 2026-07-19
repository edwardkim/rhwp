2026년 7월 4일 

일반 PR review, collaborator-mediated review, 여러 PR 체리픽 누적 검토 모두에서 PR 내용상 렌더링 결과
확인이 필요하면 [PDF/SVG visual sweep 가이드](../manual/verification/visual_sweep_guide.md)를 사용한다. 시각 검증은 모든
샘플 PR 에 기계적으로 수행하는 절차가 아니라, PR 의 수정 목적과 검증해야 할 사용자-visible 동작에 맞춰
선택한다.

manual/visual_sweep_guide.md 보면

`scripts/task1274_visual_sweep.py`는 rhwp가 만든 SVG/render tree와 한컴 기준 PDF를 비교해
문항 흐름 drift, frame overflow, 줄 순서 겹침 같은 후보를 자동으로 찾는 보조 도구다.

이 도구는 메인테이너의 최종 시각 판정을 대체하지 않는다. 대신 다음을 빠르게 확인한다.

- SVG/PDF 페이지 수 일치
- PNG/PDF raster overlay 차이 위치
- 페이지별 픽셀/잉크 영역 일치율
- 문항 marker y drift 후보
- frame/tail overflow 후보
- 수식/본문 겹침 후보
- 줄 band/order drift 후보

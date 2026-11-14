# PR #9063 메인테이너 보완 계획

2026-11-14 KST. 작업지시자는 기존 [편집 경계 보완안](assets/pr9063/edit-boundary/README.md)에 대해
**메인테이너 보완·필수 검증**을 승인했다. 이 승인은 로컬 보정과 검증 범위다.

## 기준과 원인

- 원 기여 head `b43050954c10512705d714114f5af23f1ee98ac6`과 여섯 커밋을 보존한다.
- 기준 devel `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`, 기존 로컬 검토 기록 `8bd1df5891`.
  재개 시 원 head·devel·reviewer가 변하지 않았음을 API로 확인했다.
- 같은 공개 입력·HCR 프로그램의 기본 열기 전 22쪽은 Native/fresh WASM 모두 raw 100.00%다.
- 첫 문단에 `X`를 넣으면 그림·도형이 2→1쪽으로 돌아가고 재열기 시 2쪽이 된다.
  독립 한컴 Print의 2쪽은 편집 전후 RGB가 동일하다. fit 대조군은 1쪽을 유지한다.
- HWP5 CommonObjAttr bit 13의 쪽 영역 제한은 개체 속성이다. 문서 전체의 편집 상태는
  해당 속성을 해제하는 조건이 아니다. 저장 줄의 수용 가능성과 실제 재조판 원점은 구분한다.

## 공통 소비 경로

`section.rs`가 문단 흐름 전의 `(page, column, current_height)`를 `picture_host_origin`으로 만든다.
보통 문단은 format·fit·vpos 조정 후 `section/controls.rs`가 이 값을 읽고, 표를 품은 문단은
`controls.rs::place_table_host_shape_with_storage`가 `para_start_height`를 전달한다.
현재 공통 `restricted_object_host_top`은 전체 edited 플래그에서 중단한다.
수용되면 `overflow_para_object_next_page_placement` → `defer_stored_frame` → 단/쪽 전이의
`page_start_stored_frames` → `layout_column_shapes_pass`의 확정 앵커로 이어진다.

보완은 이 공통 경계에서 저장/재조판의 유효 원점을 구분하고, 측정·본문 점유·실제 배치가
같은 결과를 소비하도록 한다. 단순 플래그 삭제로 완료를 가정하지 않는다. 특히 자리차지 개체가
이월될 때 호스트·뒤 본문이 앞쪽에서 잘못 예약되지 않는지 실제 호출 순서로 확인한다.
문서별 상수·잘못된 좌표의 clamp·출력 숨김으로 해결하지 않는다.

## 구현과 검증 순서

1. 현재 공개 샘플과 실제 편집 API로 저장 줄·flow 원점·쪽 소유를 추적한다. 저장 줄 유무,
   편집 위치와 기준 문단·개체 속성을 대조하고 최소 경계에서 수정 전 실패를 유지한다.
2. 공통 원점/이월과 필요한 실제 소비 지점을 보완한다. 그림·도형·표를 품은 문단,
   fit·제한 off·자리차지 뒤 본문의 소유·내용 보존을 먼저 확인한다.
3. Native와 최적화한 fresh WASM의 편집 직후·HWP/HWPX 반복 저장·재열기를 실제 실행한다.
   같은 편집 저장본의 독립 Print를 사용해 전쪽 TSV와 비교·overlay를 산출하고 직접 판독한다.
   실제 Studio Canvas 합성과 SVG 소비 경로도 변경 범위에 맞춰 확인한다.
4. 시각 선행 조건을 충족한 뒤 정식 관계형 회귀를 보완한다. 기존 두 회귀 파일의 같은 입력과
   과거 고정값·측정 주석을 함께 확인하고 독립 근거에 맞춰 유지·수정한다.
5. 최종 code head에서 최소 글꼴 전체 release-test, Native 공식 3종, Rust lint 전부,
   편람 HWPX Native 383쪽 gate와 정본 보호 TSV 전체의 하락 금지를 완료한다.
   #9066 등 현재 base에서 반영한 공통 배치 기능도 실제 대조한다.
6. 검토 기록을 최종 SHA·명령·실제 결과로 갱신한다. 최종 대표 PNG·전쪽 TSV·입력/PDF를 남기고,
   이전 로컬 검토에 잘못 포함했던 원시 로그·MCP 응답·분석 JSON·중복 raster는 기록으로 요약해
   최종 변경에서 제외한다. 중간 산출물은 root ignored output에만 둔다.

원격 push·PR 생성·공개 댓글·승인 리뷰·merge는 이 승인 범위에 포함하지 않는다.

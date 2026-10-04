---
kind: investigation
status: active
canonical: mydocs/tech/typesetting_architecture.md
last_verified: 2026-10-04
---

# #7207 — RowBreak 선행 병합 뒤 저장 프레임 잔여 보정

기준 devel은 `1d6bc70767fad365b07afe4ef57972d23b140f2b`이며, #7567의
RowBreak·최소 공통 메트릭 보정을 포함한다. 원 후보 `bac75f50ee4e57839f4c0ac7a259165acf0e3509`는
별도 브랜치에 보존했다. 새 후보는 기준 devel에서 시작하며, 중복 공백 메트릭 변경을 제외한
6개 source 파일의 저장 표 프레임·물리 공간 처리만 재검증한다. #7544의 본문·source는 수정하지 않는다.

## 독립 근거와 주장 경계

실제 저장 원문 `samples/task2097/18095317_eogu_geumji.hwp`와 그 원문의
`pdf/18095317_eogu_geumji-2020.pdf`를 사용한다. 보존 후보의 과거 21쪽 통과 결과는
이번 head의 증거로 승계하지 않는다. 교육과정의 기존 413/415쪽 차이는 별도 문제로 유지한다.

저장 셀 최소 높이는 내용 컷뿐 아니라 정렬에 필요한 빈 물리 공간을 소유한다.
원본 전폭 셀의 frame reset·원본 줄 정보·소유를 검증한 경로만 대상이며,
편집·합성/투영 줄·경쟁 rowspan 소유자는 적용 경계에서 제외한다.
문서 ID·임의 크기 조건을 추가하거나 baseline·허용치를 완화하지 않는다.

현재 후보의 실제 소비 연결은 `stored_full_width_row_source_height` →
`stored_full_width_row_declared_height` → `fragment/emit`의 실제 수용 높이와 누적
`stored_row_box_sum` → 다음 시작 행 override → `RowScanQuery::whole_row_height` →
`PartialTable` → `table_partial`의 저장 컷 정렬이다. emit의 높이 변경이 예산 수용과
후속 내용까지 일관되게 연결되는지 추가 대조한다. helper 이름만으로 충족 판정하지 않는다.
SectionDef·ColumnDef는 본문 프레임 설정이며 단독 표 앵커에 경쟁하는 가시 객체가 아니다.
그림·추가 표·본문 텍스트는 단독 앵커 대조군에서 제외한다.

## 실행 순서

1. 최신 devel 및 잔여 후보의 작은 기존 컷·이어받기 검사와 영향 페이지를 먼저 실행한다.
2. 같은 원문의 Native/fresh WASM 전체 TSV와 대표 compare/review/overlay를 새로 만든다.
3. RowBreak 정상 대조군 전체 18쪽 및 다른 영향 문서의 내용·소속·쪽수·배치를 대조한다.
4. 시각 선행 조건 충족 뒤 독립 기대값의 정식 회귀를 추가하고 수정 전 FAIL/후 PASS를 확인한다.
5. 최종 source에서 전체 회귀·Native Skia·필수 세 Clippy·workspace build·고정 base 정책을 실행한다.
6. 실제 실행 결과와 잔여 범위를 기록하고 PR 본문 초안을 준비한다. 원격 push·PR 생성은 별도 승인 뒤 진행한다.

사용자 제공 글꼴은 비공개 로컬 검증에만 사용한다. 글꼴 파일·식별 자료·글꼴 포함
SVG/HTML/로그는 공개 증적으로 포함하지 않는다. 허용할 raster와 수치만 별도로 검토한다.

현재 상태: 잔여 후보 이식 완료, 새 head 검증 미실행. 과거 통과 결과와 구분한다.

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

## 공통 보완과 1차 실행

2026-11-14 KST. 저장된 vpos는 편집 후 원점으로 재사용하지 않고, 편집 흐름에서는 현재 문단의
조판 커서를 공통 helper에 전달했다. 자리차지 그림의 선행 fit 경로인
`paragraph.rs::prepare_fit_budget` → `entry::edited_picture_requires_transition`은 별도 쪽을
소유하는 개체를 같은 `overflow_para_object_next_page_placement` 판정으로 제외한다.
따라서 이월할 그림 높이를 호스트 문단에 더해 문단 자체를 먼저 옮기지 않는다. 이후
`section/controls.rs`의 같은 원점·예산 판정 → queue → transition의 `next_flow_y` 예약 →
확정 앵커 paint로 이어진다. 그림을 다음 쪽에 그린 뒤 본문은 그 점유 끝 아래에서 시작한다.

선행 fit의 원 해결은 #6882 → #6883의 `17e34fdca3` → 수용 #6944의 `74d0a68b7491`이다.
[기존 검토](archives/pr_6883_review.md)와 현재 호출을 확인했다. 원점 제한 없는 개체·caption·
음수 offset·한 쪽보다 큰 개체·다단은 기존 판정 범위에 남으며, 실제 표 성장/복구 검사를
같이 실행한다. 속성 제한을 문서의 edited 플래그로 해제하지 않는다.

Native 1차 실제 편집 검사는 6개 입력에서 그림·도형·표 호스트·자리차지 뒤 본문의
2→2→2쪽, fit·제한 off의 1→1→1쪽을 확인했다. 1차 단순 플래그 수정만으로는 뒤 본문 사례가
3쪽에 남았고, 선행 fit 보완 뒤 2쪽으로 복원됐다. 새 회귀의 등록은 fresh WASM과 독립 Print
전쪽 측정을 완료한 뒤 한다. 이 기록은 최종 필수 검증의 통과 선언이 아니다.

문단 앞 간격 900HU를 추가한 fit 대조군을 실제 편집 API로 저장해 독립 한컴 2020 Print로
확인했다. Print는 1쪽을 유지한다. 흐름 원점에 문단 앞 간격을 임의로 더하는 보정은 하지 않았다.
새 입력과 PDF는 최종 증적에 보존한다.

기준 문단 직접 편집도 8개 공개 입력 전체에서 실행했다. 합성 LineSeg를 저장 원점으로 읽던
재열기 경계를 보정한 뒤, 각 입력의 열기·직접 편집·HWP/HWPX 저장·재열기·두 번째 저장 및
재열기에서 쪽 소유가 유지된다. 합성 줄은 기존 `is_synthetic_line_seg` 계약으로 제외하며,
실제 저장 줄이 있는 경로의 기존 원점 판정은 유지한다. 앞 문단 Enter로 fit 입력이 실제로
넘치는 경우는 1→2쪽으로 바뀌고 저장 형식 양쪽에서 2쪽을 유지한다.

독립 한컴 Print에서 host 직접 편집 HWP/HWPX 모두 2쪽을 확인했다. Native 전체 페이지·
실제 출력 비교와 fresh WASM은 아래 최종 검증 기록으로 확정한다.

## 시각 선행 검증과 정식 회귀 보강

source `e2d7fb08b5`의 원 공개 17문서·22쪽을 Native/fresh 최적화 WASM에서 각각 다시 출력해
모든 쪽 raw 100.00%와 독립 Print 쪽수를 확인했다. 실제 편집 12종·20쪽도 양쪽 경로에서
raw 최저 100.00%다. Native의 반복 저장·재열기 60상태·100쪽과 별도 SVG 6종·10쪽도 100.00%다.
잘못된 다른 저장 형식의 PDF를 대조한 초기 계산은 제외하고, 실제 해당 형식의 Print를 썼다.

이 선행 조건과 직접 판독 뒤 기존 두 회귀 파일에 정식 검사를 보강했다. 개체의 단일 쪽 소유,
본문 순서·누락·중복, 다음 쪽 점유와 뒤 글줄, 실제 Enter 뒤 재조판 및 HWP/HWPX 반복 저장을
검사한다. 새 source-side unit-test 모듈이나 별도 CI 호출은 추가하지 않는다.

기존 TAC 상대 이동 assertion의 웹 Print 값 대신 실제 데스크톱 PDF의 TEXT 잉크 bbox에서
얻은 독립 이동을 사용했다(3.1966 / 18.2201 / 15.0235px). 줄 상자와 잉크 위치의 서브픽셀 차이는
기존 0.6px 여유 안에서 비교한다. 허용치는 늘리지 않았고 절대 페이지 px 핀도 추가하지 않았다.
첫머리에는 실제 측정 source·경로·전쪽 최저·직전 파일 commit 시각을 구분해 기록했다.

## 로컬 보완 완료

2026-11-14 KST. 승인된 보완·필수 검증을 모두 완료했다. 원 기여와 공통 원점·이월·실제 예약 보정, 정식 관계 회귀를 보존한다. 원본/저장 파생본 58쪽씩 Native/fresh WASM raw 100.00%, 정본 보호 3,542쪽의 공식값 하락 0, 필수 편람 383쪽 passed다. 전체 회귀 13,524건·Native 공식 4,073건·집중 41건 및 모든 Rust lint/build가 통과했다. 최종 주석 이후의 집중 실행과 신규 17개 보안/쪽수 원장 실행도 exit 0이다.

[최종 검토](pr_9063_review.md), [실행 원장](assets/pr9063/evidence.md)에 제품/검사 source와 명령·개수·환경 복구를 구분했다. 원시 JSON·로그·중복 raster는 정식 증거와 요약으로 대체한다. 원격 push·새 수용 PR·CI·self-review·merge·연결 이슈 완료는 [원격 처리안](assets/pr9063/remote-action-plan.md)의 별도 승인 대상이다.

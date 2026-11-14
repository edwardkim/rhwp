# 편집 후 제한 개체 이월이 해제되는 경계

> 보정 전의 접수 진단이다. 승인된 공통 보완과 최종 검증은 [검토 기록](../../../pr_9063_review.md) 및 [실행 원장](../evidence.md)을 따른다.

2026-11-14 KST 메인테이너의 보정 전 로컬 검토 기록이다. 아래 실패는 당시 source의 결과이며, 보정 후 결과는 상위 최종 기록과 구분한다.

## Native 재현

원 공개 입력 `samples/issue8660/picture_in_front_over_1px.hwp`를 `DocumentCore::from_bytes`로
열고 `insert_text_native(0, 0, 0, "X")`를 호출했다. 첫 문단은 `L00` → `XL00`으로 바뀌며
줄바꿈은 발생하지 않는다. 기준 문단 `ANCHOR`의 원점은 편집 전후 `(113.38667, 772.26667)`로
같다. 전체 본문 높이·쪽 영역·그림 속성을 바꾸는 편집은 하지 않았다.

| 입력 | 열기 | 한 글자 편집 직후 | HWP 저장·재열기 | 같은 편집본 한컴 2020 Print |
| --- | --- | --- | --- | --- |
| 그림, 1px 초과 | 2쪽, 그림은 2쪽 상단 | 1쪽, 그림은 1쪽 하단 | 2쪽 | 2쪽 |
| 자리차지 사각형, 12px 초과 | 2쪽, 도형은 2쪽 상단 | 1쪽, 도형은 1쪽 하단 | 2쪽 | 2쪽 |
| 그림, 정확히 fit 대조군 | 1쪽 | 1쪽 | 1쪽 | 원 입력 Print 1쪽 |

이는 열기 개선 자체가 잘못되었다는 판정이 아니다. 기존 devel에서도 이월 미반영이 있었으며,
후보의 열기 개선이 일반 편집 뒤 유지되지 않는 **미해결 경계**다. 새 devel 대비 악화라고
일괄 분류하지 않는다. 앞 문단에 단 한 글자를 넣는 동작으로 페이지 소유가 달라지고 저장·재열기로
다시 돌아오므로, PR의 `Closes #8660` 완료 판단 전에 해결해야 한다.

진단 source는 [edit-probe.rs.txt](edit-probe.rs.txt), 실행 결과는
[Native 실행 요약](../evidence.md), 실제 개체·첫 문단·기준 문단의 위치와 산출물 신원은 [실행 원장](../evidence.md)에 있다.
진단 HWP는 RHWP의 실제 편집·직렬화 API로 만든 공개 파생본이며 정식 회귀 suite에 등록하지 않았다.

같은 명시적 HCR 글꼴 환경을 적용하고 실제 Print PNG도 출력했다.
[Native 직접 비교](native-edit-review.png), [실제 PNG 진단 source](edit-png-probe.rs.txt),
[PNG 실행 결과](../evidence.md)를 따른다. 원본 픽셀을 그대로 가로 배치하고 바깥 라벨만
추가했다. 그림 위치를 고치거나 이미지를 확대·축소하지 않았다.

최적화한 fresh WASM을 실제 Chromium `152.0.7946.0`에서 실행해도 그림·사각형은
2→1→2쪽, fit 대조군은 1→1→1쪽이다. [실제 WASM 결과](../evidence.md)와
[브라우저 API 진단 source](wasm-edit-probe.mjs.txt)를 보존했다. 이 probe는 Native와 같은
명시적 FontEnvironment를 적용한 portable 조판 API 검사다. 실제 HCR 프로그램 loaded SHA는
별도의 최종 전쪽 Print Canvas 검증과 [글꼴 원장](../evidence.md)에서 확인했다.

편집본 Print는 `pdf/pr9063/edited-*-2020.pdf`다. 실제 출력은 Windows 한컴 2020
`11.0.0.9136`, `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`이며 작업 영수증은
상위 [실행 원장](../evidence.md)을 따른다. 원 입력과 편집본의 한컴 2쪽은 96 DPI RGB가 [동일](../evidence.md)하다. `ANCHOR` bbox도
[별도 기록](../evidence.md)에서 같다.

## 원인 경로

`src/renderer/typeset/state/transition.rs::restricted_object_host_top`이
`profile.session_edited()`이면 즉시 `None`을 반환한다. 따라서 보통 문단의
`typeset/section/controls.rs`와 표를 품은 문단의 `typeset/controls.rs` 모두에서
`overflow_para_object_next_page_placement`와 `defer_stored_frame`을 건너뛴다.
그리기 단계는 기존 같은 쪽 하단 제한을 적용한다. 저장 후 다시 열면 edited 세션이 해제돼
같은 속성의 개체가 다시 다음 쪽 소유를 받는다.

HWP5 CommonObjAttr bit 13은 문단 기준 개체의 쪽 영역 제한 속성이다. 저장 줄 캐시의 유효성과
이 속성의 의미는 구분해야 한다. 실제 재조판에서 얻은 유효한 기준 문단 원점으로 같은 이월
규칙을 적용하는 보완이 필요하다. 단순히 편집 플래그만 제거하거나 저장 위치와 계산 위치의
작은 값을 선택하는 것으로 완료를 가정하지 않는다.

## 보완 제안

원 기여를 보존하고 공통 기준 문단 원점을 추적해 편집·미편집 경로의 실제 배치가 같은
쪽 영역 제한 규칙을 소비하도록 한다. 그림·도형·표를 품은 문단, fit·제한 off 대조군,
자리차지 뒤 본문을 포함해 편집 직후·저장·재열기를 검증한다. Native/fresh WASM 전쪽 TSV와
독립 Print 직접 판독을 먼저 완료하고 관계형 회귀를 보완한다. 새 테스트와 기존 두 파일의
측정 주석은 실제 데스크톱 결과를 기준으로 갱신한다.

보정 시 최신 devel의 #9066 등 공통 배치 기능 보존과 해당 필수 로컬 게이트도 함께 검증한다.
현재 검토만으로 제품 보정·원격 push·댓글·머지를 승인받은 것으로 해석하지 않는다.

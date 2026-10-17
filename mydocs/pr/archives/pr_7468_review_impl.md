---
kind: snapshot
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-08
---

# PR #7468 보정 구현 기록

원 기여의 저장 문단 원점 및 뒤 문단 vpos 복구를 최신 devel에 적용했습니다. 메인테이너 보정 `282d5d77f…`와 검증 입력·회귀 보강 `fdea8a4cb…`를 분리했습니다.

최종 증적에는 사용자 제공 함초롬바탕 regular/bold를 실제 공급한 Native/fresh WASM 재실행 PNG 16개를 사용합니다. 전 4쪽/두 backend 관용 실루엣 100.00000%, 미달·누락 없음, 엄격한 내용 픽셀 일치 18.45444–38.23178%입니다. 초기 회귀 추가 선행 gate와 최종 글꼴 비교의 source·명령·font SHA 및 제한은 [검토 기록](pr_7468_review.md#사용자-제공-함초롬바탕-적용-최종-비교)에 구분하여 보존했습니다.

- 최신 `restamp_indentation`, batch deferred rebuild, 기존 stored-reset guard를 보존하여 오래된 PR의 merge context가 현재 구현을 되돌리지 않게 했습니다.
- 원 PR의 전역 synthetic-tail 제외 helper는 최신 devel의 guard 및 좁은 API 복구만으로 재현을 해결하므로 최종 diff에서 제외했습니다. `typeset.rs`의 최종 diff는 없습니다.
- `char_shape_after_empty_keeps_page.rs`의 첫 검사를 최종 TextLine 배치/소속/상대 간격/누락·중복/겹침 및 삭제·재열기로 강화하고 `set_char_shape_id_native` 경로를 별도로 추가했습니다.
- 로드만 확인하던 실제 저장 0 경계 검사는 한컴이 재저장한 70줄/2쪽 정상 입력의 인접 문단 편집 및 전체 줄 소속, 상대 위치, 재열기로 강화했습니다. 기존 sample16의 미달은 #7445에 남겨 두며 기대값·기준을 완화하지 않습니다.
- Native/fresh 최적화 WASM 전쪽 시각 gate 이후에만 검사와 HWP/PDF를 commit했습니다. 절대 출력 좌표 golden을 추가하지 않았습니다. fixture 설명은 [README](../../../tests/fixtures/pr7468/README.md)에 있습니다.

생산 값의 소비 경로, 실제 빌드/회귀 결과, 독립 Print 출처와 시각 잔차는 [검토 기록](pr_7468_review.md)에 있습니다.

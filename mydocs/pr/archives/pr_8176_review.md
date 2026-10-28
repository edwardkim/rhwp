# PR #8176 리뷰 — fix(serializer/hwp): 편집으로 만든 글상자의 저장 워드에 글상자 비트 (#8172)

## 최종 판정

**통합 반영 완료 — 지정된 원 PR 범위의 로컬 검증·최종 CI를 통과한 뒤 PR #8228로 병합했습니다.**

통합 [PR #8228](https://github.com/edwardkim/rhwp/pull/8228)의 최종 head `b1e93d8de2d4842f725415aee19bd5b98ecfb8b3`가 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/40196953497)를 통과했고, 2026-10-25 22:58:32 KST에 merge `3e02fe2749866a8453af3eb3901b9c1a674cccaf`로 devel에 반영됐습니다. 원 contributor head와 실제 검증한 통합 head를 구분하여 기록합니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [8176](https://github.com/edwardkim/rhwp/pull/8176) / planet6897 / devel |
| 원 contributor head | `6d20cb5353985492fcdd3340fc8c2ee699d9ed57` |
| 통합 검토 branch | `review/planet6897-20261024` |
| 검증 source / base | `673c1d5f7cdcb5573b793ba2d2abaaedd2602134` / `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a` |
| 현재 제출 후보 | `673c1d5f7cdcb5573b793ba2d2abaaedd2602134` / runtime `38ad63a903` |
| 적용 순서 | 현재 누적 history의 8번째 PR. 원 PR별 적용 SHA는 아래 표 참조 |
| 종료 지시 / 참조 | 원 본문 Closes ['8172'] / Refs []; 실제 종료 판정 전에는 실행하지 않음 |
| 원 PR 상태 조회 | 2026-10-25T13:01:34.007187+00:00 / OPEN / MERGEABLE |
| 원 PR reviewer | jangster77 |

## 변경과 검토 범위

편집으로 만든 글상자의 HWP 저장 워드에 글상자 비트를 보존합니다.

### 원 기여와 메인터너 보정

생성·부착·제거·저장 재열기 검사와 원 기여자의 한컴 수용 근거를 구분했습니다. 비트 계약을 무관한 문서 그림으로 입증하지 않습니다.

| 원 기능 commit | 통합 branch의 적용 commit |
| --- | --- |
| `6d20cb5353985492fcdd3340fc8c2ee699d9ed57` | `a8a37a0a165af049f785ff016041be60d0c396fd` |

아래 적용 commit은 이 통합 branch의 실제 history입니다. 원 PR source branch는 변경하지 않았습니다.

### 검토할 실제 소비 경로와 회귀 원본

- `src/document_core/commands/object_ops/shape.rs`
- `src/document_core/queries/hwpctrl_sets.rs`
- `src/model/shape.rs`
- `tests/cases/issue_8172_textbox_shape_storage_bit.rs`

공개 편집 API의 글상자 생성/부착/제거가 TEXT_BOX_STORAGE_BIT와 일치하고 HWP 저장·재열기 뒤에도 보존되는 저장 계약입니다. 화면상의 위치 개선을 주장하는 PR은 아닙니다.

## 검증 입력과 결과

- code head `673c1d5f7cdcb5573b793ba2d2abaaedd2602134`, base `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`.
- 공통 통합 검증: 렌더러 집중 회귀 16/16 PASS, MCP 계약 4/4 PASS입니다. MCP 기존 실패는 제외한 sample16의 64쪽 기대값이 남은 오류였고 선택 fixture의 선언된 24쪽 기대값과 연결했습니다. 암호 stdin·비노출 검사는 유지했습니다.
- 최종 전체 회귀:      Summary [1920.075s] 11481 tests run: 11481 passed (46 slow), 45 skipped
- 필수 gate 상태 `complete`. [공통 검증 원장](../assets/integration-planet6897/final-validation-20261025/README.md).
- sample16 계열과 편람 HWP/HWPX는 사용자 지시로 제외하고 기존 검사 삭제를 유지합니다. 현재 판정 범위를 다른 #7445 항목으로 확대하지 않습니다.

## 변경 내용과 대응하는 증적

### 이 원 PR에 대응하는 정식 회귀

현재 전체 회귀 로그의 대응 PASS 3건입니다.

- `rhwp::regression_suite_014 issue_8172_textbox_shape_storage_bit::attaching_and_detaching_text_box_keeps_storage_bit_in_sync`
- `rhwp::regression_suite_014 issue_8172_textbox_shape_storage_bit::created_textbox_carries_storage_bit_and_rectangle_does_not`
- `rhwp::regression_suite_014 issue_8172_textbox_shape_storage_bit::storage_bit_survives_hwp_save_and_reopen`

공개 편집 API의 글상자 생성/부착/제거가 TEXT_BOX_STORAGE_BIT와 일치하고 HWP 저장·재열기 뒤에도 보존되는 저장 계약입니다. 화면상의 위치 개선을 주장하는 PR은 아닙니다.

이 변경은 화면 배치를 입증할 대상이 아니므로 다른 PR의 review PNG를 붙이지 않습니다. 정식 테스트의 입력·assertion·종료 결과가 증거입니다.

## 원격 검토와 후속 처리

제출 자산 223개의 실제 raw 다운로드 바이트·SHA-256과 게시 본문 API·GitHub 화면의 review/overlay 표시를 확인했습니다. 최종 head의 CI·CodeQL·Render Diff·Proptest·Adapter·Gym 계약은 모두 통과했고 MERGEABLE·CLEAN 상태에서 사용자 승인으로 병합했습니다. 통합 이후 확정 기록은 사용자 지정 옵션 2의 별도 문서 PR로 보완합니다. 원 PR에는 이 통합 PR과 개별 검토 기록을 연결해 기여·보정·잔여를 설명한 뒤 통합 반영을 이유로 종료하며, contributor 원 브랜치는 변경하지 않습니다.

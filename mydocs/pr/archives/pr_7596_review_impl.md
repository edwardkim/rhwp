---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7596 메인터너 보정

## 원인과 반례

#7565의 `ActiveFieldInfo.para_idx`는 셀 주소에서도 본문 부모 문단이다. #7596의 본문/셀 필드 제거는 section과 로컬 control/cell 경로만 비교하여 다른 부모의 활성 필드까지 해제한다. 같은 section에 별개 표 두 개를 만들고 동일한 `(0,0,0)` 셀 경로의 다른 표 필드를 지우면 이 경계가 드러난다.

독립 기대값은 공개 API의 입력 대상이다. 첫 표의 누름틀 값 `AAA` 끝을 활성화하고 두 번째 표의 필드를 지운 다음 첫 표의 끝에 `값`을 넣으면 첫 필드 값은 `AAA값`이어야 한다. 수정 전 실제 값은 `AAA`로 남고 새 글자가 필드 밖으로 들어갔다. `tests/cases/clickhere_cell_parent_identity.rs`의 정식 검사를 b3c933a88 Native 라이브러리에 연결해 1 FAIL을 확인했다. 빈 필드 시작 삽입으로 작성한 첫 진단은 결함을 검출하지 않아 증거에서 제외했다.

## 수정 범위와 소비 경로

`remove_field_at` / `remove_field_at_in_cell` → 활성 주소의 section·본문 부모 일치 확인 → 해당 부모 안의 control 번호/셀 경로 비교 → `insert_text_in_cell_native`의 활성 필드 끝 삽입 → `collect_all_fields`로 실제 소유 값 확인. 다른 부모의 삭제는 활성 상태를 유지하고 같은 부모의 영향을 받는 주소에 대해서는 기존 번호 갱신/해제 규칙을 적용한다. 좌표·줄 높이·슬롯 허용치를 변경하지 않는다.

셀 삭제와 별개 본문 문단 삭제를 각각 반례로 실행한다. 관련 기존 #7565 활성화·snapshot·저장 왕복과 #7596 슬롯 보존 검사도 보정 뒤 실행한다. 완료 결과는 개별 리뷰와 오늘할일에 연결한다.

## 실행 결과

정식 회귀 두 개는 source b3c933a88에서 각각 필드 값 `AAA` / 기대 `AAA값`으로 FAIL했다. 본문 부모 조건 보정 뒤 #7565 관련6 PASS, #7596 슬롯 보존14 PASS다. 최초 진단에서 활성화 설정에 실패한 중간 본문 반례도 결함 증거에서 제외하고, 적법한 공개 API 생성 후 두 값 불일치로 실패한 최종 결과만 보존했다. [수정 전 증거](../assets/semanticist21-20261005/pr7596-before.json), [실패 요약](../assets/semanticist21-20261005/pr7596-before-summary.txt). 최종 head의 lint·전체 nextest/fresh WASM은 이어서 검증한다.

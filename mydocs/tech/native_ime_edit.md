---
kind: reference
status: active
canonical: mydocs/tech/native_ime_edit.md
last_verified: 2026-11-07
---

# 원자적 Native IME 범위 치환

`native_host::editor::ime_edit`는 한 본문 문단의 bounded UTF-16 delta를 처리한다.
Qt·Flutter UI나 runtime/C ABI transport는 포함하지 않는다. 기존 NativeEditor의 조합·undo·저장
계약을 이용하며 실제 OS 입력기·재변환 검증을 완료했다고 주장하지 않는다.

## 입력 좌표와 신원

`ime_edit_context(revision)`은 기존 `NativeImeContext`와 Native selection, revision,
font generation, composition ID, 현재 preedit의 UTF-16 범위를 반환한다. `context.text`에는
Native 문서에 임시 적용한 preedit가 포함된다. Flutter의 oldText/delta는 이 문자열을 기준으로 한다.
Qt의 preedit 제외 문맥·replacementStart는 adapter가 제공된 preedit 범위를 이용해 이 좌표로
옮겨야 한다. raw OS offset을 추측하거나 surrogate 중간을 clamp하지 않는다.

`apply_ime_edit(revision, &context, edit)`는 전달받은 전체 context가 현재 query와 정확히 같은지
확인한다. 선택만 바뀌었거나 조합을 취소·재시작해 revision이 같아도 과거 context를 거절한다.
문서 ID와 worker owner는 transport가 추가로 묶어야 한다. 같은 text/revision을 가진 다른 editor에
context를 보내도 된다는 권한 토큰이 아니다.

모든 UTF-16 범위는 half-open이다. 처음 replacement는 원 context를, Commit의 selection과 새
preedit replacement는 commit 결과를, 새 preedit selection은 최종 결과를 기준으로 한다.
기본 selection은 삽입 끝의 collapsed caret이다. preedit 중 selection의 두 끝은 preview 안에 있어야
한다. 같은 본문 문단만 허용하고 셀·개체·semantic control·다른 문단으로의 치환은 거절한다.

문맥은 기존 query의 최대 4096 scalar이며, 전체 preview와 원 selection이 이 창에 들어와야 한다.
범위를 벗어난 긴 조합/선택은 거절하고 기존 API나 다른 호스트 경로의 명시적 처리를 요구한다.
새 selection과 preview도 다음 문맥 query의 창 안에 들어오는지 mutation 전에 검사한다.
긴 preview의 기본 끝 caret가 이 조건을 만족하지 않으면 명시적인 범위 내 selection을 보내거나
입력을 나눠야 한다. 입력 offset을 자동으로 바꾸지 않는다.
입력 commit+preedit 문자열은 합계 UTF-8 16KiB와 4096 scalar 이하다. 큰 문자열은 scalar 수를
세기 전에 byte 상한으로 거절한다. control 문자·개행·탭은 기존 본문 입력 규칙대로 거절한다.

## 조합과 history

- `Continue`는 현재 preview 내부만 바꾸며 같은 composition ID와 처음의 undo/cancel snapshot을
  유지한다. surrounding delta를 적용하며 조합을 확정하려면 `Commit`을 사용한다.
- `Commit`은 기존 조합과 surrounding replacement를 하나의 undo entry로 확정한다. 기존 조합이
  있으면 그 조합 이전 snapshot이 undo 기준이다. 변경 없는 commit은 history를 추가하지 않는다.
- optional 새 preedit는 commit 직후 별도 snapshot을 갖는다. 새 preedit 취소는 확정한 commit을
  보존하고 post-commit selection으로 돌아간다. 그다음 한 번 undo하면 앞선 commit 전체가 취소된다.
- 실제 replacement range와 undo/cancel selection은 분리한다. 새 preedit range가 현재 selection과
  달라도 기존 `update_composition`과 `cancel_composition`이 올바른 문맥을 유지한다.

## 실패와 저장

일반 거절은 모든 범위·문자열·selection·지원 범위·snapshot 여유를 검사한 뒤 mutation을 시작하므로
문서나 Core event를 만들지 않는다. mutation 중 Core 오류는 현재 문서 snapshot으로 복원하고 새
snapshot만 해제한다. Native revision, dirty/checkpoint, pending save token, selection, composition,
undo/redo는 성공 전까지 publish하지 않는다. commit+preedit가 내용을 바꿔도 Native revision은 한 번
증가한다. 선택·조합 상태만 바뀌는 호출은 revision을 늘리지 않는다.

이는 기존 `edit_transaction`과 같은 Native 공개 상태의 원자성이다. Core의 이미 발행한 event log는
append-only이고, rollback의 내부 cache epoch·snapshot allocator ID도 되감지 않는다. 이를 event
stream 자체의 transaction 보장으로 해석하지 않는다. 호스트는 이 메서드의 성공 결과를 반영한다.

먼저 준비한 저장 bytes/report/token은 이후 IME 편집이나 거절로 바뀌지 않는다. 원 bytes의 I/O 완료
후 그 token을 acknowledge하면 준비 당시 revision만 저장된 것으로 처리하고 이후 편집은 dirty로
남는다. 조합 중 새로운 사용자 저장을 시작하지 않는 기존 정책도 유지한다.

계약 회귀는 `tests/cases/issue_536_native_ime_edit.rs`에 둔다. 일반 거절·예산 거절·저장 checkpoint
보존을 public API로 확인하며, mutation 중 Core 오류 rollback은 기존 `rollback_and_discard` 재사용과
코드 검토 범위다. 내부 Core 오류를 강제로 주입하는 production hook이나 새 source unit test는
추가하지 않는다. 이 경로를 실제 실행한 검증으로 표시하지 않는다. 실물 입력기 후보창·Flutter/Qt
event mapping·transport 검증은 각 호스트 연결 단계에서 수행한다.

## Worker와 C ABI

worker `imeEditContext {target}`는 `kind: imeEditContext`와 원자적 편집의
`context`를 돌려준다. `applyImeEdit {target, expected, edit}`는 그 context 전체와
`NativeImeEdit` JSON을 받고 `kind: edited` 결과를 반환한다. 두 요청 모두 기존 FIFO·문서
identity·revision 검사를 적용한다. `expected`와 replacement/preedit의 모든 문자열·cell path
capacity를 큐 byte credit에 포함하며 context 출력도 64 KiB credit으로 제한한다.

기존 session 요청에도 `imeEditContext`/`applyImeEdit`와 `expectedRevision`을 제공한다.
capabilities의 `atomicImeEdits`로 지원을 확인한다. worker 또는 session owner 밖에서
legacy ABI를 호출해도 된다는 의미는 아니다. Flutter는 owner-thread dispatcher를 사용한다.

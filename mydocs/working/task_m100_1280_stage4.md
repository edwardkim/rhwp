# Task #1280 4단계 완료보고서 — 글상자 안 붙여넣기 회귀 테스트 보강 + 이미지 붙여넣기 결함 발견

## 배경

작업지시자가 "#1280 이슈 본문이 기대 동작에 '커서 진입·텍스트 입력·붙여넣기·이미지'를 명시했는데,
제 검증은 텍스트 입력만 다뤘다"는 점을 지적. 이슈 명시 범위를 실측으로 뒷받침하기 위해
**글상자 안 붙여넣기 회귀 테스트**를 추가했다.

## 변경 내용

**파일**: `src/document_core/commands/object_ops.rs` (`issue_1280_textbox_creation_tests`에 추가)

- `paste_text_into_textbox`: 본문 텍스트를 `copy_selection_native`로 복사 → 글상자 안에
  `paste_internal_in_cell_native`로 붙여넣기 → 내부 첫 문단에 텍스트 보존 검증.
  이 경로는 [clipboard.rs:510-512](src/document_core/commands/clipboard.rs#L510)의
  `get_textbox_from_shape_mut` 게이트를 거치므로, 수정 전(text_box 없는 Rectangle)이면 "글상자 없음"으로
  실패한다. #1280의 shapeType 수정이 **글상자 안 텍스트 붙여넣기**를 정상화함을 고정한다.

## 검증

```
test ...::create_textbox_has_textbox ... ok
test ...::create_rectangle_has_no_textbox ... ok
test ...::insert_text_into_created_textbox ... ok
test ...::paste_text_into_textbox ... ok
test result: ok. 4 passed; 0 failed
```
`cargo fmt --check`: object_ops.rs 내용 포맷 차이 0건(잔여는 저장소 전역 CRLF newline 아티팩트).

## ⚠️ 부수 발견 — 글상자 안 *이미지* 붙여넣기는 별개 결함으로 실패 (#1280 범위 밖)

처음에는 `paste_image_into_textbox`(본문 이미지 copy_control → 글상자에 붙여넣기) 테스트를 작성했으나
**실패**했다. 원인 추적 결과:

- 글상자/셀 붙여넣기는 `paste_internal_in_cell_native` → `paste_paragraphs_into_cell_paragraphs`
  ([clipboard.rs:462](src/document_core/commands/clipboard.rs#L462))로 처리되는데, 컨트롤을 가진
  클립보드 문단을 **`merge_from`**으로 병합한다.
- `merge_from` ([paragraph.rs:716](src/model/paragraph.rs#L716))은 (a) `other.text`가 비면 early return하고,
  (b) 애초에 `controls`/`ctrl_data_records`를 **병합하는 코드가 없다**. copy_control은 `text=""` +
  `controls=[Picture]` 문단을 만들므로(clipboard.rs:280), 이미지 컨트롤이 **조용히 누락**된다(에러 없음).

→ **#1280의 shapeType 수정만으로는 "글상자 안 이미지 붙여넣기"가 동작하지 않는다.** text_box는 생겨
"글상자 없음"은 사라지지만, 이미지 컨트롤은 `merge_from`에서 드롭된다. 이는 텍스트 입력/텍스트 붙여넣기와
**다른 근본 원인**(merge_from 컨트롤 누락)이며, 본문 이미지 복사·붙여넣기 무음 실패와 **같은 결함군**이다.
따라서 본 테스트는 텍스트 붙여넣기만 검증하고, 이미지 붙여넣기는 **별도 이슈**로 분리한다.

## 결론

- #1280(shapeType='textbox')는 글상자 **생성 + 커서 진입 + 텍스트 입력 + 텍스트 붙여넣기**를 정상화한다(검증 완료).
- 글상자 안 **이미지 붙여넣기**와 본문 **treat_as_char 이미지 복사·붙여넣기**는 별개 결함
  (`merge_from` 컨트롤 누락 / `paste_control_native` 인라인 삽입 부재)으로, 별도 이슈에서 다룬다.

## 승인 대기

본 보고서와 테스트 커밋 후, 별도 이슈 등록 및 #1280 최종 보고서 진행.

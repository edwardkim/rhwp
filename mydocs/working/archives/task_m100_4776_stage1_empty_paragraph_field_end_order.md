# Task M100 #4776 Stage 1 - 빈 문단 fieldEnd 순서 보정

## 목적

PR #4776이 `FieldRange::inner_slot_count`로 표·그림을 감싼 0-텍스트 누름틀의 종료 위치를
복원하지만, 빈 문단 전용 HWPX 직렬화 경로는 모든 컨트롤 슬롯 뒤에 `fieldEnd`를 일괄
출력하고 있었다. 이 경우 같은 문단의 연속 또는 중첩 누름틀은 종료 마커 순서가 뒤바뀐다.

## 원인

`render_runs`의 `para.text.is_empty()` 분기는 슬롯을 모두 소비한 뒤 `field_ranges`를 순회했다.
예를 들어 `field A -> table A -> field B -> table B`는 `fieldEnd A`와 `fieldEnd B`가 둘 다
문단 말미로 이동한다. 재파서는 field marker를 스택으로 짝지으므로 범위와 `fieldid`가 교차한다.

## 보정

빈 문단 슬롯 루프도 일반 본문 경로와 같은 조건을 사용한다.

```text
fieldRange.control_idx + fieldRange.inner_slot_count == emitted_control_idx
```

해당 슬롯을 출력한 직후에만 `fieldEnd`를 출력해, 인접 필드와 중첩 필드 모두 원래의 닫는
순서를 보존한다. 슬롯 축과 맞지 않는 범위는 기존 말미 fallback으로 남겨 보수적으로 복원한다.

## 검증 상태

- 코드 검토로 빈 문단의 슬롯별 종료 순서를 확인했다.
- 이 Stage에서는 검증 명령을 실행하지 않았다. PR #4776의 기존 CI와 후속 회귀 검증에서
  연속·중첩 0-텍스트 필드 표본을 포함해야 한다.

## 변경 파일

- `src/serializer/hwpx/section.rs`
- `mydocs/working/task_m100_4776_stage1_empty_paragraph_field_end_order.md`

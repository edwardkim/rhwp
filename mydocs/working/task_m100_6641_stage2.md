# Task M100 #6641 Stage 2 — 소유자 인식 재조판 정정

- **실행일**: 2026-09-03 KST
- **입력 계약**: `424f5e58d`, `92ad62a1c`
- **제품 파일**: `src/document_core/queries/field_query.rs`
- **가드 파일**: `tests/issue_2724_passthrough_invalidation_guard.rs`

## 1. 구현 결과

### 가상 셀 fieldId 수렴

`collect_all_fields`가 HWPX 셀 이름에 부여한 합성 `fieldId`는 이미 조회 API에 공개된다.
`set_field_value_by_id`도 `ctrl_id == 0`을 판별해 by-name 경로와 동일한
`set_cell_field_text`로 수렴하도록 고쳤다. 실제 ClickHere는 기존 `field_ranges` 교체를 유지한다.

가상 셀 편집도 section `raw_stream`을 직접 무효화한다. 이 때문에 #2724 passthrough 가드에서
`set_field_value_by_id -> set_field_text_at` 위임을 전제로 두었던 과거 면제는 더 이상 사실이 아니며,
래칫 지시대로 면제 항목 한 건을 제거했다.

### silent reflow 실패 차단

공용 `reflow_cell_paragraph_by_path`는 여러 렌더·편집 경로가 사용하는 반환값 없는 helper이므로
시그니처를 광범위하게 바꾸지 않았다. field setter 경계에서 다음을 수행한다.

1. mutation 전에 `FieldLocation`의 본문 또는 중첩 소유 문단이 실제로 존재하는지 검증한다.
2. `NestedEntry::TableCell`과 `NestedEntry::TextBox`를 손실 없이 cell path로 옮긴다.
3. 기존 본문/중첩 reflow와 vpos 재계산을 실행한다.
4. reflow 뒤 소유 문단의 LineSeg가 비어 있으면 정상 성공 대신 `InvalidField`를 반환한다.
5. 가상 셀에 편집할 첫 문단이 없을 때 종전처럼 무변경 `Ok(())`를 반환하지 않고 오류를 반환한다.

이 방식은 field query의 공개 성공 의미만 강화하며 renderer·serializer·verify 정책을 바꾸지 않는다.

## 2. 보호 불변식 대사

| 불변식 | 결과 |
| --- | --- |
| HWP5 본문 실제 frame reflow | 유지 |
| 표 셀·글상자 최내곽 폭 reflow | 유지 |
| 본문 및 컨테이너 vpos 재계산 | 유지 |
| 표 outer control dirty 처리 | 유지 |
| section raw stream 무효화 | by-id 가상 셀까지 직접 보장 |
| 필드 range·offset·채움 비트 | 기존 교체 helper 무변경 |
| HWPX 합성 LineSeg 저장 정책 | serializer 무변경 |
| verify diff/exit 정책 | 무변경 |
| Gym task/reference/oracle | 무변경 |

## 3. 집중 검증

```text
issue_838_field_set_value                 6/6 PASS
batch_fill_contract                     25/25 PASS
issue_3380_field_value_equals_guide       3/3 PASS
issue_3545_clickhere_dirty_roundtrip      8/8 PASS
issue_2724_passthrough_invalidation_guard 5/5 PASS
```

`batch_fill_contract::verify_accepts_a_roundtripped_field_fill`은 원래 #6641 수용 기준인
정상 HWP5 필드 채움의 exit 0, `verify.identical=true`, `verify.diffCount=0`을 통과했다.

파생 integration suite는 review checkout에서만 준비했으며 source diff에 포함하지 않는다.
Stage 4의 전체 lint·integration은 별도 승인 전 실행하지 않는다.

## 4. 별도 후속 후보

깊은 HWPX 필드 편집은 owner reflow에 성공하지만 합성 LineSeg를 파일에서 생략하는 #5847 정책과
strict `edit_verify_report` 사이에서 `ParagraphLinesegs expected=1 actual=0`을 보고할 수 있다.
이는 field query 전용 문제가 아니라 HWPX 편집 저장 정규형 비교의 일반 문제다. #6641에서 허용
목록이나 serializer 우회를 추가하지 않았으며, 신규 GitHub 이슈 등록은 별도 mutation 승인을 받는다.

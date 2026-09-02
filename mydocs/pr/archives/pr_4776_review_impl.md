# PR #4776 구현 및 maintainer 보정 기록

## 커밋 계보

| 순서 | 원격 커밋 | 내용 |
| --- | --- | --- |
| 1 | `e7b641279` | 0자 원본 텍스트 축 판정 보류 |
| 2 | `14740104a` | HWP3 완성형 좌표 한자·기호 복원 |
| 3 | `154b38259` | OWPML 열거 밖 FieldType 방출 차단 |
| 4 | `a3e1c16f7` | 숨은 설명·누름틀 범위·소프트 하이픈 저장 보정 |
| 5 | `22f639e1e` | IR field sweep baseline 갱신 |
| 6 | `9f6a51a01` | maintainer: 빈 문단 누름틀 종료 순서 보정 |

## maintainer 보정 내용

원본 구현은 일반 텍스트 경로에서 `FieldRange::inner_slot_count`를 사용했지만,
`Paragraph::text`가 비어 있는 별도 경로는 모든 컨트롤을 출력한 뒤 모든 `fieldEnd`를
출력했다. 이 차이로 같은 문단의 연속 필드와 중첩 필드가 잘못 닫힐 수 있었다.

보정은 빈 문단 슬롯 루프에서 다음 순서를 강제한다.

```text
control slot 출력
-> 해당 control index와 일치하는 0-텍스트 field range 탐색
-> fieldEnd 즉시 출력
```

`field_ranges`는 파서가 실제 종료 순서대로 기록하므로, 중첩 필드는 안쪽부터 닫히며
연속 필드는 다음 `fieldBegin` 전에 닫힌다. 슬롯 축과 정합하지 않는 손상·미지원 입력은
기존 말미 fallback을 통해 최소 복원한다.

## 배포

보정은 외부 기여 브랜치 `planet6897/fix/loadsave-hwpx-content-loss-20260814`에 일반
fast-forward push했다. 원래 기여 커밋은 다시 쓰지 않았고 force push를 사용하지 않았다.

## 검증 근거

- GitHub full CI, CodeQL, Canvas visual diff 통과: [PR review](pr_4776_review.md)
- 구현 단계 계약: [Stage 1](../../working/task_m100_4776_stage1_empty_paragraph_field_end_order.md)
- docs-only 후속 head는 fast-pass와 최신 mergeability 확인이 필요하다.

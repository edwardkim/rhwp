# 15 — 축 선택 (fill-fields vs set-cell vs replace-text)

`form_filling_guide.md` §0 의 표다. 이 스킬은 첫 줄만 끝까지 수행한다.

| 서식의 실체 | 축 | 판별 |
| --- | --- | --- |
| 누름틀(클릭 입력 칸) | `fill-fields` / `batch fill` | `fields --json` fieldCount ≥ 1 |
| 누름틀 없이 표의 빈 칸 | `set-cell` | fieldCount 0 + export-tables 빈 셀 |
| 완성 문서의 문구만 | `replace-text` | 위 둘이 아니고 대상이 본문 문자열 |

## 판별 순서

```
fields --json
  fieldCount >= 1  → 이 스킬
  fieldCount == 0  → export-tables --json
                       빈 셀 있음 → rhwp-table-exchange
                       빈 셀 없음 → "문구 치환이면 safe-edit 의 replace-text"
```

추측으로 set-cell 좌표를 만들지 않는다. 좌표는 export-tables 의
index/row/col 과 같아야 한다.

## 섞인 서식

한 문서에 누름틀과 맨 셀이 같이 있으면 축을 나눈다.

1. 이 스킬로 누름틀 채움 (`-o` 중간 산출)
2. table-exchange 가 중간 산출을 입력으로 set-cell
3. 필요하면 replace-text (연도·기관명)
4. 마지막 sanitize 는 이 스킬

중간 산출을 원본 자리에 덮지 않는다.

## set-cell 을 여기서 하지 않는 이유

set-cell 은 병합 칸 실패·overflow·keep-style 등 다른 계약이다.
rhwp-table-exchange 가 책임진다. 이 스킬이 격자 좌표 채움을 복제하면
ambiguous 와 overflow 를 한 봉투에서 섞어 오독한다.

## replace-text 주의

반복 필드 14칸을 replace-text 로 "성명" → 값 치환하면 라벨까지
바뀌거나 첫 출현만 바뀐다. 누름틀이 있으면 fill-fields 가 맞다.

## 이 스킬을 닫는 한 줄

`fieldCount: 0` 이면 SKILL.md 를 더 읽지 말고 인계한다.

# 06 좌표 규칙 — 없는 page 를 만들지 않는다

정본: playbook §1-2, §4. 코드: `copy_coords`.

## 키 집합

봉투가 줄 수 있는 좌표 키는 이 일곱이다.

`section` · `paragraph` · `page` · `charOffset` · `length` · `cell` · `textbox`

엔진은 이 집합의 교집합만 복사한다. 집합 밖 키(`line`, `column`,
`pdfPage`, `humanPage`)를 만들지 않는다.

## page 는 0 기준, 선택 필드

- 있으면 **0 기준** 정수다. 사람이 보는 1쪽 = 봉투 `page: 0`.
- 없으면 키 자체가 없다. `page: null` 도 발명이다.
- 1을 더해 "사람이 읽기 쉽게" 고치는 순간 재독이 깨진다. SWS L1
  재독은 봉투 좌표로 `search` 를 다시 친다.

## 언제 page 가 빠지나

실측에서 자주 빠지는 자리:

- 아직 조판에 배치되지 않은 문단
- 일부 머리말/꼬리말/숨은 설명
- 필드 가이드 텍스트
- 페이지 모델이 없는 최소 문서

빠진 것을 전쪽 검색으로 "아마 3쪽"이라고 메우지 않는다. 인용 라벨은
있는 키만 나열한다.

```
과업지시서.hwp (section=0, paragraph=41, charOffset=120)
```

`page` 가 있을 때만 `page=3` 을 붙인다.

## cell · textbox

표 칸 매치는 `cell` 객체/경로가 붙을 수 있다. 글상자 매치는 `textbox`.
이것도 있으면 복사, 없으면 생략. 표 좌표를 `paragraph` 로 환산하지 않는다.

## 재독 절차

1. EV 의 `command` 를 그대로 실행한다.
2. 나온 매치의 좌표 키가 EV 와 같은지 본다.
3. `quote` 가 그 좌표의 텍스트와 맞는지 본다.
4. 어긋나면 대장을 손으로 고치지 않는다. 엔진을 다시 돈다.

예제: [examples/04_missing_page_omitted.md](../examples/04_missing_page_omitted.md),
[examples/18_coordinate_quote_reread.md](../examples/18_coordinate_quote_reread.md),
[examples/21_cell_coords_table.md](../examples/21_cell_coords_table.md),
[examples/22_textbox_coords.md](../examples/22_textbox_coords.md).

다음: [07_search_extract_envelopes.md](07_search_extract_envelopes.md).

# 08 — stem_blocks 와 boxed 보기

`stem` 한 줄로 부족한 문항은 `stem_blocks` 시퀀스를 쓴다.
빌더는 `stem_blocks` 가 비어 있지 않으면 그것을 우선하고, `stem` 은 fallback.

## 블록 순서 = 인쇄 순서

```
text        → 발문
boxed       → <보기>
image       → 그래프 (between 이면 선택지 앞)
choices     → (문항 필드. 블록이 아님)
```

발문 없이 그림만 있는 문항도 있다. 그때는 image 가 첫 블록이고
`stem` 에는 짧은 발문을 남겨 스키마 required 를 채운다.

## boxed — <보기>

원본이 테두리 상자 안의 보조 자료면 boxed 다.

```json
{
  "type": "boxed",
  "title": "<보기>",
  "blocks": [
    {"type": "text", "text": "ㄱ. 주어와 서술어가 호응한다."},
    {"type": "text", "text": "ㄴ. 수식어와 피수식어가 가깝다."}
  ]
}
```

`title` 예: `"<보기>"`, `"[보기]"`, `"<자료>"`. 원본 표기를 따른다.
보기 안에 그림이 있으면 `blocks` 에 image 를 넣을 수 있다 (중첩 허용).

```json
{
  "type": "boxed",
  "title": "<보기>",
  "blocks": [
    {"type": "text", "text": "다음 실험 장치를 보고 답하시오."},
    {"type": "image", "ref": "img/q8_lab.png", "placement": "between"}
  ]
}
```

## boxed 에 text 를 직접 주지 말 것

```json
{"type": "boxed", "text": "소속: 성명:"}
```

이 형태는 #3358 이전에는 빈 상자가 조용히 생겼다. 지금은
`boxed 블록에 허용되지 않는 필드 'text'` 로 실패한다.
본문은 항상 `blocks`.

## 보기가 아닌 것

| 원본 | 블록 |
| --- | --- |
| 그냥 긴 지문 (테두리 없음) | `text` 또는 `passages` |
| 선택지 ①–⑤ | `choices[]` 이지 boxed 가 아님 |
| 답안 표 | 이 스킬 범위 밖. 넣지 않음 |
| 머리말 "홀수형" | `form_label` |

## stem 과 첫 text 블록

둘을 같게 맞춘다. 어긋나면 사람이 dump 를 볼 때 헷갈린다.

```json
"stem": "다음 보기를 참고하여 ㉠에 들어갈 말로 적절한 것은?",
"stem_blocks": [
  {"type": "text", "text": "다음 보기를 참고하여 ㉠에 들어갈 말로 적절한 것은?"},
  {"type": "boxed", "title": "<보기>", "blocks": […]}
]
```

첫 text 에 `"12. 다음 보기를…"` 처럼 번호를 넣었으면 `auto_number: false`.

## 중첩 깊이

보기 안에 보기 (`<보기>` 안의 `<자료>`) 는 원본이 그렇게 생겼을 때만.
3단 이상 중첩은 그리지 말고 안쪽을 이미지로 crop 한다 (한계: 정밀 박스).

픽스처: `fixtures/schemas/valid_boxed_bogi.json`,
`fixtures/schemas/invalid_boxed_text_field.json`.

# 07 — passages 와 questions

수능·모의고사 국어/영어는 한 지문을 여러 문항이 공유한다.
지문을 문항마다 복제하지 않는다. top-level `passages[]` 에 한 번 쓰고
`passage_ref` 로 가리킨다.

## 규칙

- `passages[].id` 는 문서 안에서 유일해야 한다. 권장: `p1-3`, `p4-7`.
- `questions[].passage_ref` 는 그 id 와 문자열 일치.
- 빌더는 **같은 passage_ref 를 처음 만나는 위치** 에 지문을 한 번만 출력한다.
  문항 2, 3 앞에 지문이 다시 나오지 않는다.
- 지시문 `[1~3] 다음 글을 읽고 물음에 답하시오.` 는 passage 의 첫 text 블록.
  문항 stem 에 넣지 않는다. 넣으면 문항마다 반복된다.
- 지문이 한 문항 전용이면 `passages` 를 쓰지 말고 그 문항 `stem_blocks` 에 둔다.

## 좋은 예

```json
{
  "passages": [
    {
      "id": "p1-3",
      "blocks": [
        {"type": "text", "text": "[1~3] 다음 글을 읽고 물음에 답하시오."},
        {"type": "text", "text": "환경 오염은 현대 사회의 중요한 문제 중 하나이다. …"}
      ]
    }
  ],
  "questions": [
    {
      "number": 1,
      "passage_ref": "p1-3",
      "stem": "윗글의 주제로 가장 적절한 것은?",
      "auto_number": true,
      "choices": [
        {"label": "①", "text": "환경 보호의 중요성"},
        {"label": "②", "text": "도시 생활의 편리"},
        {"label": "③", "text": "전통 음식의 역사"},
        {"label": "④", "text": "기술 발전 동향"},
        {"label": "⑤", "text": "진로 탐색"}
      ]
    },
    {
      "number": 2,
      "passage_ref": "p1-3",
      "stem": "밑줄 친 ㉠의 의미로 적절한 것은?",
      "auto_number": true,
      "choices": [
        {"label": "①", "text": "원인"},
        {"label": "②", "text": "결과"},
        {"label": "③", "text": "예시"},
        {"label": "④", "text": "대조"},
        {"label": "⑤", "text": "비유"}
      ]
    }
  ]
}
```

## 나쁜 예

문항 1, 2, 3 의 `stem_blocks` 에 같은 긴 글을 세 번 붙인다.
산출 HWPX 가 지문을 세 번 인쇄한다.

`passage_ref: "1-3"` 인데 id 는 `"p1-3"`. 빌더는 지문을 못 찾고
문항만 출력한다. 오타는 스키마가 잡지 않는다 (문자열이면 통과).
에이전트가 id 를 대조해야 한다.

## 영어 장문

영어 장문은 문단이 여러 개다. passage.blocks 를 문단마다 text 로 나눈다.
한 문자열에 `\\n\\n` 을 욱여넣지 않는다. 빈 문단을 만들지 않는다.

지문 안의 밑줄 ㉠㉡ 은 텍스트 그대로 둔다. 특수 필드가 없다.

## 수학 — 지문 공유가 드묾

수학은 보통 문항 독립. `passages` 를 비운다. 공통 자료가 있으면
국어와 같은 규칙을 쓴다.

## 선택지 라벨

권장 라벨은 원문자 `①` `②` `③` `④` `⑤` (U+2460–U+2464).
`1)` `①.` `(1)` 을 섞지 않는다. 원본이 `1.` 이면 원본을 따른다.
`label` 과 `text` 를 합쳐 `"① 환경 보호"` 로 `text` 에 넣지 않는다.
빌더가 label 을 다시 붙일 수 있다.

문항당 선택지 개수는 스키마가 5 로 강제하지 않는다. 4지선다·5지선다를
원본대로 쓴다.

픽스처: `fixtures/schemas/valid_shared_passage.json`.

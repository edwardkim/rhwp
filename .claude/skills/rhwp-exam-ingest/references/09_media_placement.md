# 09 — media placement

이미지가 **어디에 놓이는지** 는 픽셀이 아니라 의미다.
CLI는 실제 이미지 바이트와 Picture/BinData를 연결한다. 각 배치는 본문 흐름에 참여한다.

스키마 enum: `between` | `above` | `below` | `inline`. 기본 `between`.

`stem_blocks` 의 image 블록과 `media[]` 항목 **둘 다** placement 를 가질 수 있다.
블록의 placement가 우선한다. 블록에서 참조한 media는 다시 자동 삽입하지 않는다.
블록에서 참조하지 않은 media만 해당 문항의 placement에 한 번 자동 삽입한다.
공유 지문·boxed의 between/below는 해당 컨테이너의 끝에 놓는다.

## between (기본, 가장 흔함)

발문 → **그림** → 선택지.

> 다음 그래프를 보고 물음에 답하시오.
> [그래프]
> ① … ② …

수능 과학·수학 그래프, 국어 만화 한 컷, 영어 도표의 대부분.

## above

그림이 발문보다 위. 지시문 없이 자료가 먼저 오는 학원 자체 시험지.

> [지도]
> 위 지도에 대한 설명으로 옳은 것은?
> ① …

`stem_blocks` 순서도 image 가 text 보다 앞이어야 한다. placement 만
`above` 이고 블록 순서가 반대면 사람이 읽기 어렵다. **순서와 enum 을 함께**.

## below

선택지 다음에 자료. 드물다. 원본이 그렇게 생겼을 때만.

> 다음 중 옳은 것은?
> ① … ⑤ …
> [보충 자료]

`below` 를 "대충 아래쪽" 이라는 시각적 느낌으로 쓰지 않는다.
선택지 **다음** 이다.

## inline

문장 흐름 안의 작은 그림. 보기 기호 옆의 작은 도형, 빈칸에 들어갈 그림.

> 다음 중 ㉠에 해당하는 것은? [작은 도형] 을 고르시오.

image 블록의 inline은 같은 컨테이너의 앞 문단 끝에 붙는다. 앞 문단이 없으면
자체 문단을 사용하며 boxed 뒤 이미지를 그 boxed 내부에 붙이지 않는다.
정확한 문장 중간은 `{"type":"paragraph","runs":[{"type":"text","text":"앞 "},
{"type":"image","ref":"img/q1.png"},{"type":"text","text":" 뒤"}]}`로 지정한다.
출력의 Picture 참조·글자 순서와 실제 한컴 표시를 확인한다.

## 고르는 질문

1. 그림이 선택지보다 위이고 발문보다 아래인가? → `between`
2. 그림이 발문보다 위인가? → `above` + 블록 순서 앞
3. 그림이 선택지보다 아래인가? → `below`
4. 그림이 문장 중간에 끼는가? → `paragraph.runs`

애매하면 `between`. 기본값이 그 이유다.

## media 와 crop

placement 는 조립 위치다. crop bbox 와 무관하다.
bbox 는 페이지에서 **무엇을 잘라 낼지**, placement 는 잘라 낸 조각을
**문항 어디에 둘지**.

```
page_003.png  (2480×3508)
  crop  180, 620, 2100, 900  →  img/q11_graph.png
  media.placement = between
```

페이지 전체가 between 으로 들어가면 안 된다.

픽스처: `fixtures/matrices/placement.json`,
`fixtures/schemas/valid_media_{between,above,below,inline}.json`.

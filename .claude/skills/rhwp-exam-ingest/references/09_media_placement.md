# 09 — media placement

이미지가 **어디에 놓이는지** 는 픽셀이 아니라 의미다.
스키마 enum: `between` | `above` | `below` | `inline`. 기본 `between`.

`stem_blocks` 의 image 블록과 `media[]` 항목 **둘 다** placement 를 가질 수 있다.
둘을 같게 맞춘다. 어긋나면 빌더 구현에 따라 한쪽이 이긴다. 이 스킬은
빌더를 바꾸지 않고, 두 값을 동일하게 써서 불확실성을 없앤다.

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

한컴 Picture inline 직렬화가 약하면 (#182) inline 은 깨질 수 있다.
중요한 자료는 `between` 으로 내려 독립 문단처럼 넣는 편이 안전하다.
inline 을 썼으면 한계를 사용자에게 한 줄로 고지한다.

## 고르는 질문

1. 그림이 선택지보다 위이고 발문보다 아래인가? → `between`
2. 그림이 발문보다 위인가? → `above` + 블록 순서 앞
3. 그림이 선택지보다 아래인가? → `below`
4. 그림이 문장 중간에 끼는가? → `inline` (한계 고지)

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

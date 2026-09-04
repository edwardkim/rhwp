---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-08-02
---

# Task #3738 Stage 2 — 이월된 그림 23의 page-local picture anchor 분석

- 선행 commit: `7c2fe9d65` (`fix: HWP 부동 표 앵커 흐름 정합`)
- 선행 결과: HWP p23의 그림 23 조기 배치는 해소했지만 p24 상단 잘림 발생
- 기준 증적: [Stage 1 visual sweep](task_m100_3738_stage1_visual_sweep.md)

## 재현된 잔여 결함

Stage 1의 HWP p24 render tree에서 `pi=344` 표는 새 페이지 body 상단에 배치됐다.

```text
Table pi=344 bbox: x=94.5, y=90.6, w=600.9, h=490.4
Image (그림 23) bbox: x=120.9, y=-181.4, w=495.6, h=330.5
```

표 상자는 p24에 있지만 내부 그림은 page frame보다 181.4px 위에 있다. 따라서 Stage 1의
이월 판단은 맞지만, 그림 자체가 이전 저장 좌표계의 음수 상대 위치를 그대로 사용해 잘린다.
한컴 PDF p24는 그림 23 전체를 본문 상단에 배치한 뒤 EU 문단과 표 4를 잇는다.

## 원인 가설

실제 방출 경로는 `table_layout.rs`의 `layout_horizontal_cell_paragraphs`다. 이 경로는
`TopAndBottom + Para + flow_with_text` picture의 최종 y를 `content_top + picture.vertical_offset`
으로 다시 정한다. 따라서 이월된 표의 page-local `content_top`에는 맞지만, picture의 HWP5
`vertical_offset`에 남은 이전 page 기준 상쇄를 별도 판정하지 않아 음수 y가 된다.

이 표의 outer host와 inner picture는 이전 physical ladder를 상쇄하는 값을 갖는다.

- outer host `pi=344` 저장 vpos: `52230 HU`
- 다음 guide 문단 `pi=345` vpos: `30689 HU` (되감김)
- parent 표 offset: `+560 HU`
- 내부 picture offset은 signed `-52790 HU`
- 합계: `52230 + 560 - 52790 = 0 HU`

즉 내부 picture의 큰 음수 offset은 p23 기준 상대 이동이 아니라, 다음 physical page의 상단을
가리키는 상쇄 표현이다. paginator가 표를 fresh page로 이월한 뒤에는 이 값을 다시 적용하면 안
된다. 이 `0 HU` 합성 anchor를 page-local origin으로 정규화해야 한다.

## 다음 수정 경계

다음 수정은 아래 사실을 모두 확인한 parent table cell picture에만 한정한다.

1. native HWP5, 빈 host, 1×1 비-TAC `TopAndBottom`·`Para` `RowBreak` 표
2. 표가 다음 문단의 저장 vpos 되감김 때문에 fresh page에 이월됨
3. 셀 내부 picture가 있고 `host_vpos + table_vertical_offset + picture_vertical_offset`이
   page origin 근처(작은 허용오차)에 있음

이 경우 picture의 stale negative transform/offset을 새 page body 기준으로 정규화한다. 일반
셀 그림, 실제 음수 위치 그림, HWPX stored layout, 또는 큰 그림의 page-internal split에는 적용하지
않는다.

## Stage 2 구현 반증 — 조건이 실제 노드에 닿지 않음

Stage 2의 첫 구현은 위 합성식을 셀 paragraph의 첫 `LINE_SEG`에서 읽었다. 그러나 source dump의
실제 계층은 다음과 같다.

```text
outer paragraph pi=344: LINE_SEG vpos=52230
  table: RowBreak, 1×1, non-TAC TopAndBottom/Para, table offset=+560
    cell paragraph p[0]: LINE_SEG vpos=0
      picture: TopAndBottom/Para/flow_with_text, signed offset=-52790
```

즉 `52230`은 **바깥 host**의 값이며 셀 문단의 값이 아니다. 셀 layout만 받는 첫 구현은 `0 + 560 -
52790`을 검사해 false가 되어, p24 render tree의 Image bbox가 여전히 `y=-181.4`로 남았다. 이 구현은
의도적으로 좁았지만 실제 구조를 잘못 연결한 것으로, 해결 근거가 아니다.

이 residual은 Stage 2 결과·review PNG로 보존해 커밋한다. 다음 Stage 3 분석은 outer host anchor를
layout_table까지 전달해 정확한 합성식을 판정할지, 또는 이미 이월이 확정된 빈 1×1 RowBreak host 안에서
`-52790 HU` 같은 page-scale signed offset을 별도 contract로 판정할지를 새로 결정한다. HWPX는 이
native HWP5 분기의 대상이 아니며 계속 독립 residual로 추적한다.

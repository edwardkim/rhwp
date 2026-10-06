---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-08-02
---

# Task #3738 Stage 3 — outer host anchor를 이월 그림 셀까지 전달

- 선행 commit: `27661e90a` (`docs: #3738 그림 23 stage2 증적 보관`)
- 선행 결과: HWP p23의 그림 23 조기 배치는 계속 해소됐지만, HWP p24 Image bbox는
  `y=-181.4px`이고 frame overflow가 남음
- 기준 증적: [Stage 2 visual sweep](task_m100_3738_stage2_visual_sweep.md)

## 재현 구조

source dump에서 `52230 HU`는 셀 paragraph가 아니라 table을 가진 outer host paragraph의 저장
line position임을 확인했다.

```text
outer paragraph pi=344: LINE_SEG vpos=52230 HU
  table: 1×1, RowBreak, non-TAC TopAndBottom/Para, vertical_offset=+560 HU
    cell paragraph p[0]: LINE_SEG vpos=0 HU
      picture: non-TAC TopAndBottom/Para/flow_with_text, signed vertical_offset=-52790 HU
```

바깥 host 기준으로만 `52230 + 560 - 52790 = 0`이라는 page-boundary 상쇄식이 성립한다. Stage 2의
후보는 셀 내부에서 `vpos`를 읽어 이 값이 0인 탓에 의도적으로 불발했다. 효과 없는 구현은
커밋하지 않았고 증적만 `27661e90a`에 보관했다.

## 수정 경계

`layout_table`의 root-body 호출은 outer `Paragraph`와 table control을 함께 알고 있으나, table-cell
picture 배치(`layout_table_cells` → `layout_horizontal_cell_paragraphs`)에는 현재 `para_y`만 전달한다.
여기에 table-owner의 first implementation `LINE_SEG.vertical_pos`를 `Option<i32>`로 **한 방향만**
전달한다. nested/header/footer/TAC 등 다른 호출자는 `None`을 넘긴다.

셀 picture에서 offset을 0으로 정규화할 조건은 모두 필요하다.

1. native HWP5이며 전달받은 outer host vpos가 양수
2. outer table은 빈 1×1 non-TAC `TopAndBottom`·`Para` `RowBreak`
3. 현재 셀은 빈 paragraph 하나와 `vpos=0` 저장 줄 하나만 가지며, picture는 non-TAC
   `TopAndBottom`·`Para`·`flow_with_text`
4. `host_vpos + signed(table.vertical_offset) + signed(picture.vertical_offset)`이 0 HU 근처

이 네 조건은 page-scale 음수 offset을 의도적 일반 음수 위치와 구분한다. HWPX profile과 nested table
호출에는 host vpos가 전달되지 않아 적용 대상이 아니다.

## 검증 계획

추가 helper의 양성/음성 unit regression을 만들고 전용 target에서 빌드한다. 그 뒤 HWP 및 HWPX 각각
p23–p24, 144 DPI visual sweep을 실행한다. HWP p24에서 그림 23 전체·caption·EU 문단·표 4의 순서와
frame overflow가 해소됐는지를 review PNG와 render tree로 판정한다. 하나라도 residual이면 해당 결과를
새 visual sweep/결과 보고로 보관·커밋하고 다음 번호의 analysis를 시작한다.

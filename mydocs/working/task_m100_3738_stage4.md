---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-08-02
---

# Task #3738 Stage 4 — 그림 23 caption 및 다음 저장 anchor 복원

- 선행 commit: `ce3bd69b3` (`fix: #3738 이월 그림의 outer anchor 복원`)
- 선행 증적: [Stage 3 visual sweep](task_m100_3738_stage3_visual_sweep.md)
- 기준 자료: 개인정보 제거 원본 HWP·HWPX 및 각각의 한컴오피스 2020 PDF
  ([보관 목록과 hash](../../pdf/pr3740/README.md))

## 재현된 잔여 결함

Stage 3는 HWP p24의 graph image를 page frame 안으로 복원했다. 그러나 review PNG에서 그림 23의
caption이 없고 EU 문단·표 4가 기준보다 아래로 밀린다. 한컴 PDF p24의 좌표는 다음과 같다
(PDF point; 96 DPI CSS pixel 환산은 약 4/3배).

| 항목 | 한컴 PDF p24 | Stage 3 rhwp p24 |
| --- | ---: | ---: |
| 그림 23 caption 첫 줄 | y=320.3pt | 미방출 |
| `○ EU에서 …` 첫 줄 | y=406.3pt (약 541.7px) | y=626.9px |

원본 HWPX XML의 같은 picture(`image31`)에는 `BOTTOM` caption이 3개 저장 줄로 존재한다. 그림이 든
empty `RowBreak` table의 HWP5 구조도 이를 보존한다.

```text
outer p344: LINE_SEG.vpos=52230 HU
  1×1 RowBreak table, TopAndBottom/Para, height=18257 HU
    cell height=36782 HU, vertical-align=center
    picture: height=24791 HU, offset=-52790 HU, Bottom caption 3 lines
next p345: LINE_SEG.vpos=30689 HU  (p346=32689, visible p347=34689)
```

## 원인

1. `layout_horizontal_cell_paragraphs`는 셀 안 non-inline `Picture`를 그린 뒤
   `picture.caption`을 렌더하지 않는다. 일반 본문 picture 경로(`layout.rs`)와 footnote picture
   경로에는 caption 처리와 caption bottom flow가 있지만 이 셀 경로에는 없다.
2. Stage 3 이후 graph 자체는 `y=92.5px`에 정확히 복원됐지만, root table은 stale cell height
   `490.4px`를 그대로 반환한다. 이후 두 빈 문단과 p347은 table bottom을 누적 기준으로 삼아
   각각 y=573.6/600.3/626.9px가 된다. 저장된 p345 anchor는 이 이월 page의 실제 다음 flow anchor이며,
   y≈488.4px에서 다시 시작해야 p347이 PDF의 약 541.7px에 온다.

셀 전체 height를 일반적으로 축소하면 cell center·다른 `RowBreak` table·caption 없는 그림의 기존 계약을
깨뜨린다. 따라서 paint frame은 유지하고, 아래 두 좁은 경계만 보정한다.

## 수정 경계

1. 이 실제 HWP5 형상에 필요한 셀 안 non-inline picture의 **Bottom caption만** 일반 본문과 같은 방식으로
   `layout_caption`에 전달한다. caption context는 기존 cell 경로를 보존하며, position은 picture frame과
   caption spacing으로 계산한다. Top/좌우 caption은 이 회차의 실물 근거가 없으므로 범위를 넓히지 않는다.
2. native HWP5에서만 다음을 모두 만족할 때, table을 그린 뒤 **flow cursor만** 다음 문단의 저장 vpos로
   되돌린다.
   - 빈 outer host의 단일 1×1 non-TAC `TopAndBottom`·`Para` `RowBreak` table
   - cell의 단일 빈 picture 문단 및 `Bottom` caption
   - Stage 3의 outer-vpos/page-boundary offset 상쇄식
   - 다음 implementation LINE_SEG가 현재 host vpos보다 작고 page body 안에 있음

frame/cell geometry와 HWPX profile은 건드리지 않는다. 이 조건은 그림을 실제로 이월한 HWP5의 stale
cell height만 flow 예약에서 분리한다.

## 검증 계획

helper 양성·음성 unit regression과 전용 target release build를 실행한다. 이어 HWP/HWPX 각각 p23–p24
144 DPI visual sweep을 완주하고, HWP p24에서는 graph·3줄 caption·EU 문단·표 4의 순서와 좌표를
render tree 및 review PNG로 판정한다. 결과는 새 visual sweep/보고 문서와 review PNG에 보관한다.
HWPX의 독립 residual은 결과에 분리해 기록하며, 남으면 해당 commit 뒤 다음 stage 분석을 시작한다.

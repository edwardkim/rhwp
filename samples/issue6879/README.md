# issue 6879 — TAC 형제가 있는 자리차지 float 의 앵커 줄

## 무엇을 잠그나

한 문단에 **글자처럼 취급(TAC) 표**와 **비-TAC 자리차지 float 표**가 형제로 달렸을 때,
둘의 **차례**와 float 의 **세로 기준점**을 잠근다.

## 형상

7쪽(0-based `6`) 문단 73 에 표가 둘 달려 있다.

```text
  ci=0  '붙임' 라벨   treatAsChar=1  vOff=0      47675 × 3014 HU  (635.7 × 40.2px)
  ci=1  내용 표       treatAsChar=0  vOff=2512   47624 × 60816 HU (635.0 × 810.9px)
                      wrap=TOP_AND_BOTTOM · vertRelTo=PARA
```

저장 사다리가 두 줄을 적어 두었다. `textpos=8` 은 첫 인라인 컨트롤(8 슬롯) 다음이므로
**float 의 제어 문자는 줄1** 이다.

```text
  줄0  vertpos = 0      vertsize = 3580 (47.7px)   textpos = 0   ← TAC 라벨이 타는 줄
  줄1  vertpos = 4060   vertsize = 1500 (20.0px)   textpos = 8   ← float 앵커
  float vOff = 2512 (33.5px)
```

## 수정 전 증상

```text
  수정 전   ci=1 y=128.0 (h 810.9)  ·  ci=0 y=909.1 (h 40.2)
            → 라벨이 흐름 끝까지 밀려 내용 표와 29.8px 겹친다
  수정 후   ci=0 y= 98.2 .. 138.4   ·  ci=1 y=182.1
```

## 정본

`pdf/156767332-broadcast-revenue-attachment-2020.pdf` (engine 2020 — 저장 제품이
`hancom-office-2022` 라 규약 §3.5.1 의 2020 버킷).

```text
              rhwp            정본
  붙임 라벨   98.2 .. 138.4   글자 y = 107.5
  그림1       184.0           185.6      Δ 1.6
  그림2       575.4           576.5      Δ 1.1
  총 8쪽
```

## 고친 자리

`vertOffset` 의 기준점이 앵커 줄이라는 `#6860` 의 계약을 두 곳에 더 걸었다.

1. `typeset.rs` `#5807` 판별식 — `v_off` 대신 `앵커오프셋 + v_off` 를 TAC 줄 높이와
   견준다. 이 값이 줄 높이보다 작으면 "겹침"으로 보고 float 을 TAC 앞에 놓는데,
   문단 상단 기준으로는 2512 < 3580 이라 오판했다(앵커 기준 6572 ≥ 3580).
2. `layout.rs` `para_y_for_table` · `typeset.rs` `place_table_with_text` — float 의
   세로 원점을 앵커 줄만큼 내린다.

`layout.rs` 의 앵커 줄 사상에는 **글자 없는 문단** 폴백이 필요했다. 이 문단은 `hp:t` 가
하나도 없어 `char_offsets` 가 비고, `control_text_positions` 의 char 축(0,1)을 저장
`text_start` 의 HWP5 UTF-16 축(0,8)에 사상할 수 없다. 인라인 개체가 축을 정확히 8유닛씩
차지하므로 **앞선 인라인 개체 수 × 8** 로 잡는다.

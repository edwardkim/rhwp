# Stage 1 보고서 — Task #1079: 조사 (그림 pushdown ↔ 파일 vpos 이중 계상)

- 브랜치: `local/task1079` (소스 무변경)

## 파일 vpos 는 1페이지에 정합
`RHWP_TYPESET_DRIFT` (pr-149 page1) — 문단 first_vpos(상대 px):
```
pi=1 원본:   21.3   pi=2 그림  251.9   pi=3 회색조: 273.3
pi=4 그림    503.9  pi=5 흑백: 525.2   pi=6 그림   755.8
pi=7 입니다. 777.0
```
- 라벨↔그림↔라벨 블록 간격 ≈ 252px(그림 209px + 라벨/여백). **입니다. 파일 vpos=777px <
  body 876.9px → 파일 vpos 그대로면 1페이지 수용**(한컴 PDF p1 정합).
- 즉 한컴/파일은 그림이 차지한 공간을 **다음 문단 vpos 에 이미 반영**(회색조 273px = 원본 그림
  아래).

## rhwp 의 이중 계상
`cur_h` 누적(같은 진단): pi=3 cur_h=482.6 (파일 vpos 273.3 + 그림 209). pi=7 cur_h=986.5.
- **typeset**(`typeset.rs:1311~1352`, Task #409 v2): 비-TAC TopAndBottom + vert=Para 그림에
  `pushdown_h = 그림높이 + margin.bottom`(≈209px) 을 current_height 에 가산.
- **VPOS_CORR**(`vpos_snap_current_height` 1497, HeightCursor::vpos_adjust): current_height 를
  파일 vpos 로 sync — 파일 vpos 는 **이미 그림 공간 포함**.
- → 그림 공간이 (vpos-sync) + (pushdown_h) 로 **두 번** 누적 → used 986.5 > body 876.9 → 109px
  초과 + 2페이지.
- **렌더**도 동일: pushdown OFF 실험 시 typeset 은 1페이지가 되나 렌더가 여전히 overflow
  (pi=7 y=1140) → 렌더(`picture_footnote.rs` y_offset+total_height)도 파일 vpos 에 그림 높이를
  더 얹음. 즉 **typeset·렌더 양 레이어 이중 계상**.

## 실험 (pushdown 단독 OFF) — 불충분
typeset pushdown 만 끄면 페이지 수는 1로 정정되나 렌더 overflow 가 오히려 증가(pi=7/8). 렌더
pushdown 이 독립적이라 단독 변경은 typeset↔렌더 불일치. **양 레이어 정합 수정 필요.**

## #409 와의 구분 (회귀 핵심)
Task #409 v2 pushdown 은 **파일 vpos 가 그림 공간을 반영하지 않는** 케이스(다음 문단 vpos 가
그림만큼 점프하지 않음 — 21페이지/AI 184p)에 필요. pr-149 는 **파일 vpos 가 이미 반영**.
→ 수정 = "다음 문단 vpos 가 이미 그림 공간을 포함하면 pushdown 생략"(조건부). 무조건 제거는
#409 회귀.

## 수정 방향 (Stage 2 설계 대상)
- 그림 다음 문단의 first_vpos 가 (그림 para vpos + 그림 높이) 이상이면 파일 vpos 가 이미 그림을
  반영 → typeset pushdown + 렌더 pushdown 생략(파일 vpos 사용). 아니면 현행 pushdown 유지.
- 양 레이어(typeset.rs pushdown_h + 렌더 picture pushdown) 동일 조건 적용.

## 리스크
- pushdown 은 그림/도형(TopAndBottom, vert=Para) 보유 전 문서 공유 → 광범위 회귀. Stage 2
  비회귀 케이스(#409 계열) 명시 + Stage 3 전수 sweep 단계 측정.

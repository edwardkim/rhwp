---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 5 — HWPX 그림 11 이월이 만드는 그림 23 page drift

- 선행 commit: `b116c7b1f` (`fix: #3738 그림 caption과 이월 flow 복원`)
- 기준 자료: 개인정보 제거 원본 HWP·HWPX와 각각의 한컴오피스 2020 PDF
  ([경로·SHA-256·Git/LFS 판정](../../pdf/pr3740/README.md))
- 직전 시각 증적: [Stage 4 visual sweep](task_m100_3738_stage4_visual_sweep.md)

## 최초 분기점

Stage 4의 HWPX p23–p24 비교에서 그림 23만 고립해 고치면 안 된다. 전체 페이지별 text와
`dump-pages`를 역추적하면 최초 불일치는 renderer index 13(문서 하단 표기 `- 14 -`)의 그림 11이다.

| 자료 | p273(그림 11 표)의 renderer index | 문서 하단 표기 |
| --- | ---: | --- |
| HWP rhwp | 13 | `- 14 -` |
| HWPX rhwp | 14 | `- 15 -` |
| 한컴 HWP/HWPX PDF | 13 | `- 14 -` |

따라서 HWPX는 그림 11을 한 쪽 늦게 이월하고, 그림 12·20·21·22·23도 같은 방향으로 밀린다.
Stage 4 HWPX p24에 그림 21/22와 그림 23이 함께 보인 현상은 이 선행 이월의 결과이며, 그림 23
자체의 caption/offset 결함으로 단정할 수 없다.

## 저장 구조와 계산 대조

두 입력의 IR은 그림 11 주변에서 같은 저장 geometry를 가진다. 전체 `ir-diff`는 문단 0의 cc/offset
한 차이만 보고했고, p273과 그 다음 p274의 표·line-seg 값은 같다.

```text
p273: empty host, 1 table, TopAndBottom / Para / RowBreak
  host LINE_SEG vpos = 45803 HU
  table common: 41954 × 17819 HU, vertical offset = 948 HU
  next p274 LINE_SEG vpos = 65136 HU
```

원본 HWPX XML도 outer `hp:tbl`의 `textWrap="TOP_AND_BOTTOM"`, `hp:pos treatAsChar="0"
flowWithText="1" vertRelTo="PARA" vertOffset="948"`를 보존한다. HWPX parser는 해당
`pageBreak="CELL"` 저장 표기를 IR의 `RowBreak`로 정규화한다.

`RHWP_TABLE_DRIFT=1`으로 HWPX p273을 관측하면 일반 block 경로는 footnote 예약 후
`current_height=620.3px`, `table_total=241.4px`, `available=857.0px`를 사용한다. 합계가
`861.7px`가 되어 4.7px 초과로 새 쪽으로 이월한다. 그러나 저장 anchor를 쓰면
`45803 HU → 610.7px`이고 선언 높이 `237.6px`의 bottom은 `848.3px`라 같은 쪽에 안전하게
들어간다. HWP는 이미 이 후자의 lane 예약 경로를 사용한다.

## 원인과 좁은 수정 경계

`try_typeset_empty_para_float_table`의 `stored_single_topbottom_top`은 정확히 이 형상
(빈 host, 단일 TopAndBottom Para float, 다음 저장 vpos 증가, 선언 bottom이 page body 안)를
판별하지만 `native_hwp5_layout()`으로만 막고 있다. HWPX 원본도 같은 저장 anchor를 제공하지만
이 게이트 밖이라 일반 block fit으로 떨어진다.

수정은 기존 형상 조건·선언-height lane 예약을 바꾸지 않고, **native HWP5 또는 original HWPX
stored-layout**에만 그 gate를 열어 동일한 저장 anchor를 쓰게 한다. 페이지 anchor 없는 HWPX,
복수 float, text host, next vpos reset, 선언 bottom 초과 표는 기존 block 경로에 남긴다.

## 검증 결과

전용 release-test binary에서 HWPX p273은 renderer index 13에 남았고, p274부터의 후속 문단은 index
14로 이어졌다. 기준 PDF와의 144 DPI visual sweep은 최초 분기인 p13–p15와 사용자 보고 지점 p23–p24에
대해 각각 3/3, 2/2 페이지를 완료했다. p13–p15에는 visual flag가 없으며, p14의 그림 11 위치도 기준과
같은 페이지에 복원됐다.

다만 이 보정만으로 HWPX 그림 23이 해결된 것은 아니다. p344 그림 23 표는 아직 renderer index 22(문서
23쪽)에 `y=548.9px`로 남고, 내부 image는 `y=276.9px`로 위로 튄다. 그래서 index 23(문서 24쪽)에는
그림 23 image/caption이 없다. HWP의 같은 p344는 index 23에서 image `y=92.5px`로 시작한다. 이 별도
`RowBreak` reset 형상은 다음 Stage에서 새 분석 대상으로 분리한다. 모든 review PNG와 원본 HWP·HWPX·PDF의
보관 경로는 연결된 sweep 및 기준 자료 문서에 남겼다.

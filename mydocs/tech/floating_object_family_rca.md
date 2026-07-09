# 부동개체(자리차지) 계열 공통 근본원인 조사

대상 잔여: **#2004**(셀-내 부동 이미지 스택 156714340), **#1921**(부동 표 밀도/2단 59043).
(#1880/3235145는 `tac=true` inline이라 **계열 제외** — 별도 오버플로 razor.)

## 1. 공통 근본원인 (한 문장)

rhwp에는 **부동개체의 (진짜 세로 extent 예약) + (컨테이너 내 non-overlap 캐스케이드/패킹)을 담당하는 *일반* 레이아웃 패스가 없다.** 대신 단일 SSOT 술어 `non_inline_control_flow_height`가 **Square/Tight/Through wrap에 flow 높이 0을 반환**하고, non-overlap 캐스케이드·true-extent 예약은 **~6개의 좁은 특수-케이스 게이트**로 패치돼 있다. 게이트 조건 밖의 geometry(varying-offset 셀 이미지, 콘텐츠 보유 부동 표)가 한글과 발산한다.

## 2. SSOT 술어 (footprint 예약의 심장)

`non_inline_control_flow_height` — **`table_layout.rs:486` + `height_measurer.rs:285`에 중복**:
- `treat_as_char || !TopAndBottom` → **0 반환** (Square/Tight/Through 부동개체는 본문 flow **미예약**, exclusion zone으로만 그림).
- `TopAndBottom + Para + flow_with_text=false` → **0**.
- `TopAndBottom + (Page/Paper)` 또는 `+flow_with_text=true` → object height 예약.

→ **Square 부동개체는 본문에서 flow를 전혀 예약하지 않는다**(exclusion으로 텍스트만 밀어냄). 세로 스택/캐스케이드는 SSOT에 없다.

## 3. 캐스케이드가 "일반"이 아니라 6개 특수 게이트

| 게이트 | 위치 | 발동 조건 | 놓치는 케이스 |
|---|---|---|---|
| #2004 inline 재분류 | `rendering.rs:44` | Square + `!allow_overlap` + **동일 vertical_offset** + count≥2 + 무텍스트 | **156714340**: offset이 0/−3360/−2940… varying → 미발동 |
| #2007 lane 표쌍 | `typeset.rs:11743` | 빈 문단 + 자리차지 표 2개+ | 콘텐츠 보유 표, 셀-내 이미지 |
| #1535 exclusion | `layout.rs:6436` | 공동앵커 부동 표 → 직전 아래로 | 셀 내부 미도달 |
| #1994 절대배치 | `typeset.rs:12455` | Paper-anchored | Para-anchored |
| #2019 overlay anchor | `layout.rs:6262` | 빈 overlay 앵커 | 콘텐츠 보유 |
| 셀 row-height | `table_layout.rs:504` | 셀 내 Square는 **height 예약**(other_h += ) | 아래 4절 참조 |

## 4. 케이스별 정밀 매핑

### #2004 / 156714340 — 셀 reserve/place 비대칭
- 5장 전면 이미지(Square, `!overlap`, **varying offset**)가 1×1 표 셀에 스택.
- #2004 재분류 게이트는 **동일 offset** 요구 → varying이라 미발동 → 부동 유지.
- **셀 row-height 집계**(`cell_non_inline_control_flow_height:504`)는 Square height를 **예약**(`other_h += `, `:563`)하나, **셀 배치**(`table_layout.rs:3207`)는 `para_y += non_inline_control_flow_height`(=**Square는 0**)로 전진 → **5장을 같은 y에 배치(겹침)**. **예약과 배치의 술어 불일치**가 핵심.

### #1921 / 59043 — 부동 표를 2단/흐름에 패킹하는 패스 부재
- 콘텐츠(법령 텍스트·스크린샷) 보유 `자리차지` 표들이 dense/2단 영역에 다수.
- "부동 표를 2단 흐름에 조밀 패킹"하는 코드가 **없음**(#1921 라벨 코드는 vpos-reset·각주마진뿐, 패킹 아님). 각 표가 개별 예약/배치 → **15/48쪽에 얇게 펼침**(한글 37 dense).
- `col_count>1`에서 flow 규칙이 갈림(`flow_advance_height:1951` height_for_fit, 데코 표 flow 유지 `#775:11395`) — 2단 부동 상호작용이 특수분기 누적.

## 5. source-dependent 분기 (HWPX↔HWP 발산의 씨앗)
- `is_hwpx_source`: `typeset.rs:12400`(`declared_empty_para_float_total` 게이트가 HWPX 제외), `layout.rs:6413`(visible-float `v_off≤0`).
- `is_hwp3_variant`: 여러 tolerance/스케일 분기.
→ 동일 IR이라도 source flag로 배치가 갈려 convert-HWP 발산(#1880류)을 만든다.

## 6. 수정 방향 (unified, deep — 각 단계 A/B 필수)

### ⚠️ 실증 결과 (2026-07-09): cell reserve/place 캐스케이드는 **무효**
당초 "가장 bounded한 착수점"으로 **셀 콘텐츠 높이에 Square 스택 캐스케이드분 예약 + `table_layout.rs:3207` placement para_y 전진**을 구현·검증했으나 **실패**:
- 156714340 **4→5**(목표 8), 5장 **여전히 p4 한 쪽 겹침** + **spurious near-empty p5** 추가(악화). 전체 2945/0 무회귀였으나 목표 미달 → revert.
- **이유**: 부동(Square tac=false) 이미지는 **셀 앵커에 절대배치**되어 셀 높이를 키워도 캐스케이드 안 됨. 부동 이미지는 flow에 없어 **intra-cell 분할(흐름 cut)이 이들을 페이지로 못 나눔**. → reserve/place 일관화만으로는 불충분.

### ⚠️ 실증 2·3차 (2026-07-09): 재분류-into-cells 도 **무효** — blocker는 표 레벨
재분류를 (a) 셀 재귀 + (b) offset 게이트 varying 허용으로 확장 구현·검증:
- 셀 스택 **검출·재분류 적용 확인**(debug match=true)이나 무효(4쪽). 셀 composition이 빈-텍스트 이미지 문단을 1줄로 붕괴(본문 nc 줄합성 `rendering.rs:3113` 부재).
- 스택 셀문단을 이미지 1장 inline 문단 N개로 **분할**해도 무효 — **pi=42 표 871.9px 불변**.

**정밀 확정된 근본 blocker**: `156714340` pi=42 는 **`wrap=TopAndBottom tac=false` 부동 표**. 부동 표는 `non_inline_control_flow_height`로 **저장 높이(871px)에 원자 배치**되고 **셀 콘텐츠로 재측정되지 않는다**. → 이미지/셀 레벨 수정(재분류·분할·캐스케이드)은 **전부 무효**, blocker가 **표 레벨**(부동 표가 콘텐츠 미분할)에 있음.

### ✅ 확정 정답: 부동 표 콘텐츠 페이지네이션 (= #1921 수렴)
**#2004/156714340 잔여 = #1921 동일 뿌리**(부동 표 콘텐츠 미분할). 유효 수정:
- 부동 TopAndBottom 표를 **콘텐츠 높이로 재측정 + RowBreak/intra-cell 분할**, 또는 전면-이미지-부동표를 **inline 승격**(표 자체를 flow에 태움).
- = #1921 "부동 표 흐름 패킹/분할 모델 신설" 대형 과제. 이미지 재분류 경로는 **배제 확정**(3차 실증).

## 7. 결론
세 케이스(실질 2건)는 **동일 뿌리**: 부동개체(특히 부동 **표**)가 flow에 콘텐츠를 예약/분할하지 않고 저장 높이에 원자 배치된다. **#2004 잔여(156714340)와 #1921(59043)은 실증으로 동일 근본**(부동 표 콘텐츠 미분할)으로 수렴 확인. 무효 경로 3종(cell reserve/place, 재분류 셀재귀, 셀문단 분할) 전부 배제. **유효 경로 = 부동 표 콘텐츠 페이지네이션 모델 신설**(대형 과제, 전용 세션). reserve/place·재분류는 이미지/셀 레벨이라 표-레벨 blocker를 못 넘음.

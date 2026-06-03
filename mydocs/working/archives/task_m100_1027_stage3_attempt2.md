# Stage 3 구현 2차 — #1027: typeset.rs vpos 앵커 (메커니즘 확정, 과보정)

- 타스크: #1027 / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 단계: Stage 3 — 올바른 파일(typeset.rs)에 vpos 앵커 구현·검증

## 1. 구현

`TypesetState` 에 `page_vpos_base: Option<i32>` 추가(생성자/`reset_for_new_page` 정합) + `typeset_paragraph`(1439) 의 fit(1596) 직전에 앵커:
- 단단(col==1), 페이지 첫 항목이 base 설정, 이후 항목 `current_height = (first_vpos − base)px`.
- bypass: TAC 수식/그림/글앞뒤 Shape, vpos-reset(비-첫 항목 vpos==0).

## 2. 검증 결과 (비공개 184p)

| 지표 | baseline | vpos 앵커 | 판정 |
|------|----------|-----------|------|
| "추진일정은" 노트 | 9쪽 | **8쪽** | ✅ 목표 달성(메커니즘 확정) |
| 총 페이지 수 | 185 | **168** | ❌ −17 (과밀) |
| LAYOUT_OVERFLOW | 12 | **136** | ❌ +124 (대규모 회귀) |

→ **되돌림**(엔진 클린). 노트는 올바른 파일(typeset.rs)에서 이동 확인 — 수정 surface 정확.

## 3. 과보정 원인

순수 vpos 앵커는 **대부분 단락에서 vpos < 렌더러 실제 위치** → current_height 를 과도하게 낮춰 페이지 과밀 → overflow 136건.

근본: 렌더러 VPOS_CORR 는 **raw vpos 를 그대로 쓰지 않고 클램프**한다(예: #643 백워드 8px 허용, lazy_base/page_base 보정, #1022 trailing_ls 정정). 노트는 우연히 vpos 가 렌더 위치와 일치(43.6px 전부 따라감)했으나, 일반 단락은 renderer 가 vpos 만큼 안 따라간다(클램프).

→ 페이지네이터가 렌더러와 일치하려면 **raw vpos 가 아니라 렌더러의 클램프된 VPOS_CORR 결과값**을 써야 한다.

## 4. 다음 설계 (Stage 3-3)

renderer VPOS_CORR(`layout.rs:2152~2625`) 의 클램프 규칙을 페이지네이터(typeset)와 공유:
- 옵션 1: VPOS_CORR 의 보정량 계산을 공용 함수로 추출 → typeset fit 에서 동일 적용.
- 옵션 2: 앵커를 **제한적 down-sync** 로 — 보정량을 renderer 클램프 범위(예: 누적 대비 일정 한계, 또는 직전 항목이 표/개체일 때만)로 묶어 과보정 차단.
- 회귀 게이트: LAYOUT_OVERFLOW ≤ 12, 페이지 수 185 유지, svg_snapshot 무회귀.

노트 이동(목표)과 무회귀를 동시에 만족하는 클램프 규칙이 핵심.

# Task #1658 v3 최종 보고서 — 라운드 5: 페이지 하단 고정 표 "하단 배타 예약" (패턴 B)

## 요약

#1653 RCA가 확정하고 D안(보류)으로 넘긴 **패턴 B** — `vert=쪽·valign=Bottom·자리차지`
표(결재/서명 틀)의 over-pagination — 를 해소했다. 한글 실측으로 정답 모델을
"하단 스택"에서 **"하단 절대배치(겹침 허용) + 본문 하단 배타 영역(max-union)"** 으로
수정하고, 기존 #1611 경로에 통합 구현했다.

- **통제셋 92: 일치 73→75(+2), over 5→3, under 무회귀, 악화 0** — #1653 Stage 2b
  (일괄 out-of-flow, 일치 75→55 대규모 회귀)가 실패했던 지점을 통과.
- 이슈: #1658 라운드 5 / 브랜치: `local/task1658-bottomres`
  (통합 베이스 + #1748 + #1591 v2)
- 수정: `float_placement.rs`(판별), `typeset.rs`(#1611 블록 재작성),
  `table_layout.rs`(unwrap 절대 y) — 3파일.

## 모델 (1단계 한글 실측, `task_m100_1658_v3_stage1.md`)

관악 36389312(하단 틀 2개, 247+357px)에서 두 틀의 높이 합(604px)이 본문 텍스트
(577px)와 동시 수용 불가인데 한글은 1쪽 → 스택 모델 반증. 성립 모델:

- 하단 고정 틀은 본문 하단에 **절대배치**(vertOffset=0 다수는 서로 겹침, 성긴
  틀이라 내용이 시각적으로 인터리브).
- 본문 텍스트는 **하단 배타 영역(= max(틀 높이))** 위까지만 흐른다
  (1084.7−357.2=727.5 ≥ 577.4 ✓ 1쪽).
- 본문이 배타 영역을 이미 침범한 페이지면 틀을 다음 쪽에 단독 배치
  (36387725, #1611 — 한글 2쪽).

## rhwp 결함 2종과 수정 (2단계, `task_m100_1658_v3_stage2.md`)

1. **typeset flow 소비**: 틀 높이가 문서순으로 `current_height` 에 누적 → 둘째 틀이
   다음 쪽으로 (+1쪽). → **#1611 블록을 배타 모델로 재작성**: 소비 롤백 + 배타
   높이(`available_height` 차감, max-union) + **vpos 동기화에 선행 틀 소비 차감
   보정**(한글 저장 vpos 는 틀도 문서순 누적하므로 미보정 시 둘째 틀이 spurious
   이월). #1611 이월 판정·#1624 plausibility 가드 계승.
2. **렌더 절대배치 소실**: 1×1 래퍼 unwrap 이 외곽 틀의 Page·Bottom 절대 y 를
   버리고 내부 표를 flow 커서에 렌더(p2 상단 59~271pt). → 외곽이 Page/Paper 앵커
   자리차지면 `compute_table_y_position` 절대 y 를 내부 표 시작점으로 사용.

### 본 라운드 내 1차 시도 교훈

별도 배타 분기(vpos 동기화 미계승)는 36387725 를 2→1쪽으로 악화시켰다(#1611 RED).
한글의 이월 판정은 저장 vpos 기준 본문 끝(+62px 드리프트 보정)을 필요로 한다 —
기존 경로 통합이 정답이었다.

## 검증 (3단계, `task_m100_1658_v3_stage3.md`)

| 게이트 | 결과 |
|--------|------|
| 통제셋 92 (파일별 대조) | 일치 **73→75**, over 5→3, under 14 불변, **악화 0** ✅ |
| 대형 오라클 452 | 443 (98.0%) 불변 ✅ |
| clipping (92) / valign fixture | 회귀 0 / BUG 0 ✅ |
| byeolpyo 4/26 · giant 42 · scattered 53 | 무회귀 ✅ |
| #1611 가드 | 통과 (36387725 2쪽 유지) ✅ |
| 신규 가드 3건 (`issue_1658_page_bottom_fixed_exclusion`) + opengov 22건 | 통과 ✅ |
| `cargo test --release` 전체 | **2754 passed / 실질 실패 0** (7건 svg CRLF 노이즈 #1786, 내용 diff 0) ✅ |

## 산출물

- 소스: `src/renderer/float_placement.rs`, `src/renderer/typeset.rs`,
  `src/renderer/layout/table_layout.rs`
- 테스트: `tests/issue_1658_page_bottom_fixed_exclusion.rs` (3건)
- 재현 샘플: `samples/hwpx/opengov/` 편입 2건 (36389312 관악·36398366 디지털도시국,
  공개 결재문서 36~38KB) + 스냅샷 행
- 문서: 수행·구현 계획서, stage1~3 보고, 본 보고서

## #1658 잔여 지도 (라운드 5 이후)

| 갈래 | 상태 |
|------|------|
| 거대 셀 분할 (R1·R2) | landed (별표4 Δ+2 안전 ceiling) |
| valign over-count (R3) | landed |
| byeolpyo4 클리핑 코어 (R4) | **비-viable 확정·트랙 종결** (floor·render-feedback·render-fidelity 3접근) |
| **패턴 B 하단 고정 표 (R5, 본 라운드)** | **landed** — 통제셋 over 지배 원인 해소 |
| 패턴 A 텍스트 sb/sa 드리프트 | razor-thin 프로그램(#1759·#1760·#1763·#1769)·saved-vpos(#1772)와 범위 통합 — 별도 트랙 진행 중 |
| 잔여 over 3건 (+1×1, +2×2) | 하단 틀 무관 별개 원인 — 후속 분류 |

# Stage 2 보고서 (증분 1) — Task #1257/#1184: 순차-flow 골격 + 진단

- 이슈: edwardkim/rhwp#1257 · 브랜치: `local/task1257`
- 설계: `mydocs/tech/archive/endnote_seq_flow_redesign.md`

## 증분 1 — note-metrics 헬퍼 + 진단 (동작 무변경)

- `endnote_visible_span_hu(paragraphs)`: 미주 가시 세로 span(HU) = max(seg bottom)−min(seg top).
  내부 비단조(2D) vpos 흡수.
- `RHWP_EN_SEQ_FLOW` 플래그 게이트 진단(`EN_SEQ_DIAG`): 미주별 `seq_span_px`(순차 모델) vs
  `cur_footprint_px`(현 모델 current_height 증가분), 분할 여부.

## 측정 (3-09월_교육_통합_2022.hwpx)

비-분할 미주 24개:
- 평균 |cur−seq| = **3.6px**, 최대 **20.5px**.
- 차이>15px = 4개, 모두 **≈+20px**(= between-notes 갭 분량, 현 모델은 갭이 footprint 에 포함).

→ **순차 note_span + between_notes_gap ≈ 현 모델 footprint** (수 px 이내) 정량 확인.
하이브리드 순차-flow 모델이 비-분할 미주에서 현 동작 재현 가능. 남은 복잡성은 분할(broke)
미주(컬럼/페이지 경계 + ~15 특례)에 집중.

## 검증
- 빌드 OK, `cargo test --lib` 1540 passed, issue_1139 43 passed (진단은 플래그 게이트, 무영향).

## 증분 2 — 렌더 측 순차 배치 실험 (revert, 발견 보존)

`vpos_adjust`가 미주 흐름에서 순차 `y_offset` 그대로 반환(절대-vpos 분기 우회)하도록 플래그
게이트 실험. 측정:
- **문5: 1051→1067 (갭 복원, fixed)**, 문6→문7=287 유지.
- **오버플로우는 회귀 아님**(베이스라인이 더 많음): 3-11 off 8건/on 3건, 10월 off 6/on 2.
  큰 오버플로우(3-11 1530px)는 #1184 사전 결함(비단조 미주), seq-flow 가 오히려 감소시킴.
- **그러나 문26(p18 731→699)·문29(p19 430→403)는 위로 이동(갭 축소)** — 순차 렌더는
  line_spacing 이 운반하는 갭(문5/6/7)만 보존하고, **다줄/forward 케이스의 갭은 떨어뜨림**.

**발견(증분 3 지침):** 렌더-only 변경은 불충분. between-notes 갭을 **typeset 순차 flow(=모든
미주 경계의 current_height + 렌더 소비 line flow)에 일관 주입**해야 한다. 현재는 직전 문단
line_spacing 주입이 일부 경계(단일 줄 prev)에만 갭을 실어, 다줄/forward 경계에선 갭이 없다.
→ 증분 3: 미주 경계마다 between_notes_gap 을 순차 누적에 명시 포함(typeset).

증분 2 렌더 변경은 mixed 결과라 **revert**(트리는 증분 1 진단만 유지). 발견만 본 보고서에 보존.

## 증분 3 — typeset 측 순차 vpos 덮어쓰기 실험 (revert, 발견 보존)

시도: ① 미주 경계에서 between-notes 갭을 `current_height` 에 명시 누적, ② full para vpos 를
단-상대 순차 좌표(`top_hu + 내부상대`)로 덮어써 render 가 절대-vpos 보정 없이 정합하게.

- **페이지수 패리티 유지**(갭의 current_height 누적은 안전).
- **그러나 오버플로우 급증**(3-09 2→16건). 원인: typeset vpos 덮어쓰기가 render 의 base/anchor
  좌표계(첫 미주 para vpos 기준)와 불일치 + split para(절대 vpos 유지)와 혼재 → 오배치.

**발견(방향 수정):** typeset 에서 vpos 를 덮어써 render 좌표계와 싸우는 건 틀린 길.
**증분 2(렌더 측 순차)가 더 옳은 방향**(오버플로우 감소·단순). 미충족 잔여 = 다줄/forward 경계의
갭이 render `y_offset` 누적에 없음 = render 가 **다줄 prev 마지막 seg line_spacing(주입된 갭)을
누락**하기 때문. → **증분 4: render `build_single_column` 의 y 누적이 다줄 prev trailing
line_spacing 을 존중**하도록(+ 증분2 vpos_adjust→y_offset). typeset vpos 불간섭.

증분 3 typeset 변경 revert(트리 증분1 유지).

## 증분 4 — 조사 (다줄/forward 갭 누락 원인, 코드 변경 없음)

증분2가 옳은 방향이나 다줄/forward 경계 갭 누락의 정확한 지점 추적. **5개 메커니즘 제거**:
1. `vpos_adjust` 절대-vpos 분기 — 증분2가 우회.
2. inter-para gap(`layout.rs` 4669-4682) — last-seg line_spacing 을 정상적으로 더함(누락 아님).
3. `hidden_empty_paras` — engine.rs(Paginator fallback) 및 typeset 3161 hide 는 **본문 para 한정**,
   미주 emission 루프(2050-2720)와 무관 → **미주 빈 separator 는 hidden 처리 안 됨**.

→ 잔여 원인 = **빈 미주 para 의 render 배치/높이 측정 자체**(composition 빈줄 높이 또는 vpos 경로).
아직 단일 지점으로 미특정.

## 현실 평가 (체크포인트)

증분 1~4 결과: #1184 render 재설계는 **다층(vpos_adjust·composition·empty-line·좌표계·split)이
얽힌 대형 다세션 rewrite**다. 각 증분이 새 층을 드러냄(layer-by-layer). 단일 세션 probing 으로는
비효율적이고 half-state 회귀 위험. **권장: 전용 집중 세션에서 `build_single_column` 의 미주
배치·높이 모델을 통째로 재작성**(설계서 §2 순차-flow 모델 기준), 매 증분 페이지수 패리티 게이트.

확정 성과: 증분1 진단·헬퍼(36b9564e), 설계서 확정, POC·증분2/3/4 발견 모두 커밋.
#1256(PR #1259)이 명확 케이스(문6/7/12/16/18/19/20/21/23/25/26/27/28) 처리. 잔여(문5/24/26-미적/
29 등 + 2023/10월 동류)는 본 재설계 완료 시 해소.

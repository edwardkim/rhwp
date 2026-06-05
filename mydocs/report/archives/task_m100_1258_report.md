# 최종 결과보고서 — Task #1258: typeset 미주 base-flow trailing IR 명시 (A 정규화)

- **이슈**: edwardkim/rhwp#1258
- **브랜치**: `feature/issue-1258-trailing-base-flow-normalize` (base `stream/devel`)
- **선행**: #1248 조사, 메모리 `tech-trailing-model-no-ssot`
- **결과**: **중단(defer).** 통일 가능성·이득을 실증한 뒤, 결합 변경의 이득이 위험 대비 불확실하여 종료.
- **코드 변경**: 없음 (Stage 3 시도분 복원, baseline green 유지)

## 1. 목표 (재확인)

typeset 의 "base-flow 1984HU 는 vpos 에 이미 있다" 가정을 단일줄·다줄 모두 성립시켜 render 의
between-notes 재구성 특례(S8 계열)를 제거/축소.

## 2. 수행 경과

| 단계 | 내용 | 결과 |
|------|------|------|
| Stage 1 | 재현 가드 (테스트 2004 green + 문22/미주사이20 렌더 y baseline) | 완료 |
| Stage 2 | spike — 비대칭 드롭 지점 특정 (단일줄 `:4523` 포함 vs 다줄 `:4509` 제로) + 구현안 B 채택 | 완료 |
| Stage 3 | B 단순 적용 → **이중가산 8건 회귀** → 복원 + 진단 | 중단 결정 |

## 3. 핵심 결론 (실측 기반)

### 3.1 비대칭의 정확한 위치
다줄 미주 문단만 `endnote_line_vpos_base` 경로(`paragraph_layout.rs:1612`)를 타며, 문제경계 마지막 줄
trailing 을 **0**(`:4509`)으로 둔다. 단일줄은 `:4523` 에서 trailing 포함. pagination 은 양쪽 모두
포함(para_height=sum(lh+ls)) → **다줄 미주-final 에서 layout↔pagination ~1ls drift**가 상시 존재하고
height_cursor S8 가 이를 보정.

### 3.2 B 가 분리 불가인 이유 (Stage 2 가설 반증)
`#1246` 발화 조건 `stored_gap_px = result − y_offset ∈ (−0.5,4.0)` 는 **상대값**. render 4509 로 y_offset 을
내려도 vpos 기반 result 가 함께 내려가 gap 은 여전히 ≈0 → #1246 이 계속 발화 → render + #1246 **이중가산**.
실측: 문22 484.3→510.77px(gap 26.5→52.9, 정확히 2배) + overflow 15.3px.

### 3.3 가치 판단
결합 변경(4509 + #1246 제거 + 다줄 base-shift 가드)을 해도 결과는 **gap 제공 위치 이동**(height_cursor→
render) + **여전히 base-shift 가드 필요**. 순 특례 감소 불확실 → **구조적 대청소가 아닌 측면 이동(lateral
move)**. 첫 시도 8건 회귀로 #1248 의 "즉시 가치 낮음·위험 중간" 평가가 **실측 확인**.

## 4. 결정

**중단(defer).** #1258 의 본래 목적(통일 가능성·이득 실증)은 달성. 현 S8 계열(#1246/#1256/#1261)은
**검증된 동작 상태**이므로 유지한다. A 정규화는 측면 이동이라 비용 대비 이득이 낮아 착수하지 않는다.

## 5. 향후 지침 (재발 방지)

- 미주 between-notes 신규 버그는 이 영역(D/E 게이트 포함)에 **새 게이트 추가**로 대응하는 것이 현실적.
- 만약 결합 B 를 재시도한다면 **4509 + #1246 제거 + 다줄 base-shift 를 한 커밋에서** 하고, 본 보고서의
  렌더 y baseline(문22 484.3 / 미주사이20 문10·11·12)으로 즉시 대조할 것.
- 진단은 메모리 `tech-trailing-model-no-ssot` 에 반영(option B 측면이동·#1246 상대 gap).

## 6. 검증

- 최종 작업트리: **clean**, baseline 회귀 없음 (height_cursor 34, issue_1139 46, issue_1082 4).
- 산출물: 계획서 2 + 단계보고서 3(Stage1~3) + 본 최종보고서. 코드 0줄 변경.

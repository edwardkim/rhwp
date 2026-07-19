---
kind: decision
status: active
canonical: mydocs/tech/rendering_engine_design.md
last_verified: 2026-07-17
---

# 설계: 공유 측정 엔진 (페이지네이터 ↔ 렌더러 y-advance 정합)

- 관련: #1027 Stage 3 결론. #1022(측정 정합), #993(분할 표) 후속.
- 작성일: 2026-05-20

## 1. 문제

페이지 분할(`typeset.rs TypesetEngine`)과 렌더링(`layout.rs`)이 단락 y-advance 를 **다른 방식**으로 계산한다.

- **페이지네이터**: `current_height += fmt.total_height`(= sb + Σ(lh+ls) + sa). 단락당 +sb(+sa) drift.
- **렌더러**: y_offset 에 조건부 sb(non-col-top, `layout.rs:3496`) + 줄 advance 누적 후 **VPOS_CORR**(`layout.rs:2150~2480`)로 LINE_SEG vpos 에 스냅(단계당 ≤8px 백워드 클램프 `MAX_BACKWARD_PX`, prev_vpos_end/base/lazy_base, bypass: TAC 수식/그림/Shape, vpos-reset, stale table host).

→ 두 측정 공간 불일치로 콘텐츠가 한컴과 다른 쪽에 배치(#1027: 페이지 8 노트 43.6px 과측정 → 9쪽). 단일 공식/앵커 보정은 단락마다 formula↔vpos 관계가 달라 실패(Stage 3 시도 매트릭스 참조).

## 2. 해법: 공유 y-커서

렌더러의 **per-item y-advance + VPOS_CORR 로직을 단일 루틴으로 추출**하여 렌더러(그리기)와 페이지네이터(분할 판정)가 **동일 측정**을 쓰게 한다. 페이지네이터는 "그리지 않는 height-only 패스".

### 공유 대상
1. **per-paragraph advance**: 조건부 spacing_before(col-top 제외) + Σ(lh+ls) + outer_margin(TAC).
2. **VPOS_CORR**: `vpos_correct(current_y, prev_vpos_end, curr_first_vpos, base, curr_sb, col_y, col_h) -> Option<corrected_y>` (≤8px 백워드 클램프, 본문 내, stale-table-host 가드). 상태: prev_vpos_end / page_base / lazy_base.
3. **bypass 판정**: TAC 수식/그림/글앞뒤 Shape, vpos-reset(line>0 && vpos==0).

## 3. 단계별 구현 (대규모 — 각 단계 골든/테스트 게이트)

- **Stage A (무동작 추출)**: `layout.rs` VPOS_CORR 의 클램프·base 계산을 **순수 함수 `vpos_correct(...)`** 로 추출. 렌더러는 그 함수를 호출(동작 동일). 단위 테스트로 parity 확인. → 회귀 0 목표.
- **Stage B**: per-paragraph advance(sb 조건/줄/outer) 계산을 공유 함수로 추출. 렌더러 무동작.
- **Stage C**: `HeightCursor`(height-only) 구현 — A·B 함수로 y 만 추적. 렌더러 y_offset 진행과 1:1 parity 단위 테스트(여러 샘플).
- **Stage D**: `typeset.rs` 누적/ fit 을 `HeightCursor` 로 교체(단단 우선). 검증: 노트 8쪽 + LAYOUT_OVERFLOW ≤ 12 + 페이지수 + svg_snapshot(공개 골든) 무회귀.
- **Stage E**: 표/다단/partial/atomic 확장. 비공개 184p sweep + 광범위 회귀.
- **Stage F**: 골든 재판정(한컴 PDF), 최종 보고.

## 4. 리스크
- **최고 위험**: 두 엔진(layout 5476줄 + typeset)의 핵심을 건드림. Stage A·B 를 **무동작 추출 + parity 테스트**로 선행해 위험을 격리하는 것이 필수.
- 광범위 골든 재판정 불가피 → 한컴 2022 PDF 대조.
- 다단/표/footnote/zone 의 상태 상호작용이 복잡 → Stage E 에서 집중.

## 5. 권고
별도 이슈(M100, "공유 측정 엔진")로 격상하여 Stage A 부터 **무동작 추출 + parity 테스트** 중심으로 신중 진행. #1027 은 본 설계로 근본 해법을 확정하고, 노트/SFR-008 은 그 리팩터 완료 시 해소된다.

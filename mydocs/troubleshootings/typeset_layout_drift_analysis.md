# typeset / layout drift 분석 — Task #331 revert 회고

- **관련 이슈**: #331 (문단 trailing line_spacing 누적 drift)
- **작성일**: 2026-04-25
- **결과**: 단순 fix 불가 판정, 전체 revert 후 신규 이슈 #332 (예정) 로 단계적 통합 작업

---

## 시도한 fix 와 실패 경로

### Stage 1+2 (commit `af80f52`, revert `078717f`)

- typeset advance: `cur_h += fmt.total_height` → `+= fmt.height_for_fit`
- layout per-paragraph advance: 마지막 visible 줄에서 trail_ls 제외
- Golden 2 개 baseline 갱신, lib 5 개 테스트 calibration

**효과**:
- 의도: pi=26 + 보기 ①②③ 가 page 1 col 1 에 fit (PDF 일치) ✓
- 회귀: page 1 col 0 하단에 pi=10 partial 의 lines 0/1/2 가 동일 y(`col_bottom - lh`)에 piling → 글자 겹침 ✗

### 회귀 원인 체인

1. **layout drift**: 우리 폰트 메트릭(KoPub 등) 의 line_height 가 HWP 원본 폰트와 미세 차이 → 매 줄 layout y 가 typeset cur_h 보다 약 0.5~3 px 앞서감
2. **header 영역 별도 측정**: 표/Shape 영역의 layout 측정과 typeset 의 추정이 어긋나 시작점부터 ~17 px drift
3. **vpos correction 한방향 검증**: `layout.rs:1392` 의 `end_y >= y_offset - 1.0` 조건이 layout 이 vpos 보다 앞설 때 보정 미적용 → drift 누적
4. **typeset 정합 가정 깨짐**: typeset 은 cur_h=1154.3 / avail=1226.4 → 72px 잔여 판단, pi=10 partial 0..3 (3 lines, 63px) 배치
5. **layout 현실 부족**: 실제 col 0 잔여 ~10 px → pi=10 line 0 만 겨우 fit, lines 1/2 overflow
6. **clamp pile 버그** (`paragraph_layout.rs:807-816`): overflow 시 `text_y = col_bottom - lh` + `y = clamped` 로 리셋 → 후속 lines 도 동일 위치 → **글자 겹침**

### 시도한 추가 수정 (모두 회귀)

| 수정 | 결과 |
|------|------|
| layout 의 partial 마지막 visible 줄 trail_ls 제외 (전 partial) | 효과 없음 — vpos correction 이 다시 ls 더함 |
| vpos correction 의 `vpos_end` 에서 trail_ls 제거 | 부분 효과 — pi=10 line 0 만 깨끗, lines 1/2 여전히 pile |
| vpos correction 양방향 (`end_y >= y_offset - 1.0` 제거) | col 1 의 pi=10/pi=11 가 같은 y 로 collapse (회귀) |
| clamp 시 break (overflow 라인 그리지 않음) | typeset 이 그 라인을 다음 단에 발행하지 않은 케이스 → 콘텐츠 손실 |

---

## 근본 해결안 비교

### A. **단일 모델 통합** (점진적, 권장)

모든 advance/positioning 을 `height_for_fit` 모델로 통일:

1. typeset advance = height_for_fit (Task #331 의 1단계)
2. layout per-paragraph advance = height_for_fit (Task #331 의 2단계)
3. vpos correction 의 `vpos_end` 에서 trail_ls 제외, 또는 양방향 보정 (collapse 방지 검증 포함)
4. clamp pile 버그 → "stop drawing on overflow" 로 변경 + typeset 에 overflow signal 발행
5. header (표/Shape) drift 검증 — 표 매니저의 측정과 typeset 동기화

규모: 중. 위험: vpos correction 변경 시 HWP-호환 케이스 회귀 가능 → 광범위 회귀 테스트 필수.

### B. **재 pagination 루프**

layout 이 overflow 감지 → typeset 에 signal back → 재배치 → fixed point 까지 반복. 정확하지만 아키텍처 변경 (renderer 파이프라인 재설계).

### C. **HWP 폰트 충실도**

원본 폰트의 정확한 메트릭 사용 → drift 자체 발생 안 함. 폰트 가용성/라이선스 문제로 완전 해결 불가.

---

## 결정

**A 를 단계적으로 진행** — 신규 이슈 #332 로 5 개 sub-task 분해, 각 단계 회귀 테스트 보강 후 적용. 마지막 단계 완료 시 #331 의도(`pi=26+보기 fit`) 자연 해결.

**Task #321 검증 중 발견된 #331 상태**: 그대로 유지 (revert 됨). 기존 동작은 pi=26 → page 2 (PDF 와 불일치) 이지만, 글자 겹침은 없음.

---

## 교훈

- typeset 과 layout 이 다른 모델로 advance 를 누적하고 있다는 점은 사전에 알려진 적 없음 (`compute_hwp_used_height` 내 diff 로그가 힌트)
- 한 쪽만 수정하면 반대편의 buffer 가 사라져 latent 버그 노출
- pre-existing 측정 다중성(typeset, layout, vpos correction, header measurer) 이 누적 drift 를 만듦
- "fit 검사" 와 "advance 누적" 을 분리한 fix(`height_for_fit` vs `total_height`)는 fit 만 정확하면 되는 시스템에서는 유효하나, layout 도 advance 를 별도 누적하는 우리 구조에서는 두 모델 간 정합 필요

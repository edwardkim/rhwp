# Stage 2 — Task #1363 SSOT 설계 (공유 높이 함수 + 마이그레이션 전략)

Stage 1(모델 매핑 + 골든 베이스라인) 산출물을 입력으로, **공유 높이 함수의 시그니처·반환
계약**을 확정하고, **분기별 exam 의존도**를 1:1 매핑하며, **A/B 플래그 + 골든 비교 하니스**
기반 점진 마이그레이션 전략을 수립한다. 본 단계는 **설계 전용**(소스 무수정). 실제 이전은
Stage 3+ 에서 게이트별로 수행한다.

---

## 1. SSOT 계약 — `endnote_para_rendered_advance`

### 1.1 SSOT 정의 (Stage 1 §3 확정)
**layout 의 순차 format 렌더 높이**가 ground truth:

```
rendered_advance(para) = line_advances_sum(0..N)        // 실제 렌더되는 줄 높이 합
                       + trailing_ls_bridge(prev, curr)  // 단 연속 시 0, 아니면 prev trailing ls
```

typeset 누적(`acc`)은 이 값을 **정확히 예측**해야 한다. 현재 `acc = metric_advance_px.max(min_h)`
는 saved-vpos delta `(tb − base)` 기반이라 내부 rewind(A)·trailing-ls(B)에서 갈린다.

### 1.2 시그니처 (확정)

```rust
/// 미주 문단 1개가 단 안에서 차지하는 렌더 높이를 SSOT 로 산출.
/// typeset 누적(acc)과 layout vpos_adjust 가 동일 규칙을 공유하기 위한 단일 진입점.
struct EnAdvance {
    /// 페이지/단 잔여 대비 fit 판정용(= advance − trailing_ls, 마지막 줄 줄간격 제외).
    fit: f64,
    /// current_height 누적용(= 실제 렌더 점유 높이). SSOT = rendered_advance.
    acc: f64,
    /// split 가능 지점 힌트(다단 분할에서 첫 줄~N 줄 누적이 잔여에 맞는 최대 split line).
    /// None 이면 분할 불가(단일 줄/원자). 기존 split_endnote_to_fit 로직 입력.
    split_hint: Option<SplitHint>,
}

struct SplitHint {
    /// 줄별 누적 advance (line_advances_sum 의 prefix). split line 탐색에 사용.
    line_advances: Vec<f64>,
    /// 마지막 줄 trailing ls (분할 시 rest 쪽 spacing_after 회계).
    trailing_ls: f64,
}

fn endnote_para_rendered_advance(
    ctx: &EnAdvanceCtx,   // para, fmt, prev_en_bottom_vpos, col_count, profile flags
) -> EnAdvance
```

### 1.3 반환 계약 (불변식)
- `acc == fit + trailing_ls`  (단, trailing-ls bridge 가 켜진 경우 `acc == fit`).
- `acc == line_advances_sum + bridge_ls`  ← **SSOT 목표 등식** (현재 위배되는 부분이 A/B).
- `fit <= acc` 항상.
- `split_hint` 의 prefix 합은 `acc` 를 초과하지 않는다(마지막 prefix == line_advances_sum).
- `col_count == 1` 또는 offset 미상이면 `(h4f, tot)` 폴백(현행 유지, SSOT 미적용 영역).

> **핵심 전환**: `acc` 의 정의를 `metric_advance_px.max(min_h)`(saved-vpos delta) →
> `line_advances_sum + bridge_ls`(layout 순차 렌더) 로 옮긴다. `fit` 는 잔여 판정용이므로
> SSOT 변경의 영향이 작다(주로 split 게이트). divergence 제거는 **acc 경로부터** 착수.

---

## 2. 분기별 exam 의존도 매핑 (Stage 3 이전 순서 결정 근거)

`compute_en_metrics`(typeset.rs 2555) 의 각 분기가 어느 exam 골든 가드에 의존하는지:

| # | 분기 (typeset) | 조건 | 영향 exam (골든 가드) | SSOT 후 처리 |
|---|----------------|------|----------------------|--------------|
| 1 | `base` = tf vs prev | local_vpos_rewind \|\| large_vpos_jump | 전 exam (단 첫 미주 base) | 유지 — base 선택은 layout 의 prev_layout_para 대응. SSOT 무관(seed) |
| 2 | `advance_px = (tb−base)` | 항상 | — | **폐기 대상**: acc 의 입력에서 제거(rendered_advance 로 대체). fit 산출엔 잔존 가능 |
| 3 | `compact_local_rewind` | compact_profile && local_vpos_rewind | 3-09'23, 3-11'22 (issue_1082) | 유지 — min_h floor. SSOT 와 max() 결합 |
| 4 | `inline_object_formatter_overestimate` | compact && TAC pic && h4f>adv+80 | **3-09'23 12쪽** (issue_1082) | **Stage 3-C 검토**: rendered_advance 가 inline 높이 이중계상 안 하면 자연 해소 가능. 보류 |
| 5 | `min_h` 3-way | inline_obj / rewind / else h4f | 위 + 전 exam(h4f 바닥) | rewind 가지(internal_vpos_rewind) → **Divergence A**. line_advances_sum 으로 대체 |
| 6 | `stale_forward_vpos` | compact && adv>h4f+100 | 3-09'22, 3-10'22 (issue_1082) | 유지 — saved-vpos 과대 전진 캡. rendered_advance 는 saved delta 무관이라 **자연 무력화 가능**, 게이트로 확인 |
| 7 | `capped_new_endnote_advance` | new_endnote gap cap | 신규 미주 간격 (issue_1082 2022군) | 유지 — between-notes margin 회계. acc 가 line_advances_sum 이면 gap 은 별도 가산 |
| 8 | `metric_advance_px` 선택 | 위 분기 종합 | — | **acc 경로에서 제거**, fit 잔존 |
| 9 | `fit = metric − trailing_ls`, `acc = metric.max(min_h)` | 항상 | 전 exam | acc 정의 교체(§1.3). **Divergence B**(trailing_ls) 여기서 정합 |

### layout `vpos_adjust`(height_cursor.rs 146) 대응
| layout 분기 | 조건 | typeset 대응 | exam |
|-------------|------|--------------|------|
| page_path (vpos_page_base) | vpos_page_base 존재 | base #1 | 전 exam |
| lazy_path (vpos_lazy_base) | lazy_base 존재 | base #1 | footnote-01 p1 |
| trailing_ls bridge 게이트 | vpos_continuous && prev_has_text → 0 | **#9 trailing_ls (B)** | pi=872/874 (2024 target) |
| lazy_base_corrected ≥ 0 게이트 | 역산 음수 시 fallback | (typeset 무대응 — 단방향) | exam_kor p5, 복학원서 |
| backward clamp (lazy_base<0) | 자리차지 표 | (해당 없음 — 미주 전용) | — |
| compact_endnote_question_title cap | suppress_large_forward_jump | (#6 stale_forward 유사) | compact 미주 |

### 이전 순서 (회귀 위험 분산)
1. **Divergence A (rewind, #5)** — 최대(pi=894 −61.2). `internal_vpos_rewind || compact_local_rewind`
   가지의 min_h/metric 을 `line_advances_sum` 으로 교체. 영향: 3-09'23, 3-11'22 + **2024 target**.
2. **Divergence B (trailing-ls, #9)** — pi=872/874 −6. `acc` 의 trailing-ls 회계를 layout 의
   `vpos_continuous && prev_has_text` 게이트와 동일화. 영향: 2024 target.
3. **#4/#6 재검토** — A/B 이전 후 saved-vpos delta 가 acc 에서 사라지면 inline_obj·stale_forward
   캡이 불필요해질 수 있음. 골든 비교로 **제거 가능 여부 판정**(불가 시 유지).

---

## 3. 마이그레이션 전략 — A/B 플래그 + 골든 비교 하니스

### 3.1 A/B 플래그
런타임 환경변수 `RHWP_EN_SSOT` 로 acc 산출 경로 전환(Stage 3 도입, 기본 legacy):

| 값 | acc 경로 | 용도 |
|----|----------|------|
| (미설정) / `legacy` | 현행 `metric_advance_px.max(min_h)` | 기본·회귀 가드 기준 |
| `A` | Divergence A 만 SSOT(rewind→line_advances_sum) | 단계별 게이트 |
| `B` | A+B (trailing-ls 정합까지) | 단계별 게이트 |
| `on` | 최종 SSOT 전면 | 완료 후 기본값 승격 |

- 플래그는 `compute_en_metrics` 내부 `acc` 결정부에만 분기 삽입(읽기 1회 캐싱). fit/split 경로
  불변 유지 → 회귀 면적 최소화.
- 각 divergence 이전이 골든 무회귀로 확정되면 해당 단계를 **기본 경로로 승격**하고 플래그 가지
  제거(누적 부채 방지).

### 3.2 골든 비교 하니스
**목적**: para 단위로 typeset `acc` ↔ layout `rendered_advance` divergence 를 정량 측정,
이전 전후 변화를 가시화.

설계(Stage 3 에서 구현):
- 진입: `RHWP_EN_SSOT_DEBUG=1` 시 미주 문단마다 1줄 emit
  `EN_SSOT pi={} acc_legacy={:.1} acc_ssot={:.1} line_adv_sum={:.1} bridge_ls={:.1} div={:.1}`
- 수집 스크립트 `scripts/task1363_ssot_diff.py`:
  - 대상 exam 전체를 export-svg(stderr 캡처) → para별 div 표 + exam별 |div| 합.
  - **두 기준 비교**: (a) legacy vs ssot acc 차이, (b) ssot acc vs line_adv_sum(SSOT 등식 잔차).
  - 출력: `mydocs/report/task1363_ssot_diff_<stage>.tsv` + 요약.
- 판정: SSOT 등식 잔차(b) → 0 수렴 = 이전 성공. legacy 차이(a) → 회귀 영향 추적.

### 3.3 게이트 (매 divergence 이전 공통, 수행계획서 §5)
1. 전체 `cargo test` 0 failed (특히 issue_1082/1139/1274/1284).
2. `scripts/task1274_visual_sweep.py --target all` flagged ≤ 베이스라인
   (2022-09:1, 2023-09:0, 2024-below20:1, 2024-between20:1, 2022-10:0, 2022-11:1).
3. **issue_1082 px**: 비대상 exam(3-09'23 hwp/hwpx, 3-11'22, 3-09'22) **0.0 유지**,
   대상(3-09'24 sep20/20) **50.1 → 감소**(REG_LIMIT 60 이내 + p17 C×C·p22 해소 방향).
4. 골든 비교 하니스(§3.2) SSOT 등식 잔차 감소 확인.
- 임의 단계라도 ②③ 악화 시 **즉시 롤백·재설계**(무리한 일괄 변경 금지, §6).

---

## 4. Stage 3 착수 항목 (다음 단계 입력)
1. `EnAdvance`/`EnAdvanceCtx` 구조 + `endnote_para_rendered_advance` 골격 추가(legacy 경유 패스스루).
2. `RHWP_EN_SSOT` 플래그 + `RHWP_EN_SSOT_DEBUG` emit 배선.
3. `scripts/task1363_ssot_diff.py` 작성 → 베이스라인 div 표 기록.
4. **Divergence A** 이전(rewind→line_advances_sum) → 게이트(§3.3) → 단계 보고서.

> Stage 3 부터 소스 수정 — 착수 전 승인 요청. 본 Stage 2 는 설계 확정까지.

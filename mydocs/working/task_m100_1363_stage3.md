# Stage 3 — Task #1363 Divergence A 이전 (rewind → line_advances_sum)

Stage 2 설계(공유 높이 함수 + A/B 플래그 + 골든 비교 하니스)를 구현하고, **Divergence A
(내부 vpos rewind para 의 누적 과소)** 를 SSOT(layout 순차 렌더 높이)로 이전했다. 전 골든
가드 무회귀로 확정되어 **기본 경로로 승격**했다.

## 1. 구현

### 1.1 SSOT 플래그 + 디버그 (`src/renderer/typeset.rs`)
- `EnSsotLevel { Legacy, A, B, On }` + `en_ssot_level()` — 환경변수 `RHWP_EN_SSOT`.
  - **기본값 = A** (Stage 3 승격). `legacy`/`off` 로 전 divergence 원복(롤백·A/B 비교).
    `B`/`on` 은 후속 단계 opt-in.
- `en_ssot_debug()` — `RHWP_EN_SSOT_DEBUG=1` 시 미주 para 마다 stderr `EN_SSOT` 라인 emit.

### 1.2 Divergence A 이전 (`compute_en_metrics`)
- 종전: `internal_vpos_rewind` para 의 `acc = min_vpos_rewind_height`(첫 줄 높이) — saved-vpos
  delta 가 rewind 로 과소 추정.
- SSOT: layout 은 첫 줄만 vpos 로 배치한 뒤 나머지 줄을 **순차 format 렌더**하므로 실제 점유
  높이 = **전체 `line_advances_sum`**. `acc = line_advances_sum.max(min_vpos_rewind_height)`.
- `fit` 경로·split 경로·비-rewind para 는 불변(회귀 면적 최소).

### 1.3 골든 비교 하니스 (`scripts/task1363_ssot_diff.py`)
- exam 별 export-svg(stderr `EN_SSOT` 수집) + SVG overflow(issue_1082 동일식) 측정.
- `--level`/`--baseline`/`--out` 으로 레벨 간 overflow·SSOT 잔차 델타 + para TSV 출력.
- 산출: `mydocs/report/task1363_ssot_diff_stage3.tsv`.

## 2. 검증 (Stage 2 §3.3 게이트)

| 게이트 | legacy | **A(기본)** | 판정 |
|--------|--------|------------|------|
| 전체 cargo test | 2126 pass / 0 fail | **2126 pass / 0 fail** | ✅ 무회귀 |
| issue_1082 비대상 overflow | 0.0 | **0.0** | ✅ 유지 |
| issue_1082 대상(sep20/20) overflow | 50.1 | **50.1** (Δ+0.0) | ⚠️ 불변 |
| 시각 sweep flagged (6 타겟) | 1/0/1/1/0/1 | **1/0/1/1/0/1** (동일 페이지) | ✅ 베이스라인 |

- sweep flagged 페이지·플래그 종류까지 legacy ↔ A 완전 동일(2022-09 col[10]/order[10],
  2024-below20 col[10]/order[10], 2024-between20 line[11]/order[11], 2022-11 line/col/tail[13]).

## 3. 측정 결과 — Divergence A 의 정합과 한계

대상(sep20/20) 8개 rewind para 의 SSOT 잔차(acc − line_adv_sum)가 전부 0 으로 수렴
(legacy 잔차 합 ≈ 434px 해소):

| pi | acc(legacy) | acc(A)=line_adv_sum | legacy 잔차 |
|----|-------------|---------------------|-------------|
| 522 | 38.5 | 182.9 | −144.4 |
| 580 | 40.4 | 75.8 | −35.4 |
| 655 | 236.3 | 272.3 | −36.0 |
| 870 | 72.1 | 105.7 | −33.6 |
| **894** | 38.4 | 99.6 | **−61.2** (Stage 1 식별 최대) |
| 922 | 72.1 | 107.7 | −35.6 |
| 1111 | 22.4 | 42.8 | −20.4 |
| 1175 | 427.3 | 495.5 | −68.2 |

### 핵심 발견
**Divergence A 이전으로 누적 모델은 SSOT-정합화되었으나 대상 overflow(50.1)는 불변.**
누적(acc) 증가가 overflow 발생 단(p17/p22) 경계를 바꾸지 않음 → rewind para 들은 초과 단의
critical path 밖. 즉 **p17 C×C·p22 overflow 의 근본 원인은 누적(acc) 아니라 fit/split**
(마지막 줄 col0→col1 미분리, #1357 결론과 정합). → **Stage 4 = Divergence B(trailing-ls)
+ fit/split 경로** 로 범위 확정.

## 4. 산출물
- `src/renderer/typeset.rs` — EnSsotLevel/플래그/디버그 + Divergence A SSOT 이전(기본 승격).
- `scripts/task1363_ssot_diff.py` — 골든 비교 하니스.
- `mydocs/report/task1363_ssot_diff_stage3.tsv` — para 단위 잔차 기록.

## 다음 (Stage 4)
Divergence B(trailing line-spacing, pi=872/874) 이전 + **fit/split 경로 조사**(대상 overflow
근본 원인). 매 단계 하니스 + 전체 cargo test + sweep 게이트. `RHWP_EN_SSOT=B` opt-in 으로
측정 후 무회귀 시 승격.

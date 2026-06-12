# 최종 결과 보고서 — Task #1363: 미주 높이 모델 SSOT 리팩터

- **이슈**: #1363 (M100), 관련 #1357/#1336/#1082/#1248/#1302
- **브랜치**: `local/task1363` (base: `stream/devel`)
- **성격**: 대형·다회차 아키텍처 리팩터 (조사·설계 우선, 단계적 게이트 검증)
- **결과**: **대상 미주 overflow(50.1px) 근본 해소**, 전 골든 무회귀. SSOT 안전 경계 확정.

---

## 1. 문제

미주 para 높이를 **두 모델**이 따로 계산해 단 하단 본문 초과(#1357)가 발생:
- typeset 누적 `compute_en_metrics → acc`: saved-vpos delta `(tb−base)` + caps/floors → `current_height`
- layout 렌더: `HeightCursor.vpos_adjust` 첫 줄 배치 + format 순차 줄높이

두 모델이 trailing-ls·내부 rewind·그림 배치에서 갈려 typeset 이 과소 누적 → 단 과충전.
부분 조정은 모두 회귀(#1357 결론)했던 난영역.

## 2. 접근 — SSOT 점진 마이그레이션 (플래그 게이트)

**원칙**: layout 렌더가 ground truth. typeset `acc` 가 layout 렌더 높이를 정확히 예측해야 함.

- `RHWP_EN_SSOT` 단계 플래그(`EnSsotLevel: Legacy/A/B/On`) + `RHWP_EN_SSOT_DEBUG` 계측
  (`EN_SSOT` acc/line_adv, `EN_ACC` 누적 경로).
- 골든 비교 하니스 `scripts/task1363_ssot_diff.py` — exam별 overflow + para 잔차.
- divergence 1개씩 이전 → 매 단계 **전체 cargo test + visual_sweep + issue_1082 px** 게이트,
  무회귀 확정 시에만 기본 승격.

## 3. 단계별 결과

| Stage | 내용 | 결과 |
|-------|------|------|
| 1 | 모델 매핑 + 골든 베이스라인 | divergence A/B 식별, 기준 px 기록 |
| 2 | SSOT 설계 (공유 함수·플래그·하니스) | 계약 확정 |
| 3 | **Divergence A**(내부 rewind → line_advances_sum) | 무회귀 승격. **단 overflow 불변** → 근본 원인은 fit/split 으로 범위 좁힘 |
| 4 | **Divergence C**(TAC 그림 겹침→순차 적층) | **대상 overflow 50.1→0**, 무회귀 승격(기본 B) |
| 5 | 잔여 B(trailing-ls)·전면 SSOT | **음성 결과**: 안전 정합 불가, 보류 확정 |

### 핵심 규명 (Stage 4)
대상 p22 단0 overflow 50.1px = **TAC 그림 미주 pi=1131(309px)** 1건. typeset 이 이를
`max(rewind_start+adv)`(겹침 가정, 앞 제목 옆 배치)로 누적했으나, treat_as_char 그림은
렌더러가 문단 흐름에 **inline 순차 적층**(옆 배치 아님) → 단 +58px 과충전. `+= adv`(순차)로
교정해 해소.

### SSOT 안전 경계 (Stage 5)
| divergence | 안전 정합 | overflow 이득 | 상태 |
|------------|:--------:|:------------:|------|
| A 내부 vpos rewind | ✅ | 0 (모델 정합) | **기본 적용(B)** |
| C TAC 그림 겹침→적층 | ✅ | **−50.1** | **기본 적용(B)** |
| B trailing-ls (pi=872/874) | ❌ 질문흐름 회귀 10건 | 0 | 보류 |
| 전면 SSOT (전 para line_adv_sum) | ❌ 2022 overflow +166px | 0 | 보류 |

잔여는 overflow 무이득 + 회귀. 현 모델(caps + layout 조건부 bridge)이 실 렌더의 최선
근사이며, 무차별 SSOT 화는 회귀. → **안전·유익 영역 = A+C(기본 B)로 확정**.

## 4. 검증 (전 단계 공통 게이트)

| 게이트 | 베이스라인(legacy) | **최종(기본 B)** |
|--------|-------------------|-----------------|
| 전체 cargo test | 2126 pass / 0 fail | **2126 pass / 0 fail** |
| issue_1082 비대상 overflow (2023/2022×3) | 0.0 | **0.0 유지** |
| issue_1082 **대상**(sep20/20) | 50.1 | **0.0** (−50.1) |
| visual_sweep flagged (6타겟) | 1/0/1/1/0/1 | **동일**(베이스라인) |

신규 회귀 가드: `issue_1082` 대상 바운드 **REG_LIMIT 60 → TIGHT 5.0px**.

## 5. 변경 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/typeset.rs` | `EnSsotLevel` 플래그 + Divergence A/C SSOT + EN_SSOT/EN_ACC 계측. 기본 B |
| `tests/issue_1082_endnote_multicolumn_drift.rs` | 대상 타이트 가드(5.0px) |
| `scripts/task1363_ssot_diff.py` | 골든 비교 하니스 |
| `mydocs/report/task1363_ssot_diff_stage{3,4}.tsv` | para 잔차 기록 |
| `mydocs/working/task_m100_1363_stage{1..5}.md`, `plans/task_m100_1363.md` | 수행계획·단계 보고서 |

## 6. 롤백/운용
- `RHWP_EN_SSOT=legacy` → 전 divergence 원복(긴급 롤백·A/B 비교). `A`=C 제외, `on`=예약(현 B).
- 디버그: `RHWP_EN_SSOT_DEBUG=1` → 미주 para 누적 계측 stderr.

## 7. 결론
#1357 의 핵심 요구(미주 다단 col0 본문 초과)를 **근본 원인 규명 후 해소**(50.1→0)했고,
전 골든 무회귀를 단계별 게이트로 보장했다. SSOT 리팩터의 안전·유익 영역(A+C)을 확정하고,
무익·고위험 영역(B·전면)을 실증으로 경계 지어 보류했다. **종결 권고.**

### 후속(선택)
- merge: `local/task1363` → `local/devel` → devel.
- 잔여 divergence 는 향후 layout 조건부 trailing-ls bridge 의 typeset 정확 복제가 가능해질
  때만 재검토(현재는 회귀 위험 > 이득).

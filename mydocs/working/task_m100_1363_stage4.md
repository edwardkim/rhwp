# Stage 4 — Task #1363 대상 overflow 근본 해소 (Divergence C: TAC 그림 순차 적층)

Stage 3 가 "Divergence A 는 SSOT-정합이나 대상 overflow(50.1) 불변 → 근본 원인은 fit/split"
로 범위를 좁혔다. Stage 4 는 **p22 overflow 를 계측으로 정확히 귀인**하고, 그 원인(TAC 그림
미주의 겹침 가정)을 SSOT(렌더러 순차 적층)로 교정해 **overflow 50.1 → 0** 으로 해소했다.

## 1. 근본 원인 규명 (계측)

### overflow 위치
- 대상(sep20/20) overflow 50.1px 는 **전부 p22 단 0(좌측 단)** 1곳. 마지막 줄 y=1172.6 vs
  본문 하단 1092.3 → 80px 초과(페이지 하단 기준 50.1).
- p22 단 0 은 **rewind para 없음**(rewind 집합 522/580/655/870/894/922/1111/1175 모두 타 페이지)
  → Stage 3 Divergence A 가 p22 를 못 바꾼 이유 확정.

### 누적 vs 렌더 귀인 (EN_ACC 계측)
- 단 0 para별 `acc ≈ line_advances_sum`(per-para 정확). 그러나 누적
  `current_height`(=1010.7) 가 `sum(acc)`(=1068.3) 보다 **57.6px 작음** → 누적이 손실.
- 손실 지점: **pi=1131**(TAC 그림 미주, "(빈)"+Shape, adv=309.2).
  - 경로 `path=TACmax`: `ch_before=109.7` → `max(109.7, rewind_start 51.7 + adv 309.2)=360.9`.
  - 순차 적층(`+=`)이면 `109.7+309.2=418.9`. 차이 **58px** = 단 과충전량 = overflow 근원.

### 왜 겹침 가정이 틀렸나
`tac_picture_rewind_height` 는 local_vpos_rewind TAC 그림이 저장 vpos 로 앞 제목 **옆에**
배치된다고 보고 `max(rewind_start+adv)` 로 누적(겹침 가정). 그러나 **TAC(treat_as_char)
그림은 렌더러가 문단 흐름에 inline 으로 순차 적층**한다(옆 배치 아님). 렌더 측정상 그림은
y≈122→455(333px) 로 앞 텍스트 **아래** 적층 → 겹침 가정이 그림 높이를 과소 계상.

## 2. 수정 (Divergence C, `src/renderer/typeset.rs`)
- `ssot_level >= B` 시 `tac_picture_rewind_height` 경로를 **`+= en_advance`(순차 적층)** 로
  전환(렌더러 정합). legacy/A 는 종전 `max(rewind_start+adv)` 유지(롤백·비교).
- `EN_ACC` 계측 라인(`RHWP_EN_SSOT_DEBUG`) 추가 — 누적 경로(TACmax/TACstack/add)·전후 높이.

## 3. 검증 (Stage 2 §3.3 게이트)

| 게이트 | legacy | A | **B(기본 승격)** |
|--------|--------|---|-----------------|
| 전체 cargo test | 2126/0 | 2126/0 | **2126/0** |
| issue_1082 비대상 overflow | 0.0 | 0.0 | **0.0** |
| issue_1082 **대상**(sep20/20) | 50.1 | 50.1 | **0.0** (−50.1) |
| 시각 sweep flagged (6타겟) | 1/0/1/1/0/1 | 동일 | **동일**(베이스라인) |

- `tac_picture_rewind_height`(TACmax) 트리거: issue_1082 6개 exam 중 **대상만 1회**(pi=1131).
  타 exam 미트리거 → C 수정의 영향면이 대상에 격리. 전체 suite(2126) 로 그 외 문서 회귀 0 확인.

## 4. 승격 + 회귀 가드
- `RHWP_EN_SSOT` 기본값 **B 승격**(A→B). `legacy`/`off` 원복, `A` 는 C 제외, `on` 후속 opt-in.
- `tests/issue_1082_…`: 대상 가드 **REG_LIMIT 60 → TIGHT 5.0** 으로 타이트화(#1363 회귀 가드).
  종전 "근본 정정 보류·바운드 추적" 주석을 해소 기록으로 갱신.

## 5. 산출물
- `src/renderer/typeset.rs` — Divergence C(TAC 순차 적층) B-게이트 + EN_ACC 계측 + 기본 B 승격.
- `tests/issue_1082_endnote_multicolumn_drift.rs` — 대상 타이트 가드(5.0px).
- `scripts/task1363_ssot_diff.py` — 기본 B 대응(레벨 명시 설정) 수정.
- `mydocs/report/task1363_ssot_diff_stage4.tsv` — para 잔차 + overflow 기록.

## 6. 잔여 (후속 검토)
- **Divergence B(trailing line-spacing, pi=872/874, ~6px)**: overflow 무영향·미세. 별도 가드
  없어 보류(필요 시 `on` 단계에서 정리).
- per-para `sum(acc)` 와 렌더 span 의 잔차(~13.6px, 그림/수식 para 의 line_adv_sum 과소): 단
  경계 미교차로 무해. SSOT 등식 완전 수렴은 `on` 단계 과제.

## 다음
대상 overflow 해소·전 골든 무회귀로 Stage 4 완료. 최종 보고서(Stage 5/정리)에서 issue_1082
바운드 감소·신규 가드·잔여 divergence 처리 방침 정리 예정.

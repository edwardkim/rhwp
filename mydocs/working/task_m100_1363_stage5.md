# Stage 5 — Task #1363 잔여 Divergence B / 전면 SSOT 검토 (음성 결과: 보류 확정)

Stage 4 가 대상 overflow 를 해소(A+C, 기본 B)한 뒤 남은 잔여 divergence를 검토했다.
**결론: 잔여 Divergence B(trailing line-spacing)·전면 SSOT 는 `acc=line_advances_sum` 로
안전 정합 불가.** 두 방향 모두 실증 회귀로 확인하고 보류를 확정했다. 기본(B) 불변.

## 1. 검토 가설
SSOT 등식 `acc == line_advances_sum` 을 전 para 로 확장하면(레벨 `on`) 잔여 divergence
(트레일링 줄간격 pi=872/874 −6px, sum(acc)−render 잔차)가 닫힌다는 가설.

## 2. 실험 A — 전면 적용 (`on`: 모든 para acc=line_advances_sum)
- 하니스 결과: **3-11'22 overflow 0→165.9, 3-09'22 0→20.7 회귀**.
- 원인: capped/stale/overlap para 는 렌더러가 저장 vpos 로 **겹쳐**(line_adv_sum 보다 작게)
  그린다. caps(`capped_new_endnote_advance`/`stale_forward_vpos`)가 그 실 렌더를 근사하던 것을
  line_adv_sum 으로 덮어써 과누적 → 단 초과. **line_adv_sum 은 전 para 의 render 가 아님.**

## 3. 실험 B — uncapped sequential 한정 (`metric==advance_px` 만 line_adv_sum)
- 하니스(6 exam overflow): **무회귀**(전 exam 0.0, |ssot_res| 감소: 3-09'23 431→188, 대상 240→132).
- 그러나 **전체 cargo test: 10건 회귀** — issue_1139(2023 p12/13 경계, p17 문30 단배치, 페이지수),
  issue_1261(2024 p10 문8/12 tail), issue_1284(2024-between20 p13/18/19/21/22·23 질문 흐름).
- 원인: uncapped para 에 trailing-ls 가산이 미주 **질문 흐름(단 배치·페이지 경계)** 을 흔듦.
  하니스 overflow 지표로는 안 잡히고 **통합 테스트(PDF 정합)** 만 포착. [[feedback_full_cargo_test_before_pr]]

## 4. 결론 (보류 확정)
- 잔여 Divergence B·전면 SSOT 는 **overflow 무영향**(시각 이득 0)인데 **안전 정합 불가**.
  layout 의 trailing-ls 는 조건부 bridge(`vpos_continuous && prev_has_text` 게이트,
  height_cursor.rs:214)라 단순 line_adv_sum 가산과 불일치 — 정확 복제는 fit 경로까지 얽혀
  고위험. **무리한 일괄 변경 금지 원칙**에 따라 보류.
- 코드: 실험 전량 revert. `acc` 는 **A(rewind)+C(TAC) 만 SSOT**. `on` tier 는 예약(현 B 동일).
  보류 사유를 `compute_en_metrics` 주석·`EnSsotLevel::On` 주석에 인코딩(재시도 방지).

## 5. 검증
- 기본(B) 전체 cargo test **2126 pass / 0 fail**(실험 revert 후 재확인).
- `on`(현 B 동일) overflow 전 exam 0.0, 대상 0.0 유지.

## 6. SSOT 리팩터 경계 (Task #1363 종합)
| divergence | 안전 정합 | overflow 이득 | 상태 |
|------------|----------|--------------|------|
| A 내부 vpos rewind | ✅ | 0 (모델 정합) | 기본 적용(B) |
| C TAC 그림 겹침→적층 | ✅ | **−50.1 (대상 해소)** | 기본 적용(B) |
| B trailing-ls | ❌(질문흐름 회귀) | 0 | **보류** |
| 전면 SSOT | ❌(2022 overflow 회귀) | 0 | **보류** |

→ SSOT 리팩터의 **안전·유익 영역은 A+C(=기본 B)로 확정**. 잔여는 현 모델(caps+게이트)이
실 렌더의 최선 근사이며, 무차별 SSOT 화는 회귀. 종결 권고.

## 다음
Task #1363 최종 결과 보고서(`_report.md`) 작성 → 오늘할일 갱신 → 종결 승인.

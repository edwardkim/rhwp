# Stage 3 (v3) — split/fit 정합 시도 → 근본 원인 규명 (음성 결과)

측정-기반 fit/break 정합을 시도해 A3 overflow 3건을 해소하려 했으나, **overflow 가 높이
추정값에 대해 비단조(non-monotonic)** 임을 실증했다. 정확 측정만으로는 fit-parity 가 불가하며,
튜닝된 break 게이트와의 **공동 재보정**이 필요함을 규명했다(regression-prone — Task #1022/
`trailing_model_no_ssot` 영역). 소스 무변경(실험 원복).

## 1. 핵심 규명 — 렌더러는 ssot-불변, A3 는 pagination 만 바꾼다
`RHWP_EN_SSOT` 은 **pagination(어느 para 가 어느 단)만** 바꾼다. 렌더러(`build_single_column`)는
레벨과 무관하게 동일 코드다. 따라서 A3 overflow 는 "측정이 틀려서"가 아니라 **A3 pagination 이
특정 단을 과충전**해 그 단의 렌더가 본문을 초과하는 것이다.

- sep20/20: B=overflow 0/23쪽, **A3=23.5px(p19)/25쪽**. A3 는 전반적으로 쪽수가 늘지만(단당 para
  감소) **한 단(p18 col1)만 과충전** → para 961–963 이 col_bottom=1092.3 위 y=1106/1130/1150 에 그려짐.
- 961–963 자체 측정은 diff~0(휴리스틱과 동일) → **상류 cascade**(앞선 +6px trailing·pi=965
  heuristic 0→measured 138.6 등 정확화)가 break 결정을 이동시켜 이 단에 몰림.

## 2. 실험 — vpos_adjust pullback 제거 (음성)
가설: 측정 advance 는 `layout_partial_paragraph` 의 실제 렌더 기여분이므로, 컬럼 시작부터
순차 누적이 곧 렌더 bottom → `simulate` 의 `vpos_adjust` saved-vpos pullback 이 이중 압축.

A3 에서 pullback 제거(순수 측정 flow 누적) 측정:

| 문서 | 원본 A3(vpos_adjust) | 실험(pullback 제거) |
|------|---------------------|--------------------|
| sep20/20 | 23.5px | **63.3px** (악화) |
| 3-09_2022 | (fail) | 3.6px |
| 3-11_2022 | (fail) | **239.9px** (악화) |

→ **음성**. 렌더러도 항목 간 `vpos_adjust` 압축을 적용하므로 pullback 은 필요. 제거 시 over-count.
원복함.

## 3. 결정적 발견 — overflow 는 높이 추정에 비단조
- pullback 有(원본 A3): 23.5px / pullback 無: 63.3px. **더 큰 높이 추정(over-count)이 overflow 를
  줄이지 않고 늘렸다.**
- 원인: break 결정이 cascade — 높이 추정 변화가 단 경계를 재배치하면 **다른 단**이 과충전된다.
  overflow 가 단일 height 파라미터에 단조 반응하지 않음.
- 함의: **정확 측정값 주입만으로 fit-parity 불가.** break 게이트(`available*0.85`/`*0.90`,
  `a2_overflow_with_para`, `split_endnote_to_fit` 조건)가 휴리스틱 높이에 맞춰 하드튜닝돼 있어,
  측정 높이와 **공동 재보정**해야 한다.

## 4. 왜 지금 재보정하지 않는가 (regression-risk)
- break 게이트 재튜닝은 메모리 `lazy_base_trailing_ls_gate`(Task #1022: trailing-ls 무조건
  적용/제거 둘 다 회귀 — **조건부 게이트가 정답**)·`trailing_model_no_ssot`(전면 통일 금지)의
  고위험 영역이다.
- 비단조 cascade 상에서 한 문서를 맞추면 다른 문서가 깨지는 구조 → **문서별 다회차 검증** 필수.
  단발 변경은 무책임.

## 5. 상태 + 다음
- **측정 인프라(Stage 1–2) 견고**: per-para 측정은 정확(pi=965 0→138.6 정확화, 다줄 diff~0),
  부작용 격리 실증, 기본 B 무회귀(123/123).
- **A3 = 실험 opt-in 유지**: 정확 측정은 동작하나 fit-parity 는 게이트 재보정 전까지 미달.
- **다음(Stage 4 후보)**: break 게이트를 측정 기반 조건부로 재설계 — `a2_overflow_with_para` 를
  단일 신뢰원으로 두고 `available*k` 휴리스틱 임계를 제거, 문서별(`issue_1082`/`1139`/`1189`/
  `1284`) 회귀를 묶어 다회차 재보정. 비단조성 때문에 **단일 문서 최적화 금지**, 전 exam 동시
  green 을 게이트로.
- 소스 변경 없음(실험 원복 확인 — `git diff` 0, B 5/5 pass).

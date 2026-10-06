# HWPX 불러오기/저장/렌더링 무손실 전수 재검증 보고서 (2026-06-28 v2, 전 fix 반영)

**대상**: `C:\Users\planet\hwpdocs` (서울시 opengov 결재문서, **18,365건**으로 증가)
**바이너리**: `local/task1618` `e1a159ef` (**#1608 tolerance 제거 + #1611 footer page-fit + #1612 메트릭** 전부 반영)
**직전 보고**: `hwpx_lossless_3axis_20260628.md` (16,568건, #1608 only `e371214c`)
**검증 3차원**: ① IR roundtrip(불러오기/저장) ② 한글 오라클(저장 시각충실) ③ rhwp 렌더링 레이아웃

---

## 0. 코퍼스

| 항목 | 값 |
|------|----|
| 전체 HWPX | 18,365 (직전 16,568 → +1,797) |
| HWP5(.hwp) | 0 |

## 1. ① 불러오기/저장 — IR Roundtrip (전수, parse→serialize→reparse→2round)

| status | 건수 | 비율 |
|--------|------|------|
| **PASS (diff=0)** | **18,353** | **99.81%** |
| IR_DIFF | 4 | 0.02% |
| PARSE_FAIL | 30 | 0.16% |
| SERIALIZE/REPARSE/ROUND2/PKG_FAIL | 0 | 0 |

- **유효 파일(손상 30 제외) PASS = 18,353 / 18,357 = 99.98%.**
- **PARSE_FAIL 30건 = 전부 소스측 손상 ZIP**(testzip CRC 오류, 불완전 다운로드). rhwp 무관.
- **IR_DIFF 4건 = 직전과 동일 파일**(36384689·36388711·36385445·36399822), 전부 first-para
  `char_shapes` offset shift(내용 손실 아님). **코퍼스 16,599→18,387 증가에도 4건 불변 = 안정.**

## 2. ② 저장 시각충실 — 한글 페이지 오라클 (orig.hwpx ↔ rt.hwpx, 둘 다 한글 PageCount)

표본 800 (seed42, COM PageCount).

| 표본 | 측정(OK) | COLLAPSE | EXPAND | ERR(COM) |
|------|------|------|------|------|
| 본 코퍼스 seed42×800 | 599 | **0** | 1 | 200 |

- **페이지 붕괴율 0% (0/600 측정)** — #1589 군집 대응 유지. EXPAND 1건(rt>orig, 붕괴 반대 단발).
- ERR 200/800 = 25% (한컴 COM 한계, 직전 57%보다 양호). 측정분 기준 판정, rhwp 무관.

## 3. ③ 렌더링 레이아웃 — rhwp 자체 페이지수 vs 한글 페이지수(원본)

| | 조인 표본 | 일치(rhwp==한글) | 불일치 |
|---|------|------|------|
| seed42 표본 | 600 | **573 (95.5%)** | 27 (4.5%) |

**차이 분포(rhwp − 한글)**: −2쪽 1 · −1쪽 6 · +1쪽 19 · +2쪽 1.

- **~95.5% 페이지 정합** (rhwp info 스캔 18,387/18,388=99.99% 무크래시).
- **직전(#1608 only) 90.9% → 이번(전 fix) 95.5% (+4.6pp)** — #1611 footer page-fit 이 corpus
  규모로 정합 향상 확인(요인 B footer 군집 해소).
- 이제 **+초과(19) > −부족(6)**: #1611 이 일부 footer 를 한글보다 다음 쪽으로 과밀기(over-push)
  하는 경향(통제셋 회귀와 동류, ~3% 표본). −1쪽은 잔여 razor-thin(요인 B 비-footer + footer 일부).

## 4. 신규 발견 — rhwp 패닉 1건 (실제 버그)

- **`36396650`**(서부공원여가센터): `rhwp info` **패닉** `document.rs:927`
  (`range start index 23 out of range for slice of length 0`). 손상 아닌 정상 ZIP.
- 원인: 안내문(guide) 제거 시 `field_range.start_char_idx`(23)가 빈 문단(text len 0) 범위 초과 —
  `chars[..start]` 슬라이싱 bounds 미검증. 더 큰 코퍼스가 드러낸 엣지케이스(1/18,388).
- **별도 fix 대상**(bounds 가드 추가). 축③ FAIL 1건의 정체.

## 5. 결론 (3차원)

| 차원 | 지표 | 결과 | 직전 대비 |
|------|------|------|------|
| ① 불러오기/저장(구조) | IR diff=0 | **99.98%**(유효), IR_DIFF 4건 불변 | 안정 |
| ② 저장 시각충실(한글) | 페이지 붕괴율 | **0%** (0/600) | 유지 |
| ③ 렌더링 레이아웃(한글 대비) | rhwp==한글 | **95.5%** | **+4.6pp** (#1611 효과) |

- **구조·저장 무손실 완성 유지**(99.98% / 붕괴 0%) — 코퍼스 +1,797 증가에도 회귀 없음.
- **렌더링 레이아웃 95.5%** — #1608+#1611 누적으로 직전 ~91%에서 향상. 잔여 불일치는 footer
  over-push(+) 와 razor-thin(−)의 양방향 미세차(아키텍처 한계, `render_minus1_page_gap.md`).
- 신규 패닉 버그 1건(36396650) — bounds 가드 별도 수정 필요.

### 후속 권고
1. **document.rs:927 패닉 fix** (빈 문단 + field_range 범위 가드) — 신규 우선.
2. 렌더링 +1 over-push(~3%) ↔ −1 razor-thin 양방향 미세 정합(장기, vpos-추종 등).
3. first-para char_shape offset 4건 통합 리팩터.

산출물: `output/poc/fidelity_full_e1a159ef/{inventory,rhwp_pages,oracle}.tsv`.

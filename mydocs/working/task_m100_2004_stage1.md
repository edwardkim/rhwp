# task_m100_2004 Stage 1 완료보고서 — 하네스·기준선 + 픽스포인트 국소화

- 이슈 #2004, 브랜치 `fix/2004-image-stack-pagination`

## 산출

- `output/poc/task2004/baseline.tsv` — 오라클(한글 2022) vs rhwp 현재 페이지 수, 변종 분류.
- 오라클 재현: `scratchpad/hwp_paramap.py`(pyhwpx rmtree 가드, SetPos+current_page), `paramap_multi.py`(다중).

## 기준선

| doc | 오라클 | rhwp | Δ | 변종 |
|---|---:|---:|---:|---|
| 1613000 청년주거정책 | 268 | 171 | **−97** | 부동(tac=false, overlap=false) |
| 1430000 최종보고서 | 404 | 384 | **−20** | 인라인(tac=true) |
| 1790387 PrEP | 146 | 130 | −16 | 표높이(별개 #1937) |
| 1220000 관세사 | 134 | 126 | −8 | 표높이(별개 #1937) |

## 픽스포인트 국소화 (Stage 2/3 대비)

### 인라인 (Stage 2) — 1430000 pi=2607
- 21개 tac 그림 라인(각 lh=64503 HU=227mm), 전부 vpos=0 + `[vpos-reset@line1..20]`.
- rhwp: `lines 0..20`(1쪽) + `lines 20..21` — 20장을 vpos=0으로 **한 쪽에 겹침**(1회만 분할).
- **관찰**: HWPX line_seg 는 `<lineseg>` XML 파싱 실물(비합성, tag bit31 미설정)이라 forced-break 필터(`!is_synthetic && vpos==0`)에 라인 1..20 이 **포함되어야** 하는데 rhwp 는 line 20 에서 1회만 분할. → intra-para 라인별 vpos-reset 강제분할이 **tac-그림-only 문단 경로에서 미적용**되는 지점을 **계측(instrument)으로 확정 필요**(단순 synthetic 스킵 가설은 기각). Stage 2 첫 작업 = 강제분할 결정부에 pi=2607 대상 eprintln 계측.

### 부동 (Stage 3) — 1613000 pi=1004
- 96 그림 tac=false overlap=false 전부 offset0. `allow_overlap` **레이아웃 경로 미소비**(사용처 `document_core/commands/*` 편집연산 한정) → 겹침해소 재배치 규칙 자체가 부재. anti-overlap displacement + 페이지 캐스케이드 신규 구현 필요.

## Stage 2/3 리스크
typeset.rs(14k 줄) hot-layout, 다수 특수 케이스 가드 공존. 변경은 좁은 게이트 + 단계별 오라클 pi-page 무회귀 대조(발동/비발동) 필수. Stage 2(인라인)가 상대적 저위험, Stage 3(부동)가 고위험.

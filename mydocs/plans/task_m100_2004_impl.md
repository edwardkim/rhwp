# task_m100_2004 구현계획서 — 전면 이미지 다수 앵커 문단 쪽당-1장

- 이슈: #2004 (v1.0.0), 수행계획: [task_m100_2004.md](task_m100_2004.md)
- 브랜치: `fix/2004-image-stack-pagination`
- 상태: 구현계획 (승인 대기 / 자동승인)

## 정밀 근본원인 (계측 확정)

### 인라인 변종 (1430000, pi=2607)
- pi=2607 = tac 그림 21장만 있는 빈-텍스트 문단. line_segs 21개, 각 **lh=64503 HU(227.6mm=전면)**, 각 라인 vpos=0 + `[vpos-reset@lineN]`(한글이 각 그림을 새 쪽 상단에 배치).
- rhwp 결과: PartialParagraph `lines 0..20`(page 383) + `lines 20..21`(page 384) — **20장을 vpos=0으로 한 쪽에 겹침**, 1회만 분할.
- **결함**: 문단 내(intra-paragraph) 라인 분할이 **라인별 vpos-reset(각 tac 그림=전면 라인)을 강제 페이지 분할로 반영하지 않음**. HWPX 경로에서 라인별 vpos-reset intra-para 강제분할 미발동으로 추정.

### 부동 변종 (1613000, pi=1004)
- tac=false 그림 96장, wrap=Square, overlap=false, 전부 offset0. `allow_overlap`이 레이아웃 경로에서 **미소비** → 겹침해소 재배치 규칙 부재.
- rhwp: 96장 앵커 쪽에 전부 겹침.

## 단계별 구현

### Stage 1 — 하네스·기준선 (소스 계측 한정, 무위험)
- 오라클 pi-page 기준선 고정: 1613000(268), 1430000(404), 회귀 표본(부동/이미지 다수 문서 20건) → `output/poc/task2004/baseline.tsv`.
- 발동조건 판별 계측(env `RHWP_IMGSTACK_DBG`): 문단별 tac/overlap/크기/개수/동일위치 카운트.
- 산출: `task_m100_2004_stage1.md`.

### Stage 2 — 인라인 변종 수정 (리스크 중)
- typeset 문단내 라인 분할에서 **연속 tac 전면(≈본문높이) 그림 라인**을 라인별 강제 페이지 분할.
- 발동 게이트: `para_is_treat_as_char_picture_only` + 라인 lh ≥ 본문높이×0.8 + 라인수 ≥ 2.
- 검증: 1430000 pi-page 오라클 재대조(404±), export-svg 겹침 해소, baseline 표본 무회귀.
- 산출: `task_m100_2004_stage2.md`.

### Stage 3 — 부동 변종 게이트·anti-overlap 배치 (리스크 높음)
- overlap=false + wrap=Square + 다수(≥2) + 전면급 + 동일/근접 위치 부동개체 집합 검출.
- 문서순(z-order/등장순) 직전 개체 하단으로 재배치 + 페이지 경계 초과 시 캐스케이드, typeset 페이지 진행 반영.
- 좁은 게이트로 일반 부동개체 회귀 차단.
- 산출: `task_m100_2004_stage3.md`.

### Stage 4 — 통합 검증
- 1613000(−97) 오라클 268± + 시각 겹침 해소. 1430000·#1994 재확인.

### Stage 5 — 회귀검증
- `hwpx_roundtrip_baseline` 게이트, 부동/이미지 다수 표본 pi-page 무회귀(발동/비발동 양쪽), 기존 wrap/float/tac 테스트.

### Stage 6 — 최종 보고서·정리
- `task_m100_2004_report.md`, orders 갱신, PR(fork→devel).

## 리스크·롤백
- 각 Stage 게이트를 좁게 유지, 단계마다 baseline pi-page 대조로 무회귀 확인 후 다음 단계.
- 부동 변종(Stage 3) 회귀 시 게이트 축소 또는 인라인 변종만 우선 병합.

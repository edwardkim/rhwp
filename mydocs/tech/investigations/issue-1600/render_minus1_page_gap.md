---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1600/README.md
last_verified: 2026-07-16
---

# 렌더링 −1쪽 갭 근본원인 조사 (Task #1600 Stage 1)

## 결론 (확정)

rhwp 렌더링이 한글보다 1쪽 적은 −1쪽 갭의 근본은 **발신명의(sender info) 블록의 배치
차이**다. 페이지 붕괴 군집(#1589)의 발신명의 footer 메커니즘과 **동일 현상의 −1쪽 방향**.

## 증거

### 권위 정답지 (한글 2022 PDF, 36387725)

| 페이지 | 내용 | y 범위(pt) |
|--------|------|-----------|
| 1 | 본문(머리·수신·제목·1~3항·붙임·끝) | 65 ~ 526 |
| 2 | **발신명의 블록만**(서울특별시장·수신자·결재란·협조자·시행·주소·전화) | 581 ~ 783 |

→ 한글은 발신명의 블록을 **페이지 2 하단**에 단독 배치.

### rhwp 배치 (dump-pages, dump)

- 발신명의 블록 = 마지막 `Table 1행×1열`, `field="발신명의"`.
- IR 속성: `treat_as_char=false, wrap=자리차지, vert=쪽(0=0.0mm)` = **vert_rel_to=PAGE**(페이지 앵커).
- rhwp 는 이 블록을 본문 직후 **inline**(vpos=48053, 본문 pi=10 직후)에 배치 → 본문+발신명의가
  body_area(990.2px) 안에 들어가 **1쪽**.

### 전 −1 케이스 공통

36387725·36390093·36392061·36384361 **모두** 마지막 블록이 `field="발신명의"` +
`vert=쪽(vert_rel_to=PAGE)`. → 체계적 패턴.

## 메커니즘

발신명의 블록은 **vert_rel_to=PAGE**(페이지 기준 앵커) + 공간 점유(wrap=자리차지) 표.

- **한글**: 발신명의를 페이지 하단 기준 위치에 앵커. 본문이 그 앵커 영역에 닿으면 발신명의를
  **다음 쪽으로 밀어** 단독 배치(razor-thin: 본문 누적이 앵커에 닿을락말락).
- **rhwp**: 페이지 앵커를 무시(또는 offset 0 을 top 으로 해석)하고 본문 흐름 직후 inline 배치
  → 본문이 약간만 짧아도(누적 높이 < 한글) 같은 쪽에 흡수 → −1쪽.

#1589(붕괴, +방향: rt 저장 후 본문 누적이 커져 발신명의가 밀려 29→3 등) 과 **동일 앵커
블록**, 반대 방향(렌더 시 본문 누적이 작아 발신명의를 흡수 → 2→1).

## 수정 방향 (Stage 2+ 후보, 고위험)

rhwp 페이지네이션이 **vert_rel_to=PAGE 발신명의 블록을 한글처럼 페이지-앵커**로 처리:
- 발신명의 블록의 페이지 내 앵커 위치를 한컴 규칙대로 산출.
- 본문 누적이 앵커 영역에 닿으면 발신명의(+본문 잔여)를 다음 쪽으로.

**위험: 매우 높음.** 거의 모든 정부 결재문서가 발신명의 블록 보유 → 전 코퍼스 페이지네이션
영향. +1쪽 회귀 위험. #1589(붕괴) 영역과 결합 — 양방향 동시 정합 필요(과소→−1, 과대→붕괴).
통제 비교(−1쪽 해소 − +1쪽/붕괴 회귀 > 0)가 필수 게이트.

## 대안 (저위험, 부분)

발신명의 블록만의 앵커 보정 대신, 본문 누적 높이의 체계적 과소(있다면)부터 교정. 단 본문은
저장된 LINE_SEG 사용이라 per-line 은 한글과 동일 → 차이는 발신명의 앵커 자체일 가능성 큼.
```
산출물: output/poc/task1600_36387725_hangul.pdf (권위 PDF)
```

---

## [정정 — Stage 2 조사] 근본원인 수정: TypesetEngine + 표 높이 과소측정

Stage 1 의 "발신명의 PAGE 앵커 배치" 진단은 **부분적으로 틀렸다**. Stage 2 구현 중 발견:

### 1. 실제 페이지네이션 엔진은 TypesetEngine (Paginator 아님)

`rendering.rs:2245` — 기본 페이지네이션은 **`TypesetEngine`**(`src/renderer/typeset.rs`).
`Paginator`(`pagination/engine.rs`)는 `RHWP_USE_PAGINATOR=1` 일 때만 쓰는 fallback.
→ 페이지네이션 수정은 **TypesetEngine** 에 해야 함.

### 2. 진짜 원인: 발신명의 표 높이 과소측정 (선언 351px vs 측정 302px)

`RHWP_USE_PAGINATOR=1` 디버그(36387725):
```
pi=11(발신명의) vert=Page valign=Bottom th=302.3px ch(본문)=641.5 avail≈990~1011
→ 641.5 + 302.3 = 943.8 < avail → 1쪽
```
그러나 발신명의 표의 **선언 크기 height = 26353 HWPUNIT = 351.4px**(dump `size=49891×26353`).
`measured.get_table_height` 는 셀 내용 기반 **302.3px** 로 과소측정.

**선언 351.4px 를 쓰면**: 641.5 + 351.4 = **992.9 > 990.2 → 분할 → 2쪽**(한글 일치).

→ 근본 = **비-TAC 발신명의 표의 pagination 높이가 선언 크기보다 작게 측정됨**.
한글은 표를 선언 높이(351px)로 렌더하나 rhwp pagination 은 내용높이(302px)로 fit 판정.

### 3. 수정 방향 (TypesetEngine, 고위험)

비-TAC 표(특히 발신명의 등 고정크기)의 page-fit 판정에 **선언 height(common.height) 와
측정 height 중 큰 값** 사용. 단 전 표 영향 → +1쪽/붕괴 양방향 회귀 위험 → 통제셋 게이트 필수.
(height_measurer 의 non-TAC raw_table_height 가 선언 height 무시 — `height_measurer.rs:1256`.)

### 교훈
- Stage 1 의 권위 PDF 분석(발신명의가 페이지 2)은 옳았으나, **메커니즘**(앵커 배치 vs 표
  높이 측정)을 오인. 디버그로 실제 엔진·수치 확인이 추측을 정정.
- RHWP_USE_PAGINATOR 이중 엔진 구조 주의: 수정 전 어느 엔진이 기본인지 확인 필수.

---

## [정정 2 — 최종] 다요인 확정 + is_hwp3_origin 오탐지 (통제셋 측정)

표 높이 과소측정 가설도 **반증**(measure_table 디버그: footer raw=302.3=common_h=302.3, 정확).
per-paragraph 누적 추적(typeset.rs:1648 루프)으로 확정한 다요인:

### 요인 A: is_hwp3_origin 오탐지 (8건 해소, 실제 버그)

- avail=1011.6px = body 990.2 + **pagination_tolerance 21.4px**(`page_layout.rs:148`).
- tolerance 출처: `is_hwp3_origin`(hwpx/mod.rs:181) = `hwpml_version=="1.4"` → 1600 HU tolerance(Task #554, 한글97 모방).
- **그러나 36387725 version.xml: `application="Hancom Office Hangul" appVersion="11"`(한글2022) major=5 minor=1 = 네이티브 HWP5.1**. `xmlVersion="1.4"`는 HWPML 스키마 버전일 뿐 HWP3 변환 지표 아님 → **오탐지**.
- 통제셋: `is_hwp3_origin=false` 강제 시 **−1쪽 29→21(8해소), 일치 60→66(+6)**, 단 +초과 3→5(2회귀).
- 채택 게이트(개선−회귀>0) 충족(+6)이나 2회귀(36382819·36395325, 네이티브인데 보정이 우연히 맞던 케이스). 전 코퍼스 HWP3 보정 제거라 baseline/붕괴 회귀 검증 필수.

### 요인 B: footer 콘텐츠 누적 부족 (21건 미해소, 36387725 등)

- 36387725: ch(본문)=627.5 + footer 302.3 = 929.8 < body 990.2 → tolerance 없이도 1쪽.
- 즉 rhwp 콘텐츠가 한글보다 ~60px 짧음(요인 A와 별개). 본문은 저장 LINE_SEG라 per-line은
  동일한데도 누적 부족 — 미규명(발신명의 앵커 위치/gap 또는 분산 미세차 추정).
- 36387725·36390093·36392061·36394966 등 footer 보유 문서 다수가 이 부류.

### 종합 판정

−1쪽 갭은 **최소 2개 독립 메커니즘**의 다요인. 단일 surgical fix 불가.
- 요인 A(is_hwp3_origin)는 **독립적으로 가치 있는 실제 버그**(네이티브 HWPX에 부당한 HWP3
  보정 전반 적용) — 정밀 탐지(application 필드로 네이티브 식별)로 정식 수정 권고.
- 요인 B는 더 깊은 layout-fidelity 조사 필요(고난도).

산출물: `output/poc/render_gate_nohwp3.tsv`(요인 A 측정).

---

## [정밀 탐지 시도 — 판별자 부재 확정]

요인 A(is_hwp3_origin 오탐지)의 정밀 탐지를 시도했으나, **진짜 HWP3 변환본과 네이티브
한글2022 HWPX 가 메타데이터상 구별 불가**:

| 속성 | HWP3 변환본(hwp3-sample) | 네이티브(36387725) |
|------|------|------|
| header `<hh:head version>` | **1.4** | **1.4** (동일) |
| version.xml application | Hancom Office Hangul | Hancom Office Hangul (동일) |
| secCnt | 1 | 1 (동일) |
| BinData isEmbeded | 1 (임베드) | 1 (임베드) (동일) |
| appVersion | 10/13 | 11 | (저장 앱 버전일 뿐, HWP3 지표 아님) |

→ `is_hwp3_origin = (head version "1.4")` 는 **둘 다 매칭**. tolerance 는 변환본엔 정답,
네이티브엔 −1쪽 유발. **단순 메타데이터 판별자 없음**.

### 함의
- 정밀 탐지(요인 A 깔끔한 수정)는 **메타데이터로 불가능**. 가능한 길:
  1. 콘텐츠/레이아웃 아티팩트 기반 HWP3 탐지(복잡·취약).
  2. tolerance 자체 재검토 — 변환본이 +1 로 렌더되는 **근저 버그**(요인 B 동류)를 고쳐
     tolerance 자체를 제거(난이도 높음, 요인 B 와 수렴).
  3. is_hwp3_origin 제거(net +6) 수용하되 hwp3-sample baseline 시각 회귀 점검.
- **결론**: 요인 A 는 요인 B(footer 콘텐츠 누적)와 마찬가지로 근저 layout-fidelity 와
  얽혀 있어, 깔끔한 정밀 수정 불가. -1쪽 갭 전체가 **단일/저위험 fix 없는 다요인 심층 문제**.

---

## [후속] 요인 A → 별도 이슈 #1608 등록

요인 A(is_hwp3_origin 오탐지)는 −1쪽 갭과 별개로 **네이티브 HWPX 전역에 부당한 HWP3 보정을
적용하는 독립 버그**이므로 별도 이슈로 분리: **edwardkim/rhwp#1608**.

Task #1600(−1쪽 갭)은 요인 A(#1608) + 요인 B(footer 누적, 미규명)의 다요인 문제로,
통제셋·게이트(`tests/fixtures/render_page_controlset.tsv`, `tools/render_page_gate.py`)를
자산으로 보존하고 실제 수정은 #1608 및 후속 layout-fidelity 조사로 이관.

---

## [해소] 요인 A — Task #1608 완료 (tolerance 제거)

`is_hwp3_origin = (head version == "1.4")` 오탐지를 제거(파싱 시점 HWP3 tolerance 부여 삭제).
통제셋 일치 60→66(net **+6**), −1쪽 29→21(8 해소), 회귀 2건(36395325·36382819, 네이티브
인데 부당 tolerance 가 우연히 정답을 맞추던 케이스 = 요인 B 잔존). HWP3 변환본
(hwp3-sample-hwpx) 16→16 무변동. 전 회귀 게이트 통과. 상세 `mydocs/report/task_m100_1608_report.md`.

**요인 B(footer 콘텐츠 누적 부족, 21건)는 여전히 미규명** — 별도 layout-fidelity 조사 대상.

---

## [해소] 요인 B — Task #1611 (footer Page+Bottom page-fit 정합)

요인 B 의 **지배 메커니즘 규명·수정**(Task #1600 "단일 fix 불가" 판정 부분 정정):
발신명의 footer(`VertRelTo::Page`+`valign=Bottom`+TopAndBottom)를 TypesetEngine 이
① stored vpos 미동기화(Paper 만 처리) ② 측정높이(302.3) 로 fit(선언 351.4 무시) → page-fit
~60px 과소. `typeset.rs` 에 Page+Bottom 블록의 vpos 동기화 + 선언높이 fit 추가.

통제셋 일치 66→72(net **+6**), −1쪽 21→12(9 해소, 3 회귀). 요인 A(#1608)+B(#1611) 누적
60→72(+12). 상세 `mydocs/report/task_m100_1611_report.md`.

---

## [정정/특성화] 잔여 −1쪽 12건 — Task #1612

직전 "대형 tac 표 과소측정(36398709 등)" 판단은 **dump-pages 메트릭 아티팩트로 정정**:
`compute_hwp_used_height` 가 누적 vpos 를 per-page `used` 와 비교해 다페이지에서 diff 가
~800px/page 누적 증가(−3300px 등) → 과소측정 오판. 페이지 시작 vpos 차감으로 메트릭 정정
(진단 전용, 페이지수 불변).

잔여 12건 실제 특성: **단일페이지 footer 8건(본문 누적 ~20~43px 부족, razor-thin) +
다페이지 4건**. 본문 per-line 은 저장 LINE_SEG(한글 동일) → inter-paragraph gap/spacing
미세차 = **Task #1600 하드코어, 단일 surgical fix 부재**. 고위험·저마진(net>0 보장 어려움)으로
코드 수정 보류·특성 보존. 상세 `mydocs/report/task_m100_1612_report.md`.

---

## [규칙 부재 확정] 잔여 footer 8건 — Task #1616 (역공학 부정 결과)

razor-thin 8건의 footer 독립-쪽 배치 규칙을 통제셋(한글 정답) 71건 footer 특징으로 역공학
시도. **footer 기하(위치·크기·headroom·body_fill·valign 앵커)로 push/keep 분리 불가** 확정:
한글 PDF 검증 반례 **36389575(bottom 927, 선언 357)=PUSH(2쪽) ↔ 36393727(bottom 929,
선언 386)=KEEP(1쪽)** — 기하 거의 동일·PUSH 쪽 footer 더 작은데 판정 반대. 한글 결정 요인은
footer 기하 밖(본문 line-fill / 한글 내부 페이지-채움 계산)이라 stored IR 로 도출 불가.

→ **코드 수정 없음.** 임계 수정은 반례 오분류 + 전 코퍼스 footer 대량 회귀. 알려진 한계 확정.
−1쪽 시리즈 최종: 통제셋 일치 60→72(+12, 78.3%), 잔여 12건은 한글 내부 알고리즘 재현 필요.
상세 `mydocs/report/task_m100_1616_report.md`.

---

## [최종 판정] 잔여 12건 전수 분석 — 양방향 razor-thin, 단일 fix 부재

footer 8건(#1616 규칙 부재) + **다페이지 4건** 전수 분석 결과 모두 razor-thin/분산 밀도:
- 36399105·36399374: per-page diff ~0~8px(footer 8건과 동류 razor-thin).
- 36398709: rhwp p5(pi=70)가 한글 p5+p6 을 흡수 — **단일 mis-paginated 요소 아님, 분산 누적**.
  (#1612 메트릭의 다페이지 큰 diff +71~166px 는 tac 표 vpos 베이스 잔여 artifact.) 36392557 동류.

**결정적 논거 — 양방향성**: 통제셋은 −1쪽 12건 외 **+초과 8건**(+1:6, +2:2, rhwp 가 한글보다
*덜* 채움)도 보유. 즉 rhwp 의 요소높이 모델은 한글과 **양방향**으로 미세하게 어긋난다(어떤 건
조밀, 어떤 건 성김). 따라서 단일 systematic 보정(줄간격/spacing 일괄 조정)은 한 방향을 고치면
반대 방향을 악화 → net 개선 불가. 깔끔한 surgical fix 도, 일괄 보정도 부재.

**유일한 잔여 경로**: 한글의 line-level 페이지-채움 알고리즘(폰트 메트릭·줄높이 반올림·문단
간격 처리)을 per-line 으로 재현 — 대규모·고위험·불확실. 현 element-summation 아키텍처로는
78.3%(72/92)가 통제셋 실질 상한. **권고: 알려진 한계로 수용, 추가 투자는 한글 레이아웃 엔진
재현 의지가 있을 때.** 산출물 `output/poc/pagebreak_diverge.py`(분기 분석).

---

## [경로 A 1단계] vpos-추종 페이지네이션 타당성 — Task #1618

vpos-추종(한글 저장 분할 따라가기) 타당성을 위해 LINE_SEG vpos-reset 신뢰도 전수 분석
(`examples/vpos_reset_analyze.rs`). **순수 vpos-추종 비타당 확정**:
- vpos-예측 vs 한글(통제셋): 43.5%(현행 78.3% 후퇴), delta 전부 음수(과소).
- vpos-예측 vs rhwp(코퍼스 16,600): 일치 74.2%, 과소 25.8%, **과대 0.0%**(신뢰 하한이나 누락).
- 과소 원인: **표 내부 분할(PartialTable)은 IR 단일 컨트롤이라 vpos 리셋 없음** (big_table 보유
  63% 과소 vs 無 18%).

**잔여 12건 재조준**: 하이브리드라도 footer 8(앵커, vpos 무관)·다페이지 4(표 row-split 측정,
vpos 무관)을 직접 해소 못함. 잔여 레버는 ① 표 row-split 측정(4건 추적 가능) ② footer 규칙(불가).
vpos-추종은 잔여-12 타깃이 아닌 광역 body 충실도 장기 과제. 상세 `mydocs/report/task_m100_1618_report.md`.

### [Stage 2] 표 row-split 측정 분석 — 가설 반증

Stage 1 권고(다페이지 4건 = 표 row-split 측정)를 직접 검증 → **반증**. 4건 전수 **PartialTable
(단일 표 페이지 분할) 0건**: 이들은 tac 표 시퀀스 + 표지(page1 big tac 표) 구조라 단일 표가
페이지를 넘지 않음. 한글은 페이지당 608~724pt 만 채우고(가용 798pt) tac 표/문단을 안 들어가면
통째 이동(여백 남김), rhwp 는 더 조밀 → −1. = **분산 razor-thin**(추적 가능 표-레버 부재).

**−1쪽 시리즈 종결**: 잔여 12건 전부 razor-thin(footer 8 규칙 부재 + 다페이지 4 분산 누적).
단일/추적 가능 fix 없음. 유일 경로는 한글 페이지-채움 알고리즘 전면 재현(대규모·양방향 회귀).
통제셋 실질 상한 **78.3% 확정, 종결**. 상세 `mydocs/working/task_m100_1618_stage2.md`.

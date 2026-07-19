---
kind: canonical
status: active
canonical: mydocs/tech/hancom_native_line_semantics.md
last_verified: 2026-07-19
---

# 한컴 네이티브 라인 시맨틱 — 정합 모델 정리

> 한컴 워드프로세서가 라인/레이아웃을 조판하는 방식(형식 의미, formal semantics)을 **관측 가능한 정규 모델(canonical model)** 로 정리한다. 정답지(한컴 편집기 출력 PDF)와 라인세그 라운드트립을 근거로, 한컴 저장본의 조판 결과가 어떤 규칙을 따르는지와 그것을 rhwp 코어 모델에 대조한 함의를 담는다. 알고리즘 세부 구현이 아니라 **결과가 만족하는 규칙(behavioral spec)** 을 기술한다.

## 1. 라인 조판 의미 요약

- **커서 = (offset, 주소 모드)**. 주소 모드는 폭이 없는 논리 표현일 뿐이며 정규화 시 UTF-16 단위·run 정체성을 보존한다 → **모드는 지오메트리를 바꾸지 않는다**.
- **숨김 앵커 연속**(각주·머리말·꼬리말·HWPX 숨김 노드)은 **시각적 줄바꿈이 아니다**. 논리 체인의 연속일 뿐 라인 지오메트리에 관여하지 않는다.
- **라인 종결은 고정점(fixed point)** 이다. 종결점(endpoint)과 라인-로컬 폰트/자간 행렬 `M` 이 **한 관계로 동시에 결정**된다. full-em 으로 폭을 먼저 재고 그리디로 자르는 방식은 한컴과 **동등하지 않다** — 같은 명목 1400 글자가 줄에 따라 Hangul dx 915 / SPACE dx 560 으로 달라지는 것이 고정점 효과의 증거다.
- **라틴 파단 정책**: 어절 단위 유지(KEEP_WORD) / 하이픈 분리(HYPHENATION) / 문자 단위 분리(BREAK_WORD, ParaShape 속성) 세 정책이 있으며 char-shape 속성으로 선택된다.

## 2. 라인 종결·나눔

### 2.1 라인 종결
- 라인 종결은 **가용폭 내 최대 후보 종결점**을 고르는 관계로 정의된다 — 별도 수치 measure 자체가 목적이 아니라, 후보 종결점이 누적 폭 조건을 만족하는지 판정한다.

### 2.2 페이지 나눔 vs 단(段) 나눔
- 나눔은 **흐름 축(flow axis)** 에 따라 페이지 나눔과 단 나눔으로 구분된다. 세로 흐름 상태·단 상태에 따라 나눔이 어느 축으로 전진 가능한지가 게이트된다.
- **마지막 단에서는 단 나눔이 중립(무나눔)으로 강등**된다 (더 전진할 단이 없으므로).

## 3. 문단 페이지네이션 제약 (widow/orphan/keep)

수용 라인 수 `q` (프레임에 들어가는 줄 수)에 대한 유효 ParaShape 변환:

| 속성 | 변환 | 상태 |
|----|----|----|
| keep-lines | `q>0 → 0` (문단을 통째로 다음 프레임으로) | 확정 |
| widow/orphan | `n<4 → 0`; `q==1 → 0`; 선행경계 admissible → `q−1`; else `q`. (`n`=문단 총 라인 수) | 확정 |
| keep-with-next | `q==0` 이면 후속 체인을 순회하여 시작 위치로 rewind → `(시작,0)` 커밋 | 확정 |

- 세 변환 모두 폐형식(closed-form) 소유자 규칙과 **일치**한다.

## 4. 라인 측정·종결 모델

### 4.1 per-glyph advance (폰트 폭 보정)
- 원시 글자폭에 폰트별 보정 적용: **페이크-볼드 ≈ +5%**, 별도 굵기 ≈ +6%, **super/subscript ×0.64**.
- 스케일 폭 = **원시폭을 1em(=1000) 기준으로 정규화** (`raw × 1000 / fontEm`).

### 4.2 누적 펜(cumulative pen) → 글자별 advance
- 누적 펜 위치를 **차분**하여 글자별 advance 를 얻는다: `out[i] = cumulative[i] − cumulative[i−1]`. 서러게이트/CJK 쌍은 **0폭 동반 슬롯**.
- 언어별 자간 가산.
- 런 총 advance = `out[]` **합**(글자 카운트가 아니라 폭의 합).

### 4.3 고정점 행렬 M (justification slack 분배)
같은 명목 글자가 줄마다 dx 가 다른(915 vs 560) 이유:

- slack = `target − Σadv` 을 **현재 글자에 `slack/2`(round-toward-zero), 직전 비영 글자에 나머지** 로 분배(합 정확).
- 워드-스페이스 vs 자간을 각각 카운트하며, **마지막 gap 제외**(N글자 → N−1 gap).
- grid 모드는 각 advance 를 `ceil(raw/pitch)*pitch` 로 스냅 후 잔여 분배.
- 한글 **배분정렬** = 자간 gap 에 배분, **양쪽정렬** = 워드-스페이스에 배분. (정확한 워드:자간 배분 비율은 미확정 — open.)

### 4.4 grid / 원고지 / 금칙
- snap `ceil(raw/pitch)*pitch`.
- 원고지 em-center 오프셋: 한글 `+(pitch−1000)/2`, 그 외 `+(pitch−1000)`.
- **마침표+닫는큰따옴표는 한 셀 공유**.
- 파단점에서 최대 2 클러스터 후퇴(금칙 hanging). breakable/hanging 판정을 사용.

### 4.5 커밋 레코드 시맨틱 필드(개념)
라인 레코드에 기록되는 개념적 필드: endpoint, receiver, end-pos, flag, grid pitch, **줄폭**(=우측−좌측), **줄높이**(=baseline + ascent + descent), 셀 상태(start/end/overflow).

### 4.6 후보 적합(candidate-fit) 관계식
**`ACCEPT ⇔ 누적 advance(종결 후보까지) + 런 총 advance ≤ availableWidth`**. endpoint 선택 = 이 조건을 만족하는 **최대 후보**. `FIT-ACCEPT` 정리와 일치.

### 4.7 경로 분리 — CJK vs 워드/하이픈
- **라틴 워드/하이픈 경로**: 후보 생성 → 스케일 측정 → 파단 finalizer → fit 판정 (§4.6 을 이 경로가 소유). 긴 단어(예: "internationalization")·하이픈 후보.
- **CJK/공백 줄바꿈**: 경량 경로 = 위치별 CharShape 측정 → endpoint+가용폭으로 라인 커밋. 스케일 산출 미호출.

### 4.8 char-shape run 해소
- 문단은 **글자모양(char-shape) run 배열**을 가지며, 후보 endpoint 는 그 run 원소(CharShape)로 해소된다. 스케일이 CharShape 메트릭(Height / SizeHangul / RatioHangul)에서 advance 를 산출하고, 이 해소는 렌더 경로에서도 동일하게 쓰인다.

### 4.9 rhwp 함의 및 개정
과거 rhwp 는 (a) 폰트 폭 보정(bold/super-sub)을 텍스트 측정에서 누락, (b) 고정점 slack 분배를 후처리 별도 패스로 근사(엔드포인트 선택과 미결합 = 비고정점), (c) 누적-펜 항 부재를 허용오차 상수로 미봉했다.

> **개정 (항목 (c) 해소):** `src/renderer/composer/line_breaking.rs` 의 줄나눔을 그리디 토큰-필(허용오차 15 HU)에서 **§4.2/§4.6 정합 누적-펜 라인 종결**로 재구현했다. 라인-로컬 펜을 **정수 HWPUNIT 로 누적**하고 후보 종결을 **정확 `≤ available`(허용오차 상수 없음)** 로 판정한다. `LINE_BREAK_TOLERANCE` 및 보조 재계산을 제거하고 줄꼬리 금칙(§4.4)도 추가했다.
>
> **경험적 확증(라운드트립):** 정수 글자별 누적(`Σ to_hwp(advance)`)이 한컴 저장본과 정합(라운드트립 65/82 무변), 연속-px 누적(`to_hwp(Σpx)`)은 탭 라인을 어긋나게 함(lseg-05 회귀) → 한컴 펜은 §4.2 그대로 **정수 글자별 누적**임이 실증됨. 잔여 (a)(b)는 미해소.

## 5. 객체 배치(OP) — 구조 요약

- 배치 오브젝트는 **primary / secondary 두 개의 직교 extent**(스칼라 폭) 중 하나를 numeric class 로 택한다.
- 인라인 스텝 vs **outward 위임**(개체가 스스로 advance 반환) 이 admission 규칙으로 갈린다. treat-as-char 개체는 자기 폭을 advance 로 기여.
- 잔여: 각 numeric-class 값의 런타임 의미는 동작 관측을 요하는 미확정 항목.

## 6. 흐름-소유자/페이지네이션(FL) — 구조 요약

- **나눔 방향 분류**: 노드의 break 속성이 섹션/단/페이지 중 어느 방향으로 나눌지 결정한다. 마지막 단에서는 단 나눔이 중립으로 강등(§2.2).
- **적합/배치**: 수직 초과 술어(content-bottom < limit; 머리말/꼬리말 예약 높이 게이트), 다중 슬롯 배치(표셀/다단 조각화, 2D row/col), 무조건 커밋.
- **연속 게이트(각주·머리꼬리 예약)**: 이월 조각마다 후속 소유자 예약표에 예약하고 다음 노드에 연속을 표시한다 → **노트 기하는 소유자 admissibility 의 일부**(단순 paint 오버레이가 아님).
- 잔여: 섹션/다단 프레임 생성 본체, first/even/odd 머리꼬리 선택, 반복 표머리 실체화는 동적 게이트로 미확정. 후속/방향/적합 구조는 확정.

## 7. rhwp_core 함의 (가시 라인 지오메트리)

- `rhwp_core` 는 **정규 가시 라인 지오메트리 모델**(`VisibleParagraph` / `VisibleLine` / `VisibleFragment`)을 1급으로 두고, 문단 레이아웃을 단일 경로(logical item stream → visible projection → line fitting → alignment → VisibleParagraph)로 통일해야 한다.
- 편집 idempotence 는 **텍스트가 아니라 가시 라인** 기준으로 검사한다. `line_segs` 는 입력 사실일 뿐 provenance 분기가 아니다. 본문/표셀/각주/미주는 프레임만 다르고 같은 가시 라인 계약을 공유한다.
- §1 의 고정점 라인 종결·주소 모드·숨김 앵커 연속은 이 정규 경로가 재현해야 할 네이티브 기준이다.

## 8. 레인 완성 현황

| 레인 | 상태 |
|----|----|
| 라인 나눔/측정/정렬(EP + 폭보정 + M) | **확정** (§4) |
| 페이지네이션 소유자 제약(keep-lines/widow/keep-next) | **확정** (§3 + §6) |
| 객체 배치(OP) 구조 | **확정** (§5; 클래스 값의 런타임 의미만 잔여) |
| 흐름/후속/연속/예약(FL) 구조 | **확정** (§6; 프레임 생성 본체·머리꼬리 선택 잔여) |
| 도형/효과/3D, 폰트/글리프 | 완성 (`hancom_native_rendering_semantics.md` 참조) |

정규 레이아웃 시맨틱은 구조적으로 확보되었다. 잔여는 (a) 각 numeric-class 값의 런타임 의미, (b) 머리꼬리 선택·표머리 실체화 등 **런타임 동작 관측을 요하는** 항목이다.

## 9. 라운드트립 검증 — 모델 이관 판정

라인세그 라운드트립을 기준선으로 측정했다. 게이트 = `cargo test --lib lineseg`. `SectionLineSegReport.line_break_match_rate` = 한컴 저장본 `line_segs` vs rhwp `reflow_line_segs` 재계산 일치율.

**기준선 결과 (50 테스트 PASS, 회귀 없음):**
- **줄바꿈 100% 일치:** 모든 한글 전용, 모든 정렬(좌/우/중앙/양쪽/배분), 모든 폰트(바탕/돋움/굴림/맑은), 영문 단어·혼합 대부분, 다중크기, 혼합자간. → **실문서 지배 케이스는 이미 정합.**
- **줄바꿈 불일치(니치):** 라틴/숫자/한영/구두점 **무공백 연속 런**(char-level break)에 집중.
- **실문서 파단:** KTX 29%(줄수까지 불일치), 수식 문서 60%, 홍보 문서 88%. 표/필드 계열은 줄바꿈 100%·필드값만 차이(파단 아님).

**폭-델타 정밀 진단 (가용폭 566.9px, Windows Batang em=1024):**
| 샘플 | 한컴 L0 파단 | rhwp reflow | 폭 편차 | 방향 |
|----|----|----|----|----|
| 라틴 무공백 | 101자 | 99자 | +11px(2%) 과대 | 2자 early |
| 숫자 무공백 | 101자 | 88자 | +113px(20%) 과대 | 13자 early |
| 한영 무공백 | 80자 | 81자 | 소 과소 | 1자 late |
| 구두점 | 80자 | 75자 | +52px(8%) 과대 | 5자 early |

편차의 **방향·배율이 비일관** → 계통적 모델 오차가 아니라 **폰트별 per-glyph 메트릭값 차이**로 진단.

**판정 (라운드트립 증거 기반):**
1. rhwp 라인 나눔 **모델은 이미 건전** — 실문서 지배 케이스 100%. §4 의 시맨틱은 rhwp 통과 동작에 **대체로 이미 반영**됨.
2. 잔여 불일치 = 니치 무공백 런의 per-font 메트릭 정밀도 + condense 효과(실세계 영향 ~0) + 소수 실문서 엣지(객체/컨트롤 특정, 코어 텍스트 모델 아님).
3. §4 3개 항목(bold/super-sub 폭보정, M 반쪽 slack, 누적 항) **전면 이관은 정당화되지 않음** — 이미 정합된 시스템을 회귀 위험에 노출시키며 니치 이득. super/sub `×0.64` 만이 깨끗한 누락이나 현 픽스처에 부재로 **측정 불가**.

**결론:** 라운드트립 테스트가 그 자체로 **"이관 불필요/코어 모델 건전"** 을 실증한다. 실가치 후속: (i) 특정 폰트 per-glyph 메트릭 정밀도, (ii) 수식 박스-모델, (iii) KTX 줄수 발산(컨트롤/객체 특정).

## 10. 반각 셀·한영 바인딩 — 정답지(PDF) 실증 정정 (2026-07-17)

§9 의 "무공백 런 잔여는 정합불가/니치" 결론은 **성급했다**. 한컴 권위 PDF(`pdf/re-0[3-6]-*-2022.pdf`) 실측으로 **실제 한컴 시맨틱**을 확정하고 rhwp 를 정합시켰다. 라운드트립 65→68/82(+3, 무회귀), 렌더·golden SVG 무손상.

### 10.1 함초롬바탕 ASCII = **fit 반각 셀(0.50em) / render 비례** (fit-render 분리)
- **핵심 증거:** 라틴 a–z 85종(advance 상이)과 숫자가 **둘 다 정확히 85 글자/줄**(42520 HWPUNIT 컬럼). 비례폭으론 불가능, **균일 0.50em(500 HWPUNIT) 반각 셀**로만 성립(85×500=42500 ≤ 42520).
- **렌더는 비례:** 권위 PDF 의 글자별 advance 실측은 0.241~0.867em 로 **비례**. 즉 **한컴은 함초롬바탕 ASCII 를 비례로 그리되 줄나눔 fit 만 0.50em 반각 셀로 계산**한다.
- **rhwp 정정:** fit 전용 진입점 `estimate_text_width_unrounded` 만 반각 셀 적용(`haansoft_latin_fit_override`), 렌더 경로 불변. "0.42em/101 글자" 오해는 **선두 inline control 클러스터**로 인한 파싱 착시였고 실제는 **85 글자/0.50em**.

### 10.2 반각 셀 스코프 = **순수 ASCII 무공백 런만**
- 혼합 런(`한글English…`, `가,나.다!…`)은 natural(실 advance)로 fit. 반각 셀을 혼합 런에 쓰면 회귀.
- 구현: 토크나이저가 글자-인덱스 `ascii_run_mask`(maximal 무공백 런이 전부 ASCII 인지)를 1회 계산 → 토큰폭·char_widths 가 동일 스코프 공유.

### 10.3 한글→라틴 바인딩 (함초롬바탕 한정)
- 한컴은 **한글 바로 뒤 라틴 어절을 분리하지 않는다**(`글English` 한 단위로 다음 줄 이월). 바인딩 없이는 어긋남.
- **스코프 주의(실증):** 바탕/돋움/맑은 고딕의 한영 무공백 혼합은 한컴이 **바인딩 없이** 저장 → 폰트-무관 바인딩은 회귀. **바인딩은 함초롬바탕(HCR Batang) 문서에만 발화**하도록 게이트.

### 10.4 잔여 (구두점, 1-글자)
구두점 무공백은 자연폭 정정으로 ~5-글자 → **1-글자(ts=−1)** 로 축소. 잔여는 줄머리 금칙(구두점 −1 pull-back, §4.4) 또는 임베디드 구두점 폭과 실 HCR(≈320 HU) 간 미세차 — 정밀 후속. **핵심 교훈: "정합불가/니치" 로 넘기지 말고 정답지(PDF)로 검증하라.**

## 11. always-compute 대수술 결과 — 저장 line_seg 폴백 삭제 (2026-07-17)

**지시:** "저장 line_segs 를 레이아웃 소스로 쓰는 폴백을 삭제. rhwp 가 **항상** 스스로 line_seg 를 계산(true HNC line break)."

### 11.1 달성 — 줄나눔 엔진은 authentic
- 로드 경로(`reflow_zero_height_paragraphs`)가 **모든** 본문·셀 문단을 무조건 reflow. 저장 seg 는 파서 보존·라운드트립 검증 전용. render 의 "저장 seg 있으면 그걸 쓰고 없으면 계산" 폴백 **삭제 완료**.
- **실증:** `hwpx_sample2`(1452문단) reflow 가 저장 seg 와 **줄나눔 완전 일치**(over_paras=0). 큐레이션 라운드트립 68/82.
- 엔진: 정수 HWPUNIT 누적 펜(§4.2), 정확 `≤ available` fit(관용 밴드 없음), 줄끝 금칙 후퇴, 함초롬바탕 fit 반각셀(§10) — greedy 아님.

### 11.2 잔여 실패(9)는 **줄나눔 아님** — 직교 레이아웃/메트릭
줄나눔 greedy 문제는 해소됐고, 남은 실패는 저장 seg 가 인코딩하던 **복합 레이아웃**을 순수 계산이 아직 재현 못 하는 지점(각각 별도 서브프로젝트):

| 군 | 본질 | 판정 |
|----|------|------|
| **표 높이** | 블록 TAC 표 문단 높이가 한컴 저장 행높이와 정합해야 함. +1쪽은 줄나눔 아닌 **높이 드리프트** | 전용 표-레이아웃 |
| **메트릭 정밀** | 폰트별 실 advance 미세차(영문 over, 국문 under — 방향 반대 = 단일 편향 아님) | 폰트 메트릭 정밀(delicate) |
| **플로트 밴드** | 밴드 지오메트리(cs/sw)를 저장 seg 가 힌트. 그림 기하 도출은 config 다양성상 부분 도출이 회귀 | 기하 도출 float 엔진 |
| **세로쓰기** | 중첩 셀 세로쓰기 열 계산(가로 엔진 부적용). 최상위 세로셀은 가드로 보존 | 세로 레이아웃 |
| **바탕쪽** | master page 대체 로직 | 직교 |

### 11.3 broad 구조 수정은 회귀 — 되돌림
`always-compute` 는 복합 레이아웃마다 규칙 추가가 필요한데, **넓은 규칙은 다른 문서를 회귀**시킨다(실증):
- **블록 표 선두줄:** 한 케이스 고침 but **통과중이던** 페이지네이션 회귀 → 되돌림.
- **인라인 개체 advance 주입:** 목표 케이스 못 고침 + 수식 정렬 회귀 → 되돌림.
- **교훈:** 표/플로트/세로 구조 수정은 authentic 이어도 **정확한 한컴 높이/기하 없이는 순회귀**. no-new-regression 원칙상 미검증 상태로 통과 테스트를 깨지 않는다.

### 11.4 유지된 정합(무회귀)
세로셀 가드(최상위 세로셀 보존), count-changed 계약(opt-in reflow 는 실제 바뀐 문단만 카운트 → 이미 reflow 된 로드 문서엔 no-op), compose 폴백 삭제 정합 테스트, LineSeg PartialEq.

## 12. release-test 잔여 — authentic 근본원인 규명 (2026-07-18)

직접 실측(정답지 PDF + 라운드트립 + 계측)으로 잔여 실패의 **진짜** 근본원인을 규명. 오진 2건 정정.

### 12.1 ✅ 표문단 2-seg — authentic 해결 (8→7)
블록 TAC 표 문단은 **표줄(lh=표높이) + 본문줄** 2-seg, 공백만이면 표줄 흡수 1-seg (저장 seg 실증). `line_breaking.rs` reflow_line_segs 에 구현(whitespace 가드가 핵심 — 공백에도 2-seg 만들면 페이지네이션 회귀). skip-perf 회귀 2264 passed/7 failed, 무회귀 확인.

### 12.2 표-줄 높이 Δ 공식 (authentic, orthogonal)
`table_line.line_height = Σ(row_heights) + table.inMargin.top + table.inMargin.bottom` (baseline=round(lh*0.85)). **common.height 아님**(비일관 저장). 현 rhwp 는 common.height 사용 → TAC 표당 282 짧음. **단 이는 표를 더 크게 만들어 +1page 를 악화**(orthogonal). HWP5 계열 Δ 는 미해결.

### 12.3 오진 정정 — +1page 는 폰트메트릭·높이 아닌 **페이지네이션-fill**
- 최초 진단(글자 advance 과대추정)은 **오진**: rhwp 는 sample2 의 84자 URL줄을 한컴과 **동일 폭에서 1줄**로 reflow(over-wrap 없음). 해당 문단 폰트 전부 582-메트릭 DB 존재. 폭은 문제 아님.
- 높이 누적도 아님: 총높이 rhwp 가 **더 짧은데** +1page. 즉 rhwp 가 페이지당 콘텐츠를 **덜** 채움 = **페이지네이션-fill 알고리즘 차이**(widow/orphan·keep-together·usable-height·object 분할 등). 심층 pagination 분석 필요.

### 12.4 잔여 authentic 근본원인 (미해결, subsystem 별 dedicated)
- **페이지네이션-fill (§12.3)**: sample2·바탕쪽·(추정)메트릭군.
- **float wrap band**: 밴드 기하 = cs=0, sw=col−picw−margin_left (3그림 실증). 뒤따르는 문단이 그림 세로범위 안일 때 밴드폭 re-reflow 필요(render 는 re-break 안함). following-para vertical-span carry 가 난점, config 다양성상 broad.
- **task81 vertical**: 저장 세로셀 중 렌더 실패 1개(≥2열 비단조). 최상위 세로셀은 가드 보존, 실패셀 원인 미규명.
- **inline obj break**: tac=true 개체(offset-gap 앵커) 폭 미반영 under-break. 폭 주입 무효과 — 앵커/주입지점 재검 필요.

**정직한 판정**: 잔여는 페이지네이션·float·vertical·inline 각 subsystem 의 심층 분석을 요하는 multi-session 작업이며, 일부(폰트/페이지네이션 정밀 정합)는 heuristic/fallback 없이는 근본 한계에 도달 가능하다.

# PR #360 검토 — Task #356: 페이지 분기 오버플로 (vpos 권위값/spacing 누적 오차)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#360](https://github.com/edwardkim/rhwp/pull/360) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| base / head | `devel` ← `local/task356` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | **BEHIND** (devel 보다 뒤) |
| 이슈 | [#356](https://github.com/edwardkim/rhwp/issues/356) |
| 변경 통계 | +1235 / -10, 17 files |

## 결함 요약

`samples/2022년 국립국어원 업무계획.hwp` 페이지 3 의 본문이 페이지 박스(933.5 px) 초과하여 푸터가 SVG 영역 밖으로 밀려남.

근본 원인 (PR 본문):
1. **인접 문단 간 vpos 리셋 신호 미사용** — HWP 가 권위값으로 새 페이지에 보낸 문단(pi=40 vpos=500, prev pi=39 vpos=66281)을 px 누적이 무시
2. **TAC 표 측정 누적 drift** — 페이지 3 의 TAC 표 4개 측정값이 HWP 권위값보다 누적 140 px 작게 산정

## 변경 내용

### 1. `detect_inter_paragraph_vpos_reset` 헬퍼 신설

`pagination/engine.rs` + 단위 테스트 6개.

조건: 둘 다 line_segs 보유 + 같은 column_start + cur.first.vertical_pos < prev.first.vertical_pos.

### 2. 페이지네이션 엔진 통합
- `pagination::engine` 메인 루프: `process_page_break` 직후 헬퍼 호출
- `typeset.rs` Task #321 위치: 기존 strict 트리거 유지 + 헬퍼 보조 트리거 추가

### 3. TypesetState 확장 + HWP 권위값 검증
- 신규 필드: `page_vpos_base: Option<i32>`, `suppress_next_inter_para_advance: bool`
- 메인 루프: `current_items[0]` 으로부터 page_vpos_base 유도
- 가드: `prev_was_partial` (PartialParagraph 직후) / `suppress_next_inter_para_advance` 시 inter-para 검사 스킵
- typeset_paragraph 보정 (a) HWP 권위값 overflow → 강제 advance, (b) drift 보정

## **메인테이너 작업과의 중복 분석** ★

본 PR 의 변경 영역이 **메인테이너 누적 작업 (Task #359, #361, #362) 과 겹침**:

### 겹치는 영역
| 영역 | PR #360 | 메인테이너 |
|---|---|---|
| typeset.rs vpos-reset 가드 | `detect_inter_paragraph_vpos_reset` 헬퍼 + Task #321 위치 보조 트리거 | Task #321 의 strict (`cv==0 && pv>5000`) + Task #359 의 단독 항목 페이지 차단 |
| typeset.rs fit 안전마진 / 누적 | drift 보정 | Task #359 fit 누적 (`total_height`), Task #361 안전마진 |
| typeset.rs PartialTable / wrap | (없음) | Task #361 PartialTable 안전마진 + Task #362 wrap-around |
| pagination/engine.rs | `detect_inter_paragraph_vpos_reset` + 헬퍼 호출 | Task #361 finalize_pages NewNumber + PR #366 PageNumberAssigner |

### 결과 비교 — 동일 샘플 회귀

| 샘플 | PR #360 베이스라인 | PR #360 적용 | **현재 devel (메인테이너)** |
|---|---|---|---|
| aift | 74p / 30 | 83p / 4 | **77p / 3** ✅ (메인테이너 더 좋음) |
| exam_eng | 8p / 0 | 8p / 0 | **11p / 0** (페이지 수 다름) |
| exam_math | 20p / 0 | 20p / 0 | **20p / 0** |
| 2010-01-06 | 6p / 0 | 7p / 0 | **6p / 0** ✅ (메인테이너 더 안정) |
| **2022년 국립국어원 업무계획** | 35p / 5+ | **37p / 0** | **샘플 미보유** (PR #360 가 추가) |

→ 메인테이너 작업이 **aift / 2010-01-06 에서 PR #360 보다 좋거나 동등**. 그러나 PR #360 의 핵심 샘플 (`2022년 국립국어원 업무계획.hwp`) 은 devel 에 없음 → **이 샘플의 결함이 메인테이너 작업으로 해결됐는지 측정 불가**.

### 본질적 의문

PR #360 의 핵심 샘플은 메인테이너 작업이 다루지 않은 새 결함 가능성. **샘플 추가 후 메인테이너 작업 적용 시 결함 잔존 여부 확인 필요**.

## 처리 방향 후보

### 옵션 A: PR #360 샘플 + PDF 만 흡수 + 메인테이너 작업으로 검증

근거: 메인테이너 작업 (Task #359, #361, #362) 가 같은 영역의 결함을 광범위하게 정정. PR #360 의 핵심 샘플 (`2022년 국립국어원 업무계획.hwp`) 만 추가 후 메인테이너 작업이 이 결함을 해결했는지 측정.

- 결함 잔존 시: **옵션 B / C** 진행
- 결함 해결 시: PR #360 close + 샘플 + 문서만 흡수

### 옵션 B: PR #360 의 일부 코드만 흡수 (체리픽 부분)

근거: PR #360 의 `detect_inter_paragraph_vpos_reset` 헬퍼는 Task #321 의 strict 가드보다 일반화된 시멘틱. aift overflow 를 30→4 로 줄였지만 메인테이너 작업이 30→3 으로 더 줄임. 이 영역의 일부 가치는 메인테이너 작업이 흡수했을 가능성.

- 메인테이너 작업이 다루지 않은 부분만 식별 후 흡수
- 작성자 attribution 보존

### 옵션 C: PR #360 정상 머지 (rebase + 충돌 해결)

근거: PR 의 검증이 본 샘플에 대해서는 명확. 다만 메인테이너 작업과의 충돌 해결 + 통합 회귀 검증 필요.

문제:
- 메인테이너 작업과 동일 영역 변경이 많아 충돌 폭 큼
- 메인테이너 작업 위에 PR 변경이 누적되면 시멘틱 중복 가능성
- 통합 효과 (kps-ai, k-water-rfp 등 다른 샘플) 검증 필요

### 옵션 D: PR #360 close + 별도 task 로 핵심 샘플 결함 정정

근거: PR 의 변경 폭이 크고 메인테이너 작업과 충돌 폭이 큼. 핵심 샘플 (`2022년 국립국어원 업무계획.hwp`) 결함만 별도 task 로 정정.

## 권장

**옵션 A → 결과에 따라 D** 권장:
1. PR #360 의 샘플 + PDF 만 먼저 추가 (`samples/2022년 국립국어원 업무계획.hwp` + .pdf)
2. 메인테이너 누적 작업 (Task #359, #361, #362, PR #366, #371, #373) 효과 측정
3. **결함 해결 시**: PR #360 close + 작성자 기여 인정 + 샘플 흡수만 보고
4. **결함 잔존 시**: 별도 task 로 잔존 결함만 정정 (PR #360 의 일부 코드 참고 가능)

이유:
- PR #360 의 변경 폭이 크고 메인테이너 작업과 충돌
- 본 샘플의 결함 자체가 메인테이너 작업으로 이미 해결됐을 수 있음
- 잔존 결함이 있다면 PR #360 의 시멘틱을 통째로 흡수하기보다 메인테이너 작업과의 통합 가치만 추출

## 다음 단계 — 작업지시자 결정

**검증 우선** 권장 — 옵션 A 의 1, 2 단계 (샘플 추가 + 측정) 만 먼저 진행.

진행 시:
1. PR #360 의 샘플 + PDF 만 cherry-pick (코드 변경 제외)
2. 메인테이너 작업이 적용된 devel 에서 페이지 수 + LAYOUT_OVERFLOW + 시각 측정
3. 결과에 따라 옵션 D (close) 또는 옵션 B (부분 흡수) 결정

## 참고

- 이슈: [#356](https://github.com/edwardkim/rhwp/issues/356) (OPEN)
- PR: [#360](https://github.com/edwardkim/rhwp/pull/360) (OPEN, BEHIND)
- 메인테이너 작업: Task #359, #361, #362, PR #366, #371, #373

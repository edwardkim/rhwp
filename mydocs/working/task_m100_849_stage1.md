# Stage 1 보고 — Task #849 (M100) — 진단 (수정 방향 변경: 밴드 높이 정합 → 단 유형 게이트)

상태: **진단 완료 — 소스 수정 없음(검증용 실험 코드는 워킹트리에 시범 적용, 결과 확인 후 본 보고 승인 시 정식화)**. 진단 결과 #849 의 실제 원인·수정은 수행/구현계획서가 가정한 "밴드 높이 산출 정합"이 아니라 **`start_new_column_band` 의 단 유형 게이트** 임.

## 1. 진단 — exam_math 회귀의 진짜 원인

`samples/exam_math.pdf` (한컴 2022, 20페이지) 의 문제 배치:
- 페이지 2 = 문제 5, 6, 7
- 페이지 3 = 문제 8, 9, 10
- 페이지 4 = 문제 11, 12

#846 적용 후 rhwp 페이지 3 = 문제 8, 9, 10, **11, 12** (단 0 = pi 68~89 = 문제 8·9, 단 1 = pi 90~91 = 문제 10, 단 2 = pi 92 = 문제 11, 단 3 = pi 96 = 문제 12). → 문제 11·12 가 페이지 4 가 아니라 페이지 3 으로 당겨짐. 페이지 수 18→11.

원인: pi=92(`11.시각...`)는 `[단나누기]`(`ColumnBreakType::Column`) 를 가지며 zone 의 **마지막 단**(col 1) 직후 등장 → #846 의 `start_new_column_band` 가 호출되어 페이지 3 에 새 2단 밴드(zone_y_offset=710)를 만들어 문제 11·12 를 거기 배치.

## 2. 핵심 발견 — 단 유형(`ColumnType`)에 따라 다름

| 문서/zone | `ColumnDef` 유형 | 마지막 단 `[단나누기]` 시 한컴 동작 |
|-----------|------------------|-----------------------------------|
| `shortcut.hwp` "보기"/"입력" 2단 (pi=82 `[다단나누기]`) | **배분(Distribute)** — `dump`: `단정의: 2단, 유형=배분` | 같은 페이지에 새 (배분) 밴드. 콘텐츠를 두 단에 균등 배분하므로 밴드가 작음(6줄≈106px) → 페이지에 여유 大 |
| `exam_math.hwp` 2단 (구역 0 `ColumnDef`) | **일반(Normal, 신문형)** — `dump`: `단정의: 2단, 유형=일반` | **같은 페이지 새 밴드를 만들지 않고 새 페이지**. (신문형은 단 0 을 채우고 단나누기로 단 1 로 넘어가는 방식; 마지막 단의 단나누기는 같은 페이지에 적층 안 함) |

rhwp 는 `current_zone_column_type` (Normal/Distribute/Parallel) 을 이미 추적 중 — `process_multicolumn_break`/section ColumnDef 에서 갱신. #846 의 `start_new_column_band` 는 이 유형을 보지 않고 **모든** 다단 zone 에 적용 → 신문형(`Normal`) zone 에서 회귀.

(추가 메모: `shortcut.hwp` "보기" zone 이 배분이라 pi 82~93 의 12문단이 한컴/ rhwp 모두 6/6 으로 균등 분할된 것이 이 진단의 단서였음. `exam_math` 는 pi 68~89(22문단) 가 단 0, pi 90~91(2문단) 가 단 1 — 명백히 불균등 → 신문형.)

## 3. 수정 방향 (수행/구현계획서 Stage 2 대체 — 간소화)

`paginate` 의 명시적 `Column` break 경로에서 `start_new_column_band` 호출 조건에 **`st.current_zone_column_type == ColumnType::Distribute`** 추가. `Normal`(신문형) zone 의 마지막 단 단나누기는 기존 `advance_column_or_new_page`(→ `push_new_page`) 유지. `Parallel`(평행) 은 별도 의미 — 본 타스크 범위 밖, 현 동작 유지.

→ 수행/구현계획서가 가정한 "밴드 높이 산출 정합(공유 헬퍼)"·`layout.rs` 연동은 **불요**. 배분 단의 밴드는 작아서 현재 vpos 기반 밴드 높이 산출로 충분히 정확함. (`process_multicolumn_break` 의 밴드 높이 산출도 그대로 둠 — 현 샘플들에서 문제 없음.)

## 4. 검증 결과 (시범 적용)

`typeset.rs` 호출 조건에 `&& st.current_zone_column_type == ColumnType::Distribute` 추가하고 측정:

| 샘플 | 한컴 PDF | baseline | #846 단독 | #849 게이트 적용 |
|------|----------|----------|-----------|------------------|
| `basic/shortcut.hwp` | 7 (2022) | 8 | 7 | **7** ✅ |
| `exam_math.hwp` | 20 | 18 | 11 | **20** ✅ (baseline·PDF 모두 초과/정합) |
| `21_언어_기출_편집가능본.hwp` | 15(2020)/16(2010) | 15 | 15(콘텐츠 시프트) | **15** ✅ (시프트 해소) |
| `cargo test` 전건 | — | 1232 pass | 1229/3 fail | **1232 pass / 0 fail** ✅ (`test_exam_math_page_count`/`test_539`/`test_548` 복구) |

## 5. 회귀 대상 (Stage 3 에서 정밀 확인)

본 변경은 **`Distribute` 단 zone 에서 마지막 단에 `[단나누기]` 가 오는 경우**에만 동작, 그 외(`Normal`/`Parallel` zone, 비-마지막 단)는 #846 이전 동작과 동일. 그래도 다단 샘플 전수 SVG diff: exam_*, k-water-rfp, 21언어, shortcut, 다단+표분할/목차/각주. + `cargo test` 전건(이미 통과 확인).

## 6. 단일 룰 판정

"마지막 단 + `[단나누기]` + `col_count>1` + zone 유형 == Distribute → 같은 페이지 새 밴드(들어갈 공간 시), 아니면 새 페이지" — 문서가 선언한 단 유형에 기반한 분기로, 휴리스틱 아님 (메모리 `feedback_rule_not_heuristic` 자문 불요).

---
승인 요청: §3 수정 방향(`start_new_column_band` 호출을 `ColumnType::Distribute` 로 한정)으로 진행해도 되는지 확인 부탁드립니다. 승인 시 구현계획서 Stage 2~4 를 본 진단에 맞춰 간소화 갱신하고 정식화(현재 시범 적용 코드가 곧 최종).

# Task #435 Stage 3: 일반 페이지 누적 부족 조사 — 완료보고서

> **이슈**: [#435](https://github.com/edwardkim/rhwp/issues/435)
> **브랜치**: `local/task435`
> **단계**: 3/5 (일반 페이지 누적 -100~-300px 조사)
> **완료일**: 2026-04-29
> **결과**: **조사 완료, 코드 변경 보류** — 가설 재검토 필요

---

## 통과 조건 재평가

원래 통과 조건:
- exam_kor.hwp 22 → 20 페이지
- 일반 페이지 \|diff\| < 100px

**달성 여부:**
- ❌ 페이지 수 22 유지 (코드 변경 없음 — 가설 재검토 후 보류)
- ❌ \|diff\| 미개선

**조사 결과**: 원래 가설 ("표/도형 후 컬럼 잔여 공간 산정 부족") 은 **사실이 아님**. `diff = used_height - hwp_last_line_bottom` 메트릭은 typeset cur_h 와 layout last-line bottom 의 좌표계 차이를 측정하는 것이지, "rhwp 가 채울 수 있는데 못 채운 잔여 공간" 을 측정하지 않음.

---

## 가설 재검토

### 원래 가설

> 표/도형 (Shape) 배치 후 컬럼 잔여 공간 산정이 한컴 대비 작게 산정되어 다음 paragraph 가 일찍 다음 컬럼/페이지로 밀려남.

### 실제 메커니즘

`RHWP_TYPESET_DRIFT` 분석 결과:

1. **typeset cur_h 누적 방식 (`typeset.rs:911`):**
   ```rust
   st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
   ```
   - 다단: `height_for_fit = fmt_total - trail_ls` (각 paragraph 의 trailing line_spacing 제외)
   - Task #391 의 의도된 동작 (exam_eng 정상 단 채움 복원)

2. **HWP 의 vpos 진행 방식:**
   - Paragraph N → N+1 거리 = N.vpos_h + N.trail_ls + N.sa + N+1.sb = fmt_total
   - 즉 한컴은 paragraph 당 `fmt_total` 만큼 진행

3. **`hwp_used` 계산 방식 (`compute_hwp_used_height`):**
   - 마지막 항목 마지막 줄의 `vertical_pos + line_height`
   - 이는 paragraph 의 vpos progression 끝점

**결론**: rhwp 의 `cur_h` 는 paragraph 당 `height_for_fit` (= fmt_total - trail_ls) 누적, hwp_used 는 마지막 줄의 vpos 끝점.
- Per-paragraph: `cur_h` 가 vpos 보다 trail_ls (~9.2px) 적게 진행 (= 한컴 대비 누적 under-count)
- Per-column 종료 시 diff = cur_h - hwp_last_line_bottom = -(누적 trail_ls 합)

**`diff = -250` 의 의미**: rhwp typeset 의 페이지네이션 budget 이 250px 까지는 "여유 있다" 고 보지만, 이는 **메트릭 차이**일 뿐 실제로 250px 추가 콘텐츠를 넣을 수 있다는 의미가 **아님**.

증거: 페이지 3 col 0 (29 items, used=957.8, hwp_used=1208) — typeset 은 957 까지 채움, 실제 layout 은 1208 까지 그림. 추가 paragraph 를 col 0 에 넣으면 layout 상 1211px (본문 한계) 를 초과하여 overflow.

---

## 실제 22→20 페이지 단축의 장애물

### 1. 섹션 1 페이지 14 — Square wrap 표 + col 1 reserve 누락

```
페이지 14 단 0: items=25, used=1225.8 (1211.3 초과 14px)
페이지 14 단 1: items=2,  used=64.3 (pi=48, pi=49 만)
```

**원인**:
- pi=28 (페이지 14 col 0 첫 paragraph) 에 body-wide TopAndBottom 표 컨트롤 없음 → `pending_body_wide_top_reserve` = 0
- 그러나 한컴은 섹션 마스터 페이지의 body-wide 표를 매 페이지에 적용 → col 1 reserve 적용
- rhwp 는 페이지 14 col 1 cur_h_initial = 0 → 더 많은 콘텐츠 가능하지만 실제로는 pi=48, pi=49 만 들어감 (다른 메커니즘으로 col 1 종료)

추가 분석:
- pi=47 col 0 마지막 항목 — Square wrap 표 + 3-line partial paragraph
- pi=47 다음 paragraph (pi=48) 는 col 0 cur_h=1225.8 (overflow) → col 1 advance
- col 1 시작 후 pi=48, pi=49 만 들어가고 pi=50 은 [단나누기] → 새 페이지

**근본 원인**: col 0 over-fill + Square wrap 표 처리. `paginate_text_lines` (`engine.rs:702-711`) 의 표 직후 trailing line_spacing 제외 로직 검토 필요. (Stage 4 영역)

### 2. 섹션 1 페이지 15 — 단일 컬럼 page 출력

```
페이지 15: 단 0 만 (단 1 없음), items=18, used=1077.6
```

**원인**:
- 섹션 1 의 단정의는 pi=0 에서 "2단" (단일 명시)
- 그런데 페이지 15 는 단일 컬럼으로 출력됨
- pi=50 [단나누기] 후 페이지 분기 시 rhwp 의 column_contents 가 1개만 생성

**근본 원인**: pagination 의 column 처리 로직 — [단나누기] 후 새 페이지에서 단 1 으로의 advance 가 누락되거나, 단 1 시작 paragraph (pi=67+) 가 별도 page break 로 다음 페이지로 밀림.

### 3. 섹션 2 페이지 18 — pi=11 split orphan-like

```
페이지 18 단 0: items=17, used=1209.1 (overflow 8)
페이지 18 단 1: items=2, used=158.8 (pi=11 line 1 + pi=12)
```

**원인**:
- pi=11 col 0 cur_h=1184.6 + fmt_total=49 > avail 1201 → split
- pi=11 line 0 col 0, line 1 col 1
- pi=12 fits col 1
- pi=13 [단나누기] → 다음 페이지

**근본 원인**: 섹션 2 col 0 cur_h 가 1184.6 까지 진행 — HWP 좌표 (pi=11 vpos=81349 = 1085 px) 보다 약 100 px 앞섬. 이 ~100 px 의 over-advance 가 어디서 발생하는지 추적 필요.

가능성: pi=2 (Square wrap 표 9 lines partial) 처리 시 cur_h 누적이 한컴 대비 과대.

---

## PDF 비교

| 섹션 | PDF 페이지 | rhwp Stage 2 | 차이 |
|---|---|---|---|
| 0 (공통) | 12-13 | 12 | 0 ~ -1 |
| 1 (화법과 작문) | 4 (page 13-16) | 5 (page 13-17) | **+1** |
| 2 (언어와 매체) | 4 (page 17-20) | 5 (page 18-22) | **+1** |
| **합계** | **20** | **22** | **+2** |

남은 2 페이지 단축은 위 3 가지 메커니즘 (페이지 14 col 1 under-use, 페이지 15 단일 컬럼, 페이지 18 orphan-like) 해결 시 가능.

---

## Stage 4 (조건부) → 본격 진입 결정 필요

원래 Stage 4 는 "Square wrap 표 over-fill 보호" 한정이었으나, 본 조사 결과 **Stage 4 는 더 넓은 범위로 확장 필요**:

| 영역 | 추정 영향 |
|---|---|
| Square wrap 표 over-fill (페이지 14 col 0) | 단 1 정상 채움 → 페이지 14+15 통합 가능성 → **-1 페이지** |
| 페이지 15 단일 컬럼 출력 (column 누락) | 페이지 14 col 1 과 통합 → **-1 페이지** |
| 섹션 2 페이지 18 orphan (pi=11 split) | 페이지 18+19 통합 또는 pi=11 split 차단 → **-1 페이지** |

3 가지 전부 해결 시 22 → 19 페이지 가능 (목표 20 도달 가능).

---

## 권고

본 단계에서 **코드 변경 없이 종료** 하고 다음 옵션 중 결정:

### 옵션 A: 현 상태 (22 페이지) 종료

- 24 → 22 (-2 페이지) 의 큰 진전 + 회귀 없음
- 마지막 페이지 푸터 `22/20` (여전히 미스매치)
- 잔여 작업은 별도 task 로 분리 (#393 옵션 A 적용 완료, 후속 이슈 신규 등록)

### 옵션 B: Stage 4 확장 진행

- Square wrap 표 over-fill 보호 + 단일 컬럼 출력 버그 + section 2 orphan
- 위험: 코드 변경 영역이 크고 회귀 위험 (Task #391, #386 등 누적 결과와 상호작용)
- 시간 소요 큼

### 옵션 C: Stage 4 부분 진행

- Square wrap 표 over-fill 만 정정 (페이지 14 의 단 0 overflow 해소)
- 효과 측정 후 추가 진행 여부 결정

---

## 다음 단계

**작업지시자 결정 필요**: 옵션 A / B / C 중 선택.

자동승인 모드라도 본 결정은 작업 범위 / 위험도 / 시간 측면에서 critical 하므로, 사용자가 명시적 결정을 내려야 함.

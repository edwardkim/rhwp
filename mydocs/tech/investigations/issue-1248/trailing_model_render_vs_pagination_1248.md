---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1248/README.md
last_verified: 2026-07-16
---

# 조사: render/pagination trailing(line_spacing) 모델 — 현황과 통일 가능성

- **이슈**: edwardkim/rhwp#1248
- **브랜치**: `local/task1248` (base `stream/devel`)
- **성격**: 조사·설계 전용 — 코드 0줄 수정
- **상태**: Stage 1 작성 (§1 완료, §2~§4 진행 예정)

> **현행화 메모 (2026-06-03)**: 본 문서는 #1247/#1259 반영 전 `stream/devel`
> 기준으로 작성한 조사 스냅샷이다. 현재 `devel`에는 #1247과 #1259가 이미 반영되어
> 문서의 "PR #1247 미머지" 표현, 일부 상태 표현, `cargo test --lib height_cursor`
> 테스트 개수는 현재 코드와 다를 수 있다. 다만 trailing 모델의 레이어별 역할,
> 통일 가능 영역(A)과 게이트 유지 영역(D/E)의 구분은 후속 판단 기준으로 유지한다.

> 모든 코드 인용은 `stream/devel` 기준 `file:line`. PR #1247(#1246) 미머지 상태 기준이며,
> 해당 PR이 추가하는 8번째 특례(min-gap)는 §2 에서 별도 표기한다.

---

## 0. 한 줄 요약 (잠정)

trailing line_spacing 은 **하나의 모델이 아니라, 4개 레이어가 각자 다른 가정으로 취급**한다.
typeset 이 IR seg 에 굽고(bake) → pagination 이 7개 분기로 제외/복원 → render 가 7개 특례로 재구성.
"통일" 은 이 셋이 공유하는 단일 진실(SSOT)을 만드는 일이며, 가능 여부는 §4 에서 판정한다.

---

## 1. 현황 맵 — trailing 이 다뤄지는 모든 지점

### 1.1 레이어 개요

```
[typeset.rs]  IR 생성 단계: 미주 between-notes 를 "직전 문단 마지막 seg.line_spacing" 에 굽는다(bake).
     │            + base-flow(1984HU)는 vpos 에 이미 있다고 가정, 초과분만 vpos_offset 에 가산.
     ▼
[pagination/engine.rs]  적합성 판정 단계: para_height 에서 trailing_ls 를 "조건부 제외"(7개 분기).
     │            제외 조건이 분기마다 다름(페이지 끝/표 직후/빈 문단/TAC 표 …).
     ▼
[render/height_cursor.rs::vpos_adjust]  좌표 매핑 단계: 저장 vpos→y 변환 시 trailing 을
                  "조건부 재구성"(7개 compact_endnote_* 특례). #1247 이 8번째(min-gap) 추가.
```

핵심: **같은 trailing 값이 레이어마다 "이미 포함됨 / 빼야 함 / 다시 더해야 함" 으로 다르게 해석**된다.
이 해석 불일치가 #1246 같은 gap≈0 증상의 근본 구조다.

### 1.2 typeset.rs — trailing 의 출처(bake)

| 위치 | 동작 | 의미 |
|------|------|------|
| `typeset.rs:5807` `endnote_between_notes_margin()` | FootnoteShape 에서 between-notes 마진(HU) 추출 | 원천값 |
| `typeset.rs:5819` `ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU = 1984` | base-flow 상수 | "1984HU 는 연속 vpos 흐름에 이미 있다"는 가정의 핵심 |
| `typeset.rs:5822` `endnote_between_notes_pagination_margin()` | `between_notes - 1984` (≥0) | vpos 에 **추가로** 넣을 초과분만 산출 |
| `typeset.rs:2208` `extra_gap = (between_notes - prev_spacing).max(0)` | 직전 seg 기존 spacing 대비 부족분 | min-gap(가산 아님) 의미가 여기서 발생 |
| `typeset.rs:2213` `vpos_offset += pagination_gap` | 초과분을 미주 흐름 vpos 에 가산 | pagination 예약과 정합시키는 지점 |
| **`typeset.rs:2219` `last_seg.line_spacing = between_notes`** | **직전 문단 마지막 seg 의 trailing 을 between_notes 로 덮어씀** | **trailing 을 IR 에 굽는(bake) 지점 — 핵심** |

> 결론(1.2): trailing 의 "진짜 값"은 IR seg.line_spacing 에 **덮어쓰기**로 기록된다.
> 그러나 base-flow 1984HU 는 vpos 흐름에 이미 있다고 **가정**하고 초과분만 따로 가산하는 이중 경로다.
> 이 가정이 깨지는 경우(다줄 마지막 줄 trailing 이 render 에서 누락) → gap≈0 (#1246).

### 1.3 pagination/engine.rs — trailing 의 조건부 제외(7개 분기)

| # | 위치 | 조건 | 동작 |
|---|------|------|------|
| P1 | `engine.rs:512` `trailing_tac_ls` | TAC 표 포함 문단 | 마지막 seg line_spacing 을 fit 판정에서 제외 후보로 산출 |
| P2 | `engine.rs:521` `fit_without_trail` | trailing 제거 시 들어가는가 | 페이지 끝 경계 플러시 결정 |
| P3 | `engine.rs:547` `trailing_ls` (빈 문단) | 마지막 빈 문단 + col 1 + 표 없음 | 빈 문단 trailing 제외, 페이지 흡수 판정 |
| P4 | `engine.rs:1148` `trailing_ls` (일반 fit) | 모든 일반 문단 fit 판정 | trailing 제외하고 적합성 검사 |
| P5 | `engine.rs:1155` `effective_trailing` | 하단 여유 < para_height·0.5 | **trailing 제외 비율 축소(0 으로)** — overflow 방지 |
| P6 | `engine.rs:1247` `trailing_ls` (표 직후) | 직전이 표 + 전체 배치 | overflow 임계에서 trailing 차감 |
| P7 | `engine.rs:1860` `without_trail` (TAC 표 높이) | TAC 표가 페이지 마지막 후보 | host_line_spacing 제외 |
| (참고) | `engine.rs:1884` | 다중 TAC 마지막 | 마지막 TAC line_spacing 제외 |
| (참고) | `engine.rs:2204` | TAC 표 배치 후 | trailing 복원 **안 함**(주석으로 명시) |

> 결론(1.3): pagination 은 trailing 을 "fit 판정에서 빼는 게 기본"이되, **언제 빼고 언제 안 빼는지가
> 분기마다 다르다**(페이지 끝 / 표 직후 / 빈 문단 / TAC / 하단여유 비율). 단일 규칙이 아니다.

### 1.4 render/height_cursor.rs::vpos_adjust — trailing 의 조건부 재구성(7+1 특례)

`vpos_adjust` (`height_cursor.rs:96`, **342줄**) 는 저장 vpos→y 매핑 중 trailing 을 재구성한다.
특례 상세 해부는 §2. 여기서는 trailing 관련 핵심 지점만 표기.

| 위치 | trailing 관련 동작 |
|------|---------------------|
| `height_cursor.rs:163` `trailing_ls_hu` 게이트 | vpos 연속 + 실텍스트면 trailing 0(이미 포함), 아니면 마지막 seg line_spacing bridge — **Task #1022 v2 게이트** |
| `height_cursor.rs:173` `lazy_base_corrected = prev_vpos_end - (y_delta + trailing_ls_hu)` | trailing 을 lazy_base 역산에 반영 |
| `height_cursor.rs:269` `prev_line_spacing_px` | 직전 seg trailing(px) — 여러 특례의 보정 단위 |
| `height_cursor.rs:296` `y_offset + prev_line_spacing_px` (stale_note_gap) | trailing 만큼 전진 복원 |
| (PR #1247) min-gap | gap≈0 일 때 trailing 만큼 끌어올림 — **8번째** |

> 결론(1.4): render 는 IR 에 굽힌 trailing 을, vpos 연속성·미주 흐름 위치·직전 문단 성격에 따라
> "이미 반영됨 / 다시 더해야 함" 으로 갈래 처리한다. 갈래가 7개(→#1247 로 8개)다.

### 1.5 현황 맵 종합 — 불일치의 구조

| 레이어 | trailing 의 기본 취급 | "예외" 개수 | 진실 저장 위치 |
|--------|----------------------|------------|----------------|
| typeset | IR seg 에 **굽고**, base-flow 1984HU 는 vpos 에 있다고 **가정** | 가정 1개(1984) | `seg.line_spacing` (덮어쓰기) |
| pagination | fit 판정에서 **빼는 게 기본** | 7개 분기 | (저장 안 함, 판정만) |
| render | vpos 연속이면 **포함**, 아니면 **bridge** | 7(+1)개 특례 | (재구성, 저장 안 함) |

→ **세 레이어가 trailing 에 대한 단일 진실(SSOT)을 공유하지 않는다.** typeset 의 1984HU 가정과
render 의 "vpos 연속이면 포함" 가정이 어긋나는 지점이 #1246 의 gap≈0 이다. (상세 경로 §3)

---

## 2. vpos_adjust 특례 해부 + 핀 고정 테스트

> 검증: `cargo test --lib height_cursor` = **26 passed**, `cargo test --test issue_1082_endnote_multicolumn_drift` = **4 passed** (`stream/devel`, 2026-06-03).
> 모든 특례는 공통 게이트 `suppress_large_forward_jump`(= 미주 흐름 컬럼) 아래에서만 활성.

### 2.1 공통 기반 게이트

| 게이트 | 위치 | 의미 |
|--------|------|------|
| `suppress_large_forward_jump` | 진입 전 설정 | 미주 흐름 컬럼에서만 특례 허용 (본문 무영향) |
| `compact_endnote_question_title` | `height_cursor.rs:272` | 다음 문단이 `문`으로 시작 + 직전 seg line_spacing>1000 → "새 미주 제목" |
| `trailing_ls_hu` 게이트 (#1022 v2) | `height_cursor.rs:163` | vpos 연속+실텍스트면 trailing 0, 아니면 bridge. **무조건 적용/제거 둘 다 회귀** (메모리 `tech_lazy_base_trailing_ls_gate`) |

### 2.2 특례별 표 (devel 7종 + PR #1247 1종)

| # | 특례 | 위치 | 트리거(요약) | 존재 이유 / 샘플 | 핀 고정 테스트 |
|---|------|------|-------------|-----------------|---------------|
| S0 | invalid-lazy-base capped_y | `:195` | lazy_base<0 + 새 미주 제목 + 컬럼 하단 0.85↓ | 자리차지 표 뒤 역산 무효 시 미주 겹침 방지 | `invalid_lazy_base_skips_backtrack_after_tall_object` |
| S1 | bottom_rewind | `:234` | vpos_rewind + 컬럼 하단 0.75↓ | 미주 흐름 단/쪽 재배치로 vpos 되감김 → 현재 vpos 신뢰 | `compact_endnote_bottom_rewind_uses_current_vpos`, `..._rewind_above_bottom_keeps_previous_vpos` |
| S2 | new_note_jump (+gap_cap) | `:304`(`:278`) | 새 제목 + tall seg/gap_cap + end_y 32~120px 전진 | 큰 forward 점프를 7mm note-gap 으로 캡 (3-11월 p11 문13/14) | `compact_endnote_question_title_caps_large_forward_gap`, `..._after_tall_line_uses_content_bottom_gap` |
| S3 | stale_note_gap | `:309` | 새 제목 + 컬럼 0.75↑ + end_y>+120px | stale forward(과대 전진) 시 trailing 만큼만 전진 | `compact_endnote_question_title_preserves_spacing_on_stale_forward_jump`, `..._after_empty_spacer_keeps_stored_gap_only` |
| S4 | tac_picture_gap | `:318` | TAC 그림 문단 + end_y 0~12px | 인라인 그림 미주 줄 미세 gap 보존 (#1139 문27) | (통합) `issue_1139_inline_picture_duplicate`, `issue_1082` |
| S5 | deep_backtrack | `:335` | 비-rewind + end_y < y_in-8 + 컬럼 0.90↑ + ≤80px | 컬럼 끝 직전 저장 vpos 의 backward gap 존중 | `compact_endnote_deep_backtrack_uses_vpos_near_column_bottom` 외 7건 (skips_page_path/after_tall_line/new_note_title/title_after_empty_spacer/after_note_title/crosses_previous_content + allows_safe_new_note_title) |
| S6 | title_tail_backtrack | `:346` | 제목 직후 + 현재 3줄↑ + end_y<y_in-8 + 0.90↑ | 미주 제목 다음 다줄 본문의 제한적(≤16px) 되돌림 | `compact_endnote_limited_backtrack_after_note_title_tail` |
| S7 | safe_vpos_backtrack | `:362` | 비-rewind + end_y<y_in-8 + 컬럼 0.75↓ | 미드-컬럼에서 저장 vpos backward 안전 존중 (#1209 문12) | `compact_endnote_deep_backtrack_skips_if_it_crosses_previous_content` (경계 공유), `issue_1082` |
| **S8** | **min-gap (PR #1247)** | (미머지) | 새 제목 + forward + 다줄 prev + gap∈[-0.5,4.0)px | gap≈0(다줄 마지막 줄 trailing 누락=문22) 끌어올림 (#1246) | (PR 신규) `compact_endnote_min_gap_lifts_zero_gap_question_title` 외 2 |

### 2.3 관찰

- **8종 중 6종이 `compact_endnote_question_title`(새 미주 제목) 조건에 의존** → 특례 폭증의 진원지는
  "미주 제목 앞 gap 을 얼마로 둘 것인가" 한 가지 질문이다. 답이 상황(컬럼 위치·prev 성격·vpos 방향)마다
  달라 분기가 늘었다.
- **backward 계열(S5/S6/S7)과 forward 계열(S2/S3/S8)이 대칭**: 저장 vpos 가 gap 을 과소(→끌어올림)
  또는 과대(→캡/되돌림) 인코딩하는 양방향 오차를 각각 메운다. 근본은 "저장 vpos gap ≠ 실제 목표 gap".

---

## 3. 불일치 지점 — render gap ≠ typeset/pagination 예약

### 3.1 핵심 경로 (gap≈0, #1246 문22)

```
typeset.rs:2219   prev_para.last_seg.line_spacing = between_notes(=1984+α)   ← trailing 을 IR 에 굽음
typeset.rs:2213   vpos_offset += (between_notes - 1984)                       ← 초과분만 vpos 에 가산
   └ 가정: base-flow 1984HU 는 "다줄 문단 마지막 줄 trailing" 으로 연속 vpos 에 이미 있다.

render(height_cursor):163  vpos_continuous && prev_has_text  →  trailing_ls_hu = 0  (이미 포함으로 간주)
   └ 그러나 다줄 미주 마지막 줄의 trailing 이 render 누적에서 누락되면
     실제 sequential y 에는 1984HU 가 없음 → 저장 vpos gap ≈ 0
       → S2~S7 어느 분기에도 안 걸림(전부 gap>임계 가정) → 제목이 윗줄에 붙음(문22)
         → PR #1247 이 S8(min-gap)로 "gap≈0 이면 1984 끌어올림" 추가
```

### 3.2 불일치의 본질

| 레이어 | "1984HU base-flow trailing" 가정 | 실제 |
|--------|----------------------------------|------|
| typeset | 연속 vpos 에 **이미 있다** → 초과분만 가산 | 단일줄 prev 는 맞음 |
| render | vpos 연속이면 **이미 포함** (`:163` trailing_ls_hu=0) | **다줄 prev 마지막 줄 trailing 은 누락** → gap 0 |

→ **단일줄 prev 에서는 두 가정이 일치, 다줄 prev 에서 어긋난다.** 이것이 #1246 의 정확한 불일치 지점.
S8(min-gap)이 `prev_is_multiline` 한정인 이유가 바로 이 비대칭이다.

### 3.3 pagination 측 정합

- pagination 은 trailing 을 fit 판정에서 빼되(§1.3), 미주 흐름 vpos 예약은 typeset 의 `vpos_offset`
  가산(`:2213`)으로 처리 → **pagination 자체는 IR 의 굽힌 trailing 을 신뢰**한다.
- 즉 불일치는 **typeset 가정 ↔ render 재구성** 사이이며, pagination 은 typeset 편에 정렬돼 있다.
  (PR #1247 이 별도 overflow 수정 없이 pi=475 가 해소된 이유: render 가 typeset 예약값에 맞춰졌기 때문.)

## 4. 판정 — 통일 가능 영역 / 게이트 필수 영역

### 4.1 결론 요약

> **전면 통일은 권하지 않는다. 단, "typeset base-flow 1984HU 가정 ↔ render 재구성"의
> 좁은 정규화는 가능하고 가치 있다.** trailing 의 양방향 vpos-인코딩 게이트(S2~S7)는
> 한컴 LINE_SEG 의 샘플별 시각 의도를 담은 비가역(irreducible) 영역이다.

### 4.2 영역 구분

| 영역 | 분류 | 근거 |
|------|------|------|
| **A. typeset base-flow 가정** (`typeset.rs:2213/2219` + render `:163`) | **통일 가능 (권장)** | 불일치가 "단일줄=일치 / 다줄=어긋남" 한 가지 비대칭으로 환원됨(§3.2). 규칙화 가능 |
| **B. min-gap S8** (#1247) | **A 에 흡수 가능** | A 를 정규화하면 gap≈0 자체가 안 생겨 S8 이 derive 됨(특례 1개 감소 기대) |
| **C. forward 캡 S2/S3** | **부분 정규화 여지** | A 정규화 후 "과대 전진" 케이스 일부가 줄 수 있음. 단 page-base 7mm 경로와 얽혀 검증 필요 |
| **D. backward 게이트 S5/S6/S7** | **게이트 필수 (통일 불가)** | 한컴이 LINE_SEG vpos 에 음의/소 gap 을 직접 인코딩한 케이스(#1209 문12 등). 단일 규칙으로 못 만듦 |
| **E. invalid-lazy-base S0 / bottom_rewind S1** | **게이트 필수** | 자리차지 표·단재배치 등 vpos 신뢰 불가 상황의 안전장치. 제거 불가 |

### 4.3 핵심 근거 — 왜 전면 통일은 안 되나

메모리 `tech_lazy_base_trailing_ls_gate` (Task #1022 v2):
> trailing-ls 보정은 **무조건 적용/제거 둘 다 회귀**, 조건부 게이트가 정답.

§2.3 에서 확인한 backward↔forward 대칭은 "저장 vpos gap ≠ 실제 목표 gap" 라는 **데이터(한컴 LINE_SEG)
자체의 양방향 오차**에서 온다. 이는 코드 구조 문제가 아니라 **입력 데이터의 모호성**이므로,
단일 규칙으로 통일하면 한쪽 방향이 반드시 회귀한다(D/E 영역). 핀 고정 테스트 30+ 건이 이를 증명.

### 4.4 권장 행동

1. **PR #1247(S8/min-gap) 은 그대로 머지** — 본 조사는 #1247 을 막지 않는다. S8 은 A 정규화 전까지
   유효한 실용 패치이며, 회귀 없이 #1246 을 해결(검증됨).
2. **후속 이슈(A 정규화) 1건 신설** — 아래 골격. **B(S8) 흡수가 성공 지표.**
3. **D/E 영역은 건드리지 않는다** — 게이트 유지. 향후 미주 버그도 이 영역은 "새 게이트 추가" 로 대응.

### 4.5 후속 이슈 골격 (제안)

```
제목: typeset 미주 base-flow trailing 을 IR 에 명시 표현 — render 추측 제거 (A 정규화)
범위:
  - typeset.rs:2213/2219 의 "1984HU 는 vpos 에 이미 있다" 가정을, 다줄 prev 에서도
    성립하도록 IR 에 trailing 을 명시(예: 마지막 seg trailing 을 vpos 흐름에 일관 반영).
  - render height_cursor.rs:163 trailing_ls_hu 게이트가 다줄 prev 마지막 줄 trailing 을
    단일줄과 동일하게 취급하도록 정합.
성공 지표:
  - S8(min-gap) 특례 제거 후에도 #1246(문22) + issue_1082 4샘플 + height_cursor 26단위 전부 green.
  - vpos_adjust 특례 8→7(또는 그 이하)로 감소.
비범위: D/E(backward/rewind 게이트) 변경 금지.
주의: A 만 바꾸고 S8 제거 → 전체 테스트. 한 번에 D/E 까지 손대지 말 것.
```

### 4.6 가치 판단 (정직)

- A 정규화의 **즉시 가치는 낮음**(동작 동일, 특례 1~2개 감소). **위험은 중간**(C 영역이 page-base
  7mm 경로와 얽혀 회귀 가능).
- 따라서 **긴급도 낮은 기술부채 항목**으로 등록하되, 미주 관련 새 버그가 또 S-특례를 늘리려 할 때
  "그 전에 A 부터" 의 기준점으로 활용하는 것이 현실적이다.

---

## 부록. 검증 로그

| 명령 | 결과 | 일자 |
|------|------|------|
| `cargo test --lib height_cursor` | 26 passed, 0 failed | 2026-06-03 |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 4 passed (3-09/3-11 실샘플) | 2026-06-03 |
| 기준 커밋 | `stream/devel` (PR #1247 미머지) | 2026-06-03 |

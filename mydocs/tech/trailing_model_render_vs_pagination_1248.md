# 조사: render/pagination trailing(line_spacing) 모델 — 현황과 통일 가능성

- **이슈**: edwardkim/rhwp#1248
- **브랜치**: `local/task1248` (base `stream/devel`)
- **성격**: 조사·설계 전용 — 코드 0줄 수정
- **상태**: Stage 1 작성 (§1 완료, §2~§4 진행 예정)

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

## 2. vpos_adjust 특례 8종 해부 + 핀 고정 테스트  *(Stage 2 예정)*

> 각 특례의 ① 트리거 조건 ② 도입 이유/샘플 ③ 핀 고정 회귀 테스트 매핑.

## 3. 불일치 지점 — render gap ≠ pagination 예약  *(Stage 2 예정)*

## 4. 판정 — 통일 가능 영역 / 게이트 필수 영역  *(Stage 3 예정)*

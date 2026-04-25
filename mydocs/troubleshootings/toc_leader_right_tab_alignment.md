# 목차 리더 도트 + 페이지번호 right tab 정렬

## 발견일

2026-04-25 — Task #279 (PR #282 인수 마무리)

## 증상

KTX.hwp 목차 페이지에서 다음 4가지 시각 결함이 동시에 존재했다:

1. **리더 도트 모양**: `fill_type=3` (점선) 가 사각 대시처럼 렌더 (`stroke-dasharray="1 2" stroke-width="0.5"`).
2. **소제목 페이지번호 좌측 밀림**: 들여쓰기(`margin_left>0`) 가 있는 소제목 (ps_id=111) 의 페이지번호가 장제목 (ps_id=109) 보다 약 17px 좌측에 정렬됨.
3. **리더 가 페이지번호 폭 무시**: 한 자리/두 자리 페이지번호 무관 leader 끝이 동일 x 로 끊어 leader 와 페이지번호가 겹침.
4. **셀 padding_right 무시**: 페이지번호가 셀 우측 padding 영역까지 침범.
5. **장제목 ↔ 소제목 정렬 불일치**: 장제목은 +10px 우측, 소제목은 정렬됨.
6. **두 자리 페이지번호 (장제목 측) 정렬 어긋남**: 선행 공백 (`" 16"`) 처리로 +9px 우측에 그려짐.

## 근본 원인

### 0. HWP 스펙은 데이터 포맷 스펙, 조판 알고리즘 스펙이 아니다

HWP 스펙은 TabDef 의 `position`/`tab_type`/`fill_type` 의 **바이너리 위치만 정의**한다. **이 값으로 어떻게 그릴 것인지** 는 한컴 워드프로세서의 비공개 조판 엔진이 결정한다. **스펙 그대로 처리하면 한컴과 같은 결과가 나오지 않는다**.

→ rhwp 는 "스펙 충실 구현체" 가 아니라 **"한컴 조판 결과를 재현하는 엔진"** 이어야 한다. 리더 도트의 시멘틱 (`fill_type ≠ 0` = "이 줄 우측 끝까지 채움") 같은 한컴 의도를 자체 방어 로직으로 재해석해야 한다.

### 1. 리더 도트 모양 (svg.rs / web_canvas.rs)

`fill_type=3` (점선) 을 `dasharray=[1, 2] width=0.5` 로 표현해 사각 끝 짧은 대시처럼 그렸음.

### 2. `find_next_tab_stop` 의 일률적 클램핑

```rust
// 수정 전
let pos = if ts.position > available_width && available_width > 0.0 {
    available_width
} else { ts.position };
```

들여쓰기 문단의 `available_width` 는 작아지는데 RIGHT 탭 (`tab_type=1`) 도 그 값으로 클램핑하면 페이지번호 right edge 가 좌측으로 이동.

### 3. cross-run RIGHT tab pending 가드 누락

```rust
// 수정 전
if run.text.ends_with('\t') { /* set pending */ }
```

소제목의 첫 run 이 `"1. 추진배경 및 목적\t "` 즉 **`\t` 다음 trailing 공백** 으로 끝나는 케이스를 놓쳐 cross-run RIGHT 정렬이 발동하지 않았다.

### 4. 리더 시멘틱 누락

HWP 가 저장한 `tab.position` 그대로를 사용하면 셀 padding_right 영역까지 침범. 한컴은 **리더 있는 (`fill_type ≠ 0`) RIGHT 탭** 의 의미를 "**inner content 우측 끝까지 채움**" 으로 재해석한다.

### 5. 리더 길이가 페이지번호 폭을 무시

`extract_tab_leaders` 가 `\t` 다음 문자 위치 기반으로 leader.end_x 를 계산하지만, cross-run 정렬 후 페이지번호 run 이 좌측으로 이동하면 leader 가 페이지번호와 겹친다. **cross-run 정렬 시점에 직전 leader-bearing TextRun 의 leader.end_x 를 페이지번호 시작 x 직전까지 단축** 해야 한다.

### 6. 공백 only run carry-over 부재

장제목 (`"Ⅰ. 사업 개요\t" + " " + "3"`) 의 ` ` 단독 run 에 RIGHT 정렬을 적용하면 페이지번호 "3" 이 공백 폭만큼 우측으로 밀린다. 공백 only run 은 정렬 단위가 아니므로 pending 을 다음 의미있는 run 으로 carry-over 해야 한다.

### 7. trim_start 로 정렬 시 선행 공백 보정 부재

장제목 두 자리 케이스 (`" 16"` 한 run) 는 `trim_start` 후 폭으로 정렬하면 run 시작 x 가 적정 위치이지만 `draw_text` 가 공백 포함 텍스트를 그려 페이지번호 첫 글자가 +공백폭 우측에 출력. **trim 하지 않은 전체 run 폭으로 정렬**해야 시각 right edge 가 effective_pos 와 일치.

## 해결

`src/renderer/{svg.rs, web_canvas.rs, layout/text_measurement.rs, layout/paragraph_layout.rs}` 에 다음 7가지 수정 적용:

| # | 위치 | 변경 |
|---|------|------|
| 1 | `svg.rs::draw_text fill_type=3` | `dasharray="0.1 3" stroke-linecap="round" width="1.0"` 로 원형 점 표현 |
| 2 | `web_canvas.rs::draw_leader fill_type=3` | `set_line_cap("round") + dash=[0.1, 3.0] + width=1.0 + restore butt` |
| 3 | `text_measurement.rs::find_next_tab_stop` | `tab_type != 1` 가드 추가 — RIGHT 탭은 클램핑 제외 |
| 4 | `paragraph_layout.rs` (est/render) | `trim_end_matches(' ').ends_with('\t')` — trailing 공백 + \t 케이스 |
| 5 | `paragraph_layout.rs::resolve_last_tab_pending` | 시그니처 확장: `(f64, u8)` → `(f64, u8, u8)` (fill_type 추가) |
| 6 | `paragraph_layout.rs` (cross-run take) | leader 있는 RIGHT 탭은 `effective_pos = effective_margin_left + available_width` (셀 inner 우측 끝) |
| 7 | `paragraph_layout.rs` (cross-run take) | 공백 only run 은 carry-over (정렬 단위 아님). leader-bearing TextRun 검색으로 leader.end_x 단축 |
| 8 | `paragraph_layout.rs` (cross-run take) | `next_w` 를 `trim_start` 가 아닌 **전체 run 폭** 으로 — 선행 공백 시각 보정 |

## 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 992 passed |
| `cargo test --test svg_snapshot` | ✅ 6 passed (issue_267_ktx_toc_page UPDATE_GOLDEN, issue_147_aift_page3 UPDATE_GOLDEN) |
| `cargo test --test issue_301` | ✅ z-table 가드 |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean |
| 7 핵심 샘플 페이지 수 회귀 | ✅ 모두 무변화 |
| KTX 목차 시각 검증 (작업지시자) | ✅ 한컴과 동등 |

### KTX 목차 좌표 변화 (Before → After)

| 항목 | Devel | After |
|------|-------|-------|
| 장제목 페이지번호 "3" x | 709.76 | **690.76** |
| 소제목 페이지번호 "4" x | 689.88 | 690.76 |
| 소제목 두 자리 "14" x | 681.43 | 680.76 |
| 장제목 두 자리 "16" 첫글자 x | 690.09 (+9 어긋남) | **681.09** ✅ |

모든 페이지번호 right edge 가 동일 위치 (≈ 700.0) 에 정렬.

## 교훈

### 1. HWP 스펙 충실 ≠ 한컴 호환

HWP 스펙은 데이터 포맷 정의일 뿐이고, **한컴 조판 알고리즘은 비공개**다. 스펙대로 처리해도 한컴과 다르게 보이면 **한컴 결과를 정답으로 삼아 의도를 역공학** 해야 한다. `position`/`fill_type` 같은 값들은 한컴이 **시멘틱적으로 재해석** 하므로 rhwp 도 동일 방어 로직이 필요하다.

### 2. run 분할 패턴은 CharShape 변화에 따라 달라진다

같은 paragraph 라도 **CharShape (폰트/크기/색) 변화 위치마다 run 이 잘림**. 장제목 (Ⅰ 폰트 ≠ 본문 폰트) 은 6 runs, 소제목 (한 폰트) 은 3 runs. **cross-run 처리 로직은 실제 run 분할 패턴을 가정에 두면 안 된다** — 공백 only run, trailing 공백 \t run, leader-bearing run 등 다양한 패턴 모두 일관 처리해야 한다.

### 3. 정렬은 "시각 right edge" 기준

right tab 정렬은 단순히 `tab_pos` 를 적용하는 것이 아니라 **시각 출력 결과의 right edge 가 의도 위치에 오도록** 해야 한다:
- run 시작 x 와 draw_text 의 첫 글자 출력 x 가 다를 수 있음 (선행 공백)
- 페이지번호 폭 (한 자리 vs 두 자리) 에 따라 leader 끝도 달라져야 함
- 셀 padding_right 영역 침범 여부 검사

이 세 가지를 모두 만족하려면 **여러 개의 미세 가드** 가 필요하다 (find_next_tab_stop 클램핑 제외 + trailing 공백 가드 + leader 시멘틱 + carry-over + leader-bearing TextRun 검색 + trim 제거). 각 가드는 **한 가지 시각 결함만 해결** 하므로 단계별 시각 검증 (작업지시자가 매 단계 확인) 이 필수.

### 4. 셀 안 paragraph 의 col_area 는 이미 padding 적용됨

`col_area = inner_area = cell - (pad_left, pad_right)` 라 `effective_margin_left + available_width` 가 inner 우측 끝. 단 우측 끝 = 셀 padding_right 영역 침범 가드의 의미가 셀 안에서는 자동 정합.

### 5. UPDATE_GOLDEN 결정 기준

svg_snapshot 골든 영향이 발생하면:
- 영향 페이지가 **본 task 의 의도된 변경 영역** (KTX 목차) 이면 UPDATE_GOLDEN
- **무관한 영역에 영향** (예: aift 표 안 leader) 도 발견 시 leader 표현 통일 변경의 자연 결과면 UPDATE_GOLDEN
- **회귀 (의도하지 않은 좌표 변화)** 면 stop + 원인 분석

## 관련 자료

- 이슈: [#279](https://github.com/edwardkim/rhwp/issues/279)
- PR: [#282](https://github.com/edwardkim/rhwp/pull/282) (외부 기여자 [@seanshin](https://github.com/seanshin) → 메인테이너 인수)
- 골든: `tests/golden_svg/issue-267/ktx-toc-page.svg`, `tests/golden_svg/issue-147/aift-page3.svg`
- 관련 트러블슈팅:
  - `mydocs/troubleshootings/hwpx_lineseg_reflow_trap.md` — HWP 바이너리는 한컴 계산값 신뢰
  - `mydocs/troubleshootings/line_spacing_lineseg_sync.md` — 한컴은 데이터 + 자체 알고리즘 모두 사용

## 외부 기여 인정

본 task 의 핵심 진단 (리더 도트 dasharray + right tab 클램핑 제외) 은 [@seanshin](https://github.com/seanshin) (Shin hyoun mouk) 의 분석. 메인테이너 인수 후 추가 6가지 가드 (#3~#8) 보강.

---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 트러블슈팅: 탭+TAC 수식 겹침 문제 (#142, #159)

작성일: 2026-04-16

## 증상

exam_math.hwp 2페이지 6번 문항 선택지에서 ③ 원문자 위에 수식 `0`이 겹쳐 렌더링됨.

```
① -5    ② -√5    ③0    ④ √5    ⑤ 5
                   ↑ 겹침
```

## 조사 과정

### 1차 시도: 수식 TAC 너비 문제로 추정 (실패)

- `composer.rs`에서 수식 TAC 너비를 레이아웃 엔진 산출값으로 대체 → 효과 없음
- 원인: 탭이 수식 TAC 너비에 무관하게 고정 탭 스톱으로 점프하므로, TAC 너비가 원문자 간격에 영향 없음

### 2차 시도: 원문자 전각 폭 문제로 추정 (부분 효과)

- `is_fullwidth_symbol()`에 원문자(U+2460~) 범위 추가 → 간격 약간 변화하지만 겹침 미해결
- 원인: 내장 폰트 메트릭에 원문자가 없어 폴백은 적용되지만, 근본 원인이 아님

### 3차 시도: 탭 `/2.0` 문제로 추정 (분석)

- `style_resolver.rs:618`에서 탭 position을 `/2.0`으로 나누는 코드 발견
- 제거하면 간격 과다, 유지하면 불일치
- 한컴 격자 비교로 **`/2.0`이 올바른 변환**임을 확인 (HWP 탭 position이 실제 좌표의 2배로 저장)

### 4차 시도: est_x와 x의 불일치 추적 (핵심 접근)

- 정렬 계산(`est_x`)과 렌더링(`x`)의 초기값/누적값을 비교
- SVG 좌표 역산으로 `compute_char_positions`와 `estimate_text_width`의 결과가 다른 것을 발견

### 5차: 근본 원인 발견 ✅

디버그 출력 추가로 확인:
```
[RENDER_TAC] seg="\t③ " est_w=45.0 cp_last=61.7 inline_tabs=4
```

**`estimate_text_width`(45.0)와 `compute_char_positions`(61.7)의 탭 계산이 16.7px 차이!**

## 근본 원인

### `inline_tabs` 처리 누락

- `paragraph_layout.rs:1105`에서 렌더링 스타일에 `inline_tabs = tab_extended` 설정
- `compute_char_positions`는 `inline_tabs`가 있으면 **인라인 탭 경로** 사용
- `estimate_text_width`는 `inline_tabs`를 **무시**하고 **custom tabs 경로** 사용
- 두 경로의 탭 점프 결과가 다르므로, ③ 렌더링 위치와 수식 x 좌표가 불일치

### 영향

```
estimate_text_width: 탭 점프 = tab_stops 기반 → seg_w = 45.0px
compute_char_positions: 탭 점프 = inline_tabs 기반 → cp_last = 61.7px

수식 x = bbox.x + seg_w → ③ 글리프 안쪽에 배치 (겹침)
③ 렌더링 = bbox.x + char_positions[③] → 탭 점프 후 올바른 위치
```

## 수정

`text_measurement.rs`의 `EmbeddedTextMeasurer::estimate_text_width`에 `inline_tabs` 분기 추가:

```rust
if tab_char_idx < style.inline_tabs.len() {
    let ext = &style.inline_tabs[tab_char_idx];
    let tab_width_px = ext[0] as f64 * 96.0 / 7200.0;
    // compute_char_positions와 동일한 로직
    ...
}
```

정렬 계산(est_x)에서도 `ts.inline_tabs = composed.tab_extended.clone()` 추가.

## 검증

- `cargo test` 788개 전체 통과
- ③과 수식 `0`의 겹침 해소
- ③→④ 간격: 13.8mm → 18.3mm (한컴 17mm 대비 +1.3mm, 기준 ≤ 2mm PASS)

## 교훈

1. **같은 데이터를 다른 경로로 계산하는 코드는 반드시 동기화** — `compute_char_positions`와 `estimate_text_width`가 다른 탭 처리 로직을 사용하면 레이아웃이 어긋남
2. **디버그 출력으로 두 함수의 결과를 동시에 비교**하는 것이 가장 효과적인 진단 방법
3. **격자 오버레이 기능(#145)**이 한컴 비교에 큰 도움

---

## 후속 사건: #290 cross-run 탭 감지 (2026-04-24)

### 증상

동일 파일(`samples/exam_math.hwp`) 페이지 7 의 18번 "수열" 문항 첫 줄이 좌측 단의 **우측 끝 근처로 밀려 렌더**됨. PDF 는 `18. 수열 {a_n}이 모든 자연수 n에 대하여` 인데 SVG 는 `18.` ····· `수열 ...`.

### 근본 원인

`paragraph_layout.rs:1213-1226` (render 측) + `:854-868` (est 측) 의 **cross-run 우측/가운데 탭 감지 블록**:

```rust
if has_tabs && run.text.ends_with('\t') {
    if let Some(last_tab_pos) = run.text.rfind('\t') {
        // ...
        let (tp, tt, _) = find_next_tab_stop(                  // ← inline_tabs 무시
            abs_before, &tab_stops, tw, auto_tab_right, available_width,
        );
        if tt == 1 || tt == 2 {
            pending_right_tab_render = Some((tp, tt));         // ← 잘못된 설정
        }
    }
}
```

마지막 `\t` 의 종류를 판정할 때 `find_next_tab_stop` 만 사용 — 본문 `tab_extended` (inline_tabs) 는 참조하지 않음. 본 case:

1. Run `"18.\t\t\t"` 의 3 개 `\t` 는 inline 경로에서 LEFT 로 x=38.24 까지 진행 (정상)
2. cross-run 감지는 TabDef 만 봄 → `abs_before=37.19` > 모든 stops → `auto_tab_right` 폴스루 → **type=1 (RIGHT)** 오판
3. `pending_right_tab_render = Some((420.11, 1))` 설정
4. 다음 run `"수열 이 모든 자연수 "` 가 `col_area.x + 420.11 - next_w(201)` = 290.91 px 로 **우측 끝 역산 배치**

### ext[2] 포맷 실증

임시 트레이스로 확정:
- `ext[2]` 는 high/low byte 합성 값
- **high byte = 탭 종류 enum+1** (1=LEFT, 2=RIGHT, 3=CENTER, 4=DECIMAL)
- low byte = fill_type (TabDef.fill 과 일치)

현재 `text_measurement.rs:217, 320` 는 `ext[2]` 전체 u16 을 종류로 해석 → 실제 HWP 값 (최소 256) 과 매칭되지 않아 **inline RIGHT/CENTER 경로 영원히 도달 불가** (별도 후속 이슈 후보).

### 수정

`paragraph_layout.rs` 에 `resolve_last_tab_pending` 헬퍼 신규 추가:
- inline_tabs 가 마지막 `\t` 를 커버하면 `ext[2] >> 8` 고바이트로 탭 종류 판정
- LEFT (0/1) → `None` (pending 설정 안 함) — **본 수정의 핵심**
- RIGHT/CENTER (2/3) → TabDef `find_next_tab_stop` 기반 위치로 폴스루 (기존 동작 유지)
- 미지 (4=DECIMAL 등) → 보수적 `None`

est 측 + render 측 양쪽 모두 `inline_tab_cursor_*: usize` 변수 도입하여 run 을 거치며 증가 (`run.text.chars().filter(|c| *c == '\t').count()`).

### 검증

- 단위 테스트 5 건 (`task290_*`) + 통합 테스트 1 건 (`tab_cross_run::task290_exam_math_p7_item18_not_right_aligned`) 모두 pass
- 184 페이지 회귀 (exam_math 20 + biz_plan 6 + exam_eng 11 + exam_kor 25 + hwp-3.0-HWPML 122): **1 페이지만 변경** (exam_math p.7, 의도된 item 18 수정)
- RIGHT inline tab 케이스 (hwp-3.0-HWPML 저작권\t1) 회귀 없음

### 교훈 (#142 교훈의 확장)

> "**같은 데이터를 다른 경로로 계산하는 코드는 반드시 동기화**" 라는 #142 교훈이 **run 내부 탭 처리 ↔ cross-run 탭 감지** 의 불일치로 재연됨. 런 내부는 inline_tabs 를 봤지만 cross-run 감지는 TabDef 만 봄. 같은 파일에서 서로 다른 함수가 서로 다른 데이터 소스를 참조하는 패턴이 재발할 여지가 있으면 **헬퍼로 중앙화** 하는 것이 안전 (본 수정에서 `resolve_last_tab_pending` 으로 통합).

### 관련
- 이슈 [#290](https://github.com/edwardkim/rhwp/issues/290)
- 리포트: `mydocs/report/task_m100_290_report.md`
- 계획서: `mydocs/plans/task_m100_290{,_impl}.md`

## 후속 사건: #296 WASM Canvas 경로 inline_tabs 무시 (2026-04-24)

### 증상

PR #292 (#290) 머지 후 브라우저 검증에서 `exam_math.hwp` p.7 #18 "수열" 문항이 여전히 우측으로 밀림:
- SVG (CLI): ✅ 정상 (`translate(109.80, ...)`)
- Canvas (브라우저): ❌ x≈290.91

### 근본 원인

`src/renderer/layout/text_measurement.rs` 두 측정기 비대칭:

- `EmbeddedTextMeasurer` (네이티브): inline_tabs 분기 존재 (단, `tab_type = ext[2]` 전체 u16 해석 버그 보유)
- `WasmTextMeasurer` (WASM): **inline_tabs 분기 자체가 부재** → `find_next_tab_stop` (TabDef) 만 사용 → `auto_tab_right` 폴스루 → 우측 밀림

### 수정 (범위 축소판)

`src/renderer/layout/text_measurement.rs`:
- 헬퍼 `inline_tab_type(ext) -> u8` 신규 추가 = `(ext[2] >> 8) & 0xFF`
- `WasmTextMeasurer::estimate_text_width` / `compute_char_positions` 에 inline_tabs 분기 신규 추가
  - match arm: `2 => RIGHT`, `3 => CENTER`, `_ => LEFT/DECIMAL` (PR #292 실증 포맷)
- `EmbeddedTextMeasurer` 는 **건드리지 않음** — 기존 golden SVG (issue-147, issue-267) 가 우연한 LEFT 폴백에 의존하고 있어 네이티브 측 수정은 한컴 PDF 대조로 올바른 동작 확정 후 별도 이슈 처리

### 검증

- `cargo test --lib`: 992 passed (988 → 992, 단위 테스트 4건 추가)
- `cargo test --test svg_snapshot`: 6 passed (기존 golden 유지)
- `cargo test --test tab_cross_run`: 1 passed (#290 회귀 없음)
- WASM Docker 빌드 → rhwp-studio 브라우저 시각 검증 성공 (작업지시자 판정)

### 교훈 (#290 교훈의 확장)

> #142 → #290 → #296 누적: "같은 데이터를 다른 경로로 계산하는 코드는 헬퍼로 중앙화." 이번 #296 에서는 범위 축소 판단으로 WASM 만 수정 — **`inline_tab_type` 을 `pub(super)` 로 공개**해두어 네이티브 측정기가 후속 이슈에서 재사용 가능한 상태로 만듦.
>
> 또한 **범위 축소 결정의 가치**: Stage 2 중간에 네이티브 수정이 기존 golden 2건을 깨트리는 것을 확인한 시점에 방향을 전환. 무리하게 밀어붙여 golden 을 갱신했다면 한컴 PDF 대조 없이 잘못된 방향으로 갈 위험.

### 관련
- 이슈 [#296](https://github.com/edwardkim/rhwp/issues/296)
- 리포트: `mydocs/report/task_m100_296_report.md`
- 계획서: `mydocs/plans/task_m100_296{,_impl}.md`
- 후속 이슈 후보: 네이티브 `EmbeddedTextMeasurer` 의 `tab_type = ext[2]` 버그 (한컴 PDF 대조 후 수정)

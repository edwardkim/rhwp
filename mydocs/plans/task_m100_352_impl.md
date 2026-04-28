# Task #352 구현계획서: dash 시퀀스 Justify 폭 부풀림 — Option C 실행 절차

> 구현계획서 | 2026-04-28
> Issue: [#352](https://github.com/edwardkim/rhwp/issues/352)
> Branch: `local/task352`
> 수행계획서: [`task_m100_352.md`](task_m100_352.md)

---

## 단계 개요

| Stage | 산출 커밋 | 핵심 변경 | 검증 |
|-------|----------|----------|------|
| 1 | `Task #352 Stage 1: 원인 확정` | (코드 변경 없음) 임시 로그 → revert | dump 출력 + 분석 보고서 |
| 2 | `Task #352 Stage 2: dash advance 자연 폭 보정` | `paragraph_layout.rs` Justify 분배 | `cargo test` + Q32 dash advance ≈ 자연 폭 |
| 3 | `Task #352 Stage 3: dash run 시각 라인 통합` | `svg.rs`, `web_canvas.rs` | PDF 비교 + 골든 회귀 |
| 4 | `Task #352 Stage 4: 최종 결과 보고서` | `report/`, `orders/` | 전 검증 통과 |

---

## Stage 1 — 원인 확정

### 1-1. 임시 디버그 로그 삽입 위치

**`src/renderer/layout/paragraph_layout.rs` (현 line 996 직전)**:
```rust
if std::env::var("RHWP_DEBUG_352").is_ok() && needs_justify {
    let head: String = comp_line.runs.iter().flat_map(|r| r.text.chars()).take(20).collect();
    eprintln!(
        "[#352] needs_justify pi={} interior_spaces=? total_char={} text_w={:.2} avail={:.2} head={:?}",
        paragraph_index, total_char_count, total_text_width, available_width, head,
    );
}
```

분기 A/B 진입 직후 각각 한 줄 더 출력 (`branch=A ews=...` / `branch=B ecs=...`).

**`src/renderer/layout/text_measurement.rs` (현 line 209 직전)**:
```rust
#[cfg(debug_assertions)]
if std::env::var("RHWP_DEBUG_352").is_ok() && (c == '-' || c == '\u{2013}' || c == '\u{2014}') {
    eprintln!(
        "[#352] dash base_w={:.3} ratio={:.3} ls={:.3} ecs={:.3} font_size={:.3} font={:?}",
        base_w, ratio, style.letter_spacing, style.extra_char_spacing, font_size,
        style.font_family.split(',').next().unwrap_or(""),
    );
}
```

`measure_char_width_embedded` 반환값(또는 None) 도 한 줄 추가 출력.

### 1-2. 실행

```bash
RHWP_DEBUG_352=1 ./target/release/rhwp export-svg samples/exam_eng.hwp -p 4 \
    -o /tmp/p5_352/ 2> /tmp/issue_352_log.txt
grep -E "#352" /tmp/issue_352_log.txt | head -100
```

### 1-3. 판정 매트릭스

| 로그 패턴 | 결론 | Stage 2 설계 |
|----------|------|-------------|
| `branch=B` + `extra_char_sp ≈ 8.6` | 가설 1 확정 — Branch B 가 dash 라인에서 발동 | 4-A(a) 또는 4-A(c): leader-like 시퀀스를 분기 결정에서 단어처럼 취급 |
| `branch=A` + dash `base_w` ≈ em | 가설 3 — 메트릭 자체가 큼 | font_metrics_data 의 HY신명조 dash glyph_w 보정 |
| `dash base_w` 미출력 (None) | 가설 2 — fallback `font_size * 0.5` | `is_narrow_punctuation` 에 dash 추가 |
| 위 외 | 단계 1 분석 재진행 | 보고서에 명시 |

### 1-4. revert + 보고

- `git diff` 확인 → eprintln 모두 제거 후 `cargo build --release` 통과
- `mydocs/troubleshootings/issue_352_root_cause.md` 작성 (로그 발췌, 판정 결과, Stage 2 설계 결론)
- `mydocs/working/task_m100_352_stage1.md` 작성 (절차, 산출, 검증)
- 커밋: `Task #352 Stage 1: 원인 확정 (Branch B 발동 / dash advance 부풀림 경로)`

---

## Stage 2 — advance 자연 폭 보정 (4-A)

### 2-1. 설계 (Stage 1 결과별 분기)

**Case 가설 1 확정 시 (가장 유력)**:

`paragraph_layout.rs:996` 의 `interior_spaces` 산정 직후, **leader-like 반복 시퀀스를 단어 경계로 간주**하여 가상 공백 카운트를 추가한다.

```rust
// Task #352: 반복 dash/underscore 시퀀스를 단어 경계로 취급해 Branch A 강제
let leader_groups = count_leader_groups(&all_chars[..visible_count]);
let virtual_spaces = interior_spaces + leader_groups;
```

`count_leader_groups`: 동일 leader 글자(`-`, `_`) 가 3 회 이상 연속하는 그룹 수를 카운트. 분배 대상 공백 카운트에 합산하여 분기 A 가 발동되도록 한다. 단, 실제 spread 적용 단계에서는 dash 자체에 spread 가 가해지지 않도록 `extra_word_sp` 만 사용 (분기 A 의 기존 동작).

**Case 가설 3 시**: `font_metrics_data::find_metric` 의 HY신명조 dash glyph 폭 검증. 로컬 패치 또는 `is_halfwidth_punct` 목록에 `-` 추가.

**Case 가설 2 시**: `is_narrow_punctuation` 에 `-` 추가 (단, `--` 같은 텍스트 내 일반 dash 도 영향받음 → 회귀 점검 필수).

### 2-2. 검증

- `dump-pages -p 4` 에서 ls[10] 라인의 dash advance 가 자연 폭 수준
- `samples/exam_eng.hwp` SVG 에서 "of being" x 좌표가 PDF 와 근접 (~컬럼 중앙)
- `cargo test` 전수 통과
- `samples/` 의 다른 hwp 5개(KTX, 견적서, 협조전, 일반 보고서 등) 골든 SVG 회귀 점검

### 2-3. 보고

`mydocs/working/task_m100_352_stage2.md` — 변경 코드, before/after 수치, 회귀 결과.

커밋: `Task #352 Stage 2: dash advance 자연 폭 보정 (Justify 분배 leader 인식)`

---

## Stage 3 — dash run 시각 라인 통합 (4-B)

### 3-1. 설계

`src/renderer/svg.rs:1882-1947` (클러스터 렌더 루프) 에서:

1. `is_dash` 헬퍼: `c == '-' || c == '\u{2013}' || c == '\u{2014}'`
2. 클러스터 순회 시 연속 dash 시작 인덱스 ↔ 끝 인덱스 추적 (≥3 개일 때 라인화)
3. 라인 시작 x = `char_positions[start_idx]`, 끝 x = `char_positions[end_idx + 1]`
4. baseline 보정 y = `y - font_size * 0.32` (실제 dash 위치는 polyfill: 폰트 메트릭의 `dash_y` 또는 휴리스틱)
5. `<line x1=... x2=... y1=... y2=... stroke=color stroke-width=...>` 출력 + 해당 글리프 `<text>` 출력 스킵
6. 미만(≤2) 은 기존 글리프 출력 유지

### 3-2. web_canvas.rs 동일 패턴

`src/renderer/web_canvas.rs:1318` 인근에 같은 분기 추가. WASM 빌드 회귀 확인은 Stage 4 에서 옵션.

### 3-3. 검증

- exam_eng.hwp p5 SVG → PDF 와 시각 비교 (rsvg-convert + pdftoppm + 차이 차이 다이어그램 수동)
- 일반 dash 사용처 회귀: "stimulus-driven", "* taint--altruistic" 등 ≤2 dash 는 변형되지 않아야 함
- Stage 2 의 advance 보정과 결합되어 라인 길이가 PDF 와 근접

### 3-4. 보고

`mydocs/working/task_m100_352_stage3.md` — before/after 스크린샷, 임계값(≥3 dash) 결정 근거.

커밋: `Task #352 Stage 3: 연속 dash 시퀀스 SVG 라인 렌더`

---

## Stage 4 — 최종 검증 + 보고서

### 4-1. 검증 절차

```bash
cargo build --release
cargo test --release
# 주요 샘플 회귀
for f in samples/{exam_eng,KTX,견적서,협조전,일반_보고서}.hwp; do
    ./target/release/rhwp export-svg "$f" -o /tmp/regress_352/$(basename "$f" .hwp)/
done
git diff --stat output/svg/  # 골든 변동 확인 (있으면 기대 변화인지 수동 검토)
```

### 4-2. 산출물

- `mydocs/report/task_m100_352_report.md` (수행계획서 §8 검증 기준 1~5 모두 충족 명시)
- `mydocs/orders/{날짜}.md` 의 #352 항목 ✅ 상태로 갱신
- 최종 PR-ready 상태 확인 (작업지시자 review 후 local/devel merge)

### 4-3. 커밋

`Task #352 Stage 4: 최종 결과 보고서 + orders 갱신`

이후 closes #352 와 함께 local/devel merge.

---

## 롤백 시나리오

각 Stage 별 커밋 단위로 분리되어 있어 `git revert <commit>` 로 단계별 되돌리기 가능. Stage 2 (advance 보정) 가 회귀를 유발하면 Stage 3 시각 통합만 유지하는 단독 옵션도 가능 (시각만 개선 — Option A 등가).

## 비포함 / 백로그

- HWP `cs` 필드(line_seg.cs) 활용 — 별 이슈
- `_____` (underscore) 같은 다른 leader 글자 일반화 — Stage 4 회고 후 별 이슈
- font_metrics_data 의 HY신명조 dash 메트릭 보강 — Stage 1 결과 따라 별 이슈

# 구현계획서: narrow glyph(콤마·중점 등) 뒤 문자 advance 과다 보정

- **타스크**: [#257](https://github.com/edwardkim/rhwp/issues/257)
- **마일스톤**: M100
- **브랜치**: `local/task257`
- **작성일**: 2026-04-23
- **상위 문서**: `mydocs/plans/task_m100_257.md`

## 0. 진입점 요약 (사전 조사 결과)

| 경로 | 파일:라인 | 현재 동작 |
|------|----------|----------|
| `compute_char_positions` (Embedded, SVG export 경로) | `src/renderer/layout/text_measurement.rs:260-290` | `measure_char_width_embedded` 미등록 폰트·미등록 글자에서 `cluster_len > 1 \|\| is_cjk \|\| is_fullwidth_symbol` 이면 `font_size`, 아니면 `font_size * 0.5` 로 폴백 |
| `compute_char_positions` (Native, Canvas/WASM 경로) | `text_measurement.rs:584-622` | 브라우저 canvas `measure_char_width_hwp` 로 실제 메트릭 확보 — narrow glyph 자동 정확 |
| `is_fullwidth_symbol` | `text_measurement.rs:845-859` | U+00B7(중점)·U+002C(콤마)·U+002E(마침표) 등 narrow glyph **미포함** |
| `measure_char_width_embedded` | `text_measurement.rs:726` | Malgun Gothic 등 일부 폰트에만 메트릭 있음. HY헤드라인M·휴먼명조는 미등록 → 폴백 진입 |

### 기존 안전장치 (Task #229 유산)

`compute_char_positions` 는 음수 자간에서 `min_w = base_w * ratio * 0.5` 클램프로 **포지션 단조성**(비감소) 을 보장한다. 이미 다음 회귀 테스트가 있음 (`text_measurement.rs:1150-1228`):

- `test_overflow_compression_positions_monotonic_comma` — "65,063,026,600" + `extra_char_spacing=-2.88`
- `test_charshape_negative_letter_spacing_no_reverse` — 같은 시나리오 `letter_spacing=-2.88`
- `test_overflow_compression_positions_monotonic_period` — "526.278"
- `test_non_compression_width_unchanged_by_fix` — 비압축 경로 폭 50~200 범위

**본 타스크는 이 4개 테스트를 모두 깨지 않아야 한다.**

### 증상과 진짜 원인 재정리

- `1,000항목` 의 `,` 는 Malgun Gothic 메트릭 DB 에 등록이 있을 수도 있으나, `text-align-2.hwp` 에서 실제 쓰이는 폰트(HY헤드라인M·휴먼명조)는 미등록 → 폴백 진입
- 폴백에서 `base_w = font_size * 0.5` → 콤마 폭이 PDF 실제(`~0.15~0.20 * font_size`) 대비 과다
- **음수 자간이 얕거나 없는 경우, min_w 클램프는 발동조차 안 함** → 클램프가 아니라 **`base_w` 자체가 원인**
- min_w 클램프는 자간이 강하게 음수인 경우(Task #229 시나리오)에만 발동하는 별개 방어막

→ 수정의 본체는 **단계 2(`base_w` 분기)** 이며, **단계 3(클램프 우회)** 는 기존 단조성 테스트를 깨지 않는 선에서 최소 손댐. 다만 `base_w` 가 작아지면 `min_w = base_w * 0.5` 도 자동 작아지므로 추가 수정이 불필요할 가능성이 높다 — 단계 3 에서 수치 확인 후 판단.

## 1. 구현 단계 (4단계)

### 단계 1 — 현상 재현·베이스라인 고정

**목표**: 수정 전 수치를 테스트로 고정하고, 수정 후 개선 여부를 정량 판정할 기준선 확보.

**작업**

- `samples/text-align-2.hwp` 편입 확인 (`git status` 로 untracked 확인, `.gitignore` 미저촉 확인)
- `cargo run --bin rhwp -- export-svg samples/text-align-2.hwp -o output/svg/text-align-2/` 현 상태 baseline 기록 (`output/re/` 참조용 복사)
- 단위 테스트 파일에 **failing 예정 테스트** 2건 추가 (이름만 기록, 단계 2 에서 pass):
  - `test_narrow_glyph_comma_base_width` — Malgun Gothic 미등록 상황 에뮬레이션(폰트명 `HY헤드라인M` 지정, embedded 메트릭 없음 → 폴백 경로) 에서 `,` advance 가 `font_size * 0.35` 이하
  - `test_narrow_glyph_middle_dot_base_width` — `·` (U+00B7) 동일
- 기존 Task #229 테스트 4건이 **현 baseline 에서 모두 pass** 임을 확인 (변경 없음, 기록)
- 수치 측정: `compute_char_positions` 직접 호출로 font_size=13.33 (10pt), font=`HY헤드라인M`, letter_spacing=0 조건에서 `,` · `·` · `.` advance 현재값 기록 → 단계별 보고서에 첨부
- **소스 수정 없음.** 테스트는 `#[ignore]` 로 일단 marker 만 (단계 2 에서 활성화)

**커밋**
1. `Task #257: 샘플 samples/text-align-2.hwp · samples/text-align-2.pdf 편입`
2. `Task #257 단계1: narrow glyph baseline 측정 + 단계별 보고서`
   - `mydocs/working/task_m100_257_stage1.md`
   - 단위 테스트 #[ignore] 2건 추가 (text_measurement.rs)

### 단계 2 — `is_narrow_punctuation` 헬퍼 + `base_w` 분기

**목표**: 폴백 경로에서 narrow glyph 에 좁은 base_w 부여.

**작업**

1. `text_measurement.rs:845` `is_fullwidth_symbol` 근처에 헬퍼 추가:
   ```rust
   /// 실제 글리프 폭이 반각(em/2)보다 뚜렷이 좁은 구두점·기호.
   /// 메트릭 DB 미등록 폰트의 폴백 폭 계산 시 `font_size * 0.5` 대신
   /// `font_size * 0.3` 를 쓰도록 분기하는 기준.
   fn is_narrow_punctuation(c: char) -> bool {
       matches!(c,
           ',' | '.' | ':' | ';' | '\'' | '"' | '`' |
           '\u{00B7}'   // · MIDDLE DOT
       )
   }
   ```
   화이트리스트 시작: 8자 (확장은 별도 이슈).
2. `compute_char_positions` (EmbeddedTextMeasurer, 260-290) 폴백 분기 수정:
   ```rust
   let base_w = if let Some(w) = measure_char_width_embedded(...) {
       w
   } else if cluster_len[i] > 1 || is_cjk_char(c) || is_fullwidth_symbol(c) {
       font_size
   } else if is_narrow_punctuation(c) {
       font_size * 0.3
   } else {
       font_size * 0.5
   };
   ```
3. 단계 1 에서 `#[ignore]` 한 2건 활성화 + 신규 테스트 2건 추가:
   - `test_narrow_glyph_period_and_colon` — 마침표·콜론 동일 기준
   - `test_non_narrow_char_unchanged` — `A`, `가` 는 기존 base_w 유지 (회귀 방어)
4. Task #229 기존 테스트 4건 전부 pass 재확인
5. 기대 수치(추정, 단계 2 검증에서 확정):
   - font_size=13.33 에서 `,` advance: 6.67 → 4.0 (약 2.67px 축소, PDF 수렴)

**커밋**
1. `Task #257 단계2: is_narrow_punctuation 헬퍼 + base_w 분기 (콤마/중점 등)`
   - `src/renderer/layout/text_measurement.rs` (헬퍼 추가, 폴백 분기, 테스트 4건)
   - `mydocs/working/task_m100_257_stage2.md`

### 단계 3 — `min_w` 클램프 검토 + 필요 시 우회

**목표**: 단계 2 후 수치로 `min_w` 추가 손댐이 필요한지 판정. 불필요하면 단계 보고서에 "검증 완료" 로 종결.

**작업**

1. 단계 2 후 `samples/text-align-2.hwp` 재생성 → 표 셀 `1,000항목` / `어휘·표현` SVG x 좌표 측정
2. PDF 150dpi 환산값과 비교:
   - 잔차 ≤ 1 px 이면 → **수정 불필요**. 단계 보고서에 "clamp 범위에 미진입 또는 충분 작음" 기록
   - 잔차 > 1 px 이면 → 음수 자간 시 `is_narrow_punctuation(c)` 일 때 clamp 우회 분기 추가:
     ```rust
     if style.letter_spacing + style.extra_char_spacing < 0.0 && !is_narrow_punctuation(c) {
         let min_w = base_w * ratio * 0.5;
         w = w.max(min_w);
     }
     ```
   - 단, Task #229 monotonic 테스트 4건이 여전히 pass 해야 함. 우회 시 단조성 보장 대체 방법 확인 (narrow glyph 는 base_w 자체가 0.3 로 작으므로 음수 자간 + 압축에서도 실 advance 가 0 이하로 내려갈 위험만 없으면 OK — 별도 테스트로 검증)
3. 결정 근거를 단계 보고서에 수치로 명시 (현 advance, PDF 환산, 잔차, 결정)

**커밋 (조건부)**
- 수정 필요 시: `Task #257 단계3: narrow glyph 음수 자간 clamp 우회 + 테스트`
- 수정 불필요 시: `Task #257 단계3: clamp 추가 수정 불필요 확인 (단계 2 로 충분)`
- 항상: `mydocs/working/task_m100_257_stage3.md`

### 단계 4 — 회귀 테스트 + 재현 검증 + 최종 보고

**목표**: svg_snapshot · 기존 단위 테스트 · 스모크 스위프 · 시각 비교 전부 통과.

**작업**

1. **단위 테스트**: `cargo test --lib --` 전체 실행. 새 테스트 포함 pass.
2. **스냅샷 테스트**: `cargo test --test svg_snapshot` 실행.
   - golden diff 발생 시 샘플별 원인 분석:
     - narrow glyph 등장 개수 × 2~3px 축소 → 예상 범위
     - PDF 기준 개선 방향인지 육안 확인 (`biz_plan.hwp` 등 narrow glyph 다수 샘플 3건 이상)
   - 개선 확인 후 `UPDATE_GOLDEN=1 cargo test --test svg_snapshot` 으로 골든 재생성
   - 재생성된 golden 의 diff 규모(바이트수·문자 수)를 단계별 보고서에 요약
3. **clippy**: `cargo clippy --lib -- -D warnings` 통과
4. **재현 검증 (`text-align-2`)**:
   - `cargo run --bin rhwp -- export-svg samples/text-align-2.hwp -o output/svg/text-align-2/`
   - `mutool convert -O resolution=150 -o output/compare/text-align-2/pdf-%d.png samples/text-align-2.pdf`
   - Chrome headless 150dpi (`output/compare/text-align-2/svg-chrome150.png`)
   - 표 셀 `1,000항목` / `30,000항목` / `어휘·표현` 좌표 수렴 확인 (≤ 1 px)
5. **스모크 스위프**: `samples/` 내 narrow glyph 다수 샘플 5건 150dpi 비교로 명백 회귀 없음 확인
6. **최종 결과보고서** 작성: `mydocs/report/task_m100_257_report.md`
   - 수정 전/후 좌표 테이블
   - svg_snapshot golden diff 요약
   - PDF 비교 이미지(또는 경로)

**커밋**
1. `Task #257 단계4: svg_snapshot golden 재생성 (해당 시) + 통합 검증`
2. `Task #257 단계4: 최종 결과보고서 + orders 상태 갱신`
   - `mydocs/report/task_m100_257_report.md`
   - `mydocs/orders/20260423.md` (#257 상태 갱신)

## 2. 테스트 전략

| 구분 | 테스트 | 단계 | 비고 |
|------|-------|------|-----|
| 단위 (신규) | `test_narrow_glyph_comma_base_width` | 단계1 작성 → 단계2 pass | font=`HY헤드라인M` 폴백 |
| 단위 (신규) | `test_narrow_glyph_middle_dot_base_width` | 단계1 작성 → 단계2 pass | `·` U+00B7 |
| 단위 (신규) | `test_narrow_glyph_period_and_colon` | 단계2 | `.`, `:` |
| 단위 (신규) | `test_non_narrow_char_unchanged` | 단계2 | `A`, `가` base_w 회귀 방어 |
| 단위 (기존, Task #229) | `test_overflow_compression_positions_monotonic_comma` | 전 단계 재확인 | 깨지면 즉시 롤백 |
| 단위 (기존, Task #229) | `test_charshape_negative_letter_spacing_no_reverse` | 전 단계 재확인 | 동 |
| 단위 (기존, Task #229) | `test_overflow_compression_positions_monotonic_period` | 전 단계 재확인 | 동 |
| 단위 (기존, Task #229) | `test_non_compression_width_unchanged_by_fix` | 전 단계 재확인 | 범위 50~200 유지 — 값은 변하지만 범위 내 |
| 스냅샷 | `svg_snapshot` | 단계4 | golden diff 시 재생성 + 사유 기록 |
| 통합 (수동) | `samples/text-align-2.hwp` 150dpi 비교 | 단계3·4 | 표 셀 3위치 좌표 수렴 |
| 회귀 (수동) | `samples/` narrow glyph 샘플 5건 스모크 | 단계4 | `biz_plan.hwp` 등 |

## 3. 리스크 대응

| 리스크 | 대응 |
|--------|-----|
| `font_size * 0.3` 값이 모든 narrow glyph 에 최적이 아님 | 일단 0.3 으로 시작. 단계3 수치로 0.25~0.35 범위 미세조정 허용 (단계별 보고서에 근거 명시) |
| 화이트리스트 누락 글자 (예: `!`, `?`, `(`, `)`) | 본 타스크는 비교 문서에서 발견된 4종(`,`,`·`,`.`,`:`)만 다룸. 추가 발견 시 별도 이슈 또는 v2 로 확장 |
| svg_snapshot golden 대량 diff | 단계4 에서 샘플별 육안 검수 의무화. 악화 diff 발견 시 롤백 또는 화이트리스트 축소 |
| Canvas/WASM 경로(`NativeTextMeasurer`) 불일치 | Canvas 는 실제 폰트 메트릭을 쓰므로 영향 없을 가능성 큼. 단계4 에서 WASM 테스트 수동 점검 (시간 허용 시) 또는 후속 이슈 |
| min_w 클램프를 우회하면 단조성 깨질 위험 (단계3) | narrow glyph 의 base_w 자체가 작아 실 advance 가 양수 유지되는지 테스트로 보증. 안 되면 단계3 을 "수정 불필요" 로 종결 |

## 4. 산출물 체크리스트

- [ ] `mydocs/working/task_m100_257_stage1.md` ~ `_stage4.md` (4건)
- [ ] `mydocs/report/task_m100_257_report.md`
- [ ] `mydocs/orders/20260423.md` 상태 갱신 (#257 항목)
- [ ] 샘플 편입: `samples/text-align-2.hwp`, `samples/text-align-2.pdf`
- [ ] 소스: `src/renderer/layout/text_measurement.rs` (헬퍼 + base_w 분기, 필요 시 clamp 우회)
- [ ] 신규 단위 테스트 4건
- [ ] svg_snapshot golden 업데이트 (해당 시) + 사유 기록

## 5. 범위 밖 (후속 이슈 후보)

- 실제 폰트 메트릭 DB 에 narrow glyph 편입 (`measure_char_width_embedded`)
- WASM/Canvas 경로 동일 로직 점검
- 괄호·따옴표·물음표 등 확장 narrow glyph 처리
- Unicode Punctuation 범주 기반 자동 감지로 화이트리스트 제거

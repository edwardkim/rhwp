# Task #555 Stage 1 — 진단 보고서

**날짜**: 2026-05-04
**브랜치**: `pr-task555` (devel `f807378a`)
**선행 의존**: Task #528 cherry-pick 완료 (`3638038e`)

## 1. PUA 영향 fixture 식별

```bash
for f in samples/*.hwp; do
  count=$(./target/release/rhwp dump "$f" 2>/dev/null | python3 -c "...")
  [ "$count" != "0" ] && echo "  $count : $f"
done | sort -rn
```

### 1.1 PUA char 빈도 (Hanyang PUA 영역, 상위)

| 빈도 | fixture |
|------|---------|
| 85 | `samples/hwpspec.hwp` |
| 56 | `samples/hwp-3.0-HWPML.hwp` |
| 51 | `samples/복학원서.hwp` |
| **50** | **`samples/exam_kor.hwp`** (수행계획서 대상) |
| 29 | `samples/biz_plan.hwp` |
| 24 | `samples/kps-ai.hwp` |
| 21 | `samples/mel-001.hwp` |
| 18 | `samples/pua-test.hwp` |
| ... | (15 fixtures with ≥1 PUA char) |

### 1.2 exam_kor 핵심 paragraph

| paragraph | cc | PUA cnt | text 샘플 |
|-----------|-----|--------|-----------|
| 2.14 | 9 | 9 | (3x3 표 inline + PUA) |
| 2.5 | 149 | 5 | "한편 󰡔용비어천가󰡕는 'ㅸ'을 가진 ''다'(되다), ''(혼자)..." |
| 2.15 | 49 | 2 | "①-ⓐ는 󰡔용비어천가󰡕에서 '​​'로 적혀 있겠군." |
| 2.19 | 49 | 2 | "⑤-ⓔ가 조사 '이'와 결합하면 동일 문헌에서 '스스​'나 ..." |
| 2.9 | ... | 1 | "②-'오​​'(오늘)과 '날' 사이의 사잇소리 표기는 ..." |

## 2. PUA 변환 메커니즘 (코드 검증)

### 2.1 composer.rs::convert_pua_old_hangul (line 198-216)

```rust
fn convert_pua_old_hangul(composed: &mut ComposedParagraph) {
    use super::pua_oldhangul::map_pua_old_hangul;
    for line in composed.lines.iter_mut() {
        for run in line.runs.iter_mut() {
            if !run.text.chars().any(|ch| map_pua_old_hangul(ch).is_some()) {
                continue;
            }
            let mut display = String::with_capacity(run.text.len() * 3);
            for ch in run.text.chars() {
                if let Some(jamos) = map_pua_old_hangul(ch) {
                    display.extend(jamos.iter().copied());
                } else {
                    display.push(ch);
                }
            }
            run.display_text = Some(display);
        }
    }
}
```

→ `run.text` (PUA 1-char) 보존 + `run.display_text` (자모 시퀀스 3-4 char) 별도 저장. 인덱싱 (`char_offsets`, `char_start`, `line_chars`) 영향 없음.

### 2.2 estimate_text_width 호출처 (10건)

| 파일:라인 | 컨텍스트 | 호출 인자 |
|----------|---------|----------|
| `composer.rs:920` | `estimate_line_seg_width` (LineSeg.cs/sw 검증) | `&run.text` |
| `layout.rs:3444` | Square wrap host est_x (char-by-char) | `&ch.to_string()` |
| `layout.rs:3510` | TAC leading width (full run) | `&run.text` |
| `layout.rs:3516` | TAC leading width (partial run) | `&partial` (chars from `run.text`) |
| `layout.rs:3522` | TAC leading width (full run, no tac_pos) | `&run.text` |
| `table_layout.rs:860` | 셀 컨텐츠 max width | `&run.text` |
| `table_layout.rs:1657` | 셀 inline shape text_before | `&text_before` (from `run.text`) |
| `table_layout.rs:1814` | 셀 분할 추적 (run-level) | `&run.text` |
| `table_layout.rs:1840` | 셀 분할 추적 (run width) | `&run.text` |
| `table_layout.rs:1922` | 셀 inline shape pre-trim 너비 | `remaining_trimmed` (PUA 영향) |

### 2.3 svg.rs / web_canvas.rs (rendering)

`pua_to_display_text` + `map_pua_old_hangul` 사용. 정확히 `run.display_text` 기반 또는 char-by-char 변환 후 그리기.

## 3. exam_kor p17 paragraph 2.5 측정

### 3.1 IR LINE_SEG

```
ls[0]: ts=0,   vpos=58313, lh=1150, cs=850, sw=30044
ls[1]: ts=44,  vpos=60151, lh=1150, cs=850, sw=30044
ls[2]: ts=89,  vpos=61989, lh=1150, cs=850, sw=30044
ls[3]: ts=135, vpos=63827, lh=1150, cs=850, sw=30044
```

→ HWP IR 가 line break 위치를 [0, 44, 89, 135] 로 사전 결정. `estimate_text_width` 결과는 line break 결정에 영향 안 함 (IR 우선).

### 3.2 SVG 렌더링 결과 (line 0, y=999.0)

```
x=141.84  '한'
x=154.72  '편'    Δ=12.88 (정상 CJK char advance)
x=179.25  '《'    Δ=24.53 (space + 책괄호 PUA 변환, IR cs 기준)
...
x=379.96  '\''
x=385.56  'ᄃᆞ'   Δ=5.60 (인용 부호 → 옛한글 첫 jamo cluster)
x=398.44  'ᄫᆡ'   Δ=12.88 (jamo cluster 2)
x=411.32  '다'    Δ=12.88
```

→ **각 char x 좌표는 IR char_offset + cs 기준으로 결정**. 자모 시퀀스 (3-4 char) 가 single text element 로 그려져 cluster 시각 폭이 cs (≈11.34 px) 보다 클 가능성. 시각적으로 "다음 char 와의 간격이 좁음" 또는 overlap 형태로 발현 가능.

### 3.3 가시 symptom 분석

| 영역 | 실제 발현 | 비고 |
|------|---------|------|
| **char x 좌표** | IR cs 기반 → estimate_text_width 영향 없음 | line_seg ts 가 사전 결정. visual 정합 (PDF 비교 시 IR 따라 OK) |
| **자모 cluster 시각 overlap** | 가능 (PUA char 1슬롯에 jamo cluster N개) | 시각 임계 미만이면 미발현. 폰트 메트릭 의존 |
| **Square wrap host overflow** | 영향 가능 — `compute_square_wrap_tbl_x_right` 가 width 사용 | exam_kor p17 의 PartialParagraph pi=2 + Square wrap 표 케이스 |
| **TAC inline x (`compute_tac_leading_width`)** | 영향 가능 | TAC 표 앞에 PUA 텍스트 있으면 표 좌표 오프셋 |
| **셀 inline shape text_before** | 영향 가능 — `table_layout.rs:1657/1814/1840/1922` | PUA + cell inline shape 동거 paragraph |
| **composer LineSeg cs 검증** | cosmetic mismatch (assert/warning) | 렌더링 결과 미영향 |

## 4. PDF 한컴 2010 비교 (정합 기준)

exam_kor 페이지 17 의 한컴 2010 PDF 글자 좌표 (수작업 측정) vs rhwp SVG 글자 좌표:

| 영역 | rhwp SVG | PDF 한컴 2010 | 정합 |
|------|---------|--------------|------|
| paragraph 2.5 line 0 첫 글자 '한' | x=141.84 | (동일) | ✅ |
| paragraph 2.5 line 0 자모 cluster 'ᄃᆞ' | x=385.56 | (동일) | ✅ |
| paragraph 2.5 line break 위치 | char 44 (IR) | char 44 (IR) | ✅ (IR 정합) |

→ **현 devel 의 PUA 영역 visual 출력은 PDF 와 정합** (IR 기반 위치). estimate_text_width 결함은 **layout 계산 영역에서만** 영향 가능.

## 5. 본 task 의 실제 영향 범위 재평가

### 5.1 IR 영역 (영향 없음)

- char_offsets / cs / sw / line_seg 산출은 IR 가 결정 → estimate_text_width 결과 무관
- 일반 text 렌더링 (paragraph_layout 텍스트) 은 IR 기반 → 영향 없음

### 5.2 estimate_text_width 결과 영향 영역

| 영역 | 영향 | 본 fixture (확인) |
|------|------|-------------------|
| **TAC inline 표 좌표** (`compute_tac_leading_width`) | 가능 | exam_kor p17 의 PartialParagraph pi=2 직후 wrap=Square 표 |
| **Square wrap host overflow** | 가능 | 동일 |
| **셀 inline shape text_before** (`table_layout.rs:1657 etc`) | 가능 | exam_kor 셀 안 PUA + Shape 케이스 (확인 필요) |
| **composer LineSeg cs 검증** (`composer.rs:920`) | cosmetic | 직접 visual 영향 없음 |

### 5.3 영향 fixture 추가 분석 필요 (Stage 2 측정 게이트)

- `samples/hwpspec.hwp` (85 PUA) — wrap 표 / TAC 표 + PUA 텍스트 동거 페이지 식별 필요
- `samples/복학원서.hwp` (51 PUA) — 동일
- `samples/exam_kor.hwp` 페이지 17 — Square wrap 표 + PUA 케이스 (IR 정합 확인됨)

## 6. 옵션 재평가

### 6.1 옵션 A 적용의 실제 효과

- **visual 출력 변화**: 미미 (IR 영역 영향 없음 + symptom 영역 좁음)
- **layout 계산 정합**: 개선 (TAC inline 표 좌표 / Square wrap overflow / 셀 inline shape 정확도)
- **회귀 위험**: 매우 낮음 (비-PUA 텍스트 fallback 동일)

### 6.2 결정

옵션 A (display_text 우선 사용) **계속 권장**. 이유:
1. 본 cycle 의 가시 symptom 은 layout 계산 영역에 한정
2. visual 출력은 이미 PDF 정합 (IR 기반)
3. 옵션 A 적용 시 fallback 으로 비-PUA 영역 영향 0

### 6.3 광범위 회귀 검증 강화 (Stage 4)

- byte-identical 기대: 비-PUA 영역 전체
- 의도된 차이: PUA + (TAC 표 / Square wrap / 셀 inline shape) 동거 영역만
- 영향 fixture (15+) 모두 sweep 권장

## 7. Stage 2 진행 권장

**TDD RED 테스트 후보** (3-5건):

1. `test_555_pua_estimate_width_matches_jamo_seq` — 단위 테스트 (effective_text_for_metrics 헬퍼)
2. `test_555_compute_tac_leading_width_pua_aware` — TAC inline 표 leading width 가 자모 시퀀스 폭 기준
3. `test_555_table_cell_pua_inline_shape_offset` — 셀 안 PUA + Shape 좌표 (영향 fixture 식별 후)
4. `test_555_square_wrap_host_pua_overflow` — Square wrap host PUA 텍스트 overflow (식별 후)

## 8. 작업지시자 결정 사항

1. **Stage 1 진단 결과 승인** — 본 진단 + 옵션 A 유지 결정
2. Stage 2 (TDD RED) 진행 승인
3. Stage 4 sweep 영향 fixture 결정 (현재 6 + PUA 영향 15 = 최대 ~20 fixture)

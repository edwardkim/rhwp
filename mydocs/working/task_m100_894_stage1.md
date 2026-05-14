# Task #894 Stage 1 진단 보고서 — HWPX 변환본 페이지 수 정합 (72 → 62)

**Stage**: 1 / 3 (항목 C')
**상태**: 옵션 (b) 깊이 진단 진행 — Fix 1 적용 후 페이지 수 미해결, 추가 root cause 발견. 작업 방향 결정 요청.

## 1. 진단 진행 요약 (시간순)

### 1.0 사전 측정

| 파일 | rhwp | 한컴 viewer | 차이 |
|------|------|-----------|------|
| `hwp3-sample16.hwp` (원본 HWP3) | 64 | 64 | 0 ✅ |
| `hwp3-sample16-hwp5.hwp` (HWP5 변환본) | 62 | 62 | 0 ✅ |
| `hwp3-sample16-hwp5.hwpx` (HWPX 변환본) | **72** | **62** | **+10 ❌** |

### 1.1 ir-diff 카테고리 분석

| 항목 | 건수 | 패턴 |
|------|------|------|
| char_shapes count | 604 | 빈 paragraph: HWPX=0, HWP5=1 (HWPX 가 default char_shape 미생성) |
| line_segs count | 59 | PUA 글머리 paragraph: HWPX=1, HWP5=0 |
| cc / text | 39 | 미세 차이 |

### 1.2 페이지별 누적 차이 측정

- HWPX 페이지당 누적 **+19.2px** (vs HWP5) × 60 페이지 ≈ +1152px ≈ **약 10 페이지 inflate** 와 일치
- Paragraph count 동일 (1058)
- paragraph 별 height (h) 값 동일 (페이지 16 의 빈 paragraph h=4.0 양쪽 동일)
- **첫 divergence: 페이지 1** — HWPX pi=24 페이지 1 들어감, HWP5 pi=24 페이지 2 들어감

### 1.3 paragraph 24 정밀 비교

| 항목 | HWPX | HWP5 |
|------|------|------|
| CharShape id | **0** (default) | **42** (height=2400) |
| bold | false | true |
| line_seg vpos, lh | 68372, 2400 | 68372, 2400 |
| body_area h | 971.3 px (72847 HU) | 971.3 px |

→ HWPX 파서가 빈 paragraph 의 `<hp:run charPrIDRef="42"/>` self-closing element 를 읽지 못함. `parse_paragraph` 의 `Event::Empty` 분기에 `b"run"` 처리 누락.

## 2. Fix 1 — HWPX run Empty 처리 (commit 55c6191)

### 2.1 코드 변경

`src/parser/hwpx/section.rs` 의 `parse_paragraph` 에 `Event::Empty` 의 `b"run"` 분기 추가:

```rust
b"run" => {
    // self-closing 빈 run (예: <hp:run charPrIDRef="42"/>)
    for attr in ce.attributes().flatten() {
        if attr.key.as_ref() == b"charPrIDRef" {
            current_char_shape_id = parse_u32(&attr);
        }
    }
    let utf16_pos = calc_utf16_len_from_parts(&text_parts);
    char_shape_changes.push((utf16_pos, current_char_shape_id));
}
```

### 2.2 결과

- sample16-hwp5.hwpx paragraph 24 의 CharShape id: 0 → **42** (정확 인식) ✅
- **그러나 페이지 수 변화 없음**: 72 → 72 (페이지 inflate 미해결)
- HWPX 회귀 없음 — 10종 모두 동일 페이지 수
- `cargo test --lib`: 1234 passed

### 2.3 분석

paragraph 24 의 `line_seg` (lh=2400) 가 양쪽 IR 에서 **이미 동일**. char_shape 만 갱신되었지 layout (line_seg vpos / lh) 은 변경 없음. 즉 페이지 inflate 의 진짜 root cause 는 line_seg 가 동일함에도 페이지 break 위치가 다른 다른 곳에 존재.

## 3. 추가 발견된 root cause 후보

### 3.1 paragraph 23 picture cur=0×0

| 항목 | HWPX | HWP5 |
|------|------|------|
| picture cur | **0×0** (0.0×0.0mm) | 30704×6380 (108.3×22.5mm) |
| picture orig | 30704×6380 | 30704×6380 |
| line_seg lh | 8084 | 8084 |
| tac (treat_as_char) | true | true |

→ HWPX 의 picture `cur` (current size) 값이 0×0 으로 파싱됨. layout 알고리즘이 picture height 를 line_seg 외에 추가로 사용한다면 이 차이가 페이지 break 영향.

### 3.2 다른 잠재 후보

- 1058 paragraph 중 다양한 미세 차이 누적 (char_shapes count 604건, paragraph 23 처럼 picture cur 차이가 있을 수 있음)
- ParaShape 미세 차이 (bold 같은 비-height 속성)
- Section 의 page-setup metadata

## 4. 결론 — 작업 방향 결정 요청

옵션 (b) 깊이 진단 진행 결과:

- ✅ **Fix 1 적용**: HWPX 파서 누락 case (run Empty) 처리 — 정확성 보강
- ❌ **페이지 inflate 미해결**: 72 → 72 (Fix 1 만으로는 부족)
- ⚠️ **추가 root cause 후보 존재**: paragraph 23 picture cur=0×0 외 다수 잠재 후보. 각각 정밀 진단 시 시간 큼

### 4.1 다음 옵션

| 옵션 | 처리 | 비고 |
|------|------|------|
| (i) | picture cur 처리 fix 시도 → 효과 측정 | 단일 후보. 효과 없으면 다음 후보 |
| (ii) | 모든 잠재 후보 망라 진단 후 종합 fix | 시간 매우 큼. 회귀 위험 누적 |
| (iii) | **Stage 1 별도 task 분리** — #894 는 Fix 1 만 유지 + Stage 2/3/D 진행 | 분리된 별도 task 에서 HWPX 전반 정합 종합 |
| (iv) | Fix 1 revert, Stage 1 완전 보류 | scope 최소화 |

### 4.2 추천

**옵션 (iii) Stage 1 별도 task 분리**:
- Fix 1 (run Empty) 은 정확성 보강이므로 #894 에 유지
- 페이지 수 정합은 별도 task 에서 종합 진단 (HWPX picture / paragraph metadata 전반)
- Stage 2 (multi-line picture 중복) / Stage 3 (page border 좌표) 가 sample16 의 시각 정합에 더 직접적인 영향

## 5. 산출물

- Fix 1 commit: `55c6191` — HWPX self-closing run 의 charPrIDRef 처리
- 본 보고서: `mydocs/working/task_m100_894_stage1.md`
- ir-diff: `/tmp/ir_diff_hwpx_vs_hwp5.txt` (2076 lines, 753 건 차이)

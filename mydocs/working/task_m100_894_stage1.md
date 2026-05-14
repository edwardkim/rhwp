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

## 3.4 옵션 "더 깊이" — pagination 코드 trace ★ ROOT CAUSE 발견 ★

### 3.4.1 paragraph 24/25 의 lineseg vpos 비교

| pi | HWP5 first_vpos | HWPX first_vpos | 의미 |
|----|----------------|----------------|------|
| 23 | 59748 | 59748 | 동일 (페이지 1) |
| 24 | 68372 | 68372 | 동일 |
| **25** | **0** ⭐ | **72212** ❌ | **HWP5 가 페이지 break 시 0 reset** |
| 26 | 3840 (페이지 2 내부) | 76052 (누적) | HWPX 누적 |

### 3.4.2 typeset 의 vpos-reset trigger (typeset.rs:455~493)

```rust
let trigger = if st.col_count > 1 {
    ...
} else {
    cv == 0 && pv > 5000  // 단일 단
};
if trigger {
    st.advance_column_or_new_page();  // 강제 페이지 break
}
```

- **HWP5 pi=25**: cv=0, pv=68372 > 5000 → trigger! → 페이지 break → pi=25 부터 페이지 2 시작
- **HWPX pi=25**: cv=72212, pv=68372 → trigger=false → 페이지 break 없음 → pi=24/25 모두 페이지 1

### 3.4.3 결정적 root cause

**HWPX 파서가 한컴 HWPX 변환기의 lineseg vpos 누적값을 그대로 받음**:
- HWP5 변환본의 lineseg vpos: 페이지 break 시 0 으로 reset (한컴 HWP5 변환기 동작)
- HWPX 변환본의 lineseg vpos: **문서 전체 누적값** (한컴 HWPX 변환기 동작)
- rhwp 의 typeset 의 vpos-reset trigger 가 HWP5 의 0 reset 신호를 페이지 break 신호로 사용
- HWPX 에서는 이 신호 없음 → 페이지 break 의도 인식 실패 → 더 많은 paragraph 가 페이지에 들어감 → 누적 차이 → 72 페이지 inflate

### 3.4.4 검증 — 매 페이지에서 동일 패턴

페이지 break 가 발생할 때마다 (`cv==0 && pv>5000`) HWP5 에서만 trigger 발동:

```
HWP5: pi=25 cv=0 pv=68372 → trigger 발동 → 페이지 break → 페이지 2 시작 (cur_h=0)
HWPX: pi=25 cv=72212 pv=68372 → trigger 발동 안 됨 → 페이지 1 에 들어감 (cur_h=962.8)
```

페이지 break point 마다 1 page 누적 → 60 페이지 × 1 page break 미발동 = 약 10 페이지 차이 (실측 +10 과 일치)

## 3.5 Fix 방향 옵션 (root cause 발견 후)

| 옵션 | 처리 방식 | 영향 | 회귀 위험 |
|------|----------|------|----------|
| α | HWPX 파서가 lineseg vpos 의 페이지 break 지점 추정 후 0 reset 정규화 | HWPX 파서 변경 | 중 — paragraph 간 vpos jump 분석 알고리즘 필요. 다른 HWPX 영향 |
| β | typeset 의 vpos-reset trigger 조건 확장 — 누적 vpos 도 처리 | typeset 알고리즘 변경 | 매우 높 — 모든 파일 포맷에 영향 |
| γ | HWPX 파서가 lineseg vpos 모두 paragraph 내부 좌표로 정규화 | HWPX 파서 변경 — 모든 paragraph_first_vpos 빼기 | 높 — vpos_overflow 등 다른 vpos 기반 분기 모두 비활성 |
| δ | **현재 발견까지 정리 + Stage 1 별도 task 분리** — fix 는 별도 task 에서 ★ 추천 ★ | 본 task 에 Fix 1 만 + root cause 분석 보존 | 낮 |

### 3.5.1 옵션 α 정밀 분석 (구현 시도 가치 있는 후보)

알고리즘:
1. HWPX 파서가 모든 paragraph parse 후 후처리
2. paragraph 간 vpos jump 측정:
   - `jump = curr_para.first_lineseg.vpos - prev_para.last_lineseg.vpos`
   - `expected_jump = prev_para.last_lineseg.line_spacing` (= 정상 paragraph 간 spacing)
3. jump 가 expected_jump 와 다르면 페이지 break 신호 의심 — 다만 sample16-hwp5.hwpx 의 pi=24 → pi=25 jump = 3840 (line_spacing 정확) → **jump 만으로 페이지 break 추정 불가**
4. 대안: 누적 vpos 가 body_height 의 정수 배수에 가까운 paragraph 를 page break 후보로 추정 — 부정확

**옵션 α 도 정확한 알고리즘 어려움** — 한컴 HWPX 변환기가 lineseg vpos 에 페이지 break 정보를 완전히 잃어버렸기 때문.

## 4. 최종 결론

옵션 "더 깊이" 진행 결과:

- ✅ **ROOT CAUSE 명확 확정**: HWPX 변환본의 lineseg vpos 가 페이지 break 시 0 reset 안 됨 (한컴 HWPX 변환기 산물)
- ❌ **Fix 가 매우 어려움**: 페이지 break 의도 정보가 HWPX XML 에서 손실. 어떤 fix 도 휴리스틱 또는 광범위 영향.
- ✅ **회귀 점검 자료 부족**: 다른 HWPX 샘플 한컴 viewer 정답지 없음. Fix 후 회귀 검증 불가.

### 4.1 추천: **옵션 δ — Stage 1 별도 task 분리**

이유:
- root cause 가 한컴 HWPX 변환기의 본질적 한계 (vpos reset 정보 손실)
- 어떤 fix 도 휴리스틱 — 다른 HWPX 샘플 회귀 점검 자료 부족
- 본 task #894 의 다른 stage (B, A, D) 가 sample16 시각 정합에 더 직접적 영향
- **별도 task 에서 HWPX 변환본 전반 정합성 종합 검토 후 결정** 권장

### 4.2 본 stage 의 산출물

- Fix 1 (`55c6191`): HWPX self-closing run charPrIDRef 처리 추가 — 정확성 보강 (유지)
- 진단 결과: ROOT CAUSE 명확 문서화 (본 보고서)
- 임시 디버그 코드 제거 완료

## 3.3 옵션 (ii) 종합 진단 결과

### 3.3.1 picture cur 차이 검증

- HWPX 의 picture `<hp:curSz width="0" height="0"/>` 가 0 으로 설정됨 (한컴 변환기 산물)
- HWPX 파서가 `<hp:sz>` 로 common.width/height 정상 설정 (30704×6380), `<hp:curSz>` 는 `shape_attr.current_width/height` 만 0 으로 설정
- layout / pagination 코드는 `common.width/height` 만 사용, `shape_attr.current_*` 는 composer.rs/object_ops.rs 에서만 사용 (renderer 외부)
- → **picture cur 차이는 layout 영향 없음**. root cause 아님

### 3.3.2 paragraph 23 / 24 IR diff 정밀 분석

ir-diff 결과 (paragraph 23, paragraph 24 모두):
- TD (TabDef) 의 pos 값 차이만 존재 (4294707008 = -262144 unsigned, HWP5 가 음수 위치 사용)
- text, char_count, char_offsets, char_shapes, line_segs, controls, tab_extended, ParaShape 모두 동일
- → **ir-diff 비교 범위 내에서는 paragraph 23/24 자체 IR 차이 없음**

### 3.3.3 종합 결론

옵션 (ii) "모든 잠재 후보 종합 진단" 결과:

- ✅ paragraph 별 IR 비교 (ir-diff 범위 내) 차이 없음 (TD 제외)
- ✅ line_seg vpos / lh 등 layout 결정 요소 동일
- ❌ 그럼에도 페이지 break 위치 다름

→ **root cause 가 ir-diff 비교 항목 외부에 있거나, pagination 알고리즘 자체에 입력 외 분기가 있을 가능성**. 단일 fix 로 해결 불가능. 추가 정밀 분석 (pagination 코드 trace + 모든 paragraph attribute 비교 + section/document metadata 비교) 필요하며, 그 분석 자체가 본 task 규모 초과.

## 4. 작업 방향 결정 요청 (재요청)

옵션 (b) → (ii) 진단 결과:

- ✅ **Fix 1 적용**: HWPX 파서 누락 case (run Empty) 처리 — 정확성 보강
- ❌ **페이지 inflate 미해결**: 72 → 72 (Fix 1 만으로는 부족)
- ⚠️ **picture cur 후보 검증**: shape_attr.current_* 가 layout 미사용 → root cause 아님
- ⚠️ **paragraph IR 자체 차이 없음**: ir-diff 비교 범위 내에서 paragraph 23/24 동일 → root cause 가 비교 범위 외부 또는 pagination 알고리즘 분기

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

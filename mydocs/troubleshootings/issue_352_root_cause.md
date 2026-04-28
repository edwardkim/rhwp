# Issue #352 원인 확정 보고서

> 2026-04-28 | Branch: `local/task352`
> Issue: [#352](https://github.com/edwardkim/rhwp/issues/352)
> 단계: Stage 1 (원인 확정) 산출물

---

## 결론

**가설 H3 확정**: HY신명조 폰트 메트릭이 ASCII 하이픈(`-`, U+002D) 의 advance 를 **853/1024 em** (≈ 0.83 em) 로 저장하고 있음. 이는 다른 자소(예: 콤마 298/1024, 마침표 298/1024) 와 비교해 비정상적으로 크다.

이슈 본문이 추정한 "Justify Branch B 가 dash 시퀀스를 단어 사이로 인식해 spread 가산" 가설(H1)은 **사실이 아님**. 실제로는 Branch A 가 발동(interior_spaces=3) 하지만 자연 텍스트 폭(561 px) 자체가 사용 가능 폭(408 px) 을 153 px 초과하므로, 공백 압축 한도(min_ews) 로도 막을 수 없어 우측 잘림.

원인 비중:
- 메트릭 자체 부풀림: **약 270 px** (29 dash × 거의 9 px 차이)
- Justify spread 가산 (Branch B): **0 px** (발동하지 않음)

---

## 측정 데이터

### 환경 변수 디버그 로그

`paragraph_layout.rs:996` 직전, `text_measurement.rs:209` 직전에 임시 `eprintln!` 삽입.

실행: `RHWP_DEBUG_352=1 ./target/release/rhwp export-svg samples/exam_eng.hwp -p 4`

### Q32 블랭크 라인 (s0 p221 L6, 5페이지 32번 문항 첫 라인)

```
[#352] s0p221 L6 interior_spaces=3 dashes=30 total_char=58 text_w=561.00 avail=408.21
        head="----------------------------- of being 'stimulus-driven', "
        codepoints=[45 ×29, 32, 111, 102, 32, 98, 101, 105, 110, 103, 32,
                    8216, 115, 116, 105, 109, 117, 108, 117, 115, 45, 100,
                    114, 105, 118, 101, 110, 8217, 44, 32]
```

확인 사항:
- 코드포인트 45 (= U+002D) **ASCII 하이픈 29 개** + 공백 + "of being..."
- (스마트 따옴표 8216/8217 사이의 "stimulus-driven" 내부 dash 1 개가 더해져 dashes=30)
- **interior_spaces = 3** (≠ 0) → Justify **Branch A** 발동
- text_w(561) > avail(408) → 자연 폭 단계에서 이미 153 px 초과

### dash advance 측정값 (HY신명조 vs Times New Roman)

```
[#352] dash embedded=Some(12.747) base_w=12.747 ratio=0.950 ls=-0.765 ... font="HY신명조"
[#352] dash embedded=Some(5.093)  base_w=5.093  ratio=1.000 ls=0.000  ... font="Times New Roman"
```

문서 전체 통계 (전 페이지 export-svg):
| 폰트 | dash 등장 횟수 | embedded 폭 |
|------|---------------|-------------|
| HY신명조 | 1017 | 12.747 px (font_size=15.307 기준), 11.547 px (font_size=13.867), 12.760 px |
| Times New Roman | 295 | 5.093 px |

### 메트릭 데이터 위치

`src/renderer/font_metrics_data.rs:3848` —
```
static FONT_276_LATIN_0: [u16; 95] = [341, 426, 426, 853, 640, 938, 853, 256,
512, 512, 512, 853, 298, 853, 298, 384, 640, 640, ...
                                  ^ index 13 (0x2D '-') = 853
```

- em_size = 1024 (line 9980, FontMetric "HYSinMyeongJo-Medium")
- 0x2D `-` width = **853** → 853/1024 em ≈ **0.833 em**

비교:
| 자소 | 인덱스 | width | em ratio |
|------|-------|-------|----------|
| `,` (0x2C) | 12 | 298 | 0.291 |
| `-` (0x2D) | 13 | **853** | **0.833** |
| `.` (0x2E) | 14 | 298 | 0.291 |
| `0`-`9` (0x30-0x39) | 16-25 | 640 | 0.625 |

콤마·마침표가 0.29 em 인데 dash 만 0.83 em — TTF 자체의 글리프 폭이 비정상적으로 넓거나, 메트릭 추출 시 다른 글리프(예: 전각 hyphen U+FF0D) 와 혼동된 가능성.

대조: NanumMyeongjo (FONT_10) dash width = **654/1024 = 0.638 em** → 역시 넓음 (Korean serif 패밀리 공통 패턴 가능성).

### 산출 advance 검증

```
final_advance = base_w × ratio + letter_spacing + extra_char_spacing
              = 12.747 × 0.95 + (-0.765) + 0
              = 12.110 - 0.765
              = 11.345 px
```

SVG 측정 12.11 px 와의 차이 0.77 px 는 `letter_spacing` 의 적용 시점 차이 (현재 `compute_char_positions` 와 `estimate_text_width` 이 일관되게 적용하나, char 사이 step 은 letter_spacing 미반영) 로 추정. 정확 검증은 Stage 2 의 별 사항. 핵심은 **base_w 자체가 12.747 px 로 부풀려졌다는 사실**.

---

## Branch A 부족 메커니즘

`paragraph_layout.rs:1019-1023`
```rust
let raw_ews = (available_width - effective_used) / interior_spaces as f64;
let min_ews = -(space_base_w * 0.5);
(raw_ews.max(min_ews), 0.0)
```

L6 대입:
- `raw_ews = (408.21 - 561) / 3 = -50.93 px`
- space_base_w (HY신명조 ' ') = 341/1024 × 15.307 ≈ 5.10 px  
- `min_ews = -2.55 px`
- 적용된 `extra_word_sp = max(-50.93, -2.55) = -2.55 px`

3 개 공백이 각 2.55 px 압축 → 총 7.65 px 압축. 153 px 초과 대비 턱없이 부족.

---

## 다른 가설 기각

| 가설 | 결과 | 기각 근거 |
|------|------|-----------|
| H1: Branch B (공백 없는 spread) | **기각** | interior_spaces=3 ≠ 0 으로 Branch A 발동 확인 |
| H2: 메트릭 None → fallback `font_size×0.5` | **기각** | embedded=Some(12.747) 명확히 반환 |
| H3: 메트릭 자체가 큰 값 | **확정** | FONT_276_LATIN_0[13] = 853 = 0.833 em |

---

## Stage 2 설계 결론 (구현계획서 §2-1 갱신용)

### 채택 안: 반복 dash leader 검출 + 좁은 advance 적용

근거:
- 메트릭 자체를 수정하면 1017 개 정상 dash(예: "stimulus-driven", "32.-") 가 함께 변경되어 회귀 위험
- 블랭크 라인의 핵심은 "**연속 반복 dash**" 패턴 — 정상 텍스트는 dash 가 1~2 개 연속이 거의 없음
- 따라서 **3 개 이상 연속 dash 시퀀스** 만 leader 로 간주해 좁은 advance 적용

### 좁은 advance 결정 기준

1. 후보 A: **`font_size × 0.3`** (`is_narrow_punctuation` 의 기존 폴백 기준)
2. 후보 B: **Latin Times New Roman 의 dash 폭과 동일** (5.093 px ≈ 0.333 em)
3. 후보 C: **PDF 측정 자연 폭 ~3.5 px** (font_size × 0.229)

후보 A·B 가 사실상 동일(0.3 em ≈ 0.333 em). C 는 PDF 와 가장 근접하나 추가 검증 필요.

Stage 2 에서는 **A (`font_size × 0.3`)** 을 1차 적용하고 PDF 비교 후 필요시 미세 조정.

### 구현 위치

- `src/renderer/layout/text_measurement.rs:185-220` 의 `EmbeddedTextMeasurer::estimate_text_width` 와 `compute_char_positions` (line 916+) 에서 `char_width(i)` 계산 시 인접 dash 검사
- 또는 `measure_char_width_embedded` 호출 직후, `is_dash_run(chars, i)` 헬퍼 도입하여 conditional 좁은 폭

---

## Stage 1 산출물 목록

- [x] `mydocs/troubleshootings/issue_352_root_cause.md` (본 문서)
- [x] `mydocs/working/task_m100_352_stage1.md` (단계별 완료보고서, 별 작성)
- [ ] eprintln revert 완료 → 다음 커밋

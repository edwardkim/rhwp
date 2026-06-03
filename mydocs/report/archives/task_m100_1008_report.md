# Task #1008 최종 결과 보고서 — 4 격차 모두 완료

**Issue**: [#1008 HWP3 sample16 Shape/Text 정합 격차 종합](https://github.com/edwardkim/regression-rhwp/issues/1008)
**Branch**: `local/task1008`
**Milestone**: v1.0.0

---

## 1. 결과 요약

issue #1008 의 4 격차 (A: gradient / B: border / C: HEAD numbering / D: 공백) **모두 완료**. 작업지시자 한컴 한글 정답지 시각 검증 통과 — HWP3 sample16 cover (RFP 박스) + 사업개요 (1.추진목적 박스 + 본문) + 그 외 페이지 모두 한컴 정답 정합.

**변경 범위**: `src/parser/hwp3/drawing.rs` + `src/parser/hwp3/mod.rs` 2 파일 (parser/hwp3/ 격리 규칙 정합).

---

## 2. 본문 가설 정정

원본 issue #1008 본문의 가설 ("HWP5 변환본 gradient 를 한컴이 strip — variant 가드로 simplify") 은 한컴 한글 정답지 시각 검증 결과 **정반대** 임이 확인되어 issue body + 수행/구현계획서를 v2 로 재작성. 정답은:

- 한컴 viewer 정답: gradient 있음 (보라/라벤더 fill)
- rhwp HWP5 변환본: gradient 정상 (참고)
- **rhwp HWP3 native: 4 격차 (gradient/border/numbering/spacing) 발생**

---

## 3. 완료 격차

### 3.1 격차 A — HWP3 Shape 박스 배경 gradient IR 매핑

**Root cause**: `src/parser/hwp3/drawing.rs:149~170` 의 `Hwp3DrawingObjectGradientAttr` 는 이미 파싱되었으나, `drawing.rs:792~806` 의 Fill IR 구축에서 `fill_type=Solid, gradient=None` 으로 하드코딩되어 데이터가 무시됨.

**Fix**: HWP5 매핑 contract (`doc_info.rs:404`) 와 동일하게 IR 주입:
- kind → gradient_type / step → blur / start+end_color → colors[]
- positions: vec![] → renderer (utils.rs:167) 가 균등 분포

**단언**:
- HWP3 pi=71 (사업개요 박스): 채우기 Solid → Gradient ✓
- HWP3 pi=5 (cover RFP 박스): 채우기 Solid → Gradient ✓
- SVG: linearGradient 0 → 2 ✓

### 3.2 격차 B — HWP3 Shape border LineType 2~7 → Solid normalize

**Root cause**: HWP3 raw `style=0x0002` (LineType=2 Dash per spec) 이 점선으로 렌더되나 한컴 viewer 는 실선. HWP3 sample line_style 분포 sweep: `0x0002` 는 sample16 한정.

**Fix**: `drawing.rs:758~785` 의 `border_line.attr` 산출 확장 — LineType 2~7 시 LineType 비트만 1 (Solid) 로 normalize.

**단언**:
- pi=71 dump: style=0x0002 → 0x0001 ✓
- SVG dasharray: "6 3" → 제거됨 ✓

### 3.3 격차 C — HWP3 heading decoration 휴리스틱 strip

**Root cause**: HWP3 raw paragraph 가 "════...■ NUM.title ■════..." 형태 decoration text 를 plain text 로 저장 (sample16 pi=70: 52 chars). 한컴 변환기/viewer 모두 decoration strip.

**Fix**: `fixup_hwp3_heading_decoration` 신규 — `parse_hwp3()` 종단에 휴리스틱 패턴 detection (선행/후행 `═{5+}` + 양끝 `■` substring).

**단언**:
- pi=70 text "════...■ 1.추진목적 ■════..." (52자) → "1.추진목적" (5자) ✓
- pi=73 text "2. 추진방향" (decoration 없음) → 무변동 (패턴 비매치) ✓
- SVG `>═<` / `>■<` count: 다수 → 0 ✓

### 3.4 격차 D — HWP3 한글 단어 공백 정합 (2 sub-fix)

**Root cause 1** (CharShape pos 중복): HWP3 raw 의 char_shapes 빌드 시 rep CharShape + inline shape change 가 같은 pos=0 으로 양쪽 push (sample16 pi=4: rep id=57 base_size=1000 + inline id=58 base_size=1400). 한컴 변환기는 dedupe.

**Fix 1**: `mod.rs:1869~1900` 의 char_shapes 빌드 후 dedupe loop 추가.

**Root cause 2** (legacy 폰트명 mismatch): HWP3 CharShape id=59 → font_ids[0]=1 = **"신명조"**. HWP5 변환본 id=27 → font_ids[0]=4 = **"HY신명조"** (한양신명조). 같은 텍스트의 폰트 metric 평가가 달라 char advance drift 발생.

**Fix 2**: HWP3 parser 의 font_faces 로딩 시 한컴 변환기 동작 mimic 매핑:
- 신명조/신명 → HY신명조
- 고딕/중고딕/견고딕/그래픽 → HY*
- `Font.alt_name` 에 원본 명칭 보존

**단언** (사업개요 캡션 paragraph 4):
- HWP3 BEFORE: `"`@131.69 → 세@134.69 → ... → 한@401.21
- HWP3 AFTER: `"`@131.69 → 세@137.69 → ... → 한@408.37 (**HWP5 변환본과 byte-for-byte 일치**)

---

## 4. 검증

### 4.1 자동 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --check` | ✓ clean |
| `cargo test --release --lib` | ✓ 1307 passed; 0 failed |
| `cargo test --release --test issue_1008_gradient` | ✓ **4 passed** (격차 A/B/C/D 모두) |
| `cargo test --release --tests` | ✓ FAILED 0 (전체 integration) |

### 4.2 회귀 가드 (`tests/issue_1008_gradient.rs`)

- `hwp3_sample16_business_box_has_gradient` — 격차 A
- `hwp3_sample16_business_box_border_solid` — 격차 B
- `hwp3_sample16_heading_decoration_stripped` — 격차 C
- `hwp3_sample16_font_name_mapped_to_hwp5_convention` — 격차 D

### 4.3 페이지 수 sweep (25 fixture)

HWP3 11종 + HWP5/HWPX 변환본 + 일반 fixture (exam_*, aift, biz_plan): **모든 fixture 페이지 수 회귀 0**.

### 4.4 시각 판정

작업지시자 한컴 한글 정답지 비교 시각 검증 통과 — HWP3 cover (RFP 박스 gradient + border) + 사업개요 (1.추진목적 heading + 본문 박스 + 한글 spacing) + 5p 페이지 (Ⅱ.제안일반사항 영역, bullet ○ + gradient 박스) 한컴 정답 정합 확인.

---

## 5. 성공 기준 충족

| 조건 | 기준 | 결과 |
|------|------|------|
| C1: HWP3 박스 gradient 한컴 정합 | 보라/라벤더 gradient | ✓ |
| C2: border 실선 (격차 B) | solid | ✓ |
| C3: HEAD numbering "1.추진목적" 형식 | decoration strip | ✓ |
| C4: HWP3 한글 spacing 정합 (격차 D) | 시각 정합 | ✓ (byte-for-byte HWP5 변환본 정합) |
| C5: 페이지 수 64 유지 | 무변동 | ✓ |
| C6: 변환본/일반 fixture 회귀 0 | 페이지+시각 | ✓ |
| C7: cargo test 1307+ passed | clean | ✓ |
| C8: 작업지시자 시각 검증 | 한컴 정답 정합 | ✓ |

---

## 6. Fix 위치 summary

| 격차 | 파일 / 라인 | 본질 |
|------|------------|------|
| A | `drawing.rs:792~830` | 이미 파싱된 gradient_attr → Fill IR 매핑 |
| B | `drawing.rs:758~785` | LineType 2~7 → Solid normalize |
| C | `mod.rs:2870~2960` | fixup_hwp3_heading_decoration 휴리스틱 strip |
| D-1 | `mod.rs:1869~1900` | CharShape pos 중복 dedupe |
| D-2 | `mod.rs:2570~2585`, `2908~2924` | 폰트명 한컴 변환기 mimic 매핑 |

**2 파일 한정** 수정. 다른 parser (HWP5/HWPX) / renderer / model 무수정.

---

## 7. 한계 + 권고

### 7.1 격차 C 공백 정합 (cosmetic)

rhwp 출력 "1.추진목적" vs 한컴 출력 "1. 추진목적" — period 뒤 공백 차이. HWP3 raw 자체에는 공백 부재, 한컴이 자동 삽입. 현 task 범위에서는 over-aggressive risk 로 미도입.

### 7.2 휴리스틱 한계 (격차 C)

`fixup_hwp3_heading_decoration` 은 HWP3 spec 미참조 패턴 detection — 의도된 `═══...■...■═══` typography (표지 디자인 등) 회귀 risk 존재. 현재 25 fixture sweep 회귀 0 — 발견 시 매칭 기준 좁히거나 disable.

### 7.3 폰트 매핑 한계 (격차 D)

`hwp3_font_name_to_hwp5` 는 5 legacy 명칭 (신명조/고딕/중고딕/견고딕/그래픽) 만 매핑. 다른 HWP3 legacy 명칭 (휴먼명조, 견고딕, 바탕 등) 은 그대로 유지. 추가 매핑 필요 사례 발견 시 확장 가능.

---

## 8. 커밋 history

| 커밋 | 단계 |
|------|------|
| (Stage 1) | 4 격차 종합 진단 + 수행/구현계획서 v2 + Stage 1 보고서 |
| (Stage 2) | 격차 A — drawing.rs gradient IR 매핑 + 단위 테스트 |
| (Stage 3) | 격차 C — fixup_hwp3_heading_decoration + 단위 테스트 |
| (Stage 4) | 격차 B — LineType 2~7 → Solid normalize + 단위 테스트 |
| (Stage 5) | 격차 D-1 — CharShape pos dedupe |
| (Stage 5 v2) | 격차 D-2 — 폰트명 한컴 변환기 mimic 매핑 + 단위 테스트 |
| (Stage 6) | 최종 결과 보고서 + orders 갱신 |

closes #1008

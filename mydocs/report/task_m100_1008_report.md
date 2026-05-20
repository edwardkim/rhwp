# Task #1008 최종 결과 보고서 — 격차 A + C 일부 완료

**Issue**: [#1008 HWP3 sample16 Shape/Text 정합 격차 종합](https://github.com/edwardkim/regression-rhwp/issues/1008)
**Branch**: `local/task1008`
**Milestone**: v1.0.0

---

## 1. 결과 요약

본 task 는 issue #1008 의 4 격차 (A: gradient / B: border / C: HEAD numbering / D: 공백) 중 **격차 A + C 를 완료**하고 격차 B + D 는 후속 작업으로 남김. 작업지시자 시각 검증 통과 — HWP3 sample16 cover (RFP 박스) + 사업개요 p2 (1.추진목적 박스) 한컴 한글 정답지 정합.

**변경 범위**: `src/parser/hwp3/drawing.rs` + `src/parser/hwp3/mod.rs` 2 파일.

---

## 2. 본문 가설 정정

원본 issue #1008 본문의 가설 ("HWP5 변환본 gradient 를 한컴이 strip — variant 가드로 simplify") 은 한컴 한글 정답지 시각 검증 결과 **정반대** 임이 확인되어 issue body + 수행/구현계획서를 v2 로 재작성. 정답은:

- 한컴 viewer 정답: gradient 있음 (보라/라벤더 fill)
- rhwp HWP5 변환본: gradient 정상 (참고)
- **rhwp HWP3 native: gradient 누락 + decoration text 미정리 + 공백 정합 등 4 격차**

---

## 3. 완료 격차 (A + C)

### 3.1 격차 A — HWP3 Shape 박스 배경 gradient IR 매핑

**Root cause**: `src/parser/hwp3/drawing.rs:149~170` 의 `Hwp3DrawingObjectGradientAttr` 는 이미 파싱되었으나 (`basic_attr.has_gradient()` 시 read), `drawing.rs:792~806` 의 최종 Fill IR 구축에서 `fill_type=Solid, gradient=None` 으로 하드코딩되어 데이터가 무시됨.

**Fix**: HWP5 매핑 contract (`doc_info.rs:404`) 와 동일하게 IR 주입:
- `kind → gradient_type`
- `step → blur`
- `start_color + end_color → colors: vec![start, end]` (2-stop)
- `positions: vec![]` → renderer (`utils.rs:167`) 가 균등 분포

**단언**:
- HWP3 pi=71 (사업개요 박스): 채우기 Solid → Gradient ✓
- HWP3 pi=5 (cover RFP 박스): 채우기 Solid → Gradient ✓
- SVG: linearGradient 0 → 2 ✓
- HWP5 변환본 무변동 ✓

### 3.2 격차 C — HWP3 heading decoration 휴리스틱 strip

**Root cause**: HWP3 raw paragraph 가 "════...■ NUM.title ■════..." 형태 decoration text 를 plain text 로 저장 (sample16 pi=70: 52 chars). 한컴 변환기 HWP3→HWP5 와 한컴 한글 viewer 모두 decoration 을 strip 하는 것으로 추정 (HWP3 spec 미명문화).

**Fix**: `fixup_hwp3_heading_decoration` 신규 — `parse_hwp3()` 종단에 추가:
- 선행/후행 `═` 5개 이상 (보수적 매칭)
- 양끝 trim 후 `■` 로 둘러싸인 substring 존재
- 비매치 시 원본 유지 (no-op)

**단언**:
- pi=70 text "════...■ 1.추진목적 ■════..." (52자) → "1.추진목적" (5자) ✓
- pi=73 text "2. 추진방향￼" (decoration 없음) → 무변동 ✓ (패턴 비매치 정상)
- SVG `>═<` / `>■<` count: 다수 → 0 ✓

---

## 4. 후속 작업으로 남긴 격차 (B + D)

### 4.1 격차 B — Shape border 실선/점선

- HWP3 raw `style=0x0002`, HWP5 `style=0xc0010043` 둘 다 LineType 2/3 으로 점선 해석
- 한컴 viewer 는 둘 다 실선
- HWP3 line_style binary 값 의미 추가 조사 + LineType 비트 해석 정합 필요
- Fix 위치: parser 또는 renderer 미정
- 회귀 risk: 다른 fixture (시험지/aift) 의 점선 box 영향 가능

### 4.2 격차 D — HWP3 한글 공백 시각 누락

- parser 데이터 정상 (text 에 공백 포함)
- visual rendering 단계 (renderer text run / char_position 계산) 의 결함
- Fix 위치: `src/renderer/` 미정

→ 본 PR merge 후 별도 stage 또는 새 issue 로 진행.

---

## 5. 검증

### 5.1 자동 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --check` | ✓ clean |
| `cargo test --release --lib` | ✓ 1307 passed; 0 failed |
| `cargo test --release --test issue_1008_gradient` | ✓ 2 passed (격차 A + C) |
| `cargo test --release --tests` | ✓ FAILED 0 (전체 integration) |

### 5.2 단위 테스트 추가

`tests/issue_1008_gradient.rs`:
- `hwp3_sample16_business_box_has_gradient` — 격차 A 회귀 가드
- `hwp3_sample16_heading_decoration_stripped` — 격차 C 회귀 가드

### 5.3 페이지 수 sweep (HWP3 11 종 + HWP5/HWPX 변환본 + 일반 fixture)

모든 fixture 페이지 수 회귀 0.

### 5.4 시각 판정

작업지시자 한컴 한글 정답지 비교 시각 검증 통과 — HWP3 cover (RFP 박스 gradient) + 사업개요 p2 (1.추진목적 heading + gradient 박스) 한컴 정답 정합 확인.

---

## 6. 성공 기준 충족

| 조건 | 기준 | 결과 |
|------|------|------|
| C1: HWP3 박스 gradient 한컴 정합 | 보라/라벤더 gradient | ✓ |
| C2: border 실선 (격차 B) | solid | **(후속)** |
| C3: HEAD numbering "1.추진목적" 형식 | decoration strip | ✓ |
| C4: HWP3 한글 공백 (격차 D) | 시각 정합 | **(후속)** |
| C5: 페이지 수 64 유지 | 무변동 | ✓ |
| C6: 변환본/일반 fixture 회귀 0 | 페이지+시각 | ✓ |
| C7: cargo test 1307+ passed | clean | ✓ |
| C8: 작업지시자 시각 검증 | 한컴 정답 정합 | ✓ |

---

## 7. 커밋 history

| 커밋 | 단계 |
|------|------|
| (Stage 1) | 4 격차 종합 진단 + 수행/구현계획서 v2 + Stage 1 보고서 |
| (Stage 2) | 격차 A — drawing.rs Fill IR gradient 매핑 + 단위 테스트 |
| (Stage 3) | 격차 C — mod.rs fixup_hwp3_heading_decoration 휴리스틱 strip + 단위 테스트 |
| (Stage 6) | 최종 보고서 + orders 갱신 |

---

## 8. 한계 + 권고

### 8.1 격차 C 공백 정합 차이

- rhwp 출력: "1.추진목적" (HWP3 raw 보존)
- 한컴 출력: "1. 추진목적" (period 뒤 공백)
- HWP3 raw 자체에 공백 부재 — 한컴이 자동 삽입하는 것으로 추정
- 본 task 에서는 over-aggressive risk 로 미도입. 후속 task 또는 사용자 결정 시 추가 가능 (`(\d+)\.([^\s])` → `\1. \2`)

### 8.2 휴리스틱 한계

`fixup_hwp3_heading_decoration` 은 HWP3 spec 미참조 패턴 detection — 의도된 `═══...■...■═══` typography (표지 디자인 등) 회귀 risk 존재. 현재 sweep 회귀 0 — 발견 시 매칭 기준 좁히거나 disable.

### 8.3 후속 task 권고

- 격차 B + D 는 본 PR 외 별도 진행
- 격차 D 는 renderer 영역 — parser 격리 규칙 외
- 격차 B 는 다른 fixture (시험지/aift) 점선 영향 검증 필수

---

issue #1008 stays OPEN — 격차 B + D 후속 진행 예정. 본 PR 은 격차 A + C 부분 완료.

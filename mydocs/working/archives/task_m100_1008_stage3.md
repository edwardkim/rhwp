# Task #1008 Stage 3 완료 보고서 — 격차 C fix (HWP3 heading decoration strip)

**Issue**: [#1008 HWP3 sample16 Shape/Text 정합 격차 종합](https://github.com/edwardkim/regression-rhwp/issues/1008)
**Branch**: `local/task1008`
**작업 내용**: HWP3 paragraph 의 "═══■ NUM.title ■═══" decoration text 휴리스틱 strip

---

## 1. 진단 발견

HWP3 raw pi=70 의 텍스트:
```
"════════════════════■ 1.추진목적 ■════════════════════..."  (52 chars)
```

HWP5 변환본 pi=70 의 텍스트 (한컴 변환기가 strip 한 결과):
```
"1. 추진목적"  (7 chars)
```

한컴 한글 viewer 의 HWP3 rendering: "1. 추진목적" plain header (작업지시자 시각 정답 단언). 한컴 viewer 가 동일 decoration 패턴을 strip 하는 것으로 추정 (HWP3 spec 미명문화).

---

## 2. Fix — 휴리스틱 패턴 detection

`src/parser/hwp3/mod.rs` 의 `parse_hwp3()` 종단에 새 fixup 함수 추가:

```rust
fn fixup_hwp3_heading_decoration(doc: &mut Document) {
    for section in &mut doc.sections {
        for paragraph in &mut section.paragraphs {
            if let Some(cleaned) = strip_heading_decoration(&paragraph.text) {
                paragraph.text = cleaned;
            }
        }
    }
}

fn strip_heading_decoration(text: &str) -> Option<String> {
    // 1. 텍스트가 `═{5+}` 로 시작 + `═{5+}` 로 종료
    // 2. 중간 (trim 후) 이 `■` 로 시작 + `■` 로 종료
    // 3. 두 `■` 사이의 텍스트 (trim 후) 가 실제 heading
    // 비매치 시 None → 원본 유지
}
```

### 2.1 보수적 매칭 기준 (회귀 risk 완화)

- 선행 ═ 5개 이상 (산발적 사용 사례 회피)
- 후행 ═ 5개 이상 (대칭 확인)
- 양끝이 ■ 로 둘러싸인 substring 존재
- 비매치 시 원본 유지 (no-op)

---

## 3. dump 단언

### 3.1 sample16 pi=70 (사업개요 1.추진목적 heading)

```
BEFORE: "════...■ 1.추진목적 ■════..."  (52 chars)
AFTER:  "1.추진목적"                   (5 chars)
```

### 3.2 sample16 pi=73 (2.추진방향 — decoration 없는 heading, 패턴 비매치)

```
BEFORE/AFTER 동일: "2. 추진방향￼"   (변경 없음)
```

---

## 4. SVG 단언

`output/poc/pr1008/after/hwp3-sample16_003.svg` (사업개요 p2):

| 항목 | BEFORE | AFTER |
|------|--------|-------|
| `>═<` count | 다수 | **0** |
| `>■<` count | 다수 | **0** |
| 추진목적 chars at y=142 | (decoration 사이) | (단독, 6자) ✓ |

→ heading "1.추진목적" 만 page 상단에 단독 렌더.

---

## 5. 회귀 sweep

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --check` | ✓ clean |
| `cargo test --release --lib` | ✓ 1307 passed |
| `cargo test --release --test issue_1008_gradient` | ✓ 2 passed (격차 A + C) |

### 5.1 페이지 수 sweep (HWP3 9 종 + 변환본)

| Sample | 페이지 |
|--------|--------|
| hwp3-sample10 | 763 ✓ |
| hwp3-sample11 | 151 ✓ |
| hwp3-sample13 | 3 ✓ |
| hwp3-sample14 | 11 ✓ |
| hwp3-sample16 | 64 ✓ |
| hwp3-sample19 | 2 ✓ |
| hwp3-sample4 | 36 ✓ |
| hwp3-sample5 | 64 ✓ |

→ 모든 HWP3 sample 페이지 수 회귀 0 (휴리스틱 over-aggressive strip 영향 없음).

---

## 6. 단위 테스트 추가

`tests/issue_1008_gradient.rs::hwp3_sample16_heading_decoration_stripped`:
- HWP3 sample16 pi=70 의 text 에 ═ / ■ 잔존 0 단언
- "추진목적" core text 보존 단언

---

## 7. 한계 (개선 권고)

### 7.1 공백 정합 차이

```
rhwp 출력: "1.추진목적"
한컴 출력: "1. 추진목적" (period 뒤 공백)
```

HWP3 raw 에는 `1.추진목적` 으로 저장 — 한컴 변환기/viewer 가 자동 공백 삽입하는 것으로 추정. 본 task 에서 추가 휴리스틱 도입은 over-aggressive risk — 현 상태 보존.

후속 task 또는 사용자 결정 시 추가 가능: `(\d+)\.([^\s])` → `\1. \2`.

### 7.2 휴리스틱 한계

HWP3 spec 미참조 패턴 detection 으로 다음 risk 존재:
- 의도된 `═══...■...■═══` typography (예: 표지 디자인)
- 변형된 decoration 패턴 (단일 ■, 다른 line 문자 등)

회귀 발견 시 매칭 기준 좁히거나 disable.

---

## 8. 성공 기준 충족

| 조건 | 결과 |
|------|------|
| C3: HWP3 HEAD numbering "1.추진목적" 형식 | ✓ (decoration strip 성공) |
| C5: 페이지 수 64 유지 | ✓ |
| C6: 변환본 + HWP3 sample 회귀 0 | ✓ |
| C7: cargo test 1307+ passed | ✓ |
| C8: 작업지시자 시각 검증 | (PR/Stage 6 시점) |

---

## 9. 다음 단계 (Stage 4)

격차 B — Shape border 실선/점선. HWP3 raw `style=0x0002`, HWP5 `style=0xc0010043` 둘 다 LineType 2/3 으로 점선 해석되나 한컴 viewer 는 실선. parser 또는 renderer LineType 비트 해석 정합.

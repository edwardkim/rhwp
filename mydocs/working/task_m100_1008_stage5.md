# Task #1008 Stage 5 완료 보고서 — 격차 D 부분 fix (HWP3 CharShape dedupe)

**Issue**: [#1008 HWP3 sample16 Shape/Text 정합 격차 종합](https://github.com/edwardkim/regression-rhwp/issues/1008)
**Branch**: `local/task1008`
**작업 내용**: HWP3 parser 의 같은 pos 중복 CharShape dedupe — root cause 부분 해소

---

## 1. Root cause 추적

진단 test (`diag_1008_charshape`) 로 pi=4 의 char_shapes 단언:

```
HWP3 native pi=4 (BEFORE fix):
  pos=0 id=57 base_size=1000 bold=false ← rep CharShape (10pt)
  pos=0 id=58 base_size=1400 bold=true  ← inline shape change at pos=0 (14pt)
  pos=8 id=59 base_size=1400 bold=false ← `"세계 3대...기업"`
  pos=31 id=60 base_size=1400 bold=true

HWP5 변환본 pi=4:
  pos=0 id=21 base_size=1400 bold=true  ← 단일 (한컴 변환기 dedupe)
  pos=8 id=27 base_size=1400 bold=false
  pos=31 id=28 base_size=1400 bold=true
```

→ HWP3 raw 의 **rep CharShape (id=57, 10pt) + inline shape change (id=58, 14pt) 양쪽 모두 pos=0 으로 push** 됨. 한컴 변환기는 변환 시 dedupe 하여 inline override 만 유지.

---

## 2. Fix — CharShape dedupe in HWP3 parser

`src/parser/hwp3/mod.rs:1869~1900` 의 char_shapes 빌드 후 dedupe 추가:

```rust
// 같은 start_pos 에 여러 CharShape 가 push 된 경우 마지막 (inline) 만 유지
let mut deduped: Vec<CharShapeRef> = Vec::with_capacity(char_shapes.len());
for cs in char_shapes {
    if let Some(last) = deduped.last_mut() {
        if last.start_pos == cs.start_pos {
            *last = cs;
            continue;
        }
    }
    deduped.push(cs);
}
para.char_shapes = deduped;
```

---

## 3. dump 단언 (AFTER)

```
HWP3 sample16 pi=4 (AFTER fix):
  [CS] pos=0 id=58 bold=true spacing=0% char=" "   ← id=57 제거됨
  [CS] pos=8 id=59 bold=false spacing=-12% char="\""
  [CS] pos=31 id=60 bold=true spacing=0% char=""
```

HWP5 변환본 구조와 동일 (3개 CharShape).

---

## 4. SVG 좌표 단언 (BEFORE/AFTER 격차 D fix)

```
BEFORE:
  HWP3 cover caption: 첫 `"` x=131.69 → 세 x=134.69 (advance=3.0)
  HWP5 변환본 동일:    `"` x=131.69 → 세 x=137.69 (advance=6.0)

AFTER (dedupe만):
  HWP3: 좌표 변동 없음 — id=57 (10pt) 가 chars 0-7 (leading spaces) 에만 영향
        leading spaces 는 invisible glyph 이므로 visual diff 없음
```

→ **dedupe fix 는 데이터 무결성 향상이나 visual drift 직접 해소 안 됨**. char-by-char advance width 차이 (HWP3 vs HWP5) 가 별도 root cause.

---

## 5. 격차 D 잔존 — Visual drift root cause 미해소

dedupe 후에도 HWP3 vs HWP5 변환본의 char 좌표 차이 (3-7px 누적 drift) 잔존. 추가 root cause 후보:

1. **font_ids 차이**: HWP3 id=58/59/60 의 `font_ids[0]=1`, HWP5 id=21/27/28 의 `font_ids[0]=4` — 다른 font index. 같은 텍스트의 advance width 가 폰트별로 다름
2. **font_faces 매핑**: HWP3 group 0 idx 1 = "신명조", HWP5 group ? — 폰트 metric 차이 가능
3. **renderer 의 char advance 계산**: 폰트 metric 의존 — 본 task 범위 외

본 격차 D 의 visual drift root cause 는 **HWP3 polyfont mapping + renderer advance metric 처리** 영역으로 단언, 본 task 범위에서 완전 해소 불가.

→ 잔존 격차 D 는 본 PR 후속 별도 issue 로 추적 권고.

---

## 6. 회귀 sweep

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --check` | ✓ clean |
| `cargo test --release --lib` | ✓ 1307 passed |
| `cargo test --release --test issue_1008_gradient` | ✓ 3 passed (격차 A + B + C) |

### 6.1 페이지 수 sweep (전체 HWP3 + 변환본 + 일반 fixture)

| Sample | 페이지 |
|--------|--------|
| hwp3-sample (-hwp5) | 16 / 16 ✓ |
| hwp3-sample10 (-hwp5) | 763 / 763 ✓ |
| hwp3-sample11 (-hwp5) | 151 / 151 ✓ |
| hwp3-sample13 (-hwp5) | 3 / 3 ✓ |
| hwp3-sample14 (-hwp5) | 11 / 11 ✓ |
| hwp3-sample16 (-hwp5) | 64 / 64 ✓ |
| hwp3-sample19 (-hwp5) | 2 / 2 ✓ |
| hwp3-sample4 (-hwp5) | 36 / 36 ✓ |
| hwp3-sample5 + 4 variants | 64 (모두) ✓ |
| exam_kor / eng / math | 20 / 8 / 20 ✓ |
| aift / biz_plan | 74 / 6 ✓ |

→ 모든 fixture 페이지 수 회귀 0.

---

## 7. 성공 기준 충족

| 조건 | 결과 |
|------|------|
| C4: HWP3 한글 단어 공백 정합 | **부분** — dedupe 적용, visual drift 잔존 |
| C5: 페이지 수 64 유지 | ✓ |
| C6: 변환본/일반 fixture 회귀 0 | ✓ |
| C7: cargo test | ✓ |
| C8: 시각 검증 | (Stage 6 시점) |

---

## 8. 다음 단계 (Stage 6)

최종 보고서 갱신 + orders 갱신 + PR 생성 (작업지시자 승인 후). 격차 D 잔존 visual drift 는 별도 issue 로 후속 추적 권고.

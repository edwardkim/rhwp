# Task #896 Stage 1+2 통합 보고서 — 차이 1 진단 정정 + Fix

**Stage**: 1+2 / 5 (차이 1: paragraph 글머리 fixup 휴리스틱)
**상태**: ✅ 완료

## 1. 진단 정정 (이전 Task #894 Stage 4 의 진단 오류)

### 1.1 SVG x 좌표 직접 측정

paragraph 396~399 의 ◦ char SVG x 좌표:

| y | x | char | paragraph |
|---|---|------|-----------|
| 1220 | 96.69 | ○ | 395 (1단계) |
| **1249** | **107.30** | **◦** | **396** |
| **1276** | **107.30** | **◦** | **397** |
| **1303** | **107.30** | **◦** | **398 (이전)** |
| **1330** | **107.30** | **◦** | **399 (이전)** |

→ **paragraph 396/397/398/399 의 ◦ 모두 동일 x=107.30**. 이전 진단 ("paragraph 396 ○ x=117.81 vs paragraph 397~399 ◦ x=107.30") 의 paragraph 396 x 가 잘못된 비교 (paragraph 396 의 ◦ 가 아닌 다른 paragraph 의 ○).

### 1.2 진짜 root cause — paragraph 398/399 의 ◦ 자체가 잘못 추가됨

PDF (한컴 viewer) 페이지 18 의 paragraph 398 영역 분석 (pdftohtml -xml):

```xml
<text top="1009" left="159" width="6" height="30" font="3">-</text>
<text top="1014" left="174" width="468" height="19" font="5">하드웨어 및 소프트웨어 장애에 대비한 클러스터링 구성</text>
```

paragraph 398 의 첫 char 가 **`-` (left=159)** — **한컴 viewer 에서 ◦ 표시되지 않음**. sub-item dash marker 로 처리.

다른 paragraph 의 ◦ (paragraph 396/397/400) PDF 추출:
```xml
<text top="944" left="132" width="19" height="30" font="4">◦</text>
<text top="976" left="132" width="19" height="30" font="4">◦</text>
<text top="1122" left="133" width="19" height="30" font="4">◦</text>
```

paragraph 398 (top≈1014), paragraph 399 (top≈1041) 영역에 ◦ char element **없음**.

### 1.3 ROOT CAUSE 함수 추적

`src/parser/hwp3/mod.rs:2502 apply_bullet_fixup_single` (Task #877 Stage 4):

```rust
if !para.text.starts_with(' ') { return; }
let second = para.text.chars().nth(1).unwrap_or(' ');
if second == '◦' || second == '○' { return; }
```

skip 조건이 두번째 char 가 ◦/○ 인 경우만. **`-` (sub-item dash) 시작 paragraph 의 skip 누락**.

paragraph 398 raw text: `"   - 하드웨어..."` (공백+공백+공백+dash). fixup 진행 → ` ◦ ` prefix 추가 → `"  ◦    - 하드웨어..."` (◦ 잘못 추가).

대조: `apply_textbox_bullet_fixup` (line 2467, nested text_box) 의 skip:
```rust
if second == '-' { return; }
```

main fixup 이 textbox fixup 의 dash skip 정책 누락.

## 2. Fix

`src/parser/hwp3/mod.rs:2526` 의 `apply_bullet_fixup_single` 에 dash skip 조건 추가:

```rust
// 첫 non-space char 가 '-' (sub-item dash) 면 skip.
let first_non_space = para.text.chars().find(|c| *c != ' ').unwrap_or(' ');
if first_non_space == '-' { return; }
```

`apply_textbox_bullet_fixup` 의 동일 정책 적용. sub-item marker 가 이미 dash 로 표시되므로 ◦ 추가 안 함 — 한컴 변환기 정합.

## 3. 검증

### 3.1 paragraph text dump (fix 후)

```
paragraph 397: " ◦  공사 주요업무에 대한 클러스터링(Active - Active) 체계 구축"  (유지)
paragraph 398: "    - 하드웨어 및 소프트웨어 장애에 대비한 클러스터링 구성"  (◦ 제거 ✅)
paragraph 399: "    - ORACLE RDBMS의 DB 클러스터링 구성"  (◦ 제거 ✅)
```

### 3.2 SVG (fix 후)

paragraph 398/399 의 첫 char `-` 가 x=129.15 시작 (이전: ◦ x=107.30 + - x=152.37 → ◦ 제거로 dash 만).

### 3.3 회귀 점검

- `cargo test --release --lib`: **1250 passed**, 0 failed (이전 1234 + Task #894 의 신규 4 + 본 task 영향 측정 차이)
- HWP3 sample 6종 페이지 수 회귀 없음
- HWPX sample 회귀 없음
- HWP3 sample10 (해당 ◦ 휴리스틱 적용 sample) 회귀 없음 (페이지 수 763 유지)

## 4. 커밋

- `(다음 commit)` — Task #896 Stage 1+2: apply_bullet_fixup_single 의 first_non_space == '-' skip 추가

## 5. Stage 산출물

- 본 보고서: `mydocs/working/task_m100_896_stage1.md`
- Fix: `src/parser/hwp3/mod.rs` (+8 lines)

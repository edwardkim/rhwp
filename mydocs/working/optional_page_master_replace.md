---
kind: working
status: active
issue: 6323
---

# OPTIONAL_PAGE 바탕쪽이 pageDuplicate="0" 을 무시한다 (#6323)

## 증상

`samples/hwpx/exam_kor.hwpx` 20쪽에서 쪽번호 자리에 숫자 두 개가 겹쳐 찍혀 읽을 수 없다.
한 쪽에 바탕쪽이 둘 그려지고 각자의 쪽번호를 같은 좌표에 쓴다.

```
MasterPage(EVEN)          x=119.1 y=142.4  '2'
MasterPage(OPTIONAL_PAGE) x=119.1 y=142.4  '4'
```

머리말 `(언어와 매체)` 와 `홀수형` 도 같은 좌표에 두 번 그려져 획이 겹친다.

samples 전수에서 바탕쪽끼리 겹치는 문서는 3 종이다(`hwpx/exam_kor.hwpx`,
`exam-kor-3p.hwp`, `exam_science.hwp`).

## 원인 — 문서가 답을 갖고 있었다

HWPX 원본의 바탕쪽 선언을 읽으면 의도가 분명하다.

```xml
<masterPage id="masterpage6" type="EVEN"          pageNumber="0" pageDuplicate="0"/>  <!-- '2' -->
<masterPage id="masterpage7" type="ODD"           pageNumber="0" pageDuplicate="0"/>  <!-- '3' -->
<masterPage id="masterpage8" type="OPTIONAL_PAGE" pageNumber="4" pageDuplicate="0"/>  <!-- '4' -->
```

`pageDuplicate="0"` 은 **"겹치게 하기 끔"** 이라는 문서의 명시적 선언이다. 그런데 파서가
이 선언을 `LAST_PAGE` 에만 반영했다.

```rust
// src/parser/hwpx/section.rs (수정 전)
if is_last_page {
    master_page.replace_base = page_duplicate == Some(false);
    master_page.overlap = true;
}
if is_optional_page {
    master_page.overlap = true;          // replace_base 를 세우지 않는다
}
```

`replace_base` 가 false 로 남으면 렌더 선택에서 이렇게 갈린다.

```rust
// src/document_core/queries/rendering.rs
let overlap_exts = ext_mp_indices.iter().filter(|&&i| mps[i].overlap && !mps[i].replace_base);
for &i in &overlap_exts {
    if Some(mps[i].apply_to) == active_apply { /* 기본 바탕쪽을 교체 */ }
    else { remaining_overlap_exts.push(i); }   // 추가로 렌더
}
page.extra_master_pages = remaining_overlap_exts...;
```

OPTIONAL_PAGE 의 `apply_to` 는 `Both`(파서가 확장 바탕쪽을 그렇게 놓는다)이고 활성
바탕쪽은 `Even` 이라, 서로 달라서 `extra_master_pages` 로 들어가 **덧그려진다.**

## 수정 — 파서와 직렬화기 양쪽

### 1. 파서

두 확장 바탕쪽 종류가 같은 저장 계약을 따르므로 분기를 합친다.

```rust
if is_last_page || is_optional_page {
    master_page.replace_base = page_duplicate == Some(false);
    master_page.overlap = true;
}
```

`LAST_PAGE` 의 동작은 그대로다(`page_duplicate` 가 `Some(false)` 일 때만 `replace_base`).
`pageDuplicate="1"` 로 진짜 겹치기를 선언한 문서는 종전처럼 덧그려진다.

### 2. 직렬화기 — 파서만 고치면 왕복이 깨진다

`serializer/hwpx/master_page.rs::page_duplicate_str` 는 스스로 "parser 의 역" 이라고
선언하는데, 첫 가지가 `ext_flags & 0x04 == 0`(LAST_PAGE 전용)이었다.

```rust
// 수정 전
if mp.is_extension && mp.ext_flags & 0x04 == 0 {   // LAST_PAGE 만
    if mp.replace_base { "0" } else { "1" }
} else if mp.overlap { "1" } else { "0" }
```

파서는 확장 바탕쪽의 `overlap` 을 `pageDuplicate` 와 무관하게 **항상 true** 로 세운다.
그래서 OPTIONAL_PAGE 는 `else if mp.overlap` 으로 떨어져 `pageDuplicate="1"` 로 저장되고,
재파싱하면 `replace_base` 가 false 로 뒤집혀 **원래 결함으로 되돌아간다.**

수정 전에는 파서가 OPTIONAL_PAGE 의 `pageDuplicate` 를 아예 읽지 않았으므로 이 비대칭이
드러나지 않았다. **파서를 고치는 순간 나타나는 잠복 결함**이다.

```rust
// 수정 후 — 파서의 `is_last_page || is_optional_page` 와 같은 경계
if mp.is_extension {
    if mp.replace_base { "0" } else { "1" }
} else if mp.overlap { "1" } else { "0" }
```

이 비대칭은 `tests/visual_roundtrip_baseline.rs`(parse→serialize→reparse 자기 정합성)가
`exam_kor.hwpx: 구조 불일치 1페이지` 로 잡았다. 그 게이트는 `VISUAL_XFAIL` 이 0 건인
깨끗한 상태라, 등록해 통과시키는 대신 원인을 고쳤다.

### 3. 옛 계약을 못박고 있던 단위 시험 2 개

`replace_base` 를 단언하는 시험이 파서 쪽에 넷 있었는데 둘이 서로 모순이었다.

| 시험 | 픽스처 | 종전 단언 |
| --- | --- | --- |
| `test_parse_master_page_last_page_extension` | `LAST_PAGE pageDuplicate="0"` | `replace_base` |
| `test_parse_master_page_optional_page_extension` | `OPTIONAL_PAGE pageNumber="4" pageDuplicate="0"` | `!replace_base` |
| `test_parse_master_page_mixed_case_type_attrs` (LastPage) | 〃 | `replace_base` |
| `test_parse_master_page_mixed_case_type_attrs` (optionalPage) | 〃 | `!replace_base` |

**같은 `pageDuplicate="0"` 선언에 정반대를 못박고 있었다.** 근거 주석은 없고 당시 동작을
스냅샷한 것이다. 특히 `test_parse_master_page_optional_page_extension` 의 픽스처는 실제
문서(`exam_kor.hwpx` 의 masterpage8)와 형태가 같아서, 결함 있는 형상을 그대로 고정하고
있었다. 두 단언을 대칭으로 갱신하고 왜 바뀌는지 주석을 남겼다. 시험 **개수는 그대로**라
`rust-unit-test-tiers --check --base-ref` 는 영향받지 않는다(4221 유지).

## 이미 있던 짝 — 왜 한쪽만 고쳐져 있었나

`LAST_PAGE` 에 대해서는 회귀 시험이 이미 있다.

```
src/renderer/layout/integration_tests.rs
  test_1098_hwpx_last_page_master_replaces_base_master
  "HWPX LAST_PAGE pageDuplicate=0은 기본 홀/짝 바탕쪽에 추가하지 않고 대체해야 함"
```

같은 계약의 `OPTIONAL_PAGE` 쪽이 비어 있었다. 이 작업이 그 짝을 채운다.

## 검증

### 실측 — 전/후

| | 바탕쪽 노드 수 | 그려진 쪽번호 |
| --- | ---: | --- |
| 수정 전 | 2 | `2`, `4` (같은 좌표에 포갬) |
| 수정 후 | 1 | `4` |

```
$ rhwp export-render-tree "samples/hwpx/exam_kor.hwpx" -p 19 -o out
MasterPage 노드 수: 1
  [0] ['4', '(언어와 매체)', '홀수형', '*', '확인 사항']
```

시각 증적: `mydocs/report/assets/issue_6323/before.png` · `after.png`
(같은 쪽을 `export-svg` 로 렌더해 상단을 잘랐다).

### 게이트

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | (아래 실측) |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | (아래 실측) |
| `issue_6323_optional_page_master_replaces_base` | (아래 실측) |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref <base>` | (아래 실측) |
| `node scripts/rust-test-suite-manifest.mjs --check --base-ref <base>` | (아래 실측) |

시험을 `tests/cases/` 에 둔 이유는 `src/**` 의 source-side 단위 시험을 PR base 대비
늘릴 수 없기 때문이다(CI `Validate Rust test suite manifest`, `--base-ref` 로만 걸린다).
판정을 렌더 트리로 하는 이유는 `pagination` 이 crate 내부 필드라 통합 시험에서 볼 수
없고, 무엇보다 사용자가 보는 것이 "그 쪽에 바탕쪽이 몇 겹 그려졌는가" 이기 때문이다.

## 실측

### 시험이 결함을 실제로 잡는가 — 수정을 되돌려 확인

통과만 보면 그 시험이 결함을 잡는지 알 수 없다. `src/parser/hwpx/section.rs` 만 stash 로
되돌리고 같은 시험을 다시 돌렸다.

```
test optional_page_master_does_not_stack_on_the_base_master ... FAILED
test page_number_is_not_drawn_twice ... FAILED
test result: FAILED. 0 passed; 2 failed

바탕쪽 쪽번호가 하나여야 한다. 실제로 그려진 숫자: ["2", "4"]
바탕쪽이 겹쳐 그려지면 쪽번호·머리말이 같은 좌표에 포개진다. 그려진 바탕쪽 2겹,
각 글자: [["2", "(언어와 매체)", "홀수형"],
          ["4", "(언어와 매체)", "홀수형", "*", "확인 사항", ...]]
```

실패 메시지가 결함 형상을 그대로 보여준다 — 같은 머리말 `(언어와 매체)`·`홀수형` 이
두 벌 그려지고 쪽번호가 `2`·`4` 두 개다. 수정을 되살리면 2 종 모두 통과한다.

### 게이트

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref f6a6bee8f3` | 통과 (4221, 증가 없음) |
| `node scripts/rust-test-suite-manifest.mjs --check --base-ref f6a6bee8f3` | 통과 |
| `issue_6323_optional_page_master_replaces_base` | 2 통과 |
| `cargo test --lib -- master_page hwpx::section hwpx::master_page` | **210 통과** |
| `visual_roundtrip_baseline::visual_baseline_all_samples` | 통과 (12.84s) |
| `overflow_cell_baseline::overflow_cell_lines_do_not_grow` | 통과 (CI 실측) |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | 통과 |

### 검증 방식에서 배운 것

첫 제출 때는 **새로 만든 통합 시험 이름으로만 좁혀** `cargo test` 를 돌렸다. 게이트 목록
(fmt·clippy·tiers·manifest)은 전부 통과했지만, 정작 **바꾼 파일 안의 기존 단위 시험과
왕복 원장이 실행되지 않아** CI 에서 두 건이 빨간불이 됐다.

`src/**` 를 바꾸면 그 모듈의 단위 시험 전체(`--lib -- <모듈>`)와 관련 원장을 함께 돌린다.
새 시험만 돌리는 것은 "내가 쓴 시험이 통과한다" 를 확인할 뿐 "기존 계약을 안 깼다" 를
확인하지 못한다.

### 커밋 대상

```
src/parser/hwpx/section.rs
tests/cases/issue_6323_optional_page_master_replaces_base.rs
mydocs/report/assets/issue_6323/before.png
mydocs/report/assets/issue_6323/after.png
mydocs/working/optional_page_master_replace.md
```

## 남는 것

samples 에서 바탕쪽끼리 겹치는 나머지 두 문서(`exam-kor-3p.hwp`, `exam_science.hwp`)가
같은 원인인지는 확인하지 않았다. 둘 다 HWP5 라 HWPX 파서를 타지 않으므로 별도 경로일
수 있다. #6322 의 글자 겹침 래칫이 다음 측정에서 해소 여부를 보여줄 것이다.

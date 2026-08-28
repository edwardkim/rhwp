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

## 수정

두 확장 바탕쪽 종류가 같은 저장 계약을 따르므로 분기를 합친다.

```rust
if is_last_page || is_optional_page {
    master_page.replace_base = page_duplicate == Some(false);
    master_page.overlap = true;
}
```

`LAST_PAGE` 의 동작은 그대로다(`page_duplicate` 가 `Some(false)` 일 때만 `replace_base`).
`pageDuplicate="1"` 로 진짜 겹치기를 선언한 문서는 종전처럼 덧그려진다.

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
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref f6a6bee8f3` | 통과 (4221) |
| `node scripts/rust-test-suite-manifest.mjs --check --base-ref f6a6bee8f3` | 통과 |
| `issue_6323_optional_page_master_replaces_base` | 2 통과 (0.17s) |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | 통과 |

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

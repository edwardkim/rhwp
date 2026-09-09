---
kind: working
status: active
issue: 6334
---

# HWP5 확장 바탕쪽이 기본 바탕쪽을 대체하지 않는다 (#6334)

## 증상

HWP5 문서에서 확장 바탕쪽(마지막 쪽·임의 쪽)이 기본 홀/짝 바탕쪽을 대체하지 않고 그
**위에 덧그려진다.** 두 바탕쪽의 쪽번호·머리말이 같은 자리에 포개진다.

`samples/exam_science.hwp` 4쪽(0 기준 3) 실측이다.

| | 바탕쪽이 그리는 글자 |
| --- | --- |
| 한컴 정답지 `pdf/exam_science-2022.pdf` p4 | `32 32`, `* 확인 사항`, `4 (화학 I) 과학탐구 영역` |
| rhwp `devel` | **2 겹** — `['31','32']` + `['32','32','* 확인 사항', …]` |

**정답지에 `31` 이 없다.** 기본 짝수 바탕쪽이 그려지면 안 되는데 그려지고 있다.
`layout-anomaly` 가 이 쪽에서 18.0 x 15.3px 글자 겹침을 잡는다.
`samples/exam-kor-3p.hwp` 3쪽도 같은 형상이다(겹침 15.8 x 27.3px).

## 원인

```rust
// src/parser/body_text.rs (수정 전)
let overlap = ext_flags & 0x01 != 0;
let is_extension = (ext_flags & 0x02 != 0) || /* 같은 apply_to 중복 휴리스틱 */;

master_pages.push(MasterPage {
    overlap,
    replace_base: false,      // <- 하드코딩
    ...
});
```

렌더 선택(`document_core/queries/rendering.rs`)은 확장 바탕쪽을 이렇게 가른다.

```rust
let replace_exts = ext_mp_indices.iter().filter(|&&i| !mps[i].overlap || mps[i].replace_base);
let overlap_exts = ext_mp_indices.iter().filter(|&&i| mps[i].overlap && !mps[i].replace_base);
```

HWP5 는 `overlap = true`(아래) · `replace_base = false` 이므로 **`replace_exts` 에 절대
들어가지 못하고** 항상 `extra_master_pages` 로 가서 덧그려진다.

## 판정 근거는 파일이 아니라 정답지다

HWPX 는 `pageDuplicate` 로 "겹치게 하기" 의도를 명시한다(#6323 / PR #6329 가 그 경로를
고쳤다). HWP5 에는 그 속성이 없고 `ext_flags` 의 overlap 비트만 있는데, 그 비트는 의도를
구분하지 못한다.

> 한컴 HWPX -> HWP5 저장본은 LAST_PAGE 바탕쪽을 확장 바탕쪽으로 저장하면서
> `pageDuplicate="0"` 인 경우에도 overlap bit 를 함께 세운다.
> — `parser/hwpx/section.rs` 의 같은 지점 주석

즉 **파일 안에는 판정 근거가 없다.** 그런데 한컴이 실제로 어떻게 그리는지는 저장소 안에
있다 — `pdf/` 의 정답지다. 정답지가 대체를 보이므로 대체로 맞춘다.

이슈에 세 선택지를 적고 방향을 여쭈었는데, 정답지로 1 번(확장 바탕쪽은 항상 대체)이
확인됐다. 우려했던 폭발 반경은 아래 실측으로 없음이 확인됐다.

## 수정

```rust
replace_base: is_extension,
```

## 검증

### 전/후

| 문서 | 쪽 | 수정 전 | 수정 후 |
| --- | ---: | --- | --- |
| `exam_science.hwp` | 4 | 2 겹 — `['31','32']` + `['32','32', …]` | **1 겹** — `['32','32','* 확인 사항', …]` |
| `exam-kor-3p.hwp` | 3 | 2 겹 | **1 겹** |

`exam_science` 는 정답지와 정확히 일치한다 — `31` 이 그려지지 않는다.

### 폭발 반경 — 세 원장 전수

`replace_base = is_extension` 은 확장 바탕쪽을 쓰는 **모든 HWP5 문서**에 닿으므로 전수로
쟀다.

| 원장 | 결과 |
| --- | --- |
| `overflow_cell_baseline` (945 문서 전 페이지 렌더) | 통과 (230.48s) |
| `visual_roundtrip_baseline` (parse -> serialize -> reparse) | 통과 (8.23s) |
| `cargo fmt --all -- --check` | 통과 |
| `rust-unit-test-tiers --check --base-ref f6a6bee8f3` | 통과 (4221) |
| `rust-test-suite-manifest --check --base-ref f6a6bee8f3` | 통과 |
| `issue_6334_hwp5_extension_master_replaces_base` | 2 통과 |

두 원장이 무증감이라는 것은 이 변경이 **다른 문서의 조판을 건드리지 않았다**는 뜻이다.
대체가 일어나는 것은 확장 바탕쪽이 실제로 있는 소수 문서뿐이다.

글자 겹침 전수도 쟀다. 이 브랜치에는 #6322 의 범위 확장(본문 밖 후보 수집)이 없어
`Body x Body` 축만 보이는데, **4,408 건으로 devel 기준선과 정확히 같다.** 즉 본문 안
겹침은 한 건도 움직이지 않았다. 바탕쪽끼리 겹침(현재 15 건)의 감소는 #6322 가 병합된 뒤
그 래칫에서 확인된다.

### 시험이 결함을 실제로 잡는가 — 수정을 되돌려 확인

`src/parser/body_text.rs` 만 stash 로 되돌리고 같은 시험을 다시 돌렸다.

```
test hwp5_extension_master_matches_hancom_oracle_text ... FAILED
test hwp5_extension_master_does_not_stack_on_base ... FAILED

기본 짝수 바탕쪽의 쪽번호 '31' 은 정답지에 없다 — 덧그리기로 되돌아갔다:
  ["31", "32", "32", "32", "*", "확인 사항", …]
바탕쪽이 겹쳐 그려지면 쪽번호·머리말이 같은 좌표에 포개진다. 그려진 바탕쪽 2겹:
  [["31", "32"], ["32", "32", "*", "확인 사항", …]]
```

실패 메시지가 결함 형상을 그대로 낸다 — `31` 이 다시 나타나고 `32` 가 세 번 그려진다.
확인 뒤 수정을 복원했다.

## 회귀 시험을 정답지 기준으로 썼다

`hwp5_extension_master_does_not_stack_on_base` 는 바탕쪽이 1 겹인지 보고,
`hwp5_extension_master_matches_hancom_oracle_text` 는 **`31` 이 그려지지 않는지**를 본다.
"겹이 하나" 만 보면 잘못된 한 겹이 남아도 통과하므로, 정답지에 없는 글자가 나타나면
실패하도록 했다.

## 남는 것

이 수정은 HWP5 파서 경로만 다룬다. 같은 형상의 HWPX 경로는 #6323 / PR #6329 다.
두 PR 은 서로 다른 파일을 만지므로 독립적으로 병합할 수 있다.

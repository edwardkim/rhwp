# Task #2835 처리 보고서 — TAC 그림/도형 배치 margin_left 이중 가산 수정

## 배경

이슈 #2835: 테두리(border) 있는 문단 안 treat-as-char(TAC) 그림/도형이 같은 문단
본문 텍스트보다 `para_margin_left` 만큼 더 오른쪽으로 밀려 렌더링되는 결함.

## 근본 원인

`src/renderer/layout.rs` 의 TAC picture/shape 배치 경로(`layout_shape_item`
계열, `effective_margin_left` 계산부)는 다음 조건에서 `para_margin_left` 를
"inner padding" 명목으로 한 번 더 가산하고 있었다.

```rust
let has_visible_stroke = /* border_fill_id 로 실 stroke 판정 */;
let inner_pad_left =
    if has_visible_stroke && bs_left_px == 0.0 && bs_right_px == 0.0 {
        para_margin_left
    } else {
        0.0
    };
let effective_margin_left = para_margin_left + para_indent + inner_pad_left; // (indent<=0 이면 indent 생략)
```

그러나 **같은 계산을 하던 `src/renderer/layout/paragraph_layout.rs`(본문 텍스트
배치 경로)는 커밋 `a30dca73`("Task #544 v2 Stage 2: Phase A 재적용")에서 정확히
이 `has_visible_stroke`/`bs_left_px`/`bs_right_px`/`inner_pad_left` 분기를
"이중 inset 부작용"으로 판단해 완전히 제거**했다. 그 커밋 메시지:

> `paragraph_layout.rs`: 1. inner_pad 분기 제거 (line 693~717 → 693~700, -22 LOC)
> — `has_visible_stroke` / `bs_left_px` / `bs_right_px` / `inner_pad_left`/`right`
> 변수 모두 제거 — `margin_left = box_margin_left` (단일 룰, 텍스트 inset 한 번만)

`git show a30dca73 -- src/renderer/layout.rs` 결과가 비어있어, 이 수정이
`layout.rs` 의 TAC picture/shape 경로에는 전혀 반영되지 않았음을 확인했다.
즉 본문 텍스트는 `margin_left` 만 적용되도록 이미 고쳐졌는데, 같은 문단의
TAC 그림/도형은 여전히 `margin_left` 를 두 번 적용받는 **형제(sibling)
코드 경로 간 비대칭**이 남아 있었다.

## 실측 재현

`inner_pad_left` 계산 직후에 디버그 출력을 임시로 추가해
`samples/exam_kor.hwp` 렌더링을 관찰한 결과, 최소 10개 문단
(para_index/pi = 46, 50, 54, 56, 60, 63, 66, 70, 108, 111 — 대부분 페이지
18(1-based)) 에서

```
inner_pad_left=11.33 para_margin_left=11.33
```

즉 `para_margin_left`(11.33px = ParaShape margin_left 1704 HU 환산치)가
매 문단마다 그대로 이중 가산되고 있음을 확인했다. 코드 주석에도 "exam_kor
p18 pi=50/56 의 [A]/[B] 표시기 옆 그림" 위치 결함으로 이미 알려진 증상이라고
적혀 있어 이번 조사와 일치한다.

## 수정

- `has_visible_stroke`/`bs_left_px`/`bs_right_px`/`inner_pad_left` 분기를
  완전히 제거.
- `paragraph_layout.rs` 와 동일한 "margin_left(+indent) 단일 가산" 규칙을
  `tac_picture_effective_margin_left(para_margin_left, para_indent)` 라는
  작은 순수 함수로 추출해, 두 형제 경로가 향후에도 같은 로직을 공유하도록
  구조화했다 (재발 방지).
- `border_styles`/`BorderLineType` 조회 등 이제 불필요해진 코드도 함께 제거.

## 검증 (Red → Green)

`tac_picture_effective_margin_left_matches_paragraph_layout_single_margin_rule`
(`src/renderer/layout/tests.rs`) 테스트를 추가했다.

- **RED**: 헬퍼 내부에 옛 버그(`inner_pad_left = para_margin_left` 를
  무조건 추가 가산)를 임시로 재현했을 때 테스트 실패 확인
  (`22.66 이 아니라 11.33 이어야 함` 단언 실패).
- **GREEN**: 수정된 헬퍼(단일 가산)로 되돌린 후 테스트 통과 확인.

```
test renderer::layout::tests::tac_picture_effective_margin_left_matches_paragraph_layout_single_margin_rule ... ok
```

- `cargo build --lib`: 통과, 경고 없음.
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 통과.
- `rustfmt --edition 2021` 적용 후 `git diff --name-only` 는 의도한 2개
  파일(`src/renderer/layout.rs`, `src/renderer/layout/tests.rs`)만 남음.

## 변경 파일

- `src/renderer/layout.rs`
- `src/renderer/layout/tests.rs`

## 참고

- 관련 커밋(선행 수정): `a30dca73` (Task #544 v2 Stage 2)
- 이슈: https://github.com/edwardkim/rhwp/issues/2835

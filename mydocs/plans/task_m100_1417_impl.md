# 구현 계획서 — Task M100-1417: TAC host 공백 문단 높이 중복 제거

- 이슈: https://github.com/edwardkim/rhwp/issues/1417
- 수행 계획서: `mydocs/plans/task_m100_1417.md`
- 작성일: 2026-06-16
- 브랜치: `local/task_m100_1417`

## 1. 진단 요약

재현 샘플 `samples/hwpx/pagenation-001.hwpx`에서 2페이지 하단의 `pi=27` TAC 이미지 묶음이
3페이지로 밀린다. debug SVG/render tree 기준으로 2페이지 `pi=26` 표의 실제 bbox 하단은 약
`927.7px`이고, body 하단은 약 `1028.1px`라서 약 `100px`의 여유가 있다. `pi=27` 이미지 묶음의
필요 높이는 약 `62.6px`이므로 실제 배치 모델 기준으로는 2페이지에 들어갈 수 있다.

`dump-pages`와 `RHWP_TYPESET_DRIFT=1 RHWP_TABLE_DRIFT=1 RHWP_VPOS_DEBUG=1` 로그를 대조하면 첫
드리프트는 `pi=16`에서 발생한다.

| 문단 | 특징 | 현재 pagination 효과 |
|---|---|---|
| `pi=16` | 텍스트가 공백 1자이고 TAC table 1개를 가진 host 문단 | `Table pi=16` 뒤에 `PartialParagraph pi=16`이 추가되어 table 높이와 host line advance가 중복 반영됨 |
| `pi=17` | 다음 일반 문단 | `pi=16` 이후 cursor가 약 `64px` 과다 전진한 상태로 시작 |
| `pi=26` | 2페이지 하단 TAC table | 실제 bbox 하단은 body 안쪽에 남음 |
| `pi=27` | TAC shape 이미지 묶음 | 과다 전진된 cursor 기준으로 fit 실패하여 3페이지로 이동 |

중요한 점은 HWPX `lineSegArray`를 authoritative 기준으로 삼지 않는다는 것이다. 이번 샘플에서는
lineSeg 9개 필드가 내부적으로 보존되는 것은 확인했지만, 수정 기준은 원본 lineSeg 값이 아니라
pagination cursor와 render placement의 내부 높이 모델 정합이다.

## 2. 원인

`src/renderer/typeset.rs`의 `place_table_with_text`가 post-table 텍스트 존재 여부를
`!para.text.is_empty()`로 판정한다.

대상 문단 `pi=16`은 텍스트가 `" "`인 공백-only 문단이다. 이 값은 비어 있지 않으므로 현재 로직은
실제 출력할 본문 텍스트가 없음에도 post-table `PartialParagraph`를 추가한다. 그 결과 TAC table이
차지한 높이에 더해 host line advance가 한 번 더 cursor에 누적된다.

현재 관련 코드:

- `src/renderer/typeset.rs:10063` `should_add_post_text`
- `src/renderer/typeset.rs:10087` `has_post_text`
- `src/renderer/typeset.rs:302` `para_has_visible_text`

전역 `para_has_visible_text`는 공백을 visible text로 취급하는 기존 의미가 여러 분기에 사용되고
있다. 따라서 이번 수정에서는 이 함수를 바꾸지 않고, TAC table post-text 판정에만 더 좁은
"실질 텍스트" 판정을 도입한다.

## 3. 수정 방안

### 3.1 좁은 범위 helper 추가

`src/renderer/typeset.rs`에 TAC host 후속 텍스트 판정용 helper를 추가한다.

예상 형태:

```rust
fn para_has_non_whitespace_text(para: &Paragraph) -> bool {
    para.text
        .chars()
        .any(|c| c > '\u{001F}' && c != '\u{FFFC}' && !c.is_whitespace())
}
```

이 helper는 표/그림 control 자체가 아니라, post-table `PartialParagraph`로 렌더할 실제 본문
문자가 있는지 판단하는 용도다.

### 3.2 post-table 텍스트 emission 조건 변경

`place_table_with_text`에서 post text 판정을 좁힌다. 회귀 위험을 줄이기 위해 모든 공백-only
문단을 제외하지 않고, `pi=16`처럼 다음 조건을 모두 만족하는 경우만 post text가 없는 것으로 본다.

- 텍스트가 비어 있지는 않지만 non-whitespace 문자가 없다.
- TAC table host 문단이다.
- pre-text가 없다.
- `FormattedParagraph` line 수가 1 이하라서 별도 post line이 없다.

판정 변경:

1. `should_add_post_text`
   - 현재: `!para.text.is_empty()`
   - 변경: `has_post_text`
   - 효과: 공백-only TAC table host 문단은 post `PartialParagraph`를 emit하지 않는다.

2. `has_post_text`
   - 현재: `!para.text.is_empty() && total_lines > post_table_start`
   - 변경: 단일 라인 공백-only TAC host 중복 emission 케이스만 제외
   - 효과: 문제 케이스에서는 table line 높이를 한 번만 소비하고, 필요한 trailing spacing은 기존
     정책대로 유지한다. 반면 공백/탭 등으로 구성된 별도 post line이 있는 문단은 기존 동작을 보존한다.

### 3.3 수정하지 않는 항목

- HWPX parser의 `lineSegArray` 로딩/보존 로직은 변경하지 않는다.
- `para_has_visible_text`의 전역 의미는 변경하지 않는다.
- `vpos_snap_current_height`나 `HeightCursor`를 lineSeg 기준으로 강제 보정하지 않는다.
- TAC shape 자체의 배치 로직은 이번 원인 경로가 아니므로 변경하지 않는다.

## 4. 테스트 계획

### 4.1 회귀 fixture

`samples/hwpx/pagenation-001.hwpx`를 fixture로 포함한다.

### 4.2 신규 테스트

신규 통합 테스트 파일 후보:

- `tests/issue_1417_pagination_cursor_render.rs`

검증 내용:

1. `HwpDocument::from_bytes`로 `samples/hwpx/pagenation-001.hwpx`를 로드한다.
2. `dump_page_items(Some(1))`에서 2페이지에 `Shape          pi=27`이 존재하는지 확인한다.
3. 2페이지에 `Table          pi=26`도 유지되는지 확인한다.
4. `PartialParagraph  pi=16`이 더 이상 중복 emission되지 않는지 확인한다.
5. 필요 시 전체 `page_count()`가 2페이지로 줄어드는 것을 보조 검증으로 둔다. 단, 후속 빈 페이지
   처리 정책이 별도로 남아 있을 수 있으므로 핵심 assertion은 `pi=27`의 2페이지 존재 여부로 둔다.

### 4.3 관련 회귀 테스트

아래 테스트를 함께 실행한다.

```bash
cargo fmt --check
cargo test --test issue_1417_pagination_cursor_render -- --nocapture
cargo test --test issue_1070_tac_table_post_text_overflow -- --nocapture
cargo test --test issue_1152_intra_para_vpos_reset -- --nocapture
cargo test --test issue_986 -- --nocapture
```

필요하면 최종 단계에서 범위를 넓혀 `cargo test --lib` 또는 관련 renderer 테스트를 추가 실행한다.

## 5. 시각 검증 계획

수정 후 다음 산출물을 생성한다.

```bash
target/debug/rhwp export-svg samples/hwpx/pagenation-001.hwpx -o output/poc/task1417-final --debug-overlay
target/debug/rhwp export-render-tree samples/hwpx/pagenation-001.hwpx -o output/poc/task1417-render-tree
target/debug/rhwp dump-pages samples/hwpx/pagenation-001.hwpx -p 1
```

작업지시자 시각 판정 대상:

- `output/poc/task1417-final/pagenation-001_002.svg`

기대 결과:

- 2페이지 하단에 `s0:pi=27` TAC 이미지 묶음이 함께 배치된다.
- 기존 2페이지의 `pi=16`, `pi=22`, `pi=26` TAC table 배치가 무너지지 않는다.
- 불필요한 3페이지가 생성되지 않거나, 최소한 `pi=27`이 3페이지로 밀리지 않는다.

## 6. 리스크

| 리스크 | 대응 |
|---|---|
| 공백 문단이 의도적으로 렌더 공간을 만드는 사례 회귀 | 변경 범위를 TAC table post-text 판정으로 제한하고, trailing spacing 복원은 유지 |
| 다른 TAC table post-text overflow 회귀 | `issue_1070_tac_table_post_text_overflow`를 필수 회귀 테스트로 실행 |
| lineSeg 기반 보정으로 오해 | 구현에서 lineSeg를 fit 판정 기준으로 사용하지 않음 |
| 전체 pagination cursor 정책 확장 | `place_table_with_text`의 공백-only host 처리에 한정 |

## 7. 승인 요청

위 계획대로 구현을 진행한다. 승인 후에만 `src/renderer/typeset.rs`와 신규 테스트/fixture를 수정한다.

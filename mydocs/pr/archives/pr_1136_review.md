# PR #1136 검토 보고서

- PR: `#1136`
- 제목: `fix(renderer/table): 표 셀 내 문단 자동번호 누락 정정 — 본문 path 와 정합`
- 기여자: `HaimLee-4869`
- 대상 브랜치: `devel`
- Head: `8911cc81562f252bf0d8bc7bc08a1d88bdbc2f29`
- 검토일: 2026-05-27

## 1. 검토 결론

권장안: **수정 방향은 수용하되, 그대로 cherry-pick하지 않고 maintainer 보강 패치로 반영**.

PR이 지적한 원인은 타당하다. 본문 문단 경로는 `apply_paragraph_numbering`을 호출하지만,
표 셀 문단 경로는 `compose_paragraph`만 호출하므로 `head_type=Number` 문단의 `"1."`,
`"2."` 같은 자동 번호가 셀 안에서 빠질 수 있다.

다만 PR 구현은 `apply_paragraph_numbering`을 셀의 `composed_paras` 생성 시점에 일괄 호출한다.
이 함수는 렌더링용 문자열만 만드는 순수 함수가 아니라 `numbering_state.advance(...)`로 번호
카운터를 전진시킨다. 따라서 페이지 분할, 반복/스킵 셀, `table_partial` 경로에서 실제로
번호가 출력되지 않는 문단까지 카운터가 전진할 수 있다.

## 2. PR 정보

| 항목 | 값 |
|------|-----|
| 상태 | open |
| mergeable | true |
| 변경 파일 | 5 |
| 변경량 | +126 / -7 |
| PR 댓글 | 없음 |
| 리뷰 | 없음 |
| CI | success |

CI run:

```text
CI: success (run 26450995262)
CodeQL: success (run 26450995152)
Render Diff: success (run 26450995153)
```

## 3. 변경 요약

PR 변경 파일:

```text
src/renderer/layout/paragraph_layout.rs
src/renderer/layout/table_cell_content.rs
src/renderer/layout/table_layout.rs
src/renderer/layout/table_partial.rs
tests/issue_cell_paragraph_numbering.rs
```

핵심 변경:

```text
1. Number/Outline 문단 번호 문자열에 text_distance가 있으면 trailing space 추가
2. 표 셀 문단 compose 경로 3곳에 apply_paragraph_numbering 호출 추가
3. k-water-rfp.hwpx p20 셀 번호 출력 회귀 테스트 추가
```

## 4. 타당한 부분

다음 문제 정의는 맞다.

```text
본문 문단:
  compose_paragraph
  -> apply_paragraph_numbering
  -> numbering_text 생성

표 셀 문단:
  compose_paragraph
  -> numbering_text 없음
```

`samples/hwpx/k-water-rfp.hwpx`의 예정공정표처럼 셀 안 헤딩이 문단번호를 사용하면,
본문 경로와 셀 경로의 차이 때문에 한컴 정답과 달라질 수 있다.

`paragraph_layout.rs`에서 Number/Outline 번호 문자열에도 `text_distance` 기반 공백을
반영하려는 방향도 Bullet 처리와 대칭이라 자연스럽다.

## 5. 그대로 받기 어려운 부분

### 5.1 번호 카운터 전진 시점

`apply_paragraph_numbering`은 내부에서 다음 처리를 수행한다.

```text
self.numbering_state.borrow_mut().advance(...)
```

즉, 호출 자체가 문서 번호 상태를 바꾼다.

PR은 다음 위치에서 셀 문단 전체에 일괄 호출한다.

```text
table_layout.rs
table_cell_content.rs
table_partial.rs
```

이 방식은 실제 렌더 여부와 무관하게 번호 카운터를 전진시킬 수 있다.

특히 위험한 경로:

```text
table_partial.rs:
  line_ranges 계산 후 start_line >= end_line 문단은 스킵된다.
  하지만 PR 구현은 그 전에 모든 cell.paragraphs에 번호를 적용한다.

table_layout.rs:
  row_filter에서 연속 페이지 rowspan 셀 텍스트를 clear하기 전 번호가 적용될 수 있다.
```

본문 `PartialParagraph` 경로는 첫 조각(`start_line == 0`)에서만 번호를 적용한다.
셀 경로도 같은 원칙을 따라야 한다.

### 5.2 Outline fallback

PR 설명은 `Number/Outline`을 모두 언급하지만, 셀 경로 호출은 `outline_numbering_id=0`을
넘긴다.

현재 구현의 `resolve_numbering_id` 규칙:

```text
para_style.numbering_id == 0 && head_type == Outline
  -> section outline_numbering_id 사용
```

따라서 셀 안 `HeadType::Outline`을 제대로 지원하려면 가능한 경로에서는 구역의
`outline_numbering_id`를 전달해야 한다. 그렇지 않으면 이번 변경은 실질적으로
`HeadType::Number` 중심 수정으로 보는 것이 맞다.

## 6. 권장 처리

권장 반영 방식:

```text
1. PR #1136의 문제 정의와 테스트 샘플은 수용한다.
2. 구현은 최신 local/devel 위에서 maintainer 보강 패치로 적용한다.
3. 셀 문단 번호 적용은 실제 렌더 시점 또는 가시 첫 조각(start_line == 0)에서만 수행한다.
4. table_partial / row_filter / repeated header 셀에서 번호 카운터가 불필요하게 전진하지 않는지 확인한다.
5. Number text_distance 공백 처리는 유지한다.
6. 가능하면 Outline fallback용 outline_numbering_id 전달 여부를 함께 점검한다.
```

검증 게이트:

```text
cargo fmt --all -- --check
cargo test --test issue_cell_paragraph_numbering -- --nocapture
cargo test --test svg_snapshot
cargo test --lib
cargo clippy -- -D warnings
docker compose --env-file .env.docker run --rm wasm
```

시각 판정 대상:

```text
samples/hwpx/k-water-rfp.hwpx
pdf/k-water-rfp-2024.pdf
```

## 7. PR 코멘트 초안

```text
HaimLee-4869님, PR 감사합니다.

표 셀 문단 경로에서 `head_type=Number` 자동 번호가 빠지는 원인 분석은 타당합니다.
본문 문단 경로와 셀 문단 경로의 `apply_paragraph_numbering` 호출 차이도 확인했습니다.

다만 `apply_paragraph_numbering`은 번호 문자열만 만드는 순수 함수가 아니라 내부 numbering
counter를 전진시키므로, 셀 문단 전체 compose 시점에 일괄 호출하면 split/partial table,
rowspan continuation, skipped cell 경로에서 실제 출력되지 않는 문단까지 번호 상태가 전진할
수 있습니다.

maintainer 쪽에서 PR의 방향과 회귀 테스트 취지를 수용하되, 최신 `devel` 위에서 렌더되는
첫 조각에만 번호를 적용하는 방식으로 보강해 반영하겠습니다. 좋은 분석과 재현 샘플 감사합니다.
```

## 8. 승인 요청

위 권장안으로 진행하려면 다음 단계에서 `local/devel` 위에 보강 구현을 적용하고,
PR #1136의 테스트 취지를 반영한 회귀 가드를 추가한다.

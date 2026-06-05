# PR #1136 처리 보고서

- PR: `#1136`
- 제목: `fix(renderer/table): 표 셀 내 문단 자동번호 누락 정정 — 본문 path 와 정합`
- 기여자: `HaimLee-4869`
- 처리일: 2026-05-27

## 1. 처리 결론

**권장안대로 maintainer 보강 패치로 반영 완료.**

PR의 문제 정의는 수용했다. 표 셀 문단에서도 본문 문단과 동일하게 자동 문단번호가
출력되어야 한다.

다만 PR 원안처럼 셀 문단 compose 시점에 `apply_paragraph_numbering`을 일괄 호출하지
않았다. 해당 함수는 번호 카운터를 전진시키므로, 실제 출력되지 않는 partial/split 셀 문단까지
카운터가 움직일 수 있기 때문이다.

## 2. 구현 내용

변경 파일:

```text
src/renderer/layout.rs
src/renderer/layout/paragraph_layout.rs
src/renderer/layout/table_cell_content.rs
src/renderer/layout/table_layout.rs
src/renderer/layout/table_partial.rs
tests/pr_1136_cell_paragraph_numbering.rs
```

핵심 처리:

```text
1. Number/Outline 번호 문자열도 Bullet과 동일하게 text_distance > 0이면 trailing space를 붙인다.
2. 일반 표 셀 렌더링 시 실제 layout_composed_paragraph 호출 직전에 번호를 적용한다.
3. partial table은 start_line == 0인 가시 첫 조각에서만 번호를 적용한다.
4. 본문 표 경로에는 section outline_numbering_id를 전달해 Outline fallback 가능성을 열어 둔다.
5. 글상자 내부 embedded table은 현 경로상 outline 컨텍스트가 없으므로 Number 중심으로 처리한다.
```

## 3. 회귀 가드

추가 테스트:

```text
tests/pr_1136_cell_paragraph_numbering.rs
```

검증 대상:

```text
samples/hwpx/k-water-rfp.hwpx page 20
```

검증 내용:

```text
예정공정표 셀 안 heading에서 `1.` / `2.` 번호 prefix가 y≈270 / y≈558 위치에 출력되는지 확인.
```

## 4. 검증 결과

```text
cargo fmt --all -- --check
  success

cargo test --test pr_1136_cell_paragraph_numbering -- --nocapture
  1 passed

cargo test --test svg_snapshot
  8 passed

cargo test --test issue_1145 -- --nocapture
  1 passed

cargo test --test issue_554 -- --nocapture
  12 passed

cargo test --lib
  1405 passed, 0 failed, 6 ignored
  기존 warning 6건 유지

cargo clippy -- -D warnings
  success

docker compose --env-file .env.docker run --rm wasm
  success
```

작업지시자 시각 판정:

```text
2026-05-27: 통과
```

## 5. PR 코멘트 초안

```text
HaimLee-4869님, PR 감사합니다.

표 셀 문단 경로에서 `head_type=Number` 자동 번호가 빠지는 원인 분석은 타당합니다.
본문 문단 경로와 셀 문단 경로의 `apply_paragraph_numbering` 호출 차이도 확인했습니다.

다만 `apply_paragraph_numbering`은 번호 문자열만 만드는 순수 함수가 아니라 내부 numbering
counter를 전진시키므로, 셀 문단 전체 compose 시점에 일괄 호출하면 split/partial table,
rowspan continuation, skipped cell 경로에서 실제 출력되지 않는 문단까지 번호 상태가 전진할
수 있습니다.

maintainer 쪽에서 PR의 방향과 회귀 테스트 취지를 수용하되, 최신 `devel` 위에서 실제 렌더되는
첫 조각에만 번호를 적용하는 방식으로 보강해 반영했습니다. 좋은 분석과 재현 샘플 감사합니다.
```

## 6. 다음 절차

```text
1. 작업지시자 승인
2. 커밋
3. local/devel → devel 동기화
4. 원격 devel push
5. PR #1136에 코멘트 작성
6. PR #1136 및 관련 이슈 정리
```

# task_m100_4412 Stage 1 — 클립보드 셀 border_fill_id 1-based 보정

- **이슈**: [#4412](https://github.com/edwardkim/rhwp/issues/4412)
- **브랜치**: `fix/issue-4412-clipboard-border-fill`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 전체 게이트 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 결함

`clipboard.rs:1660-1661` 이 `styles.border_styles.get(cell.border_fill_id as usize)` 로 **보정 없이**
조회한다. `border_fill_id` 는 1-based 다.

## 2. 전수 확인 — 유일한 예외였다

`border_fill_id` 로 인덱싱하는 소비처 약 20곳을 전수 조사했다:

`renderer/layout.rs:3328,3441` · `layout/table_layout.rs:1408,2134,2544,2566,2713,5735` ·
`layout/table_cell_content.rs:533,582` · `layout/paragraph_layout.rs:2637,4706,4944` ·
`layout/table_partial.rs:885,3113` · `queries/hidden_text.rs:361` ·
`queries/rendering.rs:2272,2517,5901` · `commands/table_ops.rs:1642`

**예외 없이 모두** `saturating_sub(1)` 또는 `(id - 1)` 을 한다. `clipboard.rs` 만 달랐다.

1-based 근거도 확인했다 — `resolve_border_styles()`(`style_resolver.rs:892-897`)가
`doc_info.border_fills` 를 0-based 로 그대로 매핑하고, `html_table_import.rs:765` 주석
*"border_fill_id는 1-based"*, `doc_info.rs` 파서, `parser/hwp3/mod.rs:3922` 주석이 일치한다.
id=0 은 "없음", id=N 은 `border_fills[N-1]` 이다.

## 3. 재현

`border_fills = [default0, default1, decoy(회색), REAL(#00ff00/#00ffff), decoy2(#ff8c00/#ff00ff)]`,
`cell.border_fill_id = 4`(REAL) 로 수정 전 코드를 실행하니 **decoy2 색이 한 칸 밀려 출력**됐다.

## 4. `table.border_fill_id` 는 같은 결함이 아니다

`clipboard.rs` 가 표 자체 테두리를 전혀 참조하지 않는 것은 맞다. 하지만
`layout/table_layout.rs:2708-2760` 의 사용처는 단순 오프바이원이 아니라 **셀이 덮지 않는 표 외곽
엣지에만 채우는 outer-edge coverage 알고리즘**(행·열별 covered map 구축)이다. 셀 단위 `<td>` CSS
생성기에 이식하려면 새 기능 포팅이라 성격이 다르다 — 이번 범위에서 제외했다. 필요하면 별도
이슈로 분리한다.

## 5. 검증 (완료)

- 회귀 테스트 `table_to_html_uses_correct_border_fill_for_1_based_id` 신설. **수정 전 실행에서
  decoy2 색이 나와 실패를 확인**했고, 수정 후 REAL 색이 나온다.
- `cargo test --profile release-test --tests` — 497개 블록 전부 ok, **실패 0건**
  (반복 관측되던 타이밍 flake 도 이번 실행에서는 통과).
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과.

## 6. 조사 경위

문서 간 복사·붙여넣기 보존 조사에서 나왔다. 그 조사가 확인한 문서 간 경로는 하나뿐이다 —
내부 클립보드는 문서를 넘지 못하고(`paste_*_native` 가 `&self.clipboard` 만 읽는다), 실제 경로는
**Rust → HTML → OS 클립보드 → HTML → Rust** 다.

같은 조사에서 #4413(셀 안 컨트롤 미순회), #4414(도형·필드 소실)가 함께 나왔고 별도 이슈로 분리했다.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.

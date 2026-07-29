# PR #2455 리뷰 - HWPX 표 textWrap TIGHT/THROUGH 왕복 보존

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2455](https://github.com/edwardkim/rhwp/pull/2455) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G5 체리픽 통합 |
| 원 커밋 / 통합 적용 커밋 | `bb948191acb98a712960db7e3b33bf4ce117e8fb` / `9bc962a5c` |
| 적용 순서 | G5 1/8, 최신 `upstream/devel` 위 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR은 `MERGEABLE` / `BEHIND`, 원 head CI는 성공이다. 통합 PR의 최신 head CI와 mergeable 상태를 merge 전 다시 확인한다. |

## 변경 검토

표 `textWrap` 파서는 `TIGHT`와 `THROUGH`를 알지 못해 기본값 `Square`로 내렸다. 반면 HWPX
serializer는 두 값을 이미 방출하고 있어, 이 PR은 표의 빽빽하게/투과 배치가 저장 후 재로드할 때
어울림으로 바뀌는 명백한 parser-serializer 비대칭을 복구한다.

기여자 커밋은 표 parser에 두 match arm과 `TIGHT`/`THROUGH` 왕복 회귀를 추가한다. 다른 그림·도형·차트
parser의 기존 처리와도 일관된다.

## 검증

| 게이트 | 결과 |
|---|---|
| `table_textwrap_tight_and_through_survive_roundtrip` | PASS |
| `cargo test --profile release-test --tests` | PASS |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS |
| `wasm-pack build --target web --out-dir pkg` | PASS |

## 시각 검증 판단

변경은 HWPX parser의 enum 복원 규칙이며 renderer, layout, pagination, paint 경로는 바꾸지 않는다.
본문도 기준 PDF 또는 시각 fidelity 개선을 주장하지 않는다. 구조 왕복 회귀와 WASM build로 검증했으므로
PDF visual sweep 또는 MCP 기준 PDF 비교는 적용하지 않는다.

## 최종 의견

G5 통합 PR에 기여자 원 커밋을 보존해 수용한다. 통합 순서와 공통 검증은
[G5 통합 실행 계획](pr_2455_review_impl.md)을 따른다.

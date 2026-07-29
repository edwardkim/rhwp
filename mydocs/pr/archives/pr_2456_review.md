# PR #2456 리뷰 - HWPX DISTRIBUTE_SPACE 나눔 정렬 모델·렌더 정합

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2456](https://github.com/edwardkim/rhwp/pull/2456) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G5 체리픽 통합 + maintainer 보정 |
| 원 커밋 / 통합 적용 커밋 | `a3b1024e8667ac2eb44569e181c235861729930f` / `2b51a29e5` |
| 적용 순서 | G5 2/8, #2455 뒤 적용. 원 PR CI 실패 원인을 maintainer가 보정 |
| 작성 시점 참고값 | 원 PR은 `MERGEABLE` / `BEHIND`이나 원 head의 `Build & Test`는 `issue_1692` 실패로 종료됐다. 통합 head에서는 아래 회귀와 전체 검증을 다시 통과했다. |

## 변경 검토

OWPML 스키마의 `DISTRIBUTE_SPACE`는 나눔 정렬, 즉 공백에만 여분 폭을 분배하는 값이다. HWP5에서도
같은 모델 값은 `Split`이므로, 원 PR의 `DISTRIBUTE_SPACE -> Alignment::Split` 복원 방향은 맞다.

원 PR은 parser만 바꿨다. 당시 renderer는 `Split`을 글자 전체에 간격을 분배하는 `Distribute` 경로로
처리했고, 마지막 줄인 머리말은 분배 자체를 생략했다. 이 때문에 실제 `samples/SO-SUEOP.hwpx` p5의
머리말 회귀가 깨져 원 PR CI가 실패했다.

maintainer 보정은 `Split`을 `Justify`와 같은 공백 전용 분배 경로로 보내며, 기존 머리말/꼬리말 단일 줄의
폭 채움 예외를 유지한다. 따라서 model 값은 `Split`으로 보존하면서 기존 실문서 조판 계약도 유지한다.

## 검증

| 게이트 | 결과 |
|---|---|
| `parser::hwpx::header::tests::test_parse_alignment` | PASS - `DISTRIBUTE_SPACE -> Split` |
| `issue_1692_so_sueop_header_footer_page5_matches_reference_contract` | PASS - p5 머리말 model과 폭 유지 |
| `cargo test --profile release-test --tests` | PASS |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS |
| `wasm-pack build --target web --out-dir pkg` | PASS |

## 시각 검증

`samples/SO-SUEOP.hwpx`와 `pdf/SO-SUEOP-2024.pdf`의 5쪽을 대조했다. SVG/PDF는 각각 46쪽이고,
선택 5쪽의 visual sweep 자동 flag는 0/1이다. 머리말의 좌우 폭 채움과 footer 위치가 기준 PDF와 같은
계약으로 유지됨을 review 이미지에서 확인했다. 증적은
[`so_sueop_p005_review.png`](../assets/pr_2456/so_sueop_p005_review.png)에 보존한다.

## 최종 의견

원 PR의 model 복원 의도는 수용한다. parser만으로 발생한 CI 회귀는 maintainer renderer 보정과 실문서
회귀, visual sweep으로 해소했으므로 G5 통합 PR에 수용한다. 공통 검증과 최종 조건은
[G5 통합 실행 계획](pr_2455_review_impl.md)을 따른다.

# PR #2818 검토 기록

| 항목 | 내용 |
|---|---|
| PR | [#2818](https://github.com/edwardkim/rhwp/pull/2818) |
| 작성자 / base | [@planet6897](https://github.com/planet6897) / `devel` |
| reviewer | [@jangster77](https://github.com/jangster77) |
| 관련 이슈 | [#2814](https://github.com/edwardkim/rhwp/issues/2814) |
| 범위 | 한 문단의 비-TAC `TopAndBottom` co-anchored 그림이 3장 이상일 때 쪽 용량에 따라 분배 |
| 처리 경로 | collaborator 체리픽 통합 검토. 원 커밋 `68671d565`를 통합 커밋 `1f63ddf34`로 적용 |
| 통합 기준 | `upstream/devel` `491e56fcc` 위 체리픽, #2819·#2820과 충돌 0건 |

## 검토 결론

수정은 그림 스택 일반 경로를 넓게 바꾸지 않고 문단당 그림 3장 이상에만 발동한다. 현재 쪽에
다른 항목이 있고 다음 그림까지 더한 스택 하단이 본문을 넘을 때 방금 추가한 그림을 새 쪽으로
옮긴다. 2장 스택은 한컴이 한 쪽에 유지하는 실측 반례가 있어 기존 동작을 보존한다.

기여자가 사용한 5.4MB 실문서는 공개 sample에 포함되지 않아 독립 visual sweep을 재현하지
못했다. 대신 6장 스택의 2장/쪽 분배와 2장 스택 무변경을 합성 회귀 테스트로 고정했고, 최종 전체
회귀 게이트에서 함께 통과했다. 이 제한은 검토 결과에 명시하고 원 PR이 보고한 실문서 47→65쪽,
한컴 65쪽 일치 증적과 분리해 판단한다.

## 검증

- `cargo build --release`: 성공
- `cargo test --release --lib`: 2520 passed, 7 ignored
- `cargo test --profile release-test --tests`: 전체 성공
- `cargo fmt --check`, `git diff --check`: 성공
- `cargo clippy --all-targets -- -D warnings`: 성공
- `cargo test --doc`: 0 passed, 1 ignored, 실패 0
- `rhwp-studio`: `npx tsc --noEmit` 성공, `npm test` 505/505
- `wasm-pack build --target web --out-dir pkg`: 성공
- 작업지시자 WASM 브라우저 검증: 완료

## 권고

통합 PR의 CI 성공을 조건으로 수용한다. merge 뒤 #2814의 close 상태를 확인하고 원 PR에는 통합
경로와 기여 크레딧을 남긴다.

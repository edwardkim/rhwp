# kevin9327 G5 HWPX 왕복 보존 통합 실행 계획

## 대상과 범위

이 계획은 다음 원 PR의 독립적인 HWPX parser/serializer 누락을 하나의 최신 `devel` 기반 통합 PR로
검증·반영하기 위한 기록이다.

- 수용: [#2455](https://github.com/edwardkim/rhwp/pull/2455), [#2456](https://github.com/edwardkim/rhwp/pull/2456), [#2457](https://github.com/edwardkim/rhwp/pull/2457), [#2459](https://github.com/edwardkim/rhwp/pull/2459), [#2460](https://github.com/edwardkim/rhwp/pull/2460), [#2461](https://github.com/edwardkim/rhwp/pull/2461), [#2462](https://github.com/edwardkim/rhwp/pull/2462), [#2463](https://github.com/edwardkim/rhwp/pull/2463)

모든 원 PR에 reviewer `jangster77`를 먼저 지정했고, 본문과 등록된 PR 댓글을 각각 확인했다. 이 문서는
G5 전체의 체리픽 순서와 통합 보강을 설명하며, 개별 판정은 각 `pr_{N}_review.md`가 정본이다.

## 적용 순서

1. `9bc962a5c` - #2455 표 textWrap `TIGHT`/`THROUGH` parser 보완
2. `2b51a29e5` - #2456 `DISTRIBUTE_SPACE` 모델 복원과 공백 전용 분배 renderer 보정
3. `23382b987` - #2457 `BETWEEN_LINES` 줄간격 parser 보완
4. `39ef2206f` - #2459 3D 테두리 선 종류 parser 보완
5. `8dcd5d12e` - #2460 `PATTERN_8_8` 그림 효과 parser 보완
6. `e6b22d4c9` - #2461 `DOUBLE_SLIM` 탭 리더 parser 보완
7. `5d0ac30a9` - #2462 이중·삼중선 탭 리더 serializer 보완
8. `5adc69b78` - #2463 `symMark` 강조점 parser 보완

#2461은 #2459의 test module 인접 삽입과 충돌했다. 두 회귀를 모두 유지하는 위치 충돌이었으며 기능
코드의 충돌이나 원 커밋 rewrite는 없었다.

## Collaborator 보강

`685c7943c`는 기여자 동작 변경을 대체하지 않고 다음 회귀 범위를 넓힌다.

- 실제 `hp:pic` section에서 `PATTERN_8_8 -> ImageEffect::Pattern8x8`을 확인한다.
- `symMark` 지원값 NONE, DOT_ABOVE, RING_ABOVE, TILDE, CARON, SIDE, COLON을 모두 확인한다.

`2b51a29e5`는 [#2456](https://github.com/edwardkim/rhwp/pull/2456)의 `DISTRIBUTE_SPACE -> Split`
모델 복원 방향을 유지하면서, `Split`이 기존 renderer의 글자 전체 분배 경로로 들어가던 문제를 보정한다.
OWPML과 HWP5의 의미에 맞게 `Justify`와 동일한 공백 전용 분배를 사용하되, 머리말/꼬리말 단일 줄의
기존 폭 채움 계약도 유지한다.

## 검증

| 게이트 | 결과 |
|---|---|
| 8개 focused parser/serializer 회귀 | PASS |
| `cargo test --profile release-test --tests` | PASS |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS |
| `wasm-pack build --target web --out-dir pkg` | PASS |

[#2456](https://github.com/edwardkim/rhwp/pull/2456)는 정렬 renderer 경로도 보정하므로
`samples/SO-SUEOP.hwpx`와 `pdf/SO-SUEOP-2024.pdf`의 5쪽을 visual sweep으로 대조했다.
SVG/PDF 페이지 수는 46/46, 자동 flag는 0/1이며, review 증적은
[`so_sueop_p005_review.png`](../assets/pr_2456/so_sueop_p005_review.png)에 보존한다.
나머지 7건은 parser/serializer model 보존 변경이므로 별도 visual sweep 대상이 아니다.

## 다음 단계

1. 이 실행 계획, 개별 review 문서, 오늘할일을 별도 문서 커밋으로 추가한다.
2. 사용자 승인 뒤 통합 branch를 원본 저장소에 push하고 `devel` 대상 Open PR을 연다.
3. 통합 PR 최신 head CI와 mergeable 상태를 확인한 뒤 사용자 승인에 따라 merge한다.
4. merge 후 수용한 8개 원 PR에 통합 PR 링크와 검증 요약을 남기고 close한다.

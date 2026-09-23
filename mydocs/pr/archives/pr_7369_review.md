# PR #7369 리뷰 — Hancom 기준 PDF 36종 재생성

## 최종 판정

**승인.** 이번 통합에서는 추적된 기준 PDF 교체와 전체 회귀 소비 경로를 확인했다. 새 변환을 임의 생성하지 않았으며, merge 후 원 PR comment에는 교체 범위와 실제 merge SHA를 명시한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7369](https://github.com/edwardkim/rhwp/pull/7369) |
| 작성자 | `planet6897` |
| 관련 이슈 | [#7352](https://github.com/edwardkim/rhwp/issues/7352) |
| 원 contributor head | `67e76c536ba` |
| 승인분 통합 PR | 새 `upstream/devel` 기반 PR 준비 중 (`codex/planet-approved-clean-20260924`) |
| reviewer | `jangster77` |

## 변경과 통합 순서

손상되었거나 잘못된 oracle이던 기준 PDF 36종을 한컴 출력물로 교체한다.

#7374의 추가 PDF 2종보다 먼저 적용했다. 코드 충돌은 없었다.

## 통합 후보 검증

승인된 9개 원 PR만 `upstream/devel` `c84a8f8e1a97acaf8626b28a14417344a8578f41` 위에
다시 적용한 `codex/planet-approved-clean-20260924` 후보를 대상으로 한다. #7366·#7371과
#7371에 의존하는 #7372는 포함하지 않는다. 이전 #7380 혼합 head의 전체 회귀 결과는 이
후보의 검증 결과로 재사용하지 않는다. 로컬 검증과 새 head CI 결과가 확정되기 전에는
아래 수용 판정은 원 PR의 개별 검토 판정이며 통합 PR의 merge 승인이 아니다.

| 검증 | 새 후보 결과 |
| --- | --- |
| Rust format 및 Visual Sweep Python 계약 | format 통과, Python 67 PASS |
| release-test 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir /Users/tsjang/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast` | 최종 후보에서 10,193 PASS / 실패 0 / skip 50 (521.763초) |
| #7365 TAC 표 focused 6개 | 위 전체 실행 안에서 **6 PASS** |
| Native Skia 전체 lib 및 실물 fixture 2종 | 최종 후보에서 lib 3,930 PASS (13 ignored)·workspace lib 182 PASS·실물 #2225 2 PASS·p37 4 PASS |
| Clippy·workspace 빌드·fresh WASM | 최종 후보에서 기본/wasm32/workspace all-targets Clippy, workspace 빌드, 저장소 루트 `scripts/wasm-pack-locked.sh --target web --out-dir pkg` 모두 통과 |
| Native Visual Sweep 대표 쪽 | 비해당: 기준 PDF fixture만 교체하며 renderer 출력 변경 없음 |

## 증적과 범위

이 PR의 증적은 기준 PDF 자체이므로, merge comment에는 임의 PNG를 만들지 않고 교체한 파일 범위와 검증 범위를 명시한다.

승인분 후보의 cherry-pick commit `22703fc81`은 `pdf/` 아래 36개 파일을 교체한다
(`git diff-tree --no-commit-id --name-only -r 22703fc81 -- pdf` 확인). 교체본은 모두
후보 `HEAD` tree에 포함되고, 전체 `pdf/` tree ID는
`5e9d2414c3e310a44a121ffefb2f8593fe93a0c3`이다. #7374의 후속 2개 PDF 교체는
별도 commit `c94335b5b`와 [#7374 리뷰](pr_7374_review.md)에 구분한다.

이 기록은 원 PR의 최신 head를 최신 `upstream/devel` 기준으로 누적 적용한 통합 후보를 대상으로 한다.
원 PR 단독 CI와 통합 후보 CI는 구분하며, source PR의 merge 가능 상태만으로 통합 후보의 최종 head를
승인하지 않는다.

## 재산출 검증

이 PR은 renderer 동작을 고치는 변경이 아니라, 한컴 기준 PDF 36종을 다시 생성해 교체하는 증적자료 변경이다. 따라서 renderer와 PDF를 겹쳐 그린 Visual Sweep은 이 PR의 변경 계약을 검증하는 증적이 아니며, 산출·첨부·merge comment에 포함하지 않는다.

검토는 교체 파일의 추적 상태, producer·쪽수·원 PR의 fixture 검사와 통합 후보의 회귀 소비 경로로 한정한다. 새 PDF 변환은 하지 않았다.

## Merge 후 contributor PR comment 계획

최종 head CI와 실제 merge SHA가 확정된 뒤 원 PR에 한국어로 다음을 게시한다.

- 실제 merge SHA와 새 통합 PR의 최종 head CI URL, 이 PR의 원 contributor head를 구분해 기록한다.
- 교체한 PDF 범위, producer·쪽수·fixture 검증과 통합 회귀 소비 경로를 설명한다.
- 이 PR은 기준 PDF 재산출 변경이므로 renderer Visual Sweep PNG를 만들거나 링크하지 않았음을 명시한다.

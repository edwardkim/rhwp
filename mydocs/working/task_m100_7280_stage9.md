# Task #7280 Stage 9 — R2g 문단 분할 배치·이월 조정 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2f](task_m100_7280_stage8.md), 시작 head `aeadea675`.
- 상태: R2g 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.

## 1. 책임과 범위

줄 분할 루프를 `paragraph.rs::place_split_paragraph`의 조정 책임으로 옮긴다.
상위 `typeset_paragraph`는 기존의 진입 fit, 빈 구성 결과, 첫 줄 수용/저장 reset 판단을
그대로 수행한 뒤 이 조정자를 호출한다. 조정자는 스캔 → 경계 보정 → 배치 계획 → 확정 적용 →
필요 시 단/쪽 전환을 기존 순서로 실행한다.

읽기 전용 `paragraph/placement.rs::plan_fragment`는 보정된 SplitBoundary와 구성 결과에서
PageItem과 part_height를 함께 준비한다. None은 아직 배치하지 않았고 같은 시작 줄로
다음 단/쪽에서 재시도해야 한다는 의미다. 실제 상태는 받지 않으며 현재 항목 slice만 관측한다.
state의 `commit_split_paragraph_fragment`는 항목 추가 → trim 초기화 → 높이 전진만 수행하고
fit 재계산이나 페이지 전환을 하지 않는다. 범용 setter와 새 rule engine은 만들지 않는다.

기존 조판 규칙·상수·조건·연산 순서·기대값을 바꾸지 않는다. PageItem을 새 의미로 해석하거나
높이를 보정하지 않는다. 기존 heuristic의 타당성을 이번에 승인하는 것은 아니다.
테스트·baseline·ignore·IR/public API·표 셀 컷/continuation은 변경하지 않는다.

## 2. 높이·컷과 실제 소비 경로

| 경계 | 보존하는 의미 |
| --- | --- |
| 예산 | 진입에서 만든 base_available은 그대로 전달; 각주/현재 높이는 매 루프에서 다시 읽음 |
| 후보와 보정 | 기존 scan_lines → refine_split_boundary; 시작 줄과 saved-tail 채택 flag는 조정자 소유 |
| 요구 높이 | part_line_height는 보정된 범위의 advance 합; 끝 조각에만 spacing_after 가산 |
| 전체 수용 재확인 | cursor=0이고 끝 줄까지 후보인 경우에만 cumulative로 재확인; 직전 표의 trailing spacing 제외 유지 |
| 예산 실패 | None → advance_column_or_new_page → continue; cursor/trim/현재 항목을 먼저 변경하지 않음 |
| 배치 확정 | 같은 계획의 PageItem과 part_height를 적용; fit 누적량으로 높이를 대체하지 않음 |
| 남은 줄 이월 | 적용 후 끝 줄이면 종료; 아니면 단/쪽 전환 뒤 cursor=end_line |

최소 한 줄 수용과 saved-tail 예외의 정책은 R2e/R2f Query에 유지한다. 첫 줄/빈 문단/전체 fit의
별도 조기 반환은 여전히 parent 소유이며 이번 command로 일괄 치환하지 않는다.
이 절편은 본문 줄 분할의 실행 경계를 드러낸 것이며, 모든 state 직접 쓰기 제거나 R2 완료가 아니다.

## 3. 검증 계획

`output/7280/stage9/verify-placement.mjs`로 Query·Command를 원래 block으로 복원 비교한다.
높이/overflow 조건, Full/Partial 항목, 상태 적용 순서, 재시도와 cursor 진행 순서를 검사하고,
그 밖의 parent 코드는 변경되지 않았는지 대조한다. 공백/후행 쉼표 외 계산 차이는 허용하지 않는다.

이전 절편과 같은 typeset/composer 및 #6031/#6542/#6718 계약을 고정 SHA에서 새로 실행한다.
기존 문단 줄 분할/넘침/혼합 문단 계약과 실제 render tree의 하단/저장 경계 검사를 포함한다.
모든 분기의 직접 시각 검증이나 전체 포맷/다단/각주 조합의 전수 검증으로 주장하지 않는다.

review worktree에서 파생 suite 준비 → fmt·고정 baseline 정책 → native Clippy → 집중 nextest를
순차 실행한다. 전체 회귀·최종 WASM/workspace lint·workspace build·Native Skia·fresh Docker
WASM/직접 시각 검증은 통합 게이트에 남는다. 원격 push·PR·댓글은 하지 않는다.

### 고정 SHA와 선행 검사

- 제품 SHA: `effbb71cae60c1aeff1fc54367ffba1985b4bccd`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2g`.
- 정적 보존: `output/7280/stage9/extraction-proof.json` 통과.
- manifest: baseline 대비 1,382 sources / 5,965 static attrs / 48 integration targets 통과.
- unit-tier: baseline 대비 4,205 tests / 298 modules / ready 0 / support 87 /
  white-box 4,114 / cfg support items 28 통과.
- `cargo fmt --all -- --check` 통과.
- native Clippy(`--locked -- -D warnings`): exit 0, 56.18초.
- 로그: `output/7280/stage9/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

고정 review target `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`로
순차 실행했다. 파생 suite는 review worktree에서만 준비했으며 stage하지 않는다.

집중 실행 명령(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --lib --test regression_suite_011 --test regression_suite_019 --test regression_suite_028 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_6031_ladder_sb_omitted_tail_overrun::) | test(issue_6542_mid_para_vpos_rewind_breaks_page::) | test(issue_6718_native_hwp5_zero_vpos_rewind::) | test(issue_6718_zero_rewind_in_split_paragraph::)' \
  --no-fail-fast
```

선택한 기존 `test_typeset_page_overflow`는 가시 문단 100개의 전량/순서 보존을 검사한다.
`test_typeset_line_split`/`test_typeset_mixed_paragraphs`는 기존 Paginator와의 호환 검사이며
독립적인 한컴 oracle로 취급하지 않는다. #6031은 SVG 본문 하단/다음 쪽 첫 줄,
#6542/#6718은 render tree의 본문 경계를 검사한다. 실제 PageItem 소비와 paint 경로 자체는
변경하지 않았다. 전체 수용 재시도/표 직후 분기 조합 전부를 별도 실행으로 입증했다는 뜻은 아니다.

집중 결과: **154건 통과 / 실패 0건**, 필터 비선택 4,539건, 7 binaries, exit 0.
빌드 4분 57초, 테스트 0.333초. 실행 ID: `211592d1-badd-4b2b-8c36-71bdeaa5e124`.
로그: `output/7280/stage9/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행에서 같은
필터로 선택한 154개 PASS 이름과 일치함을 확인했다. 필터 비선택은 기존 ignore 50건과 별개이며
이번에 전체 회귀를 재실행한 것은 아니다.

nextest 0.9.137(권장 0.9.140), 미사용 observation profile 설정 경고는 기준 실행과 동일하다.
현재 CI 도구 버전까지 동일하다는 주장은 하지 않는다. review worktree의 tracked 변경은 없고
제품 검증 후에는 계획/완료 기록만 갱신했다. 테스트와 파생 suite/manifest는 커밋하지 않았다.

## 4. 다음 절편

문단 분할 루프의 계산·조정·확정 경계는 분리됐다. 남은 진입 fit/전체 배치 조정과
`typeset_table_paragraph`의 문단/컨트롤 흐름 책임 분리를 이어간다. state의 나머지 직접 쓰기,
공유 helper 소유권 및 최종 통합 검증은 아직 남아 있으므로 R2 전체와 #7280 완료로 판정하지 않는다.

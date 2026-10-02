# Task #7280 Stage 14 — R2l 빈 문단 조기 반환 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2k](task_m100_7280_stage13.md), 시작 head `2621b115c`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2l 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

이번 절편은 문단 진입 뒤 빈 문단의 조기 반환 세 경로를 분리한다. 저장 강제 경계·최종 fit
선택을 동시에 옮기지 않는다. 빈 문자열·가시 텍스트·공백 제외 텍스트 판정을 통합하지 않으며,
각 기존 조건·상수·항목 소유·숨김 횟수·높이 전진 여부를 그대로 유지한다.

- RowBreak 뒤 guide 흡수는 다단 처리 **전**에 판단하며 숨김 목록만 기록한다.
- `hide_empty_line` 처리는 다단 처리 **후**에만 실행한다. 옵션이 켜졌을 때 페이지별 횟수를
  먼저 초기화하고 판정한 뒤, 횟수 증가 → 숨김 기록 → FullParagraph 추가 순서로 반영한다.
- 구역 끝 빈 문단은 앞선 drift 조건에서 숨김 목록만 기록하거나, 안전여백/각주 차감 조건에서
  높이 전진 없이 항목만 남긴다. 두 효과를 동일한 숨김 처리로 합치지 않는다.

`paragraph/empty.rs`는 읽기 전용 입력으로 판단하고, paragraph 조정자가 결과별 command를
호출한다. state는 확정 항목·숨김 집합·횟수만 변경한다. tail 관측값의 base 높이는 부수효과
없는 조회이며, 실제 available은 이전 절편의 고정 예산을 그대로 소비한다.

소비 경로: 기존 fit 예산 → guide 조기 반환 → 기존 다단 경로 → 옵션/구역 끝 판정 →
숨김 집합·항목 반영 → 반환 또는 기존 저장 경계/fit/분할. 컷·높이·paint 재계산은 바꾸지 않는다.
이동 대상은 기존 호환 처리이며 올바른 한컴 규칙으로 재승인하거나 빈 줄 버그를 수정하지 않는다.
IR/API, 테스트 assertion·ID, baseline·golden·ignore, CI 정책은 수정하지 않는다.

## 2. 검증 계획

원래 block으로 복원하여 판단식·쓰기·조기 반환 순서를 대조한다. 다단 분기 사이의 호출
위치, 기존 높이 budget과 나머지 parent 불변을 확인한다. 별도 review worktree에서
파생 suite 준비 → fmt·고정 baseline 대비 manifest/unit-tier → native Clippy → 집중 nextest를
순차 실행한다. typeset/composer의 빈 문단·다단 계약과 기존 빈 문단 고아 페이지/fit 계약을
포함하되, 기존 테스트가 세 경로의 모든 조합을 직접 보호한다고 주장하지 않는다.

전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM/직접 시각 대조는 통합 게이트에
남긴다. 원격 push·PR·댓글은 수행하지 않는다.

## 3. 고정 head 검증

- 제품 SHA: `e8fb1aac2acfa47ad0660397218c0d58e2b2ec22`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2l`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 복원 비교: `output/7280/stage14/verify-empty.mjs` / `extraction-proof.json` 통과.
  세 경로의 판정·반환 효과, 페이지별 횟수 초기화와 쓰기 순서, 관측값 매핑,
  다단 분기 전후 호출 위치와 나머지 parent/기존 조정자 불변을 확인했다.
- manifest: 고정 baseline 대비 1,382 sources / 5,965 static attrs / 48 targets 통과.
- unit-tier: 고정 baseline 대비 4,205 tests / 298 modules / ready 0 / support 87 /
  white-box 4,114 / cfg support 28 통과.
- `cargo fmt --all -- --check` 통과.
- `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review
  -- -D warnings` 통과(exit 0, 56.83초).
- 로그: `output/7280/stage14/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

집중 테스트(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_003 --test regression_suite_005 \
  --test regression_suite_011 --test regression_suite_012 \
  --test regression_suite_013 --test regression_suite_022 --test regression_suite_026 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_6568_para_start_first_line_fit::) | test(issue_5755_rewind_overflow_page_break::) | test(issue_5921_neartop_reset_fits::) | test(issue_6753_lazy_base_keeps_trimmed_spacing_before::) | test(issue_6793_tac_host_page_tail_padding::) | test(issue_5870_empty_host_float_flow_advance::) | test(issue_6854_empty_para_orphan_page::) | test(issue_6970_multicolumn_synth_ladder_fill::)' \
  --no-fail-fast
```

기존 typeset의 `page_bottom_empty_paragraph_before_vpos_reset_does_not_create_blank_page`,
`page_bottom_empty_run_before_vpos_reset_does_not_create_blank_page`는 숨김 집합·문단 항목·빈 쪽을
검사한다. #6854는 빈 문단 고아 쪽 관련 기존 실물 사례, #6970은 다단의 실제 점유/예산 계약이다.
각 테스트는 관련 동작의 기존 보호 계약이며, 새 조정자의 모든 경로를 직접 실행했다고
확대 해석하지 않는다. hide_empty_line의 페이지별 2회 상한과 guide/구역 끝의 모든 조합에
대한 새 직접 계약 테스트는 이번 절편에 추가하지 않았다.

결과: **166건 통과 / 실패 0건**, 11 binaries, 필터 비선택 5,388건, exit 0.
빌드 5분 26초, 테스트 0.444초. 실행 ID: `229ece28-2dcc-4567-b4ed-15b7000818f4`.
로그: `output/7280/stage14/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행 중 같은 필터의
166개 PASS 이름과 일치함을 확인했다. 필터 비선택은 기존 ignore 50건과 별개이며,
이번 절편에서 전체 회귀를 재실행하지 않았다.

nextest 0.9.137(권장 0.9.140)과 observation profile 미사용 설정 경고는 기준 실행과 동일하다.
CI 도구 버전 일치나 전체 CI 통과를 주장하지 않는다. review worktree의 tracked 변경은 없고
파생 suite·manifest를 커밋하지 않았다. 제품 검증 후에는 계획과 결과 기록만 변경했다.

## 4. 후속

강제 저장 경계와 최종 fit 선택, 표 문단/컨트롤 흐름 분리를 계속한다.
R2 전체와 최종 통합 게이트는 남아 있으며, 이번 절편은 외부 push·PR 없이 로컬 커밋으로 닫는다.

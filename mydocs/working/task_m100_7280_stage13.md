# Task #7280 Stage 13 — R2k 문단 진입과 fit 예산 준비 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2j](task_m100_7280_stage12.md), 시작 head `da9e67b4f`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2k 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

문단 진입의 저장 꼬리 판정과 편집 세션 그림의 공간 요구를 `paragraph/entry.rs`로 분리한다.
Query는 IR·구성 결과와 필요한 읽기 전용 값만 받으며 가변 state를 받지 않는다.
`available_height()`는 진단을 동반할 수 있으므로 closure로 전달하여 원래 단락 평가 위치와
횟수를 보존한다. 저장 꼬리 판정 후 높이 적용 시 별도 조회하는 기존 동작도 유지한다.

`paragraph::prepare_fit_budget`이 저장 꼬리 → 진입 진단 → 편집 그림 이월 → 엄격 fit flag
소비 → 안전여백 → float 배제 → 각주 반환 → 꼬리 허용 → 최종 예산 순서를 조정한다.
엄격 fit 소비와 저장 꼬리 높이 적용은 state command가 소유한다. 기존 R2d 보정 command는
재사용하며 조건·상수·호출 순서·flag 수명은 바꾸지 않는다. 엔진의 session-edited 값은
진입 인자로 관측한다. 이 구간은 엔진 profile을 변경하지 않는다.

소비 경로: 진입/그림 판정 → 기존 높이/페이지 전이 → 기존 보정으로 산출한 available →
상위의 빈 문단 흡수·강제 경계·fit → 기존 전체/분할 배치. 이번에는 예산을 산출하는
책임만 분리하며 이후 분기에서의 재계산·좌표 선택·paint는 변경하지 않는다.
기존 heuristic을 올바른 한컴 조판 규칙으로 새로 승인하는 작업이 아니다.

빈 문단 흡수, 강제 저장 경계, 최종 fit 선택, 표 문단 흐름은 후속에 남긴다.
IR/API, 테스트 assertion·ID, baseline·golden·ignore, CI 정책은 수정하지 않는다.

## 2. 검증 계획

이동 block의 복원 비교로 조건/연산/상태 적용 순서를 확인하고, review worktree에서
파생 suite 준비·fmt·고정 baseline 대비 manifest/unit-tier·native Clippy·집중 nextest를 순차 실행한다.
typeset/composer와 진입 저장 꼬리·빈 host float 및 기존 문단 fit 관련 계약을 포함한다.
전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM/직접 시각 대조는 최종 통합
게이트에 남긴다. 집중 PASS를 전체 CI나 한컴 시각 일치로 보고하지 않는다.
원격 push·PR·댓글은 수행하지 않는다.

## 3. 고정 head 검증

- 제품 SHA: `08b248bc367a0b03c703c62ee6d11f99ad0c88a1`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2k`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 복원 비교: `output/7280/stage13/verify-entry.mjs` / `extraction-proof.json` 통과.
  Query의 지연 조회·연산, 예산 준비 순서, state command와 이동 외 parent/기존 조정자 불변을 확인했다.
- manifest: 고정 baseline 대비 1,382 sources / 5,965 static attrs / 48 targets 통과.
- unit-tier: 고정 baseline 대비 4,205 tests / 298 modules / ready 0 / support 87 /
  white-box 4,114 / cfg support 28 통과.
- `cargo fmt --all -- --check` 통과.
- `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review
  -- -D warnings` 통과(exit 0, 56.50초).
- 로그: `output/7280/stage13/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

집중 테스트(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_003 --test regression_suite_005 \
  --test regression_suite_011 --test regression_suite_012 \
  --test regression_suite_013 --test regression_suite_022 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_6568_para_start_first_line_fit::) | test(issue_5755_rewind_overflow_page_break::) | test(issue_5921_neartop_reset_fits::) | test(issue_6753_lazy_base_keeps_trimmed_spacing_before::) | test(issue_6793_tac_host_page_tail_padding::) | test(issue_5870_empty_host_float_flow_advance::)' \
  --no-fail-fast
```

`#6793`는 차례의 쪽 귀속·본문 상단·용지 밖 출력, `#5870`은 본표와 결재란의 실제 위치 및
정상 대조 페이지를 검사한다. typeset의 1회성 엄격 fit 검사는 기존 private helper의
flag 소비 계약이며 새 조정자 전체를 직접 검사한 것으로 확대 해석하지 않는다.
편집 세션 그림 이월의 모든 조건 조합은 이번 집중 테스트에서 직접 입증하지 못한다.

결과: **160건 통과 / 실패 0건**, 10 binaries, 필터 비선택 5,200건, exit 0.
빌드 5분 18초, 테스트 0.267초. 실행 ID: `6208f030-ba6e-44bb-ba8c-157ca193a970`.
로그: `output/7280/stage13/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행 중 같은 필터의
160개 PASS 이름과 일치함을 확인했다. 필터 비선택은 기존 ignore 50건과 별개이며,
이번 절편에서 전체 회귀를 재실행하지 않았다.

nextest 0.9.137(권장 0.9.140)과 observation profile 미사용 설정 경고는 기준 실행과 동일하다.
CI 도구 버전 일치나 전체 CI 통과를 주장하지 않는다. review worktree의 tracked 변경은 없고
파생 suite·manifest를 커밋하지 않았다. 제품 검증 후에는 계획과 결과 기록만 변경했다.

## 4. 후속

빈 문단의 조기 반환, 강제 저장 경계와 최종 fit 선택, 표 문단/컨트롤 흐름 분리를 계속한다.
R2 전체와 최종 통합 게이트는 남아 있으며, 이번 절편은 외부 push·PR 없이 로컬 커밋으로 닫는다.

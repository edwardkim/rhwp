# Task #7280 Stage 15 — R2m 전체 문단 fit 선택 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2l](task_m100_7280_stage14.md), 시작 head `b8b0cf557`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2m 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

이번 절편은 이미 계산된 강제 경계를 소비하는 전체 문단 fit 선택을 분리한다.
강제 경계 후보 chain은 각주/저장 줄 helper와 연결되어 있어 다음 절편으로 남긴다.

`paragraph/whole_fit.rs`는 원본/구성 결과와 제한된 페이지 관측으로 저장 마지막 줄 신뢰,
목록 꼬리 신뢰, 요구 높이, 되감김 및 Hangul 2024 재수용 근거를 반환한다. `available_height()`는
진단을 포함하므로 closure로 기존 단락 평가 위치에서만 호출한다. base 높이 관측은 부수효과가 없다.
그림 배제 영역을 포함한 점유 높이와 흐름 높이의 차이, full advance와 fit 높이의 차이,
저장 경계·회수량 조건·기존 상수는 보존하며 서로 합치지 않는다.

`paragraph::decide_whole_fit`은 조회 → 되감김 진단 → 해당 빈 문단의 spill 기록 → 기존 spill
관측 → fit 진단 → 전체 fit 선택 순서를 조정한다. spill 쓰기는 기존 state command를 재사용한다.
진단보다 앞에 쓰거나 fit 선택 이후로 미루지 않는다. 결과의 overflow 되감김은 기존 분할 진입에
전달하고, fits는 기존 전체 배치에 전달한다. fit 실패 뒤 overflow/분할 경로도 그대로 둔다.

소비 경로: 기존 저장 경계/fit 예산 → 읽기 전용 fit 근거 → 호환성 상태 기록 → 전체 fit 선택 →
기존 배치 또는 넘침 허용/분할. 배치 컷·높이·paint와 IR/API는 변경하지 않는다.
기존 heuristic의 타당성 승인이나 조판 수정이 아니며 테스트 assertion·ID, baseline·golden·ignore,
CI 정책도 변경하지 않는다.

## 2. 검증 계획

Query와 조정자의 결과/입력을 원래 block으로 복원 비교하여 조건·진단·spill 쓰기 순서를 확인한다.
별도 review worktree에서 파생 suite 준비 → fmt·고정 baseline 대비 manifest/unit-tier →
native Clippy → 집중 nextest를 순차 실행한다. 기존 문단 fit 계약과 저장 되감김/실제 float 점유
회귀를 포함하되 전체 호환성 조합의 직접 검증이나 한컴 시각 일치를 주장하지 않는다.

전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM/직접 시각 대조는 통합 게이트에
남긴다. 원격 push·PR·댓글은 수행하지 않는다.

## 3. 고정 head 검증

- 제품 SHA: `a10750069f2a04425bc5d6ed56c451cbc51470f9`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2m`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 복원 비교: `output/7280/stage15/verify-whole-fit.mjs` / `extraction-proof.json` 통과.
  fit 근거, 점유 높이/흐름 높이의 구별, 되감김 및 호환성 override 조건, 단락 평가 위치의
  가용 높이 조회, 진단과 spill 쓰기 순서, 나머지 parent와 기존 조정자 불변을 확인했다.
  DPI를 인자로 바꾸면서 rustfmt가 생략한 단일 표현식 closure의 중괄호만 명시적으로 정규화했다.
- manifest: 고정 baseline 대비 1,382 sources / 5,965 static attrs / 48 targets 통과.
- unit-tier: 고정 baseline 대비 4,205 tests / 298 modules / ready 0 / support 87 /
  white-box 4,114 / cfg support 28 통과.
- review worktree의 `cargo fmt --all -- --check` 통과.
- `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review
  -- -D warnings` 통과(exit 0, 55.65초).
- 로그: `output/7280/stage15/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

주 작업 폴더의 최초 `cargo fmt --all -- --check`는 이전 작업의 파생 suite가 없는 원본
`issue_7090_sibling_table_occupancy.rs`를 참조하여 실패했다. 기존 파생 파일을 임의로 고치거나
검사를 통과로 기록하지 않고, 고정 SHA의 별도 review worktree에서 `--prepare`로 준비한 뒤
위 포맷·정책 검사를 통과했다. 이 환경 오류 때문에 제품 코드나 테스트 계약을 변경하지 않았다.

집중 테스트(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_003 --test regression_suite_005 \
  --test regression_suite_011 --test regression_suite_012 \
  --test regression_suite_013 --test regression_suite_015 \
  --test regression_suite_022 --test regression_suite_026 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_6568_para_start_first_line_fit::) | test(issue_5755_rewind_overflow_page_break::) | test(issue_5921_neartop_reset_fits::) | test(issue_6753_lazy_base_keeps_trimmed_spacing_before::) | test(issue_6793_tac_host_page_tail_padding::) | test(issue_5870_empty_host_float_flow_advance::) | test(issue_6854_empty_para_orphan_page::) | test(issue_6970_multicolumn_synth_ladder_fill::) | test(issue_6855_float_band_page_fill_rewind::) | test(issue_6031_ladder_sb_omitted_tail_overrun::)' \
  --no-fail-fast
```

앞 절편의 166개 계약에 float 점유량을 이용한 저장 경계 처리 #6855의 두 계약과 문단 위 간격
누락 사다리의 본문 하단/다음 쪽 첫 줄을 보호하는 #6031의 한 계약을 추가로 선택했다.
테스트 source·assertion은 변경하지 않았다. 이 기존 계약들이 새 Query의 모든 조건 조합이나
Hangul 2024 호환성 spill을 직접 검증한다고 주장하지 않는다.
결과: **169건 통과 / 실패 0건**, 12 binaries, 필터 비선택 5,591건, exit 0.
빌드 5분 30초, 테스트 0.615초. 실행 ID: `c4e6fe58-05f4-413c-adb7-47ea96ebb7fc`.
로그: `output/7280/stage15/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행 중 같은 필터의
169개 PASS 이름과 일치함을 확인했다. 필터 비선택은 기존 ignore 50건과 별개이며,
이번 절편에서 전체 회귀를 재실행하지 않았다.

nextest 0.9.137(권장 0.9.140)과 observation profile 미사용 설정 경고는 기준 실행과 동일하다.
CI 도구 버전 일치나 전체 CI 통과를 주장하지 않는다. review worktree의 tracked 변경은 없고
파생 suite·manifest를 커밋하지 않았다. 제품 검증 후에는 계획과 결과 기록만 변경했다.

## 4. 후속

강제 저장 경계 후보 chain과 문단 조정의 잔여 책임, 표 문단/컨트롤 흐름 분리를 계속한다.
각주 helper와 연결된 강제 경계의 평가 순서와 지연 조회는 후속 절편에서 별도로 보존해야 한다.
R2 전체와 최종 통합 게이트는 남아 있다. 이번 절편은 원격 push·PR 없이 로컬 커밋으로 닫는다.

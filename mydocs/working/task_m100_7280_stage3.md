# Task #7280 Stage 3 — R2a 문단 결과·저장 줄 간격 조회 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 시작 head: `c6b79c260`. 제품 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2a 구현·절편 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임 경계

R2를 계산 결과/조회 → 구성/흐름 조정 순으로 나눈다. 이번 R2a는 실제 페이지 상태를
수정하지 않는 아래 책임을 옮긴다. 새로운 조판 규칙, IR, public API는 추가하지 않는다.

| 소유 파일 | 이동 대상 | 입력/소비자 |
| --- | --- | --- |
| `typeset/paragraph/metrics.rs` | `FormattedParagraph`, 기존 조회 메서드 5개 | 구성 결과, 원본 문단, 단 수, trim/dirty/lazy 관측값 → 기존 문단·표·미주 흐름 |
| `typeset/paragraph/stored_lines.rs` | 저장 사다리 간격·복원 가능성 helper 4개 | 원본 문단 배열, 인덱스, 해석된 스타일, dpi, 기존 경로 flag → 본문/후속 문단·미주 간격 판정 |

`metrics.rs`는 계획의 paragraph 결과 소유권을 구체화한 하위 파일이다. 전체 높이,
fit 높이, 줄 전진량을 합치거나 재정의하지 않는다. `FormattedParagraph` 생성과 상태 적용은
아직 typeset.rs의 기존 위치다. 결과 필드 접근은 typeset 범위만 허용하며 가변 페이지 상태
필드를 공개한 것이 아니다. 기존 테스트 편의를 위한 public API도 만들지 않는다.

저장 줄 조회는 `stored_ladder_encodes_spacing_before`, 그 내부 helper `stored_intra_line_gap`,
`spacing_trim_restorable`, `next_boundary_reverts_spacing_trim`이다.
내부 helper는 private이며 나머지 연결은 typeset 범위다. `ladder_spacing_omitted_signature` 등
구역 준비의 다른 저장 줄 판정은 아직 상위에 남아 있어 저장 줄 정책 전체 분리가 끝난 것은 아니다.

## 2. 보존 조건과 변경 관리

- 기존 함수 본문, 경험적 조건·상수·비교 연산, 호출 위치/순서를 보존한다.
- `flow_advance_height`의 환경 변수 기반 진단 출력도 보존한다. 이를 부수효과 없는 순수 함수로
  주장하지 않으며 IR·페이지 상태를 변경하지 않는 조회로 구분한다.
- 원본 문단의 참조 수명, fit/trim 재계산 시점, engine Cell/cache는 변경하지 않는다.
- 기존 cfg(test) 모듈·assertion·통합 계약과 baseline/ignore/pin을 변경하지 않는다.
- 기여자가 간격 판정을 변경하면 본문뿐 아니라 미주 및 표 뒤 문단의 소비 경로를 검토해야 한다.
  helper 삭제/대체 시 parent import, metrics 소비자와 아래 계약을 함께 확인한다.

기존 계약 연결: `renderer::typeset::`의 범위 밖 줄 인덱스/문단 분할, `renderer::composer::`,
endnote 측정 부수효과, #5801 저장 간격, #6031 간격 누락, #6753 lazy-base,
#6970 다단 합성 줄, #7196 다음 경계의 trim 복원 계약. 새 기대값은 만들지 않는다.

## 3. 검증 기록

검증 worktree: `/home/edward/mygithub/rhwp-review-7280-r2a`.
제품 commit: `cece33921d2bd12fc6f444437cbf58c763cdea73`.
suite 준비 후 Cargo 검증은 공유 `target/pr-review`에서 순차 실행한다.

- 이동 전후 본문 9개 및 나머지 typeset.rs 동일성:
  `output/7280/stage3/verify-extraction.mjs`, `extraction-proof.json`.
- manifest/source test 정책, fmt, 위 소비 경로 focused tests, native Clippy.
- 최종 통합의 전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM/시각 검증은 별도다.

정적 동일성 검사는 통과했다. 9개 본문은 문자열 동일하며, 결과 선언은 가시성·타입 경로와
formatter의 줄바꿈/후행 쉼표만 다르다. 이동/import를 제외한 typeset.rs 전체도 동일하다.
따라서 기존 호출부·테스트 본문은 변경하지 않았다. 이 검사는 실제 출력의 시각 판정이 아니다.

초기 제품 commit `178fc11a5`의 fmt 검사에서 import 줄바꿈 한 곳을 발견했다.
그 SHA로 시작했던 집중 빌드는 중단(exit 130)했으며 테스트 결과로 사용하지 않는다.
`cece33921`에서 수정 후 fmt·manifest·unit-tier를 재확인했고 모두 통과했다.

- manifest: 1,382 sources / 5,965 static test attrs / 28 suites + 20 exceptions.
- unit-tier: 4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support 28.
- 로그: `output/7280/stage3/{prepare,manifest-final,unit-tier-final,fmt-final}.log`.

집중 검증 명령(기본 feature, 로컬 nextest default profile):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked -p rhwp --lib \
  --test regression_suite_006 --test regression_suite_011 \
  --test regression_suite_012 --test regression_suite_020 --test regression_suite_026 \
  --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(endnote) | test(issue_5801) | test(issue_6031) | test(issue_6753) | test(issue_6970) | test(issue_7196)' \
  --no-fail-fast
```

위 suite 번호는 이번 SHA의 생성 결과에서 원본 계약 위치를 조회한 것이다. 이후 절편에서
고정 번호로 재사용하지 않는다. 로그는 `focused-final.log`다.

집중 테스트는 exit 0, **237건 실행 / 237건 통과 / 실패 0건**, 6 binaries다.
빌드 시간 5분 01초, 실행 시간 1.141초.
필터/기존 설정으로 제외된 4,661건은 이번에 실행하지 않은 범위이며 새 ignore가 아니다.
기준 전체 회귀의 제외 50건과 다른 범위다. 이번 결과를 전체 회귀 통과로 해석하지 않는다.

nextest 0.9.137은 저장소 권장 0.9.140보다 낮고, 사용하지 않은 observation profile의
`junit.report-skipped` 경고가 있다. 도구 버전까지 CI와 동일한 검증으로 주장하지 않는다.

같은 제품 SHA/worktree에서 집중 테스트 종료 후 native lint를 순차 실행하여 통과했다
(exit 0, 55.24초). 로그: `output/7280/stage3/clippy-native.log`.

```bash
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
```

검증 worktree의 tracked 변경은 없고 generated suite/manifest는 stage하지 않았다.
검증 후 변경은 완료 기록 문서뿐이다. WASM/workspace Clippy를 포함한 전체 lint 묶음과
전체 회귀·출력 비교는 미실행이며 최종 통합 게이트로 남아 있다. 원격 쓰기는 수행하지 않았다.

## 4. 잔여 작업

R2의 `format_paragraph_for_flow`, `typeset_paragraph`, `typeset_table_paragraph` 및 컨트롤/inline
흐름 조정은 아직 분리하지 않았다. 이들의 상태 변경 지점과 좁은 입력 경계를 이어서 정리한다.
거대 함수의 단순 파일 이동을 R2 완료로 판정하지 않는다. 원격 push·PR·댓글은 하지 않는다.

후속 경계 조사에서 `format_paragraph_for_flow`는 dpi 외에 `profile: Cell`,
`uniform_filler_ladder: Cell`, `float_carve_evidence: RefCell`을 읽으며 환경 변수도 조회한다.
다음 분리에서는 이 관측·borrow 시점과 저장/reflow 경로를 보존해야 한다. 전체 TypesetEngine을
하위 Query의 의존으로 넘긴 채 책임 분리가 끝났다고 보지 않는다.

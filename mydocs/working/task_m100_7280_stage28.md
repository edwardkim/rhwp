# Task #7280 Stage 28 — R2z 장식 표 배치·이어받기 준비 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2y](task_m100_7280_stage27.md), 시작 head `d3a0a31f0`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2z 구현·고정 제품 SHA 집중 검증 완료. R2 전체/PR 준비 완료가 아니다.

## 1. 책임과 보존 범위

일반 `typeset_table_paragraph`의 장식 표 단축 경로에서 실제 배치·이어받기 준비를 분리한다.
`controls/decoration_table.rs`는 기존 `FormattedTable`의 행 접두합과 문단 앵커로 현재 쪽 컷과
다음 쪽 예약량을 조회한다. `controls::place_decoration_table`은 발행·포맷·조회·진단·확정 순서를
소유한다. `state`는 Shape 발행과 기존 대기열/현재 단 컷/앵커 표시 반영만 수행한다.

R2y의 경로 선택, 부모의 host 텍스트 지연 플래그와 `continue`, 일반/TAC 표 배치·각주는
그대로 둔다. 다음 쪽에서 대기열을 인출하고 실제 조각을 이어 그리는 알고리즘은 이동하지 않는다.
새 public API·IR·PageItem 스키마·범용 rule engine을 만들지 않는다.

### 실행 순서와 의미 보존

- Shape 발행 → 현재 높이 `< 1.0` 조회 → 기존 `format_table` → 앵커/가용 영역 계산 순서다.
  포맷 callback은 부모의 기존 인자를 그대로 사용하며 상태 객체를 캡처하지 않는다.
  조회 함수에 엔진 전체나 가변 상태를 전달하지 않는다.
- 앵커는 기존 `current_height + hwpunit_to_px(vertical_offset as i32, dpi)`다.
  signed 해석·outer margin·새 기준 좌표로 치환하지 않는다. 가용 높이는 callback으로 받아
  앵커 계산 뒤에 읽는다. `FormattedTable`의 기존 접두합·effective height를 그대로 소비한다.
- `room > 0`, `effective_height > room`, 첫 초과 접두합의 `saturating_sub(1)`,
  `0 < first_unfit < ft.row_heights.len()` 조건을 유지한다. 첫 행도 못 넣는 경우나 초과가 없는 경우에
  새 이어받기를 만들지 않는다. 남은 높이의 기존 `.max(0.0)`도 바꾸지 않는다.
- 다음 문단의 첫 저장 vpos가 현재보다 작은 경우만 잔여 높이를 예약한다.
  다음 문단/첫 저장 줄 누락·같거나 증가하는 값은 기존처럼 예약 0이다. 새 보정이 아니다.
- 조회가 `Some`이면 기존 `RHWP_TABLE_DRIFT` 진단을 같은 값·문자열로 먼저 출력한다.
  그 뒤 `pending_overlay_continuations` → `current_column_overlay_cuts` 순서로 추가한다.
  `None`이어도 `overlay_shape_shortcut_para = Some(para_idx)`는 기존처럼 반영한다.
- 상태 명령은 `current_height`를 전진시키지 않는다. host 텍스트는 모든 컨트롤의 앵커와
  이어받기 범위가 확정된 후 R2x 경로에서 배치한다. 포맷 결과 수명도 이 단축 경로 안에 머문다.

기존 분기·수치의 타당성을 새로 승인하거나 피델리티를 개선한 것으로 보고하지 않는다.
테스트 source/assertion/ID, baseline/golden/ignore 및 CI는 변경하지 않는다.
포맷 알고리즘과 상태 저장소의 선언은 아직 부모에 있어 최종 캡슐화 완료가 아니다.

## 2. 검증 계획과 한계

`output/7280/stage28/verify-decoration-table.mjs`로 조회에서 원래 계산·분기를 복원하고,
발행/포맷/조회/진단/대기열/앵커 순서 및 포맷 인자를 별도 대조한다. rustfmt가 단일 식 closure의
중괄호를 추가·제거한 두 곳은 명시적으로 대응한다. 다른 부모 경로/테스트와 기존 controls/state
본문 불변도 확인한다. 정적 대조는 실행 검증이나 전체 시각 일치의 대체물이 아니다.

별도 review worktree에서 파생 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 집중 nextest를 순차 수행한다. R2y 273건에 다음 기존 2건을 추가한다.

- `issue_4514_overlay_table_flow` 1건: 실제 HWP의 46쪽 계약, 전 페이지 최상위 표 비겹침,
  ECR 구간의 연속 3쪽과 ECR-004의 앞 쪽 하단/바로 다음 쪽 상단 두 조각 존재.
  이어받기 누락·필러 흐름 대조군이다. 테스트의 한컴 참조와 이번 직접 시각 판독은 구분한다.
- `issue_5792_overlay_table_split_overlap` 1건: 실제 HWPX의 42행 보존과 후속 본문 줄의 비겹침.
  이후 #6366의 경로 선택을 거치는 입력이므로 이번 장식 표 조회만 격리 검증한 것으로
  주장하지 않는다. 비겹침 검출의 기존 90% 줄 높이 임계값도 유지한다.

예약 0/양수와 잘림 경계를 모든 합성 fixture로 새로 검증하지 않는다. 해당 세부 조합은 정적 보존
대조와 기존 테스트 범위 이상으로 실행 입증한 것이 아니다. 전체 회귀·WASM/workspace lint·
workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는 구현계획 §7의 통합 게이트에
남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

- 제품 SHA: `068cab7546cea42433cb7c094851d79b98491398`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2z`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 대조 통과: `output/7280/stage28/extraction-proof.json`.
- 파생 suite 준비 후 manifest 고정 baseline 비교 통과:
  1,382 sources / 5,965 static attrs / 48 targets.
- unit-tier 고정 baseline 비교 통과:
  4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support 28.
- `cargo fmt --all -- --check` 통과.
- native Clippy `-D warnings` 통과(exit 0, 57.28초).
- 로그: `output/7280/stage28/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

위 review worktree에서 다음 순서로 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage28/run-focused.sh
```

집중 명령은 [R2y의 명령](task_m100_7280_stage27.md#3-고정-head-검증)에서 worktree를 R2z로
바꾸고 `-E` 필터에 `test(issue_4514_overlay_table_flow::)`와
`test(issue_5792_overlay_table_split_overlap::)`를 OR 추가했다.
두 모듈의 suite 024/002는 기존 대상에 포함되어 있어 target은 그대로다.
`--locked --cargo-profile release-test --lib --no-fail-fast`와 고정 target도 동일하다.

결과: **275건 통과 / 실패 0건**, 24 binaries, 필터 비선택 7,934건, exit 0.
빌드 6분 42초, 테스트 1.582초. 실행 ID: `c6cc1796-a7b2-450a-a8a1-1db2b281a3f1`.
로그: `output/7280/stage28/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행에서 동일 필터로
선택한 275개 PASS 이름과 일치함을 확인했다. 이번에 전체 회귀를 재실행하지 않았으며,
필터 비선택은 기존 ignore 50건과 별개다. 출력 픽셀 동일성이나 전체 시각 일치 판정은 아니다.

nextest 0.9.137(권장 0.9.140) 및 observation profile 설정 경고는 기준 실행과 동일하다.
review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
검증 뒤에는 계획과 결과 기록만 수정했으며 제품 코드 변경은 없다.

## 4. 후속

일반/TAC 표의 포맷 이후 실제 배치 선택·host 소유/흐름 정산 등 남은 조정 책임을 분리한다.
R3 표 컷/이어받기 본체, 각주, 최종 상태 캡슐화와 전체 통합 검증은 남아 있다.

# Task #7280 Stage 39 — R2aj 후속 어울림 흐름 조정자 연결

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2ai](task_m100_7280_stage38.md), 시작 head `677732269`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2aj 구조 분리와 고정 제품 SHA 집중 검증 완료. R2 전체/PR 준비 완료가 아니다.

## 책임과 보존 계약

부모 `typeset_wrap_around_paragraph`는 호환 façade로 남기고 기존 후속 문단 흐름을
`controls/wrap_flow::place`로 이동한다. 매칭/그림 anchor는 wrap_match, 전체/접두 흡수는
wrap_absorption, 전폭 꼬리/매칭 실패 쪽 경계는 wrap_tail의 기존 Query를 호출한다.
등록·흡수·종료·꼬리 반영은 state의 기존 Command이며 새 조정자는 필드를 직접 읽거나 쓰지 않는다.

state의 좁은 읽기 메서드는 active/derived, 읽기 전용 layout, 현재 단 너비, 항목 존재와
단 수를 제공한다. `wrap_tail_needs_advance`는 **밴드 종료 후** 높이/가용 높이를 조회하고
빈 단에서는 `available_height` 진단을 실행하지 않는 원래 단락 평가를 보존한다.
범용 context·rule engine은 추가하지 않고 전체 state 필드 비공개화는 R5에 남긴다.

실제 호출 흐름은 구역 루프 → 기존 façade → wrap_flow다. 전체/접두 흡수 true는 호출자의
continue, 그림 등록 및 fallback false는 일반 문단 배치로 이어진다. 호출자와 profile은 불변이다.
꼬리 높이 생산(wrap_tail) → 종료 전 가용 높이 → 접두 기록 → 밴드 종료의 높이 반영 →
새 fit/필요 시 단 전환 → 꼬리 항목/높이 반영 순서를 그대로 유지한다.
포맷은 안정 형상 확인보다 먼저 수행하며, derived 읽기와 fit 진단의 지연 평가도 보존한다.
본문/미주 인덱스를 합치거나 표 분할·이어받기 알고리즘을 바꾸지 않는다.

기존 조건·상수·IR/API·테스트/assertion·baseline/golden/ignore·CI 변경은 없다.
이동한 기존 휴리스틱은 유지한 것이며 새로운 조판 규칙으로 승인한 것이 아니다.

## 검증 계획과 한계

`output/7280/stage39/verify-wrap-flow.mjs`로 state 접근자를 원본 식과 대조하고 새 조정자를
펼쳐 원래 함수 전체 및 그 밖의 parent/state/controls 불변을 확인한다. 정적 대조는
분기 커버리지·출력 픽셀 동일성·직접 시각 판정이 아니다.

고정 review worktree에서 suite 준비 → baseline 대비 manifest/unit-tier → fmt → native
Clippy → Stage38과 동일한 325건 집중 nextest → baseline PASS 이름 대조를 순차 수행한다.
모든 입력 조합의 분기 실행은 미검증이다. 주 checkout의 파생 suite는 변경하지 않는다.
전체 회귀·WASM/workspace lint/build·Native Skia·fresh Docker WASM·직접 출력 대조는
구현계획 §7의 책임 묶음/제출 전 게이트에 남긴다. 원격 push·PR·댓글은 범위 밖이다.

## 고정 head 검증

- 제품 SHA: `ebf162af3f4d551c678099b1fdcda44b58ccb453`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2aj`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
  host 16 logical CPUs / RAM 31 GiB, 시작 시 가용 약 19 GiB. 다른 Cargo 없음 확인.
- 정적 대조 통과: `output/7280/stage39/{verify-wrap-flow.mjs,extraction-proof.json}`.
- manifest: 1,382 sources / 5,965 static attrs / 48 targets 통과.
- unit-tier: 4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 /
  cfg support 28 통과.
- fmt 통과, native Clippy `-D warnings` 통과(exit 0, 56.60초).
- 로그: `output/7280/stage39/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

아래 명령을 고정 review worktree에서 순차 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage39/run-focused.sh
```

결과: **325 passed / 0 failed**, 24 binaries, 필터 비선택 7,884건, exit 0.
빌드 6분 48초, 테스트 2.809초. run ID: `23132c39-33ab-4344-89c9-7fc5890e44b9`.
정확한 선택·명령은 `output/7280/stage39/run-focused.sh`, 실행 로그는 `nextest-focused.log`,
baseline PASS 이름 대조는 `compare-focused.mjs`와 `regression-comparison.json`에 보존했다.
Stage38과 동일한 기존 325건이며 baseline 전수 로그의 같은 선택 이름들과 일치한다.
비선택 수는 기존 ignore 50건과 별개다. 전체 회귀나 직접 시각 일치의 증거는 아니다.
nextest 0.9.137 권장 버전 및 observation 설정 경고는 이전과 동일하다.

review worktree는 tracked 변경이 없으며 파생 suite/manifest는 커밋하지 않았다.
제품 SHA 이후 코드 변경은 없고 결과 기록과 구현계획만 갱신했다. 두 Markdown의 로컬 파일
링크 존재 및 `git diff --check`를 확인했다. 원격 작업은 실행하지 않았다.

## 후속

Stage34에서 R2로 분류한 `native_hwp5_square_picture_next_page_owner`의 지연 그림 후보
Query가 부모에 남아 있다. 다음 절편은 이 후보 조회와 필요한 상태 관측 경계의 분리이며,
큐 materialize·페이지 전이는 R5에 유지한다. 이후 R2 책임 묶음 통합 검증으로 이어간다.
R3 표 분할/이어받기,
R4 각주/미주 본체, R5 구역 수명/최종 상태 캡슐화와 R6 기여자 안내는 별도 책임으로 유지한다.

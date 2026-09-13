# #3587 Stage 3 — A 통합 검증과 B 설계

- 일자: 2026-09-12
- 상태: **A 통합 Rust 검증 완료: 전체 nextest 9,523 PASS / 0 FAIL / 46 skipped.
  B 상세 계획 작성 완료·승인 대기. #3587 전체는 미완료.**
- 승인: 메인테이너의 「다음 절차 진행을 승인합니다」에 따라 A 통합 검증과 B 상세 계획 작성.
- 검증 HEAD: `04fd9bcd57` (제품 `15e13c7960`, 최종 A 테스트 `3fc090bc76` 이후 문서만 추가)
- source: `task_m100_3587`, 검증: 별도 `rhwp-review-3587` detached worktree.
- [Stage 2](task_m100_3587_stage2.md), [구현계획](../plans/task_m100_3587_impl.md),
  [B 상세 계획 — 승인 요청](../plans/task_m100_3587_impl_b.md)

## 1. 범위와 환경

A1~A4의 표 생성·복제/붙여넣기·분할 신원과 Form/ClickHere 저장 보정을 함께 검증한다.
B 제품 구현, 원격 push/게시/PR, 기여자 시나리오 완료 판정은 이번 실행에 포함하지 않는다.

- WSL2 Linux, Rust 1.93.1, 16 logical CPU, RAM 31 GiB(시작 시 available 28 GiB).
- 시작 시 디스크 가용 362 GiB, 기존 고정 `target/pr-review` 105 GiB. 캐시 삭제/이동 없음.
- 기존 Cargo 실행 없음 확인 후 순차 실행. 전체 nextest 동시성은 8로 제한하여 컴파일 및
  자체 worker를 쓰는 코퍼스 검사와 메모리를 공유한다.
- 생성 suite는 review worktree에서만 준비한다. source의 Cargo/파생 suite는 바꾸지 않는다.
- 로그는 Git 제외 `output/3587/a-integrated/`에 보관한다.

## 2. 명령과 결과

review worktree에서 `--target-dir`는 동일 저장소의 고정 `target/pr-review` 절대 경로를 사용한다.
각 명령은 앞 명령의 종료 후 실행한다.

| 검사 | 현재 결과 | 로그 |
| --- | --- | --- |
| `node scripts/rust-test-suite-manifest.mjs --prepare` | PASS, 1267 sources / 48 targets | prepare.log |
| `cargo fmt --all` 및 `cargo fmt --all -- --check` | PASS, tracked source 변경 없음 | fmt.log |
| `cargo clippy --locked -- -D warnings` | PASS, 57.12초 | clippy-native.log |
| `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | PASS, 53.29초 | clippy-wasm.log |
| `cargo build --locked --workspace` | PASS, 1분 30초 | build-workspace.log |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | PASS, 1분 20초 | clippy-workspace.log |
| `node scripts/rust-test-suite-manifest.mjs --check` | PASS | manifest.log |
| `cargo nextest run --locked --cargo-profile release-test --tests --test-threads 8 --no-fail-fast` | **9,523 PASS / 0 FAIL / 46 skipped** | nextest.log |

전체 회귀는 A의 focused 45건과 달리 기존 전체 계약·코퍼스 래칫을 포함한다.
78 test binaries에서 9,523건을 실행했으며 이 안에 신규 A 계약 25건이 모두 포함된다.
컴파일 4분 3초, 테스트 410.718초(6분 50.718초)였다. 46 skipped는 실행 통과 수에 포함하지 않는다.
3건의 slow 알림은 최종 모두 PASS다. IR field sweep, overflow cell, text overlap, off canvas와
기존 HWP 저장 왕복 검사가 통과했다. 이번에는 실패가 없어 제품·baseline·fixture 보정을 하지 않았다.

다음 dump 환경변수를 함께 지정했다: `RHWP_IR_SWEEP_DUMP`, `RHWP_OVERFLOW_CELL_DUMP`,
`RHWP_OFF_CANVAS_DUMP`, `RHWP_TEXT_OVERLAP_DUMP`. 출력은 같은 로그 디렉터리의
`ir-sweep.tsv`, `overflow.tsv.partNN-of16`, `off-canvas.tsv.partNN-of16`,
`overlap.tsv.partNN-of16`이다. dump는 검사기 자체 형식이며 모든 정상 문서 목록을 뜻하지 않는다.

nextest 0.9.137은 저장소 최소 0.9.91 이상이나 권장 0.9.140보다 낮다는 경고와
`report-skipped` 설정 키 무시 경고를 출력했다. 도구/정책을 변경하지 않았으며 이 경고를
제품 회귀로 분류하지 않는다. 새 B 계획과 이 기록의 파일 단위 Markdown 링크 검사도 통과했다.

## 3. B 설계에서 확정해야 할 지점

[B 상세 계획](../plans/task_m100_3587_impl_b.md)에 기존 함수 근거와 파일별 변경을 기록했다.

1. 기존 대화형 paste의 양끝 merge/후속 빈 문단 생성과 분리한다.
2. 문단 원형은 한 번 고정하고 요청 단위 ID allocator·사본 단위 참조 map을 사용한다.
3. 모든 반환 가능한 오류를 실제 삽입 전에 처리한다. batch/snapshot을 rollback 대용으로 쓰지 않는다.
4. 앞뒤 문단의 의미 구조를 보존하되 정상 reflow 좌표 이동은 허용한다.
5. 지원 불명 컨트롤·경계 밖 필드/연결선은 strict API에서 명시적으로 거부한다.
6. 복제 예산과 대응표·원문 ID 순회 예산을 함께 제한한다. 수치 제안과 실측을 구분하며
   기존 실물 자료를 재사용한 작은 B1 계측으로 비용 누락을 검증한다.

## 4. 완료로 주장하지 않는 것

- A focused 성공은 새 한컴 편집기 시각 판정이 아니다. 기여자의 실제 원형/완성본은 미확보다.
- 이번 native/WASM Clippy는 최적화 Docker WASM 빌드나 브라우저 사용 성공이 아니다.
- renderer/조판 소스·Studio bridge는 A에서 바꾸지 않았다. A 통합의 Rust 기본 게이트는
  model/편집/저장 변경에 맞춰 수행한다. Docker WASM·실물 복제 결과의 시각 확인은 별도 남은
  사용 검증으로 기록하며, C 공개 API/최종 제출 전에 관련 게이트를 충족해야 한다.
- B 계획은 승인 요청이며 구현 완료도, B 진입 승인도 아니다. #3587 전체는 미완료다.

## 5. 다음 절차

1. [B 상세 계획](../plans/task_m100_3587_impl_b.md)의 범위·예산·실패 계약 승인.
2. 승인 후 B1 계약/방어 검사 → B2 원자적 복제 → B3 저장/사용 검증 순서로 진행.
3. A의 새 Docker WASM·실물 복제 시각 확인은 남은 사용 검증으로 명시하여 유지한다.
   이번 결과를 전체 A의 한컴 업무 호환성 또는 #3587 최종 제출 완료로 확대하지 않는다.

현재 검증 SHA 이후 변경은 계획·작업 기록·오늘할일 문서뿐이다. 리뷰 worktree의 tracked 변경은
없으며, generated suite/manifest는 검증용 ignored 산출물로만 남겼다. 원격 작업은 하지 않았다.

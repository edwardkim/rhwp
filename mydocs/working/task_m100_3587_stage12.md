# #3587 Stage 12 — C2 완결 행 복제·채우기

- 승인: 메인테이너 「다음 절차 진행을 승인합니다」, 2026-09-13.
- 기준: `3059a9d01`, `task_m100_3587`.
- 계획: [C 상세계획](../plans/task_m100_3587_impl_c.md) §4.
- 상태: C2 native 구현·집중 119건·Rust lint 완료. 2026-09-13 메인테이너 실물 판정 성공. C3 진입 또는 #3587 완료 선언이 아니다.

본문 최상위 표의 완결된 행 범위를 원형으로 선택하고 복사본별 값을 채운다.
기존 빈 행 삽입은 경계 병합을 확장하고 빈 셀을 만드는 별도 동작이므로 호출하지 않는다.
원형·삽입 경계를 가로지르는 병합/영역 속성, 제목 셀의 데이터 행 반복, 손상된 격자는
변경 전에 거부한다. 행 수와 영역 속성 수의 UINT16 한계 및 작업 예산을 함께 검사한다.

복제 대상 셀만 가진 내부 projection을 기존 소유 구조·참조·자원 검사와 채우기 helper에
연결한다. projection의 임시 표 자체는 복제 산출물에 넣지 않는다. 셀의 내부 문단·컨트롤은
새 ID로 재발급하고, 기존 표와 원형의 ID는 유지한다. 전체 문서 복제나 JSON 왕복은 하지 않는다.
대상 표의 소유 문단을 staging하여 기존 표 재계산 표시를 이관하고 마지막에 한 번 반영한다.

`RepeatTableRowsRequest`의 binding 경로는
`Paragraph(0)/Control(0)/Cell(n)/Paragraph(p)/...`이다. `n`은 원형에 포함된 셀을 행·열
앵커 순으로 정렬한 상대 인덱스다. 복제 결과는 변경 전 실제 경로와 변경 후 실제 경로의
대응표를 반환한다. 편집 후 이 경로를 영구 ID처럼 재사용하면 안 된다.

이번 검증은 native 집중 계약과 Rust lint다. C 종료 전체 회귀·WASM/CLI/MCP·비용 계측과
실제 한컴 시각 판정은 아직 남아 있다. #7065와 #7084는 별도 이슈로 유지한다.
원격 push·PR·댓글은 수행하지 않는다.

## 중간 검증과 보완

- `19e7970e8`: 기존 108건 + 행 복제 8건 = **116 PASS**, 1,996 비대상 skipped.
  `output/3587/c2-rows/contracts.log`에 기록했다. 새 표 원형·내부 문단·필드 ID,
  닫힌 병합, 영역 속성 재매핑, 오류 무변경, 실물 양 형식 저장·재열기를 검사했다.
- 추가 경계: 데이터 행을 제목 블록 앞/내부에 삽입해 기존 제목 반복 의미가 달라지는 요청을
  거부한다. 높이 미지정은 기존 실제 행 높이 합, 늘린 표는 기존 표시 행 높이를 유지하고
  복제 행 높이를 더한다. 포화 연산 대신 overflow를 거부한다.
- 준비 함수는 `&DocumentCore`에서 완전한 staging과 결과를 만들고, command만 마지막 반영을
  수행하도록 분리했다. C3 dry-run 연결은 아직 하지 않았다.
- 첫 검증 준비 명령의 작업 디렉터리를 잘못 지정하여 같은 파일 복사/누락 경로 오류가 있었다.
  제품 테스트 실패가 아니며 복사 경로를 절대 경로로 고친 뒤 suite를 준비하여 위 검증을 실행했다.

최종 보완본 검증 결과는 아래에 별도로 기록한다. 중간 116 PASS를 후속 코드의 통과로 재사용하지 않는다.

## 보완본 집중 검증과 실물 파일

- 제품·테스트 기준: `934c961f4`. `31cc54ae2`의 보완에 rustfmt만 반영했다.
  포맷 명령의 비동기 완료 전에 review HEAD를 이동하려던 첫 시도는 dirty 보호로 거부되었다.
  포맷 완료와 main/review byte 일치를 확인하고 포맷 commit 후 clean detached HEAD를 맞췄다.
- `contracts-final.log`: **119 PASS**, 2,380 비대상 skipped. 기존 108건과 신규 행 계약 11건이다.
  공개 native API를 실행하여 결과 값·행 구조·ID·원형 보존·저장·오류 무변경을 검사했다.
  nextest 0.9.137/권장 0.9.140 및 JUnit unknown key 경고는 기존 환경 경고다.
- 원본 `samples/rnote/labnote-001.hwp`의 section 0 / pi 12 / control 1 표는 34행×1열이다.
  0 기반 row 5를 유지하고 그 뒤에 같은 행을 2회 복제, 서로 다른 두 줄의 기록을 채워 **36행**으로 저장했다.
  앞뒤 표를 복사하거나 원형 셀을 채운 결과가 아니다. 한컴 시각 판정은 아직 요청 전이다.

| 파일 | 크기 | SHA-256 |
| --- | --- | --- |
| `output/3587/c2-rows/labnote-001-stage12-rows.hwp` | 9,728 B | `0d7965c28e1151aca04a781fc41bff872074202b571ef6102457bb3b788e623d` |
| `output/3587/c2-rows/labnote-001-stage12-rows.hwpx` | 9,077 B | `5c8edd2018c3744851cfcb20a903ad5c0da400da31a4584bc09e849fdfd3dd94` |

내보내기는 `export-rows.rs` local driver가 같은 native API를 호출했다. 생성 바이트 재열기에서
행 수·원형 빈 값·복제본 두 값·기존 표 ID를 검증하고, 파일 저장 후 바이트 일치와 입력 무변경을 확인했다.
기존 파일을 덮어쓰지 않도록 `create_new`를 사용했다. 산출물과 driver는 output 아래 로컬 증적이다.
WASM 파일은 갱신하지 않았으므로 기존 Studio 로드본과 이번 native 코드 SHA를 동일시하지 않는다.

## 음성 대조

review worktree에서만 live 행 대입을 생략한 변이를 실행했다. 신규 11건 중 **7 FAIL / 4 PASS**
(172 비대상 skipped, exit 100)로 복제 행·값·병합·ID·높이·실물 저장 누락을 검출했다.
거부/무변경 계약 4건은 통과했다. `negative-no-commit.log`에 남겼다.
변이는 커밋하지 않았고, main의 검증 원본을 복사해 복원한 후 review `git diff --exit-code`를 통과했다.
복원본 재검사에서 **11 PASS**를 확인했다 (`restored-green.log`, 172 비대상 skipped).

## 최종 자동 검증

모두 `/home/edward/mygithub/rhwp-review-3587`에서 실행했다.
Cargo는 `--locked --target-dir /home/edward/mygithub/rhwp/target/pr-review`를 공유하며 순차 실행했다.
제품·테스트 source는 `934c961f4`이며 이후 변경은 계획·결과 문서뿐이다.

| 검사 | 결과 | 로컬 로그 |
| --- | --- | --- |
| 기존 A/B/C1 + C2 focused nextest | 119 PASS / 2,380 비대상 skipped | `contracts-final.log` |
| 반영 누락 음성 대조 | 7 FAIL / 4 PASS, 예상한 누락 검출 | `negative-no-commit.log` |
| 복원 후 C2 nextest | 11 PASS / 172 비대상 skipped | `restored-green.log` |
| `cargo fmt --all -- --check` | PASS | `fmt.log` |
| native root Clippy `-- -D warnings` | PASS, 42.38초 | `clippy-native.log` |
| `-p rhwp --lib --target wasm32-unknown-unknown` Clippy | PASS, 37.86초 | `clippy-wasm.log` |
| `cargo build --workspace` | PASS, 1분 11초 | `workspace-build.log` |
| `--workspace --all-targets` Clippy | PASS, 1분 3초 | `clippy-workspace.log` |
| generated manifest `--check` | PASS, 1,277 원본 / 48 target | `manifest.log` |
| `node --test scripts/tests/rust-test-suite-manifest.test.mjs` | 23 PASS | `suite-policy.log` |
| 변경 문서 4개 링크 검사 | PASS | `docs-links.log` |

nextest는 `scripts/run-rust-test.mjs::resolveCase`로 `issue_3587_*` 원본 14개의 suite를 해석하고
`-E 'test(issue_3587_)' --no-fail-fast --cargo-profile release-test`로 실행했다.
음성/복원 대조는 `issue_3587_template_rows` 원본과 `test(issue_3587_template_rows::)`만 선택했다.
generated suite/manifest와 output 증적은 source commit 대상에서 제외했다.

## 메인테이너 판정과 다음 순서

두 내보내기 파일에서 **본문 표의 기존 6번째 행 뒤 새 두 행**에 서로 다른 기록이 들어갔는지,
기존 행과 뒤쪽 양식이 보존되는지 메인테이너에게 확인을 요청했다.
2026-09-13 메인테이너가 **「판정: 성공」**으로 C2 산출물의 시각 판정을 승인했다.
이 기록은 자동 재열기 결과와 별개의 메인테이너 판정이며, 새 WASM API 실행이나 C 전체 검증의
완료를 뜻하지 않는다. 검사용 projection은 복제 셀만 담은 임시 원형 사본이고,
staging은 원본 반영 전에 준비한 결과다. 이 둘 자체가 파일의 추가 표로 저장되지는 않는다.

이후 승인된 C3에서 WASM·CLI run·MCP와 공통 dry-run을 연결하고 C 종료 전체 회귀·Native Skia·
Docker WASM·실물 판정·1/10/100회 비용 계측을 수행한다. 이번 focused 119건과 WASM Clippy는
전체 회귀 또는 새 브라우저 WASM 배포를 의미하지 않는다. D/Gym도 별도 남은 단계다.

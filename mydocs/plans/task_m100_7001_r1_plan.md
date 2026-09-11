# #7001 참고 제안 — 선택형 개발 실행 도구 패키지 분리 (미승인·미착수)

- 작성일: 2026-09-11 (KST)
- 근거: [#7001 최종 분석](../report/task_m100_7001_report.md), 메인테이너의 결과·R1 우선 방향 승인
- 승인 기록: `26bf57340`
- 기준 제품 SHA: `313bd4273dc0cc36a3f3f9b01425797636601b61`
- 상태: 메인테이너의 조사 전용 범위 재확인에 따라 구현 승인 요청을 철회했다. 참고 제안으로만 보존한다.
- 이슈 경계: #7001에서는 실제 구현·후속 이슈 등록·구현 브랜치 착수를 하지 않는다.
  파일명은 기존 링크 보존을 위해 유지했으며 활성 수행계획을 의미하지 않는다.

> 아래 내용은 당시 작성한 미승인 설계 제안이다. 명령·단계·검증 계획은 실행되지 않았으며
> 이번 PR의 구현 범위나 완료 조건이 아니다. 향후 메인테이너가 별도 작업을 지시할 때만 재검토한다.

## 1. 완료 목표

루트 `rhwp` 패키지에는 제품 실행 타깃 `rhwp`만 두고, 기존 보조 실행 타깃 25개는
같은 저장소의 `tools/rhwp-devtools` 패키지에서 명시적으로 빌드·사용한다.
**명령을 없애는 작업이 아니라 타깃 소유권과 선택 경계를 바꾸는 작업**이다.

완료 상태는 다음과 같다.

1. 루트 패키지 bin 목록은 `rhwp` 1개, 개발 도구 패키지는 기존 이름의 25개다.
2. 제품 기본 빌드·소스 설치가 개발 도구를 기본 대상으로 선택하지 않는다.
3. 개발 도구 사용자는 명시적 패키지 선택으로 기존 명령을 실행할 수 있다.
4. 기존 scripts·계약 테스트가 같은 입력·출력·주소·실패 계약으로 동작한다.
5. 로컬뿐 아니라 nextest archive를 받는 별도 CI worker에서도 도구를 실행할 수 있다.
6. 제품 CLI·라이브러리·WASM에 개발 도구/Gym 필수 의존이 생기지 않는다.

현재 release workflow는 이미 `rhwp`만 포장한다. 배포 파일에 도구가 들어 있었다가
이번에 제거되는 것처럼 보고하지 않는다. 새 스토어·registry 배포는 하지 않는다.

## 2. 기준 확인과 기존 자료 재사용

2026-09-11 `git fetch upstream` 후 원격 devel이 위 기준과 같음을 확인했다.
착수 직전 다시 확인하고, 변경되었다면 대상 경로·Cargo·test·CI 차이만 먼저 대조한다.
분석 기준의 inventory를 최신 소스 결과처럼 덮어쓰지 않는다.

- [Stage 1 타깃·소비자 목록](../working/assets/issue7001/README.md): 26개 타깃과 등록 명령 목록.
- [Stage 2 계약 대조](../working/task_m100_7001_stage2.md): 변하면 안 되는 명령 의미와 차이.
- [최종 보고서](../report/task_m100_7001_report.md): R1/R2/R3 경계와 선택 이유.

전체 코퍼스·Gym 전건을 다시 측정하지 않는다. R1 착수 시 선택 타깃의 Cargo metadata,
기존 테스트 목록과 실제 바이너리 출처를 고정하고 필요한 계약 실행만 추가한다.

## 3. 패키지와 파일 이동 설계

| 변경 위치 | 계획 |
| --- | --- |
| 루트 `Cargo.toml` | `autobins = false`, 제품 bin만 명시. font-metric-gen 타깃은 새 패키지로 이관 |
| `tools/rhwp-devtools/Cargo.toml` | `publish = false`, `autobins = false`, 기존 이름 25개를 `[[bin]]`로 명시 |
| workspace | 새 패키지를 `members`에 추가하되 `default-members`에는 추가하지 않음 |
| `src/bin/rhwp-agent/**` | `tools/rhwp-devtools/agent/**`로 이관 |
| `src/bin/rhwp-q-*.rs`, q-kit/more/pack 디렉터리 | `tools/rhwp-devtools/queries/` 아래 기존 basename·내부 구조 유지 |
| `src/tools/font_metric_gen.rs` | `tools/rhwp-devtools/font-metric-gen/font_metric_gen.rs`로 이관 |
| `Cargo.lock` | 새 workspace 패키지 관계만 반영. 관련 없는 dependency upgrade 금지 |

개발 도구의 직접 의존 crate는 실제 `use`·macro·cfg를 대조해 명시한다. 제품 라이브러리는
경로 의존으로 사용하되 제품 → 개발 도구 역방향 의존은 만들지 않는다.
기존 root dependency는 제품 소비가 없음을 확인한 것만 제거 후보로 기록하며, R1에 무리하게
dependency 정리까지 포함하지 않는다. 제품 features·rlib/cdylib 순서와 DSEL 공개 API는 유지한다.

이동 전후 Rust 내용이 동일한 파일과 경로 조정이 필요한 파일을 나누어 diff를 검토한다.
`font_metric_gen.rs`는 자신의 소스 bytes를 해시 재료로 사용하므로 basename과 파일 bytes를
그대로 유지하는 것을 원칙으로 한다. 실행 안내는 새 패키지 README에서 설명한다.
이를 이유로 폰트 메트릭 DB·기대값·기존 provenance를 재생성하지 않는다.

예상 사용 형태(아직 동작 검증 전):

```bash
cargo build --locked -p rhwp --bin rhwp
cargo build --locked -p rhwp-devtools --bins
cargo run --locked -p rhwp-devtools --bin rhwp-agent -- --help
cargo run --locked -p rhwp-devtools --bin font-metric-gen -- --help
```

마지막 두 예시의 help 지원·종료 코드는 실제 기존 명령 계약으로 확인한다.
25개 모두에 같은 `--help` 동작이 있다고 가정하지 않는다.

## 4. 테스트는 루트에 보존하고 실행 파일을 명시적으로 공급

선택안은 **기존 integration source·suite 소유권을 루트에 유지**하는 방식이다.
테스트를 새 패키지 아래로 대량 이동하거나 새 harness 배정 정책을 만들지 않는다.
새 integration 원본은 기존 규칙대로 `tests/cases/`에만 작성한다.

문제 지점은 `env!("CARGO_BIN_EXE_rhwp-agent")` 및 q/font 타깃의 컴파일 시 경로다.
별도 패키지로 옮긴 실행 파일을 루트 테스트에 Cargo가 자동 공급한다고 가정할 수 없다.
다음 방법으로 연결을 바꾼다.

1. 검증 준비 wrapper가 같은 checkout·Cargo profile·target-dir에서 개발 도구를 먼저 빌드한다.
2. Cargo의 빌드 결과에서 실제 executable 경로를 수집한다. PATH나 과거 target 파일을 검색해
   성공으로 대체하지 않는다. 25개 이름·실행 파일 존재와 source SHA/profile을 확인한다.
3. 테스트의 기존 런타임 `CARGO_BIN_EXE_*` override 지원은 유지하되, 삭제된 타깃의
   컴파일 시 `env!` fallback은 명시적으로 공급한 개발 도구 디렉터리로 대체한다.
   예시 환경변수 이름은 `RHWP_DEVTOOLS_BIN_DIR`이며 구현 시 한 곳에서 정의한다.
4. 테스트 지원 코드는 `tests/support/`의 기존 관례를 우선한다. 새 지원 코드의 module 포함이
   suite generator의 blocker가 되는지 먼저 확인하고 임의로 registry·예외를 늘리지 않는다.
5. 도구·fixture·설정 누락은 경로와 준비 명령을 포함한 실패로 알린다. 테스트를 skip하거나
   조용히 return하지 않는다. 예상 테스트 수와 실제 실행 수가 감소하지 않는지 대조한다.

루트 `CARGO_MANIFEST_DIR`의 샘플 경로는 유지할 수 있지만, 이전 `src/bin` 소스를 직접 읽는
테스트와 `include_*` 경로는 이관된 경로로 맞춘다. helper의 오류와 누락/잘못된 경로는
별도 계약으로 검증한다. wrapper 안에서 secret·비공개 문서 정보를 기록하지 않는다.

## 5. CI archive 전달까지 같은 변경에서 처리

현 `.github/workflows/build-nextest-archives.yml`은 `--package rhwp`로 archive를 만들고,
`run-nextest-archives.yml` worker는 `--workspace-remap` 후 이를 실행한다.
builder에서 별도 도구를 빌드했다는 사실만으로 worker에 그 파일이 전달되는 것은 아니다.

R1에서는 다음 경로를 계획한다.

- integration archive builder에서 같은 source SHA·profile로 만든 개발 도구 묶음을 함께 보존한다.
  별도 패키지 bin이 nextest에 자동 수록·자동 remap된다고 가정하지 않는다.
- 기존 test artifact에 **상대 경로 기반 도구 묶음과 검증 manifest**를 함께 전달하는 방식을 사용한다.
  worker는 전용 디렉터리에 풀고 이름·해시·source SHA/profile이 맞는지 확인한 뒤 경로를 주입한다.
- archive label A의 lib 검증에는 불필요한 도구 빌드·다운로드를 추가하지 않는다.
  B/C/D에서는 worker별 재빌드 없이 builder 산출물을 재사용한다.
- 기존 artifact 선택의 run ID·label, testcase 배타적 배정, expected/run 총수 대조를 보존한다.
  별도 worker로 경로를 바꿔 실행하는 로컬 archive 재현을 제출 전 필수로 한다.
- PR head 산출물은 기존 비특권 검증 worker 안에서만 실행한다. privileged Controller나
  write token을 가진 후행 job으로 실행을 옮기지 않는다. artifact 압축 해제 경로 이탈도 거부한다.

운영 등급은 **O3 — 실행·artifact 공급 계약 변경**이다. 새 workflow·required check·권한·
timeout·runner를 늘리지 않으며 Gym gate도 추가하지 않는다.
`ci.yml`의 전체 workspace lint는 새 패키지를 계속 검사한다. 영향 분류와 CodeQL이
새 `tools/` 경로의 Rust 변경을 빠뜨리지 않는지 정책 테스트로 확인한다.

정책 검증에는 `scripts/tests/test_nextest_archive_workflow.py`와 관련 artifact/helper 계약을
포함한다. job 이름과 aggregate 성공 조건은 유지한다. artifact 크기·빌드 시간은 구현 전후
같은 조건에서 관찰하되, 미측정 절감률을 목표값으로 선언하지 않는다.

## 6. 소비자·문서 보정과 명시적 비범위

실제 소비자를 기존 inventory에서 찾아 Cargo 명령에 `-p rhwp-devtools` 선택을 추가하고
실행 경로만 조정한다. 우선 확인 대상은 폰트 coverage, kerning Q1/Q2, rank8 trace·qualification
scripts와 agent/q/font 계약 테스트다. private corpus 전수 실행은 요구하지 않는다.

활성 사용 매뉴얼·개발 환경 안내·새 패키지 README에 제품용/개발 도구용 명령을 구분한다.
과거 보고서의 역사적 경로는 일괄 치환하지 않는다. `src/agent`의 미래 커널 설명을 고치는 일도
R1의 타깃 이관에 필요하지 않으면 보류한다.

다음은 변경하지 않는다: 본 CLI의 102개 명령, JSON/출처 표지·종료 코드, verify/search 의미,
scan/explore 처리, q 봉투 공통화, volume-probe 계산, DSEL API, 조판 엔진, Gym 채점,
폰트 DB, 공개 배포 채널. R2·R3 작업을 R1 완료 조건으로 추가하지 않는다.

## 7. 수행 순서와 승인 지점

| 단계 | 수행 범위 | 완료·승인 지점 |
| --- | --- | --- |
| 준비 | 후속 이슈 1건·최신 devel 기반 번호 브랜치, 대상·소비자 차이와 기존 계약 실행 기준 고정 | 본 계획 승인 뒤 착수. 기존 분석 자료 재사용 |
| 구현 | 패키지 이관 + 테스트 경로 공급 + archive 전달 + 활성 사용 안내를 하나의 변경 집합으로 구성 | focused·누락 반례·다른 경로 archive 재현 결과 보고 |
| 통합 검증·제출 | 전체 Rust gate·제품 설치/WASM 경계·CI 정책 검증, 최신 base 충돌 확인 | 전체 검증/PR 준비 승인 후 수행, push·PR 별도 승인 |

파일 이동만 끝난 상태를 제출 가능한 완료로 선언하지 않는다. 반대로 도구별 25개 PR·하위 이슈로
쪼개지 않는다. 실제 구현을 시작한 뒤 단계 결과는 구현 이슈 번호의 문서에 기록한다.
검증이 준비 경로 자체의 설계 변경을 요구하면 우회·예외 추가 전에 그 근거로 수정 승인을 받는다.

## 8. 검증과 실패 처리

구현 전후 검증은 [로컬 검증 정본 4.3](../manual/pr_review/local_validation.md)과
[CONTRIBUTING](../../CONTRIBUTING.md)의 범위별 절차를 따른다.

- 정적 경계: Cargo metadata의 루트 1개/도구 25개, 기존 이름·명령 목록 일치,
  제품 → devtools/Gym 의존 없음, 기본 멤버 선택, 소스 bytes/필수 경로 조정 diff.
- 계약: agent/q/font 타깃과 실소비 scripts의 관련 focused 테스트, 실패 경로, source SHA/profile 불일치,
  worker에서 도구 누락, archive 경로 변경, fixture 누락을 명시적으로 확인한다.
- 전체 Rust: review worktree에서 suite prepare → 전체 fmt → native Clippy → WASM32 lib Clippy →
  workspace build → workspace all-target Clippy → 해당 focused 및 release-test 전체 integration →
  manifest check. source-side test 변경 시 tier policy 검사도 수행한다.
- 정책: integration suite generator 계약, 도구 공급 helper/빌드·실행 archive 계약,
  변경에 해당하는 CI impact/CodeQL 경로 계약. generated suite·manifest는 커밋하지 않는다.
- 제품 경계: 격리된 설치 경로에서 제품 설치와 파일 목록 확인, 실제 제품 실행,
  Docker WASM 빌드. 사용자의 설치물·Studio dev 서버·공유 캐시를 덮어쓰지 않는다.
- 공개 샘플과 고정 입력을 사용한다. 기대값을 새 구현 출력으로 덮어쓰지 않으며,
  렌더러를 변경하지 않은 R1에 전체 시각 전수 계측을 의례적으로 추가하지 않는다.

테스트 공급 또는 패키지 이관으로 실패하면 해당 원인을 고친다. test skip·feature 비활성화·
required check 완화·Gym bypass·timeout 증가로 통과시키지 않는다.
동작 변경이 필요한 경우 R1에서 임의 처리하지 않고 범위 변경을 보고한다.

## 9. 완료·복구·이번 턴의 경계

완료는 위 타깃 분리·기존 계약·제품 독립성·CI worker 재현이 함께 확인되는 시점이다.
R1 통합 뒤에만 R2 필요성과 비용을 다시 판단한다. 구조 변경은 독립 커밋으로 보존하여
실패 시 다른 기여자의 코드를 건드리지 않는 revert 후보를 만들고 실제 원격 복구는 승인받는다.

이 제안 작성 시 제품 코드 변경·빌드·테스트·archive 실행·후속 이슈 등록은 수행하지 않았다.
메인테이너가 #7001의 조사 전용 범위를 재확인했으므로 구현 승인 요청을 철회했다.
현재 승인된 후속 절차는 조사 문서 커밋·PR 제출·self-review뿐이다.

이 문서는 구현 또는 후속 이슈 등록의 승인 요청이 아니다.

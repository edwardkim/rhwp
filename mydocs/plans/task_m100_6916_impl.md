# #6916 구현계획 — Gym 안내 공급원 최소 분리 (A안)

- Issue: [#6916](https://github.com/edwardkim/rhwp/issues/6916)
- 상위 계획: [수행계획서](task_m100_6916.md)
- 근거: [Stage 1 조사](../working/task_m100_6916_stage1.md)
- 기준 제품 SHA: `c72ad805cc60e4a5cf5689c18b44e7214cec68fe`
- 상태: 2026-09-09 A안·Stage 2 구현 승인. focused 계약 검증 진행 중.

## 1. 고정할 설계

MCP `rhwp://docs/gym`은 삭제하지 않는다. 제품이 소유하는 짧은 **선택적 Gym 안내**를 내장한다.
Gym 전체 README를 복제하거나 동기화하지 않는다. Gym의 과제 수·점수·리더보드·세부 실행법을
제품 상수로 옮기지도 않는다. 제품 코드에서 Gym 폴더 검사, 환경변수 fallback, HTTP 요청은 추가하지 않는다.

### 공개 계약

- 유지: URI, name `gym-readme`, title, MIME `text/markdown`, resources/list→read, 오류 봉투.
- 변경: description, 본문, 본문 UTF-8 바이트 수에 대응하는 size.
- 새 본문은 Gym이 선택적 평가 도구라는 점, 제품 실행에 불필요하다는 점, 별도 Gym checkout과
  rhwp 바이너리로 이용한다는 점, 공개 참가자 안내·운영 문서 URL만 제공한다.
- 링크를 따라가는 일은 사용자의 선택이다. MCP 리소스 읽기는 오프라인에서도 성공해야 한다.

## 2. Stage 2 수정 파일

| 파일 | 변경 |
| --- | --- |
| `mydocs/manual/gym_optional_tool.md` (신규) | 제품 소유의 짧은 안내. guide/active metadata, canonical은 Gym 운영 매뉴얼. 전체 README 복사 없음 |
| `src/mcp_serve.rs` | 해당 리소스 include를 새 안내로 교체. description·주석만 실제 의미에 맞게 보완. handler·tool registry는 유지 |
| `.github/workflows/oracle-public-advisory.yml` | sparse checkout의 `gym/README.md`를 새 안내 경로로 교체. trigger/job/permissions/timeout/verdict 변경 없음 |
| `scripts/tests/test_oracle_public_advisory_workflow.py` | MCP_DOCUMENT_PATHS를 동일하게 교체. 실제 include·파일·sparse 입력 대응 유지, Gym 디렉터리 요구가 재도입되지 않도록 해당 checkout 검사 |
| `tests/cases/issue_6916_gym_optional_resource.rs` (신규) | MCP URI 존속·MIME·본문 의미·size·읽기 계약. 프로세스 대기에는 유한 timeout/종료 처리를 두어 테스트 hang 방지 |
| `mydocs/manual/gym_benchmark_operations.md` | 제품 빌드·실행과 선택적 평가 도구의 단방향 의존, MCP 안내 경계 명시 |
| `gym/AGENTS.md` | 제품 코드에 Gym 파일/채점기 의존을 추가하지 않는 불변식 명시 |
| `mydocs/manual/README.md` | 신규 제품 안내를 찾을 수 있는 최소 링크 추가 |
| `mydocs/tech/render_backend.md` | Gym sparse 추가 처방을 과거 기록으로 구분하고 새 경계 안내 연결 |

일반 CLI/API, Gym task/reference/check/score, 조직·프레임 감사, Gym workflow,
workflow promotion policy, CI classifier, package manifest는 변경하지 않는다.
테스트는 신규 `tests/cases/` 원본만 제출하고 generated suite·manifest·Cargo marker를 제출하지 않는다.

## 3. 검증과 실행 순서

### Stage 2 — focused 계약

1. 기존 Oracle/Gym workflow 경계 테스트 16건과 조직 테스트 8건을 같은 명령으로 재실행한다.
2. review 검증 환경에서 `node scripts/rust-test-suite-manifest.mjs --prepare` 후
   `scripts/run-rust-test.mjs`로 신규 source와 기존 `mcp_resources_contract.rs`,
   `mcp_server_contract.rs`, `mcp_spec_ledger_contract.rs`를 manifest에 맞춰 실행한다.
   개별 source 파일명을 `cargo test --test`에 직접 넘기지 않는다.
3. 신규 계약을 수정 전 코드에 적용하면 실패하고 수정 후 통과하는지 확인한다.
   URI 존속만 검사하지 않고 실제 선택적 안내와 목록/본문 크기 정합을 확인한다.
4. 새 내장 문서와 sparse checkout을 함께 검사한다. 해당 workflow 변경은 O3 실행 입력 변경으로 다루되
   권한·event·결과 정책은 보존한다. actionlint 및 관련 Python 계약을 실행한다.
5. 변경 문서의 링크·metadata·`git diff --check`를 확인한다. Gym 문서 변경의 구조 계약은
   운영 매뉴얼을 따르며, 채점 의미를 바꾸지 않았으므로 전수 벤치마크는 실행하지 않는다.

### Stage 3 — 제품 독립성 증명·PR 준비

1. exact 후보 SHA의 격리 source를 구성하되 Gym 디렉터리를 처음부터 제외한다. 주 checkout의 Gym이나
   기존 target을 제거하지 않는다. 테스트용 source/target/출력 경로와 도구 버전을 기록한다.
2. Gym 없는 source에서 `cargo build --locked --release --bin rhwp`를 새 target으로 수행한다.
   릴리스와 동일한 binary/LICENSE/README/README_EN 포함물을 로컬 포장한다.
3. 포장을 풀어 저장소 밖 디렉터리에서 `rhwp --version`, `--help`, MCP initialize/list/read를 실행한다.
   Gym 안내뿐 아니라 광고된 리소스 전체 읽기를 확인한다. Gym 경로를 가리키는 환경 설정은 주지 않는다.
4. 동일 Gym 없는 source를 Docker 빌드 환경에 mount하여 `scripts/wasm-pack-locked.sh --target web`을
   실행한다. 독립 target과 pkg를 사용하고 Studio가 현재 사용하는 pkg를 덮어쓰지 않는다.
   `scripts/prepare-npm.sh` 후 `npm pack --dry-run --json`을 pkg에서 실행해 포함물을 확인한다.
   외부 package publish는 하지 않는다.
5. Gym이 있는 별도 검증 공간에서는 위에서 만든 rhwp 바이너리의 절대경로를
   `gym/tools/build_baseline.py --agent maintainer-6916-canary --pack core-cli --bin <절대경로> --json`에
   전달한다. pack canary 결과를 전수 통과·제품 정확성으로 부르지 않는다. 기준풀이/채점기는 수정하지 않는다.
6. PR 준비 승인 범위에서 [로컬 검증 4.3](../manual/pr_review/local_validation.md)에 따른
   Rust lint 전체 묶음과 CLI 변경의 release-test 전체 회귀를 수행한다. fresh code head이므로
   기존 CI 재사용 예외를 미리 적용하지 않는다. 전체 integration은 nextest 경로를 사용한다.
7. native Linux, Docker WASM, npm 포함물은 실행 증적을 남긴다. frontend·VSIX·브라우저 확장 전체 재빌드는
   bin-only 변경이 도달하지 않는 경로라는 조사 근거로 생략한다. Windows/macOS 실행은 미확인으로 표시한다.

새 상시 CI job·전수 Gym 실행을 추가하지 않는다. 기존 Rust suite와 workflow 계약 검증을 이용한다.
Oracle workflow 수정으로 승인된 원격 push 시 기존 bootstrap run이 실행될 수 있다.
그때 exact head의 실제 build/verdict를 확인한다. O3 workflow의 운영 적용은 로컬 통과만으로 완료 선언하지 않는다.
main 승격 때는 기존 promotion 정책을 따르며 이번 task에서 main에 직접 반영하지 않는다.

## 4. 완료 및 중단 기준

승인 대상은 **전체 Gym README 제공을 가벼운 제품 안내로 바꾸는 것**이다. 모든 response bytes를
보존하는 변경은 아니다. URI 삭제·동적 feature·network fallback이 필요해지면 구현을 확대하지 않고 재승인을 요청한다.

성공 기준은 제품 독립 빌드·설치본 동작·배포 포함물·선택적 Gym 실행·기존 공개 계약의 동시 충족이다.
전체 저장소 감사에는 Gym 자산 검사가 남으므로 Gym 없는 checkout의 일반 CI 전체 성공을 약속하지 않는다.
실제 추가 필수 제품 의존이 발견되면 누락을 보고하고 같은 이슈의 구현계획을 수정한다.

복구는 제품 안내/include/Oracle sparse 입력·mirror test를 같은 단위로 되돌린다.
Gym 자체 자산·기존 증적은 건드리지 않는다. 구현 결과 승인 전에 remote push·PR 생성은 하지 않는다.

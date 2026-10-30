# #8391 verdict protocol의 자동 CI 실행 제외

- Issue: [#8391](https://github.com/edwardkim/rhwp/issues/8391)
- 관련 구현: [#5486](https://github.com/edwardkim/rhwp/issues/5486)
- 기준: `upstream/devel`의 `cb9c45cf774a03eddb5d56fdb1fd28919f10a969`
- 브랜치: `task/8391-exclude-verdict-protocol-ci`
- 운영 변경 등급: O3(job command). 로컬 수정·검증까지만 수행한다.

## 목표와 보존 범위

메인테이너 결정에 따라 `llm-verifier-verdict-protocol`만 자동 CI의 build/test/Clippy
실행에서 제외한다. Cargo workspace 구조, 패키지 소스·스키마·코퍼스와 수동 실행 경로는 유지한다.
다른 workspace 멤버, rhwp 제품 검사, 명시적 verifier 계약 검사 7개와 CI 보안 정책은 유지한다.

## 조사와 구현

1. `.github/workflows/*.yml`, composite action과 관련 workflow 계약 테스트에서 실제 Cargo
   호출을 확인했다. root archive는 `--package rhwp`로 한정되고 기본 Cargo 명령의
   `default-members`는 root와 `crates/*`이므로 해당 패키지를 선택하지 않는다.
2. `Test internal Rust crates`의 `cargo test --workspace … --lib`는 해당 패키지의
   단위 테스트 19개를 실행했다. `tests/corpus_sample.rs` 등의 integration test는 이 명령에
   포함되지 않았다. 2,240개 합성 코퍼스의 정답 비교를 CI가 수행했다고 해석하지 않는다.
3. 위 내부 테스트와 `Check workspace members (FFI bindings, tools)`의 workspace build,
   all-targets Clippy에만 `--exclude llm-verifier-verdict-protocol`을 추가했다.
   `--lib`, `--all-targets`, `-D warnings`와 기존 제외 목록은 보존했다.
4. `Validate LLM verifier tool contracts`의 `third_party_replay`, `shadow_agree`,
   `untrusted_sandbox`, `lineage_chain`, `abstain`, `repeat_eval`, `tool_version_gate`를
   그대로 실행한다. `claim_bind`, `criteria_decomp`도 workspace 검사에 남는다.
5. 기존 `test_llm_verifier_workflow.py`를 확장했다. 세 workspace 명령의 정확한 제외 집합,
   명시적 verifier 계약의 실행 배선, 제품 Clippy와 archive 패키지 선택을 검사한다.
   기존 CI 호출에서 자동 실행되므로 새 step이나 중복 호출을 추가하지 않았다.

공통 `cargo fmt --all -- --check` 형식 정책은 유지한다. workflow event, required check,
권한, trusted classifier, CodeQL, cache, runner와 배포 정책도 바꾸지 않는다.

## 수동 실행

저장소 루트에서 기존 명령을 그대로 사용할 수 있다.

```sh
cargo test --locked -p llm-verifier-verdict-protocol
cargo run --locked -p llm-verifier-verdict-protocol --bin vproto-classify -- observation.json
```

## 검증 기록

- 변경 전 workflow에서 새 회귀 검사: 의도한 제외 누락으로 실패했다. 대상 명령은 내부 테스트,
  workspace build, workspace Clippy 세 개였다. 다른 verifier와 제품 선택 보존 검사는 통과했다.
- 변경 후 선택적 제외 회귀 검사: 6개 통과. 별도 `-p`/manifest 호출로 다시 포함하는 경우도
  차단한다.

| 검사 | 결과 |
| --- | --- |
| 기존 workflow·배선·nextest archive·CI 영향·CodeQL·trusted reuse Python 계약 | 112개 통과(위 6개 포함) |
| CI impact classifier·policy Node 계약 | 95개 통과 |
| CI YAML 파싱 | 통과, 22개 job |
| 변경된 내부 Rust `--workspace … --lib` | 5개 패키지, 229개 테스트 통과 |
| 변경된 workspace build | 통과, 대상 패키지를 제외한 workspace 멤버 9개 확인 |
| 변경된 workspace all-targets Clippy, `-D warnings` | 통과, 대상 패키지를 제외한 workspace 멤버 9개 확인 |
| 기존 공통 `cargo fmt --all -- --check` | 통과 |
| 명시적 Python verifier 5개 | 34 + 31 + 29 + 40 + 18 = 152개 통과 |
| `repeat_eval`, `tool_version_gate` | 각각 37개, 33개 통과 |
| 제외 대상 패키지의 수동 `cargo test --locked -p llm-verifier-verdict-protocol` | 단위 19개 + integration 11개 = 30개 통과 |
| `git diff --check` | 통과 |

Cargo는 Windows MSVC Rust 1.93.1에서 `--locked`로 실행했고, 현재 작업공간의
`target/pr-review`를 순차 재사용했다. 실제 `compiler-artifact` 기록에서 내부 테스트의
`claim_bind`, `criteria_decomp`, 내부 production crate 3개를 확인했다. build에는 이 5개와
`rhwp`, Native FFI, subsecond, batch-convert가 남으며 제외 대상 패키지는 없다.
all-targets Clippy에서도 동일한 9개 멤버의 artifact와 최종 성공을 확인했다.

초기 Windows Python·Bash 계약 실행은 기본 문자 인코딩·경로·빈 환경변수 차이 때문에 실패했다.
검사 코드를 완화하지 않고 WSL Ubuntu 24.04의 Python 3.12.3, Node 24.15.0에서 위 112개·95개를
다시 실행해 모두 통과했다. 부분 체크아웃에서 누락된 실제 Rust `include_*` 입력은 같은 base의
원본 blob으로 준비한 뒤 빌드를 재실행했다. 이 준비는 제출 diff에 포함되지 않는다.

로컬 실행 로그는 ignored `output/pr-review/8391/`의 `internal-tests.log`,
`workspace-build.log`, `workspace-clippy.log`, `workflow-tests-linux.log`,
`impact-node-tests-linux.log`, `manual-verdict-tests.log`, `repeat-eval-tests.log`,
`tool-version-tests.log`에 있다. GitHub Actions는 지시에 따라 실행하지 않았다.

## 적용과 복구

원격 push, PR 생성·승인·병합, CI 재실행과 배포는 수행하지 않는다. 적용을 되돌릴 때는 세 Cargo
명령의 대상 패키지 제외와 대응 회귀 계약 변경을 함께 revert한다. 패키지 파일 복구나 workspace
재구성은 필요하지 않다.

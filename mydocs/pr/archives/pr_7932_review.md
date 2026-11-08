# PR #7932 검토 — Native 참조 호스트 패키징

## 최종 판정

**승인 — 메인테이너 통합 PR #8372의 code candidate `80a034fd30fc79c7b6b22f315908ec2b4dfd7b38`에서 로컬 검증·전체 PR CI·3개 OS 패키징·수동 CI 재실행을 완료했다.**

원 fork head의 실패를 성공으로 간주하지 않고 검증된 통합 head를 수용한다. 기록 commit은 문서만 추가하며, 이 후행 head의 최신 CI와 merge simulation 재확인 뒤 일반 merge로 통합한다. 사용자 merge 승인은 확보됐다.

## 접수와 인수 경로

- 원 PR: [#7932](https://github.com/edwardkim/rhwp/pull/7932), 작성자 `seo-rii`, base `devel`, reviewer `edwardkim` 지정과 API 재확인.
- 원 head: `4618734ce70f61bc6de86c3d990d93fe15309355`.
- 검증 base: `6a3cb5a9c194366169a9bb11ee75b6addfa2e8c4`.
- 메인테이너 통합 commit: `cbe6baeecbf81c0a1e4ed10a5c2eb7f827002b01`. 원 head를 두 번째 parent로 보존했다. contributor commit을 rewrite하지 않았다.
- 최종 code candidate: `80a034fd30fc79c7b6b22f315908ec2b4dfd7b38`. 패키징 가이드의 로컬 target 지침만 공용 `target/pr-review`에 맞췄다.
- 통합 PR: [#8372](https://github.com/edwardkim/rhwp/pull/8372). 원 PR별 기록을 이 통합 PR에 포함하며 별도 통합 PR review 파일을 중복 생성하지 않는다.
- 관련 이슈: [#536](https://github.com/edwardkim/rhwp/issues/536) 참조. 전체 멀티 렌더러 추적 이슈를 닫는 범위가 아니다.
- 사용자 지시: 2026-10-29, 메인테이너가 #7932의 CI 문제를 처리해 merge하도록 승인했다.
- 기본 경로 `maintainer_general`; 접수·로컬 검증·외부 PR 인수·review-only·후속 처리 및 GitHub 운영 O3 절차 적용.

## 기존 CI 실패와 처리

1. [원 CI 정책 job](https://github.com/edwardkim/rhwp/actions/runs/40271256716/job/121404361704)의 annotation은 `Untrusted head changed the enforcement surface.`다. live base의 `scripts/ci-impact-policy.cjs`는 write 권한 없는 fork의 workflow 변경을 `fail-closed:untrusted-enforcement-change`로 차단한다. fork에 메인테이너 커밋을 추가해도 repository identity는 바뀌지 않는다. 정책을 완화하거나 성공 status를 수동 주입하지 않고 동일 저장소 통합 PR을 만들었다.
2. [원 Archive D](https://github.com/edwardkim/rhwp/actions/runs/40271259222/job/121408207995)의 annotation은 `The job has exceeded the maximum execution time of 30m0s`다. 원 해결 #8267의 `115f2df439758b15f2b4e7aac07be2b787641b04`와 후속 `678ba12d16`을 추적했다. 최신 base의 A–E 분산·60분 worker 제한을 소비하며 자체 timeout·검사 범위 변경은 없다.
3. 이전 head에 달린 CodeQL #276은 `svg-image-header.ts`의 경고다. 현재 PR 고유 diff에는 이 파일이 없고 API의 most_recent_instance.state는 `fixed`였다. 경고 분류·탐지 설정은 변경하지 않았다.

## 변경 범위와 자체 검토

최신 base 대비 8개 파일이다. 명시된 native-skia 라이브러리의 capability를 검사하고 Python/Qt 참조 호스트를 독립 디렉터리로 패키징한다. 런처가 schema·OS/architecture·필수 파일 11개·SHA-256·경로 경계를 확인한다. 기존 파일/디렉터리/끊어진 symlink를 덮어쓰지 않으며 실패 정리는 해당 호출이 생성한 대상에 한정한다.

새 workflow는 `workflow_dispatch`와 그 수동 caller의 reusable 호출만 실행한다. `contents: read`, SHA 고정 action, `persist-credentials: false`, 기본값 false 입력, 7일 artifact 보관을 확인했다. CI 집계와 promotion은 별개이며 promotion의 exact run/head·3개 job·artifact 요구를 유지한다. release·서명·실제 배포 권한은 추가하지 않았다.

해시·declaredSourceRevision은 provenance 인증이 아니며 라이브러리는 신뢰된 실행 코드라는 문서 경계를 확인했다. 실제 OS IME·screen reader·installer·서명은 기존 후속 범위다. Qt/FFI host source는 원 head와 byte-identical이고, Rust·renderer·WASM·fixture·baseline 변경은 없다.

### 조판 원칙과 검증 입력

조판 규칙·측정·배치·paint·저장 문서 속성을 바꾸지 않으므로 조판 원칙의 구현 검토, fresh WASM·Visual Sweep은 비해당이다. 실제 패키징 host가 기존 ABI를 소비하는 동작은 아래 Native 실행으로 확인했다. 한컴 피델리티 개선을 주장하지 않는다. headless/Qt 검사는 커밋된 예제·smoke 스크립트가 자체 생성한 문서를 사용하며 외부 HWP/HWPX/PDF를 수용 근거로 사용하지 않았다.

## 로컬 검증

검증일 2026-10-29. Linux/WSL2, Rust 1.93.1, 기본 작업공간 공용 `target/pr-review`, 순차 Cargo 실행.

| 검사 | 결과 |
| --- | --- |
| `python3 bindings/Native/tests/package_native_host_contract.py` | 12 PASS |
| workflow promotion preflight/evidence/gate/wiring 4개 모듈 | 50 PASS |
| `python3 -m unittest discover -s scripts/tests -p 'test_*workflow*.py'` | 266 PASS |
| CI classifier/policy/controller/evidence Node 계약 | 147 PASS |
| actionlint 1.7.12, ci.yml/native-host-package.yml | PASS; shellcheck·pyflakes 연동 제외, Python 계약 별도 실행 |
| 문서 상대 링크·`git diff --check` | PASS |
| FFI `cargo build --locked --profile release-test -p rhwp-native-ffi --features native-skia` | PASS, 3분 17초 |
| 패키지 actual capability probe·manifest | PASS, 11/11 SHA-256 |
| 공백 경로로 이동 후 cwd `/`에서 headless 실행 | 렌더·한글 IME·undo·저장·재열기 PASS, 1쪽, IME roundtrip 확인 |
| 패키지 Qt 모듈의 adapter | 25 PASS |
| 패키지 Qt 실제 offscreen smoke | nativeRender/syntheticKoreanIME/clipboard/multiPage/caretVisible/saveReopen 모두 true |

Python 묶음은 일부 중복되므로 건수를 합산하지 않는다. FFI 빌드에는 `CARGO_BUILD_JOBS=6`, `CARGO_PROFILE_RELEASE_TEST_OPT_LEVEL=1`, `CARGO_PROFILE_RELEASE_TEST_CODEGEN_UNITS=64`를 적용했다. 빌드 시작 SHA는 통합 commit이며, 이후 code candidate까지 변경은 문서뿐이다. 라이브러리 SHA-256은 `7d95be71db8107ec2e3806a32aef874e79d0ff4b99640556ccfac1e02b43d6df`다. 패키지 manifest의 선언 SHA는 code candidate다.

기존 Rust 전체 회귀를 로컬에서 재실행하지 않았다. 이번 변경 범위의 Python·CI·실제 Native 패키지 검증을 로컬에서 완료하고 최신 GitHub CI를 별도로 확인한다. Qt offscreen은 실제 OS IME 증거가 아니다.

## GitHub 검증

- [PR CI](https://github.com/edwardkim/rhwp/actions/runs/40864498353): success. A–E·Native Skia·lint·frontend 성공.
- [수동 CI·3개 OS 패키징](https://github.com/edwardkim/rhwp/actions/runs/40864495107): attempt 2 success. 세 플랫폼은 attempt 1 성공을 유지했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40864498287): success, 네 언어 분석 성공.
- [Proptest](https://github.com/edwardkim/rhwp/actions/runs/40864498339), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/40864498296): success.

수동 CI attempt 1에서 Linux·macOS·Windows 패키징은 모두 성공했다. Archive A 빌더 `123415822453`만 checkout의 `git fetch`에서 30분 제한으로 취소됐고, 실제 빌드·테스트 step은 시작되지 않았다. [집계 job](https://github.com/edwardkim/rhwp/actions/runs/40864495107/job/123428907395)은 이에 따라 실패했다. 성공 job을 다시 돌리지 않고 `gh run rerun --repo edwardkim/rhwp --job 123415822453`로 A 빌더와 종속 worker·집계를 재실행했다. 전체 run이 진행 중일 때 GitHub가 재실행을 거부했으며, 완료 뒤 요청이 수락됐다. Attempt 2의 A 빌더 `123433444486`, A worker `123439166799`, [집계 `123440658926`](https://github.com/edwardkim/rhwp/actions/runs/40864495107/job/123440658926)가 모두 성공했다. 성공한 다른 job의 실행 시각·artifact는 attempt 1에 귀속하며 복사된 새 job ID를 새 실행으로 집계하지 않았다.

세 플랫폼의 artifact를 실제로 내려받아 각각 11개 파일의 SHA-256, OS·architecture, capability, code candidate 선언을 검증했다. 해시는 artifact의 동일성 증거이며 서명·출처 인증이라고 주장하지 않는다.

| 플랫폼 | artifact ID | 라이브러리 SHA-256 | 검증 |
| --- | --- | --- | --- |
| linux-x86_64 | `12894162948` | `ba270d600ef90db597f610c4d5f37828835dcc89ea27d4a6b0a1e85e84c766ac` | 11/11 PASS |
| macos-aarch64 | `12894704279` | `f48743cdb8bdfd7410c995a097b7f520683e59d30c7590624c0d275cfad950d6` | 11/11 PASS |
| windows-x86_64 | `12894358086` | `3a9f1701c08d87d5376b3f37bcc3ea6d5ef4d014761d1825dc692bdf76d49144` | 11/11 PASS |

## Merge 후 contributor PR comment 계획

통합 PR과 merge SHA를 확인한 뒤 원 PR에 한국어 존댓말로 감사와 결과를 남긴다. 원 커밋 보존, fork CI 정책을 완화하지 않은 인수 이유, Archive 분산 개선 재사용, 실제 로컬·3개 OS·CI 결과를 구분한다. #536은 추적 이슈로 유지한다. 패키지 배포·OS IME까지 완료했다고 주장하지 않는다. comment는 UTF-8 본문 파일로 게시한 뒤 API 본문을 재확인한다.

## 후행 기록의 검증 범위

기록은 이 review와 2026-10-29 오늘할일에 한정한다. 검증된 code candidate 이후 source·test·workflow·packaging input의 byte 변경은 없다. 최신 base와 merge tree, 공백·문서 링크·기존 오늘할일 보존을 push 전에 확인한다. 후행 기록의 exact-head CI는 trusted review-only 정책의 실제 성공을 확인하며 추정으로 생략하지 않는다.

## 종료 조건

code candidate 및 기록 trailing head의 최신 CI, 최신 base와 merge simulation, 원 커밋 ancestry, 원 PR 상태를 확인한다. merge 뒤 devel sync·duration 갱신 결과·comment를 확인하고 이번 임시 branch와 root output 전체 및 사용 종료된 target/pr-review/debug를 정리한다. contributor fork branch는 보존한다.

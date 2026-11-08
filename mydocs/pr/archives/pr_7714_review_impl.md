# PR #7714 적용 기록

## 원 기여와 통합

- 원 [PR #7714](https://github.com/edwardkim/rhwp/pull/7714)의 `688c896d065d68249b2ec370fb89f86cf30e9bd5`를
  author·`cherry-pick -x` 원 SHA를 보존해 `15067601e415beb27268612001ad95b2a189db9d`에 적용했다.
- 기준 `upstream/devel`은 `8766fb47d8ec11547f0d2ae941c53cc3c9780ab2`,
  브랜치는 `review/wook8170-20261018`이다. 7713→7714→7715 순으로 누적했다.
- #7715 README 충돌은 앞선 캐럿·이벤트 설명과 새 profile 설명을 함께 보존해 해결했다.
- 제품 보정은 별도 `cfd06cd2b5cc47a268134d0c09f657a7a9fd24d0`, Windows E2E 경로 정규화는
  `f1277506ac0d093f5109b16cbef0487f5f6a0ebd`다. 원 contributor fork source는 rewrite하지 않았다.
- 원인·생산→소비 경로·반례·독립 기대값·수정 전후 관측은 [개별 검토](pr_7714_review.md)에 있다.
  판정은 **메인터너 보정 후 승인**이며 원격 완료와 구분한다.

## 검증과 후속 순서

통합 [PR #8003](https://github.com/edwardkim/rhwp/pull/8003)의 정확한 code head는 `9c1c681ee44c65eb924a66ac18b0b86876ad767b`다.
[Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39051970167) · [CI](https://github.com/edwardkim/rhwp/actions/runs/39051970186) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39051970001) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39051970099) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39051970173) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39051970257)가 실제 완료·성공했다. 언어별 CodeQL Analyze와 GHAS check도 확인했고,
정책상 skipped·neutral은 실제 실행 성공과 구분했다. 해당 merge ref의 열린 CodeQL alert는 0건이다.

| 로컬 검증 | 실제 결과 |
| --- | --- |
| 전체 nextest release-test | 10,823 PASS / 0 FAIL / 기존 45 SKIP, 실행 1310.134초 |
| fmt·Native/WASM Clippy·workspace build·all-target Clippy | 5개 모두 exit 0 |
| Native Skia lib | 4,043 PASS / 기존 12 IGNORE |
| Skia missing-picture PNG / direct-PDF | 2 PASS / 14 PASS |
| Studio 전체 단위·타입·production build | 2,327 PASS / 기존 2 SKIP, 타입·build exit 0 |
| editor / frontend bindings / hwpctrl | 36 / 3 / 26 PASS, 선언·pack exit 0 |
| 실제 Chrome E2E | 7종 exit 0, SDK profile 9개 assertion PASS |
| manifest·unit-tier·E2E manifest | 통과, E2E 161파일 / 161행 |
| 문서 링크·metadata | 링크 통과, base/head 기존 16오류 동일·신규 0 |

전체 Rust 명령은 `CARGO_BUILD_JOBS=8 cargo nextest run --locked --cargo-profile release-test
--target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast`다.
공용 target을 순차 재사용했다. 제품 보정 source `cfd06cd2b5cc47a268134d0c09f657a7a9fd24d0`
이후 제품 입력 바이트가 동일한 것을 지문으로 확인했으며, 후속 변경은 E2E·증적뿐이다.
문서 trailing 때문에 같은 제품의 전체 회귀·WASM 빌드를 반복하지 않는다.

WASM은 Docker 없는 승인된 호스트 진단 경로에서 wrapper `--target web --out-dir pkg --no-opt`,
jobs 8·release codegen-units 16으로 fresh 빌드했다. 최적화 배포 검증으로 확대하지 않는다.
실제 Studio E2E는 Chrome 155.0.8059.79, 표준 Sweep의 지정 브라우저는 Chrome 154.0.8037.57이다.
Windows 모듈 URL은 기존 정규화 패턴에 맞췄으나 실제 Windows 실행으로 보고하지 않는다.

원격 후속 조건은 이 문서 trailing head의 실제 required check·fast-pass·mergeability 확인과 정상 merge다.
아직 실행 전인 merge·이슈 종료·댓글·정리를 완료로 쓰지 않는다. 사용자는 전체 후속 처리를 승인했다.


code CI 성공 후 이 개별 review/impl와 오늘할일만 같은 PR의 single-parent trailing commit으로
반영한다. push 전 최신 base/head의 merge-tree·공백·변경 링크·오늘할일 보존을 검사하고,
최신 fast-pass·required check 성공 후 정상 merge한다. merge 뒤 확정 SHA·duration·이슈·댓글
영수증은 영구 증거 경로에 보존하며 재검증 workflow를 시작하지 않는다.

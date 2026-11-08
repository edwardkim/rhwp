# PR #8181 리뷰 — crates.io 패키징과 외부 Rust·CLI 소비 준비

## 최종 판정

**승인 — #8174의 로컬 기술 보완 범위 v1에 대한 self-review와 code head CI 검증 완료.**

작성자 self-review다. 실행으로 재현한 독립 Skia 도구의 의존성 결함 1건을
`7068bd7d8a369178a7f5421215dbefb64ab88248`에서 수정하고 실제 host 실행으로 재검증했다.
추가 코드 수정이 필요한 미해결 발견 사항은 없다. 실제 crates.io 게시 준비 전체를 완료한 판정은 아니다.

이 문서와 오늘할일을 추가하는 trailing head의 CI·실제 fast-pass, 최신 base와의 병합 가능성,
작업지시자의 merge 승인은 별도로 확인해야 한다. GitHub approve review·merge·이슈 종료는 수행하지 않았다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#8181](https://github.com/edwardkim/rhwp/pull/8181) / `edwardkim` / `devel` |
| 최초 PR head | `06886117e5d272134a2108a50d5001075b0ae71d` |
| 보정 후 검토 code head | `7068bd7d8a369178a7f5421215dbefb64ab88248` |
| 검증 base | `def947c2149df5bc812d801e6e79be19a403b4d1` |
| 관련 이슈 | [#8174](https://github.com/edwardkim/rhwp/issues/8174)를 참조한다. 게시 정책·운영 준비가 남아 자동 종료하지 않는다. |
| 경로·상태 | `collaborator_self_merge` + `intake_and_review`·`local_validation`·입력 보존을 위한 `visual_fixture_evidence`·대형 PR의 `rework_and_exceptions`·후행 기록의 `review_only_fast_pass`; Open PR, reviewer 지정 없음 |

## 변경과 검토 범위

[승인 범위 v1](../../plans/task_m100_8174_maintainer_decisions.md)과
[2단계 보완 결과](../../working/task_m100_8174_stage2_corrections.md)를 대조했다.
검토는 manifest·실제 archive·registry 소비, 공개 오류 계약, CLI 로더·개발 도구의 호출 경로,
보안 제한·설치/API 문서 순서로 수행했다. 코드 검토와 실제 소비 검증을 구분했다.

- 내부 path 의존성에 버전을 병기하고 SVG fork를 직접 의존한다. root patch를 제거하며
  Cargo include를 package root에 고정했다. source 사전 검사는 8개 후보의 자원·고지를 확인한다.
- `HwpError`에 표준 `Error`를 구현한다. `export-text`의 loader만 동일 `DocumentCore`의 직접
  소비로 바꾼다. 기존 facade의 생성 경로·암호 분기·오류 분류·텍스트 추출이 같은지 확인했다.
- `font-metric-gen`은 `dev-tools` feature로 명시해 기본 CLI 설치와 구분한다.
  generator 검사의 기존 버전 pin은 실제 생산자의 `CARGO_PKG_VERSION` 계약에 맞췄다.
- 제품 버전 0.8.7, 내부·fork 7개의 `publish = false`, 기존 feature 기본값과 WASM export 계약은
  유지한다. 0.9.0 전환·실제 게시·MD01·MD05–MD09 확정은 이번 PR의 완료 범위에 포함하지 않는다.

## 실행으로 검출한 결함과 보정

`tools/skia-surface-probe`는 독립 Cargo workspace다. 첫 후보에서 복사된 과거
`[patch.crates-io] svg2pdf`가 이름을 바꾼 `vendor/svg2pdf`를 가리켜 실제 metadata가 exit 101이었다.
root 제품만의 검증으로 이 소비 경로를 확인하지 못한 점을 보정했다.

| 근거·실행 | 관측과 판정 |
| --- | --- |
| 원 기능 | #8097 → [통합 PR #8135](https://github.com/edwardkim/rhwp/pull/8135), 실제 merge `a6e9d18fc1061318055b4eb348daec201427ea03`; [기존 host 검증](pr_8097_review.md) |
| 독립 base 대조 | 고정한 devel의 도구에서 `cargo metadata --locked --offline` exit 0. 최초 PR 후보의 같은 명령은 package 이름 불일치로 exit 101 |
| 보정 | root가 fork를 직접 의존하므로 도구의 중복 patch를 제거. Cargo가 생성한 lock에서 SVG package 이름·버전만 바뀌며 다른 package 버전은 동일 |
| 보정 후 | locked offline metadata·도구 fmt 검사 통과. 실제 `cargo run --locked --manifest-path tools/skia-surface-probe/Cargo.toml` 통과 |
| 실제 host | `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`, llvmpipe Vulkan, scale 1.25. 내부색·host state 복원·잘못된 인자의 pixel 보존 3개 계약 통과. `hardwareAccelerationVerified=false` |

도구 manifest·lock 이외의 제품 source를 이 보정에서 바꾸지 않았다. 패키지 source 대장의
2,310개 항목은 기존 검증본과 Git blob이 같았다. 전체 로컬 회귀를 반복하지 않고 실제 실패한
독립 도구 경로를 추가 검증했으며, 새 code head의 원격 CI도 별도로 확인했다.
명령·수정 전후 종료 코드·lock 대조·실행 결과는 ignored
`output/pr-review/issue8174/pr-preparation/self-review/`에 보존했다.

## 검증 입력과 로컬 결과

패키지 source head는 `9df631769bd668d3e4b86f86d1359689a108759b`, 전체 검증 checkout은
`b1a5695479d61322c1b61cc771cfbc2c2b375cb3`다. 양쪽 및 검토 code head의 제품·패키지 입력을
Git blob 목록으로 대조했다. 후속 변경은 문서·검증 입력과 위 독립 도구 보정이다.
명령 전문과 범위는 [기존 검증 표](../../working/task_m100_8174_stage2_corrections.md#pr-제출-전-로컬-검증--2026-10-23)에 연결한다.
이번 작업에서 base를 merge/rebase해 추가한 PR은 없다.

| 검사 | 실제 결과와 한계 |
| --- | --- |
| 필수 Rust 게이트 | suite 준비·base 비교, fmt, Native Clippy, WASM Clippy, workspace build·all-target Clippy 모두 통과 |
| 실제 전체 회귀 | `cargo nextest run --locked --cargo-profile release-test --tests --test-threads 8 --no-fail-fast`: 55개 binary, 11,363/11,363 통과, 45개 기존 제외, 실패 0 |
| 표준 오류 전파 | 신규 공개 API 검사를 실제 자동 suite에서 1/1 통과. 독립 앱의 기존 E0277 → path·registry 각각 통과. 실제 열기 실패를 `?`로 boxed error에 전파하고 `HwpError`로 downcast |
| 실제 package 소비 | 정규화된 `.crate` 8개를 로컬 sparse source replacement에서 다운로드. 외부 consumer의 path/patch 없이 registry source임을 확인. 최소·기본 구성의 parse→HWP/HWPX 저장·재독→편집 smoke, README 예제 2개 compile/run, CLI 설치·rustdoc 통과 |
| CLI 보존 | 정식 입력 경로의 텍스트 17개·PDF 입력 1개에서 base와 registry CLI의 exit/stdout/stderr 일치. 암호 미지정·오입력·정상 암호와 인자 오류 포함 |
| SVG 출력 보존 | 16개 source를 같은 stable rustfmt로 정규화한 결과가 base와 동일. 같은 cwd에서 대표 SVG PDF 출력이 바이트 일치: 12,599 bytes, SHA-256 `4d6fc1c51a36ad4e90a02d4f71834ef9c764a8098e6e3dba9395a0d6b8b01548` |
| 도구·문서 | source 검사 18/18, generator 5/5, suite policy 23/23, 변경 문서 링크·장기 문서 6개 메타데이터 통과 |

환경은 Linux x86_64, Rust/Cargo 1.93.1, Node 24.15, Python 3.12.3이다.
Cargo는 공용 `target/pr-review`, jobs 8을 사용했다. nextest 0.9.137의 권장 버전 경고는 보존했다.
`cargo package --no-verify` 생성 뒤 별도 registry 소비를 실행한 결과이며 Cargo 자체 verify,
실제 crates.io·docs.rs 서비스, 하위 MSRV·새 target 검증으로 표시하지 않는다.

기존 generator 기대값은 #4964 → 원 보정 `f5f71f8c062719e73a70c2115f4265851e928758` →
통합 PR #5938 `bf30bd792b9198cec23ce5bb0f897f7d94bf6463`을 추적했다.
당시 manifest 0.8.4를 고정한 검사만 현재 manifest 소비로 바꾸며 metrics·provenance 검사는 유지했다.
렌더링 baseline·golden·래칫 허용치는 변경하지 않았다.

### 검증 입력의 커밋 확인

아래 8개는 검토 code head에 실제 포함돼 있다. 이 중 문서 입력 5개는 기존 공개 sample이고,
합성 암호·잘못된 입력·PDF의 출처와 생성 명령은
[fixture README](../../../tests/fixtures/issue8174/README.md)에 기록했다.
PDF의 Producer는 `rhwp`이며 독립 한컴 Print PDF가 아니다. private corpus는 사용하지 않았다.

| 입력 | bytes | SHA-256 |
| --- | ---: | --- |
| [pdf/issue8174/package-input.pdf](../../../pdf/issue8174/package-input.pdf) | 12,599 | `4d6fc1c51a36ad4e90a02d4f71834ef9c764a8098e6e3dba9395a0d6b8b01548` |
| [samples/143E433F503322BD33.hwp](../../../samples/143E433F503322BD33.hwp) | 73,216 | `1bcdc7a2de38680cdb715556f40c8477f8242131d28de949b09e945022bf35fa` |
| [samples/hwp_table_test.hwp](../../../samples/hwp_table_test.hwp) | 29,184 | `dc3e57d7577447ae4b47f59bdb0f499a5ac29066b62af8a650bcdd8d70eee32a` |
| [samples/hwpx/143E433F503322BD33.hwpx](../../../samples/hwpx/143E433F503322BD33.hwpx) | 169,537 | `1ee1e9045be4334fb9f940d7226cfda2884df9fecac6fa7fca76fb81c08a4a81` |
| [samples/hwpx/blank_hwpx.hwpx](../../../samples/hwpx/blank_hwpx.hwpx) | 12,707 | `cf7291b4aab53f25b48df4fce83e33db636fea8600beed369a5eb34880c803ef` |
| [samples/hwpx/hancom-hwp/blank_hwpx.hwp](../../../samples/hwpx/hancom-hwp/blank_hwpx.hwp) | 12,800 | `297a9b5f80e6ea590f1364d81fee287c781de49e4a656e16fef86f39c6756837` |
| [tests/fixtures/issue8174/invalid.hwp](../../../tests/fixtures/issue8174/invalid.hwp) | 24 | `7beaea096136a120b75751c0a5137bbd2829a513f4ef062cadfbc8d8ce8b9df5` |
| [tests/fixtures/issue8174/protected-test.hwpx](../../../tests/fixtures/issue8174/protected-test.hwpx) | 8,059 | `d818e196e35a3c2ee8fab8e0a4594f9ac1b8220554d19287f8fd41eb8b528410` |

## GitHub Actions

정확한 code head `7068bd7d8a369178a7f5421215dbefb64ab88248`에서 아래 workflow가 모두 성공했다.
CI run `39937953200`의 두 번째 실행에서 [Archive C](https://github.com/edwardkim/rhwp/actions/runs/39937953200/job/120382671748)는
**3,205/3,205 통과·17개 기존 제외**, [Build & Test 집계](https://github.com/edwardkim/rhwp/actions/runs/39937953200/job/120387307758)는 성공했다.
최종 check-run은 **32 success·5 skipped**이며 진행 중·실패한 검사는 없었다.
다른 성공한 job은 첫 실행 결과를 재사용했으며 전체 workflow를 다시 실행한 결과로 기록하지 않는다.

| Workflow | 정확한 code head의 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/39937953200) | 성공 — C·집계의 승인된 재실행 포함 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39937953093) | 성공 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39937953096) | 성공 |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39937953045) | 성공 |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39937953085) | 성공 |
| [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39937949232) | 성공 |

최초 Archive C는 checkout 17분 26초와 archive 다운로드 2분 42초 이후 테스트를 실행하다
30분 job 제한으로 취소됐다. 로그에서 3,205개 중 3,147개 PASS를 확인했고 FAIL 줄은 없었다.
이 실행은 성공으로 간주하지 않았다. 작업지시자가 승인한 job 단위 재실행만 수행했으며
성공한 다른 작업은 기존 결과를 유지했다. timeout·회귀 기대값·CI 구성을 완화하지 않았다.

## 조판 원칙 준수 검토

| 검토 항목 | 확인한 근거·비해당 사유 | 판정 |
| --- | --- | --- |
| 제출 전 로컬 검증과 PR 상태 | 위 필수 Rust·package·CLI 실행 완료 뒤 Open PR 제출. reviewer 지정 없는 본인 PR | 충족 |
| 기존 이슈·해결 PR의 선행 분석 | generator #4964→#5938 및 host #8097→#8135의 실제 통합 SHA·독립 base 실행·보정 결과를 위에 연결 | 충족 |
| 구현 근거와 일반성 | Cargo의 직접 registry 의존성과 표준 Error 계약에서 수정. 문서별·배치별 조건 추가 없음 | 충족 |
| 측정·배치 일관성 | 측정·조판·paint source 및 메트릭 변경 없음. SVG source는 같은 formatter로 base와 전 파일 일치 | 비해당 |
| 리베이스 반영 PR의 기능 보존 | 고정 base 이후 merge/rebase 없음. 기존 host 도구의 기능은 실제 software Vulkan 실행으로 별도 보존 확인 | 비해당 |
| 최종 출력 소비 계약 | 출력 backend 호출·font/resource·metric context 변경 없음. `export-text`는 같은 core 추출 사용. 기존 SVG PDF의 바이트 보존과 독립 host 실행을 확인 | 비해당 |
| 분할·이어받기 계약 | pagination·rowspan·clipping·continuation 및 컷·높이 소비 경로 변경 없음 | 비해당 |
| 줄 소속과 점유 높이 | LineSeg·줄 나눔·기준선·간격·문단 점유 로직 변경 없음 | 비해당 |
| 사례와 증거의 독립성 | 기준 devel/첫 후보/보정 도구 metadata 대조, 원 E0277, 외부 registry source 소비 및 CLI 바이트 대조. 합성 입력을 한컴 출력 근거로 사용하지 않음 | 충족 |
| 기준값 변경 | 렌더링 baseline/golden/허용치 변경 없음. generator version 계약 보정의 원 생산자·통합 이력을 위에 기록 | 비해당 |
| 회귀 추가·변경의 선행 조건 | 새 회귀는 공개 오류 전파 API 검사이며 조판·렌더링 회귀 추가·변경 없음 | 비해당 |
| 90% 통과 후 확대 판독 | 조판·렌더링 규칙 변경·한컴 충실도 개선 주장 없음. 실루엣 통과나 독립 Print 대조를 수행했다고 주장하지 않음 | 비해당 |
| 주장과 검증 범위 | 실제 archive·소비 앱·CLI·독립 host와 code head CI를 구분. 제품 배포·하위 MSRV·새 target·가속 GPU 보증은 남은 작업 | 충족 |

## 시각 증적과 남은 차이

조판·렌더링 로직에 영향을 주는 변경이 없어 Native/fresh WASM Visual Sweep·전쪽 TSV·
한컴 Print 확대 판독은 이번 변경의 필수 검증 범위에 해당하지 않는다. parser/model/serializer/
DocumentCore·PDF backend/decoder·font 로직은 그대로다. SVG fork는 source 동일성 대조와
같은 cwd의 실제 출력 보존으로 확인했다. CLI 입력 PDF에 독립 기준 출력의 역할을 부여하지 않는다.

실제 crates.io 업로드·owner/인증·게시 순서 확정·릴리즈 버전 전환과 MD01·MD05–MD09 등은
[결정 대장](../../plans/task_m100_8174_maintainer_decisions.md)의 후속 범위다.
기존 `chacha20 0.10.1` yank 경고도 제거하지 않았으며 이 검토를 보안 감사 결과로 해석하지 않는다.

## Merge 후 contributor PR comment 계획

본인 PR이며 시각 증적 기반 판정이 아니므로 외부 contributor 대상 review/overlay 안내는 비해당이다.
실제 merge가 승인되면 최종 head CI·SHA·병합 결과를 확인한 뒤 승인된 후속 절차를 적용한다.
현재는 merge·이슈 종료·실제 게시를 실행하지 않았다.

# PR #8285 검토 기록 — 분할 문단의 번호 자리표

## 최종 판정

**승인 — 원 기여를 보존한 메인터너 보정 head를 검증하고 원 PR을 병합했다.**
원 기여자 [semanticist21](https://github.com/semanticist21)의 문제 수정과 저장·재열기 회귀를
수용했다. 검토·보정은 [postmelee](https://github.com/postmelee)가 수행했다.

## 대상과 확정 이력

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8285](https://github.com/edwardkim/rhwp/pull/8285) |
| 원 기여 head | `0f6dc6841077c5ff15f404df3fa083645be475f2` |
| 첫 devel 통합 | `c940f0cb1bf5fc8391121d2fcc1e7a46d68cd846` |
| 별도 메인터너 보정 | `4b46edc0047395e6b4d22618d9bf80d2ec59d8ef` |
| 검증·원격 head | `52b89bf8ba047ab1e0092ae5b7ce1e4c2120ce87` |
| 전체 회귀 base | `28b58695cb01a20e980459e77a35f964817c7459` |
| head tree | `da14947a2afb3fb0fb9929d109662626ef7ae35e` |
| 병합 직전 devel | `7fdd5441ff1e77c083818e920a48089f294de35e` |
| 합성·실제 merge tree | `23bdfeb0df0be48875b1c0f4d013a90e7f7a6d5a` |
| 실제 merge | [`3a53945439862712be86764af73714827b15a44e`](https://github.com/edwardkim/rhwp/commit/3a53945439862712be86764af73714827b15a44e) |
| 병합 시각 | 2026-11-04 14:23:27 UTC |
| 승인 리뷰 | [5769917942](https://github.com/edwardkim/rhwp/pull/8285#pullrequestreview-5769917942) |

## 원 기여와 별도 보정

번호 넣기 바로 앞에서 문단을 나눌 때 번호 자리표 공백을 중복 계산해 저장 뒤 빈칸이 추가되는
문제를 수정하고 HWP/HWPX 저장·재열기 회귀를 추가한 기여다. 최신 devel과의 충돌을 해결하면서
현행 control 이동·앞 갭 보존 규칙을 유지했다. 실제로 새 문단으로 이동한 AutoNumber/NewNumber
자리표만 `char_count`에서 한 번 계산하도록 별도 보정했다.

원 기여자 commit을 rebase/amend/force-push하지 않았으며 일반 fast-forward push와 일반 merge로
보존했다. 보정은 이전 d39/a5 후보에서 확인한 최소 수정사항을 이력상 분리한 것이며, 최종 회귀의
실패 해결을 위해 새 제품·테스트 동작을 추가한 것이 아니다. 최종 제품 diff는
`src/model/paragraph.rs`와 `tests/cases/split_before_auto_number_placeholder.rs`, 81줄 추가·1줄 삭제다.

조판 원칙 검토의 적용 범위는 원본 번호의 8유닛 표현과 자리표의 단일 계상, 분할 뒤 control 소유와
저장·재열기 계약이다. 일반 로드·조판·renderer 경로의 품질 개선이나 문서별 배치 보정으로 확대하지 않았다.

## 로컬 전체 회귀와 재실행

Rust 1.93.1, macOS, 공용 `target/pr-review`, Cargo job 1로 순차 실행했다.
전체 nextest는 최소 글꼴 child-process 환경에서 테스트 스레드 2개로 실행했다.

| 실행 | PASS | FAIL | 종료 코드 |
| --- | ---: | ---: | ---: |
| 최초 전체 | 12,389 | 81 | 100 |
| 동일 ref의 입력 복원 후 실패 81개 재실행 | 80 | 1 | 100 |
| 추가 누락 HWP 복원 후 마지막 1개 | 1 | 0 | 0 |

동일 후보에서 합산 **12,470개가 각각 통과**, 미해결 0개이며 최초 전체의 45 skipped는 PASS에
포함하지 않는다. **단일 clean full-run PASS가 아니다.** 필터 재실행에서 제외된 테스트 수도
원 전체의 skipped 45개와 구분한다.

최초 81개는 런타임 입력/PDF/검증 JSON/기록 읽기·존재 확인 단계에서 중단됐다. 같은 후보 Git blob과
일치하는 141개 파일 11,260,462바이트, 마지막 HWP 1개 15,872바이트를 복원했다.
제품 코드·테스트·assertion·골든·글꼴 격리 정책은 변경하지 않았다. 원래 내용·배치·PDF 해시
assertion이 재실행에서 통과했다. 이전 후보의 집중 32 PASS를 이 후보의 실행 결과로 옮기지 않았다.

OS 기본 글꼴 237 entries와 라이선스 확인 저장소 글꼴 69개를 허용하고 개인·한컴 설치 글꼴을 차단했다.
기존 테스트 소유 dummy/허용 fixture 사본 예외는 그대로였다. macOS 기능 검증 성공을 Linux Landlock
보안 격리 자체의 검증이라고 표현하지 않는다.

## 필수 검사와 실행 명령

Native/WASM32 Clippy, workspace build/all-target Clippy, manifest 도구 23개, unit-tier 도구 12개,
고정 base 및 최신 base 정책, fmt·diff 검사가 통과했다. workspace build 첫 실행은 디스크 guard가
SIGTERM(-15)으로 중단했으며 공간 확보 후 별도 재실행에서 성공했다. 중단된 실행은 PASS가 아니다.

아래는 실제 명령의 공개용 표기다. 로컬 경로만 역할 변수로 바꿨다. `REVIEW_TARGET`은 기본 작업공간의
공용 target이며 `FONT_PROFILE`은 위 최소 글꼴 프로필이다. 이번 문서 작업에서 재실행한 명령은 아니다.

```sh
sandbox-exec -f "$FONT_PROFILE" cargo nextest run --locked --cargo-profile release-test --target-dir "$REVIEW_TARGET" --tests --test-threads 2 --no-fail-fast
cargo fmt --all -- --check
cargo clippy --locked --target-dir "$REVIEW_TARGET" -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir "$REVIEW_TARGET" -- -D warnings
cargo build --locked --workspace --target-dir "$REVIEW_TARGET"
cargo clippy --locked --workspace --all-targets --target-dir "$REVIEW_TARGET" -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check --base-ref "$REVIEW_BASE"
node scripts/rust-unit-test-tiers.mjs --check --base-ref "$REVIEW_BASE"
```

Cargo lint/build도 같은 최소 글꼴 wrapper 안에서 실행했다. lint/build의 dev debug symbols만 0으로
설정했으며 debug assertions나 제품 보안 정책은 완화하지 않았다.

## 저장·재열기와 독립 한컴 근거

검증 head의 rlib 출처·mtime·SHA-256을 확인한 helper로 공개 합성 문서를 분할·저장·재열기했다.
뒤 문단은 ` 뒤끝`, offsets `[0,8,9]`, `char_count=11`, 번호 위치 0이며 HWP/HWPX 재열기가 같은
모델 상태를 유지했다. 두 형식의 SVG/layer는 분할 직후와 바이트 단위로 같고 모든 상태가 1쪽이다.

| 자료 | 바이트 | SHA-256 |
| --- | ---: | --- |
| 저장 HWPX / 기존 한컴 입력 `split-number-after.hwpx` | 7,395 | `125b1c2f0c8224579cd4d9dc98e1d4c81b48fc50f596953f9dce4f7b8cca2e27` |
| 기존 `split-number-after-2020.pdf` | 9,936 | `563a5891d38e9b7186d02674199f094011dd65d6e5d03ec2a6f4c94000c4dc10` |

후속 기록 작성 시 실제 보존 파일을 다시 해시해 위 값과 일치함을 확인했다. 기존 sanitized receipt는
engine/profile `2020`, Hancom `11.0.0.9136`, `pdf_print_method=0`,
`pdf_output_mode=hancom2020_pdf_driver_one_up`을 기록했다. 이는 status.result 실행 증거이며
PDF Creator나 target 이름만으로 Print라고 판단하지 않았다. 영수증의 검증 범위는 1쪽, 595×841pt,
96dpi 래스터에서 `앞`과 `1뒤끝` 확인 및 다운로드 해시·크기 일치다.

자료는 로컬에 보존되어 있으며 이 문서 전용 변경에 HWPX/PDF/PNG를 새로 포함하지 않는다.
현재 저장 HWPX가 기존 독립 Print 입력과 같아 재변환하지 않았다. 새 Native Skia/fresh WASM 전쪽
시각 점수·overlay·확대 판독을 측정했다고 주장하지 않는다. 저장 모델·SVG/layer의 자기 일관성과
독립 PDF의 내용 확인, 새 head Render Diff를 각각의 범위로 구분한다.

## 새 head의 GitHub 검증과 병합

| 검사 | 실행 | 결과 |
| --- | --- | --- |
| CI / Build & Test | [41833357170](https://github.com/edwardkim/rhwp/actions/runs/41833357170) | 성공 |
| Render Diff | [41833357137](https://github.com/edwardkim/rhwp/actions/runs/41833357137) | 성공 |
| CodeQL | [41833357208](https://github.com/edwardkim/rhwp/actions/runs/41833357208) | 성공 |
| Proptest | [41833357132](https://github.com/edwardkim/rhwp/actions/runs/41833357132) | 성공 |
| Adapter inter-diff | [41833357057](https://github.com/edwardkim/rhwp/actions/runs/41833357057) | 성공 |
| CI Impact Policy | [41837568587](https://github.com/edwardkim/rhwp/actions/runs/41837568587) | 성공 |

병합 직전 devel의 추가 변경은 문서뿐이었고 최신 base 정책 검사를 다시 통과했다.
합성 tree와 실제 merge tree가 일치했다. [병합 후 duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/41847073484)도
성공했다. 기존 보고서·producer의 병합 전 상태는 위 공개 승인·실제 병합 기록으로 갱신해 해석했다.
함께 반영하는 기록은 [#8203](pr_8203_review.md), [#8289](pr_8289_review.md)다.

## Merge 후 contributor PR comment 계획

별도 감사·병합 결과 댓글은 작업지시자 확인 전 게시하지 않는다. 게시할 경우 원 기여의 문제 발견과
회귀를 먼저 인정하고, 최신 devel의 control 이동 규칙을 유지하기 위한 별도 보정 이유를 설명한다.
위 실제 merge와 검토 문서를 연결하고 합산 PASS 및 시각 검증 한계를 존댓말로 적는다.
공개하지 않은 로컬 PDF/PNG URL이나 다른 PR의 이미지를 증거로 사용하지 않는다.

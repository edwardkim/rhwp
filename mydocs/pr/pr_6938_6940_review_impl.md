# PR #6938/#6940 누적 체리픽 검토

## 최신 결론 (2026-09-09)

- [#6938 review](pr_6938_review.md): **메인터너 보정 후 수용 가능**. 기반 클래스/역참조, 빈 VtPicture, VtString 예약 바이트를 보정하고 원 HWP 및 추가 mixed_chart.hwp/hwpx의 빨간 선·청록 막대·이중축 복원을 직접 확인했다. 기존 단일축 폴백 보류 사유는 해제했다.
- [#6940 review](pr_6940_review.md): 각주 numbering/빈 장식 문자 보존 범위 승인. #6941의 인라인 사용자 문자 잔여는 별개다.
- 최종 문자열 보정 전 전체 회귀 9,346 passed / 46 skipped와 Clippy 3종 성공, 보정 후 metadata probe/CLI build/세 대표 페이지 직접 판독을 구분한다. 최신 코드의 전체 회귀가 다시 통과했다고 주장하지 않는다.
- 작업지시자는 증적 보관, review 갱신, 최신 upstream/devel 리베이스 후 PR 생성을 승인했다. 오늘할일 충돌은 해결하고 나머지 충돌이 없으면 추가 테스트를 실행하지 않는다. 최신 PR head CI와 merge 승인은 별도다.
- 차트 범례/격자/외곽선/글꼴·배치 및 원 문서 19/18쪽 차이는 남아 있다. 의미 복원 범위의 수용이며 전체 렌더 완전 일치가 아니다.

## 메인터너 보정과 증적 보관

- 원격 통합 PR: [#6951](https://github.com/edwardkim/rhwp/pull/6951), base `devel`. [Self-review](archives/pr_6951_review.md)는 같은 PR의 문서 전용 후속 기록이다.
- 보정/증적 commit `d3f8dff09`를 만든 뒤 `upstream/devel` `144c224193f5508a66a7dc374036995ec6de2738`로 리베이스했다. 오늘할일을 포함해 충돌이 없었다. 리베이스 후 code candidate는 `e0dc1bc8a92ccd5d940419765d2e51bfe0a4d697`이다.
- 명시 지시대로 추가 테스트 없이 위 candidate를 push하고 PR을 생성했다. 리베이스 전 직접 검증과 최신 PR head CI를 구분하며 이 기록에서 CI 성공/merge 완료를 주장하지 않는다.

- 코드: src/ole_chart/{grid,parser,mod,legacy_presentation,legacy_combo_renderer}.rs 및 src/renderer/layout/shape_layout.rs.
- 입력/기준: samples/issue6938/mixed_chart.hwp, mixed_chart.hwpx, mixed_chart.pdf를 보존하고 기존 기준 PDF 3개를 재사용한다. 추가 PDF를 출력하지 않았다.
- 대표 증적: mydocs/pr/assets/pr_6938_maintainer_20260909/의 원 HWP 3쪽/추가 HWP·HWPX 1쪽 PNG 3개. 기존 pr_6938_6940_20260909/의 원 head 실패 1개와 #6940 각주 2개는 비교 근거로 보존한다.
- 로그, 중간 SVG/raster/JSON, 임시 probe, 불필요한 PDF와 파생 suite는 커밋하지 않는다. 최신 중간 산출물은 output/pr_6938_maintainer_20260909/에 둔다.
- source·바이너리·입력·PDF·대표 PNG SHA와 정확한 검증 범위는 #6938 review의 최신 절을 따른다. 아래 최초 체리픽 검증 이력은 그 당시 후보의 결과다.

## 기준과 순서

기준은 `upstream/devel` `9a96eef92458112d5d7998f4b3390261c0e3c811`, branch는 `review/planet6897-6938-6940-20260909`다. 기본 작업공간 `/home/tsjang/rhwp`에서 최신 devel을 fetch했고 시작 worktree는 clean이었다. 두 원 PR의 최신 CI 성공을 확인하고 reviewer `jangster77`을 지정했다.

| 원 PR / 원 commit | 적용 commit | 결과 |
| --- | --- | --- |
| #6938 `893be62716f05378251e93264d6805bd67dd0d06` | 기준 devel에 patch-equivalent 변경 존재 | 중복 제외 |
| #6938 `00a6f37516d3ef00e11ee69a5bee35df28460a68` | `d95e51875ec8311f53bbc66192f26af41c3fd878` | 충돌 없음 |
| #6938 `a343125084db800bdd7bbb8b719ee8b2158665cd` | `b48e57d9f503c93e0609aab2f2dc80654e719c9a` | 충돌 없음 |
| #6940 `c15df158e793aed6bf81c4c9da3d1abf9bf0e060` | `2ca81c05870b610cdd8b5812f43c13c30ab26727` | 충돌 없음 |
| #6940 `52660ccb2c322b17b520103af06e3b5760899542` | `71783397d478cf383be933e03d48cff0bad84f77` | 충돌 없음 |

실질 변경 commit은 모두 출처를 보존하는 `git cherry-pick -x`로 적용했다.

## 최초 체리픽 후보의 로컬 검증 (보정 전 이력)

검증 code head는 `71783397d478cf383be933e03d48cff0bad84f77`이다. 아래 명령을 고정 `target/pr-review`에서 순차 실행했다. Linux 16 logical CPU / RAM 15 GiB에서 전체 nextest는 8 threads를 사용했다. `CARGO_INCREMENTAL=0`은 지정하지 않았다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
  --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review \
  --tests --test-threads 8 --no-fail-fast \
  -E 'test(/ole_chart|issue6872|issue_6872|issue6922|issue_6922|issue2742|footnote_endnote_numbering/)' \
  --status-level fail --final-status-level fail
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review \
  --tests --test-threads 8 --no-fail-fast --status-level fail --final-status-level fail
```

- lint/build/manifest/policy: 모두 exit 0. manifest 48/48 targets, source-side 4,205 tests.
- focused: 53 passed, 0 failed, 필터 제외 9,339, 0.312초. run `39d109bb-6662-4d6e-819a-49f32b3d48a7`.
- 전체: 9,346 passed, 0 failed, 46 skipped, 3 slow, 409.746초. run `c7d03fa4-e244-4541-bd8e-8b1bac54e004`.
- 별도 최소 입력: #6938 역참조 단언 exit 101, 재귀 기반 클래스 stack overflow exit 134. 기존 테스트에 없는 실패 경로이며 전체 회귀 성공과 구분한다.
- 신규/변경 sample 없음: 신규 sample 보안 sweep 및 sample 추가에 따른 baseline 재생성 대상 없음.
- 이번 변경은 Rust parser/model/serializer 범위이며 renderer/WASM source 자체는 바꾸지 않았다. native-skia 전체와 별도 WASM bundle 빌드는 반복하지 않았고, WASM32 Clippy 및 원 PR별 CI의 실행/skip을 개별 review에 구분했다.
- 진단/렌더 바이너리는 위 workspace build가 만든 `target/pr-review/debug/rhwp`이며 SHA-256은 `24eb2761ed87e2b947df4d18af4b5e09b9f6c7a2fa9114f25a2a2421af58eedf`다.

## 시각 검증 명령과 범위

```bash
RHWP_BIN=target/pr-review/debug/rhwp \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 17 \
  --source /home/tsjang/Downloads/korea_downloads/korea_policy_downloads/148735526_2012년_6월_소비자물가동향.hwp \
  --reference-pdf pdf/pr6938-148735526-2020.pdf --label pr6938-chart \
  --reference-grade 'Hancom MCP engine 2020' --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-review-6938-6940/visual/chart-ledger
venv/bin/python scripts/visual_sweep.py --key pr6938-chart \
  --hwp /home/tsjang/Downloads/korea_downloads/korea_policy_downloads/148735526_2012년_6월_소비자물가동향.hwp \
  --pdf pdf/pr6938-148735526-2020.pdf --page 3 \
  --rhwp-bin target/pr-review/debug/rhwp \
  --out /tmp/rhwp-review-6938-6940/visual/chart-sweep
```

실행 당시 임시 출력은 `pdf/pr_6938_6940_20260909/`였으며 완료 후 위 `/tmp/` 경로로 옮겼다. 재현 명령의 출력 경로는 최종 배치에 맞췄다.

#6940은 `export-hwpx`로 통합 head 왕복본을 만든 뒤 원본/왕복본 각각 MCP engine 2020 PDF를 생성했다. 전체 `pdftotext -layout` 비교와 96 DPI 대표 10쪽/13쪽 raster 비교를 수행했다. 명령, 원본/기준/왕복 해시, job id, 실제 판정과 한계는 개별 review에 있다. 서버 주소나 인증 정보는 기록하지 않는다.

## 커밋할 증적과 제외 범위

- 기준 PDF: `pdf/pr6938-148735526-2020.pdf`, `pdf/pr6940-156584446-source-2020.pdf`, `pdf/pr6940-156513948-source-2020.pdf`.
- 대표 PNG: `mydocs/pr/assets/pr_6938_6940_20260909/pr6938-chart-p003-review.png`, `pr6940-note-a-p010-review.png`, `pr6940-note-b-p013-review.png`.
- 원 PR의 원래 문서/이미지는 기여 commit에 보존하고, 새 검토 기록과 오늘 할 일을 같은 branch에 둔다.
- 왕복 HWPX/PDF, raw raster, 중복 compare/overlay/review/contact sheet, SVG, render tree/metric/MCP JSON, 로그, 임시 Rust 하네스와 generated suite/manifest는 추가 커밋에 포함하지 않는다. 검토 중간 파일은 `/tmp/rhwp-review-6938-6940/`에서 관리한다.

## 최초 검토 당시 후속 계획 (보정 완료 여부는 위 최신 결론 우선)

1. #6938의 기반 클래스 검증과 역참조 복원/거부를 제한된 메인터너 보정 commit으로 해결하고, 새 head의 lint/회귀를 검증한다.
2. #6938의 차트 종류/계열별 축/연월 라벨을 보존하거나 명시적인 미지원으로 처리한다. 이중축 복합 차트를 임의의 단일축 막대로 표시하는 상태는 수용하지 않는다. 한컴 기준과 직접 재대조하고 #6922 close 범위를 실제 결과에 맞춘다.
3. 원격 통합을 진행할 때는 최신 대상 head CI를 확인한다. review 보완은 동일 통합 PR의 trailing 문서 commit에 포함하며 검토 기록만의 별도 PR을 만들지 않는다.
4. 실제 merge 뒤에만 개별 review의 contributor comment 계획과 `post_merge.md` 순서를 따른다. 현재 단계에서는 원 PR/이슈 close, 원격 merge, 브랜치 삭제를 실행하지 않았다.

# Stage 2 — task_m100_4161 TDD 수정 (green)

- **이슈**: [#4161](https://github.com/edwardkim/rhwp/issues/4161)
- **계획서**: [`mydocs/plans/task_m100_4161.md`](../plans/task_m100_4161.md)
- **선행**: [`task_m100_4161_stage1.md`](task_m100_4161_stage1.md) — red 원문·계측·base_size 제외 확정
- **브랜치**: `task_m100_4161` (분기 기준 `upstream/devel` `0bc05ef81`)
- **작업 시각**: 2026-08-18 KST
- **수정 커밋**: `08901bb6f` `fix(model): CharShape 기본 장평을 OWPML 기본값 100 으로 (#4161)`

## 1. 구현

- `src/model/style.rs` `impl Default for CharShape` — `ratios: [0; 7]` → `[100; 7]`
  (스키마 근거 인라인 주석 추가). **프로덕션 변경은 이 한 줄이다.**
- 같은 파일 impl doc 주석 — "왜 ratios·base_size 는 그대로 두는가" 절을
  "장평 ratios (#4161)" + "왜 base_size 는 그대로 두는가"(stage1 §5 의 3논거)로 교체.
- 잠금 테스트 `char_shape_default_matches_spec_only_for_relative_sizes_and_shade` →
  `char_shape_default_matches_spec_except_base_size` 로 개명. `ratios == [100; 7]` 반전,
  `base_size == 0` 단언은 유지하고 사유를 프로버넌스 선행 요건으로 교체.
- `#[cfg(test)]` 줄 이동에 따라 `node scripts/rust-unit-test-tiers.mjs --generate` 후
  `--check` 통과 — `tests/suites/unit-test-tiers.json` 기준선 재계산 포함.

## 2. red → green 전이

Stage 1 red (5/5 실패, `finished in 12.19s`) → 수정 후:

```text
test issue_4161_ratio_default_contract::hml_roundtrip_without_ratio_child_emits_valid_ratio ... ok
test issue_4161_ratio_default_contract::public_document_core_export_also_emits_valid_ratios ... ok
test issue_4161_ratio_default_contract::so_sueop_convert_ratios_within_valid_range ... ok
test issue_4161_ratio_default_contract::hwp3_export_hwpx_emits_valid_hh_ratio ... ok
test issue_4161_ratio_default_contract::hwp3_convert_emits_valid_ratios_for_every_sample ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 81 filtered out; finished in 10.68s
```

## 3. 유닛·정적 검사

| 게이트 | 명령 | 결과 |
| --- | --- | --- |
| lib 유닛 전체 | `cargo test --lib` | **통과** — 3,886+165+15+2 = 4,068건, 실패 0 (`$TMPDIR/task_m100_4161_lib_test.log`) |
| 포맷 | `rustfmt --edition 2021 --check` (변경 2파일, LF 정규화 후) | **통과** |
| 유닛 티어 | `node scripts/rust-unit-test-tiers.mjs --check` | **통과** (재생성 후) |
| manifest 규칙 | `node --test scripts/tests/rust-test-suite-manifest.test.mjs` | 15/16 통과 — 실패 1건은 아래 호스트 아티팩트 |

**호스트 특이사항 (검토 기록)**:

- 이 호스트는 `cargo fmt --all` 이 오류(206)로 실패해 rustfmt 를 파일 직접 지정으로 실행했다.
  또한 autocrlf 체크아웃(CRLF)은 rustfmt 가 조용히 건너뛰므로 변경 .rs 를 LF 로 정규화한 뒤
  검사했다(index 는 LF 라 diff 无영향). CI 의 `cargo fmt --all -- --check` 는 Linux(LF)에서
  정상 검사된다.
- manifest 규칙 테스트의 실패 1건(`CI lint checkout은 … fetch-depth: 0`)은
  `.github/workflows/ci.yml` 을 LF 정규식으로 검사하는데 autocrlf 작업 트리가 CRLF 라 깨지는
  **호스트 아티팩트**다 — `git ls-files --eol` 실측 `i/lf w/crlf`, 해당 파일 미변경.
  본 변경과 무관하며 CI(Linux)에서는 통과한다.

## 4. 렌더 무회귀 실측 (수정 전후, 같은 debug 프로필 바이너리)

| 증적 | before | after | 판정 |
| --- | --- | --- | --- |
| exambank HML 왕복 `<RATIO>` | `Hangul="0" …` ×2 | `Hangul="100" …` ×2 | **정합 수정 실물** |
| exambank `export-svg` | — | — | **byte-identical** (`diff` 무출력) |
| `samples/SO-SUEOP.hwp` `export-pdf` (46쪽, 5,034KB) | — | — | **byte-identical** (`cmp`) |
| exambank `render-diff --via hwpx` (수정 후) | — | 최대 변위 0.00px, 구조 불일치 0 | **PASS** |

폭 경로 5곳의 `ratio > 0.0 ? ratio : 1.0` 폴백이 0→1.0 과 100→1.0 을 같은 값으로 수렴시킨다는
계획서 예측이 바이트 수준으로 적중했다. 산출물은 scratchpad `task_m100_4161_evidence/{before,after}/`.

## 5. Stage 2 게이트 판정

| 항목 | 기준 | 결과 |
| --- | --- | --- |
| red→green 전이 | 계약 5건 실패→통과, 코드 수정은 기본값 1줄 | **통과** |
| 잠금 테스트 갱신 | ratios 반전 + base_size 경계 유지 | **통과** |
| lib 유닛 무회귀 | 실패 0 | **통과** (4,068건) |
| 렌더 무회귀 (경량) | SVG/PDF 전후 동일 | **통과** — byte-identical |
| 생성물 오커밋 게이트 | `Cargo.toml`(generated)·`tests/generated/` 미커밋 | **통과** (`git status` 확인) |

## 6. Stage 3 로 넘기는 것

- release-test 전체 nextest, clippy `--all-targets -D warnings`
- Native Skia 3종, Docker WASM (renderer lane)
- 골든 SVG 7건 무변동 실측, IR field sweep 빈 diff
- 최종 보고서·PR (push·PR 생성은 작업지시자 승인 후)

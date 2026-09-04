# Stage 3 — task_m100_4161 회귀 판정·증적

- **이슈**: [#4161](https://github.com/edwardkim/rhwp/issues/4161)
- **계획서**: [`mydocs/plans/task_m100_4161.md`](../plans/archives/task_m100_4161.md)
- **선행**: [`stage1`](task_m100_4161_stage1.md) red·계측, [`stage2`](task_m100_4161_stage2.md) green
- **브랜치**: `task_m100_4161` (분기 기준 `upstream/devel` `0bc05ef81`)
- **작업 시각**: 2026-08-18 KST
- **적용 lane**: `local_validation.md` §4.3 — "Rust parser/model/CLI" + "renderer/layout/typeset/WASM"
  두 행 합집합 (모델 수정이지만 `renderer/style_resolver.rs` 가 `ratios` 를 소비)

## 1. 게이트 실행 결과 (전부 이 호스트, 2026-08-18)

| # | 게이트 | 명령 | 결과 |
| --- | --- | --- | --- |
| 1 | 포맷 | `rustfmt --edition 2021 --check` (변경 .rs, LF 정규화 후 — 호스트 특이사항 stage2 §3) | **통과** |
| 2 | manifest 규칙 | `node --test scripts/tests/rust-test-suite-manifest.test.mjs` | 15/16 — 실패 1건은 CRLF 호스트 아티팩트 (stage2 §3, 본 변경 무관) |
| 3 | 유닛 티어 | `node scripts/rust-unit-test-tiers.mjs --check` | **통과** |
| 4 | clippy | `cargo clippy --all-targets -- -D warnings` | **통과** — 경고 0 |
| 5 | 신규 계약 | `issue_4161_ratio_default_contract` 5건 | **통과** (stage2 §2 — red→green) |
| 6 | release-test 전체 | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | **통과** — `Summary [388.952s] 6892 tests run: 6892 passed (8 slow), 38 skipped` |
| 7 | Native Skia ① | `cargo test --profile release-test --target-dir target/pr-review --features native-skia skia --lib` | **통과** — 58 passed; 0 failed |
| 8 | Native Skia ② | `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- …native-skia` | **통과** — `Summary [2.024s] 2 tests run: 2 passed` |
| 9 | Native Skia ③ | `node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- …native-skia` | **통과** — `Summary [0.326s] 4 tests run: 4 passed` |
| 10 | WASM | `docker compose --env-file .env.docker run --rm wasm` | **통과** — `[INFO]: :-) Done in 13m 52s … pkg is ready` |

전체 로그: `$TMPDIR/task_m100_4161_full_test.log`(+`.err.log`), `…_skia.log`, `…_wasm.log`,
`…_clippy.log`, `…_lib_test.log`.

## 2. 골든·baseline 판정

- **골든 SVG**: `svg_snapshot` 8건(스냅샷 7 + 프로세스 내 결정성) **전건 PASS, 갱신 0건** —
  release-test 전체 실행(#6)에 포함. "골든 7건은 전부 ratio 명시 표본이라 무변동" 예측 적중.
  조건부 커밋 C5(골든 재생성)는 **불발동**.
- **IR field sweep**: `ir_field_sweep_baseline::ir_field_sweep_does_not_regress`
  **PASS (321.364s)** — baseline tsv 에 ratio 열이 없어 무영향 예측대로 빈 diff.

## 3. 시각 증적 (contributor 경로 — working 문서 + PR 본문, `mydocs/pr/assets/` 미사용)

stage2 §4 실측의 재요약 + 추가 확인:

| 증적 | 판정 |
| --- | --- |
| exambank HML 왕복 `<RATIO>` 0→100 | 정합 수정 실물 (diff 원문 stage2 §4) |
| exambank `export-svg` 전후 | **byte-identical** |
| SO-SUEOP `export-pdf` 46쪽 전후 | **byte-identical** |
| exambank `render-diff --via hwpx` | PASS — 최대 변위 0.00px, 구조 불일치 0 |
| after HWPX `hh:ratio` 분포 | 100×6,846 / 95×6,531 / 90×3,969 / 97×238, **0 = 0건** — 실데이터 편차 보존 + placeholder 만 정상화 |

**강조점·CanvasKit 정책 4곳** (계획서가 지목한 산출 변화 가능 지점): 강조점+RATIO 부재 조합은
실표본·HML 경로(리더가 강조점 미파싱)로 **도달 불가**라 시각 증적 대신 정적 판독으로 관측한다 —
4곳 모두 ratio=0 을 오판하던 종전 동작(점이 글자 왼쪽에 붙음 / 유효 geometry 를 무효 판정 /
장평 효과 오검출)의 정상화이며, canvaskit_policy·layout 유닛(#6·lib 4,068건)이 전부 green 이다.

## 4. 실물 확인 번들 (한컴 판정 핸드오프 — 선택)

`output/task_m100_4161/` (gitignore 대상, 로컬 전용):
before/after 왕복 HML 쌍, 수정 후 SO-SUEOP HWP5·HWPX 변환본, 판정 안내 README.
한컴 변환 PDF 를 같은 폴더에 넣으면 144DPI 판정을 걸 수 있다.

## 5. Stage 3 게이트 판정

| 항목 | 기준 | 결과 |
| --- | --- | --- |
| 두 lane 합집합 게이트 | §1 의 10항 전부 | **통과** (호스트 아티팩트 1건 기록) |
| 골든 무변동 | 갱신 0건 또는 판독 후 별도 커밋 | **통과** — 갱신 0건 |
| 시각 증적 | 전후 비교 + render-diff | **통과** — byte-identical |
| 생성물 오커밋 | Cargo.toml(generated)·tests/generated/ 제외 | **통과** |

## 6. 남은 것

- 최종 보고서 `mydocs/report/task_m100_4161_report.md`
- **push·PR 생성은 작업지시자 승인 후** (base `devel`, `closes #4161`)
- 후속 이슈 제안: `base_size` 기본값 프로버넌스 (stage1 §5 계측 근거)

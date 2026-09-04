# Task M100 #4964 — Stage W6-5 통합 불변식·renderer 검증

- **수행계획**: [`../plans/task_m100_4964.md`](../plans/archives/task_m100_4964.md)
- **선행 단계**: [`task_m100_4964_w6_stage4.md`](task_m100_4964_w6_stage4.md)
- **기준 source**: `upstream/devel@d1ad0eb8784dbc55f0796e2ba8775f7363247b91`
- **검증 HEAD**: `43850c620`
- **검증일**: 2026-08-23 KST
- **판정**: 통과

## 1. 결론

W6에서 분리한 595개 historical-generated metric과 5개 measured overlay는 분리 전 600개 논리
배열의 순서, first-match, style fallback, 모든 저장 폭과 renderer 출력을 바꾸지 않았다. 공개 7개
fixture 167쪽의 native/WASM SVG는 전부 byte-identical이었다.

계보 manifest는 595개 historical generated 항목의 source-exact성을 추정하지 않는다. 5개 overlay는
#2430 측정 원문과 475/475로 일치하며, 600개 모두 W1 metric entry에 연결되고 W5 Oracle Profile은
관찰 범위가 있는 2개 항목에만 연결된다.

Clippy가 generator의 UTF-16BE byte 길이 검사에 `manual_is_multiple_of` 1건을 발견했다. `% 2 != 0`을
`!is_multiple_of(2)`로 바꾸는 의미 보존 정정을 적용했고 format·Clippy·generator contract를 다시
통과시켰다. metric data와 runtime renderer는 이 정정으로 바뀌지 않았다.

## 2. 보호 불변식 판정

| 불변식 | 결과 | 근거 |
| --- | --- | --- |
| W6-I01 항목 수·index | 통과 | 600개, index 0..599 |
| W6-I02 물리·논리 순서 | 통과 | composition hash 불변 |
| W6-I03 first-match | 통과 | exhaustive lookup projection 불변 |
| W6-I04 style fallback | 통과 | Rust unit·lookup projection 불변 |
| W6-I05 data-bearing 폭 | 통과 | 7,062,099 entry-codepoint pair hash 불변 |
| W6-I06 미지원 문자 | 통과 | range topology·boundary contract 통과 |
| W6-I07 native/WASM | 통과 | 7문서 167쪽, mismatch 0 |
| W6-I08 provenance | 통과 | 595 unknown을 exact로 승격하지 않음 |
| W6-I09 overlay | 통과 | #2430 5 face × 95 = 475/475 |
| W6-I10 결정론 | 통과 | baseline·manifest·canary contract 재실행 통과 |
| W6-I11 privacy/license | 통과 | 신규 font bytes·private corpus 식별 자료 0 |
| W6-I12 generator ownership | 통과 | core·overlay·canonical canary overwrite 거부 |

분리 전후 의미 hash는 다음과 같다.

| projection | SHA-256 |
| --- | --- |
| composition | `d4cdac86b3c6ee55d8b1aa921d662f1fc1241c2809cb9c8ffe991d56a045e69a` |
| metric data | `025812eac4bad179c5b87e23b15fdf08a4e4fb3f19a6e453738e03110a140bcf` |
| width | `2cd1389a14401f6488041af3c54ff0ba5e982d944acd0b5bb56147056e3a7d1b` |
| lookup | `bb3008f9dc379bd580119a6a658388732e94358db2039dbb02d78c28ec992fdf` |
| manifest entries | `054f4725162ddc95c4b00e00186955b1d7f10599d401f66f75b6dd52a1147032` |

## 3. 검증 결과

| 검증 | 결과 |
| --- | --- |
| W1·W5·lineage·generator Node contract | 39/39 통과 |
| baseline `--check`·manifest `--check-manifest` | 통과 |
| #2430 measured overlay verify | 5 face, 475/475 exact |
| Rust `font_metrics` unit | 9/9 통과 |
| full nextest | 8,073/8,073 통과, 정책 skip 39 |
| Native Skia library | 통과 |
| #2225 Native Skia fixture | 2/2 통과 |
| direct PDF fixture | 4/4 통과 |
| native release build | 통과 |
| Docker WASM build | 통과 |
| native/WASM SVG | 7문서, 167쪽, mismatch 0 |
| Clippy `--all-targets -D warnings` | 통과 |
| Rust doc tests | 8 통과, 3 ignore |
| unit-tier policy | 4,225 tests / 299 modules, 통과 |
| `cargo fmt --all`·`--check` | 통과 |

비교 문서별 쪽수는 `exam_kor` 20, `exam_eng` 8, `exam_math` 20, `exam_science` 4,
`synam-001` 35, `aift` 74, `2010-01-06` 6이다. 실행 보고서는 ignored
`output/svg-native-wasm-diff/report.json`에 생성했으며 절대 경로를 포함하므로 제출하지 않는다.

검증 산출물은 다음 해시로 고정했다.

| 산출물 | SHA-256 |
| --- | --- |
| native `rhwp` | `cc154c20d01bcbc0f27cd92656581d0d37dd5dc162766982d5e2ab887b97cc6a` |
| `pkg/rhwp.js` | `6d7b217b66e6f9c40c4b205b41973df2aad8100075313f18505d8f18bedf2b21` |
| `pkg/rhwp_bg.wasm` | `6874903775e91d8538bbb90f6a7559a7bbc0a759b7fc1edb132334a8a36f4e5d` |
| current generator source | `70dbec5c8d730fe3611a19232412f4b16d00719ca08fef7daafe3c3ddf8bb36b` |

환경은 Node `v24.15.0`, Rust `1.93.1`, Docker `29.7.2`였다.

## 4. 검증 환경에서 발견한 두 경계

### 4.1 공유 target의 compile-time checkout 경로

첫 full nextest에서는 `rhwp-contracts`의 정책 문서 경로 검사 1건만 실패했다. 같은
`target/pr-review`에 남은 과거 review worktree 바이너리가 `env!("CARGO_MANIFEST_DIR")`에
`/tmp/rhwp-4962-pr-review...`를 포함하고 있었다. 단독 `cargo test`는 다른 `release/` 프로필을
갱신하므로 full nextest의 `release-test/` 바이너리를 고치지 못했다.

`cargo clean -p rhwp-contracts --target-dir target/pr-review --profile release-test`로 해당 패키지의
파생 캐시 26개(4.7 MiB)만 제거한 뒤 전체 suite를 현재 checkout에서 다시 링크했다. 선택된 바이너리의
경로가 현재 checkout임을 확인했고 full nextest 8,073/8,073이 통과했다. 이는 W6 source 회귀가 아니라
compile-time 절대경로와 checkout 사이에서 공유한 target cache의 계보 문제다.

### 4.2 generated suite 배치 drift

`node scripts/rust-test-suite-manifest.mjs --check`는 이 task branch의 ignored generated suite
001..032가 현재 manifest 배치와 다르다고 보고했다. source case나 Cargo target을 W6가 변경한 것은
없다. 프로젝트 규칙상 `--prepare` 산출물은 review worktree·CI 검증 증적이며 source PR에 포함하지
않으므로 이 브랜치에서 재생성·stage하지 않았다.

표준 실행기는 현재 manifest의 #2225→suite 025, direct PDF→suite 030을 선택했지만 기존 generated
파일에는 각각 suite 005와 007에 실물이 있었다. 동일 nextest 정규식 필터를 실제 배치에 직접 적용해
2/2와 4/4를 검증했다. PR review worktree와 CI에서는 `--prepare` 후 manifest `--check`를 다시 수행해야
한다.

## 5. 잔여 unknown과 Stage 판정

lineage summary는 600개 안정 ID, historical generated 595개, measured overlay 5개다. 완전한
source-exact 항목은 0개이고, 추적 Noto Sans KR source 1개만 printable ASCII 범위에서 부분 검증됐다.
595개 unknown origin은 W6 미완료가 아니라 증거 없는 exact 승격을 막는 명시적 결과다.

W6의 목표였던 행동 보존 분리, 기계 판독 계보, generator ownership과 공개 native/WASM 검증을 모두
충족했다. W7은 이 manifest를 runtime 정책으로 곧바로 가져오지 말고, 승인된 canonical registry와
backend projection 설계의 입력으로 사용해야 한다.

# PR #6949/#6952 누적 체리픽 검토

## 통합 merge 확정 기록 (2026-09-09)

- [통합 PR #6957](https://github.com/edwardkim/rhwp/pull/6957)은 2026-09-09 14:41:08 UTC에 일반 merge로 통합됐다. merge SHA는 `d43937e0de7cf465185d23ce6b06fa47e7824e57`, 승인한 PR head는 `4e422a57d6775eb2f11dffb70b37632823659829`다.
- merge 직전 최신 head는 `MERGEABLE / CLEAN`이었다. [Build & Test 및 Rust/Lint/Native Skia](https://github.com/edwardkim/rhwp/actions/runs/34363253946), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34363253722), [CodeQL 언어별 분석](https://github.com/edwardkim/rhwp/actions/runs/34363253980), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34363253961), [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34363253986)가 성공했다. GHAS CodeQL check는 `NEUTRAL`, [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/34365098894)는 `SUCCESS`였고 대기·실패 항목은 없었다.
- review·오늘할일·최종 PDF 2개·대표 PNG 5개는 코드 PR head에 이미 포함되어 merge됐다. source PR review 3개는 이번 문서-only 후속 처리에서 archive로 이동한다. 오늘할일에는 새 운영 항목을 반복 추가하지 않고 이동된 링크만 보정한다.
- 리베이스 후 추가 로컬 테스트·PDF 출력은 사용자 지시대로 생략했다. 이번 GitHub Full CI 성공은 이전 로컬 검증과 별도 근거다. devel push의 post-merge CI는 PR head CI와 구분하며 이 기록에서 성공을 선언하지 않는다.
- 후속 문서 PR 완료와 최종 devel sync 뒤 #6872/#6941을 해결 범위 내에서 종료하고 #6949/#6952를 통합 대체로 종료할 계획이다. #6865는 회색조·아이콘·영문 배너 잔여를 기록하고 OPEN을 유지한다. 이 절은 comment 게시 전 확정 기록이며 실제 게시 permalink와 종료 상태는 원 PR/이슈에 남긴다.
- 이번 작업 전용 local·remote branch 정리는 사용자 승인 범위다. 기여자 `planet6897/rhwp`의 source branch, 기본 작업공간과 공유 `target/pr-review`는 보존한다.
- 아래 Open/CI 대기/미게시 표기는 작성 당시 이력이다. 현재 통합 결과는 이 절이 우선하며 최종 판정의 기술적 범위와 잔여는 변경하지 않는다.

## 최신 upstream/devel 동기화 및 해시 대조 (2026-09-09)

- 기준 base: `d8e4ab727b70b6abfcf11766134e09a9a9bfc982`. `review/planet6897-6949-6952-20260909`를 이 base 위로 리베이스했다. 오늘할일을 포함한 모든 파일이 충돌 없이 적용됐으며 수동 충돌 해소는 없었다.
- 현재 보정 커밋: `fe92170a0cac3e21aca3281b7e4a054052dadf5b` (리베이스 전 `80ceb5ce1149106e389f7325a48545bb20a14031`). 문서 갱신 직전 head는 `bffaab24f16fd7341d15661c4905931f5ca0ec95`다. 후속 문서 commit은 이 소스 후보를 변경하지 않는다.
- 소스·테스트 7개 파일, 최종 PDF 2개와 대표 PNG 5개를 리베이스 전 head `8be25bae1`와 Git blob 대조했고 모두 동일했다. SHA-256도 다시 산출했다. 상세 해시와 커밋 대응표는 [통합 기록](pr_6949_6952_review_impl.md#리베이스-해시-대응표)을 따른다.
- 기존 전체 회귀·Clippy·Native Skia·WASM·시각 증적은 리베이스 전 검증 결과다. 작업지시자의 "다른 conflict가 없으면 추가 테스트 없이 PR" 지시에 따라 리베이스 후 build/test/lint/시각 출력은 재실행하지 않았다. 기존 바이너리 해시는 보존된 검증 실행 파일의 식별자이며 최신 base 재빌드 증거가 아니다.
- 최신 base에는 #6773의 표 삭제 및 관련 model/WASM 변경이 포함됐다. 이번 PR 파일의 바이트 동일성과 무충돌 리베이스가 저장소 전체의 실행 호환성을 보장하지는 않으므로, 최신 통합 head의 GitHub CI와 작업지시자 승인 게이트를 유지한다.
- 현재 판정은 `메인터너 보정 후 수용 가능`을 유지한다. 원격 PR 번호·head·CI의 최신 진행 상태는 [오늘할일](../../orders/20260909.md)의 통합 PR 항목과 채번 후 생성하는 self-review 기록을 따른다. 아래 검증 당시의 미게시·미생성 문구는 당시 작업 범위의 이력이다.

## 최신 결론

- [#6949 review](pr_6949_review.md): **메인터너 보정 후 수용 가능**. 원 head의 P1/P2와 보정 중 드러난 WMF 회귀를 해소했다. 원 문서 15쪽 직접 visual sweep 및 2/3/7쪽 대표 PNG 판독을 완료했다. 회색조·누락 아이콘·7쪽 영문 배너 차이는 잔여 현상으로 명시하며, #6865 전체 해결이나 완전한 시각 일치를 선언하지 않는다.
- [#6952 review](pr_6952_review.md): **메인터너 보정 후 수용 가능**. #6940의 빈 접미 보존 병합과 테스트 fixture 오류 수정을 완료했다. 실물 XML 및 두 문서 68쪽 PDF 비교를 완료했고, 최종 왕복 HWPX의 바이트 동일성을 확인해 후보 PDF를 재사용했다.
- 두 판정의 대상은 원 PR head가 아니라 메인터너 통합 보정 커밋 `fe92170a0cac3e21aca3281b7e4a054052dadf5b`이다. 집중 회귀 32/32, 전체 회귀 9,377/9,377(46 skipped), Native Skia·WASM 패키지와 세 Clippy 단계가 통과했고, 소스·문서·최종 증적을 커밋했다.
- 작업지시자 시각 승인과 최종 통합 CI는 별도 후속 게이트다. 원격 통합 PR 생성/push/merge/원 PR close/공개 review comment는 수행하지 않았다. 원 PR 두 건의 reviewer 지정은 완료했다. 아래 최초 검토·실패 후보 기록은 이력이며 현재 최종 판정을 대체하지 않는다.

## 기준과 적용 순서

2026-09-09 시작 작업 트리는 clean한 devel이었다. upstream/devel을 `92f6242af96f51ede912588fa6fe35f5447709bc`까지 fast-forward한 다음 `review/planet6897-6949-6952-20260909`를 만들었다. 열린 planet6897 PR은 두 건이었고 모두 원 head CI가 성공했다. 이미 수용한 #6938/#6940을 다시 적용하지 않았다.

| PR | 원 commit | 로컬 적용 commit | 결과 |
| --- | --- | --- | --- |
| #6949 | `8fbfd865b1190fdf95b5a98d80467a53c944ce59` | `32554ff8bf193e4da64e7855854dcb3b80bffa64` | 출처 보존 체리픽, 충돌 없음 |
| #6952 | `4529c2a0c0104a2783b44a5b798a7badce628042` | `d01f7989b25e5ecff7a6a42a04b1d901ea44c1d6` | 출처 보존 체리픽, serializer 충돌 2개 구간 해소 |

#6952의 ON_* 토큰은 최신 devel에 이미 있어서 유지했다. suffix fallback은 원본의 명시적 빈 값을 보존하는 `deco_chars_from_source`와 새 USER_CHAR 조건을 OR로 결합했다. 미설정 숫자 형식의 `)`와 실제 지정 문자를 유지한다. 원 PR 쪽 파일로 덮어써 #6940 계약을 잃는 해소는 하지 않았다.

## 검증과 산출물 정책

- 원 PR별 실제 CI run과 실행/skip은 개별 review에 기록했다. 이는 원 head의 성공이며 새 누적 candidate의 검증이 아니다.
- 이번 단계는 소스/시험/본문/코멘트 정적 검토 및 기존 contributor PNG 열람이다. 로컬 테스트, Clippy, build, 새로운 PDF/SVG 출력, visual sweep을 실행하지 않았다.
- 기존 #6949 report PNG 4개는 contributor commit의 원래 증적이다. 이를 maintainer 직접 검증 PNG로 분류하거나 중복 복제하지 않았다.
- 추가 PDF, PNG, 로그, generated suite/manifest를 만들거나 stage하지 않았다. 다음 검증에서도 기준 PDF가 적합하면 재사용하고 로그/중간 산출물은 output에만 둔다.
- #6949 소스 결함의 회귀 재현/보정과 #6952 충돌 해소본 검증은 다음 단계이며 완료 사실로 쓰지 않는다.

## 후속 단계와 승인 경계

1. #6949의 불완전/다른 영역/유색 1bpp 입력에서 삭제하지 않는 계약을 고정하고 제한된 메인터너 보정을 설계한다. 이번 검토에서는 발견한 코드 결함을 임의로 고치지 않았다.
2. 검증 승인 뒤 고정 target/pr-review에서 순차로 build/회귀/Clippy 묶음을 수행한다. #6952는 #6940/#2742와 인라인 USER_CHAR/미주/속성 파싱을 포함한다.
3. #6949는 원 문서 3/7쪽과 issue6469 최소 fixture, #6952는 기존 두 원본의 XML 및 한컴 기준 PDF를 직접 대조한다. 원 PR 증적만으로 승인하지 않는다.
4. 실제 수치/대표 PNG/미해결 범위 및 개별 contributor comment 계획을 보완한 뒤 통합 PR 생성 승인을 받는다.
5. 최종 통합 head CI와 merge 승인 뒤에만 post_merge.md의 archive/devel sync/issue/원 PR comment/cleanup을 수행한다. 이 문서는 원격 조치 승인을 대신하지 않는다.

## 1차 메인터너 검증 실행 (2026-09-09)

**당시 결과(이력, 현재 최종 판정 아님): 머지 보류. 집중 회귀 실패로 중단했으며 전체 회귀·Native Skia·WASM 패키지 검증은 미실행이다.**
앞선 정적 검토와 달리 실제 보정·빌드·Clippy·시각 검증을 수행했다. 당시 후보는
`review/planet6897-6949-6952-20260909`의 `399491937` 위 미커밋 변경이다.
기준 `upstream/devel`은 `92f6242af96f51ede912588fa6fe35f5447709bc`이며 이번 단계에서 재동기화하지 않았다.
GitHub comment, commit, push, PR 생성, merge를 수행하지 않았다.

### 실제 실행 결과

| 게이트 | 결과 |
| --- | --- |
| generated suite prepare | 통과 |
| cargo fmt / fmt check | 통과. 신규 suite 연결 전 포맷 누락은 prepare 후 fmt로 해소 |
| workspace build --locked | 통과, 107초 |
| native Clippy -D warnings | 통과, 75초 |
| wasm32 lib Clippy -D warnings | 통과, 56초 |
| workspace/all-targets Clippy -D warnings | 통과, 76초 |
| suite manifest check | fmt 후 harness drift 발생; 재prepare 후 check와 fmt-check 통과 |
| source unit tier check | 통과 |
| 집중 nextest | 28개 실행: 24 passed, 4 failed, 9391 filtered/skipped. 빌드 포함 381초, 테스트 본 실행 0.112초 |
| 전체 nextest | 집중 회귀 실패로 미실행 |
| Native Skia lib / native CLI / issue1144 | 미실행 |
| WASM 패키지 빌드 | 미실행. WASM Clippy 통과와 구분 |
| WMF 원본 visual sweep | 15페이지 비교 완료, 자동 구조 후보 0개, 시각 residual 존재 |
| 각주 원본 2종 XML 왕복 | 대상 구조·속성 동일 |
| 각주 Hancom PDF 왕복 비교 | 총 68페이지, delta >32 차이 0개; A p2에 임계값 이하 3픽셀, B는 전 페이지 완전 동일 |

실패 4개는 다음과 같다. 기존 테스트를 제거하거나 assertion을 완화하지 않았다.

1. `issue_6865_clip_change_keeps_middle_draw`: `rop_pat0` 보존 assertion 실패.
2. `issue_6469_wmf_brush_only_raster_ops::xor_pair_is_cancelled_so_gray_rect_is_emitted_once`: 도해 0 회색 사각형 기대 1, 실제 2.
3. `issue_6872_parser_and_writer_preserve_footnote_and_endnote_user_char`: literal fixture의 미등록 `charPrIDRef: [0]`.
4. `issue_6872_inline_user_char_parser_and_writer_roundtrip`: 같은 fixture 참조 오류.

보정 자체와 추가 테스트 입력에 문제가 드러났으므로 원인 수정은 사용자 확인 후 재개한다.
빌드나 실제 파일 PDF 일치만으로 이 실패를 무시하지 않는다.

### 후보 식별 SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `src/wmf/converter/svg/ternary_raster_operator.rs` | `c67b786c8150e6037ab58a5995011321f68b59ce160f7d053bd07188ad61d942` |
| `src/wmf/converter/svg/mod.rs` | `2af2d715dc37d1c9fe35ea5232b4b7071141de2c32c97ef57a48e5397182f53e` |
| `src/serializer/hwpx/section.rs` | `ab055a5c8745f89d4cf6ec25cbd1dc6662bbebd9e8e4420ef12d660e10f77b74` |
| `tests/cases/issue_6865_wmf_monochrome_mask_rop.rs` | `3713f1459d347a9dc6778c47e60a7d972ebffefd79b296e9fa7248d32e76b1e4` |
| `tests/cases/issue_6872_hwpx_footnote_autonum_roundtrip.rs` | `752ef3d3e98593b3f549c6871eb9a0b88f1a6f17b1eb0cfa8be1a62f6ab590b0` |
| `target/pr-review/debug/rhwp` | `28047859c243c6ea7ce5f0baf59aa36ee2b8cdf5755c3e230b94e728c0b1cd2a` |

### 再現 명령 및 산출물 경계

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check
cargo fmt --all -- --check
node scripts/rust-unit-test-tiers.mjs --check
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast \
  -E 'test(/issue_6865|issue_6872|issue_6469|issue6469|issue_2742|issue2742|footnote_endnote_numbering/)' \
  --status-level fail --final-status-level fail
```

Cargo 작업은 순차 실행했다. 종료 후 Cargo/Rust 프로세스가 없을 때 검증 전 백업으로
`tests/generated`와 `tests/suites/manifest.json`을 복원했다. 파생 파일을 source 변경에 포함하지 않는다.

시각 재현: `scripts/visual_sweep.py --key wmf-original --hwp <원본> --pdf pdf/156627451-quantum-science-press-note-2020.pdf
--pages 1-15 --out output/pr_6949_6952_maintainer_20260909/wmf-sweep --rhwp-bin target/pr-review/debug/rhwp --dpi 96 --svg-rasterizer rsvg`.
각주 원본은 같은 바이너리의 `export-hwpx <원본> <왕복.hwpx>`로 출력하고 engine 2020 MCP로 **왕복본만** PDF로 만들었다.
PDF 비교 스크립트는 `output/pr_6949_6952_maintainer_20260909/compare_note_pdfs.py`에 있다.

최종 대표 PNG 4개를 `mydocs/pr/assets/pr_6949_6952_maintainer_20260909/`에 보관했다.
후보 왕복 PDF 2개는 `pdf/pr6952-note-{a,b}-roundtrip-2020.pdf`에 보관하며 기존 기준 PDF 3개는 재사용했다.
`.log`, SVG, JSON, raw raster, contact sheet, 파생 test inventory는 커밋 대상이 아니다.
개별 review 문서에 실제 직접 확인 범위와 merge 후 comment 계획을 함께 갱신했다.

## 최종 보정 후보 검증 (2026-09-09)

**최신 결과: 메인터너 오류 수정 및 요청한 로컬 자동 검증 완료.**
기존 4개 회귀 실패와 추가 보정 과정에서 발견한 WMF 체크무늬 재출현을 해소했다.
아래는 `review/planet6897-6949-6952-20260909`에서 검증 후 보정 커밋
`fe92170a0cac3e21aca3281b7e4a054052dadf5b`으로 고정한 후보 결과다.
검증은 커밋 전 같은 소스 바이트에 대해 수행했으며 아래 source/binary SHA-256을 함께 보존한다.
소스·최종 증적 commit은 완료했다. 작업지시자 시각 승인·최종 통합 CI 및 GitHub 원격 작업은 별도 대기다.

### 실행 이력 구분

| 실행 | 결과 및 의미 |
| --- | --- |
| 1차 집중 회귀 | 28개 중 24 통과·4 실패. 위 이력에 보존 |
| 수정 전 전체 회귀 | 9,373개 중 9,369 통과·같은 4개 실패, 46 skipped. 실제 끝까지 실행 |
| 클립/XOR/fixture 보정 후 집중 회귀 | 28/28 통과 |
| 같은 후보의 전체 회귀 | 시각 퇴행 발견 후 추가 보정 승인에 따라 SIGINT로 중단. 4,544 통과, 실행 중 12개 SIGINT, 4,817개 미실행. exit 100은 완료/통과가 아니며 12개를 제품 assertion 회귀로 계산하지 않음 |
| 최종 하프톤 경계 보정 | 집중 32/32 및 전체 9,377/9,377 통과. 아래 모든 게이트 exit 0 |

이력 로그는 각각 `output/pr_6949_6952_maintainer_20260909/`,
`output/pr_6949_6952_full_20260909_215803/`, `output/pr_6949_6952_corrected_20260909/`에 남긴다.
최종 결과만 `output/pr_6949_6952_halftone_20260909/`에 기록했다.

### 최종 게이트

| 게이트 | 실제 결과 |
| --- | --- |
| prepare / fmt / 재prepare / fmt-check | 모두 exit 0 |
| native Clippy | exit 0, 45초 |
| wasm32 lib Clippy | exit 0, 43초 |
| workspace build | exit 0, 98초 |
| workspace/all-targets Clippy | exit 0, 77초 |
| suite manifest / source unit tiers | 모두 exit 0 |
| 집중 nextest | 32 passed, 9,391 filtered/skipped. 실행 0.124초, 빌드 포함 366초 |
| 전체 nextest | 9,377 passed, 46 skipped, 실패 0. 실행 421.769초, 단계 전체 431초 |
| Native Skia lib | rhwp 3,930 + contracts 15 + ooxml_chart 165 + password_crypto 2 = 4,112 passed, 13 ignored, 실패 0. 단계 209초 |
| native CLI | 1/1 passed, 단계 157초 |
| native issue1144 | 1/1 passed, 단계 4초 |
| WASM web package | release 컴파일 및 wasm-opt 완료, exit 0, 651초 |
| 파생 테스트 파일 복원 | Cargo/Rust 종료 후 백업으로 복원, exit 0 |

전체 회귀 run ID: `e92b71ce-9f5e-44b6-8b18-3ab33acb6840`.
집중 회귀 run ID: `57df4060-c11e-4f97-a338-cdae6932696c`.
46 skipped와 native 13 ignored는 통과 수에 포함하지 않는다. 단계 간 중복 테스트도 합산하지 않는다.

실제 최종 명령은 `output/pr_6949_6952_halftone_20260909/validate.sh`에 있고,
원시 로그는 그 아래 `validation/`에만 보관한다. Cargo는 고정 `target/pr-review`에서 순차 실행했으며,
`CARGO_INCREMENTAL=0`을 추가하거나 공유 target을 삭제하지 않았다.

```bash
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast --status-level fail --final-status-level fail
cargo test --locked --profile release-test --target-dir target/pr-review \
  --features native-skia --lib -- --test-threads 12
node scripts/run-rust-test.mjs cli_exit_codes_native -- \
  --cargo-profile release-test --target-dir target/pr-review --features native-skia
node scripts/run-rust-test.mjs issue_1144_native -- \
  --cargo-profile release-test --target-dir target/pr-review --features native-skia
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web \
  --out-dir /home/tsjang/rhwp/output/pr_6949_6952_halftone_20260909/wasm
```

### 최종 후보 SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `src/wmf/converter/svg/ternary_raster_operator.rs` | `9ca704cf7ec19c9995a622177df06181f2f383d30dfed6a63167fecc7aee7bcc` |
| `src/wmf/converter/svg/mod.rs` | `f6f9131e91352dc78ee765e70771500d3e3d0c09e755793ff56f47c01c4f6167` |
| `src/serializer/hwpx/section.rs` | `ab055a5c8745f89d4cf6ec25cbd1dc6662bbebd9e8e4420ef12d660e10f77b74` |
| `tests/cases/issue_6865_wmf_monochrome_mask_rop.rs` | `7291c9432b2c3b5c6bce50e0f6cd5647ef410712fbba08e2b0163ca509e1f764` |
| `tests/cases/issue_6872_hwpx_footnote_autonum_roundtrip.rs` | `a94ab64569af51f11152bb62682e40693bf95dfbba101cfa6c5b2b52a176b9aa` |
| `target/pr-review/debug/rhwp` | `20577fff38aa1aa2a75a68c5bdc4a2c23192bdaf8e80586041912be372470740` |

### 시각 판정과 증적 보관

- WMF 원본 15페이지를 최종 바이너리로 재비교했다. 자동 구조 후보 0개, 평균 pixel match 90.16332%,
  평균 visual_accuracy_proxy_percent 26.81390%다. p2/p3의 체크무늬 재출현을 해소했고 p7 결과도 유지된다.
- Codex가 p2/p3/p7의 실제 review PNG를 열어 확인했다. 회색 농도·아이콘·배너 영문 차이는 잔여 한계다.
  국소 mask 보정을 전체 문서 fidelity 통과 또는 작업지시자 최종 시각 승인으로 확대하지 않는다.
- 축소 WMF fixture도 15페이지 SVG 출력에 성공했다. 원본 p7의 독립 검증을 대체하지 않는다.
- 실제 각주 2종을 최종 바이너리로 왕복 출력한 HWPX가 기존 검증본과 바이트 단위로 동일하다.
  기존 XML 검사, 한컴 PDF 2개와 68페이지 비교를 그대로 재사용했으며 PDF를 다시 출력하지 않았다.
- 대표 asset은 `mydocs/pr/assets/pr_6949_6952_maintainer_20260909/`의 최종 WMF p002/p003/p007 3개와
  각주 A p010/B p013 2개, 총 5개다. WMF p003/p007은 최종 후보 이미지로 갱신했다.
- 원본 기준 PDF 3개는 기존 경로에 그대로 두고, 후보 왕복 PDF 2개도 1차 기록의 `pdf/` 경로에 보관한다.
- 최종 comment 계획은 [#6949 review](pr_6949_review.md#최종-보정-후보-검증-2026-09-09)와
  [#6952 review](pr_6952_review.md#최종-보정-후보-검증-2026-09-09)에 갱신했다.
- `.log`, 원시 raster, 중간 SVG/JSON, MCP 응답, WASM 패키지, generated suite/inventory와 실패 후보 이미지는
  커밋에서 제외했다. 소스·문서·최종 증적은 `fe92170a0cac3e21aca3281b7e4a054052dadf5b`에 커밋했으며, GitHub 게시·push·PR 생성·merge는 수행하지 않았다.

## 리베이스 해시 대응표

| 단계 | 리베이스 전 | 리베이스 후 |
| --- | --- | --- |
| 기준 devel | `92f6242af96f51ede912588fa6fe35f5447709bc` | `d8e4ab727b70b6abfcf11766134e09a9a9bfc982` |
| #6949 체리픽 | `32554ff8bf193e4da64e7855854dcb3b80bffa64` | `f39e79dd8098f105cdb1e22fcb67a7e18b2e74b0` |
| #6952 체리픽 | `d01f7989b25e5ecff7a6a42a04b1d901ea44c1d6` | `a238c11141b546e861c18e3e12354bf7476406e8` |
| 최초 review | `399491937` | `0451d49f6a2a289d1bff25cd5124f747c8a88c02` |
| 메인터너 소스·증적 | `80ceb5ce1149106e389f7325a48545bb20a14031` | `fe92170a0cac3e21aca3281b7e4a054052dadf5b` |
| 최종 판정 문서 | `8be25bae1` | `bffaab24f16fd7341d15661c4905931f5ca0ec95` |

SHA 변경은 base와 부모 commit 변경에 따른 것이다. 아래 파일 바이트는 리베이스 전후 동일하며 해시는 `sha256sum`으로 다시 확인했다. 바이너리는 재빌드하지 않았다.

| 파일 | SHA-256 |
| --- | --- |
| `src/wmf/converter/svg/ternary_raster_operator.rs` | `9ca704cf7ec19c9995a622177df06181f2f383d30dfed6a63167fecc7aee7bcc` |
| `src/wmf/converter/svg/mod.rs` | `f6f9131e91352dc78ee765e70771500d3e3d0c09e755793ff56f47c01c4f6167` |
| `src/model/footnote.rs` | `519bed04d1c65906425fee36210d32f87ae20f5ebfaa6d2e5ed960df5c386235` |
| `src/parser/hwpx/section.rs` | `68856970f05741d37ec9d66629857de40a3e5d4d51b58f8d9c8b1875fcd71951` |
| `src/serializer/hwpx/section.rs` | `ab055a5c8745f89d4cf6ec25cbd1dc6662bbebd9e8e4420ef12d660e10f77b74` |
| `tests/cases/issue_6865_wmf_monochrome_mask_rop.rs` | `7291c9432b2c3b5c6bce50e0f6cd5647ef410712fbba08e2b0163ca509e1f764` |
| `tests/cases/issue_6872_hwpx_footnote_autonum_roundtrip.rs` | `a94ab64569af51f11152bb62682e40693bf95dfbba101cfa6c5b2b52a176b9aa` |
| `target/pr-review/debug/rhwp (이전 검증 바이너리)` | `20577fff38aa1aa2a75a68c5bdc4a2c23192bdaf8e80586041912be372470740` |
| `pdf/pr6952-note-a-roundtrip-2020.pdf` | `ad3a16cfb1f2a3bdf157d3e7f51573e8c42463513d5c76c1c67322713798cf63` |
| `pdf/pr6952-note-b-roundtrip-2020.pdf` | `d0c974353dd978fbe3a47c25f6f94c22a8e72123288ec24046342f0aa84b83f0` |
| `mydocs/pr/assets/pr_6949_6952_maintainer_20260909/pr6949-original-p002-review.png` | `a2c4f5da8b26ac6d2a7c32ef9850d41c932895bc8956ff664aafa928ea594bc8` |
| `mydocs/pr/assets/pr_6949_6952_maintainer_20260909/pr6949-original-p003-review.png` | `b093599baf4e38524929110906fed4d666a88a31700d2eacca39ae454237114e` |
| `mydocs/pr/assets/pr_6949_6952_maintainer_20260909/pr6949-original-p007-review.png` | `97434772f9a8062535ec597349268fa399d983a44e9cb57d398a4f9f7ac46099` |
| `mydocs/pr/assets/pr_6949_6952_maintainer_20260909/pr6952-note-a-p010-review.png` | `00bc1bd95915c0f61ead6cb45f59d968be42fed4b98e07644ea99df8c1c5ebbc` |
| `mydocs/pr/assets/pr_6949_6952_maintainer_20260909/pr6952-note-b-p013-review.png` | `f425a5cdcd6c912c61135772515b6120ef7477b5cfded8d6cf50bdf38009e5d5` |

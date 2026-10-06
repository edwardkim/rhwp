# #5511 Stage 2 기능군 배치 Q6 — 변환·생성 adapter 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 구현 시작 기준: `upstream/devel` `980bf59e406e9cd31d4b3ac9ffa21f356487b4ce`
- 최종 통합 기준: `upstream/devel` `b914bdf4bf1a8f922f03ea6b141f0d9c2b10a98f`
- 수행일: 2026-08-20
- 상태: 완료 — Q7 진입 승인 대기

## 1. 결과

`convert`, `extract-pages`, `export-hwpx`, `export-hml`, `export-doclang`,
`build-from-ingest`, `scaffold` 일곱 file-producing adapter를 `src/main.rs`에서 책임별 모듈로
분리했다. parser·serializer·renderer 알고리즘과 공개 API는 바꾸지 않았다.

| 모듈 | 책임 | 최종 줄 수 |
|---|---|---:|
| `cli/commands/conversion.rs` | HWP/HWPX/HML 변환·쪽 추출·변환 검증 | 718 |
| `cli/commands/generation.rs` | ingest 기반 문서 생성·scaffold | 295 |
| `cli/outputs/doclang.rs` | DocLang XML·asset·loss 출력 | 169 |

세 모듈은 모두 1,200줄 상한 이하다. `src/main.rs`는 Q6 시작의 31,095줄에서 29,944줄로
1,151줄 줄었고 최상위 함수는 253개에서 241개로 줄었다. Q6 대상 함수는 이동 전후 모두
CC 25 이하이며 새 모듈에서 인지 복잡도 경고가 발생하지 않았다.

`src/main.rs`의 `rhwp::wasm_api::HwpDocument` 직접 참조는 27개에서 23개가 됐다. 줄어든 4개는
`conversion.rs`로 물리 이동한 것이며 service 경계로 전환한 것은 아니다. 이 의존 제거는
계획대로 Stage 3의 DIP 전환 입력으로 남긴다.

single convert와 Q4 batch가 함께 쓰는 `ConversionVerifyOptions`, `verification_exit_code`,
`paths_refer_to_same_file`, 문서 load·password·provenance seam은 root에 보존했다. 공유 구현을
복제하거나 Q6 모듈이 다른 기능군의 소유권을 가져가지 않았다.

## 2. 보호 계약

이동 전 inventory에서 선정한 17개 계약 모듈 123/123이 통과했다. 이 모집단이 변환 IR·쪽수
검증, exit 3/4 우선순위, JSON provenance, stdout/stderr 분리, 원본·hard link 보호, atomic
write, HML·DocLang 의미, ingest fail-closed와 scaffold round-trip을 이미 직접 보호했다.
따라서 현재 동작을 새 테스트로 중복 고정하지 않고 신규 characterization 커밋을 생략했다.

두 구현 커밋 뒤 같은 123개가 다시 전부 통과했다. 완료 직전 원격 결합 뒤에는 새
`agent_q_pack_contract` 4개까지 더해 127/127을 실행했고 모두 통과했다. 이동으로 인한 명령,
옵션, help, JSON, exit code 또는 파일 부작용의 관찰 가능한 차이는 발견되지 않았다.

## 3. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `9ffc9ea77` | Q6 시작 전 최신 원격 devel 정상 merge |
| `1222fd5ca` | Q6 대상·계약·공유 seam inventory |
| `209af6c89` | generation command와 DocLang output 이동 |
| `488dc0acf` | format conversion·extract-pages command 이동 |
| `8bdd0b2ea` | Q6 완료 직전 최신 원격 devel 정상 merge |

## 4. 최종 검증

| 검증 | 결과 |
|---|---|
| 이동 전·후 Q6 focused | 각각 123/123 통과 |
| 최신 devel 결합 후 Q6 + q-pack focused | 127/127 통과 |
| 구현 HEAD release-test 전체 nextest | 7,995/7,995 통과, 3 slow, 38 skipped, 175.595초 |
| 최신 결합 HEAD release-test 전체 nextest | 7,999/7,999 통과, 3 slow, 38 skipped, 183.020초 |
| 대상 모듈 CC 25 상한 | 경고 없음 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --all-targets` | 최신 결합 HEAD 통과 |
| `cargo clippy --all-targets -- -D warnings` | 최신 결합 HEAD 통과 |
| `cargo test --doc` | 8/8 통과, 3 ignored |
| integration manifest·unit-tier 정책 자체 계약 | 34/34 통과 |
| 최신 base manifest check | 802 sources / 3,950 static test attrs / 41/48 integration targets, 통과 |
| unit-tier base check | 4,225 tests / 298 modules, 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 67/67 통과 |
| Markdown link check | 기존 capability 등록부 무결성 오류 16건, Q6 신규 오류 없음 |

`node scripts/rust-test-suite-manifest.mjs --prepare`는 최신 정책대로 root `Cargo.toml`이나 추적
파일을 바꾸지 않았다. 파생 suite와 Cargo target 산출물을 PR 변경에 섞지 않았고 최종 작업
트리는 문서 작성 전 깨끗했다. 로컬 nextest 0.9.137이 저장소 권고 0.9.140보다 낮다는 경고를
냈지만 전체 모집단은 정상 실행되어 전건 통과했다.

Q6는 move-only CLI adapter 변경이므로 renderer·layout·WASM·native-skia·시각 검증 발생 조건에
해당하지 않는다. 변환 명령이 기존 WASM 문서 wrapper를 호출하는 사실은 그대로 보존했지만
WASM 구현이나 빌드 경계를 수정하지 않았다.

## 5. 최신 devel과 열린 PR

Q6 시작 전에 `upstream/devel` `980bf59e4`를 정상 merge했다. 완료 검증 중 원격은 다시
`b914bdf4b`로 2커밋 전진했다. 새 변경은 PR #5672의 별도 `src/bin/rhwp-q-pack/`과 계약·검토
문서였고 Q6 파일과 겹치지 않았으며 merge-tree도 충돌 없이 생성됐다. 이를 rebase가 아닌 정상
merge commit으로 흡수했다.

원격 변경이 약 75만 줄의 생성형 q-pack 소스를 추가했기 때문에 경로 비중첩 판정만으로 끝내지
않고 결합 HEAD에서 focused, all-targets check·clippy, 전체 release-test를 다시 실행했다. 최종
fetch 시 `origin/devel`과 `upstream/devel`은 `b914bdf4b`로 같았다.

최종 재조회 시 열린 devel 대상 PR은 #5647, #5689, #5691 세 건이다. #5647은
`tests/issue_4100_chart_data_edit.rs`와 #5447 문서, #5689는 별도 `src/bin/rhwp-q-more/`와 그
계약, #5691은 Studio source·test를 변경한다. 셋 모두 Q6 source·test·module 경계와 겹치지
않는다. 이 판정은 시점 증거이므로 향후 통합·push 직전에 exact base SHA, PR head와
merge-tree를 다시 확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 6. 다음 승인 단위

다음 기능군은 Q7 `internal round-trip·IR diff/sweep·verify`다. 진단·검증 exit code와 diff
계약을 보존하면서 internal conversion/verification adapter를 분리한다. Q7은 메인테이너의 Q6
완료 승인과 별도 진입 승인 전 시작하지 않는다.

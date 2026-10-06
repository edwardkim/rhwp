# #5511 Stage 2 기능군 배치 M1 — metadata projection 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 구현·최종 통합 기준: `upstream/devel` `b914bdf4bf1a8f922f03ea6b141f0d9c2b10a98f`
- 최종 코드 HEAD: `0ea5de1dc1378471f27bbd338a679e8f94cace9e`
- 수행일: 2026-08-20
- 상태: 완료 — Wave M 종료, P1 진입 승인 대기

## 1. 결과

MCP tool schema·capabilities payload·사람용 help projection을 `src/main.rs`에서
`src/cli/metadata/` 아래로 분리했다. `src/cli/catalog.rs`는 명령 정본으로 그대로 유지했고,
새 metadata 모듈은 그 정본을 읽어 외부 표면을 투영하는 adapter만 소유한다. root wrapper는
남기지 않았으며 `main.rs`와 `mcp_serve.rs` 소비자는 새 소유 경로를 직접 호출한다.

| 기능군 | 파일 | 최종 줄 수 범위 |
|---|---:|---:|
| MCP 조립·공통 계약 | 1 | 344 |
| MCP read·exchange·edit·protocol·advanced 정의 | 7 | 324~811 |
| capabilities 조립·검색·봉투 | 1 | 759 |
| capabilities command projection | 2 | 476~795 |
| help 순서 조립 | 1 | 11 |
| public·edit·protocol help | 3 | 63~626 |

15개 metadata 파일은 모두 1,200줄 상한 이하다. `src/main.rs`는 M1 시작의 28,295줄에서
20,741줄로 7,554줄 줄었고 최상위 함수는 228개에서 205개로 줄었다. metadata 모듈에 대한
`clippy::cognitive_complexity` 25 초과 경고는 0건이다.

## 2. 출력·순서 불변식

각 하위 파일은 기존 배열 순서대로 `extend`하고 parent가 기능군을 원래 순서로 조립한다.
정렬이나 map 재구성을 추가하지 않아 MCP tool order, capabilities command order,
did-you-mean 동률 해소와 help 출력 순서를 보존했다. MCP password-stdin·annotations·catalog
완전성 후처리도 최종 조립 뒤 한 번만 적용된다.

이동 전과 각 구현 절편 뒤 다음 debug binary stdout SHA-256이 모두 완전 일치했다.

| 명령 | SHA-256 |
|---|---|
| `rhwp --help` | `41e7ab065c67e23ffa74dc2ea444c7e842f286db02227f8057f07e9d872b0423` |
| `rhwp capabilities` | `a397674e46a2354b9a711f3870ad2585b532363519923b7b6b38022978f5383c` |
| `rhwp capabilities --mcp` | `0deb8c9c33577e5ac1a87d020f9dbd4c6b0012fa52f2424848b4c97363a935e0` |
| `rhwp capabilities --mcp --profile 개발통합` | `c5a2e78ad397ed1c0f53625dfcb0a5acce71b647db10a3ccbe0f94dad10b05fa` |
| `rhwp capabilities --search '표 병합' --json` | `4516487192a9c3246593c186c0b682a15aca8f4e02267c9fbc5e78b398ecee9a` |
| `rhwp capabilities --search 없는단어999 --json` | `8db3289f79ccb85879d400fe67c85c72e9b7f4722b0658b4d1fb3eed2fd3d01f` |

## 3. 계약과 정정

M1 직접 계약 12개 모듈 143/143을 이동 전 고정했다. MCP 절편 106개, capabilities 절편
87개, help 절편 54개 focused 계약을 각각 통과했고 최종 전체 nextest가 이 계약 전부를 다시
포함해 통과했다.

기존 `cli_catalog_contract`가 `main.rs`의 `/// [#3263]` 주석을 dispatch 종료 경계로 사용하던
위치 결합을 발견했다. `// [#5511] 최상위 dispatch 끝`이라는 명시적 characterization 경계를
두고 테스트가 이를 읽게 해 이후 metadata 이동과 무관하게 만들었다. 최종 all-target clippy는
MCP 원소 함수에서 공용 schema helper 앞으로 남은 소유권 주석도 발견했고, 중복 잔여 주석만
제거했다. 두 정정 모두 공개 출력과 실행 동작을 바꾸지 않는다.

## 4. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `c9655d0a9` | M1 범위·보호 계약·byte hash inventory |
| `193034df3` | MCP 공통 계약과 7개 순서 보존 tool 정의 모듈 이동 |
| `d15090193` | capabilities 조립과 2개 command projection 모듈 이동 |
| `627c5380b` | public·edit·protocol help projection 이동 |
| `0ea5de1dc` | all-target clippy가 찾은 MCP 문서 주석 소유권 정합화 |

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| M1 직접 focused 계약 | 이동 전 143/143, 절편별 106·87·54 전건 통과 |
| 최종 release-test 전체 nextest | 8,005/8,005 통과, 3 slow, 38 skipped, 158.541초 |
| 여섯 stdout byte hash | 이동 전 값과 전부 일치 |
| metadata 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --all-targets` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책·현재 상태 | 18/18, 803 sources / 3,956 attrs / 41/48 targets 통과 |
| unit-tier 정책·현재 상태 | 12/12, 4,225 tests / 299 modules 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 163/163 통과 |
| Markdown link check | 기존 capability 등록부 무결성 오류 16건, M1 신규 오류 없음 |

추가로 unit-tier를 `--base-ref upstream/devel`과 직접 비교하면 M1이 아닌 이전 Q7의 이동된
`ir_comparison.rs::tests`를 신규 module로 판정해 실패한다. 현재 매뉴얼의 정식 로컬 명령인
`rust-unit-test-tiers.mjs --check`는 저장된 inventory와 4,225/4,225로 일치한다. 이 차이는 M1
변경에 source-side test 추가가 없다는 사실과 분리해 기록한다.

`rust-test-suite-manifest --prepare`가 만든 harness는 검증 파생물이며 추적 변경에 포함하지
않았다. 로컬 nextest 0.9.137이 저장소 권고 0.9.140보다 낮다는 경고가 있었지만 전체 모집단은
정상 실행되어 전건 통과했다. M1은 move-only metadata adapter 변경이므로 renderer·layout·WASM·
native-skia·시각 검증 발생 조건에 해당하지 않는다.

## 6. 최신 devel과 열린 PR

최종 fetch에서 `origin/devel`과 `upstream/devel`은 모두 `b914bdf4b`이며 최종 코드 HEAD의
조상이다. 열린 devel 대상 PR은 #5647, #5689, #5691, #5693, #5695, #5707, #5709, #5710이다.
각 최신 head의 변경 경로를 다시 조회했으며 M1의 `src/main.rs`, `src/mcp_serve.rs`,
`src/cli/metadata/`, `src/cli/mod.rs`, `cli_catalog_contract` 또는 M1 보고서 경로와 겹치지 않는다.

이 판정은 시점 증거다. 향후 통합·push 직전에 exact base SHA, 열린 PR head와 merge 가능성을
다시 확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 7. 다음 승인 단위

M1 완료로 metadata projection 분리를 마쳤다. 다음 기능군은 P1
`replay·audit·lineage·anchor·gate·bundle·disclose·settle·harness`다. 명령별로 잘게 자르지 않고
capsule 계보, anchor/gate, disclosure/settlement, harness 책임으로 나누며 보안·파일 부작용과
exit 계약을 먼저 inventory한다. P1은 메인테이너의 M1 완료 승인과 별도 진입 승인 전 시작하지 않는다.

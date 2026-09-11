# PR #7024·#7026·#7027·#7031 통합 검증 보완 기록

## 판정: 승인

2026-09-11 로컬 통합 후보에서 승인된 검증을 완료했다. 원 PR의 코드 결함으로 남은 보류 사유는
없다. 소스 문자열 검사만으로 확인했던 snapshot 예산과 폰트 치환의 실행 경로를 메인터너 회귀
테스트로 보완했다. 제품 구현을 임의로 변경하거나 허용 오차·baseline을 완화하지 않았다.
통합 PR 생성·push·merge 및 원 PR/이슈 comment·close는 아직 수행하지 않았다.

## 후보와 원격 변경 대조

- 기준: `upstream/devel`의 `3e29b5610e0cb079976e5b23f9cb0d6caf946c9c`.
- 로컬 브랜치: `review/non-draft-7024-7031-20260911`.
- 체리픽 code head: `af0fe858c`. 그 위 작업 트리에 검증 테스트·절차 문서·리뷰 증거를 추가했다.
- draft #6984·#6966·#6965·#6670은 포함하지 않았다.
- 원 PR에는 검토 전 `jangster77`을 reviewer로 지정했다. 통합 PR owner 자동 지정은 하지 않았다.

| 원 PR | 최초 검토 source head | 로컬 체리픽 | 재조회 source head | 대조 |
| --- | --- | --- | --- | --- |
| #7024 | `733f6d56b3499439e850d4bf215a857941bfb59c` | `15b13f300`, `4a1be2162` | `4635c615fe34f1865f24711257229cbdfbaaaa67` | 두 커밋 모두 `git range-diff`에서 `=` |
| #7026 | `4e96610ed9d4b320a78e664ecef72ecb3e72cc21` | `fe61d71c5`, `c16aa15fe` | 동일 | 추가 커밋 없음 |
| #7027 | `1ca76c2f780da8b64cfd0054b180a5a427998495` | `1266ad650` | `baa804c1e23f29b8ff0939965e60fedd44fc5fac` | 한 커밋이 `git range-diff`에서 `=` |
| #7031 | `d706e7ecda2e5d3fb202a330e3ecf3f90e2fd51f` | `af0fe858c` | 동일 | 추가 커밋 없음 |

#7024의 `bf432b358`·`733f6d56b`는 각각 `49861a9ec`·`4635c615f`와,
#7027의 `1ca76c2f7`은 `baa804c1e`와 패치가 동일했다. SHA 변경을 새 기능 커밋으로 오인해
중복 체리픽하지 않았다. 원래 `-x` provenance는 유지한다. 현재 source head를 직접 체리픽했다고
기록하지 않으며, 통합 테스트 대상의 코드 내용이 동일함을 위 대조로 확인했다.

## 절차와 영구 회귀 테스트 보완

- [local validation 4.3.0.0](../../manual/pr_review/local_validation.md#4300-상수정책호출-경로-변경의-동작-기반-회귀-검증)에 상수·정책·호출 경로 변경의 실제 동작 검증 지침을 추가했다.
- [intake 체크리스트](../../manual/pr_review/intake_and_review.md)에서 해당 지침을 연결하고 테스트 완료 후 최종 판정을 작성하도록 했다.
- [회귀 테스트 진입점](../../../rhwp-studio/tests/review-runtime-contracts.test.ts)과 [실행 runner](../../../rhwp-studio/tests/support/review-runtime-contracts.runner.mjs)를 추가했다.
- 실제 `CommandHistory`·`SnapshotCommand`·`WasmBridge.snapshotCapacity()`를 호출한다. WASM 저장소 대역은 용량·FIFO 축출·문서 상태를 재현하며, 실제 WASM 검증과 구분한다.
- 용량 100·50 및 API 부재 fallback에서 예산 경계, 오래된 이력 해제, execute/undo/redo 복원, 실패 rollback, undo 후 새 편집의 redo 무효화를 확인했다.
- 실제로 설치된 Canvas `font` setter를 호출해 엔진 별칭 유지·중복 제거·기존 Studio 1순위 치환을 확인했다. Node Canvas 대역 결과를 실물 글리프 검증으로 부르지 않는다.
- 메모리 안에서 예산을 100으로 고정하거나 엔진 체인을 제거한 음성 대조는 실패해야 통과하도록 했다. 제품 파일을 과거 코드로 되돌리지 않았다.

## 실행 결과

macOS, Node `v24.15.0`, Cargo 검토 전용 `CARGO_TARGET_DIR=target/review-7024-7031-20260911`을 사용했다.
Rust/Cargo와 Studio 전체 테스트를 동시에 실행하지 않았다. 로그·중간 JSON/SVG·임시 스크립트는
커밋 대상에서 제외하고 최종 comment용 PNG만 보관한다.

| 검증 | 실제 결과 |
| --- | --- |
| 새 동작 기반 회귀 테스트 | 6개 통과, 0개 실패·skip; 음성 대조 2개 포함 |
| Studio 전체 `npm --prefix rhwp-studio test` | 1,659개 통과, 0개 실패, 2개 skip |
| `npx tsc --noEmit` | 통과 |
| suite prepare·manifest 계약 테스트·최종 check | 통과; 1,258 source, 48 integration target |
| `cargo fmt --all -- --check` | 통과 |
| `cargo nextest run --locked --cargo-profile release-test --tests --test-threads 8 --no-fail-fast` | 9,473개 통과, 0개 실패, 46개 skip; 실행 346.113초, 최초 컴파일 시간 별도 |
| 기본 `cargo clippy --locked -- -D warnings` | 통과 |
| WASM lib Clippy | `--target wasm32-unknown-unknown` 통과 |
| `cargo build --locked --workspace` | 통과 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 통과 |
| `scripts/wasm-pack-locked.sh --target web` | release 빌드·wasm-opt 완료, 3분 10초 |
| #7025 실제 WASM/Chrome E2E | 14개 assertion 통과; 다섯 배치 사례·구 중앙 정렬식과의 차이·hover 확인 |
| 추가 Chrome 오버레이 검증 | 한 쪽·두 쪽·3열·맞쪽에서 객체/회전 테두리·셀 highlight·phase marker, 16개 검사 통과 |
| 실제 WASM snapshot smoke | capacity 100 반환, 실제 텍스트 수정 전후 snapshot 복원 일치 |
| 실제 Canvas 폰트 대조 | 로컬 WOFF2 로드, 제품 setter 395.401825px = 동일 fallback 기준 395.401825px; 별칭 제거 대조 444.703125px |

WASM 출력은 `/tmp/rhwp-7024-7031-review-20260911/pkg`에 분리했다. 기존 `pkg`와 7700 서버는
사용하거나 덮어쓰지 않았다. 검증용 Vite는 자동 빈 포트를 사용하고 `/private/tmp` 실제 경로를
허용한 뒤 종료했다. Chrome 추가 검사에서 page error는 없었다.

## 초기 실패의 분류와 재검증

첫 Node 검증에서 깊은 undo 횟수 기대값을 정정했다. 예산이 가득 찬 상태의 첫 undo가 after
snapshot을 추가하면서 기존 정책대로 가장 오래된 undo 하나를 더 해제하기 때문이다.
예산 초과가 코어 FIFO 축출보다 먼저 감지되는 음성 대조도 올바른 실패로 분류했다.
최종 6개 및 Studio 전체 테스트가 통과했다.

임시 브라우저 스크립트의 모듈 해석, 사용 중인 포트, `/tmp` 실경로 허용 문제를 보정했다.
폰트 비교는 미등록 이름의 기존 Studio 우선 치환을 무시한 초기 합성 입력과 서로 다른 fallback
목록을 바로잡았다. 최종 대조에서는 미등록 폰트의 기존 1순위를 별도로 확인하고, 등록됐지만
없는 선두 face·로드된 엔진 별칭·동일한 전체 fallback을 사용했다. 실제 출력과 대조군의 차이를
검출했다. 이 실패들을 제품 결함 해결이나 사설 HWP 원본 재현 성공으로 과장하지 않는다.

## 시각 증거와 범위

- [표 hover 증거](../assets/pr_7024_7031_20260911/pr7026-table-guide-hover.png): 검토 후보 WASM의 합성 새 문서 3×3 표. 최종 hover 위치 캡처이며 다쪽 레이아웃 전체를 보여주는 사진은 아니다. 다쪽 위치는 별도 14개 assertion 및 16개 DOM 좌표 검사로 확인했다.
- [폰트 실행 대조](../assets/pr_7024_7031_20260911/pr7031-font-runtime.png): 같은 Chrome Canvas에 로컬 NotoSansKR-Bold와 검증용 별칭을 로드한 합성 입력. 사설 문서 3146683 재현 캡처나 전체 PDF 정합 증거가 아니다.
- 원 기여자의 [#7025 전후 보고서](../../report/grid-overlay-page-left-7025/README.md)와 [#6600 전후 보고서](../../report/studio-cjk-bracket-face-6600/README.md)의 PNG도 직접 열어 확인했다. 기여자 실측과 메인터너 실행을 혼합하지 않는다.

## GitHub 상태와 다음 gate

최신 source head를 GraphQL로 재조회했으며 네 PR의 `statusCheckRollup.state`는 모두 `SUCCESS`였다.
#7026·#7031의 별도 CodeQL 표시 `NEUTRAL`은 성공으로 바꿔 적지 않는다. 원 head의 실행된
CodeQL 분석 성공과 expected skip을 구분해 확인했다. 통합 후보의 원격 CI는 아직 실행하지 않았다.

통합 PR이 만들어지면 최종 head의 required check·mergeability를 별도로 확인해야 한다.
원 PR 본문의 `closingIssuesReferences`는 네 건 모두 비어 있었으므로 관련 이슈를 자동 종료됐다고
가정하지 않는다. merge 후 `post_merge.md`에 따라 이슈 범위·실제 종료 상태를 다시 확인한다.

## 후속 comment 계획

원 PR과 관련 이슈에 merge SHA, 실제 PR/devel CI, 해당 리뷰 판정을 한 번만 기록한다.
이미 기록이 있으면 새 comment를 중복 생성하지 않고 수정한다. 시각 PR #7026·#7031은 아래처럼
확정된 merge SHA의 이미지 URL을 Markdown 이미지 문법으로 삽입해 comment에서 바로 보이게 한다.
아래 `<merge-sha>`는 미확정 자리표시자이며 게시 전에 반드시 확정 SHA로 바꾼다.

```markdown
![표 hover 위치 검증](https://github.com/edwardkim/rhwp/blob/<merge-sha>/mydocs/pr/assets/pr_7024_7031_20260911/pr7026-table-guide-hover.png?raw=true)
![Canvas 폰트 체인 합성 대조](https://github.com/edwardkim/rhwp/blob/<merge-sha>/mydocs/pr/assets/pr_7024_7031_20260911/pr7031-font-runtime.png?raw=true)
```

#7024·#7027은 불필요한 시각 자료 대신 실제 실행 결과와 코드·회귀 테스트를 연결한다.
원 PR을 직접 merge한 것처럼 기록하지 않고 체리픽 통합 수용 사실을 명시한다.

## 통합 PR 제출 기록

- 기본 경로: `collaborator_external_pr.md` 9.1.1의 체리픽 통합.
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`, `visual_fixture_evidence.md`.
- 동작 검증과 지침 보완 commit: `fe926575b4c9da8d3ed3de3355057d4a70169f60`.
- 이 commit은 위에서 테스트한 작업 트리를 고정한 것으로, 검증 뒤 제품·테스트 코드를 추가 변경하지 않았다.
- 2026-09-11 사용자 `pr` 승인으로 source PR별 리뷰·최종 PNG 두 장·오늘할일을 같은 통합 PR에 제출한다.
- 제출 직전 fetch한 `upstream/devel`은 검증 기준 `3e29b5610e0cb079976e5b23f9cb0d6caf946c9c`와 같고 오늘할일도 동일했다. 기존 항목을 보존하고 이번 항목만 추가했다.
- 정식 판정 용어는 매뉴얼 1.1에 따라 `승인`으로 통일한다. 이는 검증 보완을 포함한 통합 후보의 승인으로, 아직 미실행인 통합 PR CI·merge·원 PR/이슈 후속 처리의 완료를 뜻하지 않는다.

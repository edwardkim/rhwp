# PR #7978 검토 — HF/FN Unicode scalar 입력·undo/redo

## 최종 판정

**승인 — 원 기여와 정식 회귀를 반영한 통합 head의 로컬 검증과 실제 code CI 완료.**
code CI head는 `fd3d97b0b9effa2d0d4a1f65f47e3475ba4ec5c8`이며 로컬 제품·테스트 producer와의 차이는 검토 문서뿐이다.
원 contributor head의 직접 merge 승인과 구분한다. 사용자 병합·후속 처리 승인은 받았고
문서 trailing head의 최신 CI·mergeability가 남은 병합 조건이다.

## 접수 정보

- 원 PR: https://github.com/edwardkim/rhwp/pull/7978 · 작성자 `donggyun112`(기존 기여자).
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/7977 · 원 PR의 Closes 범위.
- 원 head: `5558cbcfb8e0d2cb30422e12aa719288371164a5` · 누적 적용 commit: `e386f7db8173b59ea10450944611c428eb321ca5`.
- 기준 devel: `7d01ef610f378157b79d36ef23e8ed4a3c8ab051` · 메인터너 보정: `607f1f7b515553d2c1b618448d345b1d72e8e3e3`(아래 개별 범위 구분).
- 리뷰어: `jangster77` 선행 요청 후 API 재조회 확인. 원 PR base devel, non-draft,
  MERGEABLE/CLEAN 및 최신 원 head check 실패 없음은 조사 시점 상태다.
- 원 PR [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/39027342904/job/117383140929)는 참고이며 통합 후보 검증을 대체하지 않는다.
- [적용 기록](pr_7978_review_impl.md) · [누적 계획](../../plans/pr_review_donggyun112_20261017.md).

## 개별 검토와 실제 소비 경로

일반 입력의 `onInput → insertTextInHeaderFooter/Footnote → record command → HF/FN cursor`와
`InsertTextInHeaderFooterCommand/InsertTextInFootnoteCommand`의 constructor·execute/redo 문맥을
대조했다. 코어 문단 위치가 Unicode scalar 단위라는 기존 계약에 맞춰 여섯 곳이 `charCount`를
소비한다. undo의 삭제 길이와 HF/FN 소유 문단·대상을 유지한다.

제품 보정은 이 원 PR의 scalar 변경에 추가하지 않았다. 동일 통합에 포함된 #7985의 빈 누름틀
사전 거절 보정은 별도 커밋이며 본 PR의 기여로 계산하지 않는다. `MANIFEST.md` 충돌은 두 PR의
항목을 모두 유지했다. 단위 대역에서는 3모드 × 4문자열(뒤 B 보존·반복 왕복·문맥)을 확인했고,
실제 브라우저에서는 3모드 × 3문자(가/😀/𠮷)의 실제 input·Ctrl+Z·Ctrl+Shift+Z를 확인했다.
음성 대조는 입력 후 커서 두 곳만 UTF-16 길이로 되돌렸을 때 assertion으로 실패했다.

## 공통 로컬 검증

최종 제품·테스트 검증 head는 `2dee7730306dcc7f57615c87c25cb09af3c90f63`, 최신 기준은 `7d01ef610f378157b79d36ef23e8ed4a3c8ab051`다.
기존 producer `92e952295a161cb153f562d3e988e5da629f2f3e`에서 완료한 focused·lint·workspace 검사는 리베이스 후에도
`src/tests/crates/vendor/bindings/tools/scripts/Cargo.toml/Cargo.lock/build.rs`의 Git object가 같음을
확인했다. 전체 회귀는 사용자 리베이스 요청으로 컴파일 중 중단한 실행을 완료로 계산하지 않고,
리베이스 후 다시 실행했다. 새 기준 manifest·unit-tier와 frontend도 재확인했다.

| 검사 | 실제 결과 |
| --- | --- |
| 신규 Native 검색 회귀 | 수정 전 4건 assertion FAIL / 수정 후 4 PASS |
| Rust lint 묶음 | fmt, Native clippy, WASM clippy, workspace build·all-targets clippy 모두 exit 0 |
| 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast` | Summary [ 751.818s] 10820 tests run: 10820 passed (11 slow), 45 skipped |
| Rust manifest·unit-tier | 최신 base 고정, 모두 exit 0 |
| TypeScript | `npx --prefix rhwp-studio tsc --noEmit -p rhwp-studio/tsconfig.json`, exit 0 |
| 전체 Studio 단위 | 2,320건 중 2,318 PASS / 0 FAIL / 2 SKIP |
| E2E manifest | 파일 158 / 행 158, 누락 0 |
| production build | `npm --prefix rhwp-studio run build`, exit 0 |
| fresh WASM | 공용 target, jobs 8, codegen-units 16, wrapper `--target web --out-dir pkg --no-opt`, exit 0 |
| 실제 Chrome 검색·치환 | API 6조합의 원문 길이·one/all·HWP 저장/재열기, 실제 UI 3조합 replace-all·undo/redo PASS |
| 실제 Chrome 교차 구역 | 8개 입력 진입점, 비어 있지 않은 undo/redo, 빈 누름틀, 정상 대조군 PASS |
| 실제 Chrome undo | 기존 계약 + HF/FN 9사례 + Unicode 치환 3사례 PASS |

Chrome for Testing 154.0.8037.57, Linux x86_64, Node v24.15.0, npm 11.12.1, Rust 1.93.1에서 실행했다.
기존 SKIP 2건은 `pkg-node` 전용 혼합 모양/서식 왕복 검사이며 신규 검사는 실행했다.
실제 OS 클립보드·마우스 드래그·IME 후보창은 미검증이다. DOM clipboard/composition과 실제 키보드를 구분한다.
Docker 최적화 배포 검증이 아닌 host release `--no-opt` fresh WASM UI 검증이다.
브라우저 실행 시 `pkg/`와 Studio public의 JS/WASM 바이트가 같으며 이전 검색 수정 전 WASM과는 다르다.

| 검증 산출물 | SHA-256 | 바이트 |
| --- | --- | ---: |
| JS | `7dcd8775b5ff2e6d6d46f6811d03cd3e0f0db9d7d0f0d1a0606741e762bce06e` | 475795 |
| WASM | `3daf35122ef497f05e4927a85ee449a564d7f615b0eccfdffbf6c838f0674229` | 22343456 |

원장은 ignored `output/pr-review/donggyun112-20261017/`의 `rebased-rust-gates.json`,
`three-pr-rust-gates.json`, `three-pr-frontend-early-gates.json`, `three-pr-browser-gates.json`,
`three-pr-fresh-wasm-identity.json`, `rebase-input-equivalence.json`과 `logs/`다.
공유 `target/pr-review`는 보존하며 검증용 생성 glue는 제출 코드에 섞지 않는다.

## 조판 원칙 준수 검토

| 항목 | 근거 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 위 개별 계약과 실제 호출 경로; 문서 ID 예외·픽셀 보정 없음 | 충족 |
| 측정·배치 일관성 | 줄 메트릭·치수·레이아웃 계산을 변경하지 않음 | 비해당 |
| 최종 출력 소비 계약 | 편집기의 실제 WASM·브라우저 경로로 내용·커서·히스토리 검사 | 충족(편집 계약); 출력 피델리티 주장은 없음 |
| 분할·이어받기 | pagination·rowspan·continuation 변경 없음 | 비해당 |
| 줄 소속과 점유 높이 | 저장 LineSeg·높이·폭·글꼴 변경 없음 | 비해당 |
| 사례와 독립성 | scalar 위치 및 요청하지 않은 변경 금지 계약; 테스트 대역과 실제 WASM 구분 | 충족 |
| 기준값 변경 | golden·baseline·허용치 완화 없음 | 비해당 |
| 시각 회귀 추가 선행 조건 | 렌더링 위치/모양 회귀를 추가하지 않음; 편집 내용·순서·거절 검사만 추가 | 비해당 |
| 수식·간격 확대 판독 | 수식·paint·조판 치수 수정 및 피델리티 개선 주장 없음 | 비해당 |
| 주장과 증거 | 개별 검사 및 위 실제 실행 결과; 원 PR CI를 통합 head CI로 대용하지 않음 | 충족(로컬 검증 범위) |

Native/Print PDF Visual Sweep은 비해당이다. 바뀐 값은 HF/FN 편집기의 문자 위치와
지원하지 않는 선택 요청의 허용 여부이며, 추가 #7988은 검색 결과의 원문 문자 범위다. 모델·serializer의 속성/저장 조판 정보·줄 메트릭·paint를
변경하지 않았으며, 일반 요청의 조판 소비 규칙도 바꾸지 않는다. 새로운 텍스트 내용이 화면에
반영되는 사실을 조판 알고리즘·한컴 출력 피델리티 개선으로 주장하지 않는다.

## Merge 후 contributor PR comment 계획

원 PR을 직접 merge한 것으로 표시하지 않는다. 통합 PR의 실제 MERGED·merge SHA,
최종 code/trailing CI와 devel의 개별 review·적용 기록 존재를 확인한 뒤 한국어 존댓말로
감사와 수용 범위를 게시한다. 해당 PR의 고유 검증·메인터너 보정 여부와 남은 미검증을
분리하고 통합 PR·개별 기록·실제 CI URL을 direct link로 연결한다. 게시 뒤 UTF-8 원문·선두
BOM·URL을 API로 확인하고 원 PR을 close한다. contributor fork는 보존한다.
시각 피델리티 수치나 미실행 Native/Print·OS IME 검사를 통과로 적지 않는다.

## 통합 PR code CI와 남은 후속 처리

[통합 PR #7997](https://github.com/edwardkim/rhwp/pull/7997)의 정확한 code head
`fd3d97b0b9effa2d0d4a1f65f47e3475ba4ec5c8`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/39041517840) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39041517836) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39041517829) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/39041517834) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/39041517854) · [Controller](https://github.com/edwardkim/rhwp/actions/runs/39042954892)를 확인했다.
집계 Build & Test·언어별 CodeQL·Canvas·Adapter·Proptest와 정책 status가 완료됐고
실패·진행 중 check는 없다. GHAS CodeQL의 neutral과 영향 정책상 skip을 실제 성공 worker와 구분한다.
PR 생성 전 source PR CI를 재사용한 결과가 아닌 이 통합 PR의 실제 실행이다.

사용자는 CI 모니터링 이후 병합·후속 처리를 승인했다. 이 기록은 동일 PR의 문서-only trailing이며
제품·테스트·workflow·fixture·baseline은 변경하지 않는다. 최신 base와 merge tree의 문서 링크·
오늘할일 보존을 확인하고 push한 뒤, trailing head의 실제 fast-pass·required aggregate·mergeability를
다시 확인한다. 아직 실행 전인 trailing CI·merge·이슈 종료·정리는 완료로 표시하지 않는다.

원본 로컬 영수증·로그의 종료 후 보존 경로는 `/home/tsjang/rhwp-pr-evidence/pr-7997/`다.
저장소의 ignored `output/pr-review/donggyun112-20261017/`는 실행 당시 경로이며,
최종 보존 파일을 해시 확인한 뒤 `post_merge.md`에 따라 root `output/` 전체를 정리한다.
공용 `target/pr-review`와 contributor fork는 보존한다.

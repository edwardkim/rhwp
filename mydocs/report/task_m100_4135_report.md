# Task M100 #4135 구현 결과 및 PR 본문 초안

- **Issue**: [#4135](https://github.com/edwardkim/rhwp/issues/4135)
- **브랜치**: `codex/issue-4135-contextual-shortcut`
- **전체 검증 후보**: `8d3fdf011`
- **최신 통합·focused 검증 후보**: `985396791`
- **통합한 upstream**: `upstream/devel@2bcf9b261` (PR #6379 포함)
- **원격 상태**: push·PR 생성 전
- **현재 판정**: 최신 devel 통합·최소 재검증·PR 본문 초안·장기 검증 산출물 정리 완료

## 1. 사용자 결과

#4135의 단축키 도달성만 고치는 데서 끝내지 않고 실제 한컴 사용자 여정까지 보정했다.

1. F5 셀 블록에서 `Ctrl/Cmd+Shift+S`를 누르면 Save As나 셀 나누기가 아니라 블록 합계를 실행한다.
2. 선택 범위의 오른쪽 또는 아래 빈 결과 가장자리에 행별·열별 합계를 한 번의 undo 단위로 기록한다.
3. 한글 IME에서도 수정자 없는 물리 `S`/`M`이 셀 나누기·합치기로 동작하고 `ㄴ`/`ㅡ`가 남지 않는다.
4. F5 1·2·3회 상태를 셀 안 회색/주황 마커와 하단 상태 문구로 구분한다.
5. 셀 계산식은 `AA` 이상 다중 문자 열 주소를 처리하고, 두 자릿수 이상 결과도 일반 셀 텍스트 치환 경로로
   기록해 화면과 저장 데이터가 일치한다.

작업지시자는 실제 macOS 한글 IME에서 셀 나누기 뒤 `ㄴ`이 남지 않는 것과 F5 단계 표시를 각각
`수정이 반영되었어.`, `확인되었어.`로 승인했다.

## 2. 검증 결과

전체 검증 후보 `8d3fdf011`에서 다음 결과를 확보했다.

| 범위 | 결과 |
| --- | --- |
| Rust focused | #4135 5/5, `table_calc` 33/33 통과 |
| Studio focused | 56/56 통과 |
| unit tier policy | 4,225 tests / 299 modules, 통과 |
| release build | `cargo build --locked --release`, 통과 (11분 45초) |
| release lib | 4,075 통과 / 13 ignored / 0 실패 |
| release-test 전체 | 8,556 통과 / 43 skipped / 0 실패 |
| Native Skia | lib 전체 통과, placeholder 2/2, direct PDF 4/4 통과 |
| 정적 검사 | `cargo fmt --all -- --check`, `git diff --check`, clippy `-D warnings` 통과 |
| Rust doc test | 8 통과 / 3 ignored / 0 실패 |
| Studio TypeScript | `npx --no-install tsc --noEmit`, 통과 |
| Studio 전체 | 1,264 tests / 1,263 통과 / 1 skip / 0 실패 |
| Studio production build | Vite 242 modules, 통과 |
| 사용자 수동 검증 | 한글 IME 셀 나누기·F5 단계 UX 통과 |

최신 `upstream/devel@2bcf9b261`을 충돌 없이 통합한 `985396791`에서는 변경 범위의 최소 재검증을
다시 수행했다.

| 범위 | 최신 통합 후보 결과 |
| --- | --- |
| Rust focused | #4135 5/5, `table_calc` 33/33 통과 |
| Studio focused | 43/43 통과 |
| unit tier policy | 4,225 tests / 299 modules, 통과 |
| Studio TypeScript | 통과 |
| Studio 전체 | 1,272 tests / 1,271 통과 / 1 skip / 0 실패 |
| Studio production build | 깨끗한 `dist`에서 Vite 242 modules, 통과 |
| 포맷 | `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check` 통과 |

Docker 표준 WASM build는 현재 후보에서 완료 판정하지 않는다. 중지돼 있던 x86_64 Colima의 빈 이미지와
Cargo cache를 새로 만들면서 `wasm-pack` 설치에 11분 이상이 들었고, x86 에뮬레이션 release/LTO가 장시간
계속돼 작업지시자가 PR 준비 범위를 본문 초안까지로 축소한 뒤 컨테이너를 중지했다. 이는 컴파일 오류가
아니지만 성공도 아니다. R4의 이전 코드 후보에서는 fresh WASM과 embed E2E가 통과했으나 현재 후보의
통과 결과로 승격하지 않는다.

## 3. #6379와 장시간 검증의 관계

[PR #6379](https://github.com/edwardkim/rhwp/pull/6379)는 PR에서 새로 추가된 sample만 보안 clean sweep 대상으로
삼도록 `security_corpus_regression`과 `injection_scan_contract`의 CI 선택 범위를 줄인다. 따라서 첫 전체
nextest에서 오래 걸렸던 보안 코퍼스 전수 검사에는 관련이 있다.

반면 중지한 작업은 Docker 내부의 x86_64 WASM release/LTO **컴파일**이다. #6379는 CI 테스트 선택 정책을
바꿀 뿐 WASM 컴파일 경로·프로파일은 바꾸지 않으므로 7시간 남아 있던 컨테이너와는 관련이 없다.

## 4. 장기 검증 산출물 정리

이번 작업에서 만든 3GB 검토 worktree, 8.1GB `target/pr-review`, 48MB Studio `dist`를 제거했다. Docker의
`rhwp-wasm:latest`, `rhwp_*` volume 4개, `rhwp_default` network와 이번 빌드 시각의 BuildKit cache 11개도
제거했다. 정리 뒤 해당 Docker 프로필은 기존 `fedora:42` 이미지만 남고 container·volume·build cache가
모두 0임을 확인한 뒤 Colima를 종료했다. 이전 R4에서 만든 유효 `pkg`는 이번 실패 산출물이 아니므로
보존했다.

## 5. push 전 남은 항목

최신 `upstream/devel@2bcf9b261` 통합과 변경 범위 최소 재검증은 완료했다. 다음은 실제 push 승인 뒤 수행한다.

1. 원격이 더 진행됐는지 fetch로 최종 확인한다.
2. `cargo fmt --all`과 `cargo fmt --all -- --check`, `git diff --check`를 push 직전에 다시 실행한다.
3. 표준 WASM·browser 결과는 GitHub required checks 또는 정상적인 native Docker 환경에서 확인한다.

## 6. 제안 PR 제목

```text
fix(studio): F5 셀 블록 계산과 한글 IME 셀 명령을 바로잡는다
```

## 7. 복사 가능한 PR 본문 초안

```markdown
## 변경 요약

- F5 셀 블록의 `Ctrl/Cmd+Shift+S`를 Save As·셀 나누기보다 먼저 블록 합계로 라우팅합니다.
- 선택 범위의 오른쪽/아래 빈 결과 가장자리에 행별·열별 합계를 all-or-nothing snapshot으로 기록합니다.
- 한글 IME에서도 물리 `KeyS`/`KeyM` 셀 나누기·합치기를 처리하고 후속 `ㄴ`/`ㅡ` 입력을 억제합니다.
- F5 1·2단계를 셀 안 회색/주황 마커로, 1·2·3단계를 하단 상태 문구로 함께 표시합니다.
- `AA` 이상 계산식 열 주소와 두 자릿수 이상 결과의 셀 렌더링·저장 경로를 보정합니다.

## 관련 이슈

closes #4135

## 테스트

- [x] `cargo fmt --all -- --check` 통과
- [x] 최신 `upstream/devel@2bcf9b261` 통합 및 변경 범위 focused 재검증
- [x] `src/**` test 변경에 대한 `node scripts/rust-unit-test-tiers.mjs --check` 통과
- [x] focused Rust: #4135 5/5, table calculation 33/33
- [x] release lib: 4,075 pass / 13 ignored
- [x] release-test 전체: 8,556 pass / 43 skipped
- [x] Native Skia lib + 지정 회귀 2/2, 4/4 통과
- [x] `cargo clippy --locked --all-targets -- -D warnings` 통과
- [x] Rust doc test: 8 pass / 3 ignored
- [x] Studio TypeScript 검사 및 production build 통과
- [x] 최신 통합 후보 Studio 전체: 1,272 tests / 1,271 pass / 1 skip
- [x] 실제 macOS 한글 IME에서 셀 나누기 후 `ㄴ` 미입력 확인
- [x] 실제 browser에서 F5 1·2·3회 마커·하단 상태·Escape 해제 확인
- [ ] 최신 devel 통합 후보의 표준 Docker WASM build / embed E2E

상세 구현·수동 검증 기록:

- `mydocs/plans/task_m100_4135_impl.md`
- `mydocs/working/task_m100_4135_recovery_r4.md`
- `mydocs/working/task_m100_4135_recovery_r5.md`
- `mydocs/report/task_m100_4135_report.md`

전체 Rust 게이트 후보 통과 뒤 최신 devel을 통합했고, 새 head에서 Rust/Studio focused, TypeScript,
Studio 전체와 production build, 포맷을 다시 통과했습니다. Docker WASM은 x86_64 Colima 빈 캐시의
release/LTO가 장시간 진행돼 로컬 성공으로 기록하지 않았으며, required check 또는 정상 native Docker
환경에서 확인합니다.

## 성능 영향 및 측정 결과

- 예상 영향: 일반 입력 경로 영향 없음. 셀 블록 계산 시 선택 범위 크기에 비례하는 preflight와 결과 기록이 추가됩니다.
- 재현·측정: 별도 성능 benchmark는 실행하지 않았습니다. focused·전체 회귀에서 기능 회귀는 없었습니다.

## 스크린샷

- F5 1회: 포커스 셀 중앙 회색 마커 + `셀 선택 · 방향키로 이동`
- F5 2회: 포커스 셀 중앙 주황 마커 + `셀 범위 선택 · 방향키로 확장`
- F5 3회: 표 전체 선택 + `표 전체 선택`
```

원격 push, PR 생성, GitHub comment는 작업지시자의 별도 승인 전에는 수행하지 않는다.

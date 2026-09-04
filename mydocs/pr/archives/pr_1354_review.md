# PR #1354 검토 — 수식 PUA 조건부 막대 U+E04D 매핑

- PR: https://github.com/edwardkim/rhwp/pull/1354
- 제목: Task #1343: 수식 PUA 조건부 막대(U+E04D) → '|' 매핑
- 작성일: 2026-06-10
- 작성자: `planet6897`
- 관련 이슈: #1343 "수식 조건부 막대 U+E04D(PUA)가 두부(▦) 박스로 렌더"
- base: `devel` (`cb4923dc`)
- head: `planet6897:fix/equation-pua-conditional-bar-1343` (`24156ad0`)
- 로컬 검토 브랜치: `local/pr1354-upstream`

## 1. 요약 판단

**수용 가능**으로 판단한다.

PR의 본질은 HWP 수식 스크립트 안에 PUA 문자 `U+E04D`로 저장된 조건부 확률 막대를 표준 기호
`|`로 매핑하는 것이다. 문제 범위는 #1343에 정확히 대응하고, 변경도 수식 tokenizer/symbol table에
한정되어 있다.

다만 PR에는 source 변경 외에 contributor 작업 문서가 활성 `mydocs/plans`, `mydocs/report`,
`mydocs/working` 폴더에 함께 추가된다. 수용 시에는 소스 변경은 그대로 반영하되, 작업 문서는
프로젝트 archive 정책에 맞춰 이동하는 것이 좋다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | MERGEABLE |
| 변경량 | 7 files, +216 / -0 |
| 작성자 | `planet6897` |
| 관련 이슈 | #1343 |

커밋:

- `24156ad0` — `Task #1343: 수식 PUA 조건부 막대(U+E04D) → '|' 매핑`

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

## 3. 변경 검토

### 3.1 수식 PUA 매핑

`src/renderer/equation/symbols.rs`:

- `EQUATION_PUA` 매핑 추가
  - `U+E04D` → `"|"`
- `lookup_equation_pua(ch)` 조회 함수 추가

판단:

- HWP 수식 도메인 안에서만 적용되는 매핑이므로 영향 범위가 좁다.
- `U+E04D`가 조건부 확률 막대라는 이슈 본문 및 PR 본문 설명과 일치한다.
- 향후 다른 HWP 수식 PUA가 발견될 경우 같은 테이블에 확장 가능하다.

### 3.2 tokenizer 처리 위치

`src/renderer/equation/tokenizer.rs`:

- 단일 ASCII symbol, 숫자, 영문 command 처리 이후
- 기존 non-ASCII `Text` 폴백 직전에 PUA 매핑을 조회
- 매핑되면 `TokenType::Symbol` + `"|"` 토큰으로 반환

판단:

- 샘플 스크립트의 `rm P LEFT ( it A U+E04D B RIGHT )`에서는 `U+E04D`가 ASCII/공백 사이 단일
  문자로 나타나므로 이 위치에서 정확히 잡힌다.
- 생성되는 토큰이 기존 단일 `|`와 같은 `Symbol("|")`이므로 parser/layout/render 경로를 재사용한다.
- 매핑되지 않은 PUA는 기존 non-ASCII `Text` 폴백으로 내려가므로 알 수 없는 PUA의 동작은 유지된다.

남는 작은 주의점:

- non-ASCII 연속 문자열 중간에 PUA가 섞인 형태, 예를 들어 `한\u{E04D}글` 같은 경우에는 앞의
  non-ASCII `Text` 루프가 PUA까지 함께 소비할 수 있다. 이번 이슈의 수식 스크립트는 Latin/명령어
  중심이라 blocker는 아니지만, PUA 매핑 테이블이 늘어나면 Text 루프 안에서도 PUA를 끊는 보강을
  고려할 수 있다.

### 3.3 문서 파일

PR은 다음 문서를 활성 폴더에 추가한다.

- `mydocs/plans/task_m100_1343.md`
- `mydocs/plans/task_m100_1343_impl.md`
- `mydocs/report/task_m100_1343_report.md`
- `mydocs/working/task_m100_1343_stage2.md`
- `mydocs/working/task_m100_1343_stage3.md`

외부 PR 처리 문서는 `mydocs/pr/pr_1354_review.md`, `mydocs/pr/pr_1354_report.md`로 관리하는 것이
현재 프로젝트 절차에 맞다. 다만 contributor 작업 문서 자체는 작업 이력으로 의미가 있으므로,
수용 시 archive로 이동하는 것을 권장한다.

권장 이동:

- `mydocs/plans/archives/task_m100_1343.md`
- `mydocs/plans/archives/task_m100_1343_impl.md`
- `mydocs/report/archives/task_m100_1343_report.md`
- `mydocs/working/archives/task_m100_1343_stage2.md`
- `mydocs/working/archives/task_m100_1343_stage3.md`

## 4. 로컬 검증

검토 브랜치: `local/pr1354-upstream`

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib test_pua_conditional_bar -- --nocapture` | 통과 |
| `git diff --check local/devel...HEAD` | 통과 |
| 최신 `local/devel` 기준 source patch `git apply --check` | 통과 |

GitHub Canvas visual diff도 pass 상태다.

## 5. 리스크

| 리스크 | 평가 | 완화 |
|---|---|---|
| PUA 매핑이 수식 외 텍스트에 적용될 가능성 | 낮음 | tokenizer가 수식 파이프라인 전용 |
| 알 수 없는 PUA 동작 변경 | 낮음 | 매핑 없는 PUA는 기존 Text 폴백 유지 |
| non-ASCII 연속 문자열 중간 PUA 미매핑 | 낮음 | 이번 샘플 구조와 무관, 후속 확장 시 보강 가능 |
| contributor 작업 문서가 활성 폴더에 남음 | 중간 | 수용 시 archives로 이동 |
| 시각 회귀 | 낮음~중간 | GitHub Canvas visual diff pass, 수용 후 SVG/WASM 검증 권장 |

## 6. 권장 수용 절차

작업지시자 승인 후:

1. `local/devel` 기준으로 PR 커밋 `24156ad0` cherry-pick
2. contributor 작업 문서 5개를 archive 폴더로 이동
3. 검증
   - `cargo fmt --check`
   - `git diff --check`
   - `CARGO_INCREMENTAL=0 cargo test --lib test_pua_conditional_bar -- --nocapture`
   - `CARGO_INCREMENTAL=0 cargo test --lib renderer::equation::tokenizer -- --nocapture`
   - `CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings`
4. 필요 시 대상 샘플 16페이지 SVG export 및 작업지시자 시각 판정
5. WASM 빌드 후 rhwp-studio 시각 판정 여부 결정
6. 처리 보고서 작성
7. 승인 시 `devel` no-ff merge, push, PR #1354 close, issue #1343 close

## 7. 승인 요청

위 검토 결과 기준으로 PR #1354 수용 절차를 진행해도 되는지 승인 요청한다.

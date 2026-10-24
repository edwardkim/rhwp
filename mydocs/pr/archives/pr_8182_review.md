# PR #8182 리뷰 — 표시 문자열 export와 모델 오프셋 재검토

## 최종 판정

**승인 — #7909의 읽기 전용 export 계약 범위에서 추가 결함을 발견하지 않았다.**
작성자 본인의 별도 재검토 기록이며 GitHub approve event나 merge 승인이 아니다.
검토한 head는 `a46cf501d38e5f900360044b24d75169ded07e19`, base는
`def947c2149df5bc812d801e6e79be19a403b4d1`이다. 최종 기록 commit 뒤에는 새 head의
required check를 별도로 확인해야 한다. 실제 API 소비자의 표시/편집 인덱스 검토는 사용자 확인으로 남는다.
최초 재검토 당시 merge·release·배포·이슈 종료는 승인 범위 밖이었다. 2026-10-24 사용자가 리뷰 기록 반영과 최신 CI 확인 후 merge를 추가 승인했다. release·운영 배포·수동 이슈 종료는 수행하지 않는다.

## 접수와 변경 범위

- [PR #8182](https://github.com/edwardkim/rhwp/pull/8182), 작성자 postmelee,
  devel 대상 Open PR, reviewer 미지정인 collaborator self 경로다.
- [이슈 #7909](https://github.com/edwardkim/rhwp/issues/7909)와 댓글 0건, PR 댓글 0건,
  14개 변경 파일, 원격 base/head 및 최신 devel을 다시 확인했다.
  코드 재검증 당시 devel은 기준 SHA와 같았다. trailing push 직전에는
  `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`로 전진했으므로 새 base의 병합 트리에서
  문서 링크·기존 오늘 기록 보존을 다시 검사했다. source에 merge/rebase하지 않았으며
  새 base의 제품 변경을 이 PR의 구현이나 기능 검증으로 주장하지 않는다.
- 공통 `get_page_text_layout_native`의 JSON export만 동작을 바꾼다.
  wasm_api는 계약 주석만 수정했고, CLI도 같은 getter를 호출한다.
  #7910, renderer의 display 생성 규칙, 모델·parser·serializer·paint는 변경하지 않았다.

## 구현과 검증 주장 대조

| 검토 항목 | 근거와 판정 |
| --- | --- |
| 선행 계약 | #3323의 모델/표시 분리는 통합 #3325의 `9c26be219babbcc9d53c6fc6d20ee46bb6c4f844`, #7053의 혼합 자동번호 치환은 `522a2e80db04cbd84264406ccb8bdd33a21dcc55`에서 확인했다. 두 원 PR의 merged 플래그로 통합 여부를 추정하지 않았다. 충족. |
| 원인 계층과 일반성 | `TextRunNode.display_text`를 공통 순회에서 optional 두 키로 export한다. 파일명·이슈 번호·좌표별 예외나 문자열 교체는 없다. 충족. |
| 모델 오프셋 | 기존 text/charX/charStart 및 문서·셀 식별자·bbox를 수정하지 않는다. 원본 69런·첨부 대조 70런의 기존 전체 JSON 비교도 다시 확인했다. 충족. |
| 표시 경계 | 기존 replay_positions_for(display)를 사용한다. 검증된 layout_positions 우선, 기존 계산 fallback, scalar 수를 보존하는 PUA 투영을 추적했다. 기존 렌더 소비 경로와 같은 helper다. 충족. |
| JSON·경계값 | 기존 json_escape의 모든 C0·quote·backslash 처리를 재사용한다. 빈 표시값, 비 BMP, 긴 파일명과 동적 필드 갱신을 실제 WASM으로 추가 실행했다. 충족. |
| 기존 소비자 | rhwp-dev의 모델 text/charStart 소비는 그대로 동작한다. 표시 인덱스에서 모델 오프셋으로의 대응표를 제공한다고 주장하지 않는다. 실제 외부 앱 연동은 사용자 확인으로 남는다. |
| 조판·시각 정책 | 동작 diff는 JSON export 한 곳이며 조판·측정·paint·page-visible 경로와 baseline/golden은 수정하지 않았다. 신규 검사는 API 계약 검사다. 배치 개선·한컴 PDF 일치·전쪽 시각 일치율 판정은 비해당이다. SVG 조회 전후 불변을 별도 확인했다. |
| 분할·저장 LineSeg·재페이지네이션·자원/글꼴 | 생산 코드 변경이 없으므로 비해당. 파생 대조본의 LineSeg 무효화는 fixture 생성 절차이며 원본은 보존했다. |
| 입력 커밋 일치 | 생산 소스 2개·Rust 검사·fixture 디렉터리 전체·기존 #6986 HWPX 총 10개 파일을 실제 실행 bytes와 head의 git object로 대조했다. 모두 동일. 충족. |
| 테스트 기대값 | 원시 atno가 없는 첨부 no-picture를 정상 음성 대조로 유지하고 번호 보존 파생본을 별도 사용한다. base 4 FAIL/3 PASS 로그와 후보 7 PASS를 다시 확인했다. 기대값·허용치를 완화하지 않았다. |

## 실제 재실행 결과

코드와 fixture는 변경하지 않았다. 광범위 전체 회귀·Native Skia와 lint는 기존 로컬
[결과보고서](../../report/task_m100_7909_report.md) 및 정확한 head의 Full CI로 재확인하고,
같은 전체 검사를 중복 실행하지 않았다. 이번 추가 실행을 이전 전체 11,369건의 재실행으로 세지 않는다.

| 명령·검사 | 이번 결과 |
| --- | --- |
| `node scripts/run-rust-test.mjs issue_7909_text_layout_display -- --cargo-profile release-test --target-dir /workspace/rhwp/target/pr-review --no-fail-fast` | 7 PASS / 0 FAIL, suite의 다른 255건은 focused filter 제외 |
| `node tests/fixtures/issue7909/verify-wasm.mjs pkg` | 실제 WASM 원본/대조 4개·HWPX 재열기·선택/커서/hit/edit PASS |
| `node work/issue-7909/re-review/boundaries.mjs` | 8개 추가 사례 PASS. 파일명 0/8/10/512 scalar, 빈 문자열·이모지·보충 평면 문자·개행/탭/quote/backslash, 변경 후 재조회·필드 뒤 삽입; 4개 fixture 각각 HWP/HWPX 저장·재열기 |
| `node work/issue-7909/browser.mjs` | Chromium 151.0.7922.173에서 4개 입력, 번호 유무·문구·N+1 경계·SVG 불변 PASS |
| pkg/public JS·WASM bytes와 보고서 SHA-256 | 모두 일치. 검증된 최적화 산출물과 다른 번들을 실행하지 않았음 |
| 기존 JSON 비교 / `git diff --check base...HEAD` / 변경 Markdown 6개 상대 링크 | PASS |

이번 로컬 상세 로그와 추가 경계 스크립트는 ignored `work/issue-7909/re-review/`에 보존했다.
전체 문서 metadata의 기존 오류 20건은 여전히 FAIL이며 이번 diff 밖 5개 파일에서 발생한다.
변경 장기 문서 metadata 오류 0건과 구분한다.

## 정확한 head의 원격 CI

- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/39943620439): success.
  Lint, Archive A/B/C/D 실제 검사·합계 검증, Native Skia, frontend fresh WASM/package gate가 성공했다.
  개별 WASM Build job의 skipped를 별도 실행 성공으로 세지 않았다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39943620525),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39943620689),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39943620497),
  [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39943620617): 모두 success.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/39946478179): success.
  직전 pending은 로그상 `workflow-not-completed:CI:in_progress`였으며 성공으로 덮어 보고하지 않았다.

## 미검증 범위와 사용자 확인

외부 소비자 앱 연동, 비공개 HWPX, HWP3 자동번호 별도 focused 검사, 한컴 Print PDF와의
시각 일치는 검증하지 않았다. #7909의 HWP5 원본 및 rhwp 생성 HWP/HWPX export 계약과 구분한다.
사용자는 `displayText ?? text`와 `displayCharX ?? charX`로 표시하되, 파일명 확장 필드 뒤의
선택·커서·삽입에는 기존 hit-test의 모델 오프셋을 사용해 앱에서 확인해야 한다.
[계약 문서](../../tech/text_layout_display_contract.md)와 [입력 출처](../../../tests/fixtures/issue7909/README.md)를 따른다.

최초 작성 세션은 원격 approve/comment·merge·close·release·배포를 수행하지 않았다. 아래는 추가 사용자 지시에 따른 후속 검토 기록이다.

## 2026-10-24 독립 재검토

## 검토한 구현과 검증


### 확인한 코드

동작 변경은 get_page_text_layout_native의 optional displayText/displayCharX export다. 기존 text/charX/charStart/bbox/셀 주소 직렬화는 그대로이고, displayCharX는 TextRunNode.replay_positions_for(display)를 소비한다. helper의 layout_positions 검증 및 fallback, Unicode scalar 경계, 기존 JSON escaping, 빈 표시값과 모델 오프셋 구분을 확인했다. wasm_api 변경은 계약 주석이다. 조판·paint·저장 모델 변경이 아니다. 새 actionable code finding 없음.

### 이번 직접 실행

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref def947c2149df5bc812d801e6e79be19a403b4d1
node scripts/run-rust-test.mjs issue_7909_text_layout_display -- --cargo-profile release-test --target-dir /Users/melee/Documents/projects/forks/rhwp/target/pr-review --no-fail-fast
```

- Native focused: **7 PASS / 0 FAIL**. 같은 suite의 다른 255개 검사는 focused filter로 제외했다.
- PR base를 고정한 manifest 정책 검사: PASS.
- 공개 원본·번호 없는 음성 대조·번호를 유지한 그림 없는 생성 HWP/HWPX, 일반 문구, 모델 선택/커서/편집 유지 검사를 실제 실행했다.
- fixture 4개 SHA-256이 README와 일치하며 검토 head의 Git blob에 포함됨을 확인했다. no-picture 원본에는 자동번호도 없으므로 이를 양성 대조로 오해하지 않았다.
- 이번에 전체 Rust/Skia/WASM 빌드를 반복하지 않았다. [정확한 code candidate CI](https://github.com/edwardkim/rhwp/actions/runs/39943620439)의 PR #8182, head a46cf501d38e5f900360044b24d75169ded07e19, source branch, 성공 상태를 확인했다. lint·Native Skia·archive A/B/C/D·frontend package가 실제 success다.
- candidate와 현재 head 사이 diff는 mydocs/orders/20261023.md 및 pr_8182_review.md뿐이다. 최신 head CI preflight 로그에서 그 candidate의 성공 결과를 재사용했음을 확인했다. skipped worker를 재실행 성공으로 세지 않았다.
- 실제 외부 소비자 앱 연동, HWP3 별도 focused, 비공개 HWPX, 독립 한컴 Print PDF 일치는 미검증이다. 읽기 전용 export 수용을 배치 품질 보증으로 확대하지 않는다.

사용자 확인: displayText ?? text와 displayCharX ?? charX로 표시하되, 파일명처럼 한 모델 문자가 여러 표시 문자로 늘어나는 필드 뒤의 선택·삽입은 기존 hit-test가 반환하는 모델 오프셋으로 실행한다.

## 2026-10-24 병합 승인과 검토 범위

- 검토 head: `d097a922ae469778d4860816c747512e059754b6`. 검토 base: `def947c2149df5bc812d801e6e79be19a403b4d1`.
- 문서 반영 전 최신 devel: `e7a0dec55c6c6c6365e4ff50babe7583e1cf1c70`.
- 사용자가 각 PR의 리뷰 코멘트, 같은 source branch의 리뷰 문서 반영, 최신 CI 확인 후 merge를 명시적으로 승인했다. 운영 배포는 승인 범위에서 제외했다.
- [게시한 리뷰 코멘트](https://github.com/edwardkim/rhwp/pull/8182#issuecomment-6379332092). collaborator self 경로이며 GitHub APPROVE 이벤트를 제출하지 않는다.
- 사용자 실제 환경 확인은 미검증으로 유지한다. 이를 자동 검사로 대체하거나 완료로 표시하지 않는다. 사용자의 이번 병합 지시에 따라 검증된 변경 범위의 수용과 후속 사용자 확인을 구분한다.
- 이번 후행 변경은 리뷰·오늘할일 문서뿐이다. source/test/fixture/workflow/baseline을 변경하지 않는다. 최신 devel을 source branch에 merge/rebase하지 않는다.
- 코드 검토와 실행에서 추가 결함 없음. 조판/측정/paint/페이지 분할/저장 LineSeg 및 기준값 변경은 비해당이다. Visual Sweep을 실행했거나 한컴 출력과 일치한다고 주장하지 않는다.
- push 전 고정한 base/head의 merge-tree, 문서 상대 링크, 공백, 양쪽 오늘할일 기록 보존을 확인한다. 새 trailing head의 CI·실제 fast-pass·mergeability·head SHA를 다시 확인한 뒤에만 squash merge한다. 검사 실패·pending을 우회하지 않는다.

## Merge 후 contributor PR comment 계획

merge 후 실제 merge SHA와 이 리뷰 문서의 영구 링크, 마지막 head의 CI URL 및 fast-pass/reused candidate를 코멘트에 기록한다. 이번 직접 실행과 작성 세션의 기존 검증을 구분하며 미검증 범위를 그대로 남긴다. 연결 이슈 [#7909](https://github.com/edwardkim/rhwp/issues/7909)의 실제 상태를 조회해 기록하되 이슈 수동 종료·운영 배포는 수행하지 않는다. 별도의 후속 문서 PR은 만들지 않는다. 이번 검토 전용 clean worktree는 정리하고 다른 세션의 source branch와 공유 target은 보존한다.

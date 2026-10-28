---
kind: report
status: active
last_verified: 2026-10-02
---

# PR #7493 리뷰 — Studio 수정 모드·IME·Undo

## 최종 판정

승인 — 기여자의 수정 head `849955949f2f0d4a3f98e0006e727d6f3443333f`에서 기존 보류 사유를 재검증해 해소했습니다.

아래 최초 검토는 당시 실패의 이력입니다. 현재 판정은 마지막 재검증 절의 exact head와 결과에 근거합니다.
병합 직전 최신 head·required CI·작업지시자 승인 범위를 다시 확인합니다.

검토일: 2026-10-02. 작성자: semanticist21. 대상: devel.
기준 devel: `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`.
누적 진단 head: `c6ef30ea943308c37e5d68c8304dfdabdd7b8f74`.

누적 실행 명령·로그·제한은 [일괄 검토 기록](../pr_semanticist21_20261002_review_impl.md#누적-검증-결과)에 연결한다.
원 PR의 exact-head 녹색 CI와 누적 진단 head의 결과는 별개다. 누적 head는 7건을 포함하며
원 PR 또는 최종 수용 그룹의 전체 CI 통과로 간주하지 않는다. 메인터너 source/test 보정은 없다.

원 PR code head: `d29d483e4f5fc759c067e33766700bcad87133ed`.
[원 PR](https://github.com/edwardkim/rhwp/pull/7493) · [exact-head Build & Test](https://github.com/edwardkim/rhwp/actions/runs/36847784882/job/110327871695).
CI 집계 실패·진행 중 없음(확인 당시). 원 PR head는 최초 접수 이후 바뀌지 않았다.
Reviewer edwardkim 지정. 원격 GitHub 승인 이벤트는 아직 게시하지 않았다.

## 범위와 검증

#7489 종료 제안. Studio 입력·InsertTextCommand·fragment/Undo만 변경하며 Rust 조판 수정은 없다.
누적에 6개 기능 commit을 적용했다. TypeScript PASS, Studio 전체 1813 PASS / 2 skip / 0 FAIL.
작성자의 overwrite runner를 직접 실행한 25개 시나리오는 PASS했다.
Node 24.15.0에서 `allowedNodeEnvironmentFlags` 기반 지원 판단 때문에 정식 wrapper가
실제 실행 가능한 `--experimental-transform-types` 테스트를 skip했다. 직접 runner를 실행해 보완했다.

## 추가 반례 — 실행 결함과 검토상 우려

실제 TypeScript handler/command/history를 작성자의 scalar-index mock core와 실행했다.
이는 브라우저/WASM 사용자 여정 검증이 아니라 재현 가능한 명령 계층 반례다.

| 반례 | 기대 | 관측 |
| --- | --- | --- |
| abcd에서 수정 모드로 😀 다음 X 입력 | 😀Xcd | 😀bXd |
| 첫 😀 뒤 caret을 scalar 1로 맞춰 X 입력 후 한 묶음 Undo | abcd | 😀bcd |
| IME 조각 생성 후 실제 deactivate 호출 | 조각 해제 또는 복원 | 보관 조각 1개 잔류 |

Core의 `get_text_range_native`는 Unicode scalar 단위다.
`command.ts`의 703·713·764줄 등은 JS UTF-16 `text.length`로 replay/caret/merge 경계를 정한다.
기존 삽입 명령의 위치 가정이 새 수정 모드에서 잘못된 덮어쓰기 대상으로 연결된다.
두 번째 반례는 Undo에서 먼저 실패했으므로 Redo 실패를 실행했다고 주장하지 않는다.
`input-handler.ts:4421`의 deactivate는 fragment ID를 null로 만들지만 저장소의 조각을 해제하지 않는다.
`dispose`의 같은 처리도 코드상 우려이지만 이번 실행 반례는 deactivate만 확인했다.

## 해제 조건과 범위

기여자가 scalar/UTF-16 경계를 일관되게 처리하고 명령 병합·Undo/Redo·IME 중 문서 종료 수명을
검증해야 한다. wrapper의 skip 검출도 고쳐 정식 테스트에서 실행되게 해야 한다.
실제 browser/WASM 편집 검증은 아직 미실행이므로 UI 전체 정상 판정은 하지 않는다.
소스에 임의 메인터너 보정을 넣거나 기존 필드 Redo 문제까지 이번 결함으로 합치지 않았다.
보류 근거 comment는 작업지시자의 승인 뒤 게시했다.

## 승인 후 게시 기록

2026-10-02 작업지시자의 댓글 게시 승인 후 [보류 사유 comment](https://github.com/edwardkim/rhwp/pull/7493#issuecomment-5944041776)를 게시했다.
게시 직전 원 head가 그대로 OPEN임을 확인하고 API 재조회로 한글 본문·BOM/치환 없음 및
작성 문안과의 일치를 확인했다(파일 끝 개행만 정규화). 코드 변경·push·GitHub 승인·merge 없음.

## 기여자 대응 재검증 — 2026-10-02

[대응 댓글](https://github.com/edwardkim/rhwp/pull/7493#issuecomment-5944497953)을
최신 head `849955949f2f0d4a3f98e0006e727d6f3443333f`의 실제 변경과 대조했습니다.
base route는 maintainer_general이며 intake_and_review, local_validation, rework_and_exceptions,
post_merge를 적용했습니다. 메인터너 source/test 보정은 없습니다.

| 기존 지적 | 변경과 실행 결과 |
| --- | --- |
| astral 입력의 UTF-16/scalar 혼용 | command replay·caret·merge 및 IME caret이 `charCount`를 소비합니다. 수정·삽입 모드의 연속 입력, grouped Undo/Redo 및 IME 확정이 통과했습니다. |
| 종료 시 조합 fragment 잔류 | 실제 `deactivate`·`dispose`가 fragment를 discard합니다. 문서 교체 후 옛 내용을 restore하지 않으며 새 문서·파일 열기 뒤 늦은 composition 이벤트도 검사했습니다. |
| 실행 가능한 Node runner를 skip | 플래그 추측을 제거하고 실제 자식 실행·오류·exit·성공 마커를 확인합니다. Node 24.15.0에서 wrapper가 skip 없이 통과했습니다. |

검증은 해당 head에서 다음과 같이 완료했습니다. 변경되지 않은 head의 검사를 병합 준비 과정에서 반복하지 않았습니다.

| 실행 명령·검사 | 결과 |
| --- | --- |
| Studio `npx tsc --noEmit` | PASS |
| Studio `npm test` | 1,813 PASS / 0 FAIL / 2 skip; overwrite wrapper는 skip 0 |
| `node --experimental-transform-types --no-warnings tests/support/overwrite-mode.runner.mjs` | 31개 시나리오 PASS |
| `node --experimental-strip-types --test tests/overwrite-mode.test.ts` | 1 PASS / 0 FAIL / 0 skip, 내부 runner 31개 실행 |
| 독립 최초 리뷰 반례 3건 | PASS; 기존 ignored 진단의 중복 InputHandler import만 메모리에서 제거해 실행했습니다. 최초 setup 오류는 제품 결함으로 세지 않았습니다. |
| 신규 반례의 음성 대조 | Node load hook으로 command/input-handler-text/input-handler만 `d29d483e4f5fc759c067e33766700bcad87133ed` 코드로 메모리에서 대체했습니다. 신규 6건 모두 의도한 원인으로 FAIL했고 최신 코드에서는 PASS했습니다. 작업 파일을 되돌리지 않았습니다. |
| `node e2e/run-with-vite.mjs -- node e2e/overwrite-mode-issue7489.test.mjs --mode=headless` | 실제 Chrome/WASM 편집 19/19 PASS. 키보드·Undo/Redo·IME·새 문서·파일 열기 후 조합 수명 확인 |
| current-base `git merge-tree --write-tree upstream/devel upstream/pr7493-head` | base `f911b91da3c69f9ecf867f69e369ebc181cbec8b`, head 위 SHA, tree `149f2bd9f080817a84cb8a3b90aabf52396615c1`, exit 0 |
| `git diff --check upstream/devel...upstream/pr7493-head` | PASS |

실제 E2E 보고서는 ignored `output/e2e/overwrite-mode-issue7489-report.html`입니다.
사용한 기존 root WASM SHA-256은 `aca4f15f2f6ab86180bf1bca33342b6b59c9c9868f9604fea62082036c80e869`입니다.
이 검사는 fresh WASM 빌드 또는 한컴 PDF 시각 일치의 증거가 아닙니다. 변경 범위가 Studio에 한정되고
Rust renderer/WASM 소스·새 sample·baseline 변경이 없어 local_validation 4.3의 Studio gate를 적용했습니다.
Rust 전체 회귀·Clippy·Native Skia·fresh WASM·Visual Sweep은 비해당으로 새로 실행하지 않았습니다.

검증 입력 커밋 확인: **충족**. 새 문서는 커밋된 E2E에서 생성하며 파일 열기는 기존
`samples/para-001.hwp`를 사용했습니다. 실행 파일과 해당 head의 blob은 동일하며 SHA-256은
`bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e`입니다.
조판 원칙 준수 검토: **비해당**. 입력·명령·fragment 수명 수정이며 측정·배치·페이지네이션을 변경하지 않습니다.
동작 기반 검증: **충족**. 실제 제품 handler/history와 Chrome 입력 경로를 실행했고 이전 코드의 음성 대조를 확인했습니다.

[CI run 36955697670](https://github.com/edwardkim/rhwp/actions/runs/36955697670)의 exact head와
Frontend package gates·Build & Test 성공을 확인했습니다. CodeQL, Render Diff, Adapter, Proptest도 성공했습니다.
Rust 영향 축의 skip은 Studio-only 정책이며 전체 Rust 실행 성공으로 보고하지 않습니다.
current-base merge tree는 충돌 검사만 수행했으며 그 tree를 별도로 빌드·실행한 결과는 아닙니다.

수용 범위는 본문·표 셀·글상자의 덮어쓰기 및 명시한 IME/Unicode/Undo 경계입니다.
붙여넣기·Enter/Tab·머리말/꼬리말·각주는 삽입을 유지하며 셀 내부 개체 경계, iOS fallback,
수정 모드 caret 모양 등 원 PR이 명시한 비범위는 이번 수용으로 구현 완료 처리하지 않습니다.

### 후속 처리 계획

작업지시자의 PR 처리 지시에 따라 exact head 승인·병합을 진행하고 merge SHA를 확인합니다.
review를 archive로 이동하고 일괄 기록·오늘할일만 갱신합니다. #7489에는 실제 해결 범위와
남은 비범위를 안내하며 원 PR에는 감사·병합·검증 결과를 게시합니다.
전용 로컬 response review branch만 병합 포함을 확인해 정리하고, 보류 PR의 누적 진단 branch,
공유 target/pr-review와 contributor fork branch는 보존합니다. 이 계획은 운영 기록 push의 별도 승인을 대신하지 않습니다.

### 병합 완료

[승인 리뷰](https://github.com/edwardkim/rhwp/pull/7493#pullrequestreview-5388327505)를
위 exact head에 게시하고 API로 내용을 재확인했습니다. required Build & Test 성공·head 불변·
MERGEABLE/CLEAN을 다시 확인한 뒤 작업지시자의 PR 처리 지시에 따라 병합했습니다.
병합 시각은 2026-10-02 13:31:19 KST이며 merge SHA는
`f536f15e750d9a32d33b013da7be079ecb10d1f9`입니다. 로컬 devel fast-forward와 원 head 포함을 확인했습니다.
#7489는 자동 종료됐으며 [자동 종료 기록](https://github.com/edwardkim/rhwp/issues/7489#issuecomment-5945637979)을 확인했습니다.
운영 기록은 maintainer 직접 반영 대상 문서만 로컬 commit으로 보존하며 원격 push는 별도 승인 대기입니다.

[이슈 종료 workflow 36964919753](https://github.com/edwardkim/rhwp/actions/runs/36964919753)는 성공했습니다.
[duration workflow 36964919732](https://github.com/edwardkim/rhwp/actions/runs/36964919732)도 성공했으나
`ready:false, reason:no-verified-pr-duration-measurements`로 갱신을 보류했습니다.
Studio-only CI의 Rust worker skip으로 수집할 측정값이 없는 것이며 CI를 다시 실행하지 않았습니다.
운영 기록의 push 승인 뒤 이슈·기여자 안내를 게시하고 전용 branch 정리를 완료합니다.

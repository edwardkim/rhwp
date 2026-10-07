---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-03
---

# PR #7539 리뷰 — Studio Enter 뒤 새 쪽 캐럿·스크롤

## 최종 판정

**코드 검토에서 차단 결함 없음 — 검토 head `356f2915`의 필수 CI가 통과했고 MERGEABLE/CLEAN이다. 작업지시자가 병합을 승인했으며 새 문서 head의 필수 게이트 확인 뒤 진행한다.** 작성자 self-review이고 GitHub Approve는 제출하지 않았다. collaborator self-merge 경로에서는 self-review 기록을 사용하며 타인의 리뷰를 임의로 필수 조건에 추가하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7539](https://github.com/edwardkim/rhwp/pull/7539) / postmelee / devel |
| 시작 base | 선행 #7487 merge `83bf0f3c840c9afc1de584c17b167610fd9caf3b` |
| 검증 source | `3f66b618c0842221981eb3353a818b696315ee53` |
| 게시 candidate | `43fb71ef01609ec50ace7758f506898ad4836a08` — 같은 source에 증적·문서만 추가 |
| 이번 검토 원격 head | `356f2915e41244269c67bb899db55a4ad37788c4` — 검증 source와 제품 코드 동일 |
| 관련 이슈 | #7486 참조만 사용; 표 뒤 Enter 잔여 문제로 OPEN 유지 |
| 경로 | collaborator 본인 PR; reviewer 지정 없음; review·review_impl와 오늘할일을 동일 PR의 문서 후속 commit으로 포함 |
| 생성 시점 원격 참고값 | non-draft, MERGEABLE/BLOCKED, CI 실행 중. 병합 판단 때 재조회 필요 |
| 이번 검토 원격 참고값 | non-draft, MERGEABLE/CLEAN; CI·CodeQL·Render Diff·CI Impact Policy 성공 |

## 변경과 검토 범위

본문 Enter command의 비동기 layout 완료 뒤 기존 one-shot 캐럿 reveal을 예약한다. `splitParagraph`를 허용 목록에 추가하고 `executeOperation(command)`가 예약하도록 했다. 기존 Undo/Redo의 history type 예약도 이를 사용한다. `insertText` 및 셀·머리말·각주 Enter는 대상에서 제외했다.

실제 소비 경로를 다시 확인하면 본문 여러 줄 plain-text 붙여넣기도 `pastePlainText` → `SplitParagraphCommand` → 동일 command router를 사용한다. 따라서 예약은 Enter 키에만 한정되지 않는다. 이번 추가 검증에서 이 경로의 세 개 개행과 연속 Enter 열 번이 마지막 커서 위치를 정확하게 표시하는지 확인했다. rich/internal clipboard의 전체 동작을 검증한 것으로 확장하지 않는다.

실제 생산·소비 호출 경로, 원인과 반례는 [구현·검증 보고서](../../report/task_m100_7486_report.md)에 연결했다. 엔진 조판·문단 소유·쪽수·저장 줄 정보·baseline을 바꾸지 않았으며 #7487의 원 기여 코드를 다시 수정하지 않았다.

## 검증과 증적

- 루트 wrapper의 source head fresh dev WASM과 root pkg/public hash 일치를 확인했다. generated JS/WASM은 제출에서 제외했다.
- TypeScript·production build PASS; 전체 Studio/package 계약 1,816 PASS / 0 FAIL / 0 skipped.
- 정식 E2E 160/200/300% × 100% 한 쪽/66% 두 쪽 보기의 실제 Enter·Undo·Redo: 수정 전 15 FAIL, 수정 후 84 PASS / 0 FAIL. 최종 source에서 재실행했다.
- Ctrl+Enter 정상 대조군 6 PASS, 기존 편집 Undo 계약 PASS, E2E manifest 147개/147행 일치.
- 사용자 Chrome UI의 200%·33번째 Enter에서 추가 입력 없이 새 쪽 캐럿과 스크롤 이동을 확인했다.
- 이번 head self-review 추가 진단: 200% 줄간격 경계에서 연속 Enter 열 번 및 세 개 개행의 plain-text 붙여넣기 × 100% 한 쪽/66% 두 쪽 보기, fixture·논리 문단·owner·DOM·viewport 총 20 PASS / 0 FAIL. Enter는 실제 keyboard 입력, 붙여넣기는 `DataTransfer`와 `ClipboardEvent`로 실제 paste handler를 실행했다. 시스템 clipboard UI 시험과 구분한다.
- 입력은 E2E 코드 생성이며 파일로 소비한 HWP/HWPX/PDF가 없다. 원점 투영·viewport 계약을 검증했고 한컴 PDF의 caret 좌표와 일치한다고 주장하지 않았다.
- [검증 JSON](../../working/assets/issue7486-studio-caret/validation.json)에 source SHA, 전체 18개 전후 상태, WASM·대표 PNG 해시를 포함했다.

전후 PNG 4개는 직접 판독했고 PR 본문에 정확한 head의 실제 Markdown 이미지로 표시했다. 원점 잔차를 수치·화면으로 함께 설명했다. 세부 명령·로그 위치·한계는 보고서에 보존했다. DOM overlay 변경이므로 Native/WASM 문서 Visual Sweep과 90% silhouette gate는 비해당이다. Rust 전체 회귀·lint 및 release WASM은 Studio 단독 범위라 생략했다.

### 검토 head의 원격 CI와 추가 진단 기록

| 검사 | 실제 결과·증적 |
| --- | --- |
| [CI 37020644721](https://github.com/edwardkim/rhwp/actions/runs/37020644721) | SUCCESS. Studio unit 1,814 PASS / 0 FAIL / 2 SKIP; 두 skip은 CI에 `pkg-node`가 없는 기존 WASM 왕복 검사. 로컬의 1,816 PASS 기록과 구분한다. responsive toolbar 2,666 PASS / 0 FAIL |
| [CodeQL 37020644702](https://github.com/edwardkim/rhwp/actions/runs/37020644702) | SUCCESS |
| [Render Diff 37020644126](https://github.com/edwardkim/rhwp/actions/runs/37020644126) | SUCCESS. KTX·biz_plan·tac-case-001 세 canvas 비교 PASS; 이 검사를 DOM 캐럿 검증의 대체 근거로 사용하지 않음 |
| [CI Impact Policy 37022178944](https://github.com/edwardkim/rhwp/actions/runs/37022178944) | 정확한 head `356f2915`에 SUCCESS. 이전 37021733251은 `workflow-not-completed:CI:in_progress`로 PENDING을 게시했고 자동 완료 집계에서 해소됨. 수동 재실행 없음 |
| 추가 경계 진단 | `output/pr-review/studio-enter-caret-20261002/probe-caret-boundaries.mjs`, `logs/review-caret-boundaries.log`; 20 PASS / 0 FAIL, exit 0. 루트 fresh WASM 및 기존 source 검증 재사용; 제품 코드 변경 없음 |

추가 진단 명령은 `rhwp-studio/`에서 `CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/run-with-vite.mjs -- node ../output/pr-review/studio-enter-caret-20261002/probe-caret-boundaries.mjs --mode=headless`다. 실행 파일 경로만 절대 경로로 지정해 같은 명령을 실행했다. 기존 7700 서버를 보존하고 임시 7701 서버를 종료했다. 진단·원격 로그는 ignored output에 보존하며 정식 E2E source 및 대표 이미지는 기존 PR에 포함되어 있다.

추가 진단의 최종 상태는 아래와 같다. 모두 논리 `charOffset=0`, 엔진·VirtualScroll 2쪽, page owner 1이며 DOM 좌표와 완료된 쪽 원점의 투영이 일치하고 viewport 내부에 표시됐다. 로그·임시 진단 파일은 후속 정리 대상이므로 이 관측 요약을 영구 기록으로 남긴다.

| 입력 / 보기 | 최종 문단 | DOM top / left (px) | scrollTop (px) |
| --- | --- | --- | --- |
| Enter 10회 / 100% 한 쪽 | 42 | 1514.8 / 133.55 | 810 |
| 개행 3개 paste / 100% 한 쪽 | 35 | 1328.1 / 133.55 | 623 |
| Enter 10회 / 66% 두 쪽 | 42 | 252.318 / 708.144 | 0 |
| 개행 3개 paste / 66% 두 쪽 | 35 | 129.096 / 708.144 | 0 |

## 공통 원칙 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 완료된 쪽 원점에 표시하는 UI 계약; 특정 spacing/문서 조건·clamp 없음 |
| 측정·배치 일관성 | 비해당 | 엔진 측정·배치 미변경; UI 투영은 완료된 VirtualScroll과 page-local rect를 사용 |
| 분할·이어받기 계약 | 비해당 | SplitParagraph의 내용·컷·flush를 바꾸지 않고 화면 표시 완료 예약만 추가 |
| 줄 소속과 점유 높이 | 비해당 | LineSeg·높이·조판 함수 미변경 |
| 사례와 증거의 독립성 | 충족 | 합성 fixture setup과 실제 키 입력을 분리; source baseline FAIL/수정 PASS; 논리 커서·완료 원점·viewport 검사 |
| 기준값 변경 | 비해당 | baseline/golden/허용치 변경 없음 |
| 주장과 검증 범위 | 충족 | 6개 조합·18개 Enter/Undo/Redo 상태, 연속 Enter·plain-text paste 4개 추가 경계, 제외 경로 unit, 사용자 Chrome 확인; IME/iOS·HF 실사용·rich clipboard 전체는 미검증으로 구분 |

## 병합 후 계획

2026-10-03 작업지시자는 리뷰 문서 push → 새 head CI 확인 → 일반 병합 → 이미지 comment·후속 정리 순서의 실행을 승인했다. 최신 `upstream/devel`과의 병합 시뮬레이션, 문서 링크·공백·기존 오늘할일 보존을 확인하고 문서-only single-parent commit을 push한다. 새 head의 필수 검사·실제 fast-pass·mergeability를 확인한 뒤 해당 head를 고정해 일반 병합한다. GitHub self-Approve는 수행하지 않는다. 이 기록 작성 시점에는 새 문서 head push·병합이 아직 완료되지 않았다.

### Merge 후 contributor PR comment 계획

- PR #7539의 실제 merge SHA와 최종 문서 head를 명시한다. 선행 #7487의 엔진 보정과 이번 Studio UI 보정을 구분한다.
- Enter 뒤 새 쪽 DOM 캐럿·viewport 갱신과 Undo/Redo, 84개 정식 E2E 및 20개 추가 진단의 통과를 설명한다. 최종 head에서 재사용된 candidate CI·preflight·aggregate·정책 결과를 실제 URL로 연결한다.
- merge된 `mydocs/working/assets/issue7486-studio-caret/`의 `before-zoom100.png`, `after-zoom100.png`, `before-zoom66.png`, `after-zoom66.png`를 merge SHA 고정 raw URL의 실제 Markdown 이미지 표로 표시한다. 브라우저 UI 캡처이며 문서 Native/WASM Visual Sweep이나 한컴 caret 좌표 비교로 보고하지 않는다.
- #7486의 표 뒤 Enter 잔여 문제로 OPEN을 유지하고 IME/iOS·HF 실사용·rich clipboard 전체의 미검증 범위를 적는다. 이슈에도 같은 merge와 해결 범위의 후속 안내를 남긴다.
- review·report·asset·오늘할일이 merge에 포함됐는지와 자동 duration 갱신의 성공 또는 자료 부족 보류를 확인한다. 추가 운영 문서 commit은 만들지 않고 확정값은 comment에 기록한다. 이후 이번 작업의 임시 output·소유 branch를 안전 조건에 따라 정리한다.

# PR #9136 셀프 리뷰 — 공통 대화상자 화면 경계와 조작 보존

## 최종 판정

**승인 — 공통 대화창 보완의 로컬 검증과 동일 code head CI를 완료했다.** 사용자에게 push·PR·CI·self-review·merge와 후속 처리 승인을 받았다. 문서-only trailing head의 실제 fast-pass·required check와 최신 base/head 병합 검증을 완료한 뒤 정상 merge한다.

## 접수 정보와 경로

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#9136](https://github.com/edwardkim/rhwp/pull/9136) / `edwardkim` / `devel` |
| code candidate | `15837c689f1a84eaba4661794dd7d8d70bc3f94a` |
| 로컬 검증·WASM 생산 | `b57fb3d5ef8ea924e5cd37fed3798ed1b18e54e7`; candidate까지 추가 변경은 검증 문서뿐 |
| 검증 base | `ae9277dda0ca17be279cba89c4deea74fb75dcae` |
| 현재 관측 | 2026-11-16 19:11 KST, OPEN·non-Draft·mergeable true/CLEAN·reviewer 없음 |
| reviewer | 작성자 self-review; 본인 PR에 reviewer를 지정하지 않음 |
| 관련 이슈 | [#9133](https://github.com/edwardkim/rhwp/issues/9133) 해결, [#8905 추가 질문](https://github.com/edwardkim/rhwp/issues/8905#issuecomment-6897670979) 후속 |

기본 경로는 `collaborator_self_merge`, 보조는 `intake_and_review`·`local_validation`·`review_only_fast_pass`·`post_merge`다. 모 workflow·선택표·기본/보조 절차와 review template·GitHub 운영 매뉴얼을 읽었다. CI 실행 정의 변경이 없어 일반 문서 trailing 경로이며 옵션 2 대상이 아니다.

## 변경과 선행 해결 추적

[#8905](https://github.com/edwardkim/rhwp/issues/8905) → [원 PR #9092](https://github.com/edwardkim/rhwp/pull/9092) → merge `4c4bd16a7a9bc8b00738e6c3cd74bca04ab3345c`의 제품 정보 전용 개선을 보존했다. 복구 창의 [#3250](https://github.com/edwardkim/rhwp/issues/3250)·[PR #3255](https://github.com/edwardkim/rhwp/pull/3255)·merge `4061ddfe`도 원 계약과 대조했다. [결과 보고](../../report/task_m100_9133_report.md)에 원인·기존 범위와 보완을 연결했다.

공통 wrap을 viewport 안의 flex로 제한하고 제목/footer를 고정했다. 중간 flex 소유를 `cs-main-row → cs-left-col → dialog-body`까지 전달하고 본문·탭·후보 목록을 스크롤한다. 실제 DOM의 배치를 측정하며 문서별 좌표나 JS resize 보정을 추가하지 않았다. 복구 목록이 0px로 축소되는 경계는 후보 최소 공간과 본문 스크롤로 보완했다.

기존 복구 소스 검사에서 잘못된 `overflow:hidden`·`min-height:0` 고정 두 곳만 실제 브라우저 회귀로 대체했고 DOM hook·viewport·긴 이름 보호는 유지했다. 기존 자동 복구 E2E가 #8397의 글꼴 선택에서 멈추는 현상은 base에서도 재현했다. 실제 ‘대체 글꼴로 보기’를 선택하도록 셋업을 보완했으며 복구·dirty·원 draft 삭제 assertion은 유지했다.

## 로컬 검증·입력과 리베이스 보존

초기 승인 후보 `e27e724802` 뒤 devel에 #9134·#9135가 추가돼 최신 base로 한 번 rebase했다. `git range-diff`의 네 patch가 모두 같고 CSS 3개·관련 E2E 2개는 바이트가 같다. [새 기반 원장](../../working/assets/issue9133/rebase/README.md)과 [source·입력·의존성](../../working/assets/issue9133/rebase/source-and-inputs.json)을 확인했다.

| 실제 검사 | 결과 |
| --- | --- |
| TypeScript 일반·CI unit 설정 | 각각 exit 0 |
| Studio 전체 | 2,805 PASS·기존 2 SKIP·실패 0 |
| production/PWA build | 성공 |
| 공통 창 11종×6개 viewport | 66개 조합 PASS; base CSS의 67개 실패 조건은 그대로 재현 |
| 확인/취소·resize | 실제 F7 확인·Undo/Redo·취소 Enter·입력 보존·배경/Escape·CDP 터치 PASS |
| 제품 정보·PDF·복구 | 기존 19개·실제 PDF Worker/취소/재시도·새 문서/HWP/HWPX 복구 PASS |
| 반응형 | 2,666 PASS·실패 0 |
| #9134 기능 유지 | 실제 CanvasKit 빈 일반 선/사각형 패턴과 명시적 패턴 거부·브라우저 종료/파이프 소유 검사 PASS; 두 제품 파일은 새 base와 동일 |
| 프로그램 신원 | 새 Rust 의존성의 release+wasm-opt 완료, `pkg`/Studio public JS·WASM 일치 |

최적화 WASM SHA-256은 `44eaf435692537c84b1e13d7fc34017953318961171533d6a970e07badd33ef6`이다. 실제 Chromium 152.0.7946.0 / build 1660786을 사용했다. Android Firefox 실기기 검증은 포함하지 않았다.

**검증 입력 커밋 확인: 충족.** 기존 `samples/셀보호2.hwp`, `samples/셀보호2.hwpx`, `pdf/issue8403/image-patterns.pdf`는 실행 파일·Git blob·SHA-256·검토 candidate의 내용이 같다. [commit별 해시](../assets/issue9133/code-ci.json)에 기록했다. 새 문서·표/그림·복구 metadata는 테스트 내부에서 생성한 UI 계약 입력이며 한컴 Print 자료로 주장하지 않는다.

## GitHub CI와 게시 확인

같은 PR·branch·source repository의 code candidate에 대해 아래 실행 모두 completed/success를 확인했다. [실행·check·status 원본 값](../assets/issue9133/code-ci.json)을 보존했다.

- [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/43841116498)
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43841116312)
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43841116109)
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/43841116145)
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/43841116172)
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/43841116286), 최종 status success

trusted-base classifier v9가 `frontend_mode=package`, `frontend_canvas_required=true`, `render_required=true`, `rust_required=false`, `native_skia_required=false`, CodeQL `javascript-typescript`로 판정했다. Rust archive·Native·비대상 언어 skip을 실제 실행 성공으로 세지 않았다. 전체 workflow 성공과 코드 후보의 38개 check 완료·실패/진행 중 0을 반복 확인한 뒤 이 기록을 작성했다.

PR 본문·제목이 게시 payload와 같고 BOM이 없음을 확인했다. 전후 PNG의 HTTP 200·MIME·SHA-256을 대조했으며 실제 GitHub 페이지에서도 두 800×360 이미지가 로드됐다. [게시 영수증](../assets/issue9133/publication.json)을 참조한다.

## 공통 원칙과 직접 판독

| 항목 | 판정·근거 |
| --- | --- |
| 선행 해결 분석·공통 처리 | 충족. #9092·#3255·#8397의 통합/검증과 현재 경로를 추적했고 공통 CSS 소비를 보완 |
| 실제 배치·독립 기대 | 충족. viewport 포함과 본문 끝/실제 클릭으로 판정; base RED·정상 데스크톱의 크기/위치 동일 확인 |
| rebase 기능 보존 | 충족. 새 의존성·CanvasKit/브라우저 종료 경계를 포함한 실제 재검증 |
| 문서 조판·LineSeg·분할/이어받기·Native/SVG 소비 | 비해당. 문서 엔진·배치·paint 또는 형식 API를 변경하지 않음 |
| document px/golden/보호값·편람 | 비해당. UI 관계 회귀이며 문서 baseline·허용치·정본 TSV 변경 없음 |
| 새 회귀·기존 검사 정리 | 충족. 실제 UI 66개 조합과 기존 전체 검사 유지; 잘못된 소스 모양 핀만 동작 검사로 대체 |

최신 기반의 편집 용지 전후·복구 목록 화면을 직접 판독했다. 원 화면은 제목/버튼이 잘리고, 후보는 제목과 조작이 화면 안에 있으며 마지막 복구 항목도 읽고 선택할 수 있다. [전쪽수 대신 UI 전체 조합 측정](../../working/assets/issue9133/rebase/after-measurements.json)을 문서 Visual Sweep 수치와 구분했다.

## 승인된 merge·후속 처리

현재 남은 조건은 문서-only trailing head의 실제 fast-pass·required checks와 최신 base/head 병합 검증이다. 일반 `devel` 보호 조건을 만족한 정확한 head로 merge하며 admin 예외를 쓰지 않는다.

merge 후에는 duration 갱신 성공 또는 증거 부족 보류를 확인하고 운영 기록·devel sync를 마친다. #9133 완료 댓글·close, #8905의 기존 질문 답변, PR 결과 댓글과 전용 branch/worktree·root output 전체·사용 종료한 debug 정리를 사용자 승인 범위에서 수행한다. 병합 후 검증 CI는 시작하지 않는다.

UI 댓글에는 실제 merge SHA로 고정한 `mydocs/working/assets/issue9133/rebase/after-page-800x360.png`와 결과 보고·이 리뷰를 연결한다. raw PNG 응답·해시를 검증하고 `--body-file`/구조화 API로 게시한 뒤 본문·이미지를 재조회한다. 문서 Visual Sweep의 지표나 Android Firefox 실기기 결과로 확대하지 않는다. 최종 원격 상태는 [PR #9136](https://github.com/edwardkim/rhwp/pull/9136)과 [이슈 #9133](https://github.com/edwardkim/rhwp/issues/9133)에서 확인한다.

# #9077 Canvas 예약 별칭과 실제 글꼴의 표시 정책

## 범위와 선행 근거

2026-11-15 사용자 지시로 `upstream/devel`을 동기화하고
`task/9077-canvas-reserved-font-alias`에서 수정한다. 기준은
`2d83fa4c8a5ed3513afcde26367fb5ffebeb294c`다.
[이슈 #9077](https://github.com/edwardkim/rhwp/issues/9077)의 재현 2는 문서 없이
Canvas의 `font` 설정만으로 실행된다. 연결된 해결 PR은 조회 당시 없었다.

기존 #6600 보정 `af0fe858c69db244fb44064a3646ae2f6b99163b`는 실제 setter를
통과시켜 설치 별칭의 보존과 괄호 전진폭을 확인했다. #8316의 메인터너 보정
`f70a8c937d3dd56ac942d019dfca40c851e0a6a3`에서 원본 TTF용 예약 별칭을 추가했다.
그 구현의 계약은 별칭이 미공급이어도 뒤의 원래 CSS 사다리를 사용하는 것이다.
현재 Studio는 별칭을 실제 문서 이름으로 해석하므로 이 계약을 위반한다.

## 생산과 소비 경로

`TextStyle.source_font_is_ttf` → `CanvasTextFont::family_chain`의 예약 별칭과 원래
후보 체인 → `wasm-bridge.ts`의 `canvasFontResolution` →
`substituteCssFontFamily` → 원시 `CanvasRenderingContext2D.font` setter →
`measureText` 및 `fillText`다. 보충 메트릭도 같은 setter/해소 함수를 소비한다.
`scopedHostCanvasFont`가 해소한 호스트 공급 경로는 그 결과를 먼저 사용한다.

CSS 해소 함수를 기존 `font-substitution.ts`로 옮겨 제품 함수 자체를 단위 검사한다.
`__rhwp_ttf__`와 `__rhwp_intrinsic__` 별칭은 순서대로 보존하고, 첫 실제 이름으로
문서 대체와 로컬 승인 정책을 계산한다. 일반 엔진 별칭과 기존 원본 제외 정책은 유지한다.
기존 음성 대조의 주입 대상도 이동한 제품 함수로 변경하며 결함 검출 assertion은 유지한다.

## 진단과 검증 계약

| 축 | 확인 범위 |
| --- | --- |
| 환경 | Windows 네이티브 PowerShell, Node 24.19.0, headless Chrome, 명시 Canvas2D |
| 원문 | 이슈 재현 2의 CSS descriptor; 문서별 저장 LineSeg·언어 슬롯은 입력이 아님 |
| 열거 | 등록 웹 글꼴 이름과 실제 로드한 로컬 FontFace를 구분하고 호스트 공급 검사를 유지 |
| probe | 원시 Canvas setter로 동일 프로그램을 직접 선택한 측정·paint 대조군 |
| 선택 | 미공급 TTF/intrinsic 별칭 및 둘의 조합; 공급된 별칭의 원 프로그램 우선 선택 |
| backend | 실제 Studio 전역 setter의 CSS 순서·전진폭·전체 Canvas 픽셀 |
| oracle | MIT 합성 RHWPHostFixture의 독립 Regular/Italic TTF를 사용한 공급 계약 대조 |

테스트의 합성 윤곽선은 실제 함초롬바탕·궁서·Arial의 외형 정답지가 아니다.
한컴 Print 일치율이나 페이지 좌표 golden을 추가하지 않는다. Native/fresh WASM의 독립
Print exporter는 Studio의 이 setter를 소비하지 않으므로 그 TSV를 이 결함의 검출 근거로
대신하지 않는다. 엔진의 줄 구성·pagination과 portable 출력 코드는 변경 대상이 아니다.

## 검증 기록

수정 전 CSS 관계 검사 3개가 실제로 실패했다. 원래 4개는 통과했다.
수정 후 추가한 로컬 공급 검사까지 8/8 통과했다. 기존 글꼴 정책·호스트 공급·해소 scope와
음성 대조도 집중 실행했다. 최종 실행은 아래와 같다.

| 명령 | 결과 |
| --- | --- |
| `scripts/wasm-pack-locked.ps1 --target web --out-dir pkg --mode no-install` | release 컴파일·wasm-opt 포함 성공, 전체 35분 53초 |
| `npx.cmd tsc --noEmit` (Studio) | 성공 |
| `npm.cmd --prefix rhwp-studio test` | 2,800건 중 2,798 PASS, 0 FAIL, 기존 선택 검사 2 SKIP |
| `npm.cmd run build` (Studio) | TypeScript·Vite·PWA production build 성공 |
| `npm.cmd run e2e:issue-9077` (Studio) | headless Chrome 157, 명시 Canvas2D, 12/12 조합 PASS |
| `venv/Scripts/python.exe scripts/check_e2e_manifest.py` | tracked 191개 / manifest 191개, 이상 없음 |
| `git diff --check` | 통과 |

검증한 source는 `d255fc603f5fca7c5e8cefa7bff0cecd264a3b6e`다.
WASM 빌드 시작 시점은 `92a3778ea5`이며 이후 변경은 테스트·기록과 TypeScript 공백 정리다.
`src`, `crates`, `vendor`, `Cargo.toml`, `Cargo.lock`, `build.rs`의 diff가 비어 있음을 확인했다.
Rust 1.93.1, wasm-pack 0.15.0, Cargo.lock과 일치하는 wasm-bindgen-cli 0.2.127을 사용했다.
WASM은 27,649,950바이트이며 SHA-256은
`16e3f571c57b82d666ec87967e2d0144db5c41d7f5bef04cbd8a30986da38f54`다.
초기화 시 브라우저가 실제 받은 응답의 SHA-256도 같으며 HTTP 200, `matched: true`다.

웹 등록 이름과 별도의 로컬 공급 이름 각각에 대해 별칭 없음, TTF 미공급, intrinsic 미공급,
두 별칭 미공급, TTF 공급, intrinsic 공급을 검사했다. 미공급 8건은 원시 setter로 실제 face를
고른 대조군과 Canvas 전체 픽셀·측정 폭이 동일하고 시스템 fallback의 그림과 다르다.
공급된 4건은 별도 Italic 프로그램을 직접 고른 대조군과 동일하며 일반 프로그램과 다르다.
실제 screenshot에서도 테스트 프로그램의 의도된 기울어진 합성 윤곽을 확인했다.

건너뛴 2건은 `pkg-node` 전용 기존 편집 검사다. 이 작업은 web WASM과 Canvas2D CSS 경로를
검증하며 Node 전용 package를 만들거나 skip 조건을 추가하지 않았다. 런타임은 fresh JS/WASM을
유지하고, 동기화 과정에서 변경된 tracked 생성 타입 선언 2개는 검증 후 복원했다.

중간 로그는 Git 제외 경로 `output/issue9077/`에 보관한다. 기존 전체 검사에서
Windows CRLF를 LF로 읽어야 하는 `chrome-mode` source 검사가 실패해 해당 working tree의
줄바꿈만 정규화했다. Git blob은 변경 전과 같은
`f3051e6f6d8c80591a6ddaf709e1343daff9afb3`이며 13/13이 통과했다.
동기화 전 WASM에서 `insertMemo`를 찾지 못한 검사도 새 빌드에서 통과했다.
최종 로그는 `studio-test-final.log`, `studio-build.log`, `e2e.log`, `wasm-build.json`이며
실제 초기화 응답 해시는 `output/issue9077/consumption/`에 있다. 대표 screenshot과 HTML은
각각 `rhwp-studio/e2e/screenshots/issue9077-canvas-font-alias.png`,
`output/e2e/canvas-font-alias-issue9077-report.html`이다.

## PR 제출 준비

2026-11-15 사용자가 PR 생성을 승인하고 frontend 검증만 적용하도록 명시했다.
완료된 타입 검사·전체 단위 테스트·production build·Chrome Canvas 계약 검증을 재사용한다.
최종 구현의 frontend tree는 검증한 `d255fc603f`와 동일하다.

base route: `collaborator_self_merge.md`.
modifiers: `local_validation.md`의 frontend 범위.
loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_self_merge.md`, `local_validation.md`, `intake_and_review.md`,
`visual_fixture_evidence.md`, `codex/docs_and_git_workflow.md`.

인증 계정은 `jangster77`, 저장소 권한은 `write`다. 본인 열린 PR은 모든 base·Draft를 포함해
0건이었고 리뷰어 지정 예외 및 기존 PR 사이의 병합 시뮬레이션 대상도 0건이다.
생성 직전에 목록과 원격 SHA를 다시 확인한다.

최신 base `2d83fa4c8a5ed3513afcde26367fb5ffebeb294c`와 후보
`61dd0e81bd9c99144dccb9247f44d29a48971472`의
`git merge-tree --write-tree <base> <candidate>`는 exit 0이며 결과 tree는
`e22211b135c31e9d893b45fc79a1f6ef460267f3`다. 추가 변경은 이 제출 기록뿐이며,
원격 push 전 최종 후보 SHA로 시뮬레이션을 다시 실행한다.

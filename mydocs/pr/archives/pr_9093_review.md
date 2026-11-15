# PR #9093 리뷰 — Canvas 예약 별칭 뒤의 실제 글꼴 선택

## 최종 판정

**승인 — #9077의 frontend 글꼴 선택 계약 수정과 검증을 완료했다.**
검증한 code head는 `f68c4d9bd7d3f037f43e1820e49783af6b979479`이며 해당 head의
CI·CodeQL·Render Diff·Adapter·Proptest와 최종 정책 검사가 모두 성공했다.
이 기록을 추가한 문서-only trailing head의 검사·mergeability 확인과 병합은 남은 조건이다.

## 접수 정보와 범위

- [PR #9093](https://github.com/edwardkim/rhwp/pull/9093), 작성자 `jangster77`, base `devel`.
- [Issue #9077](https://github.com/edwardkim/rhwp/issues/9077)를 해결한다.
- 기준 SHA: `2d83fa4c8a5ed3513afcde26367fb5ffebeb294c`. 작업 중 rebase·base merge는 없었다.
- collaborator self-merge 경로다. 리뷰어 요청은 없으며 작업지시자가 CI 모니터링과 전체 후속 처리를 승인했다.
- 사용자가 frontend 검증만 명시했다. Rust 조판·Native 출력·페이지 golden 변경은 없다.
  문서 없이 재현되는 CSS/FontFace 공급 계약을 검사하며 HWP/PDF 전쪽 TSV 검증으로 확대하지 않았다.

## 변경과 원인 검토

엔진의 `__rhwp_ttf__`·`__rhwp_intrinsic__` 예약 별칭을 실제 문서 글꼴로 판정해
Batang·Malgun Gothic이 앞에 들어가던 문제다. 기존 공통 CSS 해소 함수를
`font-substitution.ts`로 이동하고, 예약 별칭과 실제 family 목록을 나누었다.
첫 실제 family로 기존 표시·문서 대체·로컬 승인 정책을 판정하며 별칭의 선두 순서는 보존한다.

생산·소비 경로는 `CanvasTextFont::family_chain` → `canvasFontResolution` →
`substituteCssFontFamily` → 원시 Canvas font setter → `measureText`·`fillText`다.
호스트 scope가 이미 해소한 face의 우선권과 미승인 원본 제외 정책을 유지한다.
문서 ID별 분기나 픽셀 보정은 없다. 이동한 함수에 맞춰 기존 음성 대조의 주입 위치도 옮겼다.

선행 #6600·#8316의 원 PR/보정 커밋과 수정 전후 결과는
[작업 기록](../../working/task_m100_9077_stage1.md)에 연결했다.
예약 별칭 미공급, 공급, 문서 대체, 로컬 승인, 기존 일반 alias를 함께 검토했다.

## 로컬 frontend 검증

Windows 네이티브 PowerShell에서 수행했다. 최종 code head의 frontend tree는
검증 source `d255fc603f5fca7c5e8cefa7bff0cecd264a3b6e`와 동일하다.

| 검증 | 실제 결과 |
| --- | --- |
| CSS 후보·정책 계약 | 8 PASS. 신규 관계 검사 3건의 수정 전 FAIL / 수정 후 PASS 확인 |
| 런타임·음성 대조 | 7 PASS. 이동한 제품 함수가 음성 대조의 실제 주입 대상 |
| Studio 전체 `npm test` | 2,798 PASS / 0 FAIL / 기존 선택 검사 2 SKIP |
| `tsc --noEmit`·production build | 모두 성공 |
| `e2e:issue-9077` | Chrome 157 Canvas2D, 12/12 PASS |
| E2E manifest·공백 | tracked/manifest 191개 일치, `git diff --check` 통과 |

fresh release WASM은 wasm-opt까지 완료했고 실제 브라우저 초기화 응답의 해시가 빌드와 같았다.
로컬 WASM은 27,649,950바이트, SHA-256
`16e3f571c57b82d666ec87967e2d0144db5c41d7f5bef04cbd8a30986da38f54`다.
Rust 입력은 빌드 시작 source 이후 바뀌지 않았다. WASM 재생성 과정의 tracked 생성 타입 변경은 복원했다.

MIT 합성 Regular/Italic TTF를 명시 공급했다. 웹 등록 family와 로컬 공급 family 각각의
미공급 alias 8건은 원시 setter 대조군과 폭·Canvas 전체 픽셀이 같고 시스템 fallback과 달랐다.
공급 alias 4건은 선두 alias의 별도 Italic 프로그램을 선택했다.
[12개 실제 결과](../assets/issue_9077/canvas-contract-12-cases.json)를 보존했다.
이는 함초롬바탕·궁서의 실제 외형이나 한컴 Print 일치율에 대한 주장이 아니다.

## 정확한 code head의 GitHub Actions

모든 run은 같은 PR·branch·code SHA의 attempt 1에서 완료됐다.
[CI 원장](../assets/issue_9077/code-ci-f68c4d9bd7.json)에 job·step·최종 check 결과를 보존했다.

| Workflow | 결과와 실행 근거 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/43713738530) | 성공. Frontend package gates와 Build & Test 성공. Studio 2,798 PASS·0 FAIL·2 SKIP, Canvas 바인딩 23 PASS, 반응형 2,666 PASS, production/extension build 성공 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43713738307) | 성공. 선택한 JavaScript/TypeScript 분석 실행, 최종 GHAS check neutral |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43713738324) | 성공. 실제 Canvas visual diff·PDF report·선택 CanvasKit readiness 실행 |
| [Adapter](https://github.com/edwardkim/rhwp/actions/runs/43713738346) | 실제 worker 성공 |
| [Proptest](https://github.com/edwardkim/rhwp/actions/runs/43713738309) | 실제 worker 성공 |

Render Diff의 기본 1쪽 비교는 KTX 96/891,662px 차이(0.01077%), biz_plan·tac-case-001 각각
0px 차이로 모두 PASS다. 독립 한컴 Print 전쪽 Visual Sweep의 raw 지표와 구분한다.
CI 분류상 `render_required=true`인 frontend Canvas 경로이므로 Render Diff 결과까지 기다렸다.
로컬 Native 전체 검사나 TSV를 추가 수행한 것으로 기록하지 않는다. CI 실행 정의는 수정하지 않았다.
코드 head의 최종 `CI Impact Policy`는 success, 조회 시 `MERGEABLE`·`CLEAN`이었다.

## 문서 trailing과 후속 처리

이후 변경은 이 리뷰·오늘할일·검증 원장뿐이다. push 전에 최신 base와 trailing head의
merge simulation, merge tree의 공백·Markdown 링크·기존 오늘할일 보존을 확인한다.
push 뒤에는 실제 검증 재사용 판정과 최신 head의 required check를 별도로 확인한다.

## Merge 후 contributor PR comment 계획

최종 merge SHA가 `upstream/devel`에 포함되고 리뷰·오늘할일·원장이 존재하는지 확인한다.
병합 후 duration workflow의 결과 또는 측정 부족에 따른 보류 사유를 확인하며 검증 CI를 재실행하지 않는다.
이슈 종료 상태를 재조회하고 필요하면 completed로 닫는다. 이슈와 PR에 한국어 존댓말로
merge commit, 이 리뷰, code CI run, frontend 2,798 PASS·E2E 12 PASS와 검증 범위를 게시한다.
한컴 Print 시각 증적을 주장하거나 다른 문서의 이미지를 첨부하지 않는다.
UTF-8 BOM 없는 `--body-file`로 게시한 뒤 API 본문 일치와 한글 보존을 확인한다.
소유 임시 branch·산출물을 정리하고 기존 다른 작업의 `output/os-font-review`는 보존한다.

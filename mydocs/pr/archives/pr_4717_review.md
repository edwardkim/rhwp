---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4717 검토 - Studio 차트 숫자 데이터 편집 UI

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4717](https://github.com/edwardkim/rhwp/pull/4717) |
| 작성자 / source | @johndoekim / `task4694` |
| 기여 이력 | 기존 기여자. 차트/OLE·표 셀 경로의 선행 변경 이력이 있다. |
| 원 code candidate | `8bf10fa2dcef252a80c3686ffded05b373d4abff` |
| 대상 브랜치 | `devel` |
| 작성 시점 참고 상태 | `MERGEABLE`, `CLEAN` |
| 변경 규모 | 25개 파일, +2,133/-5, 8개 commit |
| 관련 이슈 | [#4694](https://github.com/edwardkim/rhwp/issues/4694), PR 본문의 `Closes #4694` |
| reviewer | @jangster77 지정 완료 |

PR은 차트 숫자 편집 엔진(#4100, PR #4603)을 Studio까지 연결한다. WASM의 차트 열거·읽기·쓰기
표면 5종을 bridge로 노출하고, 선택한 차트를 문서 순번으로 대조한 뒤 차트 데이터 대화상자,
컨텍스트 메뉴, 더블클릭으로 편집한다. 저장은 코어의 dry-run 검증 뒤 snapshot 경로를 사용하므로
실변경만 undo 기록으로 남긴다.

## 검토와 메인터너 보정 판단

표 셀·글상자 안 OLE 차트는 본문 직속 3좌표만으로 식별하면 같은 루트 문단의 다른 차트로
오매칭될 수 있다. PR은 RawSvg/placeholder OLE 레이아웃 노드에 `cellPath`를 실어 Studio의
`matchChartRef`가 컨테이너 주소까지 대조하도록 보완한다. 컨테이너 정보가 없는 모호한 3좌표는
열지 않는 안전 축소로 처리한다.

`table_cell_content`가 전달하는 내부 OLE control index와 최상위 표 경로, 차트 열거기의
`tableCell` container 주소, Studio 선택 ref의 `cellPath`가 같은 좌표 계약을 사용함을 추적했다.
코드·테스트·대화상자 배선에서 추가 결함이나 누락된 메인터너 보정은 확인되지 않았다.

머리말·꼬리말과 각주/미주 OLE 편집은 #4694의 본문 직속·표 셀 수용 범위 밖이다. 해당 문맥이
모호하면 다른 차트를 열지 않고 메뉴 노출을 거부한다.

## 완료한 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| 표 셀 OLE 레이아웃·차트 계약 | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --test issue_4694_chart_list --test-threads 12 --no-fail-fast` | 5/5 통과. 실제 1x1 표 구조에서 `cellPath` 방출, 열거 주소와 snapshot 원복을 확인했다. |
| Studio 단위 회귀 | `npm --prefix rhwp-studio test` | 918/918 통과. 주소 대조, 입력 검증, no-op, snapshot 라우팅을 포함한다. |
| TypeScript | `npx tsc --noEmit` | 통과. |
| WASM 표면 | `wasm-pack build --target web --out-dir pkg` | 통과. 차트 API가 실제 web 패키지로 생성됐다. |
| Studio 실동작 | `VITE_URL=http://127.0.0.1:5173 node rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs --mode=headless` | 통과. 메뉴 노출, 더블클릭 진입, `4.3 → 91.7` 저장, Ctrl+Z 원복, 무편집 무흔적, 비차트 OLE 미개방을 확인했다. |
| 최신 devel 정합 | `git merge-tree --write-tree upstream/devel HEAD`, `git diff --check upstream/devel...HEAD` | 충돌 없이 merge tree `cb350c4ad1b942a923bc9b339efb840f04eb205e` 생성, 공백 오류 없음. |
| GitHub code candidate | `8bf10fa2d`의 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/31690819480/job/94421635286), [Native Skia](https://github.com/edwardkim/rhwp/actions/runs/31690819480/job/94418829587), [Lint](https://github.com/edwardkim/rhwp/actions/runs/31690819480/job/94417849819), CodeQL, [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/31690819305/job/94417534973) | 모두 성공. |

이번 renderer 변경은 OLE 선택 메타데이터 방출만 추가하며 PDF/SVG paint geometry를 바꾸지 않는다.
따라서 한컴 PDF sweep은 적용하지 않았고, 실제 Studio Canvas에서 차트 선택·편집을 확인하는
headless e2e와 캡처 검토를 시각 증적으로 사용했다.

## 판정

**통합 수용 권고.** contributor code candidate 뒤에는 이 archive review와 오늘할일만 single-parent
trailing commit으로 추가한다. 최신 기록 head의 review-only fast-pass aggregate, CodeQL, mergeability를
다시 확인하고 작업지시자 승인에 따라 merge한다. merge 뒤 #4694 close 상태와 원 PR 후속 처리는
`post_merge.md` 절차를 따른다.

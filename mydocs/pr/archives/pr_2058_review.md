# PR #2058 리뷰 — Through 배치 무변경 확인 보존

- PR: #2058 `Task #2054: 배치 버튼 미선택 시 개체의 원래 textWrap 보존`
- URL: https://github.com/edwardkim/rhwp/pull/2058
- 기준 브랜치: `devel`
- head branch: `fix/wrap-through-preserve`
- 작성자: @lpaiu-cs
- 관련 이슈: #2054
- 문서 작성 시점 참고값: 원 PR head `b67a31813a86561afacbe3eaaf95051493e4a65f`, merge state `BEHIND`
- 처리 경로: 여러 PR 체리픽 누적 검토. 통합 PR #2062 에 기능 커밋을 포함하고, 본 review 문서를 같은 PR head 에 포함한다.
- 최종 merge 조건: 통합 PR #2062 최신 head 기준 GitHub Actions 통과 + 작업지시자 승인.

## 결론

merge 후보로 본다. `PicturePropsDialog` 의 배치 버튼 목록에 `Through` 가 없어 모든 배치 버튼이 비활성인 상태에서
`getSelectedWrap()` 이 기본값 `Square` 를 반환하던 문제를, 기존 `props.textWrap` 보존으로 바꾼다. 사용자가
아무것도 바꾸지 않고 확인만 눌렀을 때 문서 배치 속성이 조용히 손상되는 문제를 막는 좁은 수정이다.

원 PR 은 `BEHIND` 상태라 #2057, #2059 와 함께 `upstream/devel` 기준 체리픽 통합 PR #2062 로 처리한다.

## 변경 범위

- `rhwp-studio/src/ui/picture-props-dialog.ts`
- `rhwp-studio/tests/wrap-through-preserve.test.ts`

## 체리픽 기록

| 항목 | 값 |
|------|----|
| 원 커밋 | `b67a31813a86561afacbe3eaaf95051493e4a65f` |
| 통합 PR 커밋 | `24bbac43b36cbb67f2a2d2a039434c3e55e017fa` |
| 체리픽 순서 | 2 / 3 |
| 충돌 | 없음 |
| 선행 PR 의존 | 없음. #2057/#2059 와 같은 통합 PR 에 포함 |

## 검증

| 검증 | 결과 |
|------|------|
| 원 PR GitHub Actions | CodeQL/CI/Render Diff 계열 통과. WASM Build 는 원 PR CI 에서 skip |
| `cd rhwp-studio && npm test` | 통과, 181 passed |
| `cd rhwp-studio && npm run build` | 통과 |
| 실제 앱 검증 | 브라우저에서 실제 Vite 모듈을 import 해 `getSelectedWrap()` 동작 확인 |

실제 앱 검증 관측값:

```text
buttons inactive + props.textWrap='Through' => Through
active Square button => Square
props 없음 fallback => Square
```

## visual sweep 판단

이 PR 은 그림 배치 값의 UI 저장 로직을 고치지만, 기준 PDF 와의 시각 위치 개선을 주장하지 않는다. 사용자가
속성창에서 아무것도 바꾸지 않았을 때 기존 속성이 유지되는지가 검증 대상이므로, 실제 브라우저 모듈 동작 확인으로
대체했다.

## merge 후 처리

- #2062 merge 후 #2054 auto-close 여부를 확인한다.
- 원 PR #2058 에 통합 PR #2062 로 처리됐음을 남기고 close 한다.

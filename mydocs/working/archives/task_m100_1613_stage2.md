# Task M100 #1613 2단계 완료보고서 — 파일 메뉴 항목 추가

- 이슈: #1613
- 브랜치: `local/task1613`
- 작성일: 2026-06-28
- 단계: 2/3

## 변경 내용

`rhwp-studio/index.html` 파일 메뉴(`data-menu="file"`)에 포맷 지정 저장 항목 2개 추가
(다른 이름으로 저장 아래, 구분선 위):

```html
<div class="md-item disabled" data-cmd="file:save-as-hwp">...HWP 형식으로 저장...</div>
<div class="md-item disabled" data-cmd="file:save-as-hwpx">...HWPX 형식으로 저장...</div>
```

## 활성/비활성 토글 — 자동

`menu-bar.ts` `updateMenuStates` 가 모든 `.md-item[data-cmd]` 를 순회하며
`dispatcher.isEnabled(cmdId)`(= 명령의 `canExecute`)로 `disabled` 클래스를 토글한다.
신규 명령은 `canExecute: ctx.hasDocument` 이므로 문서 열림 시 자동 활성화된다(별도 하드코딩 불필요).
`file:save` 의 title 특별 처리(line 141)는 신규 항목과 무관(title 없음).

## 검증

| 항목 | 결과 |
|---|---|
| studio `tsc` | 에러 0 |
| `npm test` | 147/147 |
| `npm run build` | 통과 |
| `dist/index.html` 신규 메뉴 항목 | 2건 반영 |
| 번들에 `saveAsFormat` 포함 | 확인 |

## 다음 단계

3단계: 수동/e2e 포맷 명시 저장 검증(HWP→HWPX, HWPX→HWP) + 기본 저장 회귀 확인 + 최종 보고서.

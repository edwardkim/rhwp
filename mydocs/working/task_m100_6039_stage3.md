---
kind: working
status: done
canonical: mydocs/plans/task_m100_6039.md
last_verified: 2026-08-25
---

# 작업 기록 — Task M100 #6039 Stage 3: 화면 확대·쪽 모양 UI

- **이슈**: [#6039](https://github.com/edwardkim/rhwp/issues/6039)
- **브랜치**: `codex/issue-6039-page-arrangement`
- **기준 commit**: `upstream/devel` `385e93b2c`
- **기록일**: 2026-08-25 KST

## 참고 범위

- [한컴 공식 화면 확대/축소 도움말](https://help.hancom.com/hoffice/multi/ko_kr/hwp/view/zooming/zoom.htm)
- [한컴오피스 한글 — 화면에 여러 쪽 보이게 설정하기](https://ttend.tistory.com/830)
- 사용자가 제공한 Windows 한컴 `확대/축소` 대화상자 화면

한컴의 `비율`과 `쪽 모양` 정보 구조 및 선택 동작을 참고했다. `쪽 이동`은 공식 도움말에서도 별도
동작이고 #6039의 수용 기준에 포함되지 않으므로 이번 단계에서 확장하지 않았다. 외형은 운영체제 고유
색상을 복제하지 않고 rhwp의 `ModalDialog`와 색상·포커스·간격 디자인 토큰을 사용했다.

## 구현 결과

### 확대/축소 대화상자

`ZoomDialog`는 한 화면에서 다음 설정을 함께 다룬다.

- 비율: `100%`, `125%`, `150%`, `200%`, `300%`, `500%`
- 맞춤: `폭 맞춤`, `쪽 맞춤`
- 사용자 정의: 정수 `10~500%`
- 쪽 모양: `자동`, `한 쪽`, `두 쪽`, `맞쪽`, `여러 쪽`
- 여러 쪽: 가로·세로 각각 `1~8`

사용자 정의와 여러 쪽 숫자 입력은 대응 라디오를 선택했을 때만 활성화된다. `여러 쪽`은 별도 비율
선택보다 가로×세로 전체가 뷰포트에 들어오는 계산 배율을 우선하며, 나머지 쪽 모양은 배율과 독립적으로
유지된다. 현재 배율은 대화상자를 다시 열 때 고정 프리셋, 폭/쪽 맞춤, 사용자 정의 순서로 복원한다.

### 공통 진입점과 상태 범위

- `보기 > 화면 확대/축소...`
- 상태 표시줄의 현재 배율 버튼

두 진입점은 모두 `view:zoom-dialog` 커맨드를 실행한다. 확인 시 `rhwp-settings.view.pageArrangement`를
저장하고 `page-arrangement-changed` 보기 이벤트를 보낸 뒤 수치 배율을 적용한다. 문서 변경 이벤트,
undo 기록, HWP/HWPX 데이터 변경은 발생시키지 않는다.

한컴의 500% 프리셋과 여러 쪽 저배율을 실제로 수용하도록 `ViewportManager`의 범위를 공용 문서 배율
상수 `0.05~5.0`과 일치시켰다. 기존 상태 표시줄의 폭/쪽 맞춤 및 확대/축소 버튼은 그대로 유지한다.

## Red 계약

구현 전 신규 테스트를 실행해 다음 실패를 확인했다.

- `zoom-dialog.ts` 및 비율 선택 모델 미존재
- 보기 메뉴와 상태 표시줄의 공통 대화상자 커맨드 미존재
- 한컴 비율·쪽 모양 UI 및 전용 토큰 CSS 미존재
- 500%와 여러 쪽 최소 배율을 허용하는 ViewportManager 범위 미존재

## 브라우저 검증

`http://127.0.0.1:7700/`의 새 문서에서 다음을 확인했다.

- 상태 표시줄 `100%` 버튼과 보기 메뉴가 같은 대화상자를 연다.
- 다크 테마에서 공통 표면·테두리·강조·비활성 토큰이 정상 표시된다.
- `여러 쪽` 선택 시 가로·세로 입력만 활성화되고, 2×2 적용 뒤 `22%`와 2×2 설정이 복원된다.
- `한 쪽 + 사용자 정의 500%`가 상태 표시줄에 `500%`로 반영된다.
- 각 적용 전후 `documentState.isDirty()`는 `false`를 유지한다.
- 조작 중 브라우저 콘솔 오류는 0건이다.
- 검증 후 사용자 설정은 `자동 + 100%`로 되돌렸다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `node --test tests/zoom-dialog.test.ts tests/zoom-dialog-integration.test.ts tests/page-arrangement.test.ts tests/virtual-scroll-page-arrangement.test.ts tests/canvas-view-page-arrangement.test.ts tests/user-settings.test.ts tests/zoom-fit.test.ts tests/zoom-anchor.test.ts tests/viewport-manager-smooth-zoom.test.ts tests/menu-shortcut-labels.test.ts` | 60/60 통과 |
| `npx tsc --noEmit --pretty false` | 통과 |
| `npm test` | 1,103 통과, 1 skip, 실패 0 |
| `npm run build` | 통과 |
| `git diff --check` | 통과 |

## 다음 단계

Stage 4에서 전체 단계 변경을 다시 검토하고 실제 다중 페이지 문서로 자동·한 쪽·두 쪽·맞쪽·여러 쪽의
전환, 클릭·현재 쪽·PageUp/PageDown을 통합 검증한 뒤 최종 보고서를 작성한다.

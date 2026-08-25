---
kind: report
status: done
canonical: mydocs/plans/task_m100_6039.md
last_verified: 2026-08-25
---

# Task M100 #6039 — 화면 확대·쪽 모양·쪽 이동 통합 보고서

- **이슈**: [#6039](https://github.com/edwardkim/rhwp/issues/6039)
- **브랜치**: `codex/issue-6039-page-arrangement`
- **기준 commit**: `upstream/devel` `385e93b2c`
- **검증일**: 2026-08-25 KST

## 결론

rhwp-studio의 기존 저배율 자동 다중 열 동작을 `자동`으로 보존하면서, 배율과 독립적인 `한 쪽`,
`두 쪽`, `맞쪽`, `여러 쪽` 배치를 추가했다. Windows 한글 2024 실측 피드백에 따라 `쪽 이동`의
`세로 방향`·`가로 방향`도 같은 확대/축소 대화상자에 통합했다. 가로 방향은 한컴과 같이 `한 쪽`을
자동 선택하고 다른 쪽 모양을 잠그며, 선택에 따라 세로 휠 입력을 가로 스크롤로 변환한다.

상황 선의 확대/축소 컨트롤은 한컴 2024의 흐름을 따라 다음 순서로 정리했다.

1. 폭 맞춤
2. 쪽 맞춤
3. 100%
4. 축소
5. 배율 슬라이더
6. 확대
7. 확대/축소 설정 메뉴
8. 현재 배율

현재 배율 버튼과 설정 메뉴는 같은 대화상자를 열고, 슬라이더·100%·축소·확대·맞춤 버튼은 기존
`ViewportManager` 배율 상태와 양방향으로 동기화된다.

## 한컴 동작과 적용 판단

- [한컴 공식 확대/축소 도움말](https://help.hancom.com/hoffice/multi/ko_kr/hwp/view/zooming/zoom.htm)은
  가로 방향을 한 쪽 보기에서만 제공하고, 세로 마우스 휠로 좌우 스크롤하는 선택 항목을 설명한다.
- [한컴 공식 보기 도구 상자 도움말](https://help.hancom.com/hoffice130/ko-KR/Hwp/view/toolbar/menu_view.htm)은
  상황 선의 배율 조절과 폭/쪽 맞춤 진입점을 별도 조작으로 제공한다.
- 사용자 제공 Windows 한글 2024 화면에서 세로 방향이 기본이고, 가로 방향 선택 시 한 쪽으로
  전환되는 것을 실측 기준으로 삼았다.

`쪽 윤곽`은 단순 아이콘 추가가 아니라 편집 영역의 용지 경계·쪽 간격 표시를 바꾸는 기능이다.
[한컴 공식 쪽 윤곽 도움말](https://help.hancom.com/hoffice/webhelp/9.0/ko_kr/hwp/view/page_outline.htm)의
렌더링 의미를 구현해야 하므로 #6039에는 아이콘만 추가하지 않고 독립 후속 범위로 남겼다.

## 구현

### 보기 상태와 저장

`PageArrangement`와 별도의 `PageMovementSettings`를 추가했다. 정규화 함수는 저장 데이터가 없거나
잘못됐을 때 `세로 방향 + 휠 변환 사용`으로 복구한다. 가로 방향과 다른 쪽 모양이 함께 들어오면
공통 해석 함수가 항상 `한 쪽`으로 정규화한다. 설정은 `rhwp-settings.view`에만 저장되며 문서 모델,
undo, HWP/HWPX 직렬화에는 들어가지 않는다.

### 세로·가로 레이아웃

`VirtualScroll`은 세로 방향에서 기존 단일 열·그리드 행 계약을 그대로 사용한다. 가로 방향에서는 모든
쪽을 한 행으로 잇고 쪽별 실제 폭과 간격을 반영한다. 화면보다 짧은 행은 중앙 정렬하고, 긴 행은 왼쪽
간격에서 시작한다. 가로 가시 영역과 앞뒤 한 쪽 프리페치를 X축으로 계산하므로 화면 밖의 모든 쪽을
동시에 렌더 대상으로 만들지 않는다.

`CanvasView`는 쪽 모양과 쪽 이동을 하나의 보기 전환으로 적용한다. 전환 전 현재 쪽의 상대 초점을
기억해 새 좌표에 복원하고, 행·열 토폴로지가 실제로 바뀔 때만 Canvas를 해제한다. 현재 쪽 판정,
쪽 이동, 캐럿 자동 스크롤도 가로 좌표를 사용한다.

### 입력과 UI

- 일반 세로 방향에서는 기존 휠 스크롤을 유지한다.
- 가로 방향에서 휠 변환을 켜면 수직 휠의 주축 입력을 `scrollLeft`에 적용한다.
- `Ctrl`/`Command`가 포함된 핀치·휠 확대는 일반 스크롤보다 먼저 처리하므로 우발적 배율 변경 계약을
  바꾸지 않는다.
- 확대/축소 대화상자에서 가로 방향을 고르면 한 쪽이 즉시 선택되고 `자동`, `두 쪽`, `맞쪽`,
  `여러 쪽`은 비활성화된다. 세로 방향으로 돌아오면 다시 선택할 수 있다.
- 상황 선 현재 배율은 44px 고정 폭과 tabular 숫자를 유지해 자리 수 변화로 왼쪽 버튼이 움직이지 않는다.

## Test-first 계약

구현 전 신규 focused test에서 다음 실패를 확인했다.

- `PageMovementSettings`와 가로 방향 정규화 모듈 미존재
- `VirtualScroll`의 가로 한 행 좌표·X축 visible/prefetch 계약 미존재
- 가로 방향 세로 휠→좌우 스크롤 변환 미존재
- 확대/축소 대화상자의 쪽 이동 UI와 한 쪽 강제 계약 미존재
- 상황 선의 100%·슬라이더·설정 메뉴 및 한컴 순서 미존재
- 사용자 설정의 쪽 이동 저장·복원 계약 미존재

구현 후 같은 테스트가 모두 통과해 기존 코드에서도 통과하던 사후 확인이 아니라 이번 변경 범위를
직접 포착했다.

## 브라우저 검증

`http://127.0.0.1:7700/`에서 새 문서에 쪽 나누기를 두 번 입력해 3쪽 문서로 검증했다.

- 상황 선의 버튼·슬라이더·설정 메뉴·현재 배율이 요청 순서와 접근성 이름으로 노출됐다.
- 가로 방향 선택 즉시 `한 쪽`이 선택되고 나머지 쪽 모양이 비활성화됐다.
- 휠 좌우 스크롤 선택 항목은 가로 방향에서만 활성화됐다.
- 적용 후 `horizontal-page-movement` 레이아웃이 활성화되고, 뷰포트 1,260px에 대해 문서 폭
  2,421px의 가로 스크롤 영역이 생성됐다.
- 슬라이더를 125%로 옮기면 현재 배율이 125%로 동기화되고, 축소 버튼 뒤 슬라이더와 배율 표시가
  함께 115%로 바뀌었다.
- 검증 후 사용자 설정은 `세로 방향 + 자동 + 100%`로 복원했다.

## 검증 결과

| 명령 | 결과 |
| --- | --- |
| `node --test tests/page-movement.test.ts tests/virtual-scroll-page-arrangement.test.ts tests/viewport-manager-smooth-zoom.test.ts tests/zoom-dialog.test.ts tests/zoom-dialog-integration.test.ts tests/zoom-fit.test.ts tests/user-settings.test.ts tests/canvas-view-page-arrangement.test.ts tests/virtual-scroll-grid-page.test.ts tests/page-scroll-step.test.ts` | 69/69 통과 |
| `npm test` | 1,115 통과, 1 skip, 실패 0 |
| `npm run build` | 통과 |
| `git diff --check` | 통과 |

## 후속 범위

- 줌 중 Canvas 교체·토폴로지 전환 성능: #6040
- 배율별 적응형 Canvas 렌더 해상도·픽셀 예산: #6041
- 스크롤 행 가상화·페이지 LRU·프리페치 정책: #6042
- 쪽 윤곽 표시/숨김과 실제 렌더 계약: 별도 이슈로 분리

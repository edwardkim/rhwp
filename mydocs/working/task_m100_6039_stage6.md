---
kind: working
status: done
canonical: mydocs/plans/task_m100_6039.md
last_verified: 2026-08-25
---

# Task M100 #6039 Stage 6 — 가로 이동 축 잠금·통합 배율 아이콘 보정

## 문제

가로 쪽 이동과 휠 좌우 변환을 켠 단일 페이지 문서에서 가로 우세 트랙패드 입력이 세로로 움직였다.
여러 페이지는 가로 overflow가 있어 같은 입력이 가로로 움직였다. 함께 제공한 화면에서는 상태 표시줄
우측 통합 배율 버튼의 CSS 돋보기가 왼쪽 축소·확대 스프라이트보다 크게 보였다.

## 원인

- `ViewportManager.onWheel()`은 세로 우세 입력만 `scrollLeft`로 변환하고, 가로 우세 입력은
  브라우저 native 스크롤에 맡겼다.
- 단일 페이지 실측은 `scrollWidth=clientWidth=1,260px`로 가로 이동 범위가 없지만,
  `scrollHeight=1,143px`, `clientHeight=525px`로 세로 overflow가 있었다. 혼합 축 트랙패드 이벤트의
  `deltaX`는 이동하지 못하고 작은 `deltaY`만 세로로 적용될 수 있는 조건이었다.
- 통합 버튼의 CSS 돋보기는 14px 원 밖으로 손잡이가 넘쳤고, 18px 아이콘 상자 자체를 공유하지 않았다.
  18px로 바꾼 첫 브라우저 실측에서도 flex 축소 때문에 실제 너비가 17.36px였다.

## 수정

- 가로 방향과 휠 좌우 변환이 켜졌으면 `abs(deltaX)`와 `abs(deltaY)` 중 큰 signed delta 하나를 골라
  `scrollLeft`에 적용한다.
- 가로·세로 우세 입력 모두 `preventDefault()`해 페이지 수와 overflow에 따른 native 축 전환을 막는다.
- 두 축을 합산하지 않아 대각선 제스처가 더 빠르게 움직이지 않도록 한다.
- 우측 돋보기는 18×18px 상자 안에 12px 원과 7px 손잡이를 그리고 `flex-shrink: 0`으로 고정한다.
- 배율 텍스트 영역은 36px로 조정해 통합 버튼 전체 68px 고정 폭을 보존한다.

## Test-first 증거

구현 전에 다음 실패를 확인했다.

- 가로 이동에서 `deltaX=24`, `deltaY=3`인 가로 우세 입력이 `preventDefault()`되지 않고
  `scrollLeft`도 증가하지 않았다.
- `.icon-zoom-menu::before` 원이 없고 아이콘 상자가 18px가 아니어서 시각 크기 계약이 실패했다.

구현 후 가로 우세 입력은 `scrollLeft 132→156`, 세로 우세 입력은 `100→132`로 이동하며 두 경우 모두
`scrollTop=0`을 유지했다. 휠 좌우 변환을 끄면 기존 native 동작을 유지한다.

## 브라우저 검증

`http://127.0.0.1:7700/`의 단일 페이지 문서에서 가로 방향이 한 쪽 배치를 강제하고 위 overflow 수치를
만드는 것을 확인했다. 상태 표시줄 computed geometry는 다음과 같다.

- 왼쪽 축소 스프라이트: 18×18px
- 우측 통합 배율 돋보기: 18×18px, `flex-shrink=0`
- CSS 원: 12×12px, 2px 선
- CSS 손잡이: 7px, 2px 선
- 배율 텍스트: 36px; 통합 버튼: 68px

검증 뒤 보기 설정을 `세로 방향 + 자동`으로 복원하고 브라우저 탭을 닫았다.

## 검증 결과

| 명령 | 결과 |
| --- | --- |
| `node --test tests/viewport-manager-smooth-zoom.test.ts tests/zoom-fit.test.ts` | 15/15 통과 |
| `npx tsc --noEmit --pretty false` | 통과 |
| `npm test` | 1,122 통과, 1 skip, 실패 0 |
| `npm run build` | 통과 |
| `git diff --check` | 통과 |

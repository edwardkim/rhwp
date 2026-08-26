---
kind: working
status: active
issue: 4768
stage: 1
last_verified: 2026-08-14
---

# #4768 Stage 1: Subsecond 대화형 개발 제어 활성화

## 배경

`subsecond:serve`는 Dioxus 기동 배너에 `/`, `r`, `p`, `v` 단축키를 표시하면서도 `--interactive false`를 전달해 실제 입력을 막고 있었다. 배너와 실제 동작의 불일치를 제거한다.

## 변경

- `subsecond:serve`의 Dioxus interactive 모드를 켠다.
- `7711`의 `127.0.0.1` 바인딩, hot-patch feature, Vite의 `7700` 공개 경로는 유지한다.
- 개발 가이드에 `/` 메뉴로 대화형 모드를 확인하고 자동 rebuild와 수동 rebuild를 구분하는 절차를 추가한다.

## 테스트 근거

- 동일 Dioxus 명령을 `--interactive true`로 실행해 `/` 단축키 메뉴가 나타나는 것을 확인했다.
- `npm run subsecond:serve -- --help`로 package 스크립트가 `--interactive true`와 loopback 주소를 전달하는 것을 확인했다.
- 일반 `npx vite`는 이 npm 스크립트를 호출하지 않으므로 변경 범위 밖이다.

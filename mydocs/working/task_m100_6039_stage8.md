---
kind: working
status: done
canonical: mydocs/plans/task_m100_6039.md
last_verified: 2026-08-25
---

# Task M100 #6039 Stage 8 — 배율 슬라이더 손잡이·중앙 눈금 보정

## 문제

rhwp 상황 선의 배율 슬라이더 손잡이가 한컴보다 크게 보였다. 현재 배율이 100%가 아닐 때 중앙의
100% 눈금이 슬라이더보다 앞 계층에 그려져 손잡이와 가까워지면 눈금이 손잡이 위에 겹칠 수 있었다.

## 원인

- 손잡이는 명시 크기 없이 브라우저의 기본 `range` thumb를 사용했다.
- 중앙 눈금은 12px였지만 `z-index: 3`, 슬라이더는 `z-index: 2`여서 눈금이 항상 앞에 놓였다.
- 100%일 때 눈금을 숨기는 별도 규칙이 계층 문제를 가렸고, 자연스러운 thumb 가림 계약이 아니었다.

## 수정

- 슬라이더 래퍼에 `--stb-zoom-thumb-size: 12px` 공통 토큰을 추가했다.
- WebKit과 Firefox thumb 모두 너비·높이에 이 토큰을 사용하고 1px 테두리와 원형 배경을 적용했다.
- 중앙 눈금 높이도 같은 토큰을 사용하고 20px 상황 선 안에서 수직 중앙 정렬했다.
- 중앙 눈금을 `z-index: 1`, 슬라이더를 `z-index: 2`로 유지해 눈금이 뒤에 놓이도록 했다.
- 100%에서 눈금을 숨기던 CSS를 제거해 thumb가 눈금을 자연스럽게 덮도록 했다.

## Test-first 증거

구현 전 focused test는 WebKit·Firefox thumb 규칙이 없고, 래퍼 공통 크기 토큰도 없어 7개 중 1개가
실패했다. 구현 후 같은 테스트 7개가 모두 통과했으며, 중앙 눈금이 앞에 놓이거나 100%에서 별도로
숨겨지는 CSS가 다시 생기지 않도록 고정했다.

## 브라우저 검증

`http://127.0.0.1:7700/`에서 100%와 63% 상태를 확인했다.

- 슬라이더 래퍼: 높이 20px, thumb 공통 크기 12px
- 중앙 눈금: 높이 12px, `top=4px`, `z-index=1`
- 슬라이더: 높이 20px, `z-index=2`
- 63%에서 중앙 눈금이 슬라이더 뒤에 유지되고, 100%에서는 thumb가 눈금을 덮음
- 기존 파란 진행 트랙과 100% 스냅 동작 유지

검증 뒤 배율을 100%로 복원하고 브라우저 탭을 닫았다.

## 검증 결과

| 명령 | 결과 |
| --- | --- |
| `node --test tests/zoom-fit.test.ts` | 7/7 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 1,123 통과, 1 skip, 실패 0 |
| `npm run build` | 통과 |
| `git diff --check` | 통과 |
| `cargo fmt --all && cargo fmt --all -- --check` | 소스 브랜치에서 제외하는 `tests/generated/regression_suite_*.rs` 32개가 없어 실행 불가; Rust 변경 없음 |

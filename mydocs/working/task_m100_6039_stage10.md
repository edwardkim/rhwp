---
kind: working
status: done
canonical: mydocs/plans/task_m100_6039.md
last_verified: 2026-08-25
---

# Task M100 #6039 Stage 10 — 통합 배율 버튼 내부 간격 보정

## 문제

통합 배율 버튼은 68px 고정 폭을 사용해 배율 자리 수가 달라져도 이웃 컨트롤이 움직이지 않았지만,
돋보기와 숫자 사이가 실제보다 넓게 보였다. 특히 `56%`처럼 짧은 값에서 여백이 두드러졌다.

## 원인

- 버튼 내부의 `justify-content: space-between`이 돋보기와 36px 숫자 영역을 양 끝으로 벌렸다.
- 숫자 영역을 우측 정렬해 짧은 값의 왼쪽에 추가 빈 공간이 생겼다.
- 버튼의 `gap: 4px`도 한컴 상태 표시줄보다 넓었다.

## 수정

- 68px 버튼 폭과 36px 숫자 예약 폭은 유지해 자리 수 변화에 따른 레이아웃 이동을 방지했다.
- 아이콘과 숫자 영역을 `flex-start`로 왼쪽부터 배치했다.
- 실제 아이콘·숫자 간격을 2px로 줄이고 숫자 영역은 왼쪽 정렬했다.

## Test-first 증거

구현 전 focused test에 `flex-start`, 2px 간격, 36px 숫자 폭, 왼쪽 정렬 계약을 추가하자 기존
`space-between + 4px + 우측 정렬` 구현에서 7개 중 1개가 실패했다. 구현 후 같은 7개가 모두 통과했다.

## 브라우저 검증

`http://127.0.0.1:7700/`에서 통합 배율 버튼을 실측했다.

- 버튼 폭: 68px
- 돋보기 SVG 상자: 18px
- 숫자 예약 폭: 36px
- 돋보기 오른쪽과 숫자 영역 시작점 사이 실제 간격: 2px
- 56%와 100% 모두 버튼 폭과 실제 간격 유지

## 검증 결과

| 명령 | 결과 |
| --- | --- |
| `node --test tests/zoom-fit.test.ts` | 7/7 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 1,123 통과, 1 skip, 실패 0 |
| `npm run build` | 통과 |
| `git diff --check` | 통과 |
| `cargo fmt --all && cargo fmt --all -- --check` | 소스 브랜치에서 제외하는 `tests/generated/regression_suite_*.rs` 32개가 없어 실행 불가; Rust 변경 없음 |

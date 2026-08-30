# 작업 기록 — Task M100 #6040 Stage 1.1

- **이슈**: [#6040](https://github.com/edwardkim/rhwp/issues/6040)
- **브랜치**: `codex/issue-6040-zoom-topology`
- **부모 commit**: Stage 1 재구성 문서 `ce3402e52` (재배치 전 `354b2d417`)
- **구현 commit**: `049683ee9` (재배치 전 `1d597e81b`)
- **작성일**: 2026-08-30 KST
- **결과 승인**: 2026-08-30 작업지시자 승인
- **Stage 범위**: 수평 눈금자 끝 라벨의 종이 경계 초과만 억제. 종이 경계·끝 tick과 기존 줌·배치 경로는 유지

## 문제와 판단

A4 155%에서 21cm 라벨은 중심이 종이 오른쪽 끝에 놓여 글자의 오른쪽 절반이 종이 밖 경계와 겹쳤다.
21cm 위치 자체와 끝 tick은 올바르므로 tick을 제거하거나 A4 길이를 예외 처리하지 않고, 가운데 정렬된
라벨의 실제 글자 폭이 현재 종이 범위에 온전히 들어오는지만 판정한다.

이 결함은 Stage 1 이전 기준 commit에도 있던 표시 문제다. Stage 1 재구성의 수동 검증에서 발견했으므로
#6040의 작은 후속 polish로 포함하되, 줌 좌표·`VirtualScroll`·Canvas 배치에는 손대지 않는다.

## 변경

- `ruler-label-geometry.ts`: 라벨 중심·실제 폭·종이 왼쪽·표시 폭으로 종이 내부 여부를 계산하는 순수 함수를 추가했다.
- `ruler.ts`: 수평 숫자 라벨을 그리기 직전에 `measureText()` 결과로 판정한다. 경계선과 모든 tick은 기존대로 그린다.
- `ruler-label-geometry.test.ts`: A4 155%의 20cm 표시·21cm 숨김, 경계 교차, 비정상 입력을 고정했다.

## 검증

- 눈금자 focused: 17건 중 17 pass
- TypeScript `npx tsc --noEmit`: 통과
- 전체 Studio: 1,278건 중 1,277 pass·1 skip·0 fail
- production build: 243 modules, 통과. 기존 대형 chunk 경고만 확인
- `cargo fmt --all -- --check`: 파생 regression suite 준비 뒤 통과. 파생 파일은 PR에 포함하지 않음
- 실제 Chromium, 6쪽 복구 문서, 자동 모드 155%:
  - 20cm 라벨 표시
  - 21cm 라벨 숨김
  - 종이 오른쪽 경계선과 끝 tick 유지
  - warning/error 0건
- 같은 문서 자동 60%: 2열, 점유 묶음 중심 오차 0.14px
- 같은 문서 자동 50%: 3열, 점유 묶음 중심 오차 0.07px
- `git diff --check`: 통과

## 범위 감사와 다음 게이트

Stage 1.1 source 변경은 수평 눈금자 그리기 한 지점과 순수 라벨 판정 함수뿐이다. `CanvasView`,
`VirtualScroll`, `ViewportManager`, caret/input 경로는 변경하지 않는다.

작업지시자가 직접 검증한 뒤 이 구현으로 확정했다. source·test·이 보고서를 별도 commit으로 고정하고,
#6040에서는 폐기한 Stage 2·3 설계를 재개하지 않는다. #6040의 제출 준비와 #6041 stack branch는 이
commit 뒤에만 진행한다.

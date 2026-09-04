# task-m100-3009 처리 결과: 줄 간격 줄이기 단축키 하한 clamp 누락 수정

## 이슈

- 이슈: https://github.com/edwardkim/rhwp/issues/3009
- 배경: 툴바 줄 간격 상한 clamp(PR #2930)를 조사하던 중, 짝을 이루는 감소 방향
  경로에도 같은 유형의 clamp 누락이 있는지 점검했다.

## 원인

`rhwp-studio/src/command/commands/format.ts`의 `format:line-spacing-increase`
커맨드(Alt+Shift+Z)는 `Math.min(500, current + 10)`으로 상한을 clamp하는데, 짝인
`format:line-spacing-decrease`(Alt+Shift+A)는 `current - 10`으로 하한 검사가 전혀
없었다. 단축키를 연타하면 줄 간격이 0 이하, 심지어 음수까지 내려갈 수 있었다.
toolbar.ts의 ▼ 버튼은 이미 `Math.max(5, cur - 5)`로 하한을 5로 clamp하고 있어,
같은 "줄 간격 줄이기" 동작인데 단축키 경로와 버튼 경로의 clamp가 어긋나 있었다.

## 수정 (rhwp-studio/src/command/commands/format.ts, 1줄)

- `newValue = current - 10` → `newValue = Math.max(5, current - 10)`.
- toolbar.ts ▼ 버튼과 동일한 하한(5%)으로 맞췄다.

## 테스트 (Red → Green)

- 신규 소스-가드 테스트: `rhwp-studio/tests/line-spacing-decrease-clamp.test.ts`
  - `format.ts` 소스에서 `format:line-spacing-decrease` 블록에
    `Math.max(5, current - 10)` 패턴이 있는지 정규식으로 확인.
  - Red: 수정 전 소스에는 해당 패턴이 없어 실패.
  - Green: 수정 후 통과 (`npx tsx --test tests/line-spacing-decrease-clamp.test.ts`).

## 커밋 / PR

- 브랜치: `task/m100-2965-linespacing-decrease-clamp` (origin/devel 기준)

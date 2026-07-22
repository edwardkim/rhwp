# Task #2880 처리 결과 — 표 객체 선택 Ctrl+C/Ctrl+X cellPath 누락 수정

## 이슈

- GitHub Issue: [#2880](https://github.com/edwardkim/rhwp/issues/2880)
- 대상 파일(단독 작업 영역): `rhwp-studio/src/engine/input-handler-keyboard.ts`

## 문제

`onKeyDown`의 표 객체 선택 모드(`isInTableObjectSelection`) Ctrl+C/Ctrl+X 핸들러가
`wasm.copyControl(ref.sec, ref.ppi, ref.ci)` / `wasm.exportControlHtml(ref.sec, ref.ppi, ref.ci)`를
`cellPathJson` 없이 호출했다. 같은 파일의 그림/글상자 객체 선택 모드 Ctrl+C/Ctrl+X 핸들러(및
`onCopy`)는 `pictureCellPathJson(ref)`로 계산한 `cellPathJson`을 반드시 함께 넘긴다 — 형제
핸들러 간 비대칭. `getSelectedTableRef()`가 반환하는 `ref.cellPath`(`CellPathEntry[]`)는 같은
블록 안에서 중첩 깊이 판정(`ref.cellPath.length > 1`)에는 쓰이고 있었지만 정작 copy/cut
호출에는 전달되지 않아, 셀 안에 중첩된 표를 복사·잘라내기 하면 native 가 본문 레벨 표로
오인해 엉뚱한 표를 클립보드에 담거나 조용히 실패했다.

## 수정

- `rhwp-studio/src/engine/input-handler-keyboard.ts` 표 Ctrl+C(867~890행 부근), Ctrl+X(891~919행
  부근) 블록에서 `pictureCellPathJson(ref)`로 `cellPathJson`을 계산해 `copyControl`/
  `exportControlHtml` 호출에 전달하도록 수정. 그림 개체 핸들러와 동일한 패턴.
- `deleteTableControl`은 시그니처상 `cellPathJson`을 받지 않아(별도 이슈 소지) 이번 수정
  범위에서는 제외 — 기존 가드(`ref.cellPath.length > 1`)로 중첩 표 삭제는 계속 차단됨.

## 테스트

- `rhwp-studio/tests/table-object-copy-cellpath.test.ts` 신규 추가 (source-guard, 최소 1개).
  - 표 Ctrl+C/Ctrl+X 블록 소스에서 `const cellPathJson = pictureCellPathJson(ref);`,
    `copyControl(..., cellPathJson)`, `exportControlHtml(..., cellPathJson)` 패턴을 확인.
  - Red: 수정 전(origin/devel 버전) 소스로 교체해 실행 → 실패 확인
    (`AssertionError: 표 Ctrl+C 핸들러가 ref.cellPath 를 cellPathJson 으로 계산하지 않음`).
  - Green: 수정본으로 복원 후 실행 → 통과.

## 검증

```
cd rhwp-studio
npm ci          # node_modules 부재 상태였어서 선행 필요
npm test        # 500 tests, 499 pass, 1 fail (cell-flow-boundary.test.ts — 기존 실패, 허용 범위)
npx tsc --noEmit  # TS2307 2건(@wasm/rhwp.js, 기존 baseline) — 신규 오류 0건
```

## 커밋/PR

- 브랜치: `task/m100-2880-table-copy-cellpath` (origin/devel 기준)
- 변경 파일: `rhwp-studio/src/engine/input-handler-keyboard.ts`,
  `rhwp-studio/tests/table-object-copy-cellpath.test.ts`,
  `mydocs/report/task_m100_2880_report.md`

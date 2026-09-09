# Task #2934 Report — 글자 모양 다이얼로그 장평/자간 clamp 누락 수정

## 요약

#2925/#2930(툴바 줄 간격 직접입력의 500% 상한 clamp 누락)과 동일한 패턴의 결함을
`rhwp-studio` 의 다른 UI 컨트롤에서도 찾기 위해, `src/command/commands/format.ts` 의
단축키 커맨드들과 그 UI 대응물(`toolbar.ts`, 관련 다이얼로그)을 짝지어 훑었다.

| 속성 | 단축키 커맨드 (format.ts → input-handler.ts) | UI 대응물 | 결과 |
|---|---|---|---|
| 줄 간격 | `format:line-spacing-increase` → `Math.min(500, ...)` | `toolbar.ts` 줄 간격 드롭다운/버튼 | 이미 #2930 에서 수정됨(대칭) |
| 글꼴 크기 | `format:font-size-increase/decrease` → `Math.max(100, ...)` | `toolbar.ts` `btnSizeUp/Down` → `Math.max(1, pt-1)` | 대칭 (둘 다 하한만 존재, 값도 동일) |
| 장평(W) | `format:char-ratio-*` → `adjustCharRatio` → `Math.max(50, Math.min(200, ...))` | `char-shape-dialog.ts` `saveLangFields()` | **비대칭** — clamp 없음 |
| 자간(P) | `format:char-spacing-*` → `adjustCharSpacing` → `Math.max(-50, Math.min(50, ...))` | `char-shape-dialog.ts` `saveLangFields()` | **비대칭** — clamp 없음 |

장평/자간 입력 필드는 `numberInput(50, 200)`/`numberInput(-50, 50)` 로 `<input type="number">`
의 `min`/`max` HTML 속성만 설정하는데, 이 속성은 스피너(▲▼) 클릭 시에만 값 범위를 강제하고
키보드로 직접 범위 밖 숫자를 입력한 뒤 폼 유효성 검사 없이 `.value` 를 읽으면 그대로 통과된다.
`saveLangFields()` 는 `parseInt(...) || 기본값` 만 수행하고 별도 clamp가 없어, 단축키로는
만들 수 없는 범위 밖 값(예: 장평 999%, 자간 -999)이 다이얼로그를 통해서는 문서에 기록될 수
있었다.

## 수정

`src/ui/char-shape-dialog.ts` 의 `saveLangFields()` 에서 `ratio`/`spacing` 파싱 결과를
`input-handler.ts` 의 `adjustCharRatio`/`adjustCharSpacing` 과 동일한 범위로 clamp 했다
(장평 50~200, 자간 -50~50). 변경은 2줄, diff 총 4줄이다.

## 검증

새 테스트 `tests/char-shape-dialog-ratio-spacing-clamp.test.ts` 는 소스 가드 방식으로,
`saveLangFields()` 블록에 clamp 패턴이 존재하는지와 `input-handler.ts` 의 기준 범위
(50~200, -50~50)가 그대로인지를 함께 확인한다.

- 수정 전: clamp 정규식 미매치로 `FAIL` 확인 (red)
- 수정 후: `PASS` 확인 (green)

```bash
cd rhwp-studio
npx tsx --test tests/char-shape-dialog-ratio-spacing-clamp.test.ts   # PASS
npm test                                                              # 500 tests, 1 fail
                                                                       # (cell-flow-boundary.test.ts, 기존에도 실패 — 무관)
npx tsc --noEmit                                                     # 기존 TS2307 2건만, 신규 0건
```

## 재현 (수정 전)

1. 문서에서 텍스트 선택 → Alt+L(글자 모양)
2. 장평(W) 필드에 `999` 입력 후 확인
3. 문서에 장평 999%가 적용됨 (단축키 Shift+Alt+K로는 최대 200%까지만 증가 가능했던 것과 대비)

## 영향 범위

`char-shape-dialog.ts` 의 `saveLangFields()` 4줄만 수정했으며, 다른 필드(상대 크기/글자 위치)
는 이번 결함과 무관하여 손대지 않았다.

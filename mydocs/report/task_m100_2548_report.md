# task_m100_2548 처리결과 보고서 — IME 조합 삭제 count 의 char 단위 정합

- **이슈**: [#2548](https://github.com/edwardkim/rhwp/issues/2548)
- **브랜치**: `task/m100-2548-char-offset-scalar` (base `devel` @ `3c54abfd`)
- **범위**: `rhwp-studio/src/engine/input-handler-text.ts` IME 조합 경로의 삭제 count 단위
- **분류**: 결함 수정 (astral 문자 인접 문자 손실)

## 1. 문제

[#2337-review] 에서 확립된 계약 — **WASM 삭제/조회 count 는 Rust `Paragraph::delete_text_at`
의 char(Unicode scalar) 단위**이며, JS `String.length`(UTF-16 code unit)를 넘기면 astral
문자(😀 등)에서 실제보다 많이 지운다. 당시 `InsertTextCommand.undo` 와 HF/FN 경로에는
`charCount()` 가 적용됐으나 **IME 조합 경로는 누락**돼 있었다.

```ts
this.compositionLength = text.length;               // :487  UTF-16 code unit
...
this.deleteTextAt(anchor, this.compositionLength);  // :481  scalar count 로 전달
```

`deleteTextAt(pos, count)`(:685)은 `count` 를 그대로
`wasm.deleteText / deleteTextInCell / deleteTextInCellByPath / deleteTextInHeaderFooter /
deleteTextInFootnote` 로 넘긴다. iOS 조합 폴백의 `_iosLength`(:553→:546) 도 동일하다.

### 영향

1. IME 조합 중 후보에 astral 문자가 포함(일본어·중국어 IME 의 이모지 후보 등)
2. 조합 업데이트 N 이 `😀`(UTF-16 2) 삽입 후 `compositionLength = 2` 기록
3. 조합 업데이트 N+1 이 anchor 에서 **scalar 2개** 삭제 → `😀` + **뒤따르는 실제 문자 1개** 제거
4. 조합 종료 시 `getTextAt(anchor, finalLength)` 도 이웃 문자를 포함해 읽어 undo 레코드가 부정확

## 2. 분석 — 오탐 배제

최초 이슈 등록 시 `InsertTextCommand.execute()` 의 `charOffset + this.text.length` 도 함께
지적했으나 **검증 결과 오탐이라 철회**했다(이슈 본문에 정정 고지 기재).

근거: 해당 값은 *커서 오프셋* 이며, 저장소가 UTF-16 관례로 **의도적으로 고정**해 두었다.

- `src/engine/command.ts:922` `charCount` 주석: "커서 오프셋은 studio 의 UTF-16 관례를
  유지하므로 여기서만 char 단위를 쓴다."
- `tests/undo-delete-char-count.test.ts`: "커서 오프셋(`charOffset + text.length`)은
  studio 의 UTF-16 관례를 유지하므로 제외하고, …"

즉 `execute()`(커서 오프셋, UTF-16)와 `undo()`(삭제 count, scalar)는 **서로 다른 양**이며
모순이 아니다. 본 수정은 **삭제/조회 count 에만** 적용하고 커서 오프셋은 손대지 않았다.

## 3. 변경

`rhwp-studio/src/engine/input-handler-text.ts`

| 위치 | 변경 | 성격 |
|---|---|---|
| 파일 상단 | `charCount()` 헬퍼 추가(`command.ts` 와 동일 정의, 계약 주석 포함) | 신규 |
| `:488` | `this.compositionLength = charCount(text)` | 삭제 count |
| `:553` | `this._iosLength = charCount(text)` | 삭제 count |

**불변으로 둔 지점**(커서 오프셋, UTF-16 관례 유지): `:497` `anchor.charOffset + text.length`,
`:615`/`:634` HF/FN `charOffset + text.length`.

## 4. 검증

### 신규 테스트

`rhwp-studio/tests/ime-composition-char-count.test.ts` (3건) — 선례
`tests/undo-delete-char-count.test.ts` 와 동일한 **정적 소스 가드** 방식.

1. `compositionLength` 가 `charCount(text)` 로 계산되고 `.length` 가 아님
2. `_iosLength` 동일
3. 삭제 count 로 쓰이는 두 변수에 UTF-16 `.length` 대입이 남아있지 않음(정규식 스캔)

### red→green 실증

수정을 되돌려(`compositionLength = text.length`) 실행 → **3건 중 2건 실패**.
복원 후 → **3건 전부 통과**. 가드가 실제로 회귀를 잡는다.

### 회귀

```
node --test tests/*.test.ts  →  pass 436 / fail 1
```

유일한 실패 `tests/cell-flow-boundary.test.ts` 는 **사전 실패**다. 변경을 `git stash` 로
제거한 깨끗한 `devel` 에서 같은 테스트를 단독 실행해 `pass 0 / fail 1` 로 동일하게 실패함을
확인했다 — 본 변경과 무관.

### 미실행 항목 (투명 고지)

- **행위 증명(브라우저 왕복)**: 이모지 후보 조합 → 인접 문자 보존 확인은 실행하지 않았다.
  선례 `undo-delete-char-count.test.ts` 도 동일 방침("행위 증명은 브라우저 왕복(PR 검증)")을
  주석에 명시하고 있어 그에 따랐다.
- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소 규약상
  작업지시자 별도 승인 사항이라 실행하지 않았다. Rust 코드는 변경하지 않았다.

## 5. 잔여 / 후속

같은 이슈 본문에 기록한 **각주 커서 경계**(`cursor.ts:1738,1752`)와
**단어 경계 탐색**(`cursor.ts:1775,1790`)은 본 변경에 포함하지 않았다. 이들은 *커서 오프셋*
계열이라 위 UTF-16 관례와의 관계를 먼저 확정해야 하며(머리말/꼬리말 쌍둥이는 WASM 이 준
`info.charCount` 를 쓰는 반면 각주판은 JS `.length` 로 계산하는 비대칭이 있다), 별도 판단이
필요하다. 범위를 섞지 않기 위해 분리했다.

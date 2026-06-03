# Task m100 #1155 Stage 1: 장평/자간 단축키 구현

## 1. 변경 범위

한컴 편집기 호환 단축키로 선택 범위의 장평/자간을 1%씩 증감하도록 rhwp-studio 명령을 추가했다.

대상 사용 시나리오:

```text
2줄로 배치된 문장을 블럭 선택한 뒤 Shift+Alt+J 로 장평을 줄여 1줄 배치에 가깝게 조정한다.
```

## 2. 구현 내용

### 2.1 단축키 매핑

`rhwp-studio/src/command/shortcut-map.ts`

| 기능 | 영문 키 | 한글 입력 모드 키 | 커맨드 |
|---|---|---|---|
| 장평 줄이기 | Shift+Alt+J | Shift+Alt+ㅓ | `format:char-ratio-decrease` |
| 장평 늘리기 | Shift+Alt+K | Shift+Alt+ㅏ | `format:char-ratio-increase` |
| 자간 줄이기 | Shift+Alt+N | Shift+Alt+ㅜ | `format:char-spacing-decrease` |
| 자간 늘리기 | Shift+Alt+W | Shift+Alt+ㅈ | `format:char-spacing-increase` |

IME 입력 중 `KeyboardEvent.key`가 `Process`로 들어오는 경우를 위해 `KeyboardEvent.code` 기반 매칭도 추가했다.

```text
KeyJ
KeyK
KeyN
KeyW
```

macOS의 Option 키는 브라우저 이벤트에서 `altKey`로 전달되므로 동일 경로로 처리된다.

### 2.2 서식 명령

`rhwp-studio/src/command/commands/format.ts`

다음 format command를 추가했다.

```text
format:char-ratio-decrease
format:char-ratio-increase
format:char-spacing-decrease
format:char-spacing-increase
```

### 2.3 선택 범위 적용

`rhwp-studio/src/engine/input-handler.ts`

선택 범위가 있을 때만 기존 `ApplyCharFormatCommand` 경로로 글자 속성을 적용한다.

```text
장평:
  current = ratios[0] 또는 100
  delta = -1 또는 +1
  clamp = 50..200
  적용 = ratios[7]

자간:
  current = spacings[0] 또는 0
  delta = -1 또는 +1
  clamp = -50..50
  적용 = spacings[7]
```

### 2.4 장평/자간 변경 후 reflow

메인테이너 동작 테스트에서 다음 문제가 확인되었다.

```text
단축키로 장평/자간 값은 변경되지만,
선택 블럭의 텍스트 폭 변화에 따라 줄나눔이 다시 계산되지 않는다.
```

원인:

```text
DocumentCore::apply_char_format_native()
DocumentCore::apply_char_format_in_cell_native()
```

위 경로에서 LineSeg 재계산 조건이 `base_size` 변경에만 걸려 있었다.
장평(`ratios`)과 자간(`spacings`)은 텍스트 폭을 직접 바꾸므로 글꼴 크기 변경과 동일하게
문단 reflow가 필요하다.

조치:

```text
char_shape_mods_affect_text_flow()
```

헬퍼를 추가하고 다음 글자 모양 변경은 LineSeg reflow 대상에 포함했다.

```text
base_size
font_ids
ratios
spacings
relative_sizes
char_offsets
```

본문 문단과 셀 내부 문단 모두 동일하게 적용했다.

## 3. 테스트

추가 테스트:

```text
rhwp-studio/tests/shortcut-map.test.ts
```

검증 항목:

```text
Shift+Alt+J/K/N/W
Shift+Alt+ㅓ/ㅏ/ㅜ/ㅈ
IME Process + event.code
```

실행 결과:

```text
cd rhwp-studio
npm test
```

결과:

```text
success
```

빌드:

```text
cd rhwp-studio
npm run build
```

결과:

```text
success
```

Rust 검증:

```text
cargo fmt
cargo check
cargo test char_ratio_and_spacing_changes_require_text_reflow
cargo test paint_only_char_shape_changes_do_not_require_text_reflow
```

결과:

```text
success
```

WASM:

```text
docker compose --env-file .env.docker run --rm wasm
cp pkg/rhwp.js pkg/rhwp_bg.wasm pkg/rhwp.d.ts rhwp-studio/public/
cd rhwp-studio
npm run build
```

결과:

```text
success
```

## 4. 메인테이너 수동 판정 필요 항목

브라우저에서 다음 동작을 확인한다.

```text
1. 2줄 문장 일부 또는 전체를 블럭 선택한다.
2. Shift+Alt+J 를 반복 입력해 장평이 1%씩 줄어드는지 확인한다.
3. Shift+Alt+K 로 장평이 다시 1%씩 늘어나는지 확인한다.
4. Shift+Alt+N/W 로 자간이 1%씩 줄고 늘어나는지 확인한다.
5. 한글 입력 모드에서 ㅓ/ㅏ/ㅜ/ㅈ 조합도 동일하게 동작하는지 확인한다.
6. macOS에서는 Shift+Option+J/K/N/W 조합을 확인한다.
```

## 5. 판정

자동 검증 기준으로 구현 완료.

메인테이너 브라우저 동작 판정 대기.

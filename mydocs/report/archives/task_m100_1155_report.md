# Task m100 #1155 완료 보고서

## 1. 이슈

```text
#1155 rhwp-studio: 단축키로 장평/자간 1% 증감 지원
```

## 2. 완료 내용

한컴 편집기 호환 장평/자간 단축키를 rhwp-studio에 추가했다.

| 기능 | 단축키 | 한글 입력 모드 | 처리 |
|---|---|---|---|
| 장평 1% 줄이기 | Shift+Alt+J | Shift+Alt+ㅓ | 성공 |
| 장평 1% 늘리기 | Shift+Alt+K | Shift+Alt+ㅏ | 성공 |
| 자간 1% 줄이기 | Shift+Alt+N | Shift+Alt+ㅜ | 성공 |
| 자간 1% 늘리기 | Shift+Alt+W | Shift+Alt+ㅈ | 성공 |

macOS에서는 Option 키가 브라우저 `altKey`로 전달되므로 같은 경로로 처리한다.

## 3. 핵심 수정

### 3.1 단축키 매핑

```text
rhwp-studio/src/command/shortcut-map.ts
```

영문 키, 한글 입력 모드 키, IME `Process` 상태의 `event.code` 매칭을 지원하도록 확장했다.

### 3.2 서식 명령

```text
rhwp-studio/src/command/commands/format.ts
rhwp-studio/src/engine/input-handler.ts
```

선택 블럭에 대해 기존 `ApplyCharFormatCommand` 경로로 `ratios`와 `spacings`를 적용한다.

### 3.3 리플로우

동작 테스트 중 다음 문제가 확인되었다.

```text
장평/자간 값은 변경되지만 줄나눔이 다시 계산되지 않는다.
```

원인은 Rust core의 `apply_char_format` 경로가 LineSeg 재계산 조건을 `base_size` 변경에만 한정한 것이었다.

```text
src/document_core/commands/formatting.rs
```

다음 글자 모양 변경이 텍스트 폭/높이에 영향을 주는 것으로 판단되면 문단 reflow를 수행하도록 보강했다.

```text
base_size
font_ids
ratios
spacings
relative_sizes
char_offsets
```

본문 문단과 셀 내부 문단 모두 적용했다.

## 4. 검증

자동 검증:

```text
cd rhwp-studio
npm test
npm run build

cargo fmt
cargo check
cargo test char_ratio_and_spacing_changes_require_text_reflow
cargo test paint_only_char_shape_changes_do_not_require_text_reflow

docker compose --env-file .env.docker run --rm wasm
cp pkg/rhwp.js pkg/rhwp_bg.wasm pkg/rhwp.d.ts rhwp-studio/public/
cd rhwp-studio
npm run build
```

결과:

```text
success
```

메인테이너 동작 테스트:

```text
자간, 장평 모두 단축키로 동작 확인
장평 줄이기 후 선택 줄 전체 텍스트 reflow 변경 확인
```

결과:

```text
통과
```

## 5. 판정

```text
#1155 구현 완료
```

# Task #241 완료 보고서

## 1. 이슈

- GitHub Issue: #241
- 제목: `[그림] 때문에 공백이 생기는 문제`
- 브랜치: `local/task241`
- 샘플:
  - `samples/hwpx/issue_241.hwpx`
  - `samples/hwpx/hancom-hwp/issue_241.hwp`
  - `samples/hwpx/issue_241.pdf`
  - `pdf/hwpx/issue_241-2022.pdf`

## 2. 문제

`samples/hwpx/issue_241.hwpx`의 도장 그림은 다음 속성을 가진다.

```text
textWrap = InFrontOfText
vertRelTo = Para
verticalOffset = 754 HU
treatAsChar = false
```

초기 관찰:

```text
1. rhwp-studio 개체 속성 대화창에서 세로 기준 `문단`이 바인딩되지 않음
2. 디버그 SVG에서 도장 그림이 포함된 `s0:pi=9` 문단의 flow 높이가 확보되지 않아
   `s0:pi=10` 날짜 문단이 같은 y에서 시작함
```

## 3. 수정

수정 파일:

- `rhwp-studio/src/ui/picture-props-dialog.ts`
- `src/renderer/layout.rs`
- `src/renderer/svg.rs`

수정 내용:

```text
1. rhwp-studio 그림 속성 대화창의 세로 기준 option value를 `Paragraph`에서 `Para`로 정정
2. Para-relative InFrontOfText/BehindText 그림 Shape item이 본문 flow cursor를
   host paragraph y로 되감지 않도록 정정
3. debug overlay 라벨 충돌 회피 및 이미지 bbox 라벨을 추가해 문단/이미지 관계를 추적 가능하게 함
```

## 4. 회귀 테스트

추가 파일:

- `tests/issue_241.rs`

검증 내용:

```text
1. 도장 이미지 front overlay bbox가 Hancom PDF 기준과 1px 이내로 일치
2. `s0:pi=9` 도장 host paragraph가 flow line advance를 확보하여
   `s0:pi=10` 날짜 문단이 아래로 진행됨
```

## 5. 검증

실행한 검증:

```text
cargo fmt
cargo check
cargo test --test issue_241
./rhwp-studio/node_modules/.bin/tsc --noEmit -p rhwp-studio/tsconfig.json
git diff --check
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
cargo check: success
issue_241: 2 passed
tsc: success
diff check: success
WASM build: success
```

디버그 SVG:

```text
output/poc/task241_debug_grid_image_label/hwpx/issue_241.svg
```

수정 후 주요 라벨:

```text
s0:pi=9  y=877.3
s0:pi=10 y=898.7
s0:pi=9 ci=0 image y=887.4
```

## 6. 시각 판정

작업지시자 판정:

```text
SVG 시각 판정: 통과
rhwp-studio 시각 판정: 통과
```

## 7. 결론

#241의 그림 위치 문제는 두 축으로 정리되었다.

```text
1. UI 바인딩: `Para` 값을 `문단` option으로 정상 매칭
2. 렌더링 flow: InFrontOfText/BehindText 그림을 그린 뒤 host paragraph의 line advance를 유지
```

도장 이미지 좌표는 Hancom PDF 기준과 1px 이내로 일치하고,
도장 그림이 정의된 `s0:pi=9` 문단도 한컴 편집기처럼 독립적인 문단 높이를 확보한다.

## 8. 다음 절차

보고서 승인 후 다음 절차를 진행한다.

```text
1. 커밋
2. local/devel 병합
3. devel 검증
4. 원격 devel push
5. 이슈 #241 close
```

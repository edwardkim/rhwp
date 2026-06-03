# Task M100 #1139 Stage 14

## 목적

한컴오피스 격자 설정에서 `쪽`/`종이`를 바꿀 때 자동 계산되는 격자 기준 위치가 rhwp-studio와 다르게 표시되는 원인을 찾는다.

## 작업지시자 기준 화면

`hwp3-sample16-hwp5.hwp` 한컴오피스 기준:

- `격자 기준 위치: 쪽`
  - 가로 `0.00mm`
  - 세로 `3.99mm`
- `격자 기준 위치: 종이`
  - 가로 `15.00mm`
  - 세로 `24.00mm`

현재 rhwp-studio 기준:

- `격자 기준 위치: 쪽`
  - 가로 `0mm`
  - 세로 `0mm`
- `격자 기준 위치: 종이`
  - 가로 `15mm`
  - 세로 `20mm`

## 초기 차이

- rhwp-studio의 `paper` 기본값은 현재 `PageDef.marginLeft`, `PageDef.marginTop + marginHeader`만 사용한다.
- 한컴 기준 `paper.y = 24.00mm`는 `page.y 3.99mm + (marginTop 10.00mm + marginHeader 10.00mm)`와 맞는다.
- 따라서 누락된 값은 `쪽` 기준 세로 원점 약 `3.99mm`다.

## 분석 계획

1. `hwp3-sample16-hwp5.hwp`의 `PageDef`, `PageBorderFill`, `SectionDef`, HWP3 변환 흔적을 비교한다.
2. `쪽` 기준 세로 원점 `3.99mm`가 어떤 raw field 또는 기존 모델 값과 대응되는지 찾는다.
3. rhwp-studio 표시값과 실제 grid overlay 원점 계산 경로를 함께 확인한다.
4. 원인이 확인되면 소스 수정 전 작업지시자 승인을 받는다.

## 상태

자동 검증 완료. 작업지시자 시각 검증 대기.

## 확인 결과

- `hwp3-sample16-hwp5.hwp`와 `3-09월_교육_통합_2022.hwp`의 `SECTION_DEF` grid 값은 모두 0이다.
  - `lineGrid = 0`
  - `charGrid = 0`
- 두 파일의 `CTRL_HEADER(secd)`와 `CTRL_DATA`도 같은 값이다.
- CFB 추가 스트림에는 격자 기준 위치로 보이는 별도 저장값이 없다.
  - 두 파일 모두 `DocOptions/_LinkDoc`만 있고 내용은 0으로 채워져 있다.
  - `DocumentPane`, `ViewProperties`, `PrinterSettings` 같은 view/grid 후보 스트림은 없다.
- rhwp-studio 설정창은 현재 `view.ts`의 `getGridOriginDefaults()`에서 고정 계산한다.
  - `쪽`: `{ x: 0, y: 0 }`
  - `종이`: `{ x: PageDef.marginLeft, y: PageDef.marginTop + PageDef.marginHeader }`
- rhwp-studio 설정창의 기준 전환은 한컴처럼 현재 절대 원점을 보존해 환산하지 않는다.
  - `GridSettingsDialog.onOriginChanged()`는 현재 값이 이전 기준의 기본값과 같을 때만 다음 기준의 기본값으로 치환한다.
  - 현재 값이 기본값이 아니면 `쪽`/`종이`를 바꿔도 좌표를 환산하지 않는다.

## 한컴 값 해석

작업지시자 한컴오피스 화면 기준:

- `쪽`: `0.00mm`, `3.99mm`
- `종이`: `15.00mm`, `24.00mm`

두 값의 차이는 `PageDef.marginLeft`, `PageDef.marginTop + PageDef.marginHeader`와 맞는다.

- `x`: `15.00 - 0.00 = 15.00mm`
- `y`: `24.00 - 3.99 = 20.01mm`
- `hwp3-sample16-hwp5.hwp`의 `PageDef`:
  - `marginLeft = 4252HU = 15.00mm`
  - `marginTop + marginHeader = 2836HU + 2836HU = 20.01mm`

따라서 한컴은 기준을 바꿀 때 다음 관계를 유지한다.

- `종이 기준 값 = 쪽 기준 값 + 본문 기준 위치`
- `쪽 기준 값 = 종이 기준 값 - 본문 기준 위치`

현재 rhwp-studio는 이 환산을 하지 않고 기준별 기본값으로만 교체하므로, 같은 화면에서 `쪽 0/0`, `종이 15/20`으로 보인다.

## 남은 판단

한컴 화면의 절대 격자 원점은 `종이 기준 15.00mm, 24.00mm`이다. 이 값은 `PageDef`만으로는 `15.00mm, 20.01mm`까지만 설명된다.

추가 `3.99mm`는 파일 내 별도 grid origin 필드에서는 발견되지 않았다. 현재까지 확인한 후보 중 값 자체는 `SECTION_DEF.columnSpacing = 1134HU = 4.00mm`와 일치하지만, 두 비교 파일 모두 같은 `columnSpacing`을 갖기 때문에 이것만으로 일반 규칙으로 삼으면 `3-09월_교육_통합_2022.hwp`가 회귀할 수 있다.

## 수정 후보

1. rhwp-studio 기준 전환 로직부터 한컴처럼 절대 원점 보존 방식으로 바꾼다.
   - 현재 기준의 base를 더해 종이 절대 원점을 계산한다.
   - 다음 기준의 base를 빼서 표시값을 갱신한다.
   - 예: `paper(15,24) -> page(0,3.99)`, `page(0,3.99) -> paper(15,24)`
2. `hwp3-sample16-hwp5.hwp`의 초기 절대 원점이 한컴에서 왜 `15/24`가 되는지는 별도 보정 후보로 분리한다.
   - HWP3 변환본 플래그와 `SECTION_DEF.columnSpacing` 조합을 의심하지만, 일반 HWP5/HWPX에 바로 적용하지 않는다.

## 승인

작업지시자가 `hwp3-sample16-hwp5.hwp` 기준 수정을 승인했다.

## 구현

- WASM `getDocumentInfo()` JSON에 `hwp3Variant` 플래그를 추가했다.
- rhwp-studio `DocumentInfo` 타입에 `hwp3Variant`를 반영했다.
- rhwp-studio 격자 설정 기본값 계산을 기준별 표시값과 절대 좌표 base로 분리했다.
  - `paper` base: 종이 좌상단 `0,0`
  - `page` base: `PageDef.marginLeft`, `PageDef.marginTop + PageDef.marginHeader`
- 격자 기준 전환은 현재 표시값을 절대 종이 좌표로 바꾼 뒤 다음 기준의 base를 빼는 방식으로 수정했다.
  - `paper 15,24 -> page 0,3.99`
  - `page 0,3.99 -> paper 15,24`
- `hwp3Variant && pageBorderFill.basis === 'page'`인 경우 `SectionDef.columnSpacing`을 HWP3 변환본 쪽 기준 기본 세로 보정으로 사용했다.
  - `hwp3-sample16-hwp5.hwp`: `page 0,3.99`, `paper 15,24`
  - `3-09월_교육_통합_2022.hwp`: page border basis가 `paper`이므로 `page 0,0`, `paper 9,24` 유지
- 격자 기준 환산 단위 테스트를 추가했다.

## 검증

- `cargo fmt --all -- --check`
- `npm test -- --test-name-pattern=격자`
- `npm run build`
- `cargo test -q test_empty_document_info --lib`
- `wasm-pack build --target web --out-dir pkg`
- `cargo build`
- Node WASM probe:
  - `samples/hwp3-sample16-hwp5.hwp` → `page {x:0,y:3.99}`, `paper {x:15,y:24}`
  - `samples/3-09월_교육_통합_2022.hwp` → `page {x:0,y:0}`, `paper {x:9,y:24}`
- `git diff --check`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`

검증 결과:

- 모두 통과.
- `npm run build`는 기존 대형 chunk 경고만 표시했다.
- Rust test는 기존 warning만 표시했다.

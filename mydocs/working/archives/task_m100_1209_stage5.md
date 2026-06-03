# Task M100 #1209 Stage 5

## 목적

`3-09월_교육_통합_2024-구분선아래20.hwp` 8쪽 `문29)`의 그림 개체가 한컴오피스 기준 `어울림`으로 확인되었음에도 RHWP에서 본문과 다른 위치 관계로 렌더링되는 문제를 분석한다.
한컴 도움말의 “본문과의 배치” 기준에 맞춰 `어울림` 그림이 같은 줄을 나누어 쓰되 본문 영역을 침범하지 않도록, 파싱 라벨/진단기/공통 레이아웃 처리를 분리해서 보정한다.

## 시작 기준

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- 작업 브랜치: `local/task_m100_1209`
- 선행 커밋: `ca9eaace` (`task 1209: Stage4 문19 줄 겹침 보정`)
- 대상 HWP: `samples/3-09월_교육_통합_2024-구분선아래20.hwp`
- 대상 PDF: `pdf/3-09월_교육_통합_2024-구분선아래20-2024.pdf`
- 대상 페이지: 8쪽
- 대상 문항: `문29)` 그림 개체

## 확인 질문

1. HWP 바이너리에서 해당 그림의 wrapping/placement 필드가 실제로 `어울림(Square)`인지 확인한다.
2. `어울림` 본문 회피 위치가 CommonObjAttr 오프셋인지, `LINE_SEG`의 줄폭/세로 위치 정보인지 구분한다.
3. 한컴 도움말의 `어울림` 의미처럼 본문과 그림이 같은 줄을 나누되 서로 침범하지 않는지 확인한다.
4. 기존 `자리차지(TopAndBottom)` 및 PR #1015 계열 처리와 충돌 없이 공통 처리할 수 있는가?

## 진행 계획

1. 8쪽 RHWP SVG를 내보내고 `rsvg-convert`로 PNG를 생성해 현재 배치 기준을 고정한다.
2. 대상 PDF 8쪽을 PNG로 변환해 한컴/PDF 배치 기준과 비교한다.
3. `dump-pages` 또는 전용 진단으로 `문29)` 그림 control의 raw property, IR placement/wrap 값을 확인한다.
4. 파서, IR 변환, Studio 속성 표시 중 어느 레이어의 문제인지 좁힌다.
5. 수정이 필요하면 공통 매핑/파싱 로직을 보정하고 `tests/issue_1139_inline_picture_duplicate.rs`에 회귀 테스트를 추가한다.

## 현재 상태

- 2026-06-01: Stage4 커밋 `ca9eaace` 이후 새 스테이지 문서를 만들고 분석을 시작했다.
- 2026-06-01: 작업지시자가 `3-09월_교육_통합_2024-구분선아래20.hwp` 8쪽 `문29)` 그림 개체가 RHWP에서는 `어울림`으로 표시되지만 한컴오피스 2024에서는 `자리차지`로 확인된다고 보고했다.
- 2026-06-01: PR #1015의 `samples/test-image.hwp`/`samples/test-image.hwpx` fixture 기준을 재확인했다. HWP5 raw/HWPX `textWrap` 기준 매핑은 `TopAndBottom(bit 1)=자리차지`, `Square(bit 0)=어울림`, `BehindText(bit 2)=글뒤로`, `InFrontOfText(bit 3)=글앞으로`이다.
- 2026-06-01: `test-image.hwp`가 각 배치 방식을 별도 페이지/문단의 단일 그림으로 다시 저장되었다. 현재 control record 순서와 문단 라벨 순서는 `TopAndBottom(자리차지) → Square(어울림) → InFrontOfText(글앞으로) → BehindText(글뒤로)`이다.
- 2026-06-01: 작업지시자가 제공한 현재 `test-image.hwp` 기준으로 `글자처럼 취급` 그림도 재확인했다. 이 그림은 별도 문단의 단일 control이고 `tac=true`, `wrap=BehindText/글뒤로`, `hrel=Para`, `vrel=Para`로 저장되어 있다. `글자처럼 취급`은 wrap 종류가 아니라 `treat_as_char` 플래그로 별도 표시해야 한다.
- 2026-06-01: 기존 `dump` 진단 라벨이 잘못되어 있었다. `TextWrap::TopAndBottom`을 `위아래`로, `TextWrap::Tight`를 `자리차지`로 표시해 PR #1015 기준과 어긋났다. `dump` 라벨을 `Square=어울림`, `TopAndBottom=자리차지`, `BehindText=글뒤로`, `InFrontOfText=글앞으로` 기준으로 정리했다.
- 2026-06-01: `hwp5-anchor-trace`도 CommonObjAttr 비트 해석을 함께 출력하도록 보강했다. 이제 `CTRL_HEADER(GenShape/Table)`에 `tac`, `wrap bits`, `vrel`, `hrel`, `flowWithText`, `allowOverlap`, `bit26`이 함께 표시된다.
- 2026-06-01: PR #1015의 `paragraph_layout.rs` fallback은 새 `test-image.hwp`에서도 1쪽 `자리차지` 문단의 `line_seg.vpos=15180` 처리에 필요하므로 유지한다. 다만 기존 주석의 “혼합 배치” 설명은 현재 fixture 구조와 달라 `test-image.hwp page 1: TopAndBottom Picture`로 정정했다.
- 2026-06-01: 작업지시자가 후속 캡처로 대상 그림(`pi=429`, `ci=21`)의 한컴 2024 개체 속성이 **어울림**임을 확인했다. 대상 raw 속성은 `properties=0x070a2b10`, `wrap bits=0`, `IR=Square`, `tac=false`, `vrel=Para`, `hrel=Para`, `halign=Right`이다. 따라서 이 대상은 `자리차지`로 강제 변환하지 않고 `Square/어울림` 레이아웃을 보정한다.
- 2026-06-01: 스펙 원문 표는 유지하고, 아래에 Task #1209 Stage5 보완 주석을 추가했다. 기존 표는 HWPX/스펙 명칭을 제시하지만, HWP5 CommonObjAttr 실측 기준은 `Square=어울림`, `TopAndBottom=자리차지`, `BehindText=글뒤로`, `InFrontOfText=글앞으로`로 표시한다. raw 저장값/IR enum/한컴 UI 명칭은 분리해서 다룬다.
- 2026-06-01: `cargo fmt --all --check`, `cargo check --bin rhwp` 통과. 진단기 수정은 컴파일 가능한 상태로 확인했다.
- 2026-06-01: 작업지시자가 `test-image.hwp` 1쪽 `자리차지` 그림이 편집기에서 본문과 겹쳐 보인다고 보고했다. `export-svg`/`rsvg-convert` 기준 단일 라벨 케이스는 그림 하단과 텍스트 줄 상단이 맞닿아 있지만, 기존 로직은 첫 줄 `vpos`만 문단 시작에 한 번 더하는 fallback이라 “텍스트-그림-텍스트”처럼 한 문단 안에서 흐름이 갈라지는 자리차지 사례를 공통 처리하지 못한다.
- 2026-06-01: `paragraph_layout`에서 비-TAC `TopAndBottom` + `VertRelTo::Para` 그림/도형을 가진 문단은 `LINE_SEG.vertical_pos`를 줄별 y 좌표로 반영하도록 공통화했다. 기존 `test-image.hwp` 1쪽 fallback은 이 공통 경로로 대체하고, 회귀 테스트로 자리차지 그림 하단보다 텍스트 줄이 위로 올라오지 않는 조건을 추가했다.
- 2026-06-01: 대상 파일 화면 하단이 `3-09월_교육_통합_2022.hwp`로 표시되어 2022/2024-구분선아래20 양쪽을 같은 `pi=429`, `ci=21`로 재확인했다. 두 파일 모두 raw `properties=0x070a2b10`, `wrap bits=0`, `tac=false`, `vrel=Para`, `hrel=Para`, `halign=Right`이며, 8쪽 `LINE_SEG`도 후반 4줄 폭이 `26788`에서 `10862`로 줄어 그림 옆 본문 흐름을 저장한다. 이는 `test-image.hwp`의 `Square/어울림` 저장값 및 실제 어울림 레이아웃과 일치하므로 이 개체를 `TopAndBottom/자리차지`로 강제 보정하지 않는다.
- 2026-06-01: 한컴 도움말 `본문과의 배치` 설명을 확인했다. `어울림`은 개체와 본문이 같은 줄을 나누어 쓰되 서로 자리를 침범하지 않도록 흐르는 배치이며, `자리 차지`와 별도 의미다. 따라서 대상 그림은 배치 종류가 아니라 `LINE_SEG`의 첫 좁은 줄 위치에 맞춰 그림 top을 내리는 쪽으로 보정한다.
- 2026-06-01: `Square/어울림 + VertRelTo::Para` 비-TAC 그림은 문단 시작 y가 아니라 `LINE_SEG`가 처음 좁아지는 줄의 `vertical_pos`를 그림 상단으로 삼도록 보정했다.
- 2026-06-01: `cargo fmt --all --check`, `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`, `wasm-pack build --target web --out-dir pkg`를 통과했다. `rsvg-convert`로 2024 구분선아래20 8쪽 PNG를 생성했고, 작업지시자가 시각적 판단을 확인했다.

## 현재 판단

진단기 라벨은 `test-image.hwp/.hwpx` 기준으로 정정한다. HWP5 CommonObjAttr의 `wrap bits=0`은 `Square/어울림`, `wrap bits=1`은 `TopAndBottom/자리차지`이다.
대상 그림의 raw CommonObjAttr는 `wrap bits=0`, rhwp IR은 `Square`이므로 현재 진단 기준으로는 `어울림`이 맞다. 문단 `LINE_SEG`도 그림 옆에서 줄 폭이 줄어드는 어울림 배치를 저장하고 있으므로, 이 대상은 파서/렌더러에서 자리차지로 뒤집지 않는다.
`Square/어울림 + 문단 기준` 그림은 문단 첫 줄 y가 아니라 `LINE_SEG`가 처음 좁아지는 줄의 `vertical_pos`를 그림 상단으로 삼아야 한컴처럼 위쪽 full-width 본문과 그림이 겹치지 않는다.
다만 실제 `TopAndBottom/자리차지` 문단의 렌더링은 개별 fallback이 아니라 줄별 `LINE_SEG.vertical_pos`를 따르는 공통 경로로 처리해야 한다. 이 경로가 자리차지 그림 주변의 본문 겹침을 막는 기준이다.

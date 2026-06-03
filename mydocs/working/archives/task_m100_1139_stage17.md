# Task M100 #1139 Stage 17

## 목적

Stage 16 커밋 이후 `3-09월_교육_통합_2022.hwp` 12쪽의 `글자처럼 취급` 그림 흐름을 다시 분석한다.

## 기준 커밋

- `70c7da48 task 1139: 12쪽 미주 흐름 정합 보정`

## 작업지시자 피드백

- 12쪽까지의 큰 미주 흐름은 Stage 16으로 커밋한다.
- 현재 남은 문제는 Stage 17에서 다시 시작한다.
- `글자처럼 취급` 그림인데 그림과 본문이 겹친다.
- 두 번째 그림도 같은 문제가 있다.
- 출력물을 만들면 SVG 대신 PNG 미리보기 경로를 우선 보고한다.

## 초기 관찰

한컴오피스 개체 속성 기준 해당 그림은 `글자처럼 취급`이다. 따라서 그림은 본문 줄 높이에 포함되어야 하고, 다음 본문이 그림 영역을 침범하면 안 된다.

Stage 16 후반의 임시 보정처럼 빈 줄을 과하게 압축하면 그림 아래 본문이 그림 영역으로 올라와 겹친다. Stage 17에서는 다음을 분리해서 확인한다.

- 조판 단계의 paragraph 높이 계산
- 렌더 단계의 treat-as-char picture line advance
- `LINE_SEG.vertical_pos` 기반 높이와 실제 그림 높이의 관계
- 미주 다단 흐름에서 그림 문단 직후 `vpos` 보정이 다음 문단을 앞으로 당기는지 여부

## 1차 판단

12쪽의 그래프 두 개는 모두 `글자처럼 취급` 그림이며, 원본 `LINE_SEG`에는 그림이 포함된 줄 다음에 같은 그림 높이를 다시 차지하는 빈 phantom 줄이 뒤따른다.

렌더러와 조판기가 이 빈 줄을 실제 줄로 한 번 더 advance하면 그림과 그림 사이가 과하게 벌어지고, 반대로 모든 빈 줄을 압축하면 후속 본문이 그림 영역과 겹친다. 따라서 `글자처럼 취급` 그림이 직전 줄에 포함되어 있고 현재 줄이 공백뿐이며 현재 줄 높이가 그림 높이와 일치하는 경우만 phantom 줄로 간주해 advance를 제외한다.

## 재현 명령

- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage17_tac_picture -p 12 --show-grid=3mm --grid-origin=9mm,24mm`
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 12`
- `RHWP_DEBUG_PARA_TAC=1 target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage17_tac_debug -p 12 --show-grid=3mm --grid-origin=9mm,24mm`

## 검증 대기

- `cargo fmt`
- `cargo build`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_page12_endnote_shape_picture_properties_resolve_virtual_para_index -- --nocapture --test-threads=1`
- `wasm-pack build --target web --out-dir pkg`

## 개체 속성 진입 추가 분석

작업지시자 확인으로 12쪽 그래프를 우클릭하면 `개체 속성(P)...` 메뉴는 보이지만 속성창이 열리지 않는 문제가 남았다.

기존 Stage 9/10의 `[다른 풀이]` 개체 속성 보정은 다음 구조였다.

- page control layout에는 미주 내부 가상 문단 `paraIdx=518`, `controlIdx=0`, `type=group`으로 노출된다.
- rhwp-studio의 `insert:picture-props`는 `group` 타입을 차단하지 않고 `PicturePropsDialog`로 넘긴다.
- `PicturePropsDialog`는 `group` 타입이면 `getShapeProperties`를 호출한다.
- `getShapeProperties`는 실제 본문 문단뿐 아니라 미주 가상 문단 인덱스를 역해석해 `Control::Shape`를 찾는다.

12쪽 그래프는 선택 참조가 `type=image`로 노출된다. 그러나 원본은 독립 `Control::Picture`가 아니라 미주 가상 문단 안의 `Control::Shape(ShapeObject::Picture)`다.

따라서 rhwp-studio는 `getPictureProperties(0, 651, 1)` / `getPictureProperties(0, 652, 1)`을 호출하지만, 기존 native API는 실제 본문 문단의 `Control::Picture`만 허용해 실패했다.

이번 보정은 그림 속성 조회/수정 API도 Stage 9와 같은 미주 가상 문단 역해석을 사용하고, `Control::Shape(ShapeObject::Picture)`도 그림 속성 대상으로 허용하도록 한다.

## 개체 속성 보정 결과

- `src/document_core/commands/object_ops.rs`
  - `getPictureProperties` / `setPictureProperties`가 실제 본문 문단뿐 아니라 미주 가상 문단 인덱스를 역해석한다.
  - `Control::Picture`뿐 아니라 `Control::Shape(ShapeObject::Picture)`도 그림 속성 대상으로 허용한다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - 12쪽 그래프 두 개가 page control layout에서 `type=image`, `paraIdx=651/652`, `controlIdx=1`로 노출되는지 확인한다.
  - 같은 참조로 `get_picture_properties_native(0, 651, 1)` / `(0, 652, 1)`가 성공하고 `treatAsChar=true`, 크기 값이 조회되는지 확인한다.

## 자동 검증 결과

- `cargo fmt` 통과
- `cargo build` 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_page12_endnote_shape_picture_properties_resolve_virtual_para_index -- --nocapture --test-threads=1` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- WASM 산출물 위치: `/Users/tsjang/Cloud/Devel/rhwp/pkg`

## 그림 탭 자르기 속성 추가 분석

작업지시자 확인으로 첫 번째 그래프의 한컴오피스 `그림` 탭은 다음 값이다.

- 확대/축소: 가로/세로 67.05%
- 그림 자르기: 왼쪽/위쪽/오른쪽/아래쪽 모두 0.00mm
- 그림 여백: 왼쪽/위쪽/오른쪽/아래쪽 모두 0.00mm

rhwp-studio는 같은 그래프를 열 때 오른쪽/아래쪽 자르기를 각각 원본 이미지 크기처럼 표시했다. 원인은 native 그림 속성 JSON이 HWP 내부 `pic.crop.right/bottom` 값을 그대로 내보낸 데 있다. HWP 내부 crop은 UI의 “잘라낸 양”이 아니라 원본 이미지 source rect 좌표이며, crop rect가 원본 전체를 가리키는 경우 한컴 UI에서는 네 방향 자르기가 모두 0으로 보인다.

이번 보정은 `getPictureProperties`에서 source rect를 UI 자르기 양으로 변환하고, `setPictureProperties`에서는 UI 자르기 양을 다시 source rect로 변환해 저장하도록 한다.

## Stage 17 정리

작업지시자 확인으로 12쪽 `글자처럼 취급` 그래프 주변 흐름은 이전보다 개선되었다. 특히 그래프 선택/그림 속성 진입, 그림 탭 자르기 값, `(ⅰ) a>1일 때` / `(ⅱ) a=1일 때` 라벨이 그림에 가려지는 문제를 집중 보정했다.

현재 남은 문제는 다음 Stage 18에서 다시 시작한다.

- 한컴오피스 정답지와 page 12 전체 밀도/하단 흐름이 아직 완전 일치하지 않는다.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 전체 실행 시 `issue_1139_endnote_spacing_reference_files_match_hancom_page_counts`가 아직 실패한다.
- 실패 내용: `3-09월_교육_통합_2024-미주사이20.hwp`가 한컴 기준 24페이지여야 하나 현재 23페이지로 계산된다.
- Stage 18에서는 `미주 사이` 간격이 페이지네이션 높이에 반영되는 지점과, 그림 문단 이후 수식 줄이 줄 단위로 다음 단/쪽으로 넘어가는 기준을 함께 재분석한다.

최근 확인 산출물:

- PNG: `/Users/tsjang/Cloud/Devel/rhwp/output/task1139_stage17_current_png/3-09월_교육_통합_2022_012.png`
- SVG: `/Users/tsjang/Cloud/Devel/rhwp/output/task1139_stage17_current_svg/3-09월_교육_통합_2022_012.svg`
- WASM: `/Users/tsjang/Cloud/Devel/rhwp/pkg`

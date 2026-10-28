# Task M100 #2069 Stage 3: OLE 문단부호 구조 기준 보정

## 목표

`samples/한셀OLE.hwp`와 `samples/한셀OLE.hwpx`에서 문단부호 보기 상태의 OLE 주변 표식을 저장 구조 기준으로 맞춘다.

## 사용자 확인 차이

- HWP/HWPX 파일 구조는 빈 문단 1개와 OLE 1개인데, rhwp가 OLE 미리보기의 행 개수처럼 문단부호 3개를 표시하면 한컴과 다르다.
- 한컴은 파일을 처음 열었을 때 저장된 문단 구조 기준으로 OLE 오른쪽 끝에 문단부호 1개만 표시한다.
- OLE 오른쪽에서 Enter를 계속 누르면, 한컴은 OLE 높이와 겹치는 줄까지만 문단부호/캐럿을 OLE 오른쪽에 두고 이후 줄은 본문 왼쪽 흐름으로 되돌린다.
- OLE 오른쪽 영역 안에서 생성된 빈 문단의 문단부호 간격은 미리보기 행 간격이 아니라 저장된 `LINE_SEG`의 줄 높이와 줄 간격을 따른다.

## 분석 방향

1. HWP와 HWPX 모두 `dump` 기준 빈 문단 1개, `LINE_SEG` 1개, OLE 1개 구조다.
2. `ir-diff samples/한셀OLE.hwpx samples/한셀OLE.hwp --summary`는 문단/LINE_SEG/control 관점 차이 0건이다.
3. OLE preview payload나 내부 시각 행 개수는 문단부호 개수의 근거가 아니다.
4. 문단부호 anchor는 저장된 문단 구조를 기준으로 1개만 생성해야 한다.
5. Enter로 추가되는 빈 문단의 OLE 오른쪽 흐름은 무한 상속이 아니라 `다음 줄 vertical_pos < OLE bottom` 범위 안에서만 유지한다.
6. 비 글자 OLE는 렌더러에서 `PageItem::Shape`로 배치되므로, 빈 OLE host 문단도 저장된 줄 높이만큼 다음 문단 시작 y를 예약해야 한다.

## 수정 계획

- Stage 2의 단일 `push_ole_empty_para_end_anchor` 동작을 유지한다.
- focused test에서 HWP/HWPX 모두 OLE 문단부호 anchor가 1개만 생성되는지 검증한다.
- OLE 높이 안쪽 Enter는 저장된 wrap-zone `LINE_SEG`를 보존하고, OLE 높이를 벗어난 Enter는 일반 본문 빈 문단으로 재조판한다.
- 브라우저 검증에서는 처음 열기/문단부호 토글 상태에서 OLE 오른쪽 표식이 1개인지 확인한다.
- 비 글자 OLE shape 배치 후 다음 문단이 저장된 `line_height + line_spacing` 피치로 내려가는지 검증한다.

## 검증 계획

- `cargo fmt --all -- --check`
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2069_ole_object_selection -- --nocapture`
- `target/release-test/rhwp export-svg samples/한셀OLE.hwp --show-para-marks --show-control-codes`
- `target/release-test/rhwp export-svg samples/한셀OLE.hwpx --show-para-marks --show-control-codes`
- `wasm-pack build --target web --out-dir pkg`
- 기존 `localhost:7700`에서 HWP/HWPX를 각각 열고 문단부호 표식 1개 확인

## 검증 결과

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2069_ole_object_selection -- --nocapture` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
- 기존 `localhost:7700` 런타임에서 `samples/한셀OLE.hwp`를 열고 Enter 6회 상태를 확인했다. 첫 두 빈 줄은 OLE 오른쪽 x에 남고, OLE 높이를 벗어난 뒤에는 본문 왼쪽 x로 돌아간다.
- 런타임 확인값: p0/p1/p2 조판부호 x=513.4, p3 이후 조판부호 x=113.4. y는 저장 줄 피치 기준 132.9, 154.3, 175.6, 195.8 순서로 증가한다.

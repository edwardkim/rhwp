# Task M100 #2069 Stage 2: OLE 편집 표식/속성 한컴 호환

## 목표

`samples/한셀OLE.hwp`와 `samples/한셀OLE.hwpx`에서 OLE 개체 선택 이후의 편집 표식을 한컴 동작에 더 가깝게 맞춘다.

## 사용자 확인 차이

- 캐럿이 한컴처럼 OLE 오른쪽 글줄 높이로 보이지 않고, 현재 구현은 OLE bbox 전체 높이에 묶일 수 있다.
- 우클릭 `개체 속성(P)...`에서 OLE는 한컴처럼 기본/여백·캡션/선 수준의 속성만 수정 가능해야 한다.
- 문단 부호 버튼을 눌러도 OLE가 들어간 빈 문단의 문단 표식이 한컴처럼 보이지 않는다.

## 분석 순서

1. `getCursorRect(0, 0, 0)`의 OLE fallback이 문단 `LineSeg`/글자 모양 기준 높이를 쓰도록 조정한다.
2. Studio에서 OLE 선택 상태에서도 캐럿 overlay를 숨기지 않고, 한컴처럼 OLE 오른쪽에 표시되는지 확인한다.
3. `PicturePropsDialog`의 OLE 타입 탭/저장 필드를 제한해 숨겨진 shape 전용 속성이 변경되지 않게 한다.
4. `show_control_codes` 렌더 경로를 확인해 빈 문단 + 비-TAC OLE에서도 문단 부호가 생성/표시되도록 보정한다.

## 검증 계획

- focused Rust test: `issue_2069_ole_object_selection`
- WASM build: `wasm-pack build --target web --out-dir pkg`
- Studio build: `npm run build`
- 기존 7700 Vite 서버에서 `samples/한셀OLE.hwp` 실제 열기 후 클릭, 우클릭 속성, 문단 부호 토글 확인

## 구현 결과

- OLE RawSvg/placeholder control metadata를 유지해 Studio에서 `type: "ole"` 개체로 선택할 수 있게 했다.
- 빈 문단 OLE의 캐럿 fallback을 OLE bbox 높이가 아니라 문단 `LineSeg`/글자 모양 기준 글줄 높이로 제한했다.
- Studio OLE 선택 상태에서 선택 레이어와 캐럿이 함께 보이도록 보정했다.
- `개체 속성(P)...` 대화상자는 OLE일 때 `기본`, `여백/캡션`, `선` 탭만 노출하고, 회전/대칭/기울이기/채우기/그림자/글상자 전용 저장을 차단했다.
- 빈 문단 OLE 오른쪽에 기본 문단 끝 표식 anchor를 추가했다.

## 검증 결과

- `cargo fmt --all -- --check`: 통과
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2069_ole_object_selection -- --nocapture`: 통과, 2개 테스트
- `rhwp-studio` `npm run build`: 통과
- `localhost:7700` Puppeteer 확인:
  - `samples/한셀OLE.hwp` 로드 성공
  - OLE 클릭 후 `selectedRef.type == "ole"`
  - 선택 레이어 유지, 캐럿 `x=OLE 오른쪽`, `height=13.3px`
  - 우클릭 메뉴 `개체 속성(P)...` 활성
  - OLE 속성 대화상자 탭 `기본/여백/캡션/선`
  - OLE 회전/좌우 대칭/상하 대칭/기울이기 입력 비활성

## Stage 3 이관

문단부호 표시는 아직 한컴과 완전히 같지 않다. 한컴은 OLE 미리보기 내부의 3개 줄 오른쪽 끝마다 문단부호를 표시하지만,
현재 Stage 2 구현은 OLE 오른쪽에 단일 문단 끝 표식만 만든다. 이 차이는 Stage 3에서 OLE 미리보기 높이와
`LineSeg` 점유 높이를 기준으로 가상 줄 끝 표식을 산출하는 방향으로 별도 분석 후 수정한다.

## 커밋 분리

Stage 1 커밋 `3f5d92811`은 OLE preview를 선택 가능한 개체로 노출하는 기반 작업이다. Stage 2는 편집 UI fidelity 보정만 별도 커밋으로 묶는다.

# 실제 API·브라우저·회귀 결과

## R6 실제 공개 WASM API

현재 최적화 WASM을 실제 공개 API 드라이버로 실행해 12개 그룹 PASS, exit 0을 확인했다.
원본 입력은 유지했으며 실제 생성 저장본 HWP/HWPX 30개를 이전 실제 저장본과 SHA-256 대조해 각각 bytes가 같다.
따라서 아래 기존 독립 CFB 검사는 같은 저장 bytes에 한해 재사용한다.
이 검증은 독립 Print 전쪽 시각 확인을 대체하지 않는다.

| 원 PR | 현재 실제 API | 저장·동작의 수정 전 대조 |
| --- | --- | --- |
| [PR #8526](https://github.com/edwardkim/rhwp/pull/8526) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8528](https://github.com/edwardkim/rhwp/pull/8528) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8541](https://github.com/edwardkim/rhwp/pull/8541) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8558](https://github.com/edwardkim/rhwp/pull/8558) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8564](https://github.com/edwardkim/rhwp/pull/8564) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8570](https://github.com/edwardkim/rhwp/pull/8570) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8571](https://github.com/edwardkim/rhwp/pull/8571) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8572](https://github.com/edwardkim/rhwp/pull/8572) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8586](https://github.com/edwardkim/rhwp/pull/8586) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8602](https://github.com/edwardkim/rhwp/pull/8602) | PASS | base passed → 후보 passed; 원 결함 검출 불가, 아래 독립 레코드 검사 사용 |
| [PR #8616](https://github.com/edwardkim/rhwp/pull/8616) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |
| [PR #8629](https://github.com/edwardkim/rhwp/pull/8629) | PASS | base failed → 후보 passed; 원 결함 검출 가능 |

재현: `node mydocs/pr/assets/semanticist21_20261105/validation-r5/api/claim-probe.mjs <최적화 패키지 절대 경로> <output 아래 결과 경로> <제품 SHA>`.
드라이버의 fixture-map.json은 런타임 입력을 지정하는 정식 소스 계약이므로 보존한다.
그림·탭 이동 PR #8549는 이 12개 그룹이 아니라 별도 Native 생성·회귀·독립 Print 증거로 확인한다.

### PR #8602: 독립 CFB 원시 레코드

`olefile` CFB 읽기와 표준 zlib·struct 파싱으로 RHWP 파서·저장기를 쓰지 않고 대조했다.
기준의 대상 빈 문단은 char_count=1, PARA_TEXT=[13], LineSeg=[0]였다.
후보는 char_count=1·LineSeg=[0]을 보존하고 PARA_TEXT를 생략한다.
RHWP 재열기만으로 원래 손상을 검출했다고 주장하지 않는다.

- base 저장본 SHA-256 `9a5df8a64e6eff354fa2a21617f3b855e9d4120dcc76bf30b925fccb02491729`.

- candidate 저장본 SHA-256 `a2ec7d4fe6dcb4520a113693dd615ac654fc7a8ff47c63c1d3d7bd9cc37c3cf5`.

독립 재현: `venv/bin/python mydocs/pr/assets/semanticist21_20261105/validation-r5/api/read-hwp-paragraph-records.py <기준 HWP> <후보 HWP>`.

### PR #8564: Print 파일명 필드 문맥

독립 Print는 한컴 2020 MCP 11.0.0.9136, PrintMethod=0, one-up,
input_preprocess=scripts_disabled다. 두 입력의 Scripts는 같은 빈 기본 스텁이며 실행 코드가 없다.
한컴은 파일명 필드를 `rhwp-no-scripts-*.hwp`로 다시 평가하고 RHWP는 저장된 `.result.*.partial.hwp`를 표시한다.
이 문자열 차이를 숨기거나 위치 개선으로 주장하지 않는다. 필드 앞뒤의 ‘앞’·‘뒤’와 뒤 본문 C는 보존된다.
원 주장인 경계는 같은 원본의 실제 API가 기준에서는 필드 값에 문자를 포함해 FAIL,
후보에서는 필드 값 밖에 남겨 PASS함으로 검출한다.

- before: Print job `6dae1c8f-31d2-45fa-a7b7-c60f6542bcab`, 입력 SHA `9a2edc8583033f54037722111827ffffb2496047b9c9a168086f61f029d09a7b`, PDF SHA `3ffbc8328d91a0a3121efaf324e63cb7cd2e9dcf3b9b4a4c78b40d380239db17`.

- after: Print job `59c95a85-eeed-4b30-a29e-e8b3404b0069`, 입력 SHA `122d77da16b50d15906833806871f5046541d89c4a2a00fe7d4941cc569227a4`, PDF SHA `a25f0c071774fcdd6b4f7d04772d3fb76220c88eb57404450555ae487e312608`.

## 실제 브라우저와 production build

현재 public 패키지의 실제 네트워크 WASM bytes가 producer SHA와 같음을 확인했다.
스타일 모달의 -1 실패 반환 때 상태·undo 보존, 정상 생성·undo가 PASS다.
새 base PR #8630의 실제 그룹 클릭·문자 입력·undo/redo·Esc도 PASS다.
잘못된 입력의 주입은 소비 경계 검사이며 자연 발생 문서 손상으로 표현하지 않는다.
`npm run build`의 TypeScript·Vite production build exit 0.
실제 WASM export와 두 public 선언 파일 각각 631개 export의 존재·인수 수 검사 PASS,
생성 타입 소비 Node 10건 PASS.

전체 Node 2,723 PASS·실패/취소/skip 0의 이전 실행을 재사용했다.
해당 실행 뒤 Studio/editor/shared/scripts 변화는 같은 선언 행의 순서만 바뀐 public 타입 두 개이며
JS glue SHA도 같다. 현재 ABI·소비·실제 API·브라우저를 다시 실행했다.
이전 2,723건을 현재 source에서 새로 실행한 것으로 표현하지 않는다.

## 집중 회귀와 전체 검증의 구분

빈 번호 머리: 수정 전 원본 HWP/HWPX 2 FAIL, R6 집중 184 PASS,
정식 원본·0 거리 대조·독립 원점 등 기존 3+신규 5의 총 8건 PASS, exit 0.
Mac 네 원본의 전체 1쪽·내용 계약을 보완한 관련 6건 PASS, exit 0.
모든 회귀 실행에서 bwrap으로 저장소 공개 글꼴 5개만 실제 노출했다.
이전 R5 전체 12,735 PASS·45 ignored는 이전 제품 결과이며 R6 전체 실행의 완료가 아니다.
최종 코드의 공식 편람·R6 전체 Rust·lint·Native Skia·release·doctest는 완료 뒤 별도로 확정한다.

### 전체 Node 재사용의 실제 식별자

- 이전 실제 전체 Node 로그 SHA-256 `0c2e9eb2cf0f3f95f29b8b462bff7cf95b53cae7f9a973d75de701bd90da2ff1`.
- 비교한 이전 frontend source SHA `af0505ce2b863f4712ab1da3a101c35cee71ca30`.
- 해당 source 이후 현재 frontend·테스트·npm/editor·shared·scripts의 변화는 public 선언 두 개다.
- `rhwp-studio/public/rhwp.d.ts`: 실제 SHA-256 `3b13b78ffee6847681d4d9a8ba4acd5359d9c22dbec1eafdcb4c8541f2a5340c`, 같은 선언 행이며 순서만 다름.
- `rhwp-studio/public/rhwp_bg.wasm.d.ts`: 실제 SHA-256 `fb02e9d0706c033df6dbce1699d2ae065f9f50bbeff128d966be3ebef9a294c5`, 같은 선언 행이며 순서만 다름.
- JS glue SHA-256 `2a099009a9582a9b34cccb3d61f41a1e4d21085eefd85b0ad8df466b18766138`.

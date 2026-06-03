# Stage 5 재분류 — Task #1139

## 배경

작업지시자가 Stage 4 보정 후에도 한컴오피스와 완전히 다르다고 재보고했다.
Stage 4의 분수 높이 괄호 glyph 실험은 문24 일부에만 영향을 주며, 제시된 화면의 주요 차이를 충분히 설명하지 못한다.

## 새 관찰

`output/diag_1139_stage4/3-09월_교육_통합_2022_005.svg`를 확인한 결과, 우측 문27 줄 끝에 반복되는 검은 기호는 텍스트나 수식 명령 노출이 아니라 같은 작은 `Picture` 컨트롤이다.

- 위치 예: `x=690.52 y=147.13 width=23.8 height=17.6`
- 동일 이미지가 문27 단락 내부에서 여러 번 반복된다.
- 한컴 캡처에서는 해당 지점이 편집 표시/문단 흐름과 다르게 보이므로, 수식 괄호보다 이 inline picture 처리 정책을 먼저 재검토해야 한다.

## 판단

Stage 4의 괄호 glyph 변경은 현재 피드백의 본질과 맞지 않아 커밋하지 않는다.
다음 단계에서는 작은 inline picture가 실제 인쇄 대상인지, 편집 보조/제어 표식 성격인지 식별하고, 한컴 화면 기준으로 렌더링 여부와 baseline 정렬을 조정한다.

## 다음 액션

1. 문27에 배치된 작은 `Picture` 컨트롤의 속성(`CommonObjAttr`, `Picture` crop/bin/effect)을 덤프한다.
2. 같은 base64 이미지의 반복 조건과 일반 그림(`문26`, 도표) 조건을 분리한다.
3. 필요 시 해당 유형은 렌더링을 생략하거나 별도 제어표시 옵션에서만 보이도록 한다.
4. Stage 4 괄호 glyph 실험 변경은 되돌리고, 새 보정만 별도 검증한다.

## 확인 결과

진단 예제로 원본 direct `Picture` 컨트롤과 렌더 트리 `ImageNode`를 비교했다.

- 원본: `pi321 ci10`, `pi323 ci4` 두 개만 존재한다.
- 기존 렌더 트리: `pi321 ci10` 3회, `pi323 ci4` 2회 출력됐다.
- 원인: `paragraph_layout.rs`의 run 종료 후 fallback이 현재 줄 뒤쪽의 미래 TAC까지 매 줄마다 렌더했다.
- 조치: fallback에서 현재 line range 밖(`tac_pos > line_end_char`)의 TAC는 건너뛰도록 제한했다.

## 검증 결과

- `issue_1139_small_inline_picture_rendered_once_per_control` 신규 회귀 테스트 추가
- page 5 SVG 재생성 후 작은 `23.8x17.6` inline picture가 5개에서 원본 컨트롤 수와 같은 2개로 감소함을 확인
- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate`: 1 passed
- `cargo build --release`: 성공
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_stage5`: 성공
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored
- `wasm-pack build --target web --out-dir pkg`: 성공

# #7333 Stage 6 — 공통 꼬리말 원점

## 분석

한컴 2020 PDF와 Native full Visual Sweep을 96 DPI로 대조했다. 2·9·31·40~44·47쪽에서
꼬리말의 파란 선은 rhwp `y=997px`, PDF `y=1003px`이고, 왼쪽 로고는 rhwp `y=1036px`,
PDF `y=1043px`이다. 같은 꼬리말의 자동 쪽번호 glyph도 `y=1010px` 대 `y=1016~1017px`로
반복된다. 따라서 페이지별 도형 문제가 아니라 footer 공통 원점 또는 저장 줄 좌표의 소비 문제다.

이 회차는 이 공통 수직 기준만 다룬다. 31·40~44·47쪽의 본문 도형 위치는 footer 기준을 고정한
다음 회차에서 별도의 저장 frame·앵커 계약으로 다룬다.

## 독립 기준

`pdf/issue7333/aaaaaa-2020.pdf`는 `samples/issue7333/aaaaaa.hwp`의 한컴 2020 출력이다.
PDF raster에서 첫 footer 파란 rule의 상단은 2·31쪽 모두 `1003px`이며, render tree의 그림
상자는 PNG 잉크보다 0.3px 위에 있으므로 회귀 기대값은 `1002.7px`로 둔다.

저장 HWP5 footer의 세 문단은 각각 한 줄이고 `vpos=0·1352·2704 HU`,
`line_height=900 HU`, `line_spacing=452 HU`다. `list_attr=0x00400000`는 `BOTTOM`이며,
grid는 선언 `text_height=5669 HU` 안에서 닫힌다. 현재 그림 혼합 경로는 이 grid의 첫
`line_spacing`을 버려 rule이 `996.7px`에서 그려진다. 수정 전 PDF 기준 회귀는 이 값으로
실패했다. 이 6.03px 선행은 동일한 완전 grid·BOTTOM·선언 높이 조건에서만 적용한다.

## 진행 중인 보정

footer 공통 원점 보정 뒤 새 Native build에서는 파란 rule과 쪽번호의 공통 수직 오차가 해소됐다.
다만 왼쪽 로고의 실제 잉크 영역은 PDF와 아직 미세한 차이가 있어 이 회차를 완료로 판정하지
않는다. 그림 frame과 투명 여백을 분리해 다시 측정한다.

31쪽은 별도 원인이 확인됐다. 둘째 빨간 화살표(`p430/c4`)의 HWP5 `renderingInfo`는
`[-0.896, 0, 7508; 0, 86.490, 0]`인데, top-level 직선 경로는 과거에 frame 폭·높이의
양수 비율만 적용해 부호와 이동을 버렸다. 따라서 PDF의 오른쪽 위→왼쪽 아래 화살표가
rhwp에서 왼쪽 위→오른쪽 아래로 뒤집혔다. 직선 양 끝점에 전체 signed affine을 적용하고,
31쪽 긴 빨간 화살표의 방향과 끝점에 대한 SVG 회귀 검사를 추가했다. 이 수정 뒤 큰 화면 그림,
두 화살표, 사각 주석, 표를 하나의 p31 overlay로 다시 판정한다.

31쪽 스크린샷은 `treat_as_char` 표 셀 그림의 저장 frame `48190×38370 HU`를 cell 안쪽
폭으로 축소하던 경로도 통과하고 있었다. 한컴은 이 그림을 원래 frame으로 그리고 셀 경계에서
clip한다. 따라서 셀 문맥의 글자처럼 그림만 축소를 건너뛰고, 원래의 비글자 그림·회전 그림
container 제한은 유지한다. render tree에서 `642.5×511.6px` frame을 확인하는 회귀 검사를
추가했다.

이후 31쪽 인쇄 창은 frame 보존과 별개로 20.8px 아래에 남아 있었다. 표 셀 inline 경로가
줄 안 개행 판정을 위해 폭에 맞춰 줄인 `clamped_h`(약 487px)를 baseline 보정에 사용한 반면,
실제 paint 경로는 저장 frame(511.6px)을 사용한 것이 원인이다. 저장 `text_height`와 원본
그림 높이가 같고 caption·바깥 세로 여백이 없는 경우에는 이 축소값이 아니라 저장 frame을
기준으로 full-line을 판정해 baseline 보정을 0으로 둔다. 좁은 셀의 line-wrap 판단과 실제
frame clip 계약은 그대로 유지한다.

## 검증

- `CARGO_TARGET_DIR=target/pr-review cargo test --profile release-test --test regression_suite_020 issue_7333_overlapping_picture_lines -- --nocapture`: 7 passed.
- `issue7333-stage6-p31-inline-frame` Native Visual Sweep: 31쪽 complete, visual flag 없음.
  이전 스크린샷 영역 ECC의 세로 보정은 약 `+21.3px`이었고 수정 후 전체 페이지 변환은
  `x=+0.19px`, `y=+0.20px`이다. overlay `pixel_match=95.271%`, `ink_match=76.008%`.
- `issue7333-stage6-p08-after-p31` Native Visual Sweep: 8쪽 complete, visual flag 없음.
  상단 안내 창·설치 창·빨간 화살표의 위치가 한컴 2020 PDF와 같은 저장 frame에 놓였고,
  overlay `pixel_match=96.659%`, `ink_match=56.777%`이다.
- `rhwp-studio` 선택 정책 test 3건과 `npx tsc --noEmit` 통과. 표준 root wrapper WASM build로
  `pkg/`와 `rhwp-studio/public/`의 `rhwp.js`·`rhwp_bg.wasm` SHA-256 일치 확인.

## 8쪽 객체 선택

8쪽 `p141/c0` 사각 주석과 `p141/c1` 화살표는 `p141/c2` 표 셀의 스크린샷과 겹친다.
Studio의 기존 선행 hit-test는 같은 문단에 Shape가 하나라도 있고 cellPath 그림이 겹치면
그림을 무조건 먼저 반환했다. 글상자 control이 자기 내부 그림을 감싸는 경우에는 필요하지만,
독립 전경 도형까지 같은 컨테이너로 보면 보이는 주석을 선택할 수 없다. cellPath 조상인
글상자 Shape에만 기존 picture 우선을 유지하고, 경로 밖의 전경 Shape는 먼저 선택하도록
구분했다.

## 결과

footer의 direct Picture 두 개를 render tree로 분리해 확인했다. 오른쪽 `Column/Para` 그림은
`para_y`를 쓰므로 이미 leading을 받았고, 왼쪽 로고만 `Paper/Paper` 기준이라 `para_y`를 무시했다.
따라서 `BOTTOM`·선언 `textHeight`·완전한 한 줄 HWP5 grid가 모두 성립하는 footer에서만,
`Paper`/`Page` direct Picture의 header/footer 틀 원점을 첫 줄 leading(452 HU = 6.03px)만큼
내렸다. `Para`/`Column` 그림은 이 경로에서 제외해 leading을 중복 적용하지 않는다.

수정 뒤 31쪽 render tree는 다음과 같다.

| 항목 | 수정 전 | 수정 후 | 한컴 2020 PDF 기준 |
| --- | ---: | ---: | ---: |
| footer rule frame y | 996.7px | 1002.7px | 잉크 상단 1003px |
| Paper 기준 좌측 로고 frame y | 1021.1px | 1027.1px | 로고 잉크 상단 1042px |
| 좌측 로고 잉크 상단 | 1036px | 1042px | 1042px |

31쪽 인라인 스크린샷의 full frame 기준선 보정과 footer 로고 보정을 함께 적용한 뒤,
2·31쪽 overlay에서 rule·쪽번호·로고가 같은 위치에 겹치는 것을 직접 확인했다. 선택 쪽
(8·13·14·16~23·31·33·36·40~44·47)의 overlay contact sheet도 확인했다. 이는 이번
저장 frame·본문 흐름·footer 원점 범위의 시각 판정이며, 전체 pixel/ink 비율만으로 문서 전체
fidelity를 단정한 결과가 아니다.

## 최종 검증

- `CARGO_TARGET_DIR=target/pr-review cargo test --profile release-test --test regression_suite_020 issue_7333_overlapping_picture_lines -- --nocapture`: **7 passed, 0 failed**.
- `cd rhwp-studio && node --test tests/picture-hit-policy.test.ts && npx tsc --noEmit`:
  **3 passed**, TypeScript 오류 없음.
- 저장소 루트에서 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`
  완료. `pkg/rhwp.js` ↔ `rhwp-studio/public/rhwp.js`, `pkg/rhwp_bg.wasm` ↔
  `rhwp-studio/public/rhwp_bg.wasm`의 SHA-256이 각각 일치한다.
- `issue7333-stage6-footer-paper-leading` Native Visual Sweep: 2·31쪽 complete, 자동 visual
  flag 없음. 대표 31쪽 overlay에서 footer rule과 좌측 로고의 PDF 잉크 위치를 직접 확인했다.
- `issue7333-stage6-all-pages` Native Visual Sweep: **50/50쪽 complete**, 누락 0, 자동 visual
  flag 0. 평균 pixel match 94.508%, 평균 ink match 61.588%이며, 최저값과 raster glyph·기존
  이미지 차이는 전체 fidelity 통과 수치로 해석하지 않고 위의 해당 요소별 좌표·overlay로 판정했다.

## 다음 회차

Stage 6은 footer 공통 원점, Paper 기준 로고, 31쪽 inline screenshot baseline, Studio의
page-local picture 재선택 범위를 완료했다. 다음 회차가 필요하면 새 `_stage7.md`에서 남은
개별 도형의 저장 frame·selection 편집 동작을 독립 분석으로 시작한다.

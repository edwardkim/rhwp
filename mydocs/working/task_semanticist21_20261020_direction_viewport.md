# semanticist21 누적 검토: 글줄 방향과 저장 그림 영역

## 현재 판정

중간 구현 단계다. Native 진단과 Studio 계약 검증을 마쳤으며 통합 PR 제출은 보류한다.
fresh WASM 전쪽·확대 판독, 전체 Rust 및 #8064 기능 보존의 최종 head 검증이 남았다.
이 문서는 원 PR별 최종 review·오늘할일 trailing 기록을 대신하지 않는다.

## 범위와 기준

- 브랜치: `review/semanticist21-20261019`. 고정 base `5e2b9bfbf08573a31fa1e11a2445ff63fc0755c0`.
- 원격 재조회: 열린 PR 45개, Draft 10개. 기존 44개 head 변경 없음. 새 #8096의 원 head
  `67acb6d0562dc3b8df04e4a2ebbc6bf39ea9ba07`를 `65c2404187b32329413db18f925e7c07363d4030`로 체리픽했다.
- #8096 reviewer `jangster77` 요청을 GitHub API로 재확인했다. 원격 push·새 PR·comment·merge는 하지 않았다.
- 독립 Print는 공개 MCP client의 `--engine 2020 --target pdf`, PrintMethod0 one-up 경로다.
  실제 제품11.0.0.9136과 입력/PDF SHA는 아래 output receipt에 보존한다.

## 원인과 공통 결과

### 글줄 방향

같은 줄의 서식·원문 런을 각각 Bidi 해소해 `A مرحبا 123 Z`의 숫자와 아랍어 순서가 달랐다.
TextLine에서 한 번 해소한 방향 구간을 표시 scalar 주소에 투영한다. 원문 주소·논리 전진과
기존 run 소유는 유지한다. 글자·배경·장식·그림자·편집 표식이 같은 표시 원점을 사용한다.

생산은 `layout.rs → text_direction::resolve_page → TextStyle.bidi_layout`, 소비는
`skia/text_replay.rs`, `web_canvas.rs`, `svg.rs`, nominal `paint/font_glyph.rs`와
`paint_bounds.rs`다. CanvasKit legacy complex-script는 기존 명시적 fallback을 사용하며,
producer가 위치를 확정한 GlyphRun은 직접 replay한다. 제품 FontResolver의 별도 exact shaping은
현재 구현이 없으며 테스트 resolver만 있어 해당 경로를 검증 완료로 주장하지 않는다.

### 저장 그림 영역

독립 Print의 원점 이동·반전·회전·안쪽 여백·외곽선·절반 폭 대조군에서 viewport와 bitmap
자체 원점이 다른 것을 확인했다. viewport 폭을 줄이면 bitmap도 축소된다. imgRect 꼭짓점
순서를 flip으로 추정하지 않으며 실제 저장 행렬의 방향은 기존 공통 변환을 사용한다.

생산은 `picture_source_rect → ImageNode.paint_geometry`. bitmap 출력 상자와 외곽선 viewport를
분리하고, 실제 회전 다각형 clip을 페이지 좌표로 Native/Canvas2D/SVG/JSON/DOM/CanvasKit에
전달한다. 논리 layout frame과 hit-test 소속은 바꾸지 않는다. 처음의 AABB clip 가정은 실제
PDF W* 경로와 반례로 기각했고 관련 이전 제품/결과를 보존했다.

## 실제 실행

증적 root는 ignored `output/pr-review/semanticist21-20261019/`다. 로그·중간 JSON은 커밋하지 않는다.

| 실행 | 관측 결과 | 증적 파일 |
| --- | --- | --- |
| Native 032 빌드 | PASS, 228.12초, 빌드 잡 8개, 공용 target/pr-review | `char-overlap-native-build-032-result.json` |
| Native 033 빌드 | PASS, 209.83초, 빌드 잡 8개, 공용 target/pr-review | `char-overlap-native-build-033-result.json` |
| 기존 정상 61조건 Native 031→032 |61/61≥90%, PNG61/61동일, 0.1%p 이상 하락 0건 | `native031-native032-preservation-138.json` |
| 기존 정상 61조건 Native 031→033 |61/61≥90%, 0.1%p 이상 하락 0건 | `native031-native033-preservation-139.json` |
| 새 경계 22조건 Native 031→032→033 |6→15→18조건≥90%; 숫자·괄호 혼합은 보정 후100% | `layout-boundary-138-viewport-fixed-native/results.json` |
| DOM 그림 전달 합성 계약 2건 |수정 전2 FAIL→수정 후2 PASS | `logs/authored-image-viewport-{before,current}-138.log` |
| 정식 같은 TS 계약 검사 |2 PASS /0FAIL | `logs/authored-image-viewport-formal-139.log` |
| Studio TypeScript |수정 후 exit 0 | `logs/studio-typescript-viewport-139-final.log` |

Native 033 source snapshot과 현재의 차이는 TS 테스트 import 이름 수정 한 건뿐이다.
`native033-frontend-test-only-equivalence-139.json`에서 제품 해시와 모든 Rust·Cargo 소스의
일치를 확인한다. Native 033이 수정 후 TS 검사를 실행했다고 쓰지 않는다.
확대 review와 standalone overlay는 `layout-boundary-panels-137/`에 함께 보존했다.

최신 Studio 전체 검사도 2,354 PASS / 0 FAIL / 2 SKIP(전체 2,356건)다.
`logs/studio-viewport-all-139.log`에 보존했다.

## 남은 필수 항목

- 수동 imgRect 진단 4개는 한컴 clip/bitmap 접경의 잔여 선 때문에 여전히 90% 미만이다.
  입력 가정·실제 Print clip·래스터를 대조하며 기대값이나 threshold는 완화하지 않았다.
- #8096 원 재현 생성 함수를 그대로 실행한 HWP/HWPX와 각각 독립 Print를 확보했다.
  Native 033은 HWP 60.05486%, HWPX 74.37026%다. HWPX 개체 소실은 개선됐으나 배경/프레임
  소비와 HWP 저장 줄 높이 차이가 남아 수용 완료가 아니다.
  `pr8096-current-138-viewport-fixed-native/results.json`, `mcp-pr8096-137/`을 따른다.
- 최종 head의 fresh WASM·Native 전쪽, 직접 확대 판독, 전체 Rust/lint/Studio/browser와
  #8064의179조건·2,082쪽/각 backend 기능 보존을 완료한다. 기존 점수나 CI로 대체하지 않는다.
- 모든 필수 검증을 마친 구체적인 후보로 원격 PR 승인을 받은 뒤 제출한다.

## 추가 확인

WASM32 라이브러리 Clippy `-D warnings`도 exit 0(91초)이다.
`logs/wasm32-layout-clippy-139.log`에 보존했다.
미달 수동 그림 4개의 bitmap/viewport 교차 면적은 모두 0이었다. 세 입력은 독립 Print의
어두운 픽셀이 3~13개뿐이고, 회전 입력 하나는 경계선 140개다. 세로 접경 대조군 역시
교차 면적 0이나 Native/PDF의 래스터 위상 때문에 raw 100%다.
`degenerate-source-clip-observations-139.json`의 원본 좌표·SHA·픽셀 관측으로
실제 그림 면적의 누락과 구분한다. 시각 threshold나 기존 기대값은 바꾸지 않았다.

#8096에서는 HWPX serializer의 기본 BorderType 0과 HWP serializer의 타입별 기본값
(버튼 4, 콤보/입력 5)이 다르다. XML의 drawFrame은 이미 1이므로 boolean 철자 문제로
추정하지 않는다. renderer가 BorderType 0을 무시하는 경로와 체크/라디오의 OPAQUE
배경 미소비를 원본 속성·독립 Print와 대조해 다음 단계에서 보정한다.

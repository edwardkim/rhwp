# #7642 Stage 85 — 원점과 수학 축을 공유하는 연산자 표시

Issue: #7642

## 독립 기준과 공통 원인

Stage 84의 CanvasKit·Native 확대 판독에서 등호의 가로선 높이가
다르다. 논리 폭은 HYhwpEQ의 수학 전진을 사용하지만 실제
표시는 Latin Modern Math의 기호와 수학 축을 그대로 사용한다.
같은 계약 불일치가 덧셈과 뺄셈에도 적용된다. 줄·논리 상자의
높이와 실제 잉크의 원점을 구분해야 한다.

20개 크기별 NOT Print에서 등호 U+E047의 전진은 795,
잉크 범위는 (57,164,738,403), UPM 1024다. 별도 기존
`binary-operator-probe-2024.pdf`의 수학 덧셈 U+E048은
(57,-58,738,623), 전진 795이고 뺄셈 U+E046은
(57,267,718,308), 전진 774다. 본문 ASCII 기호의 메트릭은
다르므로 수학 PUA와 대용으로 쓰지 않는다. 이 PDF와 원본은
Stage 61의 MCP 정상 저장·Print 기록을 따른다.

공개 Latin Modern Math의 해당 윤곽을 독립 수학 잉크 프레임에
배치한다. 사설 글꼴에서는 범위·전진·변환의 메트릭만 소비한다.
원 저작자와 GUST 라이선스를 보존하며 사설 윤곽을 코드·자산으로
복제하지 않는다. 상자 크기·기준선·연산자 자동 간격은 유지한다.
표시 원점은 기존 600dpi 물리 인쇄 격자와 명목 글자 크기에서
계산한다. 문서 ID·절대 화면 px에 대한 분기를 추가하지 않는다.

## 실제 생산·소비 경로

`layout/source_font_symbol`이 같은 공통 FontGlyph 윤곽을 발행하고
SVG·Canvas·Native Skia·paint JSON·CanvasKit이 기존 윤곽 표시
경로로 소비한다. NOT는 등호 본체가 FontGlyph여도 같은 전진과
사선 기하를 사용한다. 한컴 수학 face가 아닌 경로와 굵은 기호는 기존 표시를
보존한다. 보통 굵기의 독립 Print 메트릭을 굵은 기호에 적용하지 않는다. 같은 개체 소속과 다음 내용의 원점을 대조한다.

## 검증 범위

원점·축 변화가 보인 등호와 같은 원인의 덧셈·뺄셈을 함께
보정한 뒤 한 번에 빌드·판독한다. 정상 기호, NOT와 앞뒤 변수,
6/9/12/18pt, 다른 face를 확인한다. 관련 모든 페이지의 Native/
fresh WASM 90% 이상과 확대 판독 뒤에만 정식 회귀를 추가한다.
검증 중인 Stage 83의 전체 원본 55개·853쪽은 고정 바이너리·
package로 변경 전 증거를 보존하고 새 source의 통과로 승격하지 않는다.
새 후보의 집중 검증과 영향 전체 원본 검증을 별도로 기록한다.

대문자 `W`의 독립 Print shear 0.333과 실제 face 공급의 잔여
차이는 아직 미완료다. 이번 연산자 표시 보정으로 해결했다고
판정하지 않는다. 관련 회귀의 관계형 전환과 최종 게이트도 남아 있다.

## 후보 구현

공개 `plus`·`minus`·`equal`의 윤곽과 출처를
`operator_outline.rs`에 보존했다. 실제 수학 잉크 범위·기준선·
인쇄 원점은 같은 글자 크기와 출력 DPI로 계산한다.
`source_font_symbol::layout_operator_outline`이 논리 상자의
폭·높이·기준선을 유지한 채 FontGlyph 표시 결과를 발행한다.
굵은 기호와 다른 face는 이 보통 굵기 메트릭을 사용하지 않는다.

뒤 소비도 대조했다. `layout/symbol_spacing.rs`는 Symbol과
FontGlyph를 모두 같은 AST 연산자의 전진·여백으로 처리하며,
`logical_font_frame.rs`의 font_line·prime 판별도 FontGlyph를
지원한다. NOT의 등호 판별은 FontGlyph를 함께 인식하도록
바꿨다. 실제 사선 기하의 계산·논리 크기를 변경하지 않는다.
기대값·baseline과 정식 실물 회귀는 변경하지 않았다.

## 확인한 실행 결과

- 검증 source는 `acf766cc9b8d4e3548261bdbe0373b84b6079709`다.
  고정 Native 바이너리 SHA-256은
  `f3594ef7dbd0651d1d770a58fbf5a3e2e57b65d20f03c75a03bfe506a1df28ac`이며,
  `--profile release-test --target-dir target/pr-review --bin rhwp` 빌드는
  exit 0이다. fmt와 Native Clippy도 exit 0이다. 나머지 필수 lint와 전체 회귀를
  통과한 결과로 확대하지 않는다.
- 저장 크기별 NOT 원본 HWPX의 300dpi 전체 한 쪽은 같은 독립 Print와
  94.57865% → 95.41683%다. 6/9/12/18pt 등호의 가로선은 Print 높이에
  가까워졌으며, 사선·다음 본문의 물리 위치도 함께 판독했다.
  `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`
  아래 `stage85-operator-origin-selected-detail-review.png`와 provenance가
  같은 입력·물리 영역의 수정 전/후/Print/overlay를 연결한다.
- 루트 WASM wrapper는 exit 0으로 완료했고 고정 package의 JS/WASM과
  Studio public 파일 SHA가 일치한다. source별 Native/fresh WASM 집중 검증은
  계속 진행 중이다. 실제 Studio CanvasKit software surface로 정상 저장 HWP를
  300dpi Print 프로필로 출력하고 공식 HCRDotum을 공급한 결과도
  94.58213% → 95.61075%다. 렌더 오류·글꼴 로드 실패·등록되지 않은 fallback은
  없고, 4개 중괄호와 4개 등호가 공통 윤곽으로 재생된다.
  `stage85-canvaskit-operator-frame/receipt.json`과 `comparison.json`이
  source·WASM·frontend·입력·실제 PNG의 출처를 고정한다.
  `stage85-canvaskit-operator-selected-detail-review.png`는 같은 저장본의
  이전 실제 CanvasKit 출력/현재 출력/Print/overlay다. 대문자 W의 차이는 남아
  있으며 이 한 쪽 통과를 원본 전체나 다른 backend의 통과로 확대하지 않는다.
- 변경 전 source `cd1b2b029`의 전체 원본 검증에서 9월 2024 HWPX
  `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx`의 21쪽은
  89.39422%다. 같은 입력·독립 Print를 이번 source로 재출력한 결과도
  89.89297%로 미달이다. 쪽수는 양쪽 23쪽으로 일치한다.
  `stage85-september-hwpx-current-review/.../review/review_021.png`를 직접
  확인했으며 문29 앞 간격 차이는 다음 단계에서 원인을 보정한다.
  이 원본을 통과 목록으로 집계하거나 관계형 테스트의 선행 조건으로 쓰지 않는다.

Native/fresh WASM 전체 55개·853쪽의 최신 source 검증, 대문자 W의
표시 차이, 관련 기존 검사의 관계형 전환과 최종 게이트는 미완료다.
이 기록은 부분 개선의 증거이며 PR 제출·전체 완료 판정이 아니다.

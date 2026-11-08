---
kind: plan
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-07
---

# PR #7600 메인터너 보정 실행 기록

> 이 문서는 원 PR 검토 중의 로컬 보정 이력이다. 원 PR #7600은 2026-10-07에
> `22ddc2ab81c405b0c3e9b1f25a397a87de109948`으로 병합됐다. 이하의 미완료·보류 표현은
> 당시 code head에 대한 기록이며 현재 원 PR의 상태를 뜻하지 않는다.
> 새 원인의 수식 비율·페이지 배치 보정은 [#7642 1단계](../../working/task_m100_7642_stage1.md),
> 별도 재저장본 문제는 [#7640](https://github.com/edwardkim/rhwp/issues/7640)에서 진행한다.


## 범위와 독립 근거

사용자가 메인터너 보정을 승인했습니다. 별도 과거 79.8%/64.3% 자료는 없다고 확인했습니다. 공개 #7593 원본은 확보했고 MCP engine2020의 Print PDF(job `c0af74c9-1bcf-454d-b177-08ab91b24e59`, Hancom 11.0.0.9136, pdf_print_method 0, hancom2020_pdf_driver_one_up)와 새 Native 전체 1쪽에서 79.58463%를 재현했습니다. 이 원본을 삭제/축소하지 않고 같은 독립 Print 기준으로 보정합니다. 미제공 문서와 같은 파일이라고 추정하지 않습니다.

## 단계

1. 원 기여 체리픽 `5d7a2ad98`, 원시 TSV local output 분리 `08abab871`와 초기 리뷰를 보존합니다.
2. 공개 원본/Print PDF를 커밋하고 마스터 쪽 colPr 파싱 → MasterPage.paragraphs의 ColumnDef → build_master_page의 실제 paint 소비 경로를 고칩니다. 본문 단 구분선의 기존 column 영역 계산·선 스타일 결과를 재사용하고 바탕쪽 프레임 전체 높이를 적용합니다. 문서 전용 좌표·문서 ID·임계값 예외는 추가하지 않습니다.
3. 공개 원본을 먼저 재출력해 90%와 내용/경계 직접 판독을 확인합니다. 새 회귀는 독립 Print·관계 기반이고 그 선행 조건을 통과한 뒤에만 추가합니다. 기존 원 기여 테스트는 삭제하지 않습니다.
4. PR 최소 입력은 MCP가 기본 빈 OnDocument_New 스크립트 때문에 무인 변환을 거절했습니다. 매크로 실행·보안 설정 변경은 하지 않습니다. 기본 빈 템플릿을 정적으로 확인해 스크립트 없는 별도 검증 사본을 만들 경우 변경 entry를 공개하고 모든 조판/본문 XML의 동일성 및 Native 출력 동일성을 입증합니다. 원본은 보존합니다.
5. Native/fresh WASM과 바탕쪽 관련 대조군·정식 회귀 red/green, fmt/native/WASM32/workspace Clippy·base 정책·필요 전체 회귀를 순차 검증합니다. 로그/TSV/중간 JSON은 output에만 둡니다. source 변경 후 이전 캡처를 완료 근거로 쓰지 않습니다.
6. 결과를 개별 review와 오늘 할 일에 확정하고 원 PR 기여/보정 source와 실제 검증 head를 구분합니다. 원격 PR 생성·push·merge는 사용자의 해당 단계 지시에 따릅니다.

## 적용 계약과 반례

마스터 쪽 단 구분선은 배경이지만 보이는 개체이며 본문 흐름을 전진시키지 않습니다. masterPage의 텍스트 프레임과 그 안 colPr 정의에 따라 위치/높이를 계산하고 별도 본문 단 설정으로 바꾸지 않습니다. 1단/구분선 없음에는 선을 추가하지 않으며 가변 너비·여백·여러 단의 위치도 기존 column 계산 계약을 재사용합니다. 본문 쪽수·다음 내용·번호와 다른 바탕쪽 개체 보존을 확인합니다. 분할/이어받기 규칙 변경은 없습니다.

## 2026-10-06 추가 원인 추적

바탕쪽 선 복원 뒤 공개 원본 전체 1쪽은 86.42433%로 아직 미달입니다. 굵은 정식 face를 추가 공급해도 점수는 동일했습니다. 보기 ㄴ의 수식은 저장 높이 2580 HWPUNIT, baseLine 85이며 MCP Print에서 앞뒤 글자보다 위에 표시됩니다. `equation::flow_metrics_hwp → composer/line_breaking.rs의 flow_inline_controls 및 줄 기준선 → paragraph_layout.rs의 두 인라인 EquationNode 생성`을 추적하면 측정은 저장 기준선 거리를 쓰고 배치는 intrinsic 기준선으로 다시 계산한 뒤 줄 위를 clamp합니다. 같은 flow 메트릭을 실제 원점도 소비하도록 고치며 셀/본문, 텍스트 혼재/수식 전용 경로와 최소 입력을 함께 재확인합니다. SVG/Canvas2D/Skia는 생성된 EquationNode 원점을 그대로 소비하며 압축 프레임의 공통 세로 비율은 유지합니다. 독립 Print 확인 없는 합성 저장값을 근거로 새 예외를 만들지 않습니다.

두 파일 비교는 (1) 원 최소 입력과 스크립트 제거 사본의 최신 Native 전체 페이지 raster, (2) 기존 기여자 Hancom 2022 PDF와 새 MCP 2020 Print PDF, (3) 각 HWPX와 대응 MCP Print PDF를 구분해 같은 96 DPI로 전체 페이지를 비교합니다. 원본의 조판 entry는 보존하고 TSV/로그는 output에만 남깁니다.

선택지의 원본 HWPX 문단에는 저장 LineSeg가 없고 각 LEFT 탭 폭 4000은 고정 전진 폭으로 남아 있습니다. MCP Print의 ②/③은 문단 기본 탭 격자에 놓이지만 Native는 앞 수식 폭을 포함한 커서에 저장 전진 폭을 다시 더해 점점 오른쪽으로 밀렸습니다. `compose_paragraph_scoped → ComposedParagraph.tab_extended → paragraph_layout의 런/개체 커서 → text_measurement의 저장 전진/TabDef 분기`를 추적했습니다. 저장 줄 부재/합성 줄의 수용 여부는 기존 `para_has_no_stored_line_segs` 계약을 그대로 쓰고, 이 경우 저장 advance를 전달하지 않아 현재 문단 TabDef/기본 격자에서 계산하도록 보정합니다. 원본의 tab/type/leader 메타데이터는 model에 보존하며 실제 저장 LineSeg가 있는 경로는 기존 advance를 사용합니다. 선택지/수식의 순서·본문과 셀·저장 줄 대조군을 최종 검증 범위에 포함합니다.

실제 SVG data-URI를 fontTools로 읽어 파일 공급의 오판도 확인했습니다. 선언된 한컴산뜻돋움은 실제 Noto Sans KR ExtraLight, 함초롬바탕은 Haansoft Batang이었습니다. 단순 fc-match 결과를 실제 SVG 공급으로 간주했던 초기 기록을 정정합니다. 시스템은 정식 HanSantteutDotumRegular/HCRBatang을 보유하지만 SVG 파일 탐색이 시스템 하위 폴더와 영문/해시 접두 파일명을 해석하지 못했습니다. 명시적 공급 우선순위 뒤에 fontdb의 family/weight 조회를 추가해 실제 등록 face를 파일명 폴백 전에 찾습니다. 이름/굵기 일치 여부와 최종 data-URI face를 다시 확인하고, 일반 시스템 등록만으로 실행하여 별도 RHWP_FONT_PATH나 사후 SVG 교체를 쓰지 않습니다.

## Native 선행 확인 및 입력 보존

source 보정의 선행 Native 전체 1쪽은 공개 문항 92.28714%, 최소 입력 96.47259%로 통과했습니다. 시스템 등록 글꼴만 사용한 full embed의 실제 name table은 Han Santteut Dotum Regular/Bold와 HCR Batang으로 확인했습니다. 단 구분선·보기 사각 틀·그림 경계가 대응하고 보기 ㄴ의 수식이 앞뒤 본문보다 위에 놓이며 ②/③이 기본 탭 격자로 복귀했습니다. 수식 글리프의 장평/획과 발문 간격의 잔여 차이는 남아 있고, 보조값 통과를 전체 glyph fidelity로 확대하지 않습니다. 최종 source의 fresh WASM·회귀·lint·전체 검증은 이어서 실행합니다.

원 최소 입력/스크립트 제거 사본의 Native 전체 페이지 raster는 픽셀 동일(실루엣 100%), 기존 기여자 Hancom 2022 PDF/MCP 2020 Print PDF도 픽셀 동일(실루엣 100%)입니다. paired PNG를 직접 열어 4행·화살표·번호 보존을 확인했습니다. 이는 원본 매크로 실행 또는 기존 PDF의 Print 절차 확인을 뜻하지 않습니다. 새 Print의 실제 MCP 실행 근거와 스크립트 없는 사본의 동일성으로 기준 출력 대응을 입증합니다. 자료는 각각 output의 min-original-vs-safe, min-old-vs-print-pdf와 compare 패널에 남겼습니다.

MCP 산출 PDF는 원본 형식·엔진·상대 디렉터리의 canonical 짝짓기에 맞게 `pdf/issue7593/repro-equation-issue7593-hwpx-2020.pdf`, `pdf/issue7593/repro-dyad-min-no-script-hwpx-2020.pdf`로 보존하며 출력 바이트를 바꾸지 않았습니다. 원문은 samples/issue7593 아래 두 파일이며 원 기여자의 최소 입력/PDF도 기존 경로에 보존합니다. 최종 검증은 이 안정 경로에서 다시 실행합니다.

## 정식 회귀와 Studio의 직접 누락 확인

Native/fresh WASM에서 두 대응 Print의 최저 92.28714%를 먼저 확인한 뒤
`tests/cases/issue_7593_print_placement.rs`에 구분선의 바탕쪽 소유/프레임 관계,
보기 ㄴ 수식의 저장 기준선/셀 소유, 선택지의 기본 탭 격자/순서/단 내부 포함을 추가했습니다.
원래 source `a0db9c052`의 실제 실행은 3 FAIL이며 각각 선 0개, Print의 기준선 순서 위반,
탭 격자 비율 3.5375(독립 Print는 3)로 실패했습니다. 빌드 실패를 red로 세지 않았습니다.
초기 테스트 작성 중 TextRun의 셀 문단 인덱스와 Equation의 최외곽 표 문단 인덱스를
혼동한 assertion은 CellContext의 실제 소유 관계로 수정했습니다. 최초 harness 준비 뒤
format으로 suite 배정이 바뀌어 0 tests가 실행된 결과도 통과로 세지 않았습니다.

실제 Studio CanvasKit 캡처를 직접 열자 공개 원본의 ①/② 수식이 빠진 것을 확인했습니다.
WASM render tree의 7 Equation 존재만 확인한 초기 browser 결과는 표시 성공 근거가 될 수 없으므로 정정합니다.
기본 Noto Sans KR에 U+2AFD glyph가 없으면 drawEquationText가 false를 반환하고,
PictureRecorder가 정상 AB/CD·dyad 장식까지 포함한 수식 전체를 버렸습니다.
기본 CJK → 기존 D2Coding → 실제 glyph를 가진 Noto Sans Math 순서로 font를 고르도록
보정했고 문자를 다른 codepoint로 바꾸거나 실패한 수식을 숨기지 않았습니다.
시스템 설치본의 전체 font를 WOFF2로 변환하고 SIL OFL 1.1을 함께 보존했습니다.
원 TTF SHA-256 `8242bd1e55368b27e32455260754cf9aa58f3ad7ea80664b66c21f1b09910d6c`,
WOFF2 `6836027b99d29fd0e6f668cf0c2e89107de4131b91116cd96516ad4a7e19d66d`이며 subset/문자 매핑 변경은 없습니다.
정확한 glyph 의미/한컴의 평행 기호 모양은 #7594의 잔여 축이고 그 이슈를 종료하지 않습니다.

보정 전 실제 browser는 ①/②의 내용 0과 `equation:invalidLayout`을 검출했습니다.
보정 후 실제 제품 loadDocument/font 준비/paint 경로의 Canvas2D·CanvasKit × 두 파일은 4 PASS,
모든 수식에 내용이 있고 CanvasKit unexpectedUnsupportedOps가 0이며 숨은 Canvas2D overlay는 없습니다.
`rhwp-studio/e2e/issue-7593-dyad-display.test.mjs`가 이 실제 경로를 보존하고,
기존 font coverage 검사도 Noto Sans Math의 U+2AFD/2225/2194 공급을 확인했습니다.
실제 내려받은 WASM SHA는 `040ffe26e4de9a064e350c6264452e3b767c4808f798661907ec42b29ba22a34`입니다.
Rust/lib source는 1a7ad89e4와 같고 TS 보정은 이 WASM을 재사용하는 paint 경로의 변경입니다.

신규 두 sample은 전체 1쪽 Print/PDF/Native/fresh WASM 대조를 확인한 뒤 oracle 쪽수 원장에 1/1로
등록합니다. regenerate.py의 canonical 선택을 두 입력에 적용해 형식·상대 디렉터리·저장 제품별
engine2020 짝이 실제 위 PDF로 결정됨을 확인했습니다. 기존 원장의 다른 행·허용치는 바꾸지 않습니다.
모든 최종 필수 lint·전체 Rust/Skia 검증의 완료 여부는 다음 단계 결과에서 확정합니다.

보정 복원 후 동일 정식 배치 검사 3 PASS / 0 FAIL (exit 0)을 확인했습니다.
로그 `output/pr-review/pr7600/logs/placement-red.log`, `placement-green.log`를 보존했습니다.

## 발문 수식의 저장 프레임과 실제 paint 폭 분리

사용자가 발문 첫 수식이 Print와 너무 다르다고 지적했습니다. 원본 `rm dyadAB`의
common.width 3000 HWPUNIT은 흐름 예약 폭 40 (96 DPI), 동일 BaseUnit 1100의 intrinsic AB 장식 폭은
20.02이며 SVG의 최종 transform이 약 1.998배 확대했습니다. 독립 PDF의 실제 AB·화살표 glyph는
약 20의 폭이고 뒤 본문은 예약 폭을 지나 시작합니다. 같은 보기의 `rm dyad AB`는 저장 폭 1500이므로
기존 강제 확대가 거의 없어 같은 글자가 문맥에 따라 2배 다른 장평이 됐습니다.

생산 `Equation.font_size/BaseUnit → EqLayout::for_equation의 공통 glyph/장식 layout`과
흐름 측정 `flow_metrics_hwp → tac_w/줄 커서 → EquationNode.bbox`를 구분합니다.
SVG/web_canvas/Skia/Studio CanvasKit은 현재 bbox.width/layout.width로 paint만 다시 늘립니다.
최종 paint에서 이 가로 확대를 제거하고 BaseUnit과 공통 intrinsic layout을 그대로 소비합니다.
예약 bbox와 뒤 텍스트의 위치·내용 소유는 보존하고, 기존 압축 높이의 공통 세로 비율은 유지합니다.
문서 ID·dyad 전용 폭·특정 비율 조건·clamp는 추가하지 않습니다.

먼저 동일 두 문서를 Native로 다시 대조하고 발문의 실제 glyph/화살표·뒤 글자 및 선택지를 직접
확인합니다. Native/fresh WASM 최저 90% 이후 동일 글자의 paint 장평/본문 예약 관계를 정식
회귀에 추가합니다. 수정 전 red와 수정 후 green, normal 최소 입력·숫자 대조와 필수 lint·전체
회귀·Skia를 새 source에서 검증합니다. 이전 source의 시각/CI 결과로 완료를 선언하지 않습니다.

Native 선행 재출력은 공개 원본 92.29683%, 최소 입력 96.24413%, 모두 gate passed / 전체 1쪽입니다.
발문 glyph/화살표 폭의 이중 확대가 없어졌고 흐름 예약 폭과 뒤 글자 위치는 보존됐습니다.
전후·독립 Print의 확대 패널 `output/pr-review/pr7600/equation-first-line-before-after-print.png`를
직접 열어 확인했습니다. 전체 보조값의 변화와 별개로 표시 장평의 결함이 해결됐음을 구분합니다.
CanvasKit의 Text 기본 이탤릭 처리가 명시적 Roman을 다시 덮던 경로도, 기본 상태와 명시적 상태를
구분해 rm의 정체·it의 이탤릭을 상속하도록 보정합니다. 수식 글자 자체의 backend별 폰트와 작은
획/원점 차이는 여전히 남아 있으며 이 선행 결과는 전체 검증 완료 문구로 쓰지 않습니다.

## 선택지의 내부 관계 기호 간격

새 출력에서 ④/⑤의 문단 탭 원점을 독립 Print와 같은 좌표계로 확인했습니다.
사용자가 지적한 ①/②의 수식 안 간격은 별도로 남았습니다. 일반 공백은 tokenizer가 명령 경계로
제거하며, layout_row는 형제를 간격 0으로 붙입니다. BOT는 MathSymbol이지만 옆 여백이 없고
⊥의 advance를 0.5em으로 추정해 실제 수학 글꼴의 advance(배포 Latin Modern Math 0.778em,
시스템 DejaVu Math 0.788em)보다 작습니다. 다음 dyad가 관계 기호와 붙거나 겹칩니다.
직접 U+2AFD는 Text로 취급되어 관계 기호의 정체·간격 계약도 소비하지 못합니다.

공통 관계 기호 분류를 parser/layout에 적용하고 기존 연산자 여백 계약 0.15em을 관계 기호의
양쪽에도 소비하게 합니다. glyph/간격을 명시적인 공통 Row 결과로 만들어 SVG/Canvas2D/Skia/
CanvasKit이 같은 원점을 소비합니다. ⊥/⊤의 추정 advance는 해당 실제 수학 글꼴 폭에 맞는
0.8em 계열로 분리하고 °/‰ 등의 기존 좁은 폭은 보존합니다. 이 값은 원본 페이지 좌표에서
역산한 px 값이 아니라 독립 글꼴 메트릭의 차원 없는 비율입니다. 다섯 선택지의 문단 탭 위치·
수식/글자 순서·누락과 최소 정상 입력을 다시 확인합니다. width 수정 전 WASM build는
새 source 검증으로 오인하지 않도록 종료했고 원시 로그만 보존했습니다.

Native 간격 보정의 재출력은 공개 원본 92.98588%, 최소 입력 99.43478%로 모두 전체 1쪽 gate를
통과했습니다. ①/② 관계 기호와 양쪽 수식의 간격, 다섯 선택지/뒤 정답의 순서·누락,
④/⑤ 탭 원점을 전체 페이지와 확대 패널에서 직접 대조했습니다. 수식/글자 획·작은 font 원점
차이는 남아 있고 완전 동일 출력이라고 쓰지 않습니다. 새 source는 `9ee2878be`입니다.

사용자 요청에 따라 `THIRD_PARTY_LICENSES.md`에 폰트의 2.001 버전·저작권·OFL 전문·출처를
등록했습니다. 실제 배포에서 전문이 빠지지 않도록 Chrome/Firefox의 OFL text/집계 문서 복사와
VS Code의 math font/라이선스/집계 문서 및 실제 webview URI를 보완했습니다.
Studio·Chrome·Firefox build, VS Code typecheck/webpack build, font/extension 배포 계약 9 PASS입니다.
새 37번째 폰트의 226,456 bytes 추가만큼 inventory는 37개 / 22,877,752 bytes로 변경했습니다.
이는 알려진 신규 자산의 재고 등록이고 renderer golden·래칫 오차 허용치 변경이 아닙니다.
최신 Rust WASM의 렌더 검증과 font 배포 자산 검사를 구분하며 위 build만으로 새 WASM의
시각 확인을 완료했다고 주장하지 않습니다.

## 9ee2878be의 전체 페이지 재검증과 paint/간격 회귀

최적화 fresh WASM build exit 0 (13분 48초), pkg/public/실제 브라우저 download의
WASM SHA-256은 `cf850c529720d37cef7953f6ef03d857662110d408e55a2decb9bd5029a43c18`이다.
Rust 런타임 source 9ee2878be와 동일한 84fda6939에서 Native/fresh WASM 각각
공개 원본 92.98588% / 최소 입력 99.43478%, 대응 Print 전체 1쪽 gate passed를 확인했다.
전체 페이지 compare·review·standalone overlay와 실제 Studio Canvas2D/CanvasKit 4개 경로를
직접 확인했다. 실제 수식 내용·backend·download SHA 및 숨은 overlay 없음은 4 PASS다.
원 최소 입력↔스크립트 제거 사본의 최신 Native 전체 페이지 raster도 100% / 픽셀 동일하다.
원시 TSV 원본 두 개를 PR 첨부용 ZIP으로 묶었고 output 안에만 보존했다.

이 독립 Print 및 Native/fresh WASM 90% 선행 조건을 충족한 뒤
`tests/cases/issue_7593_print_placement.rs`에 실제 SVG paint 장평 동일성, BOT/직접 Unicode
관계 기호의 양쪽 간격·내용 순서를 추가했다. 절대 px나 출력 해시를 기대값으로 쓰지 않는다.
수정 소스 선행 GREEN 6 PASS (`logs/paint-green-initial.log`), 수정 전 7a61272f1의
실제 RED 3 PASS / 3 FAIL (exit 100, `logs/paint-red.log`)을 확인했다. 장식 실제 폭
39.99996 대 20.02/20.02, BOT의 왼쪽 간격 부재, 직접 Unicode의 MathSymbol 0개가
각각 의도한 결함으로 실패했다. 기존 3개 기하/소유 검사는 RED 대조에서도 통과했다.
최신 소스 복원 후 같은 6개를 다시 실행하며 필수 lint·전체 Rust/Skia 게이트를 이어서 수행한다.

최신 WASM으로 Studio/Chrome/Firefox/VS Code 배포본을 다시 빌드했다. typecheck/webpack과
폰트·확장 배포 계약 9 PASS를 확인했다. Studio dist WASM도 위 cf850c52 해시와 일치한다.
Noto Sans Math의 OFL 전문·원 저작권 표기는 모든 배포본에서 원문 바이트 그대로 보존됐다.
Vite ENOSPC는 저장소 루트에서 실행한 데 따른 것이며 사용자가 Studio 디렉터리에서 정상
동작을 확인해 Vite 설정 수정은 범위에서 제외했다.

최신 source 복원 후 같은 6개 회귀도 6 PASS / 0 FAIL (exit 0)로 종료했다.
`logs/paint-green.log`의 실제 실행 count와 summary를 확인했다. 검증 후보는
`31c1d4ecd6cfd0fce50658fccce158cf876b8077`이며 9ee2878be 이후의 Rust 런타임 diff는 없다.
필수 검증은 upstream/devel을 fetch해 base a048da32d로 고정하고 순차로 실행한다.

## 전체 회귀의 실패와 사용자 승인 범위 확장

후보 31c1d4ecd의 fmt/세 Clippy/workspace build/base 정책과 원 기여 focused 3개는 통과했다.
전체 nextest는 10,517개 실행, 10,514 PASS / 3 FAIL / 50 SKIP, exit 100, 730.063초로 실제 종료했다.
Native Skia는 이 실패 뒤 실행하지 않았고 PR 생성/승인/merge로 보고하지 않는다.

- #1139 문 28의 고정 x/y 영역은 `(ii)` 소유인 pi817의 두 인라인 수식까지 선택했다.
  실제 앞 수식은 pi816이며 점유 끝 779.853, 다음 pi817 시작 789.853으로 10의 분리 공간이 있다.
  오류의 801.853은 다음 문단 자체의 수식 끝이다. 원본 문단 소유·순서 관계로 바꿀 필요가 있다.
- #1308의 HWP/HWPX 두 검사는 SVG 수식을 245..260의 고정 y 안에서 찾지 못했다.
  문단/명시적 개행의 줄 소유·내어쓰기 관계로 선택하고 실제 Print에서 확인한다.
- 사용자 추가 승인: #1139 파일 전체(86개 검사 / 3,731줄)의 px 고정 검사를 검토·보정한다.
  새로운 문서/원본의 시각 미달을 기존 차이라는 이유로 면제하지 않는다.

기존 2020 suffix PDF는 실제 Creator가 2022이고 p15의 문 28은 왼쪽 단 끝으로 재배치돼
현재 저장 문서의 오른쪽 단 시작과 다르다. 새 Native/fresh WASM 대조에서 35.87554%를 확인했다.
기존 원 PDF 2024의 p15와 Native는 84.63888%이며 이것도 90% 충족으로 보고하지 않는다.
둘의 Print 실행 출처를 Creator만으로 추정하지 않고 같은 HWP를 metadata 2022에 맞는 MCP 2020
Print로 다시 출력한다. job 15b90e11-d4b7-45f6-847a-baf3d0a67891은 Hancom 11.0.0.9136 /
PrintMethod 0 / one-up / 18쪽으로 실제 성공했다. 원본과 실패 증거를 보존한다.

Native TSV ZIP의 GitHub 첨부 API는 ZIP MIME 형식을 HTTP 422로 거절했다.
이는 자동 승인 거부가 아닌 서버 첨부 형식 제한이며 원시 파일은 output에 그대로 남아 있다.
파일 형식을 위장하거나 Git에 커밋하지 않고 PR 게시 시 정상 첨부/동일 파일 링크 경로를 확정한다.

## #1139 전체 px 고정 검사 재검토 단계

사용자 답변으로 파일 전체를 포함했다. 원본 86개 검사 중 58개가 px 고정/영역 선택 검토 후보이며
실제 입력은 19개다. 진단 inventory는 output/issue1139-audit.json에만 보존한다.

사용자가 재확인한 순서는 **원 회귀 테스트의 원본 입력 출력 보정 → 독립 한컴 Print PDF와
Native/fresh WASM의 관련 모든 페이지에서 90% 이상 확인 → 필요한 회귀 검사 변경**이다.
#1139와 #1308의 기존 검사도 이 순서를 적용하며, 미달 상태에서 px 고정 검사를 먼저 관계 검사로
바꾸지 않는다. 90% 이상을 확인한 뒤 문단·줄·개체의 소유·순서·보존·포함·겹침 관계를 검사한다.
기존 검사가 독립 기대값에 맞으면 유지하고 구현을 고친다. 원본과 실패 증거는 보존한다.

1. 각 검사의 주장·원본 문단/컨트롤 소유·쪽 범위와 실제 px predicate를 대조한다.
2. 각 원본을 저장 제품에 맞는 MCP Print로 출력해 Native/fresh WASM의 90% 선행 조건을 확인한다.
   저장 제품 2022의 기본 engine2020 p15는 새 Print도 35.9635%로 미달이다. 이전 PDF의 상태로 면제하지 않는다.
   같은 원본의 2024 Print는 버전 차이 진단 대조로 별도 산출했고 자동으로 기준을 대체하지 않는다.
3. font/배치의 실제 원인을 공통 생산·소비 경로에서 고친 뒤, 기존 검사의 pixel 영역 선택·절대 좌표를
   원본 문단/줄/개체 소유·완전성·앞뒤 순서·단 내부 포함 관계로 바꾼다. 픽셀 허용 범위 확대는 하지 않는다.
4. 변경 파일은 1,000줄 이하의 모듈 구성으로 정리하고 정식 integration 검사 count/등록을 보존한다.
5. 최종 source의 시각·focused·fmt/세 Clippy/base 정책·전체 nextest·Native Skia·fresh WASM을
   순차 완료한 뒤 오늘 할 일/개별 리뷰와 같은 통합 PR을 등록하고 CI/merge/후속 처리를 진행한다.

## 회귀 보정 순서의 지침 반영

`AGENTS.md`·`CLAUDE.md`·`CONTRIBUTING.md`, 기여자 스킬과 PR 템플릿, 시각 검증 정본,
메인터너·콜라보레이터·첫 기여자 절차와 리뷰 공통 표에 위 90% 선행 조건과 px 고정 금지를 반영했다.
PR 템플릿에는 실패한 원 검사·원본 입력·출력 보정 SHA·90% 이상 확인 증거·검사 변경 이유를
연결하는 항목을 추가했다. 기록의 띄어쓰기 수정도 함께 유지한다.

문서 검증은 `git diff --check`와 변경 Markdown 20개 상대 링크 검사에서 오류가 없었다.
기여자 스킬 계약 검사는 26개 중 24개 통과, 2개 실패였다. 수정 전 HEAD의 동일 검사도 같은
2개가 실패했으며, 기존 `fixtures/catalog.json`·`분석` 문구 요구 실패로 확인했다.
이 문서 검증 결과를 #1139·#1308의 시각 미달 해소나 전체 회귀 완료로 보고하지 않는다.

## 90% 통과 뒤 수식·간격의 확대 판독 지침 보완

사용자가 지적한 “이 부분 수식 너무 다른데?”와 “여기도 간격이 다름”을 독립적인 검토 항목으로
명시했다. 90% 통과 후에도 변경 영역·알려진 차이·사용자 지적 영역을 독립 Print PDF와 변경 전·후
Native/fresh WASM의 같은 내용·배율로 확대해 수식 모양·장평·기준선·첨자·화살표·장식,
관계 기호 양쪽·앞뒤 본문·보기 번호·탭·줄 사이 간격을 직접 대조한다.

공통 지침·기여자 지침과 스킬·역할별 검토 절차·PR 템플릿·리뷰 공통 표에 상세 판독 절차를
연결했다. 페이지 점수와 판독 결과를 구분하고, 확인된 결함은 점수 통과로 면제하거나 정상
기대값으로 고정하지 않는다. 미해결 결함·미검증은 해당 범위의 완료·회귀 추가/변경·승인·통합
보류 사유로 기록하며, 출력 보정 뒤 새 source에서 영향 영역과 관련 페이지를 다시 검증한다.

## #1308 원본 수식의 기준선 원인 분석과 보정 계획

기준 입력 `samples/eq-002.hwp`와 `samples/hwpx/eq-002.hwpx`는 저장 제품 2020이며,
기존 MCP 2020 Print 두 파일을 보존한다. 저장 프레임의 기준선과 현재 AST의 위첨자 배치를
대조했다. 분수가 포함된 위첨자의 깊이를 반영하지 않고 높이의 0.7배로 본체 기준선을 정하는
`layout_superscript` 때문에 본체가 위첨자 아래에서 충분히 내려오지 않는다. HWP 원본의 두 번째
줄에서 실제 Print 숫자 3의 잉크 하단은 266.18이고, 현재 paint의 본체 기준선은 약 257.87이다.
이 좌표는 진단 관측값이며 회귀 기대값이나 특정 문서 분기에 사용하지 않는다.

독립 규칙은 OpenType MATH의 위첨자 기준선 올림과 위첨자 하단의 최소 여유다.
[OpenType MATH](https://learn.microsoft.com/en-us/typography/opentype/spec/math)와
[Noto 수학 폰트 제작 문서](https://github.com/notofonts/math/blob/main/documentation/building-math-fonts/index.md)를
확인했고, 배포 `LatinModernMath-Regular.woff2`의 UPM 1,000에서 SuperscriptShiftUp 363,
SuperscriptBottomMin 108, ScriptPercentScaleDown 70을 읽었다. 본체/위첨자의 기준선·깊이에서
배치 결과를 계산하는 공통 helper를 마련하고 기존 첨자 크기 비율은 유지한다.

호출 경로는 수식 AST → EqLayout의 위첨자/Row 결과 → inline_baseline_px와 EquationNode →
SVG/Canvas2D/Skia/CanvasKit이다. 저장 흐름 프레임을 글자 장평으로 확대하는 예외나 페이지별
좌표 보정은 추가하지 않는다. 단순 지수·분수 지수·키 큰 본체 및 #7593 두 원본을 대조하고,
Native/fresh WASM 90%와 확대 판독을 충족하기 전에는 회귀 테스트·golden을 변경하지 않는다.

## 위첨자 후보 재검토 결과와 원본 Print 자료 보존

지침 변경은 `4acd0aa11db57e1078694501fdc95e5018b747c3`로 커밋했다. 그 뒤 미커밋
위첨자 보정 후보를 Native에서 빌드하고 독립 Print와 대조했다. HWP/HWPX 각각의 전체 1쪽은
77.29227%에서 **84.55997%**로 개선됐지만 90%에 미달했다. 공개 #7593 대조군은
92.98588%를 유지했고, #1139 문28의 MCP Print 대조군 p15는 35.98720%였다.
확대 판독에서 근호·분수 지수·글자 폭과 내부 간격, 원본 `eq-002`의 자동 번호 누락이 남았다.
원본 HWPX의 자동 번호는 `num=11`, 앞 장식 `문`, 뒤 장식 `）`이며 Print에 표시된다.

기존 위첨자 검사는 12건 중 11건 통과, `test_superscript_tall_base_no_overshoot` 1건 실패
(exit 101)였다. 이는 기존 계약 실패의 실행 증거이며, 한컴의 정상 동작에 대한 최종 판정과
구분한다. 시각 선행 조건과 기존 계약을 충족하지 않아 이 후보를 채택하지 않았고, 에이전트가
추가한 helper와 부모 모듈 변경만 실험 전 source로 복원했다. 기존 회귀·기대값은 변경하지 않았다.
후보 patch·helper·실행 파일·로그·review PNG는 `output/pr-review/pr7600/`에 보존한다.
미커밋 후보의 실행 결과를 커밋 `4acd0aa11` 자체의 검증 결과로 보고하지 않는다.

브라우저의 `CSS.getPlatformFontsForNode`로 실제 수식 face가 Times New Roman임을 확인했다.
설치되지 않은 Latin Modern Math의 local 선언 때문에 배포 웹폰트 공급이 선택되지 않았다.
배포 폰트를 넣은 **진단 SVG**만 따로 캡처한 보조값은 85.01772%였다. 이는 source 수정이나
정식 Visual Sweep 통과가 아니며, 다른 face의 MATH 상수 적용만으로 완전한 보정을 주장하지 않는다.

#1139 파일 전체의 86개 검사에서 사용하는 원본 19개를 보존한 채, 저장 제품별 MCP engine을
명시해 Print PDF를 확보했다. 모두 `pdf_print_method=0`·1-up 출력이며 입력 preprocessing은
없다. engine 2020의 실제 빌드는 11.0.0.9136, engine 2024는 13.0.0.3901이다. 다음 PDF는
**독립 비교 자료**이며 RHWP의 기대값·golden을 갱신하거나 쪽수 차이를 승인하는 자료가 아니다.
입력 SHA와 실제 Print 상태·로그·TSV·향후 실행 목록은 ignored output에 보존하고 Git에 넣지 않는다.

| 입력 ID | 원본 | 독립 Print | RHWP / Print 전체 쪽수 | MCP job | PDF SHA-256 |
| --- | --- | --- | --- | --- | --- |
| `input-01` | [`samples/3-09월_교육_통합_2022.hwp`](../../../samples/3-09월_교육_통합_2022.hwp) | [2020 Print PDF](../assets/pr_7600_control_print_20261006/input-01-3-09월_교육_통합_2022-2020.pdf) | 23 / 23 | `40c81c4e-94c4-4949-9139-840389147099` | `523477f1bd0572f95665c3f3651d1deb4fb94701e96301abe8835edaee69dd04` |
| `input-02` | [`samples/3-09월_교육_통합_2022.hwpx`](../../../samples/3-09월_교육_통합_2022.hwpx) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-02-3-09월_교육_통합_2022-2024.pdf) | 23 / 23 | `7fb180e8-77a5-47bc-bb39-4bad74738062` | `052f390b36e53a2374b3883a82c96164802784497259f1e7d5e3fe06b9a163f6` |
| `input-03` | [`samples/3-09월_교육_통합_2023.hwp`](../../../samples/3-09월_교육_통합_2023.hwp) | [2020 Print PDF](../assets/pr_7600_control_print_20261006/input-03-3-09월_교육_통합_2023-2020.pdf) | 20 / 20 | `ec4e2d74-ad11-4f59-9b20-e84df82dbb38` | `9e28de788c17853eedc49d08b0f5e8f0a6907a5034d5270e44967c9a56d49853` |
| `input-04` | [`samples/3-09월_교육_통합_2024-구분선아래20.hwp`](../../../samples/3-09월_교육_통합_2024-구분선아래20.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-04-3-09월_교육_통합_2024-구분선아래20-2024.pdf) | 23 / 23 | `ef0fc6d0-23f4-4741-9deb-4c30a6e50263` | `aff11a7842cc517da8e1104a799a9ce788765fccddcba7d74131cc1de41f0eba` |
| `input-05` | [`samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`](../../../samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-05-3-09월_교육_통합_2024-구분선아래20구분선위20-2024.pdf) | 23 / 23 | `8f6a8721-a89f-457a-a61f-8615637e2591` | `729936c1e0d1ca0fe90924b8a396fcbbf871fd39f5c465081fffe63bb7545b28` |
| `input-06` | [`samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx`](../../../samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-06-3-09월_교육_통합_2024-구분선아래20구분선위20-2024.pdf) | 23 / 23 | `ec29c101-6145-48a8-a5c5-b1044659f4d5` | `56e352f4aa3748b952460176c6bdb1d2d1bcae242e00a214ac9084c5826d2fc3` |
| `input-07` | [`samples/3-09월_교육_통합_2024-미주사이20.hwp`](../../../samples/3-09월_교육_통합_2024-미주사이20.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-07-3-09월_교육_통합_2024-미주사이20-2024.pdf) | 24 / 24 | `b17db545-d449-442b-a8f7-1d27d9950dcc` | `444f9338d0987130ffa850d755d90c95c4e3ae60d6b032c7c1dee69215a988e4` |
| `input-08` | [`samples/3-10월_교육_통합_2022.hwp`](../../../samples/3-10월_교육_통합_2022.hwp) | [2020 Print PDF](../assets/pr_7600_control_print_20261006/input-08-3-10월_교육_통합_2022-2020.pdf) | 18 / 18 | `15b90e11-d4b7-45f6-847a-baf3d0a67891` | `91255993eb9fe5a9aa598ed42ea4d2ffcebf346f0f446bd80af31f43a439cbba` |
| `input-09` | [`samples/3-11월_실전_통합_2022.hwp`](../../../samples/3-11월_실전_통합_2022.hwp) | [2020 Print PDF](../assets/pr_7600_control_print_20261006/input-09-3-11월_실전_통합_2022-2020.pdf) | 21 / 21 | `cc8b4953-2dbd-4bfd-b46a-4a3db2d8fc30` | `b5dc3580c0c3716963c9686ec62fd779cedfea97f50d8f49eb98a2ec8fae369a` |
| `input-10` | [`samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp`](../../../samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-10-3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20-2024.pdf) | 23 / 22 | `6ecebecb-7e6f-418a-8964-f126c12897cb` | `12f3b28ca1339359fd295b89d5c2a99994a12efbced8444f293b374190616eaf` |
| `input-11` | [`samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`](../../../samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-11-3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0-2024.pdf) | 21 / 21 | `c80861fa-691f-4707-9295-2154f1a55f59` | `d62195ebee8adb8694ba69d89d989f7d03166bd0c76f5bbca1375aeb06c0de6c` |
| `input-12` | [`samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwp`](../../../samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-12-3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2-2024.pdf) | 21 / 21 | `405acaad-e56e-4918-b2c5-078718d66841` | `7b434e2943eebd19e330452a92c9f9f1a74c5e16cd54e453c43c9f377a398b32` |
| `input-13` | [`samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp`](../../../samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-13-3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20-2024.pdf) | 21 / 21 | `601c0352-2e86-4170-8bf8-f0ef92a98c0b` | `41c62d023d66e36d21a9913762b4e063fd7179c95ae1757da1fe98c0d22b61de` |
| `input-14` | [`samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwp`](../../../samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-14-3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20-2024.pdf) | 21 / 21 | `09e82f65-e333-4bb7-a2a6-2cc68fbb2e38` | `063a7ac10fccbb63096c6d09b934fce77c7e723a103e1fb36124588750924218` |
| `input-15` | [`samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp`](../../../samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-15-3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2-2024.pdf) | 21 / 21 | `7c995fc0-505b-4fec-b76b-e5109db308ea` | `0d77e32e483ed383056554ef4963ce2ca0b234cf2fd2a23e1a0859449803017f` |
| `input-16` | [`samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp`](../../../samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-16-3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7-2024.pdf) | 21 / 21 | `4ae4bd30-54bc-433b-9728-a1f5d9fc0717` | `244ec011e7dbff3c8b62889d50ed4858181a8fb9d15b484217fece8d04a98cc1` |
| `input-17` | [`samples/hwp3-sample16-hwp5.hwp`](../../../samples/hwp3-sample16-hwp5.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-17-hwp3-sample16-hwp5-2024.pdf) | 65 / 64 | `228402de-7723-4344-8823-9a71a65c2ead` | `e25441458879234dee150af856cc45a38a576e607fa44e0630a13e1883112967` |
| `input-18` | [`samples/test-image.hwp`](../../../samples/test-image.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-18-test-image-2024.pdf) | 5 / 5 | `0b39bf1a-dd27-4737-8e94-43e49bc9dbfc` | `c50cd5e6421d026385670aa631917c3d5b635c4539fa24cfadac8fcd75e88e3c` |
| `input-19` | [`samples/수식-문자처럼취급-아님.hwp`](../../../samples/수식-문자처럼취급-아님.hwp) | [2024 Print PDF](../assets/pr_7600_control_print_20261006/input-19-수식-문자처럼취급-아님-2024.pdf) | 1 / 1 | `17eb2cf2-ac78-444a-9367-5aaf5325b11a` | `a195e2377cc542a5611cd967aaa4f40b760aea4360c1873ff4028e6c85b6148c` |

쪽수가 다른 원본은 구분선 없는 2024 실전 문서(23 / 22)와 hwp3-sample16-hwp5(65 / 64)다.
차이를 기대값 변경이나 글꼴 예외로 숨기지 않는다. 전체 범위의 Native/fresh WASM 90%와
직접 판독을 충족하기 전까지 #1139 전체의 px 고정 검사 및 #1308 검사는 변경 보류 상태다.

#1308의 두 독립 PDF도 같은 자료 폴더에 보존한다:
[HWP Print PDF](../assets/pr_7600_control_print_20261006/pr7600-eq002-hwp-print-2020.pdf),
[HWPX Print PDF](../assets/pr_7600_control_print_20261006/pr7600-eq002-hwpx-print-2020.pdf).

사용자에게 최신 보정 후보의 `eq002-sup-hwp/review/review_001.png`(84.55997%)와
공개 입력 `sup-public-control/review/review_001.png`(92.98588%)를 직접 표시했다. 두 이미지는
실험 후보의 증적이며 이후 source의 최종 head 증적으로 재사용하지 않는다. 최종 판정은
전체 검증 완료 전까지 머지 보류다.

실험 전 source로 복원한 뒤 같은 기존 위첨자 검사 12건이 모두 통과했다(exit 0).
이는 복원 확인이며 기존 전체 nextest의 #1139 1건·#1308 2건 실패를 해결한 결과가 아니다.

## 원본 90% 보정을 계속하는 다음 구현 단계

사용자의 재확인에 따라 미달 후보의 보류 기록을 작업 종료로 사용하지 않는다. 원본 두 수식과
#1139 대조군을 독립 Print에 맞게 90% 이상 보정하고 확대 판독을 마친 뒤 검사 변경·전체 검증을
진행한다. 관련 자료는 이미 보존했으며 이후 source의 결과를 새로 산출한다.

원본 HWP의 raw `atno` 레코드에는 번호 11·앞 장식 `문`·뒤 장식 `）`가 있고 HWPX도 같다.
HWP parser의 자동 번호 할당이 이를 1로 덮고, 본문 composition은 표제 표시를 생략한다.
실제 note 본문에 귀속한 번호의 순차 할당과 귀속 없는 원본 표제 번호의 보존을 구분하고,
control 표시 투영을 원문 문자 위치·측정·배치에 연결한다. 공백 검색으로 텍스트를 덧붙이지 않는다.

수식의 common.margin.left/right는 각각 56 HWPUNIT인데 composer의 TAC 메타데이터 이후
측정·최종 배치에서 그림/도형에만 바깥 여백을 가산한다. 독립 Print에서 연속한 수식 이후 글자
원점의 차이는 앞 수식들의 좌우 여백과 현재 수식의 왼쪽 여백으로 설명된다.
`flow_metrics_hwp`의 내용 상자 → 공통 인라인 점유 상자 → FlowInlineControl/TAC 폭 →
EquationNode의 여백 안쪽 원점·내용 폭 → 각 paint backend로 연결한다. 저장 캐시의 유효성
비교에는 내용 폭을 유지하며, 바깥 여백의 추가를 글자 폭 재측정으로 잘못 취급하지 않는다.

기준선·첨자 후보도 수식 본체와 첨자의 실제 Print 배치로 다시 검토한다. 단순 지수의 기존
상단 계약이 독립 Print와 같은지 확인하고, 원본 90%를 충족하기 전에 실패를 없애려고
테스트를 변경하지 않는다. 새로운 source에서 HWP/HWPX 전체 1쪽, #7593 두 대조군, #1139
영향 페이지를 다시 캡처하고 review PNG와 TSV·최저값·남은 차이를 연결한다.

좌우 여백과 원본 번호의 표시를 연결한 Native 선행 대조는 HWP/HWPX 모두 89.37923%였다.
원본 내용·Print·임계값을 유지하고 남은 글자 모양과 괄호 간격을 보정한다.

기본 수식 face로 선언한 Latin Modern Math가 이 호스트에 없어서 Times New Roman의
합성 이탤릭이 그려지는 점을 실제 browser face로 확인했다. 기존 배포 WOFF2와 같은 face를
사용자 시스템 폰트로 등록했고, 이후 캡처는 이 변경을 font 환경에 기록한다. 새로운 폰트나
비공개 한컴 폰트를 저장소에 추가하지 않으며, 기존 GUST 라이선스·배포 face를 그대로 사용한다.
변수의 이탤릭/굵기는 [MathML Core의 수학 알파벳](https://www.w3.org/TR/mathml-core/#mathematical-alphanumeric-symbols)의
표준 Unicode glyph로 표시해 합성 기울임의 중복을 피한다. 원본 script와 AST는 보존하고
공통 layout의 표시 glyph를 모든 paint 경로가 소비하도록 연결한다.
로만 `rm`·한글·함수 이름에는 이 표시 변환을 적용하지 않는다.

일반 괄호 토큰의 불필요한 연산자 여백과 ASCII 하이픈의 수학 빼기 표시도 구분한다.
앞뒤 관계 기호 여백은 유지하고 일반 괄호는 글꼴의 중립 전진폭을 소비하며, 빼기는 수학
minus glyph로 표시한다. 문서별 px·비율 조건이나 저장 프레임으로 글자를 늘리는 처리는 하지 않는다.
새 Native/fresh WASM 대조와 확대 판독을 수행해 90% 선행 조건을 충족한 뒤 검사를 변경한다.

### 수학 글리프·근호의 후속 보정

좌우 여백·표제 복구 후보에서 기본 수식 face를 실제 공급한 Native 결과는 89.79562%였다.
수학 변수의 표준 이탤릭 글리프, 중립 괄호와 minus 표시를 연결한 새 후보는 HWP/HWPX
전체 1쪽 모두 **89.91367%**였다. 반올림해 90% 통과로 기록하지 않는다. 기존 검사와
독립 Print를 유지하고 다음 원인을 계속 보정한다. 같은 배율의 확대 패널에서도 근호의
내용 시작과 연속 수식의 폭이 좁은 차이를 확인했다.

원본 HYhwpEQ의 수학 radical 전진폭은 1em이고 수학 숫자는 0.5em이다. 배포
Latin Modern Math의 U+221A 전진폭은 833/1000em이며 숫자는 역시 0.5em이다.
기존 추정은 각각 0.6em·0.55em이었으므로, 원본의 내용 폭과 paint 내부 원점이
일치하지 않았다. 같은 layout에서 source face의 논리 폭과 숫자 폭을 소비하도록 보정한다.
근 지수에는 MATH의 scriptscript 50%와 RadicalDegreeBottomRaisePercent 60%를 사용하고,
SVG·Canvas2D·Skia·CanvasKit이 같은 배치와 글자 크기를 소비한다.
문서 ID·절대 px 좌표·점수에 맞춘 조건은 추가하지 않는다.

이 후보의 source patch·새 helper·binary 해시와 실제 등록 face는 ignored
`output/pr-review/pr7600/math-glyph-provenance.json`과
`math-glyph-font-environment.json`에 기록했다. 기본 face 등록은 검증 환경 변경이며
새 폰트 배포가 아니다. 다음 보정은 별도의 binary 해시와 새 PNG/TSV로 검증한다.

근호 폭·숫자 전진폭·근 지수 크기를 보정한 Native 결과는 두 원본 HWP/HWPX가 모두
**93.97142%**였고, 공개 #7593 원본은 92.66991%, 최소 입력은 96.81195%였다.
같은 배율의 확대 판독으로 근 지수의 가로 원점도 확인해 MATH의 before/after degree kern을
공통 배치에 추가 연결했다. 본문 최초 composition과 편집 후 재조판은 같은 자동 표제 투영을
소비하도록 연결했으며, Skia는 수학 알파벳 glyph를 제공하는 실제 face를 선택한다.

#1139 원본 p15는 새 Native 후보에서도 **35.91820%**로 문항·문단의 단 소속 차이가 남았다.
이 입력의 px 고정 검사는 아직 변경하지 않았다. 메인터너 보정 범위의 후속 원인 분석을 계속한다.
MCP 2020 job `b4f7a1ba-7454-4b03-b269-b4b1c6db485a`의 다시 저장한 HWP는 저장 줄·높이
진단용 자료이며, 원본과 기준 Print PDF의 일치 판정에 대입하지 않는다.

작업 중 upstream/devel이 #7610을 포함한 `48ff4bb9353bf8d29b7e1c0ac44df9bc5800dce8`로
변경되어 진행 중인 fresh WASM 빌드는 최신 base 재정렬을 위해 종료했다(exit 143).
출력 공백이나 검사 실패로 중단한 것이 아니며, 이 실행은 완료 증거가 아니다. 새 head에서
Native/fresh WASM을 재산출하고 관련 모든 페이지의 90% 및 확대 판독을 확인한 뒤 회귀 검사를 변경한다.

### 최신 base 재검증과 BOX 수식 보정

최신 `upstream/devel` `48ff4bb9353bf8d29b7e1c0ac44df9bc5800dce8` 위로 기존
체리픽과 보정 commit을 재배치했다. source `a4e94f55c8b1812d6d234839e44893c3f6f574d3`의
Native와 fresh WASM을 새로 빌드해 비교했다. 원본 `eq-002.hwp`와 `eq-002.hwpx`의
전체 1쪽은 두 출력 경로 모두 **94.02474%**였다. 공개 #7593 원본은 두 경로 모두
92.66991%, 최소 입력은 96.81195%였다. 자동 점수 통과와 확대 판독 완료는 구분한다.

#1139의 `3-10월_교육_통합_2022.hwp` 전체 18쪽도 두 경로에서 산출했다.
18쪽 중 13쪽이 90% 미만이고 최저값은 p15 **35.91820%**다. p2/4/6/9/10/11/12/13/14/15/16/17/18이
미달이며, 이 상태에서 #1139의 검사·기대값·baseline은 변경하지 않았다. 원본과 독립 Print를
그대로 보존하고 실제 출력 보정을 계속한다. 페이지별 TSV·명령·빌드 해시는 ignored
`output/pr-review/pr7600/rebased-case-native-all/`과 `rebased-case-wasm-all/`에 있다.

p2의 같은 배율 review에서 한컴은 사각 테두리로 출력한 `BOX{…}`가 Native에서는
문자 `BOX`로 표시됨을 확인했다. 원본 raw 수식 script에 BOX가 7개 있으며 영문과
한글 payload가 모두 있다. 수식 fallback 표에 BOX가 없어 Text로 넘어가는 parser 결함이다.
독립 Print의 사각 테두리와 MCP 재저장본의 일반 글자 BOX 높이(BaseUnit 900, 상자 높이 1080)가
내용을 둘러싼 0.1em 여유의 근거다. 재저장본은 메트릭 진단이며 원본 대체 입력이 아니다.

수정 범위는 `DECORATIONS → EqNode::Decoration → EqLayout::layout_decoration`의
내용 폭·높이·기준선과 SVG/Canvas2D/Skia/CanvasKit의 동일 상자 테두리 출력이다.
JSON decoration 이름과 CanvasKit 지원 판정도 함께 연결한다. 위쪽 장식인 bar/dyad와
사각 테두리인 BOX를 구분하고, 원문 script·일반 장식의 상자 계약은 보존한다.
문서 ID·절대 px·점수에 맞춘 조건은 사용하지 않는다. 영문/한글·분수 BOX와 중첩 BOX,
일반 bar/dyad를 대조해야 하며, 원본 시각 선행 조건 충족 전에는 새 회귀를 추가하지 않는다.

초기 BOX Native 빌드는 기존 `release-test` 대신 `release`를 지정한 실행 오류로
중단했다(exit 143). 공유 캐시는 보존하고 올바른 profile로 다시 실행했다. 이 중단을
컴파일 결함이나 통과 증거로 기록하지 않는다. BOX 수정 후의 수치·확대 판독·회귀 결과는
새 source에서 산출하며 위 source의 점수를 재사용하지 않는다.

### 저장 줄의 표 진입 간격 누락 진단

BOX source `e23993cd2`의 원본 p2 재출력은 **69.62101%**였다. BOX 문자가
테두리로 복원된 것은 직접 판독했지만 표 높이·본문 위치 차이는 남았으므로 시각 완료가 아니다.
Studio TypeScript 검사는 exit 0이었다. Native만 산출한 BOX 후보를 fresh WASM 검증으로
보고하지 않는다.

p9의 답안 표 다음 빈 문단은 독립 Print와 원본 저장 사다리에 같은 위치를 기록한다.
원본 앞 글줄은 `vpos 0 + lh 900 + gap 452 = 표 vpos 1352`이고, 표는
`lh 16373 = 내용 높이 16093 + 위 여백 140 + 아래 여백 140`, 다음 문단은
`vpos 18177 = 1352 + 16373 + 452`다. 표의 실물 높이와 한컴 외곽선은 일치하지만
기존 Native 배치는 다음 문단을 사다리보다 280HU 아래에 놓았다.

실제 경로 `try_place_stored_tac_paragraph → stored_tac::prepare`에서 앞 글줄의
후행 간격을 제외한 수용 커서가 900HU이고 표 저장 원점은 1352HU여서 단일 저장 표의
원점 확인이 실패한다. 일반 fallback의 별도 간격/여백 계산이 선택되고, paint에서
후속 문단이 원본 위치를 넘는다. 유효한 앞 저장 글줄의 내용 끝이 현재 물리 커서와
같고 그 줄 끝·간격이 표의 저장 시작과 정확히 이어지는 경우, 누락된 후행 간격을
표의 공통 원점에 한 번만 연결한다. 이 원점에서 fit/end/paint가 모두 같은 결과를
소비해야 한다. 편집·무효 저장 정보·좌표축 불일치·가시 부동 개체가 끼는 경우는
이 앞 글줄 계약으로 정당화하지 않는다. 다음 단/쪽의 오래된 누적 vpos를 받아들이는
완화는 하지 않는다. 원본과 대조군의 실행 확인 후 선행 조건을 충족할 때 회귀를 추가한다.

앞 간격 후보 `160125f322027f70824621c283eaff345812df13`는 원본 p9/14/15에서
각각 67.24537% / 48.75776% / 35.91820%로 변화가 없었다. 실제 실행 경로를
다시 대조해 `layout_profile().hwp3_layout = true`, `native_hwp5_layout = false`를
확인했다. 원본 요약정보의 작성일 property20에 1997년이 남아 있고 출처 marker는 없다.
최종 저장 프로그램은 한컴 2022이며 현재 컨테이너는 HWP5다. 작성 이력과 현재의
저장 줄·개체 상자 계약은 별개의 근거이므로, 기존 HWP3 계보를 없애거나 가짜 원본으로
대체하지 않고 실제 HWP5 컨테이너 질의를 모델에서 제공한다. 저장 TAC 계획은 이 질의와
유효한 원문 저장 줄·전체 상자·같은 좌표축의 앞 글줄 증거를 함께 소비한다.

최초 앞 간격 후보의 fresh WASM 빌드는 원본에서 적용 경로가 제외된 사실을 확인해
후속 source 보정을 위해 중단했다. 이 빌드는 최종 검증 증거가 아니며, 새 source에서
다시 빌드한다. 조판 개선 완료나 회귀 검사 변경은 아직 선언하지 않는다.

### 저장 표 경로의 적용 확인과 미주 구분선 보정

source `5c6d605c0e1a2510afe1571ddc17ff40bd157477`의 Native를 새로 빌드했다.
p9 실행 trace에서 원본 표 417 뒤 문단의 끝이 336.8px에서 333.1px로 바뀌어
원본 저장 사다리의 18177HU와 일치했다. 이는 독립 Print와 대조한 관측 좌표이며
회귀 테스트의 절대 px 기대값이 아니다. p9 실루엣은 **72.53738%**로 개선됐지만,
p14 **48.90900%**, p15 **35.75830%**로 여전히 미달이다. 같은 배율 review PNG를
직접 열어 표 뒤 흐름 개선과 남은 미주 문단·수식 차이를 구별했다. raw 증적은
`output/pr-review/pr7600/container-stored-band-native-review/`에 보존한다.

원본 미주 구분선 종류는 10(굵은선-가는선 이중선)인데 기존 layout은 선 종류를
무시하고 단일 실선으로 만들었다. 독립 Print에서 이중 획을 확인했다. 일반 도형의
HWP 선 종류 변환을 공통 helper로 옮겨 구분선에서도 같은 LineRenderType을 소비한다.
또한 LineNode와 모든 painter의 획은 경로 중심의 양쪽으로 그려지므로 아래 여백은
경로에서 전 굵기만큼 내려간 지점이 아니라 아래쪽 잉크 경계에서 시작해야 한다.
`NoteSeparatorGeometry → EndnoteFlowProfile::separator_height_px / layout_endnote_separator_item
→ endnote_sep_body_floor → 실제 다음 문단`으로 같은 advance를 소비하도록 연결한다.
compact 경로의 pagination 예약 생략과 paint의 후속 floor는 별도로 대조하며,
visible/숨긴 선·단일/이중 선·위아래 여백 대조군을 확인해야 한다. 이 후보의 실제 결과는
다음 source 빌드에서 재산출하며, 점수에 맞춘 문서 조건이나 테스트 변경은 추가하지 않는다.

### 미주 제목과 본문을 묶는 조건의 독립 근거

원본 p15 문28 제목(source 본문 305, 미주 내부 0)의 문단 속성은
`keep_with_next = false`이고 한컴 Print는 제목을 왼쪽 단 하단에, 다음 본문을 오른쪽
단에 출력한다. Native trace에서 제목의 fit는 12px, 앞 흐름 높이 948.7px,
단 예산 1001.6px로 제목 자체는 들어가지만 `judge_default_question_title_tail_fits_by_line_height`가
다음 본문 문단 전체까지 들어갈 것을 요구하여 `query_new_note_fit`의 조기 단 넘김을
억제하지 못한다. 제목 자체의 수용 여부와 다음 문단의 수용 여부를 구분해야 한다.

해당 tail 판정에서 실제 resolved 문단의 `keep_with_next = false`이면 다음 본문의
전체 높이를 제목의 필수 fit 예산으로 묶지 않도록 보정한다. keep 속성이 있는 경우의
기존 제한과 다음 본문의 실제 fit/줄 분할은 보존한다. source ID나 px 좌표로 문28을
선택하는 조건은 추가하지 않는다. 저장 문단·다음 문단의 단 소속과 내용 보존을 Print와
다시 대조한 뒤 전체 대조군에서 의도하지 않은 제목 분리 여부를 확인한다.

### 단 이동 뒤 남은 예산으로 fit 분할 재판정

제목 속성 source `2a29815f7`의 Native는 문28 제목을 왼쪽 단으로 옮겼지만
실제 제목이 frame 아래에 남았으며, p15 실루엣은 **35.01950%**로 미달이다.
단 소속 개선만으로 완료를 판정하지 않는다. 같은 배율 review에서 본문 한 줄만
오른쪽 단에 있고 나머지 본문·그래프가 다음 쪽에 밀린 사실을 확인했다.

원본 virtual 문단 806은 저장 글줄 3개다. `query_initial_fit`는 왼쪽 단의
현재 높이 966.7px·남은 높이 34.9px에서 cut 1을 만들고 `query_tail_fit`가
문단을 오른쪽 단으로 옮긴다. 이후 `emit_endnote_split`가 오래된 cut 1로
오른쪽 단에 첫 줄을 쓰고 다시 다음 쪽으로 이동했다. 시각 원인은 글자 존재나
수식 폭만이 아니라 **fit 후보가 소유한 단과 실제 emit 단의 불일치**다.

fit 계산의 (쪽, 단)을 기록하고, emit 전에 단이 바뀌었다면 새 단의 실제 남은
높이와 동일한 FormattedParagraph의 line advance로 fit 컷을 다시 정한다.
전체가 들어가면 컷을 없애고, 일부만 들어가면 실제 수용한 줄 끝을 컷으로 쓴다.
저장 vpos reset에서 나온 내용 컷은 이 예산 컷과 구분해 보존한다. 기존 단에
머리를 남기는 경우·새 단 전체 수용·새 단에서도 일부만 수용·예산보다 큰 첫 줄을
대조하며 유닛 누락·중복과 불필요한 빈 쪽을 확인한다. 원본 전체 18쪽의
Native/fresh WASM 및 정상 대조군 선행 조건 충족 전에는 정식 회귀를 변경하지 않는다.

단 예산 컷 source `e5600f806`의 Native 전체 18쪽을 재산출했다. p15는
**59.73313%**로 올라갔고 첫 본문 3줄과 그래프가 오른쪽 단에서 실제 복원됐다.
하지만 제목의 아래쪽 frame 초과와 앞 쪽에서 이어진 추가 두 줄은 남아 있다.
전체 최저값은 p14 48.90900%이며 14쪽이 미달이다. raw TSV는
`output/pr-review/pr7600/fit-split-native-all/case-fit-split/silhouette.tsv`에 있다.

전체 sweep에서 p12가 64.97128%로 내려간 것을 확인해 각 immutable Native binary로
같은 원본·Print p12를 재출력했다. 저장 표 source `5c6d605c0`와 구분선 source
`1a44ec182`는 둘 다 89.25743%이고, 제목 속성 source `2a29815f7`부터
64.97128%다. 작성자가 만든 제목 후보의 부작용을 다른 회귀 기대값 수정으로 숨기지 않는다.
해당 후보의 keep 속성 완화는 철회하고, 분할 예산 재계산은 별도로 유지한다.
속성만으로 fit를 허용하기 전에 paint의 실제 제목 원점·점유 끝과 예산 결과를
공통 결과로 연결해야 한다. 이 철회는 원본 제목의 정확한 위치 개선 완료가 아니다.

본문 CharShape 11의 라틴 face는 실제 `Haansoft Batang`이고 해당 언어의 HFT 반각
플래그는 false다. HWP3 계보 플래그만을 근거로 본문의 모든 라틴 간격 차이를 HFT로
분류하지 않는다. resolved face 진단은 ignored `hft-font-probe.txt`에 보존한다.

### 현재 결과와 수식 기준선의 생산·paint 연결

제목 완화를 철회한 source `6714204c4`의 Native 전체 18쪽에서 p12는
**89.25743%**로 복원됐다. p9는 74.05746%, p13은 53.95712%, p15는
35.75830%다. 단 이동 후 fit 컷 갱신은 유지했지만 원본 p15의 현재 기본 경로는
제목을 오른쪽 단에 배치하므로 해당 컷 재계산이 이 쪽에서 활성화되지는 않는다.
앞서 본문·그래프가 한 줄 뒤에서 다음 쪽으로 밀린 경우는 제목 후보
`2a29815f7`에서 새로 노출된 경로다. 그 문제의 해결을 원래 기본 경로의
그래프 누락 개선으로 확대해서 보고하지 않는다. 현재 원본 p15의 그래프는
오른쪽 단에 있고 한컴보다 제목 한 줄만큼 늦게 시작한다.

p13의 원본 미주 마지막 문단(본문 190의 내부 47, virtual 674)은 저장 vpos 0,
명시적 문단 쪽/단 나눔 없음이다. 독립 Print는 앞 식 다음 왼쪽 단 하단에
이 문단을 둔다. Native의 local rewind 조기 이동과 측정/paint 문맥 차이를
계속 추적한다. 또한 같은 줄의 그림은 한컴과 서로 다른 세로 기준선을 사용하므로
좌우 그림을 한 줄 상단에 놓은 것을 일치로 판정하지 않는다.

수식 배치 경로를 대조해 저장 수식 상자의 기준선(원본 간단한 식: 높이 900HU,
baseline 86)으로 EquationNode 원점을 정하면서 paint는 자연 LayoutBox의
0.8em 기준선에서 글자를 출력하는 차이를 확인했다. 기준선 필드 자체는 한컴
공식 HWP 5.0 표 105에 있으며, 단위 해석은 기존 inline_baseline_px의 계약을
그대로 소비한다. 별도의 수치 가정이나 문서 조건을 추가하지 않는다.

`flow_metrics_hwp → inline_baseline_px → EquationNode 원점`에서 얻은 같은 기준선을
공통 LayoutBox의 paint 원점에도 연결한다. 압축 프레임의 기존 scale을 고려해
`LayoutBox.y + LayoutBox.baseline`의 최종 위치가 해당 기준선과 같아야 한다.
본문/부분 문단·표/부분 표·도형 텍스트의 실제 Equation producer가 같은 helper를
소비하고 SVG/Canvas2D/Skia/CanvasKit이 같은 root 원점을 그린다. 무효·없는
기준선은 기존 자연 기준선을 쓰며, CJK 재계산 경로는 그 메트릭을 소비한다.
원본 HWP/HWPX·분수/첨자·압축 프레임·일반 장식 대조를 새 Native와 fresh WASM에서
재산출한 뒤 확대 판독하며, 아직 정식 회귀 기대값 변경은 하지 않는다.

### 같은 글줄의 그림 기준선 비교

수식 기준선 후보 `8c6ccc086`의 원본 eq-002.hwp Native는 **92.20872%**로
90%를 넘었지만 이전 94.02474%보다 낮다. 원본 10월 문서 p9/13/15도
73.28822% / 52.66234% / 34.96384%로 감소했다. 점수와 실제 기준선 판독을
분리하며 이 후보를 최종 개선으로 판정하지 않는다. 확대 비교와 동일 HWPX의
재출력, 저장 메트릭의 유효성 대조를 계속한다.

p13의 네 그래프는 독립 Print의 실제 image rect와 Native ImageNode를 직접
대조했다. 첫 줄 두 그림 높이는 209.6px와 186.8px인데 Native의 두 원점은
모두 294.0px였다. Print는 각각 290.72px와 310.06px다. 다음 줄은 Native
작은 그림이 681.1px, 큰 그림이 674.8px였고 Print는 696.20px / 671.11px다.
좌표는 진단 관측값이며 정식 회귀를 px로 고정하지 않는다.

빈 runs 경로에는 형제 그림의 0.85 어센트 차이를 반영하는 코드가 있지만
일반 run/미배치 TAC 경로는 각 그림의 전체 높이를 글줄 기준선에서 뺀 뒤
줄 상단으로 clamp해 서로 다른 그림을 같은 상단에 놓는다. 캡션·위아래 여백을
포함한 각 상자의 어센트를 공통 helper로 계산해 같은 글줄 기준선에서 원점을
정하고, 잘못된 개별 원점을 줄 상단으로 clamp하지 않는다. 기준선이 없는 줄은
그 줄 소유 그림들의 최대 상자 어센트로 구성한다. 문단의 다른 줄 그림을
형제로 쓰지 않도록 기존 `tac_offsets_for_line`의 실제 줄 집합을 전달한다.

단일/다른 높이의 같은 줄 그림·텍스트 혼재·명시적 다음 줄의 큰 그림·캡션과
바깥 여백을 대조해야 한다. 줄 구성/높이 소비의 일관성과 후속 내용도 확인한다.
현재 원본은 시각 미달이므로 새 테스트/fixture/golden은 추가하지 않는다.

수식 root를 저장 baseline에 무조건 맞춘 후보는 원본 eq-002와 10월 p9/13/15의
일치율을 모두 낮췄다. 저장 필드와 자연 glyph 메트릭을 연결해야 한다는 코드 관측만으로
저장 캐시의 실제 유효성을 입증하지 못했으므로 해당 paint 원점 후보를 철회한다.
기준선 필드의 존재를 확인한 한컴 공식 근거는 [HWP 5.0 표 105](https://cdn.hancom.com/link/docs/한글문서파일형식_5.0_revision1.3.pdf)이며,
이 표만으로 단위·새 glyph 메트릭과의 호환성을 추가 가정하지 않는다. 실제 Print와
저장 메트릭의 대응을 확인해 후속 원인 보정을 진행한다. 새 그림 정렬 후보는
철회한 수식 root 이동을 포함하지 않는 source에서 다시 빌드한다.

### 저장 vpos 되감김과 명시적 단 나눔의 구분

원본 p13 마지막 미주 문단(본문 190, 내부 47)은 한컴이 저장한 정상 HWP의
단일 LineSeg vpos 0이고 column_type은 None이다. MCP 2020 재저장 진단에서도
해당 줄과 앞 두 줄의 저장 높이·기준선·간격은 그대로였다. 이 캐시를 임의 생성된
가짜 입력으로 분류하거나 재저장본으로 원본을 대체하지 않는다. 독립 Print는
이 문단을 앞 식 뒤 왼쪽 단에 둔다.

기존 local rewind 이동은 문단의 명시적 나눔과 실제 fit를 함께 확인하지 않고
저장 vpos 감소와 단 사용 비율만으로 다음 단으로 이동한다. 명시적 나눔이 없고
현재 FormattedParagraph의 콘텐츠 fit가 기존 단 예산/허용 경계 안에 들어가면
vpos 감소 하나만으로 조기 단 이동을 요구하지 않도록 보정한다. 뒤 내용은
같은 formatter와 현재 단 상태에서 기존 emit/paint 경로를 계속 소비한다.
무조건적인 좌표 clamp나 문서 ID·고정 px·줄 개수 예외는 추가하지 않는다.
명시적 나눔과 실제 내용 overflow가 있는 대조군, 뒤 제목·전체 쪽수·단 소속과
내용 누락/중복을 원본 Print와 대조하며, 측정과 최종 paint가 다른 경로는
추가 보정 대상이다. 시각 기준 충족 전 회귀 기대값은 변경하지 않는다.

`6ad9daf13` 후보의 전체 Native 재출력은 17쪽으로 줄어 독립 Print 18쪽과
일치하지 않았다. p13에서는 그래프가 쪽 상단으로 이동하고 오른쪽 단의 일부
내용이 용지 아래로 내려갔다. `FormattedParagraph.height_for_fit`와 누적 상태만으로
저장 vpos 단 이동을 해제할 수 있다는 가정은 실제 배치 경로의 높이를 보장하지
못한다. 이 조건을 철회하고 기존 단 이동을 복원했다. 원본·후보 바이너리와
페이지 수 불일치 로그는 ignored `output/pr-review/pr7600/`에 보존한다.
이는 기대값을 바꾸거나 원본 출력이 개선되었다고 판정할 근거가 아니다.

### 단 되감김 판단의 실제 paint 문맥 보정

앞 후보의 단독 fit 가정을 제거한다. 현재 단 항목과 후보 문단을 함께 실제
`build_single_column` 경로로 재생해 fit를 확인한다. 기존 scratch는 로컬 문단으로
재색인하고 기본 프로파일·빈 원본 소유자·미주 사이 여백만 사용했다. 같은 함수를
호출해도 실제 배치의 본문/미주 구분과 trailing 간격을 소비하지 못했다.

전역 문단 식별자, 원본 소유자 매핑, source 프로파일, 구분선 위·아래/미주 사이
여백, contextual composer를 전달한다. 아직 emit하지 않은 다음 내부 문단의
존재는 원본 Endnote에서 확인해 trailing 간격을 실제 paint와 같이 결정한다.
`measure_endnote_column_bottom`의 base=0 덮어쓰기를 제거한다. 실제 배치의
단 점유 끝점이 fit하는지 확인하고 명시적 단 나눔은 보존한다.

이는 현재 scratch의 표·float·복수 zone 문맥까지 동일함을 주장하지 않는다.
해당 문맥에는 여전히 표 측정/단 원점 계약 대조가 필요하다. 원본 모든 쪽의
단 소속·전체 쪽수·용지 밖 내용과 독립 Print를 다시 대조하며 새 회귀는 보류한다.

`8c10d28b6` 전체 Native는 18쪽을 유지했으나 p13/14/15가
41.94810% / 39.07549% / 32.66797%로 감소했고 최저값은 p18 30.19743%였다.
원본 소유자·프로파일 전달의 누락 보정과 저장 단 이동을 임의 해제하는 정책을
구분한다. 앞의 문맥 보정은 유지하고, 실제 scratch 끝점만으로 저장 단 나눔을
해제한 조건은 철회한다. source의 저장 단 경계와 Print의 재조판 줄 높이가
갈리는 원인을 먼저 확인해야 한다. 같은 함수 호출·전체 쪽수 유지·일부 문단 fit가
원본 일치를 입증하지 않는 반례를 새 후보의 TSV와 PNG로 보존했다.

### 수학 글리프 전진폭과 표시 문자열의 일치

기존 `layout_text`는 소문자 0.55em·대문자 0.65em에 이탤릭 1.05를 곱하고,
paint는 Unicode 수학 이탤릭/굵은 이탤릭 글리프를 사용했다. 로만 함수도 같은
소문자 추정값으로 측정했다. 실제 배포 Latin Modern Math의 전진폭은 변수마다
다르므로 이 추정으로 함수 내부·위 장식 폭과 뒤 식 원점이 어긋날 수 있다.

배포 공개 글꼴의 cmap/hmtx(UPM 1000)에서 ASCII와 수학 이탤릭·굵은 이탤릭
글리프의 전진폭을 추출한다. AST 원문을 보존하고 실제 표시 문자열을 같은
글리프 메트릭으로 측정한다. 지원하지 않는 문자와 한글 혼재의 기존 별도 계약은
보존한다. 예를 들어 `rm AB`의 전진폭은 1.3em 추정에서 실제 1.458em으로,
함수 `cos`는 1.65em 추정에서 실제 1.338em으로 바뀐다. 수치의 근거는 특정
문서의 좌표가 아니라 함께 제공하는 공개 글꼴이다. 모든 painter가 같은
LayoutBox를 소비하며 새 Native·fresh WASM/Print 대조 전 완료로 판정하지 않는다.

### 실제 수식 상자와 저장 줄 점유의 재발행 후보

MCP 2020 재저장 진단은 원문 수식 스크립트 1535개를 보존하면서 56개 수식의
상자/기준선을 다시 계산했다. 예를 들어 원본 미주 22의 `f prime (4)=0`은
높이 1125·기준선 78에서 높이 900·기준선 86으로 바뀌었으나 같은 문단의
저장 LineSeg 높이는 1125로 남았다. 재저장본 Native p9/13/15도
72.49952% / 55.98329% / 36.29902%로 미달이므로 이를 원본 통과로 대체하지 않는다.

보호되지 않은 실제 수식 상자의 높이·기준선을 같은 AST 표시 메트릭으로 만든다.
변경된 상자를 포함한 저장 줄은 원래 캐시의 어센트/하강부 합집합과 저장 줄 높이가
정확히 대응하는 경우에만 새 상자의 합집합을 공통 ComposedLine에 발행한다.
실제 텍스트도 포함하고 그림·표·도형의 별도 계약은 적용 범위에서 제외한다.
별도 줄 높이·정렬을 임의로 낮추거나 px 좌표를 고정하지 않는다. 저장 폭의 전진
계약과 보호된 크기는 유지한다. formatter와 실제 paragraph layout이 같은 결과를
소비하는지, 이후 vpos 보정이 이를 덮어쓰는지 새 출력에서 확인한다.

이는 아직 보정 후보이며 원본/대조군 Native·fresh WASM 검증 전 완료 판정이나
회귀 기대값 변경의 근거가 아니다.

`d37755410`의 p13 진단에서 첫 미분 기호 줄의 실제 TextLine 높이는 12.7px,
실제 출력 전진은 18.7px로 바뀌었으나 다음 문단이 저장 vpos로 21.0px에 다시
배치됐다. formatter fit도 원래 15px를 소비했다. 생산한 새 높이가 후속 원점/fit에
전달되지 않았으므로 공통 helper 호출만으로 일관성을 달성했다고 보고하지 않는다.

미주 내부의 같은 저장 줄을 보존하는 경로에서 새로운 높이·기준선과 후속 vpos를
함께 가진 파생 문단을 만든다. 저장 단/쪽 되감김마다 투영 원점을 재시작한다.
이 파생 문단이 기존 formatter와 실제 가상 미주 배치의 입력이 되며 원본 편집 IR,
파일 바이트·텍스트·줄 소유·명시적 단 나눔을 바꾸지 않는다. 줄 수가 달라진 경로는
이 투영의 범위로 처리하지 않는다. 단독 좌표 clamp나 특정 문서 ID는 사용하지 않는다.
또한 글자 크기보다 작은 저장 프레임은 기존 세로 압축 출력 계약이므로 실제 상자
재발행에서 보존한다. 압축 프레임과 보호된 크기를 새 Glyph 자연 높이로 덮어쓰지 않는다.

### 프레임과 실제 점유·leading의 구분

`e14664d0b`는 p13의 fit/실제 전진을 함께 보정했으나 전체 Native에서 다른
쪽의 일치율이 감소했다. 모든 AST 자연 높이가 원래 한컴의 논리 프레임 높이라는
가정은 철회한다. 위첨자·분수·배열의 유효한 저장 프레임을 현재 fallback 글꼴의
자연 높이로 재정의하지 않는다. 한 글자 높이의 표현식과 같은 본체의 BOX에 남은
더 큰 캐시만, 앞서 원본/독립 MCP 재저장·Print로 확인한 논리 상자 계약으로 보정한다.
보호·LINE 모드·세로 압축·다줄 표현식의 계약은 보존한다.

원본 저장 줄은 여러 상자의 최대 논리 높이와 최대 기준선을 기록한다. 실제 배치는
기준선 정렬의 어센트/하강부 합집합을 점유하므로 그 합집합과 남은 leading을
함께 발행해야 한다. 논리 상자 높이와 물리 점유 높이를 더하면 같은 공간을 이중
예약한다. 새 점유 높이와 논리 높이의 차이를 leading에서 소비하고, 후속 vpos에는
높이뿐 아니라 간격 변화까지 투영한다. 예산/원점/후속 문단이 같은 결과를 소비하는지
원본과 대조군 전체 페이지를 다시 확인한다. 아직 원본 통과나 회귀 변경을 승인하지 않는다.

### 같은 미주의 전체 잔여 내용 fit

`0300ead46`의 p13 첫 미분 줄은 formatter와 paint 모두 18.0px 전진을 소비했고
다음 줄은 18.0px에서 시작했다. 미주 마지막 직전 누적도 976.0px에서 968.0px로
변했다. 관측 좌표는 진단용이며 회귀 기대값으로 고정하지 않는다.

단독 문단의 fit만으로 저장 단 이동을 해제했던 앞 후보의 가정을 제거한다.
현재 단 항목과 같은 미주의 **남은 전체 내용**을 동일한 원본 소유자·프로파일·
구분선 여백·시작점/간격으로 실제 배치한다. 뒤의 그림·다줄 문단까지 넘치면
기존 단 경계를 유지하며 명시적 나눔도 보존한다. 실제 emit과 재생의 미주 문단
시작점/compact 간격 변환은 같은 helper를 사용한다. 표·float·글상자처럼 현재
scratch 측정 문맥이 부족한 경로는 단 경계를 해제하지 않는다.

이 조건이 p13 마지막 문장을 왼쪽에 유지하고 다음 제목·p14/p15의 내용 소속을
Print와 맞추는지, 이전에 회귀가 난 p12와 전체 쪽수도 함께 대조한다.

`0300ead46` 전체 Native는 18쪽이며 p9/11/12/13/16/17이
76.83814% / 75.60168% / 89.99724% / 62.37671% / 65.24466% / 64.41011%였다.
p12의 89.99724%는 90% 미만이며 반올림해 통과로 취급하지 않는다. p5는
일반 자연 높이 재정의 후보의 78.66306%에서 92.79008%로 복원됐다.
이 단계도 전체 gate 미달이며 fresh WASM의 새 source 검증은 아직 없다.

### 미주 끝의 실제 점유와 후행 간격의 분리

`ba2e62d10`에서 p13은 91.01323%로 상승했지만 p14/15는
43.20895% / 32.76446%로 감소했다. 직접 PNG에서 문27의 마지막 항목과
문28 제목·본문이 쪽 아래에 남는 부작용을 확인했다. 전체 18쪽 중 13쪽은
90% 미만이며 이 후보를 완료나 회귀 검사 변경의 근거로 사용하지 않는다.

같은 미주 전체의 fit를 기존 24px bleed 범위까지 허용한 가정을 제거한다.
마지막 줄이 점유한 상자와 그 뒤의 줄간격·다음 미주 앞 간격을 실제 paint에서
별도로 반환한다. 같은 미주의 남은 내용은 점유 끝이 단 경계 안에 들어갈 때만
저장 되감김을 해제한다. 새 미주의 첫 문단은 이 계약의 적용 대상이 아니다.
빈 줄도 흐름 상자가 있으므로 가시 글자의 존재로 점유 여부를 판단하지 않는다.

생산 지점은 paragraph layout의 마지막 흐름 줄 상자이며 scratch column이
이 결과와 후행 간격을 포함한 전진 끝점을 함께 반환한다. 전체 미주 tail fit는
점유 끝점을, 기존 일반 흐름 측정은 전진 끝점을 소비한다. 현재 후보는 새
Native 전체 페이지와 p12/13/15의 직접 판독 전 미검증이다.

`b70c6d1c7`은 p13 92.69547%로 상승하고 다음 문제의 쪽 아래 넘침을
제거했다. p14/15/16/17/18은 48.85210% / 35.87996% / 65.24466% /
64.41011% / 55.50842%로 앞 후보의 추가 회귀가 복원됐다. 전체 18쪽 중
13쪽이 여전히 90% 미만이므로 완료가 아니다. 해당 Native 전체 TSV와
p12/13/15의 review PNG는 `output/pr-review/pr7600/endnote-occupied-tail-native-*`에
보존했다. 같은 바이너리의 A2/A3 환경 진단도 동일 점수로 관측했으며
플래그 변경을 구현이나 통과 근거로 삼지 않는다.

### 내용 맞춤 글상자의 줄 메트릭·외곽·뒤 문단 원점

원본 본문 p115 글상자는 저장 줄의 마지막 끝+상하 여백 19905 HWPUNIT과
그리기 현재 높이 19904, host 줄 높이 19905가 대응한다. MCP 2020 재저장은
같은 원문·설정에서 내부 BOX 줄 등을 재계산해 현재 높이 19197, host 높이
19198로 줄였다. Print p2의 글상자 하단과 뒤 문단도 원본 Native보다 위다.
단순 글꼴 차이로 분류하지 않으며 원본 파일은 보존한다.

한컴은 글상자 내용이 크면 높이를 자동 확장한다는 공식 계약이 있다
([한컴 글상자 도움말](https://help.hancom.com/hoffice/multi/ko_kr/hwp/insert/textbox/textbox.htm)).
현재 후보는 저장 내용 끝+여백과 현재/host 높이가 대응하는 TAC 사각 글상자만
파생 상태에서 갱신한다. 일반 고정 크기·보호·회전·스케일·세로쓰기·중첩 개체는
동일 계약으로 간주하지 않는다. 내부 줄은 미주와 같은 수식 줄 메트릭 투영을
소비하고, 외곽 높이와 host 줄·후속 저장 원점까지 같은 변화량을 전달한다.

소비 경로는 `compute_render_normalized`의 파생 문단 → 공통 측정/조판 →
`section_render_paragraphs` → 글상자 외곽 및 내부 배치다. 편집 IR과 저장 파일을
바꾸지 않는다. 저장 원점의 되감김에서 변화량을 초기화하며 특정 문서 ID나
픽셀 위치로 조건을 만들지 않는다. 원본/대조군 재출력 전 후보는 미검증이다.

`eb478b8a2`는 전체 점수와 p2/4 PNG가 이전 head와 같았다. 원본 p115의
저장 스케일은 1이 아니라 현재/초기 크기의 비율(폭 26078/22451,
높이 19904/4517)이어서 적용 조건에서 제외됐다. 일반 크기 변경의 파생
스케일과 실제 회전·전단을 혼동한 조건을 보정한다. 초기/현재 extent의
비율과 대응하는 축 스케일만 수용하고, 새 높이의 파생 스케일도 함께 갱신한다.
원인 없는 조건 추가나 효과 없는 후보를 완료 근거로 삼지 않는다.

`f11034f75`도 점수가 같았다. 저장 y 스케일을 초기 높이 4517에 곱하면
19905 HWPUNIT이지만 현재 높이 필드는 19904다. 동일한 내용 끝/host와
대응하는 1 HWPUNIT 정수 저장 차이를 실수 스케일의 epsilon만으로 비교해
거절했다. 비교는 px가 아닌 extent HWPUNIT에서 수행하고, 새 축 스케일도
기존 저장 잔차를 보존하며 같은 줄 변화량을 전달한다. 고정 크기·보호·
회전·전단의 제외 조건을 단순히 제거하지 않는다.

### 미주의 문단 묶음과 후속 단일 줄 fit

Print p13에는 가상 문단 712의 "이때 구하는 경우의 수는"과 713의
"(ⅰ), (ⅱ)에서 구하는 확률은"이 있고 분수 문단 714부터 p14다.
현재 Native는 712부터 p14여서 92.69547% 통과만으로 소속 일치를
보고하지 않는다. 원본 643/674/712/713/714의 keep_with_next, keep_lines,
widow_orphan, page_break_before는 모두 false다. 해당 독립 관측과 원본
속성은 ignored `endnote-keep-contract-probe.txt`에 보존했다.

미주 전체의 잔여 내용을 하나의 keep 그룹으로 묶는 가정을 제거한다.
다음 문단과 묶지 않는 현재 문단은 실제 점유 끝이 단 안에 들어갈 때
수용하고, 이후 같은 단의 단일 줄도 실제 column replay의 점유로 fit한다.
후속 줄이 넘치면 새 단에서 시작한다. 앞 단독 문단 후보와 달리 24px
bleed로 점유 넘침을 허용하지 않는다. 실제 분할 컷이 필요한 다줄은 이
단일 줄 계약으로 보정했다고 보고하지 않으며 별도 검증 범위로 남긴다.
새 미주의 제목·명시적 break·scratch 문맥이 부족한 개체는 기존 계약을
보존한다. 원본 p12/13/14/15와 전체 페이지 재출력 전 이 후보도 미검증이다.

`e3756f23e` 전체 Native는 18쪽이며 p2 85.55529%, p13 93.82293%,
p14 69.49750%였다. p12 89.99724%와 p15 35.87996% 등 13쪽이
미달이므로 gate는 보류다. 직접 p13/14 PNG와 트리에서 712/713은
p13, 714는 p14로 Print와 같은 소속을 확인했다. p2의 글상자 보정도
실제로 적용됐으며 복합 BOX의 남은 높이 차이는 별도 원인이다.

내용 맞춤 글상자 p115의 일반 BOX 두 줄 외에도 sqrt/첨자 안의 BOX와
bar+BOX 행이 있다. 독립 MCP 재저장은 각각 1440→1296, 1305→1192로
프레임을 갱신했다. 공개 글꼴의 같은 합성 배치는 1280.7/1215 HWPUNIT이고,
단순 BOX 두 줄과 합친 높이 변화는 699로 독립 관측의 707에 근접한다.
BOX 본체가 한 글자 높이인 단일 표현식의 장식·괄호·근호·첨자 합성만
같은 프레임/줄 투영 경로로 수용한다. 분수·배열·큰 연산자·다줄·보호·
LINE·세로 압축의 저장 프레임을 다시 일괄 덮어쓰지 않는다.

14쪽의 원본 문단 751은 두 저장 줄 사이 vpos가 5408→0으로 되감긴다.
현재 Native는 첫 줄만 왼쪽에 남기지만 Print는 두 줄과 뒤 문단 752까지
왼쪽, 753부터 오른쪽이다. 같은 단의 실제 전체 문단이 들어가는 경우
내부 저장 컷도 재적합하고, 뒤 단일 줄은 같은 실제 점유로 fit한다.
명시적 break와 측정 문맥 부족은 보존한다. 새 후보의 전체 TSV·직접
PNG·정상 대조군 확인 전엔 완료나 회귀 변경의 근거로 쓰지 않는다.

### 제목·prefix 재적합 후보의 철회

`98556538090adf2262d78d6ca5e9a87ae6b929b0`의 전체 Native는 18쪽이지만
p12가 89.99724→65.82781%, p13이 93.82293→67.71258%로 악화했고,
p15는 34.87016%로 불변이었다. 새 제목을 실제 점유만으로 수용하고
후속 다줄을 재분할한 가정은 원본의 올바른 묶음·개체 소속을 보존하지
못했다. 해당 코드 변경을 철회한다. 원본 입력·Print PDF와 실패 TSV/PNG는
ignored `title-prefix-fit-native-all/`, `title-prefix-fit-native-review/`에
보존하며, 이 후보를 개선·회귀 변경의 근거로 사용하지 않는다.

### 전폭 첫 줄 뒤의 어울림 차선

원본 4쪽 문단 239의 저장 줄은 전폭 26788, 둘째 줄은 표 왼쪽 19220
HWPUNIT이며 문단 240 첫 보기 줄도 같은 19220 폭이다. vpos는
8962→10314→11666으로 각 900 높이+452 간격을 연속 소비한다.
Print도 보기를 표 옆에 두지만 Native는 표 바닥으로 흐름을 보내
후속 본문을 밀었다. 기존 #7548 판별의 첫 줄만 좁다고 가정한 부분을
수정한다. 기존 첫 줄 계약은 유지하며, 추가 경로는 원본 마지막 줄의
좁은 차선·host 내부/후속 줄의 연속 점유·수정되지 않은 저장 분할을
함께 확인한다. 합성 줄·저장 되감김·끊긴 간격은 이 근거로 수용하지 않는다.

공통 결과는 `float_placement/square_lane.rs`에서 생산하고
`typeset.rs`의 표 옆 후속 흐름 예약과 `layout.rs`의 실제 표 뒤 커서가
같은 판별을 소비한다. 표의 실물 높이는 그대로 배타 영역을 소유한다.
문서 계보 전체를 native HWP5로 바꾸거나 좌표를 clamp하지 않는다.
전체 TSV와 해당 쪽 직접 PNG, #7548 정상 대조군 확인 전에는 미검증이다.

`6893adbd2` Native 전체 18쪽은 이전 `c70851e8a`와 점수가 같다.
4쪽 73.71093%는 개선되지 않았으므로 차선 helper 변경만으로 소비 경로가
해결됐다고 보고하지 않는다. #7548 언어 입력 14쪽 대조는 98.35994%다.
ignored `square-last-lane-*`의 TSV/PNG와 실제 Print 출력 진단을 보존한다.

14쪽 문24→문25의 실제 Print 프로필 호출을 추적했다. 직전 mixed 수식
문단의 콘텐츠 끝 868.72와 흐름 끝 895.17은 다르다. 미주 사이 1984
HWPUNIT을 흐름이 이미 소비했지만 `should_preserve_endnote_title_gap`이
다시 커서 뒤에 더해 제목을 921.63으로 밀었다. 독립 Print 제목은 900.09다.
보존 하한은 실제 마지막 줄 점유 끝+미주 사이로 계산하고 이미 소비한
흐름 끝보다 낮추지 않는다. 측정 값이 없는 경로는 기존 fallback을 보존한다.
같은 `build_single_column`을 호출하는 미주 scratch와 실제 출력에서 검증한다.
좌표 수치를 구현 분기나 clamp에 사용하지 않는다. 새 head의 전체·직접
비교 전 이 변경도 미검증이며, 제목 분할 휴리스틱을 다시 완화하지 않는다.

`1c07e7439` 전체 18쪽의 Native는 p14 87.50342%, p15 55.68394%로
개선됐고 p12 89.99724%, p13 93.82293%는 유지됐다. 원본·기준 PDF는
동일하며 ignored `note-gap-once-native-all/`, `note-gap-once-native-review/`에
전체 TSV·직접 review를 보존한다. 15쪽 문26/27의 시작 흐름은 개선됐지만
문28 제목은 여전히 다음 단 상단이어서 Print의 앞 단 하단과 다르다.
원본의 실제 출력 개선은 부분 결과이며 90% 통과·완료 판정이 아니다.

4쪽 표 커서와 문28 단 이동에서 분기 후보를 더 추측해 완화하기 전에,
표 차선 입력과 실제 단 이동 호출 위치를 환경 변수로 진단한다. 진단은
`RHWP_TABLE_DRIFT`, `RHWP_FLOW_BREAK_DEBUG`에서만 stderr에 출력하며
일반 출력·상태 전이에 영향을 주지 않는다. 로그는 ignored output에만 남긴다.

### 소비 순서와 제목의 자연 점유 하한

`86c447808` 진단에서 4쪽 표는 차선 helper 호출 전 `table_visual_shift`
분기가 흐름을 표 하단으로 결정했다. 저장 차선 판정을 이 덮어쓰기 앞에서
소비한다. typeset도 pre-text가 없는 host에서는 텍스트의 post 소유 항목과
별개로 같은 줄을 먼저 전진시키던 분기를 피하고 표 배타 밴드는 유지한다.
같은 helper 이름만으로 실제 호출이 공유됐다고 판단했던 누락을 보정한다.

새 미주 한 줄 제목은 이전 항목 실제 점유 끝+설정 미주 사이+제목 점유가
단 예산 안에 들어가는지 먼저 확인한다. 이 자연 하한과 실제 전체 replay가
모두 들어가고 keep_with_next/명시적 break가 없을 때만 기존 임의 head 묶음
판정에서 분리한다. 제목을 남긴 뒤 다음 내부 문단의 첫 줄조차 실제 점유
끝+저장 간격에서 안 들어가면 그 문단은 새 단에서 시작한다. 이전 후보의
전역 다줄 prefix fit·저장 컷 변경은 다시 적용하지 않는다. p12 문22 이동,
p12/13 그림 보존과 p15 문28 소속을 같은 원본 전체 TSV·PNG에서 확인한다.

`61d85ded4`의 전체 Native 18쪽에서 p4는 73.71093→94.48386%,
p15는 55.68394→85.18673%로 개선됐다. p12 89.99724%, p13 93.82293%,
p14 87.50342%는 유지됐고 #7548 언어 14쪽 대조군도 98.35994%다.
최신 review PNG에서 p4 보기의 표 옆 배치와 후속 문단 순서, p15 문28
제목의 왼쪽 단 하단 소속과 뒤 본문의 오른쪽 단 시작을 직접 확인했다.
전체 TSV와 영향 페이지 PNG는 ignored `square-head-natural-fit-*`에
보존한다. 11쪽이 90% 미만이므로 완료·회귀 검사 변경 판정은 보류다.

### 저장 줄의 실제 콘텐츠 전진과 다음 줄 상자

원본 미주 8의 첫 줄은 수식 높이 900, text_height 900, baseline 774이며
다음 원점은 900+452만큼 전진한다. 그러나 줄 상자 높이 2385는 다음 줄의
cases 수식 높이와 같다. 기존 미주 paint는 저장 전진을 얻어도 TextLine의
큰 상자 높이를 다시 하한으로 소비했다. 원본의 수정되지 않은 줄 소유,
현재 수식·텍스트의 논리 높이와 기준선, text_height, 다음 저장 원점의
연속 간격이 함께 일치하는 경우에만 현재 줄의 실제 점유를 재발행한다.
표·그림·도형을 함께 소유하거나 편집·합성·끊긴 간격은 수용하지 않는다.
`composer/equation_line_metrics.rs`의 결과를 실제 paint와 측정이 쓰고,
`stored_row_projection.rs`는 다음 저장 원점이 이미 새 전진을 나타내면
후속 원점을 다시 당기지 않는다. 같은 메트릭을 미주와 내용 맞춤 글상자
투영도 소비하므로 전체 출력과 해당 대조군을 다시 확인한다. 새 후보는
빌드·전체 TSV·직접 PNG가 끝나기 전까지 미검증이며 회귀 기대값을 바꾸지 않는다.

16쪽 문29의 실제 `HeightCursor` 호출에서도 간격을 중복 소비했다.
직전 줄의 점유 끝 351.1 뒤 1984 HWPUNIT을 이미 소비한 흐름 377.5에,
stale 저장 원점을 접는 분기가 다시 간격을 더해 제목을 404.0에 뒀다.
`endnote_spacing.rs`의 점유 끝+간격과 기존 흐름 중 큰 값을 커서·제목
보존·textless 지연 복원이 함께 소비하도록 한다. 측정 값 없는 경로의
기존 fallback은 유지한다. 이 관측 좌표는 설명 근거이며 구현 상수가 아니다.
원본 전체와 문28/29 경계의 실제 출력 재확인 전에는 이 후보도 미검증이다.

10쪽 문14 제목은 순차 원점을 보존했지만, `HeightCursor`가 앞서 더한
수식 tail gap을 포함한 중간 result로 base를 다시 덮어썼다. 제목은 758.4,
본문은 768.4로 겹쳤으며 Print는 제목 758.3, 본문 776.4다. 최종 반환이
순차 원점인 경우 기존 저장 목표 end_y와 그 원점의 차이로 base를 정한다.
중간 보정을 소비하지 않는 반환에 그 보정량을 섞지 않는다. 문서 번호나
8px 상수를 추가하지 않으며 전체·해당 경계 재출력으로 영향 범위를 확인한다.

`4b113e757`의 전체 Native는 9쪽이 77.08186→90.74875%로 개선됐다.
원본 수식 HWP/HWPX 93.79351%, 공개 dyad 원본/최소 입력
92.85502%/96.30967%의 정상 대조도 유지됐다. 최신 9쪽 PNG에서
cases 앞 줄과 뒤 문항의 점유·순서를 직접 확인했으며, 미세 수식 폭의
남은 차이를 점수 통과로 면제하지 않는다. 증거는 ignored
`stored-content-cadence-*`와 `cadence-middle-before-gap/`에 보존한다.

`26fefe5b4`는 16쪽을 65.24466→71.30975%로 개선했지만 14쪽이
87.50342→83.92697%로 악화했다. textless 지연 복원은 다음 미주의
저장 기준을 복구하는 별도 계약이므로 그 변경은 철회한다. 실패 TSV와
14쪽 직접 PNG도 보존한다. 실제 점유+설정 미주 간격 외에 고정 8px를
추가하던 수식 tail 제목 보정은 제거하고 전체·해당 경계에서 검증한다.
이 후보도 기존 회귀 기대값이나 승인 판정으로 대신하지 않는다.

미주 26의 한글 포함 수식은 `flow_metrics_hwp`가 대체 글꼴 결과를
발행했지만 줄 구성은 특정 simple-math helper의 실행 여부만으로 변경을
판정했다. 실제 발행 높이·기준선의 변경을 소비하도록 한다. 원본 351의
내부 문단 3은 저장 높이 1305/기준선 1057, 수식도 1305와 81%이며
대체 메트릭은 1125/945다. 현재 줄이 그 개체 합집합에서 나온 경우에만
기존 공통 재발행 경로를 적용하고, 별도 저장 줄 높이 계약은 보존한다.
같은 콘텐츠를 측정과 paint가 다른 상자 높이로 소비하는 누락을 보정하며,
복합 수식의 유효한 저장 프레임을 다시 일괄 교체하지 않는다. 전체와
원본 수식·dyad 대조군의 재출력 전 이 후보도 미검증이다.

`00c987712`는 10쪽을 89.35277%, 14쪽을 87.53676%로 개선했다.
후속 `ba95662eb`는 11쪽 82.50469%로 개선했지만 14쪽 83.03647%였다.
`ccd25e3d1`은 17쪽을 62.21764→64.41011%로 부분 개선했고 원본 수식과
dyad 대조군은 각각 93.79351%/92.85502%/96.30967%를 유지했다.
모든 결과는 해당 immutable binary와 전체 TSV·PNG에 연결하며 미달은 보류다.

14쪽 제목 보존의 최종 base 덮어쓰기를 고친 뒤에는 지연 복원의 소비
관계도 다시 확인한다. 이전 후보의 분류만으로 그 경로를 확정하지 않는다.
제목 앞 순차 원점이 실제 점유 끝+주입 간격을 이미 예약한 경우, 작은
전방 저장 이동도 간격을 추가로 소비할 근거가 아니다. 이 상태를 기존
새 제목 경계의 base 재정렬 조건에 포함한다. 실제 점유가 없는 경로와
별도 하단·spacer 계약은 유지한다. 지연 복원에도 같은 점유 하한을 다시
적용해 최종 base 보정과 함께 전체·직접 PNG로 검증하며 좌표 상수를 쓰지 않는다.

### 수학 기호의 실제 글리프 전진폭

Latin Modern Math의 같은 cmap/hmtx에서 수식 명령 표의 비 ASCII 기호 191개의 전진폭을 추출했다(UPM 1000, asset SHA는 `glyph_advances.rs`와 같다). 그리스 문자·무한대·관계 기호의 추정 폭 대신 실제 표시 글리프의 폭을 배치와 장식이 함께 소비하며, 원본 AST·저장 프레임·없는 글리프의 기존 대체 경로를 유지한다.
숫자·문서 좌표를 맞춘 값이 아니라 제공 글꼴의 em 메트릭이다. source 매핑과 추출 증거는 ignored `symbol-advance-candidate.json`에 보존하고, 새 head의 전체 Native·대조군·직접 PNG·fresh WASM 확인 전까지 미검증으로 남긴다.

`e2639d618` Native 전체에서 p12는 90.21589%로 통과했고 p10은 89.83646%,
p14는 88.41588%였다. 원본 수식 HWP/HWPX는 모두 95.10894%, dyad
원본/최소 입력은 92.90434%/96.63596%다. 전체의 미달은 해소되지 않았다.

### 한컴 논리 폰트 프레임과 글리프 상자의 구분

독립 MCP 2020 재저장본과 원본은 수식 1535개의 순서·스크립트가 같고,
56개 프레임/기준선이 달랐다. 일반 글자 줄은 1125/78→900/86,
단일 위첨자 줄은 1163/79→1050/88, bar+prime 줄은 1305/81→1080/88이다.
원본과 재저장본의 SHA 및 기존 Print 증거를 유지하고 이 입력으로 원본을 대체하지 않는다.
`logical_font_frame.rs`는 같은 BaseUnit에서 이 논리 프레임을 발행한다.
대체 글꼴의 intrinsic glyph 높이를 한컴의 흐름 상자·기준선으로 잘못
소비한 부분을 보정한다. 보호·LINE·미지의 크기·분수·배열·아래/복합 첨자·
근호·다줄·다중 장식의 계약은 유지한다. 구성된 Row는 같은 기준선 합집합을 쓰며,
실제 flow 메트릭·composer·줄 투영·미주 scratch와 paint가 그 결과를 소비한다.
반복 진단은 같은 `target/pr-review`의 debug 캐시도 재사용하며 profile을 기록한다.
최종 Native/fresh WASM·독립 Print 비교 전에는 이 후보도 미검증이다.

### 새 미주 제목의 자연 점유 하한과 단순 분수 프레임

`8eaabc755`의 원본 18쪽 Native 진단에서 17쪽은 92.40990%였지만,
문30 제목이 마지막 수식과 겹쳤다. 제목을 남기는 기존 묶음 판정이
새 제목의 자연 점유 하한 확인을 우회했다. 기존 판정 결과와 관계없이
실제 앞 내용의 점유 끝 + 설정 미주 간격 + 제목 높이가 단 예산을
넘으면 새 단으로 보낸다. 제목 유지 경로에서도 같은 하한을 소비한다.
문28의 정상 하단 제목과 문30의 이월을 함께 직접 확인한다.

같은 스크립트의 독립 재저장에서 `{f'(0)} over {3} =2`는
2295/69→2070/66이었다. 분자와 분모가 각각 단일 글자 줄인 분수에만
BaseUnit의 2.3배 높이와 66% 기준선을 적용한다. 중첩 분수·근호·
첨자 분자/분모·배열·복합 수식은 이 근거로 확장하지 않는다.
14쪽 문24 풀이와 문25 시작의 간격, 전체 원본 및 정상 대조군에서
검증하며, 새 회귀 기대값과 최종 승인으로 대신하지 않는다.

`d6fa1241f` Native 전체에서 18쪽은 97.33508%로 개선됐으며 문30의
제목과 마지막 수식 겹침은 해소됐다. 그러나 16·17쪽은 각각
55.25904%/47.24028%로 악화했다. 실패 TSV와 review PNG를 보존하고
분수 프레임 변경을 철회해 이월 판정과 영향을 분리한다. 14쪽 제목
900.1은 Print와 맞아도 전체 원본의 악화를 허용하지 않는다.
정상 수식 HWP/HWPX와 공개 dyad 원본/최소 입력은 모두 90% 이상을
유지했지만 원본 전체의 미달을 대체하지 않는다.
독립 재저장에서 8pt의 정상 분수 1890/65는 바뀌지 않았다. 단일
9pt 사례로 모든 크기의 프레임을 2.3배라고 일반화한 가정은 철회한다.

분수 후보를 철회한 `5f2a35e3d`에서도 16·17·18쪽은
51.32912%/50.65549%/55.55627%였다. 자연 점유 하한의 이월을
추정만으로 강제한 변경도 원인이다. 같은 scratch에서 제목 추가 전후의
실제 점유를 대조하고, 하단 보정이 제목을 앞 내용 안으로 되돌린 경우를
확인한 뒤 이월한다. 원점 예산과 실제 출력의 호출 계약을 구분하며
전체 원본에서 다시 확인한다. 실패한 후보는 승인 근거로 사용하지 않는다.

### 연속 저장 수식 줄 뒤의 추정 빈 밴드 제거

16쪽 미주 29의 내부 문단 13→14는 저장 원점 차이가
4410 + 452 = 4862 HU다. 실제 수식 줄은 y=946.773, 높이는 58.8이며,
순차적인 다음 원점은 1011.600이다. 그러나 HeightCursor가 수식 높이의
40%를 추가 바닥으로 추정해 다음 문단을 1026.933으로 밀었다.
독립 Print의 마지막 본문 기준선은 1025.760인데 Native는 1045.146이었다.
이 지연은 뒤 문항의 이월에도 영향을 줬다.

수정되지 않은 연속 저장 줄의 다음 원점이 앞줄 높이와 간격의 끝에
일치하고 실제 점유 측정값이 있으면 그 점유 결과를 소비한다. 측정 없이
수식 높이 비율로 만든 밴드를 같은 경로에 중복해서 추가하지 않는다.
실제 배치와 미주 scratch가 모두 같은 HeightCursor 결과를 소비한다.
저장 정보가 단절된 경우, 편집 경로와 측정 불가 경로의 기존 fallback은
유지한다. 원본 전체와 해당 경계의 직접 PNG를 확인하며 회귀 변경의
시각 선행 조건도 지킨다.

크기별 합성 분수의 MCP 2020 HWP 저장과 Print 출력도 진단했다.
저장 결과가 제공된 9999/50 캐시를 그대로 유지했으므로 이 저장본을
새 논리 프레임의 독립 기대값으로 사용하지 않는다. job, Print 모드와
입력·산출 SHA는 ignored `fraction-font-size-diagnostic` 기록에 보존한다.

`086bcb754`의 직접 Print SVG에서 16쪽 마지막 본문 기준선은
1025.747로 독립 Print의 1025.760과 일치했다. 문29 제목도 기존
387.135에서 383.067로 개선됐다. 그러나 뒤 제목의 순차 원점 1061.587을
저장 목표 1041.160과 하단 backtrack으로 1037.160까지 당겨 설정된
미주 간격을 지웠다. 이에 문30 뒤의 흐름이 한 줄씩 앞당겨졌으며,
전체 TSV의 16·17·18쪽은 여전히 미달이었다.

실제 점유 뒤에 주입된 설정 미주 간격이 이미 예약됐으면 그 순차
원점을 유지하고 최종 원점에 맞춰 저장 base를 재정렬한다. 기존
하단 보정의 이름이나 앞 문단의 한 줄 여부로 실제 간격 소비를 부정하지
않는다. 단 예산은 앞 내용의 실제 점유, 설정 간격과 제목으로 판정한다.
겹침 여부만 검사해 간격 축소를 허용했던 후속 후보도 철회했다.

### 본문 참조 번호와 한 줄 미주의 실제 수용

독립 Print의 문3 참조 번호는 본문 9 pt의 3/4 크기이며, 기준선은
본문의 459.658보다 위인 457.101이다. '문'은 돋움, 숫자와 장식은 바탕을
쓴다. 번호 끝인 x=57.249에서 본문이 바로 시작하며 원문에 없는 공백은
없다. 기존 본문 prefix는 본문 크기로 번호를 쓰고 공백까지 추가해
뒤 글자를 x=71.013으로 밀었다. 동일 resolver의 언어별 런을 측정과
표시에 사용하고 선언 크기의 3/4 및 줄어든 어센트로 참조 번호를 표시한다.
미주 영역의 번호 옵션과 구분한다. 숫자의 바탕 적용은 확인했으나,
전각 닫는 괄호의 언어 슬롯은 Print와 추가 대조가 필요하다.

`8159cdf7f`의 16쪽 문30 제목 기준선 1071.187과 첫 본문 기준선
1089.933은 Print의 1071.840/1089.920에 근접했지만, 다음 내부 문단의
한 줄 '조건 (가)에서'가 1107.840에 남아 잘렸다. 실제 단 수용 검사는
앞서 compact rewind를 수용했다는 상태에만 묶여 있었다. 한 줄 미주
후속 문단은 그 분류와 무관하게 같은 실제 전체 replay에서 점유가
단 예산을 넘으면 이월하도록 바꿨다. 명시적인 나눔, 다줄의 분할 계약과
측정 불가 경로는 유지했다. 그러나 `e203275ba`의 진단에서 해당 줄의
scratch 점유 끝은 994.507로 단 예산 1001.560 안에 있다고 판정됐다.
실제 paint와의 차이가 남았으므로 이 변경만으로 해결됐다고 판단하지 않는다.

### 미주 간격의 예약 판정과 원점 이동의 반올림

같은 원본의 Print 출력 경로를 `RHWP_VPOS_DEBUG`로 대조했다.
16쪽 제목 직전의 scratch 순차 원점은 970.880이며, 실제 출력 원점은
본문 상단을 더한 1061.587이다. scratch는 제목 뒤의 저장 base를 793667로
바꿨고, 실제 출력은 791835로 바꿨다. 그 결과 scratch의 다음 한 줄 점유
끝은 994.507이지만 실제 출력에서는 잘림이 발생했다. 진단 로그는
ignored `output/pr-review/pr7600/single-line-vpos-diagnostic.log`에 보존했다.

간격 예약 여부의 엄격한 부동소수점 비교는 같은 점유 끝과 간격을
더해도 원점 이동에 따른 반올림으로 경계 판정이 달라질 수 있다.
예약 여부 비교에 각 피연산자의 크기와 `f64::EPSILON`으로 계산한
반올림 범위만 적용한다. 문서의 절대 px 여유나 시각 허용치를 추가하지
않으며 실제 점유·간격·단 예산을 변경하지 않는다. 이는 측정과 paint의
판정 일치 여부를 확인할 구현 후보이며, 새 원본 출력과 전체 TSV로
검증하기 전에는 완료로 판정하지 않는다. 정식 회귀 검사·기대값은
아직 변경하지 않았다.

`ff6d0cb0d7bfe1a9feff39fbc39b82a79af42131`의 Native 재출력 전체 TSV는
`output/pr-review/pr7600/gap-roundoff-debug-all/case-gap-roundoff-debug/silhouette.tsv`에
보존했다. 16쪽은 62.40037%에서 85.08534%, 17쪽은 47.22169%에서
92.56532%로 개선됐다. '조건 (가)에서'가 16쪽 오른쪽 단으로 옮겨져
Print와 같은 소속임을 새 `review_016.png`에서 확인했다. 전체 18쪽 중
6·11·14·15·16쪽은 여전히 90% 미만이며 정식 회귀 변경의 선행 조건을
충족하지 않았다. 진단용 Native dev 빌드 결과이므로 최종 Native 및
fresh WASM 검증을 대신하지 않는다. 이미지 출처는 같은 출력의
`gap-roundoff-debug-panels/capture-source.json`에 고정했다.

### 분수 자식의 기준선과 표시 글꼴 MATH 메트릭

14쪽 문24의 단순 분수에서 독립 Print의 분자와 분모 기준선은
856.320/872.000으로 간격이 15.680이다. 기존 Native는
858.120/870.600으로 간격이 12.480이다. 분자·분모 모두 글자 크기는
12이며, 저장 줄 원점 및 본문 기준선 차이와 수식 내부 간격 차이를
별도로 확인했다. 기존 구현은 분자와 분모의 전체 글자 상자를 고정
비율로 쌓아서 표시 글꼴의 기준선 이동과 괘선 주변 간격을 소비하지 않았다.

공개 `LatinModernMath-Regular.woff2`의 MATH 테이블을 직접 읽었다.
UPM은 1000이고 FractionNumeratorDisplayStyleShiftUp=677,
FractionDenominatorDisplayStyleShiftDown=686, AxisHeight=250,
FractionRuleThickness=40, 분자·분모의 표시 스타일 최소 간격은 각각 120이다.
원본 폰트 SHA와 추출값은 ignored `fraction-math-constants.json`에 보존한다.
[표준 분수 배치 규칙](https://www.w3.org/TR/mathml-core/#fraction-with-nonzero-line-thickness)에
따라 이 이동량과 자식 기준선·하강부가 요구하는 최소 간격 중 큰 값을
사용한다. 새 `fraction_geometry::position`이 자식 원점과 전체 점유·기준선을
동시에 생산하고 EqLayout의 같은 LayoutBox를 각 출력 backend가 소비한다.
수평 여백, 원본 저장 프레임과 크기 보호 계약은 이 후보에서 변경하지 않는다.

단순 분수뿐 아니라 중첩 분수·분수 안 첨자·정상 대조군의 실제 출력을
대조한다. 표준 수학 배치를 한컴의 저장 프레임 재발행 근거로 대체하지
않으며, 본문 이월이나 괘선 겹침이 생기면 같은 실제 소비 경로에서 다시
검토한다. 검증 전에는 개선 완료로 판정하지 않으며 정식 회귀·golden은
변경하지 않는다.

`a845f7184`의 전체 Native dev 비교는 18쪽을 유지했다. 14쪽은
88.33585%→88.75878%, 15쪽은 86.54484%→86.68841%, 16쪽은
85.08534%→85.18105%로 소폭 개선됐으나 11쪽은
89.06581%→88.90723%로 내려갔다. 원본 전체를 통과했다고 판단하지 않는다.
수식 HWP/HWPX 대조군은 각각 94.80866%, dyad 원본과 최소 입력은
96.53098%/96.63596%다. 새 직접 SVG에서 위 단순 분수의 분자·분모
기준선은 855.720/872.120으로 Print의 856.320/872.000에 근접했다.
분수 내부의 의미 개선과 페이지별 점수 변화를 구분한다.

### 극한 이름과 하한 크기의 측정·paint 공유

같은 독립 Print의 11·14·15·16쪽에서 lim 이름의 글자 크기는 14.4이며,
BaseUnit 9 pt인 수식의 기본 표시 크기 12보다 크다. 14쪽의 하한은
8.16이다. Native는 이름과 하한을 각각 12/8.4로 출력했다.
14쪽 첫 lim의 기준선은 Native 814.387, Print 816.000이며 하한 0의
기준선은 Native 823.507, Print 824.800이다. 크기와 기준선 모두
같은 영역에서 직접 대조했다.

HYhwpEQ 경로의 이름·하한 크기를 각각 BaseUnit의 1.2배/0.68배로
구성한다. 이름과 하한의 실제 글자 크기를 LayoutKind에 함께 기록하고
SVG·Canvas2D·Skia·JSON·CanvasKit이 이 값을 소비하도록 연결한다.
CanvasKit에서 하한까지 포함한 전체 상자 높이로 이름 글자 크기를
다시 추정하던 경로도 제거한다. 다른 수식 face의 기존 크기·하한 원점
계약은 유지한다. 하한은 표시 글꼴의 MATH LowerLimitGapMin=167,
LowerLimitBaselineDropMin=600과 자식 기준선으로 배치한다. lim/Lim
글리프의 하강부 0도 실제 폰트의 윤곽에서 확인했다. 이 메트릭은
UPM 1000 기준이므로 문서 좌표의 px 허용치를 넣지 않는다.

기존 golden 검사 원본은 enum의 추가 메트릭 필드를 무시하도록
패턴에 `..`만 추가했다. 검사 항목·기대값·golden은 바꾸지 않았다.
이 후보도 Native 전체·fresh WASM·정상 대조군과 확대 판독 전에는
완료로 판정하지 않는다.

`4c78d9a94`의 전체 18쪽 Native dev 비교에서도 6·11·14·15·16쪽이
90% 미만이었다. 14쪽은 88.89629%, 15쪽은 86.80945%, 16쪽은
85.00630%다. 수식 HWP/HWPX와 dyad 대조군은 모두 90% 이상을 유지했고
`npx tsc --noEmit`도 종료 코드 0으로 끝났다. 이는 Native 후보와
TypeScript의 확인이며 fresh WASM·CanvasKit 실제 실행을 대신하지 않는다.

### 보정된 메트릭에서 수식 paint 기준선 연결 재검토

극한 이름의 크기를 바꿔도 Row가 자식 기준선에 맞춰 원점을 재정렬하므로
14쪽 첫 lim의 기준선은 814.387로 유지됐다. 독립 Print는 816.000이다.
인라인 원점은 저장 수식 기준선으로 정하지만, paint는 별도의 자연
LayoutBox 기준선에서 시작하기 때문이다. 이 연결은 초기에 검사한
`8c6ccc086`에서도 후보였으나 당시 큰 원본 차이가 남아 있었고, 그 뒤
철회된 후보의 결과를 현재 코드의 근거로 그대로 재사용하지 않는다.

현재는 단순 글자·장식의 논리 폰트 프레임과 연속 저장 줄의 점유·미주
간격 예약 차이를 별도로 보정했으므로 같은 계약을 새 head에서 다시
검증한다. `flow_metrics_hwp → inline_baseline_px → EquationNode 원점 →
layout_for_paint의 루트 원점 → 각 backend의 표시 기준선`을 연결한다.
본문·부분 문단·표·부분 표·도형 텍스트의 실제 6개 수식 생산 경로를
같은 helper로 전환한다. 저장 프레임 압축과 기준선이 없는 경우의
기존 계약은 유지한다. 입력별 확대 판독과 전체 Native 결과를 확인한
뒤 부적합한 후보는 철회하며, 기존 실패를 기대값으로 고정하지 않는다.

`49c75769a`의 Native dev 전체 TSV에서 6쪽은 90.44428%로 통과했다.
15쪽은 86.80945%→88.37050%, 16쪽은 85.00630%→86.04084%,
17쪽은 92.55880%→93.17663%로 개선됐다. 그러나 12쪽은
90.03935%→89.33264%로 내려갔다. 11·12·14·15·16쪽이 미달이며
아직 승인 조건을 충족하지 않는다. 수식 HWP/HWPX 대조군은 각각
92.76185%, dyad 원본·최소 입력은 94.61177%/95.89197%다.
같은 출력에서 새 `shared-paint-baseline-debug-panels/review`를 만들었고,
원점 연결의 개선과 남은 복합 수식의 차이를 따로 판독했다.

### prime 표시와 cases의 원본 글꼴 leading

독립 Print는 미분 기호를 작은따옴표가 아니라 U+2032 prime으로 표시한다.
Native는 Symbol의 ASCII 작은따옴표를 그대로 그렸고 일반 기호 여백도
추가했다. 원본 AST·인용 문자열은 보존하면서 수식 Symbol의 표시만
prime으로 투영하고, postfix prime에 일반 연산자 여백을 추가하지 않는다.
표시 글리프와 같은 글꼴의 전진폭을 측정과 paint가 함께 사용한다.

12쪽 첫 cases의 독립 Print 글자 기준선은 456.320/470.080/483.840으로
간격이 13.760이다. 원본 수식 face HYhwpEQ의 hhea와 OS/2에서
ascender=820, descender=-204, lineGap=152, UPM=1024를 직접 읽었다.
BaseUnit의 기본 줄 상자와 이 leading의 합은 12 × (1 + 152/1024) =
13.78125로 Print와 일치한다. 기존 cases는 일반 행렬의 0.3 em 간격을
함께 사용해 단순 글자 줄의 기준선 간격을 15.600으로 키웠다.
HYhwpEQ cases는 원본 글꼴의 논리 leading을 사용하고, 분수·첨자가 있는
행의 실제 점유와 다른 face 및 일반 Matrix의 기존 계약은 유지한다.
글꼴 메트릭 사실값만 사용하며 사설 폰트 바이트는 추가하지 않는다.

원본과 MCP 재저장의 수식 1535개에서 너비가 바뀐 것은 56개였으며,
높이·기준선이 바뀐 항목과 함께 원본에 낡은 폭도 있음을 확인했다.
따라서 저장 Common.width에 모든 수식을 기계적으로 압축·확대하는
후보는 구현하지 않는다. 상세 값은 ignored `equation-source-width-probe.txt`에
보존한다. prime·cases 보정도 원본 전체와 정상 대조군을 새로 출력해
검증하며, 정식 회귀 기대값과 golden은 아직 변경하지 않는다.

### 무효인 논리 폰트 프레임의 너비 소비

현재 `flow_metrics_hwp`는 무효인 단순 글자·윗줄·위첨자 프레임의
높이·기준선을 재발행하면서 너비만 이전 common 캐시에서 가져왔다.
원본과 MCP 재저장에서 56개 항목의 너비도 달라졌으므로 한 축만
기존 캐시에 남겨 두는 것이 같은 결과의 소비인지 재검토했다.

이미 독립 근거로 무효를 확인한 논리 폰트 프레임의 정규화 결과는
실제 표시 LayoutBox의 너비·논리 높이·논리 기준선을 함께 반환하도록
바꾼다. 원본 문서의 AST와 Common 객체는 변경하지 않는다.
`flow_metrics_hwp → inline_box::horizontal_metrics → 글줄의 점유 폭과
content_origin_x → EquationNode와 각 backend`가 같은 너비를 소비한다.
크기 보호, LINE 형식, 다른 face와 유효한 프레임은 기존 값이 유지된다.
별도 계약인 기존 BOX의 외곽 너비는 이 후보의 적용 대상이 아니다.

공개 대체 글꼴의 실제 전진폭을 사용하는 후보이므로 독립 Print의
새 캐시 너비와 같다고 미리 단정하지 않는다. 숫자 보정값을 더하지
않으며 뒤 글자의 위치·줄 소속·겹침 및 전체 쪽수를 원본 출력과
대조한다. 새 source에서 정상 대조군과 전체 TSV를 확인하고,
90% 미만인 상태에서는 정식 회귀 검사·기대값을 변경하지 않는다.

### 기호·함수의 실제 전진폭과 cases 자식 원점

새 직접 SVG에서 12쪽의 `g(t)` 기준선은 470.243으로 Print의
470.080에 근접하지만 cases 가운데 행의 0은 473.843이었다.
첫째·셋째 행도 460.063/487.623으로 Print의 456.320/483.840보다
같이 내려가 있었다. 내부 Row의 `total_h / 2`는 물리 상자의 중간이지
가운데 행의 글자 기준선이 아니다. HYhwpEQ cases의 기준선은 실제
가운데 두 행의 기준선 평균으로 정하며, 홀수 행이면 같은 가운데 행을
두 번 참조한다. 자식 점유와 이미 확정한 행 간격은 유지한다.
또한 외곽 폭에 더한 왼쪽 중괄호 폭이 실제 inner 원점에서는 누락돼
있으므로 해당 예약 폭을 자식 원점에도 전달한다.

실제 전진폭 테이블의 소비 경로를 다시 대조하니 일반 Symbol·Function·
Limit 이름·큰 연산자는 `estimate_text_width`에서 문자 종류별 비율로
다시 추정했다. 예를 들어 등호는 0.5 em으로 측정했지만 표시 글꼴의
hmtx는 0.778 em이다. 연산자 여백을 별도로 예약해도 실제 등호 글리프가
상자를 넘어 뒤 내용에 닿을 수 있었다. 함수 이름의 같은 글자당 0.55 em
추정은 실제 글리프 간격보다 큰 빈 공간도 만들었다.

일반 표시 문자열을 지원하는 실제 hmtx 값이 있으면 이 helper에서도
그 전진폭을 반환한다. 측정과 각 painter의 기호·함수·큰 연산자 소비가
같은 결과를 사용하고, 미지원 글리프 및 별도의 합성 italic fallback은
기존 추정을 유지한다. 문서 ID·관측 px 값·글자 위치에 맞춘 조건은 넣지
않는다. 겹침·cases의 가운데 행 정렬과 전체 원본·대조군을 새 source에서
확인하며, 이미 통과한 쪽의 점수와 소속도 같이 대조한다.

`7e44c920d`의 너비 재발행만 적용한 비교에서 14쪽은 83.67219%로
내려갔다. 조사한 본문 기준선은 이전 source와 동일해, 새 점유 폭과
남아 있던 기호·함수의 부정확한 폭 추정을 구분해 확인한다. 이 결과를
개선 완료로 보고하지 않는다. 현재 추가 후보는 해당 추정 경로의
실제 hmtx 소비와 cases의 실제 자식 원점 연결이며 새 head에서 전체
재출력·직접 판독을 수행한다. 원본 및 모든 후보 증거를 보존한다.

`11e4e16dd`의 원본 전체 Native dev TSV는 18쪽을 유지했고, 11쪽
91.53192%, 12쪽 92.97692%, 16쪽 89.20368%, 17쪽 95.08380%로
개선됐다. 14쪽 86.01517%와 15쪽 88.82425%도 포함해 세 쪽이
여전히 미달이었다. 원본 수식 HWP/HWPX 대조군은 89.48492%로 내려갔고
새 직접 review에서 기호 간격이 커진 것을 확인했다. dyad 원본과
최소 입력은 94.61497%/95.89197%다. 이 후보는 승인 조건을 충족하지 않는다.

### 실제 연산자 전진폭과 HWP 기본 face의 중복 여백

설치된 HYhwpEQ의 원본 연산자 글리프에서 등호와 덧셈 전진폭은
795/1024 em, 빼기는 774/1024 em이다. 표시 글꼴 Latin Modern Math의
등호·덧셈·빼기 전진폭은 778/1000 em이다. 이전 기본 등호 측정의
0.5 em + 양쪽 0.15 em은 약 0.8 em으로 이 전진폭을 대신하던 값이다.
실제 hmtx를 소비한 뒤에도 같은 양쪽 여백을 더하면 등호 전진폭이
1.078 em으로 커져 원본 HWP face보다 큰 빈 공간을 예약한다.

HYhwpEQ의 기본 Symbol 연산자는 실제 전진폭을 직접 소비하고 일반
수학 스타일의 자동 여백을 추가하지 않는다. 원본 스크립트의 명시적
Space는 그대로 유지하며, 다른 face의 기존 연산자 여백 계약과 별도
Unicode 관계 기호 경로는 이 보정에서 변경하지 않는다. source face의
독립 메트릭과 Print의 실제 간격을 근거로 구분하고 문서 ID나 px 여유를
사용하지 않는다. 원본 수식 대조군과 18쪽 전체를 다시 출력해 검증한다.

`db5e89bd4`는 수식 대조군을 93.24791%로 회복했지만 원본 11·12·14·
15·16쪽을 각각 88.68898%/89.68636%/82.39767%/86.51644%/84.20740%로
내렸다. 원본 글꼴의 hmtx만으로 실제 수식 조판의 자동 여백을 부정한
가정은 맞지 않았다. 이 여백 제거 후보는 철회한다. 기여자·메인터너
모두 정상 대조군 하나의 통과를 전체 원본의 근거로 대신하지 않는다.

기존 9999/50을 명시한 분수 진단은 메트릭을 그대로 보존했다.
추가 진단에서는 Equation의 크기와 기준선을 0으로 두고 저장 LineSeg를
제거해 한컴이 새 메트릭을 생성하게 했다. MCP 2020 job
`eedd0919-d70e-4c5b-90b4-9f5b8f5a9d6d`가 양수 크기의 HWP를 생성했고,
6·8·9·10·12·18 pt의 단순 분수 높이는 1380/1890/2070/2250/2760/4140이었다.
단순 2.3배 추정은 8·10 pt에서 다르다. 14개 글자 크기의 56개 수식을
추가 생성한 job `48e69ca9-f73a-42f4-8bcf-8206c0967ef0`에서도 같은 차이를
확인했다. 일반 글자 줄의 높이는 source face의 격자에서 변했고,
prime이 있으면 본문 대체 face의 점유와 합집합이 되는 차이도 관측했다.

해당 원본은 ignored `equation-font-frame-matrix.hwpx`이고 생성 방식과
입력 SHA, job 및 산출 SHA는 같은 output 디렉터리에 보존한다. 이것은
원본 문서의 일치 증거가 아니라 폰트 프레임의 독립 진단이다.
글자 줄·윗줄·위첨자·분수의 같은 글자 크기별 결과를 추가 대조하고,
진단 저장본의 Print도 MCP로 출력해 정상 표시를 확인한다. 원본 18쪽의
기준 PDF는 그대로 사용하며 재생성 진단의 통과로 원본 결과를 대체하지 않는다.


### 글자 크기별 독립 폰트 메트릭의 소비 후보

0 크기·0 기준선·저장 줄 없음으로 작성한 14개 글자 크기별 일반 글자 줄,
prime 줄, 윗줄, prime 윗줄, 위첨자, 단순 분수, prime 분수의 98개 수식을
MCP 2020 job `909d6081-12ef-42be-8982-9d653aa1c464`로 재생성했다.
양수 크기의 생성 HWP를 다시 MCP Print job
`028fc9e0-c6cb-49e1-8197-3eaf15d8dc79`로 인쇄했다. PrintMethod 0,
한컴 11.0.0.9136의 독립 3쪽 PDF와 입력·산출 해시는 ignored provenance에
기록한다. 이 진단은 원본 18쪽의 대체 기준이나 시각 통과 증거가 아니다.

일반 줄 높이를 항상 명목 글자 크기로 간주하던 가정을 제거하고,
독립 생성본의 논리 프레임을 글자 크기별 폰트 메트릭으로 소비하는 후보를
확인한다. 문서 ID나 실제 문단 좌표를 사용하지 않는다. 지원하지 않는
글자 크기, 중첩 분수, 근호, 배열, 복합 첨자는 추정으로 확장하지 않는다.
혼합 글꼴·스타일 및 임의 위첨자 내용의 범용 일치는 아직 미검증이다.
측정과 실제 paint가 같은 논리 프레임을 사용하도록 기존 연결을 유지한다.

단순 분수의 독립 너비와 비교해 기본 수학 face의 양쪽 여백을 각각
0.25 em으로 소비하는 후보도 함께 재출력한다. 글자 크기별 힌팅 차이가
있으므로 모든 저장 너비가 정확히 같다고 주장하지 않는다. 원본 18쪽과
수식 HWP/HWPX·dyad 대조군에서 새 출력과 직접 판독을 확인하며,
90% 미만인 동안 회귀 검사·golden은 변경하지 않는다.


### 높이 무효와 너비 무효의 분리

`7e44c920d`에서 논리 높이의 무효를 근거로 저장 너비까지 공개 대체 글꼴의
자연 폭으로 바꾼 판단을 재검토했다. 원본과 한컴 재저장의 너비 차이는
1535개 중 56개였으며 높이 변경과 너비 변경이 항상 함께 발생하지 않는다.
대체 글꼴 hmtx의 자연 폭은 한컴의 글줄 점유 폭과도 같은 계약이 아니다.

정규화된 논리 높이·기준선은 공통 결과를 유지하되 저장 전진폭을 보존한다.
실제 paint의 자연 폭과 저장 표시 폭 사이의 기존 스케일 계약을 사용한다.
이 변경은 문서별 값을 맞추는 너비 보정이 아니라 근거가 부족한 너비 무효
판정을 철회한 것이다. 원본 및 수식 대조군을 같은 기준으로 재검증한다.


### 첨자 글자 크기의 공통 소비 후보

독립 98개 수식의 Print에서 위첨자 글자 크기를 실제 PDF 글리프 단위로
대조했다. 6·9·12·15·18·24 pt에서 본체 대비 0.68배였고, 나머지 크기는
한컴의 글자 크기 양자화 차이가 있었다. 원본 수식 대조군의 9 pt 본체와
첨자도 같은 0.68배다. PDF span의 첫 원점은 여러 글리프의 서로 다른
기준선을 합칠 수 있으므로 개별 char origin으로 비교한다.

`EqLayout::script_font_size → layout_node의 자식 측정 → LayoutBox.font_size
→ SVG/Canvas2D/Skia 및 paint JSON → CanvasKit`으로 실제 측정 글자 크기를
전달한다. HYhwpEQ는 0.68, 다른 face는 기존 0.7의 계약을 유지한다.
각 backend에서 자식 첨자의 비율을 다시 계산한 값으로 실제 글자를
그리지 않게 한다. 극한 하한도 같은 생산 helper를 소비한다.

기존 sample LayoutBox의 새 필드 초기화는 컴파일 보완이며 assertion이나
회귀 기대값을 바꾸지 않는다. 첨자 크기·기호 간격·뒤 본문·줄 소속과
전체 쪽수를 원본 Print에서 다시 대조한다. 진단 Print의 이전 Native 결과는
92.53070%/90.87723%/87.54969%로 마지막 쪽이 미달이었다. 이 진단의
원인을 포함해 보정 후보를 확인하며 원본이나 진단의 통과를 서로 대신하지 않는다.


### 글줄 끝과 인접 연산자의 자동 여백 후보

독립 Print에서 `=-9`의 등호·빼기·숫자 각각의 char origin을 대조했다.
등호와 빼기 사이에는 일반 피연산자 양쪽에 있던 여백이 없고,
빼기 뒤 숫자에는 기존 여백이 남는다. 따라서 모든 연산자의 여백을
제거했던 후보와 구분해 실제 경계에서만 자동 여백을 계산한다.
한컴 기본 face에서 글줄 처음/끝의 바깥 여백과 인접 연산자 사이의
중복 자동 여백을 예약하지 않는다. 피연산자 경계의 기존 여백과
스크립트에 명시한 Space 및 다른 face의 기존 계약은 유지한다.

`layout_row → row_children → glyph와 경계별 여백을 포함한 공통 Row
→ 측정 폭과 각 painter`로 전달한다. SVG의 가운데 정렬 Symbol만으로
비대칭 여백을 흉내 내지 않고 실제 glyph 원점을 Row 자식에 보존한다.
문서별 좌표나 픽셀 여유를 사용하지 않으며, 원본 전체·수식 대조군과
일반 피연산자가 있는 반례를 함께 대조한다. MathML Core의 prefix/infix
구분은 구조상 참고 근거이고 실제 여백 관측은 독립 한컴 Print에 따른다.


### 원본 face의 기본 이탤릭 전진폭 후보

한컴 원본 Print에 포함된 HYhwpEQ subset의 130개 hmtx 값을 읽어
설치된 공식 HYhwpEQ와 대조했고 모두 일치했다. 추출 폰트 바이트는
ignored 진단 경로에만 보존한다. 원본과 설치본이 다른 폰트라는 추정은
이 범위에서 해당하지 않는다.

특히 기본 이탤릭 f의 원본 전진폭은 553/1024 em, 공개 표시 글꼴은
490/1000 em이다. 다른 기본 소문자 대부분은 정규화한 전진폭이 거의
같다. 원본의 점유와 공개 대체 글리프의 자연 전진폭을 동일시하지
않고, 기본 소문자 이탤릭 a..z의 원본 face 메트릭으로 각각의 원점을
구성한다. 공개 글리프의 모양은 유지하고 같은 Row 결과를 각 backend가
소비한다. Roman·Bold·다른 face와 미지원 문자열은 기존 경로를 유지한다.

별도의 폰트 바이트나 윤곽을 저장소에 추가하지 않으며 픽셀 위치나
문서 ID로 간격을 맞추지 않는다. 원본 18쪽과 수식 HWP/HWPX·dyad
대조군을 재출력해 변화와 남은 모양 차이를 직접 확인한다.
`8ded6d2f2`의 원본 15·16쪽은 89.52913%/89.49810%로 아직 미달이며
그 결과를 승인 또는 회귀 변경 근거로 사용하지 않는다.


수식 결과 타입과 분수·첨자·행렬·장식의 기존 계산은 작은 모듈로
옮겼고, `layout.rs`는 965줄로 줄였다. 공개 `layout::LayoutBox` 및
`LayoutKind` 경로는 재수출로 유지한다. 이 이동에서는 계산식·검사
기대값을 변경하지 않았으며 새 후보는 분리 후 Native 빌드에서 확인한다.


### 큰 연산자의 표시 메트릭과 실제 소비 경로

원본 Print와 독립 6·9·12·18 pt 진단을 대조했다. 새 MCP HWP job은
`9ada78b5-d8b1-4c5d-be4f-395e1e3336c8`, Print job은
`df146899-86cc-4a99-852b-bf47ab2ba8a5`다. 합 기호의 원본 글꼴 크기는
명목 크기의 1.8배, 적분은 2배였다. 합 기호의 원본 ink 높이는
700/1024 em이고 공개 표시 글꼴의 같은 기호는 1000/1000 em이다.
같은 font-size로 그리면 실제 높이가 같지 않다. 원본 ink 높이에 맞춘
공개 글리프 크기와 공통 수학 축 정렬을 배치 결과에 전달한다.

기존 큰 연산자는 측정 기준선 0.6과 paint 기준선 0.8을 별도로 계산했고,
Skia는 x 원점도 다른 폭 추정으로 계산했다. 실제 표시의 글자 크기·x·
기준선을 생산 단계에서 정하고 SVG/Canvas2D/Skia/JSON/CanvasKit이
그 결과를 소비한다. 다른 face의 기존 표시 계약은 유지한다.

적분의 기존 2.5배 논리 슬롯과 원본 ink 영역을 구분한다. 원본 글꼴의
상승 820·하강 202/1024와 2배 글리프, 수학 축을 사용해 path의 상하
끝을 계산한다. 사설 글꼴 윤곽은 복사하지 않는다. 상·하한의 원점과
점유를 공통 결과에서 만들고 일반 첨자 경로와 BigOp 경로가 같이
소비하게 한다. CanvasKit에서 적분을 작은 일반 텍스트로 다시 그리는
경로도 같은 path 기하를 소비하도록 고친다. 실제 경계와 원본 전체를
재출력하고 90% 선행 조건을 통과하기 전에는 회귀 기대값을 변경하지 않는다.

#7548의 추가 전체 대조에서 1쪽은 60.74834%였다. 보정 전 `a4e94f55c`
에서도 같은 60.74834%임을 별도 실행으로 확인했다. 14쪽은 98.35994%다.
전체 문서의 시각 통과로 보고하지 않으며 기존 차이는 별도로 기록한다.


`c3999f44f`의 전체 원본은 15·16쪽 89.82599%/89.81567%로 여전히
미달이다. 독립 연산자 진단은 92.92923%지만 점수만으로 완료하지 않는다.
추가 char-origin 판독에서 네 크기의 적분 하한 x는 약 0.75 em,
상한 x는 약 1.3 em, 하한 기준선은 본체보다 약 0.9 em 아래, 상한은
약 1 em 위였다. 기존 갈고리 끝에서 단순 여유를 더하던 첨자 원점은
이 계약과 다르다. 원본 face에서 이 상대 메트릭과 실제 자식 기준선을
소비하고, 복합 하한은 공개 MATH SubscriptTopMax의 경계도 적용한다.

기존 0.45 em trailing 여백은 잘못된 상한 x를 일부 보완하던 값이었다.
독립 평범한 하한/상한 출력에서 뒤 본문은 상한의 점유 끝에 이어지므로
원본 face는 자식의 실제 점유 끝을 소비하고 명시적 Space만 보존한다.
적분의 논리 전진폭과 갈고리 x 범위도 원본 font hmtx/bounds로 정의한다.
사설 윤곽은 복사하지 않으며, 변경된 기하·점유가 원본 쪽수·뒤 문단·
상하한과 본문을 보존하는지 다시 확인한다.


### 90% 통과 뒤 글꼴 전환과 공백의 확대 판독

`fe7595c01`의 원본 18쪽 Native dev 최저값은 15쪽 90.65996%이고
16쪽은 91.17900%다. 수식 HWP/HWPX는 92.51376%, dyad 원본/최소
입력은 95.16512%/95.89197%, 독립 4개 크기의 연산자는 96.40216%다.
이는 중간 Native 결과이며 fresh WASM과 최종 검증을 대신하지 않는다.

15쪽 확대 판독에서 `rm` 뒤 백틱 공백이 문자로 표시되고 그 뒤 벡터
문자가 이탤릭으로 남는 결함을 확인했다. 원본 스크립트의
``rm `angle`A _{it `n rm} B _{it `n rm} D _{it `n rm}``와
같은 영역의 독립 Print는 백틱을 표시하지 않고 A/B/D를 정체로, n을
이탤릭으로 출력한다. 단일 인자 파서가 Whitespace를 Text로 소비한
뒤 글꼴 지시를 끝내는 경로가 원인이다. 점수 통과로 면제하지 않는다.

한컴 `rm`/`it`은 현재 그룹 안에서 다음 글꼴 전환까지 적용하고,
명시적인 Space는 그대로 보존한다. 그룹 내부 전환은 바깥 형제에
전파하지 않는다. 명시적 인자를 받는 LaTeX 명령의 범위는 유지한다.
`parser → Row simplify의 글꼴 범위 → FontStyle 공통 배치 → 각 paint`로
연결하며 문서 ID나 화면 좌표를 조건으로 사용하지 않는다. 원본 전체와
대조군을 재출력해 정체/이탤릭, 첨자, 분수 앞뒤 간격을 직접 확인한다.

전체 Clippy에서 누락된 LayoutBox.font_size와 ResolvedStyleSet.resolved_dpi
테스트 초기화도 보정한다. 기존 검사 조건과 기대값은 변경하지 않는다.


하한만·상한만·복합 하한의 추가 진단 Print job은
`d89f7b95-03b6-4b9b-8352-ee86dd683d5c`다. engine 2020,
11.0.0.9136, PrintMethod 0으로 출력했다. 현재 Native 전체는
83.00015%이고 라벨 폭·줄 위치와 적분 윤곽 차이가 함께 보인다.
이를 통과 증거나 정식 회귀 fixture로 사용하지 않는다.

기존 적분의 단일 S곡선에는 실제 표시의 갈고리와 획 굵기 변화가
없었다. 설치된 사설 글꼴 윤곽을 복사하지 않고 저장소의 공개
Latin Modern Math 1.959 integral 윤곽을 사용한다. 기존 공통 ink 영역에
좌표 변환하며 SVG·Canvas2D·Skia·JSON/CanvasKit이 같은 명령을 소비한다.
원본 저작권과 GUST Font License를 파일 및 THIRD_PARTY_LICENSES.md에
보존한다. 배치 폭·첨자 원점·논리 프레임과 표시 윤곽의 계약은 구분한다.
원본 및 진단 입력을 다시 비교해 위치와 모양을 검증한다.


`8edd3f8c2`의 원본 18쪽 Native dev 전체는 90% 이상이며 최저는
6쪽 91.18877%다. 15·16쪽은 93.45365%/91.42286%, 수식 HWP/HWPX는
92.51376%, dyad 원본/최소 입력은 95.16512%/95.89197%다. dyad 원본의
첫 캡처는 Chrome 시작 timeout으로 실패했고 별도 새 key로 재실행했다.
환경 실패를 시각 회귀로 분류하지 않는다.

추가 진단의 한컴산뜻돋움 라벨은 내장 폭 DB에 없는 face다. 실제 SVG에서
11 pt 라틴 문자가 모두 7.3333 px(0.5 em)의 textLength로 배치된 것을
확인했다. 공식 글꼴 설치와 내장 메트릭 지원을 같은 상태로 보고하지 않는다.
해당 진단은 81.73091%이며 실패 입력과 원 Print를 보존한다.

수식 기하를 별도로 판독하기 위해 지원되는 함초롬돋움 라벨 대조군을
생성했다. 변경은 Contents/header.xml의 라벨 face 이름뿐이며 수식
스크립트·크기·문단 XML은 동일하다. HWP job
`78556bae-3dfa-4308-be7d-2f461a1a7619`, Print job
`973b5172-693c-4fa5-b8bf-0603ebadb439`는 engine 2020으로 실행했다.
기준 PDF SHA-256은
`9a03f67883dfeef93e837e644139a97dfe9c9a4eb01968f3bd371a077521119f`다.
이 6개 적분 대조군 Native의 2px 실루엣은 100.00000%였다. review PNG에서
적분 윤곽·상하한·복합 하한과 뒤 본문 위치를 직접 확인했다. 이는 별도
대조군 결과이며 한컴산뜻돋움 진단이나 원본 18쪽의 통과로 바꾸어 보고하지 않는다.

같은 source에서 fmt check·Native/WASM/workspace all-target Clippy,
workspace build, base `48ff4bb93` 대비 integration manifest와 unit tier
정책 검사가 모두 exit 0으로 끝났다. TypeScript 검사도 exit 0이다.
최종 fresh WASM과 원본 대조군 전체 검사, Skia·Studio·회귀 검사는 남아 있다.
MCP Print로 만든 추가 진단 PDF도 커밋에 포함하며 TSV·실행 로그·중간
JSON·nextrun은 ignored output에만 보존한다.


### 저장 줄 높이와 다음 줄의 실제 개체 소유 대조

원본 18쪽은 source `1be719b47`의 fresh WASM에서도 최저 91.18877%로
Native와 같은 값이었다. 패키지/public JS와 WASM의 바이트 일치도 확인했다.
98개 역할의 함초롬돋움 라벨 대조군 Native 3쪽은
99.78707%/98.77101%/96.48090%다. 기존 진단과 원본을 구분해 보존한다.

#1139의 추가 원본은 아직 미달이다. 9월 2022 HWP 23쪽 중
12·13·14·19·21·22쪽이 90% 미만이고 최저는 13쪽 54.69165%다.
보정 전 `8c6ccc086`의 같은 Print 대조 12·13쪽은
85.39554%/74.35107%였으며 현재는 83.93119%/54.69165%다.
새 보정에서 증가한 차이도 회귀 후보로 보존하고 조사한다.

실제 원본의 note owner 122, 내부 문단 18의 분수 줄 4·5는
line_height 2298, text_height 2070이며 다음 줄 text_height도 2070이다.
기존 stored_content_cadence는 다음 vpos가 text_height+line_spacing이라는
이유로 이 line_height까지 줄였다. 이것만으로 다음 큰 개체의 중복
예약이라고 판단하는 가정은 부족하다. 반면 원본 10월 owner 70의
조건 문단에는 다음 줄이 실제로 소유한 더 큰 수식 상자가 있다.

다음 줄의 소유 범위와 실제 원 저장 인라인 Equation/Picture 높이를
대조해, 그 개체가 현재의 큰 line_height와 다음 text_height를 실제로
소유하는 경우만 중복 예약 후보로 삼는다. 작은 분수 줄의 유효한
line_height는 유지한다. 문서 ID·고정 px·우연한 특정 수치를 조건으로
사용하지 않는다. 이 판단은 같은 ComposedLine 결과를 갱신해 측정과
최종 문단 배치에 전달한다. 다음 단계는 9월의 실패 경계와 원본 10월
18쪽을 다시 출력해 이 가정과 반례를 실제 Print에 대조하는 것이다.
회귀 검사·기대값은 아직 바꾸지 않는다.


`3e737465d`는 원본 10월 18쪽 Native의 90% 이상을 유지했지만
9월 12·13쪽은 83.93119%/54.69165%로 바뀌지 않았다. 이 가드 보강을
9월 결함의 해결 증거로 쓰지 않는다.

실제 후속 소비 지점에서 별도의 낮추기가 있었다. paragraph_layout.rs의
stored_line_flow_height 호출은 인라인 Table/Shape만 줄 상자의 점유
하한에 포함하고, 별도 tac_controls에 들어 있는 Equation은 제외했다.
height_measurer.rs의 fallback도 같은 누락이 있었다. 따라서 구성 결과의
유효한 line_height를 보존해도 실제 배치 커서가 저장 vpos 차를 선택해
수식 줄을 짧게 전진할 수 있었다. helper 반환 이후의 소비를 확인한 결과다.

측정과 배치가 기존 typeset의 실제 TAC 수식 줄 소유 판정을 공유하고,
수식이 소유한 줄의 점유 하한에도 구성된 line_height를 적용한다.
`treat_as_char`만으로 모든 문단 수식을 같은 줄로 간주하지 않는다.
`composed 줄 → line_tac 소유 판정 → source 전진 선택의 하한 → 최종 y`
경로와 정상 text/다른 줄의 수식 경계를 다시 대조한다. 새 source의 실제
Print 비교 전에는 회귀 검사·기대값을 변경하지 않는다.

### 9월 문서 미주 경계의 간격 생략 재검토

`bf4246fb1`의 10월 원본 18쪽 Native 최저값은 91.18877%였지만,
9월 12·13쪽은 83.93119%/54.69165%로 개선되지 않았다. 10월 2·3쪽의
위치도 달라졌다. 수식 소유만으로 저장 line_height 전체를 물리 점유
하한으로 정한 가정은 독립 출력으로 입증되지 않아 해당 후보를 철회한다.
수식이 실제로 점유한 합집합과 저장 큰 프레임은 구분해야 한다.

9월 12쪽 문15 앞에서는 미주 사이 1984 HWPUNIT, 직전 간격 452,
부족분 1532가 기록된다. ENDNOTE_BOUNDARY의 current_height는 787.88,
available은 1001.56이며 직전 미주에는 내부 vpos 되감김이 있다.
prepare.rs는 이 단의 높이가 65%를 넘었다는 이유만으로 부족한 미주
간격의 render tail 반영을 생략했다. PDF와 비교하면 문15 앞 간격이
부족하고 뒤 문항의 단 분할도 달라진다. 단 하단 비율을 이미 소비한
간격의 대용으로 삼는 분기를 제거한다. 나머지 경계 판단의 정당성도
최종 출력에서 확인하며, 이 후보만으로 전체 미주 계약이 충족됐다고
보고하지 않는다. 원본 10월 18쪽과 9월의 실패 경계를 새로 출력한다.

`d1fb972da`의 9월 12·13쪽도 83.93119%/54.69165%로 바뀌지 않았다.
미주 tail의 line_spacing을 1984로 전달해도 최종 HeightCursor는 문15를
직전 줄 상자의 하단에 놓았다. 렌더 트리에서 문15 문단 665의 y는
872.6이며 직전 문단 664 마지막 줄은 y=845.0, 높이 27.6이다.
독립 Print의 문15 글자 기준선은 909.27995이고 차이는 미주 사이
7mm에 해당한다. 임의의 px를 더하지 않고, gap의 전달·이미 소비한
간격·실제 줄 하단을 구분해 최종 커서 분기의 입력을 확인한다.
기존 RHWP_VPOS_DEBUG에 미주 gap·실측 하단·후보 원점도 함께 기록해
값을 잃는 호출 지점을 식별한다. 진단 로그는 output에만 보존한다.

진단 source `b69e98081`에서 문15 앞 gap_hu는 정상적으로 1984가
전달됐다. 그러나 직전 콘텐츠 하단은 844.96, 실제 흐름 줄 하단은
872.56이었다. gap_cap도 872.56으로 잘못 계산됐다. 숫자 전달 누락이
아니라 점유의 누락이다. 문단 664 마지막 tab run 끝 위치 33의 수식은
emit_line_runs가 실제 방출하지만, is_blank_spacer_line은 반열림 TAC
집합을 보고 빈 줄로 판정해 콘텐츠 하단을 줄 상단으로 기록했다.

정렬 폭 계산이 이미 사용하는 실제 방출 TAC 집합은 마지막 run 끝의
수식도 포함하고 다음 줄 선두의 수식은 제외한다. 빈 줄 판정과 수식
전용 tail 판정도 같은 집합을 소비하게 한다. 이로써 물리 줄 하단은
그린 수식까지 포함하며 HeightCursor가 설정된 미주 간격을 그 뒤에
한 번 예약한다. 문서 ID나 절대 위치로 분기하지 않는다. 다음 줄
선두·문단 끝 수식·진짜 빈 줄과 원본 10월의 경계를 재출력해 확인한다.

98개 수식 역할의 함초롬돋움 라벨 대조군 기준 PDF도 커밋에 포함한다.
파일은 pdf/pr7600-equation-font-frame-diagnostic/
equation-font-frame-roles-hcr-print-2020.pdf이며 SHA-256은
`5bd19d79405f45bb28d01e91891467a26c14ca0435a6c74387948ba83827d9d9`다.
MCP Print job `9371d3c0-5402-4011-809a-9fdc45a3c983`, engine 2020,
Hancom 11.0.0.9136, PrintMethod 0의 3쪽 출력이다. source `1be719b47`의
fresh WASM과 `8edd3f8c2` Native는 각각 99.78707%/98.77101%/96.48090%
였다. 별도 3쪽 review PNG에서 분수·첨자·윗줄·prime을 직접 판독했다.
이전 runtime의 결과를 새 source의 검증으로 바꾸지 않는다. 현재
점유 판정 보정 source에서도 Native 전체를 다시 출력하고 있다.
진단용 HWP/HWPX와 로그는 아직 output에 있어 최종 제출 증적의
재현 입력 커밋 확인은 미완료다. 이를 정식 회귀 통과로 보고하지 않는다.


### 수식 표의 파생 프레임 재발행 — 구현 후보

- 원본 9월 입력의 미주 19 표는 셀 높이 1182 HWPUNIT, 상하 안 여백 각 141입니다.
  둘째 행의 prime 수식 저장 줄은 1125이며, 원 행 요구 높이 합계는
  `1182 + (1125 + 282) + 1182 = 3771`로 표의 저장 높이와 같습니다.
  한컴 Print PDF의 표 괘선 높이는 약 47.308px이며, 새 prime 논리 줄 높이 900에서
  계산한 `1182 * 3 = 3546 HWPUNIT`와 대응합니다. 이 수치는 진단 관측값이며
  회귀 테스트의 절대 픽셀 기대값으로 사용하지 않습니다.
- `math_table::normalize`는 별도 높이 보호가 없고, 단일 저장 줄 셀의 내용에서
  표 높이가 파생됐음을 확인한 경우에만 셀 줄, 표 프레임, 호스트 줄 높이와
  같은 저장 단 조각의 후속 원점을 함께 갱신합니다. 원본 문서 모델은 보존합니다.
  추가 프레임 공간, 병합 행, 중첩 개체, 재줄바꿈 셀은 이 투영 계약에서 제외합니다.
- 소비 경로는 `compute_render_normalized → para_src → HeightMeasurer/Typeset →
  실제 표 layout`입니다. 미주 콘텐츠도 같은 파생 섹션을 해석하게 합니다.
  표 높이만 줄여 마지막 행의 성장만 막는 보정으로 끝내지 않습니다.
- 직전 실행 코드 `7d916dd8201e5ec739fd11a459c1a6d4ac3cc3f1`의 Native 원본 10월
  18쪽은 최저 91.18877%이며, 9월 23쪽은 최저 69.99895%입니다.
  9월 12쪽 89.82289%는 미달입니다. 이를 90%로 반올림하거나 회귀 추가의 근거로 쓰지 않습니다.
  전체 Rust lint 묶음은 통과했으나 새 표 보정 후보의 검증 결과가 아닙니다.
- 직전 fresh WASM은 문서/PDF 후속 head `d6cec40f5edf391db5e01e5fdfd569ba37dfb2ab`에서
  루트 wrapper로 산출했습니다. WASM SHA-256은
  `bdf7fa98dece9151d72b371e76bcea604e64b858d7985a7813ccc3afe7812c65`이며,
  pkg와 Studio public의 JS/WASM 해시는 각각 일치합니다. 수식 대조군 4건의 Native/WASM
  최저값은 eq002 HWP/HWPX 92.51376%, dyad 원본 95.16512%, 최소 입력 95.89197%입니다.
- 현재 항목은 구현 후보입니다. 새 후보의 원본 전체 페이지와 표/뒤 문단의 직접 판독,
  비적용 경계 검증, fresh WASM 재산출이 끝나기 전에는 완료·회귀 변경·승인으로 판정하지 않습니다.
  TSV와 실행 로그, 다음 실행 메모는 ignored output에만 보존합니다.


### 단 중간의 미주 경계 간격 누락 — 원본 대조와 보정 후보

- 같은 9월 원본의 19쪽 문29 제목은 Native 줄 원점 393.8px에 놓여 이전 수식 줄의
  점유 끝 393.8px에 붙었습니다. 독립 Print PDF 제목 기준선은 430.56px입니다.
  Native SVG 제목 기준선은 403.4px입니다. 기준선 차이 27.16px의 대부분은 설정된
  미주 사이 1984 HWPUNIT의 26.4533px에 대응하며, 나머지 기준선 차이는 별도로
  대조합니다. 이 수치는 원인 확인용 관측입니다.
- `VPOS_NOTE_GAP pi=995`에서 앞 줄의 실제 점유 끝은 보존됐지만 `carries_gap=false`
  였습니다. `prepare.rs`가 내부 되감김과 단 높이 25~50%의 조합만으로 render tail
  간격을 생략했습니다. 단의 상대 위치는 실제 줄이 간격을 소비했다는 증거가 아닙니다.
- 해당 `skip_default_mid_column_between_notes_trailing` 분기를 제거합니다. 기존 설정값을
  render tail에 전달하고, 실제 배치는 공통 점유·간격 계약으로 소비하게 합니다.
  다른 간격 생략/수식 후 제목 보정 분기는 별도 근거 없이 함께 변경하지 않습니다.
- 정상 10월 원본 전체와 9월 영향 페이지를 재출력해 제목, 뒤 본문, 페이지 수를 대조합니다.
  이 후보의 검증 전에는 기존 테스트를 변경하거나 개선 완료로 보고하지 않습니다.


### 원본별 시각 선행 조건 충족 후 회귀 검사 전환

- 검증한 구현 head는 `909855f3c295044149b826816c231b233618adec`입니다. Native와
  루트 wrapper에서 새로 만든 WASM을 독립 한컴 Print PDF와 전체 페이지 비교했습니다.
  10월 원본 18쪽 최저 91.18877%, eq002 HWP/HWPX 각 1쪽 92.51376%, dyad 원본
  95.16512%, dyad 최소 입력 95.89197%, HCR 수식 역할 3쪽 96.48090%, 복합 적분
  HCR 1쪽 100.00000%입니다. 이 결과의 입력·PDF 해시와 명령은 ignored
  `output/pr-review/pr7600/math-table-mid-gap-wasm-controls/`의 case manifest와 TSV에 있습니다.
- WASM SHA-256은 `4fb1a17ba91075ee185ddbc06052d4da0e3993df5d9f80a73227428e28632341`
  입니다. pkg/public JS·WASM 해시가 각각 일치하며, 브라우저에서 실제 응답 바이트도
  확인했습니다. Canvas2D/CanvasKit × dyad 원본/최소 입력 4건은 요청한 backend로
  동작했고 fallback, 실행 오류, 수식 누락이 없었습니다. 이는 CanvasKit의 픽셀 일치율
  측정을 대신하는 결과가 아닙니다.
- 10월 원본 6·15쪽의 새 Native/WASM compare·overlay·review를 직접 열어 원본 수식,
  적분 꼬리, (ii) 본문 순서와 그림을 대조했습니다. 글꼴 윤곽 차이를 완전한 동일성으로
  보고하지 않습니다. 현재 9월 원본은 여전히 미달이며 승인 판정이 아닙니다.
- #1139에서 이 10월 입력을 사용하는 9건을 `tests/cases/issue_1139_oct_source_relations.rs`로
  이동합니다. 기존 사례 수를 유지하며, 고정 px 영역으로 수식을 선택하지 않고 문단·수식의
  원본 미주 소유, 실제 Column 소속, 앞뒤 순서와 단 내부 점유 관계를 검사합니다.
  기존 문28 검사는 740~790px 구간에서 (ii) 줄 자체의 수식을 선택할 수 있었습니다.
  새 검사는 앞 수식의 원본 소유와 다음 미주 내부 문단을 연결합니다.
- #1308의 8건도 원본 컨트롤 소유, 강제 개행 전후의 축, 수식/쉼표/탭의 순서와 커서 관계로
  변경합니다. 기존 4건은 고정 y 구간과 SVG의 일반 `f` 문자열을 전제해 실제 수식 그룹을
  찾지 못했습니다. 수학 이탤릭 글자가 Unicode 수학 문자로 출력된 경우도 해당합니다.
  SVG 문자열 형식 대신 실제 소유와 의미를 검사합니다.
- 적용 전 관계 검사 진단은 #1139 9 PASS, #1308 8 PASS입니다. 이것은 정식 Cargo 검증의
  완료가 아닙니다. source를 커밋한 뒤 파생 suite를 준비하고 lint·정식 회귀를 다시 실행합니다.
- 전체 #1139 파일의 나머지 px 검사도 처리 대상입니다. 90% 미만 입력의 기대값을 바꾸거나
  기존 검사를 자동 삭제하지 않습니다. 9월 19쪽은 Native 92.43394%로 개선됐지만
  12/13/14/21/22쪽은 아직 미달이며, 표 보정 후보의 13쪽은 76.83987%입니다.
- 이미 시각 측정한 HCR 진단 HWP/HWPX를 `tests/fixtures/pr7600-equation-font-frame/`에
  동일 바이트로 보존합니다. 원래 한산뜻돋움 진단 입력의 미달 결과를 이 대조군의 통과로
  바꾸어 보고하지 않습니다. 대응 Print PDF는 기존 커밋한 `pdf/` 파일을 유지합니다.


### 저장 표 줄의 여백 중복 예약 — 소비 경로 확인과 보정

- `65e96c9a687b33a4a22d57313f8d971dbb05bdb0`의 정식 release-test 검증에서
  원본 10월 관계 검사 9건과 #1308 강제 개행 관계 검사 8건이 각각 모두 통과했습니다.
  fmt, Native/WASM/전체 target Clippy, workspace build, 고정 base 대비 manifest와
  unit tier 정책 검사도 통과했습니다. 전체 nextest 완료를 뜻하지 않습니다.
- 9월 원본은 HWP5 형식이지만 파서의 HWP3 계보 호환 프로필이 적용됩니다.
  단일 표 호스트의 저장 높이는 표 본체와 상하 바깥여백의 합계입니다. 측정은
  이 줄의 높이와 후행 간격을 한 번 예약하지만, 실제 배치의 HWP3 저장 사다리
  재설정은 테두리 원점을 pen으로 사용하고 아래 여백도 후가산해 중복했습니다.
- 추적 경로는 `math_table::normalize_host → 미주 row_projection/format의
  저장 lh+ls → layout_table_control_block의 표 본체 배치 → HWP3 target
  덮어쓰기 → layout_table_item의 후행 간격·바깥 아래 여백`입니다. 기존
  공통 높이 계산 뒤의 별도 원점 덮어쓰기가 결과 공유를 깨뜨린 경우입니다.
- 독립 Print의 13쪽 표 뒤 본문 기준선은 569.44px입니다. 변경 전 실제
  기준선은 576.92px이며 약 7.48px 차이는 상하 여백 568 HWPUNIT의
  7.5733px와 대응합니다. 이 관측을 고정 px 회귀 기대값으로 사용하지 않습니다.
- 기존 `stored_first_tac_line`이 증명하는 단일 표·저장 줄 소유 계약을 소비해
  사다리 재설정의 원점을 여백 전 pen으로 환산합니다. 이 경우 후가산 예정인
  아래 여백도 한 번만 소비하고, 기존 2px 보정 임계값에 의존하지 않습니다.
  텍스트 혼합·여러 표·합성/편집 줄·다른 소유 줄에는 이 단일 표 계약을
  확장하지 않습니다. 새 Native/fresh WASM 시각 검증 전에는 완료로 판정하지 않습니다.


### 방출 완료한 호스트 줄 뒤의 흐름 밖 그림 — 좌표 기준 보존 후보

- `e7c4da5b4` Native 9월 전체 23쪽의 22쪽은 79.61781%에서 93.49551%로
  개선됐습니다. 13쪽은 76.83987%에서 83.40392%입니다. 12/14/21쪽은
  각각 89.82289%/78.02485%/81.20332%로 아직 미달입니다. 새 head의
  fresh WASM을 실행하지 않았으므로 Native 단독 결과로 기록합니다.
- 표 뒤 첫 본문의 줄 상단은 566.6px에서 559.0px로 이동했습니다. 독립
  Print 기준선 569.44px와 실제 SVG 기준선을 같은 좌표계에서 대조합니다.
  표 괘선만 맞춘 결과로 후속 본문 완료를 판정하지 않습니다.
- 남은 13쪽 문20의 전진 차이는 원본 미주 19 내부 문단 10의
  `FullParagraph → Shape → 다음 문단` 경계에서 처음 생깁니다. FullParagraph가
  수식 줄과 후행 간격을 이미 소비했으며 Square 그림 항목의 실제 전진은 0입니다.
  그런데 표/Shape 후처리가 기준을 초기화해 다음 lazy base가 같은 줄간격을
  다시 다리로 더했습니다. 흐름 밖 그림의 가시성이 줄의 점유를 대신하지 않습니다.
- 소비 경로는 `저장/파생 줄 메트릭 → FullParagraph 배치 완료 →
  layout_shape_item의 그림 출력과 전진 결과 → build_single_column의 base 무효화 →
  HeightCursor의 lazy base 복구 → 다음 저장 줄 배치`입니다. 새 helper는
  같은 호스트의 마지막 저장 줄이 실제 방출됐고 그림 항목이 흐름을 전진시키지
  않았을 때 그 줄의 기준을 보존합니다. 자리차지 그림, TAC 개체, 미완료 조각,
  편집/합성 줄에 이 계약을 확장하지 않습니다. 원점 동일성은 부동소수점
  반올림만 허용하며 고정 px 관용치를 추가하지 않습니다.
- 단독 그림, 실제 전진하는 개체, 본문과 수식만 있는 호스트, 다줄 호스트의
  경계와 원본 전체 출력을 재검증합니다. 90%를 충족하기 전에는 새 회귀를
  추가하지 않고, 현재 코드는 보정 후보로 기록합니다.


### 흐름 밖 그림 후보의 정상 본문 반례와 계약 재검토

- `ee269c1b6` Native 9월 13쪽은 83.40392%에서 97.44297%로 개선됐지만,
  6쪽은 97.44190%에서 91.48515%, 7쪽은 93.07072%에서 83.70855%로
  악화했습니다. 이는 실행으로 검출한 후보의 회귀이며 기존 차이로 면제하지 않습니다.
  원본 10월 전체 최저와 수식 대조군은 직전 값과 같았습니다.
- 잘못된 가정은 `그림의 전진 0 + 마지막 저장 줄 인덱스 존재`만으로 현재 배치가
  저장 좌표축을 그대로 소비했다고 판단한 것입니다. 정상 본문은 같은 인덱스의
  수식 줄을 다른 메트릭으로 구성할 수 있습니다. 6쪽 문단 348은 마지막 저장
  높이 1125 HWPUNIT와 실제 약 12.1px 상자가 다르고, 7쪽 문단 386은
  첫 줄부터 마지막 줄까지의 실제 전진이 저장 span과 다릅니다. 관측값은
  정상 반례의 원인 확인이며 고정 px 회귀 기대값으로 사용하지 않습니다.
- 저장 태그를 신뢰하는 조건을 추가로 완화하지 않습니다. 같은 source 소유의
  모든 실제 줄 원점·높이와 저장 줄 상자가 같은 좌표축에 놓였는지, 마지막 줄의
  높이·후행 간격·문단 뒤 간격까지 실제 흐름이 소비했는지 대조합니다.
  이 계약이 성립할 때만 그림 출력 후 원점을 보존하고, 재조판·부분 예약 경로는
  원점을 확립했다고 주장하지 않습니다. 전진 0 자체를 줄 소유의 증거로 쓰던
  첫 후보의 가정을 제거한 보정이며 다시 원본 전체를 출력해 검증합니다.


### 비가시 호스트 줄의 실제 저장 예약을 전달하는 후보

- 14쪽의 그림 호스트 가상 문단 741도 `FullParagraph → Shape → 다음 문단`에서
  전진이 중복됩니다. FullParagraph의 저장 사다리 분기는 18.0267px의 줄
  공간을 실제로 예약하고, Square 그림의 전진은 0입니다. 이후 기준을
  초기화해 다음 문단에 6.0267px의 기본 줄간격을 다시 더합니다. 이 수치는
  진단값이며 회귀의 고정 픽셀 기대값으로 쓰지 않습니다.
- 이 경로는 글자가 없어서 TextLine을 출력하지 않습니다. 줄 노드 부재를
  점유 부재로 취급하지 않고, 기존 예약 분기가 실제 수용한 저장 사다리의
  끝을 같은 단·같은 호스트의 그림 후처리에 전달합니다. 후처리에서 예약
  조건·높이를 다시 추정하지 않습니다. 단 시작과 다음 텍스트 조각에서
  예약 결과를 초기화하고, 소유 문단과 현재 전진 끝이 같을 때만 소비합니다.
- 합성·편집 줄과 폴백 추정 예약은 저장 결과로 전달하지 않습니다. 실제
  전진하는 그림과 다른 문단/조각의 결과도 재사용하지 않습니다. 이 후보는
  13쪽의 실제 방출 줄 계약과 별개인 비가시 줄 예약 경로를 같은 완료
  계약으로 연결한 것이며, 정상 본문 대조군과 원본 전체 재출력으로 검증합니다.


### 윗줄 위첨자와 근호의 낡은 수식 프레임 — 독립 재생성 보정 후보

- `1ee24dabf` Native 9월 원본의 남은 미달은 12쪽 89.82289%, 21쪽 81.20332%입니다.
  13·14·22쪽과 정상 본문 6·7쪽은 90% 이상입니다. 10월 원본 18쪽 최저와
  기존 수식 대조군은 직전 값과 같았습니다. 아직 새 fresh WASM 검증 전입니다.
- 21쪽 문27 내부의 윗줄·위첨자를 포함한 복합 근호 두 줄 뒤에서 본문이 각각
  약 2.96px씩 밀립니다. 원본 저장 수식 높이는 1584 HWPUNIT입니다. 단순
  글자·윗줄 계약의 미지원 구조여서 `flow_metrics_hwp`가 이 저장 높이를 유지했습니다.
  뒤 간격만 줄여 오차를 숨기지 않고 이 수식 프레임의 유효성을 독립적으로 확인했습니다.
- 기존 HCR 진단 문서에서 크기·기준선·저장 줄을 비운 수식 84개를 생성했습니다.
  14개 BaseUnit마다 숫자 근호, 윗줄 위첨자, prime 포함/미포함 윗줄 근호와
  단일 근호를 포함합니다. MCP 2020 재생성 job은
  `9e55a726-899d-4a7f-90e1-1e6a277946cb`이며 한컴 11.0.0.9136입니다.
  900 HWPUNIT의 윗줄 위첨자 높이/기준선은 1227/89%, 그 표현식의 근호는
  1362/91%입니다. prime 유무가 이 프레임을 바꾸지 않았습니다.
- 같은 재생성 HWP를 Print PDF로 출력한 job은
  `cb0715b4-70c0-486c-9258-0d8b1e96e889`입니다. `pdf_print_method: 0`,
  `pdf_output_mode: hancom2020_pdf_driver_one_up`, 전체 3쪽을 확인했습니다.
  입력·산출 해시와 원시 메트릭은 ignored output에 보존합니다. 재생성 대조군을
  원본 입력의 통과로 보고하지 않습니다.
- 소비 경로는 `독립 프레임 → flow_metrics_hwp → refresh_equation_rows의
  같은 줄 기준선 합집합 → 미주 row_projection/format의 높이·후속 vpos →
  paragraph_layout의 실제 줄/흐름 → layout_for_paint의 같은 기준선 원점`입니다.
  특정 문서 ID·수식 문자열 대신 윗줄에 단일 위첨자가 붙은 구조와 그 구조의
  단일 근호만 지원합니다. 중첩 근호·분수·아래첨자·근 지수·별도 크기 변경은
  이 프레임으로 접지 않으며 유효한 원래 높이는 유지합니다.
- 이는 원본 재출력 전 보정 후보입니다. 독립 대조군과 원본 전체 페이지의
  Native/fresh WASM 및 실제 review PNG를 다시 확인한 뒤 판정합니다.


### 수식만 있는 줄의 실제 소속과 일반 내어쓰기 원점 공유

- `a608a2867` 후보는 수식 paint 프레임을 바꾸었지만 21쪽의 흐름 일치율은
  81.20332% 그대로였습니다. 이를 개선 완료로 기록하지 않습니다. 원본의
  해당 문단은 text가 비어 있고 수식 위치가 `[0, 1, 2, 3]`입니다. 메트릭 계산은
  text 길이 0을 끝으로 삼아 첫 수식만 포함했지만, 실제 배치는 네 수식을 같은
  저장 줄에 소속시켰습니다. 같은 helper 뒤의 좌표 공유 이전에 소속부터 달랐습니다.
- 수식 전용 문단의 줄 메트릭 계산에서 표시 경로가 사용하는
  `equation_only_tac_line_assignment`의 소속 결과를 소비합니다. 다줄·퇴화 위치도
  같은 결과로 처리하며, 일반 텍스트 혼합 줄의 소속 규칙은 유지합니다. 단순히
  모든 TAC를 한 줄이라고 간주하는 조건을 추가하지 않습니다.
- 12쪽은 별도 수식 재줄바꿈이 없는 문단의 빈 run 줄에서 일반 글줄의 확정
  내어쓰기 원점을 버리고 HWP3 계보 0.5배를 다시 적용했습니다. 원본 문단의
  정상 내어쓰기는 3824/2 HWPUNIT이며 일반 글줄에는 이미 반영됩니다.
  별도 수식 흐름이 없으면 `effective_margin_left`를 그대로 사용합니다.
  폭에 따른 별도 수식 줄 구성의 원점과 이 정상 저장 줄 경로를 구분합니다.
- 확대 PNG에서 절댓값과 중괄호가 없다고 추정했던 것은 잘못된 진단입니다.
  원본 SVG에는 둘 다 존재하며 파서는 동일 구조를 보존합니다. 파서 변경으로
  처리하지 않습니다. 같은 쪽의 실제 원점과 최종 래스터를 다시 대조합니다.
- 두 변경은 원본 재검증 전 후보입니다. 새 head의 전체 페이지 Native/fresh WASM,
  정상 본문·혼합/수식 전용·다줄 대조군과 직접 PNG 판독을 이어서 수행합니다.


### 원본 9월 전체 페이지의 실제 검증 결과

- source `a441b407642255d818e477f33e8dd6a6b37634c7`에서 원본 23쪽의
  Native/fresh WASM 최저 일치율은 모두 90.78108%입니다. 12쪽은 96.32357%,
  21쪽은 97.03697%이며 정상 본문 6·7쪽도 각각 97.44190%와 93.07072%입니다.
  새 review PNG를 직접 열어 12쪽의 내어쓰기와 21쪽의 복합 근호 뒤 본문 간격을
  확인했습니다. 두 출력 경로와 기준 Print의 쪽수는 모두 23쪽입니다.
- 10월 원본 18쪽, eq-002 HWP/HWPX, dyad 원본/최소 입력, HCR 프레임·복합 적분·
  근호 경계 대조군까지 Native/fresh WASM 9건을 실행했습니다. 각 최저값은
  91.18877%, 92.51376%, 92.51376%, 95.16512%, 95.89197%, 96.48090%,
  100%, 95.65151%입니다. 원시 TSV·실행 로그는 ignored output에만 보존합니다.
- WASM SHA-256은 `6b0bb621b113a143217171c1b4362822953bb377e6fd9e399ca8718a575304cf`이며
  루트 pkg와 Studio public의 해시가 같습니다. 실제 headless Chrome에서
  Canvas2D·CanvasKit 4건을 새로 실행해 수식 7개/4개의 표시를 확인했습니다.
  CanvasKit의 한컴산뜻돋움·함초롬바탕 대체는 남아 있으므로 이 실행을 해당 글꼴의
  정확한 공급이나 CanvasKit의 90% 시각 통과로 확대하지 않습니다.
- 같은 source의 기존 #1139 77건은 54 PASS / 23 FAIL입니다. 90%를 통과한
  9월 7쪽 검사도 기존 고정 좌표 기대값으로 실패했습니다. 다른 실패 입력은
  원본의 Print와 직접 대조하여 실제 배치 결함과 잘못된 기대값을 구별합니다.
  아래 추가 보정 뒤에는 이 source의 결과를 최종 head 검증으로 재사용하지 않습니다.

### 단 밖 그림 뒤의 빈 호스트 줄 중복 예약 보정

- 11월 원본 1쪽은 `a441b4076`에서 83.45544%입니다. 문9 본문은 한컴 Print와
  저장 LineSeg보다 호스트 줄 하나만큼 아래로 밀립니다. 문단 69의 그림은
  단 오른쪽 밖에 놓여 그림 높이를 예약하지 않습니다. 빈 FullParagraph 경로가
  저장 사다리의 다음 줄까지 실제 전진한 뒤, Shape 경로가 #683의 뒤 간격으로
  같은 줄을 한 번 더 더했습니다. 이때 기준을 초기화하면 다음 줄도 중복된
  후행 간격에서 원점을 재구성합니다.
- 소비 경로는 `FullParagraph의 저장 델타 예약 끝 → 단 밖 그림의 비예약 결과 →
  TopAndBottom 뒤 간격 추가 → HeightCursor 원점 초기화 → 다음 본문`입니다.
  단 밖 그림의 기존 실제 프레임 판정을 뒤 간격에도 사용해 그림 띠를 예약하지
  않는 경우에는 그 뒤 간격을 더하지 않습니다. 실제 완료된 저장 호스트와
  전진량이 0인 TopAndBottom Shape도 같은 원점을 보존합니다. 그림의 표시·앵커는
  유지하며, 실제 그림 띠를 예약하는 경우와 저장 줄이 무효인 경우는 적용하지 않습니다.
- 보정 후보의 원본 재출력과 정상 대조군 검증은 진행 중입니다. 1쪽의 출력 개선과
  fresh WASM 확인 전에는 관련 기존 회귀 기대값을 바꾸지 않습니다.


- `bfca7db5e`의 11월 원본 전체 21쪽 Native 재출력에서 1쪽은
  83.45544% → 93.79509%입니다. 새 review PNG의 문9 시작·선택지가 Print에
  복귀했습니다. 8·11·12·14·17·18·19·20·21쪽은 여전히 90% 미만이므로
  전체 문서 통과로 보고하지 않습니다. 같은 source의 9월·10월·기존 수식
  대조군 Native 8건은 기존 최저값을 유지합니다. fresh WASM 검증은 진행 중입니다.

### 다음 줄 원점의 과도한 투영 — 추가 보정 근거

- 기하 기호를 일반 프레임으로 잘못 분류했다는 가설은 독립 MCP 재생성으로
  기각했습니다. job `6189603e-e466-4ea1-a1db-e06a3d5c2551`의 9pt 네 기하
  표현식은 높이 900 HWPUNIT·기준선 86%로, 현재 논리 프레임과 같습니다.
  따라서 기하 기호를 지원 대상에서 임의로 제외하지 않습니다. Print job은
  `16297e61-0d94-430a-9e11-6a20c8de9c1e`입니다.
- 11월 17쪽 문28의 문단 3 첫 줄은 저장 선언 높이 2070, 실제 내용 높이 1125,
  뒤 간격 452이며 다음 줄까지의 저장 델타는 1577입니다. 다음 줄의 실제 수식이
  2070을 소유해 앞줄 선언에 중복 예약된 계약을 만족합니다. 프레임을 900으로
  바꾼 뒤 현재 투영은 선언 높이 차이 1170까지 다음 원점에서 빼므로 글줄이
  5.4267px만 전진해 겹칩니다. Print의 같은 줄은 1577 HWPUNIT 전진을 보존합니다.
- 이 계약에서 유효한 저장 전진을 실제 줄 간격에 포함해 표시·투영이 같은 결과를
  소비하도록 보정합니다. 일반 낡은 수식 프레임의 재조판과 다음 개체가 중복
  예약되지 않은 경로는 기존 논리 메트릭을 유지합니다. 현재 후보는 ignored
  output에만 준비했고 WASM 빌드 종료 후 적용·재검증합니다.


### 90% 통과 쪽의 미세 그림 위치 결함 재확인

- 기존 9월 7쪽 그림 검사 실패를 잘못된 기대값으로 판단했던 기록을 정정합니다.
  PDFium으로 독립 PDF의 실제 이미지 경계를 추출하면 타원 그림은 y=588.156,
  현재 Native는 y=594.760입니다. 같은 쪽 포물선의 PDF/Native 시작은
  524.7053/525.320으로 일반 출력 좌표 차이이며, 타원의 추가 6px는 별도 배치 결함입니다.
  93.07072%의 페이지 점수는 이 결함을 면제하지 않습니다.
- 타원 호스트 문단 386의 1·2줄 수식 프레임 갱신이 각각 225 HWPUNIT만큼
  줄 전진을 줄입니다. 본문은 이 실제 줄 메트릭을 소비하지만 그림 앵커는
  원래 좁은 줄까지의 저장 vpos 델타를 그대로 더해 450 HWPUNIT를 중복 보존합니다.
  그림도 같은 저장 줄 투영 결과의 좁은 줄 원점을 소비하도록 보정합니다.
  원본·독립 Print와 포물선 대조군을 다시 판독한 뒤 관계 검사를 확정합니다.


- `59e080fd5` Native에서 9월 원본 23쪽의 최저 90.78108%는 유지됐고,
  타원 그림의 y는 594.760 → 588.760으로 실제 좁은 줄과 같아졌습니다.
  11월 17쪽은 72.08955% → 89.04464%로 개선됐지만 아직 미달입니다.
  이어지는 18쪽은 59.31995% → 44.21021%로 나빠졌으며, Print에 있는 마지막
  적분 줄을 다음 쪽으로 보내는 컷 차이를 확인했습니다. 전체 문서 통과가 아닙니다.
- 정상 10월 원본에서도 9·10쪽이 80.56245%/65.25582%로 낮아졌습니다.
  이 후보를 승인하거나 회귀 기대값 변경의 근거로 쓰지 않습니다. 저장 앞줄
  공간을 보존하는 변경이 커진 실제 수식 상자의 전진도 묶는지 추적하고,
  원인을 보정한 뒤 원본·정상 대조군을 다시 실행합니다.
- `bfca7db5e`의 fresh WASM 11건은 각각 Native와 같은 최저값으로 통과했습니다.
  11월은 전체 문서가 아니라 영향 1쪽만 93.79509%로 확인했습니다.
  WASM SHA-256은 `1c4f96196c726029f711c8a3bc92a37091aa71507df58350f6f9cc05a80abd43`이며
  pkg/public 해시와 실제 Studio의 응답 바이트가 같습니다. Canvas2D·CanvasKit
  4건에서 수식 표시와 실행 오류 없음도 확인했습니다. 이후 `59e080fd5` 결과로
  이 검증을 재사용하지 않습니다.


### 대체 본문 글꼴의 재조판 결과와 저장 줄 공간 구분

- 수식 메트릭의 생성 경로를 `Stored / LogicalFont / ReplacementText`로 함께
  발행합니다. 기존 `flow_metrics_hwp`의 폭·높이·기준선 계산은 유지하고,
  줄 구성은 같은 반환 결과의 출처도 소비합니다. HYhwpEQ의 한글을 실제 본문
  글꼴로 대체하여 재조판한 행에는 원래 수학 글꼴의 줄 공간을 재사용하지 않습니다.
  이 경우 새 논리 점유와 간격을 사용하고, 유효한 저장 앞줄 예약을 확인한 일반
  수식 행에서는 앞서 증명된 줄 공간을 보존합니다. 문서 ID·좌표 조건을 추가하지 않습니다.
- 이 수정은 이전 후보가 정상 10월 대조군에 만든 회귀를 보정하는 단계입니다.
  Native 빌드는 통과했으며, 같은 소스의 원본과 대조군을 재출력합니다. 해당
  재검증 전에는 이 단계의 개선이나 90% 통과를 단정하지 않습니다.
- MCP 진단의 원본·재생성 HWP는
  [진단 입력 보존](../assets/pr_7600_equation_font_diagnostic/README.md)에
  바이트와 해시를 함께 기록했습니다. 각 Print PDF도 보존합니다. 원시 TSV·로그·
  JSON은 output에만 두며 실패 분석용 84개 문서를 회귀 golden으로 고정하지 않습니다.


- `1f462f9d4`에서 정상 10월 원본 전체 18쪽의 Native 최저 일치율은
  91.18877%로 복구됐습니다. 9·10쪽 미달이 해소됐으며, 불필요하게 테스트
  기대값을 변경하지 않고 메트릭 소비 경로를 보정했습니다. 나머지 대조군과
  최종 source의 fresh WASM 확인은 계속 진행합니다.

### 저장 원점 되감김의 실제 fit 재생 후보

- 11월 원본 17쪽의 문29 마지막 적분은 Print에 표시되지만 이전 보정 후보는
  이를 18쪽으로 넘겼습니다. 그 줄은 source owner 330의 미주 문단 9이며
  저장 vpos가 0으로 돌아갑니다. 기존 `move_internal_rewind_equation_to_next`는
  단 높이 75%를 넘으면 실제 fit을 확인하지 않고 이월합니다.
- `simulate_endnote_column_bottom_y(..., Some(en_para_idx), true)`의 실제
  LayoutEngine 재생 결과를 사용해 끝까지 수용 가능한지 확인합니다. source
  원점 되감김만으로 들어가는 수식을 넘기지 않으며, 측정 불가는 기존 이월을
  유지합니다. 기존 fit 허용치를 넓히거나 그림/본문을 숨기지 않습니다.
- 경로는 `현재 단의 source 소유 항목과 다음 전체 문단 → 실제 paint의 흐름 끝 →
  예산 실패 시 이월 → 실제 방출`입니다. 새 후보의 Native 빌드·17/18쪽과 전체
  쪽수·정상 대조군을 먼저 검증합니다. 실제 컷·누락·중복·이어지는 본문을 확인한
  뒤에만 회귀 관계 검사를 확정합니다. 현재는 미검증 후보입니다.


### 저장된 실제 전진 기준의 투영 보정 — 앞 후보의 가정 재검토

- 실제 재출력에서 `75b6c9b1a`의 되감김 fit 후보는 11월 17·18쪽을 바꾸지
  못했습니다. 해당 마지막 수식은 한 줄짜리 문단의 첫 vpos 되감김으로,
  수정했던 문단 내부 되감김 분기의 대상이 아닙니다. 조건 이름으로 원인을
  추정한 진단을 정정하고 이 불필요한 후보 변경을 복원했습니다.
- 앞줄의 실제 저장 전진은 `text_height + line_spacing`인데, 줄 투영은 다음
  개체가 포함된 선언 `line_height`와 새 높이의 차이를 뺐습니다. 11월 문28의
  예에서는 원래 실제 전진 1577 HWPUNIT에서 새 전진 1352로 바뀌므로 차이는
  225입니다. 선언 높이 2070을 기준으로 1170을 빼는 것이 과도한 당김의 원인입니다.
  원래 내용 높이 1125를 새 줄 공간으로 계속 고정하는 앞 후보도 실제 새 전진을
  반영하지 못해 제목과 마지막 적분 줄을 밀었습니다.
- `stored_row_projection`은 원래 다음 vpos가 내용 높이와 간격의 합을 증명할 때
  그 실제 전진 높이를 기준으로 재조판 차이를 계산합니다. 메트릭을 계속 낡은
  strut로 묶는 변경과 이를 좁히기 위한 `EquationFlowFrame` 추가도 정리했습니다.
  수식의 실제 메트릭·앞줄/뒤줄의 소속은 기존 공통 구성 결과를 그대로 사용합니다.
  특정 문서·수식 문자열·픽셀 위치를 조건으로 추가하지 않습니다.
- 어울림 그림이 같은 줄 투영 원점을 소비하는 보정은 유지합니다. 새 후보의
  Native 원본·정상 대조군과 확대 판독을 다시 수행하며, 이 단계는 검증 전입니다.


- 뒤늦게 완료된 `75b6c9b1a`의 9월 전체 대조군에서도 최저 36.59233%의
  회귀가 검출됐습니다. 이 후보의 통과를 주장하지 않습니다. 해당 되감김 후보는
  앞서 불필요한 변경으로 이미 복원했으며, 실제 저장 전진을 사용한 새 후보에서
  9월 전체 페이지의 복구도 다시 확인합니다.


### 실제 저장 전진 보정의 원본 재검증

- 구현 head `18cae770d960ffff8da59a70c1d4dbe4f5076a17`의 Native 재출력에서
  11월 17쪽은 95.07303%로 통과했습니다. 문28의 두 줄이 겹치지 않고,
  문29 마지막 적분도 독립 Print와 같은 17쪽에 남습니다. 1쪽은 93.79509%입니다.
  전체 21쪽의 최저값은 여전히 32.34179%이므로 전체 통과나 회귀 변경 완료로
  판정하지 않습니다. 같은 head의 fresh WASM 재검증은 진행 중입니다.
- 정상 대조군은 9월 23쪽 최저 90.78108%, 10월 18쪽 최저 91.18877%를
  유지했습니다. 9월 7쪽은 95.28674%로 개선됐으며 타원과 첫 좁은 줄 원점이
  같습니다. 대표 review PNG를 직접 판독하여 타원·축의 추가 위치 차이가
  해소됐음을 확인했습니다. 수식과 그림의 남은 윤곽 차이를 완전 일치로 보고하지 않습니다.

### 빈 미주 그림 호스트의 저장 프레임 연결 후보

- 11월 11쪽의 원본 owner 99, 내부 문단 11은 빈 호스트이며 그림의 위 오프셋은
  97 HWPUNIT, 높이는 11694입니다. 앞 줄 끝과 호스트 vpos는 6852로 같고,
  뒤 첫 줄은 `6852 + 97 + 11694 = 18643`에서 시작합니다. 독립 Print에서도
  그림 뒤에 호스트 한 줄을 다시 예약하지 않습니다. 양수 오프셋만 제거하는
  예외나 특정 문서 분기를 추가하지 않습니다.
- 이전 head의 실제 로그에서는 호스트 문단까지 y=145.40이며 그림 항목 뒤는
  320.64입니다. 이는 실제 그림 끝에 호스트 전진 1352 HWPUNIT를 다시 더한
  값입니다. 한편 페이지네이터는 그림 문단에 18.03만 누적하여 물리 띠를
  누적 높이에 충분히 반영하지 않습니다. 배치와 측정의 두 소비 지점을 함께 보정합니다.
- 기존 `stored_picture_empty_host_placement`의 앞/현재/뒤 줄 검증을 호스트
  상대 프레임으로 재사용합니다. `상대 프레임의 흐름 끝 → en_fit/total_advance_fit →
  단 이동 뒤 실제 원점의 확정 placement와 en_advance → FullParagraph/Shape 출력`
  순으로 같은 결과를 소비합니다. scratch 단 측정도 확정 placement를 전달받습니다.
  기준 PDF가 없는 합성 입력으로 수용 조건을 완화하지 않습니다.
- 현재는 구현 후보의 Native 빌드 단계입니다. 영향 페이지와 정상 대조군의
  실제 출력·쪽수·후속 내용·기준과의 미세 간격을 확인하기 전에는 개선 완료로
  판정하거나 해당 회귀의 기대값을 변경하지 않습니다.


### 원본 컨테이너 확인과 실제 줄 소비 기록

- `74413abb6`의 Native 9건은 앞 head와 같은 수치였습니다. 11월 원본의
  `layout_profile`은 HWP5 컨테이너이지만 HWP3 계보가 남아 `native_hwp5_layout`
  및 `hwp5_stored_pagination_layout`이 false입니다. 앞 후보의 적용 조건을
  잘못 골라 보정이 실행되지 않았습니다. 실제 원본은 바꾸지 않고, HWP5
  컨테이너 또는 저장 HWPX의 유효한 앞/현재/뒤 줄 프레임으로 판정합니다.
- 8쪽 문28은 처음 세 본문 줄의 실제 baseline이 Print와 일치하지만 다음
  문단에서 옛 저장 높이로 복원되어 추가 공간이 생깁니다. Native/Print의
  `직선` baseline은 613.173/607.200이며, 뒤 그림의 실제 원점은
  696.000/686.2893입니다. 이 숫자는 원인 진단이며 회귀의 고정 px 기대값이 아닙니다.
- 기존 커서는 폭 재조판의 수용 조건을 세로 수식 메트릭 변경의 대용으로
  사용했습니다. 원본 미주를 소유한 본문은 이 조건에서 빠질 수 있습니다.
  실제 구성 결과와 저장 줄의 높이·간격 차이를 소비 기록으로 전달하고,
  다음 저장 줄이 원래 줄 끝에 연속될 때 현재 흐름 원점을 유지합니다.
  줄 수가 달라진 경로·dirty 저장 분할·합성 줄은 이 기록의 대상이 아닙니다.
  표시와 측정은 같은 구성 결과를 사용하며 글자를 숨기거나 좌표를 clamp하지 않습니다.
- `18cae770d`의 fresh WASM 12건은 Native와 같은 최저값으로 통과했습니다.
  11월은 전체 문서가 아닌 1쪽 93.79509%, 17쪽 95.07303%입니다.
  WASM SHA-256은 `0bba788b6bdffdb2b1bc1441baede0ffef2857a241dfeb821eaf0f73d3e39676`이며
  pkg/public 및 Studio 실제 응답 바이트가 일치합니다. Canvas2D·CanvasKit 4건의
  수식 표시·오류 없음도 확인했습니다. CanvasKit의 한컴 글꼴 대체 기록은
  남아 있으므로 이 실행을 화면 픽셀 충실도의 별도 증거로 승격하지 않습니다.
- 이번 추가 보정은 새 Native 및 fresh WASM 검증 전입니다. 원본 미달 페이지의
  출력 보정과 90% 이상 확인을 먼저 수행하며 해당 회귀 기대값은 아직 변경하지 않습니다.


- 새 커서 필드를 수동으로 구성하는 단단 본문 경로에서도 실제 구성 결과를
  전달하도록 호출부를 연결했습니다. 첫 Native 빌드는 이 초기화부의 새 필드
  누락으로 E0063이 발생했으며 자체 구현 오류로 보정했습니다. 이는 원 결함의
  수정 전 FAIL 증거로 세지 않습니다. 새 일반 커밋에서 빌드와 원본 시각 검증을
  다시 실행합니다.


### 사용자 지적: 11쪽 하단의 누락과 배치 차이

- `7230daa86` Native에서 8쪽은 96.52127%, 11쪽은 94.70319%, 1쪽은
  95.76223%로 개선됐습니다. 그러나 11쪽 하단 확대에서 `ⅳ)` 다음의
  `h(x)=x(x+2)`와 극한·최솟값 설명이 Print와 같은 쪽에 표시되지 않는
  차이를 확인했습니다. 사용자의 지적을 반영해 이 쪽의 90% 점수 통과를
  내용 보존·분할 통과로 판정하지 않습니다. 영향 11·12쪽을 함께 보정합니다.
- 원본 owner의 미주 14, 내부 문단 8은 11줄이며 8번째 다음 저장 vpos가
  0으로 돌아갑니다. Native는 이 내부 저장 컷을 8줄/3줄로 유지하지만
  독립 Print는 지적한 수식과 설명을 11쪽에 함께 배치합니다. 저장 컷과
  실제 줄 점유·실제 fit 경로를 대조하며, 테스트의 기존 컷을 먼저 바꾸지 않습니다.
- 새 후보의 정상 10월 9·10쪽에서도 82.51399%/45.18476%의 회귀가
  검출됐습니다. 갱신된 실제 세로 메트릭과 폭 재조판의 수용 조건을 혼용한
  커서 조건을 재검토합니다. 실제 소비 기록 없이 폭 변경만으로 세로 저장
  전진을 면제하지 않으며, 이 후보의 새 WASM을 최종 검증으로 진행하지 않습니다.
- 최신 upstream은 `fe9ffd27e91d8d260fbccde5ff68aa08aadf94ca`입니다.
  이전 base 이후 #7627과 #7544의 두 변경이 추가됐습니다. 동기화와
  정책 base 갱신 뒤 해당 최종 head를 다시 검증합니다.


### 뒤 수식 예약을 공유하는 여러 앞줄의 점유 보정 후보

- 사용자 지적의 원본은 owner 117의 미주 14, 내부 문단 8입니다. 저장
  7번째 줄과 8번째 줄은 내용 높이 900에 선언 높이 1662를 공유하고,
  9번째 줄의 실제 극한 수식이 1662를 소유합니다. 8번째 줄은 다음 원점이
  `내용 높이 + 간격`에서 시작하므로 기존 보정이 적용됐지만, 내부 저장 컷
  앞의 7번째 줄과 앞 문단의 일반 텍스트 줄에는 같은 예약이 남았습니다.
- 직후 한 줄만 보는 대신 같은 선언 높이를 공유하는 연속 저장 예약 구간을
  실제 후속 개체의 소유까지 확인합니다. 각 중간 줄의 전진은 내용 높이와
  간격으로 닫히거나 저장 원점으로 돌아가야 합니다. 후속 실제 개체 높이·
  source의 내용 높이·현재 텍스트/수식의 기준선이 함께 맞는 경우에만 현재 줄의
  실제 합집합을 재발행합니다. 빈 줄의 strut·dirty 분할·합성 사다리는 이 근거로
  삭제하지 않습니다. 같은 구성 결과를 format·투영·paint가 소비합니다.
- Native HWP5 커서의 저장 전진 면제는 실제 세로 메트릭 소비 기록이 있을 때만
  적용하도록 보정합니다. 기존 HWPX의 폭 재조판 계약은 유지하며, 폭 변경만으로
  Native 문단의 세로 저장 원점을 건너뛰는 앞 후보의 혼용을 제거했습니다.
- 새 후보는 원본 11·12쪽의 빠진 수식·문장과 다음 쪽 내용, 10월 정상 대조군을
  재검증하기 전입니다. 전체 쪽수와 source 소유·내용 순서·누락·중복도 함께
  확인하며, 페이지 점수만으로 이 사용자 지적을 완료 처리하지 않습니다.

### 실제 부분 문단의 점유로 저장 분할 지점 재검토

- `f8faede1017727c7bc2a2def585cb2b10563f97b`의 Native 빌드와 9개 문서
  재출력이 끝났습니다. 10월 전체 18쪽은 최저 91.24244%로 정상 대조군을
  회복했고, 9월 전체 23쪽은 최저 90.78108%를 유지했습니다. 수식 6개
  대조군도 모두 90% 이상입니다. 11월은 21쪽이며 최저 32.34179%로
  전체 검증을 통과하지 못했습니다. 11쪽 94.70319%에서도 지적한 내용이
  다음 쪽으로 넘어가므로 승인 근거가 아닙니다.
- 확대 판독을 보완했습니다. Print의 11쪽에는 `h(x)=x(x+2)`와 극한 식을
  포함한 최솟값 설명이 있고, 마지막 `존재하지 않는다.` 줄은 12쪽에서
  이어집니다. 전체 11줄을 11쪽에 고정하는 것은 올바른 기대값이 아닙니다.
  기존 Native 컷은 8줄/3줄로 두 줄을 더 넘깁니다. 기존 실패 입력과 출력은
  `following-reservation-rows-native-controls/nov/`에 보존했습니다.
- 실제 수식 줄 메트릭이 원본과 달라진 내부 저장 컷에 한해, 확정 후보보다
  뒤의 부분 문단을 실제 `PageItem::PartialParagraph`로 scratch LayoutEngine에
  전달합니다. 현재 단의 항목·전역 소유자·그림 프레임·구성 결과를 같은 paint
  경로로 재생하며, 마지막 줄 뒤 간격 대신 실제 점유 끝을 가용 높이와
  대조합니다. 들어가는 가장 긴 앞부분을 분할 후보로 사용합니다. 고정 px
  위치나 허용치 증가는 사용하지 않습니다. 표·그림·도형을 포함한 수정 대상
  문단과 명시적 단 구분은 이 텍스트/수식 부분 분할 보정 대상이 아닙니다.
- 생산 결과는 원본 대비 실제 줄 메트릭 소비 기록이며, 측정은 부분 PageItem의
  실제 점유 끝, 소비는 `emit_endnote_split`의 같은 시작·끝 줄 범위입니다.
  실제 출력에서 11·12쪽 소속과 뒤 그림·문장까지 재검증할 예정이며,
  Native/fresh WASM 선행 조건을 충족하기 전에는 정식 회귀 기대값을 바꾸지 않습니다.

### 하단 두 줄 복귀와 후속 단 경계의 추가 보정

- `8c252adfb51718d35801ced127de4295a99cbb23`의 Native 빌드는 성공했습니다.
  원본 11쪽의 미주 14, 내부 문단 8은 10줄/1줄로 분할됐으며,
  실제 앞부분 점유 끝 997.32가 단의 가용 높이 1001.56 안에 들어갑니다.
  수식과 최솟값 설명은 11쪽, 마지막 문장은 12쪽으로 독립 Print와 같은
  소속을 회복했습니다. 확대 PNG와 11쪽 전체 review PNG를 직접 판독했습니다.
  11쪽은 94.70319% → 96.33025%, 12쪽은 46.00971% → 67.37533%입니다.
  12쪽은 아직 미달이며 이 결과를 분할 변경의 최종 승인으로 사용하지 않습니다.
- 12쪽 비교에서 미주 15, 내부 문단 17의 시작 두 줄이 독립 Print의 왼쪽 단
  하단에 있지만 Native는 문단 전체를 오른쪽 단으로 넘기는 차이가 남았습니다.
  원본에는 명시적 단 구분·keep-lines·keep-with-next·widow/orphan 설정이 없습니다.
  실제 수식 메트릭이 갱신된 문단 간 저장 되감김에도 같은 부분 점유 질의를 적용합니다.
  현재 단에 들어가는 앞부분이 확인되면 전체 문단 이동을 억제하고 같은 컷을
  기존 분할 방출 경로에 전달합니다. keep 설정과 widow/orphan의 양쪽 줄 수
  계약은 부분 후보 계산에서도 확인합니다.
- 이미 배치된 표가 있는 단의 부분 측정은 빈 표 캐시로 추측하지 않습니다.
  호출부의 실제 `MeasuredTable` 목록을 scratch LayoutEngine까지 전달해 실제
  paint와 같은 표 높이·현재 항목·전역 소유자를 사용합니다. 대상 문단 자체에
  표·그림·도형이 있으면 이 텍스트/수식 부분 분할 경로는 적용하지 않습니다.
- 검토 브랜치의 106개 커밋을 최신 `upstream/devel`의
  `fe9ffd27e91d8d260fbccde5ff68aa08aadf94ca` 위로 충돌 없이 rebase했습니다.
  원본 이력은 `backup/pr7600-before-prefix-rebase-20261007`에 보존했습니다.
  부분 점유 보정의 새 SHA는 `feea6ec0df99ff16a9323a11839523416e4b2ab8`입니다.
  원 contributor fork에는 어떤 변경도 하지 않았으며, 새 base/head는 재검증합니다.
- 이전 Native 9개 대조군 실행은 rebase가 진행되는 동안 마지막 두 fixture 경로가
  잠시 사라져 runner가 실패했습니다. 이는 실행 순서 오류이며 렌더 회귀 증거가
  아닙니다. rebase가 끝난 뒤 원본 SHA와 입력 해시를 유지한 immutable Native
  실행 파일로 해당 두 건을 재실행해 각각 96.48090%/100.00000%를 확인했습니다.
  9월·10월 전체와 수식 대조군 결과는 모두 이전 source `8c252adfb`의 증거이며
  rebase 이후 source의 시각 통과로 재사용하지 않습니다.

### 사용자 재지적: 12쪽 추가 그래프와 overlay 색상 소속

- `3e37b93ea366d269868ee55d9a4fbc35f4ea0c95` Native의 11·12쪽 수치는
  96.33025%/92.16245%지만, 12쪽 직접 판독에서 미주 19의 그래프가 본문
  하단 밖으로 내려오고 문20과 겹치는 결함을 확인했습니다. 전체 13쪽도
  기존 97.61688%에서 34.49027%로 악화됐습니다. 사용자가 지적한 그래프는
  독립 Print의 13쪽에 있어야 합니다. 12쪽의 수치 통과를 승인으로 사용하지 않습니다.
- 원본 owner 162의 미주 19, 내부 문단 5에는 명시 크기의 TAC 그림이 있고
  저장 원점은 0으로 돌아갑니다. 실제 그림 높이와 저장 줄 높이는 동일합니다.
  그러나 단일 줄의 실제 점유 질의는 앞서 배치된 표를 미래 문단 지원 여부
  검사로 함께 기각해 `None`을 반환했습니다. 이 경로가 실제 그림 넘침을
  확인하지 못하고 12쪽의 기존 저장 전진을 수용했습니다.
- 명시 크기의 TAC 그림 단일 줄은 현재 단 항목·전역 소유자·실제 표 측정을
  공유하는 기존 scratch LayoutEngine에서 전체 PageItem의 실제 점유 끝을
  질의합니다. 미래에 추가할 문단이 없는 질의에 이미 배치한 표의 일괄 지원
  기각을 적용하지 않습니다. 다른 텍스트/수식의 기존 지원 조건은 유지하고,
  실제 그림 점유 끝이 가용 높이를 넘으면 기존 단 이동을 실행합니다.
  12·13쪽, 뒤 문20의 소속·내용 순서와 전체 대조군은 새 head에서 확인합니다.
- 사용자가 색상 소속도 지적했습니다. 과거 `visual_sweep.py` 규약은 RHWP만
  빨강, PDF만 파랑이었습니다. 요청한 판독 기준으로 **RHWP만 파랑, PDF만
  빨강**으로 변경하고 standalone overlay와 review PNG에 범례를 포함했습니다.
  `overlay_color_legend`도 결과에 기록하며 가이드와 governance를 함께 수정했습니다.
  기존 `3e37b93ea`의 같은 12쪽 raster를 새 색상으로 합성해 누락 기준에 없는
  그래프가 파랑인 것을 직접 확인했습니다. 이는 새 런타임 재출력 증거가 아닙니다.
- 실제 양쪽 잉크의 차이와 일치 색상, 추가/누락 잉크 소속, 산출 PNG와 차이
  픽셀 수를 확인하는 도구 검사 3건을 추가했습니다. Visual Sweep 관련
  Python 검사 92건이 통과했고, deprecated pixel API를 제거한 뒤 해당
  3건을 다시 통과했습니다. 점수·잉크 분류·관용 반경·승인 허용치는 변경하지 않았습니다.

### TAC 그림 쪽 소속 복구와 다음 미달 수식의 원본 토큰 확인

- `e34cfd0ad5ebebdd9aec3d01a14994443c3a680f` Native 빌드는 5분 15초에
  성공했습니다. 실제 그림 점유 끝 1033.08이 단의 가용 높이 1001.56을
  넘는 것을 같은 paint 질의로 확인해 12쪽 오른쪽 단에서 다음 쪽으로 이동했습니다.
  11쪽은 96.33025%, 12쪽은 96.04768%, 13쪽은 97.61688%입니다.
  새 review PNG와 13쪽 전체 compare PNG를 직접 판독해 그래프·문20의 소속과
  앞뒤 내용 순서가 독립 Print에 맞는 것을 확인했습니다. 이전 12쪽 추가 그래프와
  문20 겹침, 13쪽 34.49027% 회귀는 새 출력에서 해소됐습니다.
- Native 전체 9개 대조군은 다시 완료했습니다. 9월 전체 23쪽은 최저
  90.78108%, 10월 전체 18쪽은 91.24244%, 수식 6개 대조군도 모두
  90% 이상입니다. 11월 전체 21쪽은 아직 14·18·19·20·21쪽이 미달이고
  최저 32.34179%입니다. 이 head의 fresh WASM과 최종 전체 회귀는 아직
  실행하지 않았으며, Native만으로 최종 승인이나 회귀 기대값 변경을 하지 않습니다.
- 14쪽 원본 스크립트에서 `CDOT3^{5-r}`가 곱셈 점과 숫자 3으로,
  `RMN LEFT ( itm,~ sigmaRIGHT )`가 닫힌 괄호와 그리스 기호로 Print됩니다.
  수정 전 tokenizer/AST 진단은 각각 문자 `CDOT3`, 문자 `sigmaRIGHT`와
  오른쪽 괄호가 비어 있는 Paren을 만들었습니다. 실제 원본 owner 195의 미주 23,
  내부 문단 2와 owner 229의 미주 27, 내부 문단 1에서 추출한 원 스크립트입니다.
- 숫자에 붙은 `cdot`는 대소문자에 관계없이 연산자와 숫자로 분리합니다.
  `cdots`와 `cdotValue`는 이 숫자 조건으로 분리하지 않습니다. 그리스 기호와
  `RIGHT`가 붙은 경우는 알려진 단일 그리스 기호와 뒤 닫는 구분자를 함께 확인해
  명령 경계를 복원합니다. 일반 그리스 접두사의 식별자는 임의로 분리하지 않습니다.
  원본 14쪽·전체 문서와 정상 대조군을 재검증하고, 실제 glyph·괄호·앞뒤 간격을
  직접 확인한 뒤 판단합니다. 정식 회귀의 추가·변경은 기존 시각 선행 조건을 유지합니다.

### 빈 줄의 점유 끝과 다음 미주 간격의 소비 지점

- `c4574b920` Native 전체 9개 실행은 완료됐습니다. 9월 23쪽·10월 18쪽과
  수식 6개 대조군은 모두 90% 이상입니다. 원본 11월 21쪽의 최저값은
  32.34179%이며, 14쪽은 토큰 경계 복구 후에도 71.34044%입니다.
  이 결과는 최종 승인이나 회귀 검사 변경의 선행 조건을 충족하지 않습니다.
- 원본 owner 204의 미주 24, 내부 문단 3은 마지막 개행 뒤 빈 줄을 저장합니다.
  해당 줄의 높이는 900HU이며, 실제 구성·paint도 높이 12의 줄 상자를
  소비합니다. 그러나 콘텐츠 하단은 빈 줄 원점 380.8267이고, 흐름 줄 상자
  끝은 392.8267입니다. 다음 미주 간격 소비자가 전자만 선택해 문25부터
  한 줄 높이만큼 위로 당겼습니다. 독립 Print의 문25 상단은 420.11이며
  Native는 407.3입니다. 단순 픽셀 보정 대신 실제 소유 줄 상자를 소비합니다.
- 생산은 `paragraph_layout.rs`의 마지막 줄 상자 기록이며, 소비는
  `layout.rs`의 `HeightCursor` 전달과 미주 제목 최종 원점,
  `height_cursor.rs`의 간격 예약·저장 원점 처리입니다. 공통 `occupied_end`는
  보이는 내용과 흐름 줄 상자의 끝을 합치고 후행 간격을 포함하지 않습니다.
  기존 콘텐츠 하단의 가시 영역 의미는 유지합니다. 실제 fit 재생은 같은
  LayoutEngine의 이 결과를 소비합니다. 스킵된 유령 줄은 기존 점유가 0이므로
  높이를 되살리지 않습니다. 같은 수정 범위의 원본·정상 대조군을 다시 실행하며
  결과 확인 전에는 빈 줄 회귀 기대값을 새로 고정하지 않습니다.

### 앞선 수식 줄의 변경과 저장 내부 컷의 잔여 높이

- `29e79e1bc` Native에서 빈 줄 간격 보정 후 14쪽은 95.98702%입니다.
  새 review PNG를 직접 확인했고 11·12·13쪽 점수는 그대로입니다. 18~21쪽은
  미달이 남습니다. 18쪽 원본 owner 340의 미주 30, 내부 문단 4는 저장
  컷 1/3을 유지하지만 독립 Print는 앞 세 줄을 왼쪽 단에 배치합니다.
- 현 조건은 현재 문단의 줄 높이가 변해야 실제 부분 점유를 조회합니다.
  그러나 이 문단 자체의 높이가 같아도 같은 단의 앞 수식 줄이 재발행되면
  저장 컷의 잔여 높이는 달라집니다. 원본 소유자를 따라 같은 단에 이미
  방출한 항목의 저장 줄과 실제 구성을 대조해 그 변경을 확인합니다.
  폭 차이·글자 유무·문서 식별자로 세로 변경을 추정하지 않습니다.
- 기존 PartialParagraph scratch 점유 질의를 재사용하고 keep 조건·명시적
  단 나눔을 유지합니다. 적용 증거는 같은 단의 재발행된 수식 줄이며,
  실제 컷은 현재 표 측정·원점·소유자를 공유하는 paint 경로로 정합니다.
  18쪽과 뒤 내용, 11~14쪽 및 정상 대조군을 새 source에서 확인합니다.

### 사용자 가스용품 표 캡처의 원본 확인

- 대응 원본은 `samples/issue5877/fragment_ghost_vrules.hwp`의 10쪽이며
  독립 기준은 `pdf/fragment_ghost_vrules-2020.pdf`입니다. 입력 SHA-256은
  `263a96fa24b4070a4a22cc4a62d878c50b7586d0741e039c58aa18615273d5a3`,
  PDF SHA-256은 `71dd147ab8d9be9f0d79b6e8d42c62d293cbb88e9a7cb645715aa6896343e22f`입니다.
  보존된 기준의 출력 출처는 과거 PR #7275의 MCP Print 기록입니다.
- 현재 작업 트리에서 immutable `c4574b920` Native 실행 파일로 재출력했습니다.
  런타임 binary SHA는 `4f9cfbb6665490749c8e427ff224b0b3ff6e1e562943a8f0a2eb2f763ac7b992`입니다.
  10쪽 실루엣은 95.36398%지만 실제 판독에서는 마지막 가스용품 행이
  다음 쪽으로 넘어갑니다. RHWP는 32쪽, PDF는 31쪽입니다. 이 차이는
  글꼴 예외나 점수 통과로 면제하지 않고, 기존 기대값도 수정하지 않습니다.
- 실제 scanner 진단은 행 23~25의 병합 블록 요구 높이 107.4, 잔여 63.6,
  행 오프셋 컷 높이 60.0이지만 `fully_consumed=true`를 반환합니다.
  뒤 행의 빈 밴드를 내용 완결과 구별하지 못해 밴드 후보가 기각되는
  경로를 추적 중입니다. 전체 구조 차이는 아직 해결하지 않았습니다.
  마지막에 보낸 마스크 이미지는 검증이나 산출에 사용하지 않습니다.

- `29e79e1bc`의 Native 9개 실행은 최종 종료까지 확인했습니다. 9월 23쪽
  최저 90.78108%, 10월 18쪽 최저 91.24244%, 수식 대조군 6개는
  92.51376%~100%이며 기존 정상 대조군의 점수 감소는 없습니다.
  11월 14쪽 보정은 Native에서 확인했지만 18~21쪽·fresh WASM·최종
  lint 및 전체 회귀가 남아 있어 PR 제출·통합은 보류 상태입니다.

### 일부 쪽 Visual Sweep의 전체 쪽수 gate 보완

- 기존 실행은 선택 쪽의 실루엣만 gate에 전달했습니다. 같은 10쪽의
  95.36398%가 전체 32쪽/31쪽 차이를 판정에 반영하지 못하는 것을 실제
  입력으로 확인했습니다. `--silhouette-only`도 일반 모드도 Native/WASM의
  export 전체 `pageCount`와 독립 PDF `pdfinfo`의 쪽수를 기록하고, 차이가
  있으면 선택 쪽 점수와 관계없이 `re_review_required`로 판정합니다.
- 실제 동일 입력 재실행 `gas-table-document-page-gate`는 원래 점수를
  유지하면서 `document_page_counts={rhwp:32,pdf:31}`,
  `page_count_mismatch=true`, exit 1을 기록했습니다. 이미지·점수 허용치나
  잉크 분류를 바꿔 결함을 숨기지 않았습니다. 원본 없는 `--png-pair`는
  전체 쪽수의 증거로 사용하지 않습니다.
- 도구 검사 5개를 추가했습니다. 일부 쪽의 높은 점수, 실제 쪽수와 모순된
  글꼴 예외, 같은 쪽수의 정확히 90%, Native/WASM 메타데이터, 동일 raster의
  추가 쪽을 검사합니다. 전체 97개 검사 첫 실행에서 기존 손상 글꼴
  검사의 실패 우선순위를 바꾼 오류를 발견했습니다. 실제 코드에서 쪽수
  확인을 임베디드 글꼴 검사 뒤로 옮겨 기존 손상 증거를 보존했고 재실행은
  97 PASS입니다. 기존 검사의 기대값은 완화하지 않았습니다.
- 다음 Native source `dba07c6bf2308ac8452e2af971a962026ffd98b6` 빌드는
  exit 0, 4분 33초입니다. immutable binary SHA-256은
  `2a807f7459897ed337435906e060f3bfa048a76a837e277b6844767b968a05d1`이며
  원본 영향 쪽·전체 정상 대조군을 재실행 중입니다.

### 그림 뒤 풀이 전체를 같은 단에 묶던 저장 원점 보정

- `dba07c6bf` Native에서 18쪽의 원본 내부 컷은 1/3에서 3/1로
  복구됐습니다. 실제 부분 점유 끝 990.52가 단 예산 1001.56 안에
  들어가며 독립 Print와 같은 줄 소속입니다. 18쪽은 59.31995%에서
  84.17703%로 개선됐으나 원본 그래프는 여전히 다음 쪽에 있습니다.
  11~14쪽 점수는 유지됐고 20쪽은 32.71963%에서 32.20795%로 변해
  직접 판독·후속 보정이 필요합니다. 이 부분 개선을 완료로 보고하지 않습니다.
- owner 340의 미주 30, 내부 문단 33은 높이 15704HU의 단일 TAC 그림입니다.
  현재 오른쪽 단의 높이는 735.05이며 그림 자체는 들어갑니다. 그러나
  `tac_picture_tail_group_height`는 뒤의 모든 내부 문단의 저장 끝을
  한 번에 예약해 그래프를 이월합니다. 독립 Print는 그래프와 뒤 두 줄을
  18쪽에 놓고 나머지 풀이를 19쪽에서 이어갑니다. 전체 미주가 원자적
  묶음이라는 가정을 제거합니다.
- 실제 단의 PageItem·표 측정·원본 소유자를 받는 기존 단일 그림 점유
  질의가 이월을 결정합니다. 일반 단일 줄 넘침 확인과 같은 scratch paint
  결과를 소비하고, 이미 소비한 trailing 간격과 뒤 풀이 전체의 저장
  높이를 그림 높이에 재가산하지 않습니다. 명시적 나눔과 실제 넘치는
  12쪽 그래프 대조군도 유지되는지 새 source에서 확인합니다.

- 일부 쪽 일반 모드도 동일 원본에서 직접 실행했고 실제 32쪽/31쪽 차이로
  exit 1을 확인했습니다. 이때 점수 미달과 쪽수 결함을 혼동하던 CLI
  문구도 고쳐 실제 사유와 두 쪽수를 표시합니다. 추가 focused 14개 검사와
  실제 원본 재실행이 통과했으며 `pdfinfo`도 필수 도구 확인에 포함했습니다.
- 그림 자체 점유 보정 `3020c479878aaed0d05c8d200f36f2b4b2587807`의
  Native 빌드는 exit 0, 5분 12초입니다. 영향 쪽·정상 대조군을 재실행 중입니다.
- 표의 새 MCP Print job `eac34cfc-c6d9-4b27-a3e0-92876fe7491d`는
  63초에 성공했습니다. 결과의 engine 2020, 한컴 11.0.0.9136,
  `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`을
  확인했습니다. 원본 HWP는 수정하지 않았으며 결과 다운로드·기존 기준과
  실제 출력 대응 확인이 끝나기 전에는 새 PDF 검증 완료로 보고하지 않습니다.

### 새 Print 대응 확인과 아직 남은 미달

- 표의 새 MCP Print PDF는 31쪽이며 SHA-256은
  `e1060c018f70663501fc8eb01fbb1786881f6c23ebf111899934fb9d46309179`입니다.
  기준 PDF의 10쪽과 새 Print의 10쪽을 96dpi로 출력했고 PNG SHA-256이
  모두 `32fcdfd2c1f57a4c0a59638fbf6b63fe0aac435a5effb6caaa8882219884b67a`로
  같습니다. 기존 PDF의 해당 쪽 출력이 현재 Print와 픽셀까지 동일함을
  확인했으며 기준 파일을 바꿔 표 차이를 숨기지 않습니다. 원본과 기존
  PDF는 이미 커밋된 파일을 재사용하고 새 다운로드·원시 receipt는 output에 보존합니다.
- `3020c4798` Native 영향 11~14쪽은 96.33025%/96.04768%/97.61688%/
  95.98702%입니다. 그래프는 독립 Print와 같은 18쪽으로 돌아왔으나
  18쪽 82.58639%, 19쪽 41.16275%, 20쪽 65.80087%, 21쪽 85.89813%로
  미달이 남습니다. 18쪽의 수치 감소와 뒤 쪽의 개선을 구분해 보존하며,
  쪽 소속 하나의 개선을 시각 완료로 보고하지 않습니다.
- 새 18쪽 compare를 직접 판독했습니다. 오른쪽 owner 340 미주 30의
  내부 문단 26은 RHWP에서 한 줄인데 Print는 마지막 수식 `f'(0)=0`과
  `이다.`를 다음 줄에 놓습니다. 이 문단 뒤부터 줄 높이 차이가 누적됩니다.
  원본 수식 글꼴은 `HYhwpEQ`이며 공식 설치본과 PDF의 임베디드
  `HyhwpEQ`를 확인했습니다. 글꼴이 없다는 예외로 면제하지 않습니다.
  원본 상자·새 한컴 구성·실제 줄 구성의 폭 소비를 계속 추적합니다.

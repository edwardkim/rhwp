# #8174 v2 S08 — 연속 MoveTo의 이미지 clip 오판 보정

- 착수: 기존 v2 승인 범위에서 사용자가 `PDF_IMAGE_UNSUPPORTED: non-rectangular image
  clipping is not supported`와 실제 PDF 경로를 제공해 정상 입력 호환 결함을 재현했다.
- 시작 source: `a1a6db76be`. S07의 계상 보정·시각 보류 기록은 보존한다.
- 상태: 재현·원인 확인, 공통 보정과 실제 재검증 진행 중. 완료·PR 제출 판정이 아니다.
- 비공개 원본·전체 Form 내용·로컬 실행 로그는 ignored
  `output/pr-review/issue8174/v2-s07-20261025/image-clip-user/`에만 둔다.

## 선행 분석과 수정 계약

원 기능은 #7913 통합 `4662c069e557e928ed7968d061121fde90b1f027`, 볼록 clip 확대는
#7970 통합 `515bd72dcde0eecdeece333666b675354b7fa91d`, 후속 mask 보존은 #7980 통합
`f9d2ab17ab`다. 현재 공통 clip tracker와 기존 `issue_7966_pdf_affine_clip`의 독립 볼록
도형·변환·alpha·저장 검사를 대조했다. 기존 source에서 거절 경계와 정상 경계를 보존한다.

사용자 PDF는 15쪽이며 12쪽에 그림 한 개가 있다. 현재 CLI의 전체/12쪽 선택 가져오기에서
같은 오류가 발생한다. 독립 MuPDF는 사각 clip을 인식한다. 실제 backend image handle은
`Unsupported`이고, 정확한 tracker source로 Form 연산자를 재생하면 선행 `0 0 m` 뒤 실제
사각형의 `m l l l h W n`에서 미지원으로 바뀐다. geometry trace의 색 공간은 더미 슬롯이며
제품 image handle에서도 동일 오류를 별도 재현했다. 예산 소진 오류가 아니다.

[ISO 32000-1 표 59](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)와
[Adobe PDF 1.7 표 4.9](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf)는
앞선 path construction operator가 `m`이면 다음 `m`이 이를 대체하도록 정한다. 연결 선분이나
이전 이동의 흔적을 남기지 않는다. `re`의 등가 경로도 같은 시작 규칙을 소비해야 한다.

생산·소비 경로는 `page/Form operator → ImageClipTracker의 공통 경로 구성 → image handle
clip → visible_bounds·alpha 합성 → extract의 동일 visible bbox → lowering의 그림 프레임·crop
→ Native/WASM/저장 재열기`다. 이미지 호출부에서 clip을 제거하거나 bbox로 덮어쓰지 않는다.

보정은 연결 선분 없는 연속 이동을 마지막 시작점 하나로 처리한다. 선분·곡선이 있는 실제
다중 subpath, 오목·곡선 clip의 미지원 계약은 확대하지 않는다. 이름이나 문서 ID에 따른
예외를 추가하지 않는다. graphics state의 q/Q와 current path의 수명을 혼동하지 않는다.

## 실행·완료 조건

1. 독립 생성 PDF로 선행 이동이 없는 사각형, 연속 이동 뒤 polygon·`re`, 볼록 clip과 Form
   변환을 대조한다. 실제 두 subpath·곡선 거절을 반례로 유지한다.
2. Native/fresh WASM 전쪽 TSV·직접 판독 후 정식 관계 검사로 이미지 alpha·기하·누락/중복·
   HWP/HWPX 저장 재열기를 보호한다. 신규 렌더링 회귀는 시각 선행 조건 충족 뒤 추가한다.
3. 사용자 원본의 전체 가져오기·12쪽 그림·저장과 실제 브라우저 경로를 재실행한다. 원본 PDF는
   RHWP export이므로 원본 PDF 표시 계약과 한컴 독립 조판 일치를 구분한다.
4. 최종 source의 필수 lint·전체 회귀·Native Skia·fresh 제품 WASM, 정상 clip 대조와 비용을
   확인한다. S07의 별도 문자 폭·줄바꿈 보류와 MD05 잔여 감사는 유지한다.

최종 실행 source·명령·관측 결과·증적과 남은 조건은 아래에 이어 기록한다.

## 후속 실행에서 드러난 선택 쪽수 예산

clip 보정 source `8ce8545502`에서 12쪽 선택과 15쪽 각각의 단독 가져오기는 모두 성공했다.
전체 15쪽을 한 요청으로 열면 별도의 `PDF_RESOURCE_LIMIT: content operators`가 발생한다.
단독 요청들의 연산 합계는 1,731,414이며 각 쪽은 86,338–142,142다. 현재 1쪽에는 250,000을
배정하면서 15쪽에는 1,500,000만 배정해 배치의 쪽당 허용량이 오히려 좁아진다.

MD07의 공통 정책 보정으로 operator만 1쪽과 같은 250,000/쪽을 배정하고 standard의 절대
4,000,000·constrained의 기존 상한·사용자가 낮춘 cap을 계속 적용한다. 참조/XObject의
기존 비례식은 바꾸지 않는다. 실제 연산을 덜 계상하거나 카운터를 초기화하지 않는다.
정상 단독 요청·같은 페이지의 batch·낮춘 host cap·절대 상한의 반례와 실제 문서 전체를
Native/WASM에서 재검증한다. 앞선 clip-only producer는 중간 결과로 보존한다.

사용자의 400만 근거·800만 확대 문의는 [상한 검토 기록](task_m100_8174_operator_ceiling_review.md)에
별도로 정리했다. 숫자만 올리면 집중 입력의 거절 전 메모리 비용이 늘어나므로 공통 수집기의
사전 버퍼 제한 보강 뒤 standard 800만 확대를 권고했다. 이번 검토에서는 상한을 바꾸지 않았다.
실제 15쪽은 두 정책 모두 1,731,414회·유효 한도 3,750,000이며, 뒤따르는 HWPX 직렬화의
`invalid editable text clip metadata`는 상한과 독립적으로 보정·재검증할 대상으로 남아 있다.

## T02 — 전체 저장 실패의 실제 조건

T01 구현 source `fd4386a5cc`의 실제 IR을 typed visitor로 확인했다. clip record가 **25,078개**로
기존 4,096개 상한을 초과했다. 0 ID·소유자 중복·잘못된 geometry는 각각 0건이고 record ID도
모두 고유하다. clip path의 Move 개수는 1회 25,078개·2회 27,048개이며, 독립 MuPDF가 사각형으로
판정한 경로에 `Move, Move, Line, Line, Line, Close`가 남아 있다. private typed 관찰은 실제
`text_clip::encode`의 동일 실패와 함께 ignored `operator-assessment/t01-consumer/`에 보존한다.
일반 RenderTree JSON은 이 clip 속성을 내보내지 않아 그 JSON의 0건을 원본 IR의 부재로
해석하지 않았다. 비공개 원문의 텍스트·clip 전체는 공개 자료로 옮기지 않는다.

원 text clip 보존은 #8120의 `80968497c2`다. 생산·소비는 retained `PathExtractor`의 경로
구성 → `PathClip.operations` → `table_inference::rectangle`·`text_clip::has_nonrectangular` →
body/structured lowering → TextBox `paint_clip` → HWP/HWPX sidecar 검사다. 앞선 이미지용
유한 tracker와 별개의 retained vector도 같은 PDF 경로 구성 규칙을 소비해야 한다.

같은 사양에 따라 retained vector에서 연속 MoveTo를 대체하고 `re`의 암묵적 이동도 앞선
빈 이동을 대체하도록 보정한다. 실제 선분이 있는 다중 경로·곡선과 sidecar 상한은 유지한다.
사각 판정 호출부마다 clip을 지우거나 4,096개 상한을 올리는 처리는 하지 않는다. 실제 원본과
정상 사각/볼록/변환·진짜 비사각 clip의 표시·소유·저장을 다음 source에서 재검증한다.

## T02 Native 재검증 — source `9cabbff39d`

최적화 Native 제품을 같은 shared target에서 빌드해 별도 고정했다. 공개 이미지 5쪽·텍스트
3쪽·batch 2쪽, 비공개 전체 15쪽·이미지 선택 1쪽을 실제 CLI로 HWP와 HWPX에 각각 저장했다.
비공개 전체의 두 저장본을 다시 열어 각각 15쪽을 확인했다. 정규화된 사각 path에는 불필요한
비사각 clip sidecar가 없다. 실제 compound/curve 이미지 입력은 계속 거절하며 기존 출력 파일을
보존한다. 기존 `issue_7998_pdf_text_clip`·placement·owned_lists와
`issue_7966_pdf_affine_clip`의 실제 비사각·소유·저장 회귀 **11건이 통과**했다.

96 DPI·2022 조판으로 전쪽 TSV를 먼저 산출했다. Native 사용자 전체 15쪽의 최저 raw 일치율은
**99.13508%**, 공개 이미지 5쪽·batch 2쪽은 각각 **100%**다. 원본 PDF와 같은 영역의 이미지·
경계·본문 보존도 직접 판독했다. 이 원본 PDF 자체의 독립 raster 계약을 확인한 결과이며,
원 HWP를 같은 방식으로 편집한 독립 한컴 재조판 성공을 주장하지 않는다.

새 공개 TrueType 텍스트 대조군은 PyMuPDF가 실제 DejaVu Sans 프로그램을 `DejaVu Sans Book`으로
선언한다. 프로그램 이름 차이를 방치한 실행은 세 쪽 모두 70.16374%로 실패했다. 이 실패를 보존하고
동일 실제 글꼴의 family 이름 `DejaVu Sans`에 명시적으로 연결한 뒤 **세 쪽 모두 99.88074%**를
확인했다. 임의의 대체 글꼴·글꼴 예외·기준 완화가 아니다. 생성기를 두 번 실행해 PDF bytes가
일치함도 확인했다. 이 환경과 원본 글꼴은 fresh WASM에도 동일하게 공급한다.

증적은 ignored `output/pr-review/issue8174/v2-s08-20261025/`의
`t02-native-producer.json`·`t02-runtime/imports.json`·`t02-existing-clips.log`·
`t02-native-silhouette-summary.json`·`t02-runtime/visual/native/`와
`dejavu-family-environment.json`에 보존한다. 새 공개 렌더링 회귀는 fresh WASM 전쪽 시각 검증과
직접 판독 뒤 추가한다. 현재 결과는 Native 부분 완료이며 T03·보안 감사와 최종 통합 검증은 남아 있다.

## T02 fresh WASM·Studio와 수정 전후 대조

제품 source `9cabbff39d`의 locked WASM을 최적화하고 실제 Chrome에서 동일한 공개 대조군과
사용자 원본을 가져왔다. 공개 이미지·batch·텍스트는 각각 전쪽 100%다. 사용자 15쪽은 직접
가져오기와 HWPX 저장본 재열기의 PNG가 전쪽 동일하며 최저 raw는 **99.11167%**다. 공개
텍스트·batch는 HWP와 HWPX 두 형식의 실제 WASM 저장·재열기도 통과했다. 대표 Native/fresh WASM
review·standalone overlay에서 사각 crop·삼각 alpha·Form 기하·텍스트 순서/간격을 직접 판독했다.

동일한 글꼴 공급으로 보정 전 Native `fd4386a5cc`·WASM `8a3746a75d`의 사용자 PDF 전체
15쪽도 다시 출력했다. Native 전쪽 최저는 99.10575% → 99.13508%, WASM은
99.09012% → 99.11167%다. 각 쪽은 사실상 유지되거나 개선됐다(Native p7의 차이는
반올림 자리 0.00001%p). 글꼴 outline·굵기 차이는 보정 전에도 존재하며 source PDF와 완전히
같은 글꼴을 복원했다고 표시하지 않는다. 사각 판정이 정상화되며 일부 문단이 일반 paragraph
경로를 사용하므로 전후 모든 PNG가 동일한 것은 아니다. 이를 숨기지 않고 전체 TSV와 대표
본문·이미지·끝쪽을 대조했다.

실제 Studio에서 사용자 15쪽 가져오기·두 저장 형식 생산에 성공했다. T01의 실패·취소 상태
보존 후 정상 복구까지 같은 제품으로 확인했고, Worker도 이미지 미지원 오류 후 이전 문서를
유지했다. 비공개 원문·PNG·source 경로는 공개하지 않으며 [제출 증거](../pr/assets/issue8174/v2-s08/README.md)에
공개 PDF의 전쪽 TSV·대표 이미지와 익명 집계만 보존한다. 원 retained path 보정 전 CLI는 공개
텍스트에 불필요한 clip sidecar를 생성했지만 같은 입력의 보정 후 sidecar는 없다. 이 전후 실행도
ignored output에 보존했다. 신규 관계 회귀는 양쪽 시각 선행 조건을 통과한 공개 입력만 사용한다.

## T02 절편 종료와 다음 작업

시각 선행 조건을 확인한 뒤 `tests/cases/issue_8174_pdf_consecutive_moves.rs`를 자동 수집
suite에 등록했다. crop·alpha·Form 기하, 실제 compound/curve 거절과 정상 요청 분리,
batch/낮은 cap, 사각 텍스트의 불필요한 sidecar 부재·문자 순서·두 저장 왕복의 **5건이 통과**했다.
`rustfmt` 뒤 `node scripts/rust-test-suite-manifest.mjs --prepare`로 수집하고
`CARGO_TARGET_DIR=<repo>/target/pr-review CARGO_BUILD_JOBS=4 node scripts/run-rust-test.mjs
issue_8174_pdf_consecutive_moves -- --cargo-profile release-test`로 실제 실행했다.
생성된 suite·manifest는 커밋하지 않는다. 기존 비사각 clip 11 PASS와 함께 공개 계약을 보호한다.

T01의 구현·해당 Native/WASM/Studio 여정과 T02의 오류 보정·해당 시각/저장/회귀 검증을
완료했다. T03의 CFF 메트릭·Type3 줄바꿈, T04–T06 감사/호환과 최종 통합 게이트는 별도로 남는다.
이 절편 변경을 커밋한 뒤 승인된 순서에 따라 T03을 진행한다.

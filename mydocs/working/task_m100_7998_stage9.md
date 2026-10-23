# #7998 P4 여덟 번째 절편 — 내용 근거의 지역 축과 빈 구역

- 기준 checkpoint: `8c98a9d082af2a9822f08cff4be45d41f134f888`.
- 2026-10-19 지시: 다음 절차 진행. B18의 시각·편집 구조 통과 판정은 유지한다.
- 이번 절편은 B03/B18의 남은 지역 격자와 빈 구역을 좁혀 구현한다. 원격 반영·PR 생성과 P5 전체 게이트는 포함하지 않는다.

## 구현 전 근거·소비 경로

현재 `table_detection::grid`는 paint 축만으로 face를 먼저 만들며, 축 부족·L자 연결은
`cell_regions::select`에 도달하지 못한다. `select`도 모든 셀에 글자가 있어야 완전한
tiling으로 수용하여, 독립적인 사방 경계가 있는 빈 face까지 보류한다.

이번 입력은 정상 create/insert/merge/border API로 만든 HWPX와 독립 한컴 Print다.
내부 선을 감춘 별도 셀, 부분 연결의 L자 paint, 병합 영역과 선으로 둘러싸인 빈 셀을
대조한다. 내용의 활성 직사각 clip은 실제 text-show 사용 관계에서만 가져온다.
선으로 확인한 외곽 안에서만 지역 축을 보완하고, 빈 구역은 사방의 실제 경계가 독립적으로
뒷받침될 때만 선택한다. 글자·선이 없다는 사실로 셀을 발명하지 않는다.
누락·죽은 clip, 불완전한 tiling·겹친 범위와 경계 없는 빈 구역은 보류하고 원문을 보존한다.

`ObservedText/실제 clip → 지역 축·셀 소유 → table_detection의 최종 cell bbox/span →
Builder/source_paint_family/fixed_lines → 편집·HWP/HWPX 저장·재열기 → 최종 원점/높이`
경로를 대조한다. paint와 의미 셀 경계를 구분하고 기존 glyph/문단 결과를 재사용한다.
페이지 내부의 anchored Table 경로이며 HWP 일반 표 분할·이어받기 변경으로 주장하지 않는다.

로컬 증거: `output/pr-review/issue7998/p4-local-regions/`.
작은 경계·Native 직접 출력부터 확인하고 fresh Docker WASM·실제 Studio·양 저장으로
검증한다. 새 fixture/정식 회귀는 관련 전쪽 Native/fresh WASM 90% 이상과 확대 판독 뒤
등록한다. B18의 검증된 빌드·자료는 기준으로 보존하며 제품 변경 이후 영향 검증만 새로 실행한다.
P4 전체 완료, 보안 할당 경계 완료와 P5 최종 제출 준비는 이번 절편의 완료 주장이 아니다.

## 독립 입력과 첫 실행

정상 API 생성본 8개는 engine 2020의 PrintMethod 0·
`hancom2020_pdf_driver_one_up`·11.0.0.9136 인쇄를 모두 마쳤다.
내부 세로선 없음·외곽만 있음·L자 paint 연결·병합과 닫힌 빈 셀·한 행 표를 포함한다.
비어 있는 병합 행과 공유 변을 숨긴 빈 셀은 별도 반례다. 빈 tile끼리 선 부재를 따라
합치지 않으므로, 작성 문서가 병합 셀인 경우라도 PDF의 근거가 부족하면 보류한다.
닫힌 빈 **단일 tile의 사방 경계**와 빈 병합 구역의 연결 추측을 구분한다.

기준 B18 CLI는 같은 새 PDF를 정상 import했으나 실제 Table을 요구하는 API 검사에서
실패했다(`before-api.log`). 첫 보정에서도 paint의 거친 2행·1열 후보를 조기에 확정해
내용 clip의 지역 축 비교가 생략되는 경로를 검출했다. 실제 내용 근거가 있는 세분화는
이미 닫힌 paint 후보에서도 비교하고, 동등한 기존 결과는 그대로 유지하도록 보완한다.
외곽은 선 중심축으로 정하며 stroke join의 끝점 overhang을 외곽으로 오인하지 않는다.
원본·중간 실패 producer와 로그를 `diagnostic/`에 보존한다. 빌드 오류는 원인 FAIL로 세지 않는다.

## 새 소비 경계의 보정

첫 지역 축 검사는 정상 4개에서 실제 편집·양 저장을 통과했으나 한 행 표가 인식 단계의
최상위 1행/1열 거절 조건에서 보류됐다. 유효한 다중 셀·토폴로지·내용 소유 검증은 이미
공통으로 있으므로 차원 수만으로 결과를 덮는 조건을 제거했다. 독립 한 열 생성본도 추가해
정상 API·한컴 Print 9개와 PDF scope 변형 5개, 총 14개를 대조한다. 의미가 미확정인 더
세밀한 지역 후보를 거친 확정 후보로 되돌려 쓰지 않고 보류 상태로 남긴다. 원래 paint
후보와 같은 축·셀 해석은 재발행하지 않으며 clip의 작은 inset을 셀로 발명하지 않는다.

자간 보정 전 Native의 새 42쪽은 최저 90.426%로 90%는 넘었지만 기준 B18보다 내려갔다.
`diagnostic/native-before-spacing-units/`의 전쪽 TSV와 확대 비교를 보존했다. 관측된
`CELL 1 1`의 원문 공백이 복원 불가 프레임으로 내려가 숫자 간격이 벌어진 것이 주요 원인이다.
`observed_line_text`의 공백 폭 비율이 source font의 명목 폭만 사용하여, TJ/Tw 및 text-show
사이에 기록된 실제 공백 이동을 담지 못했다. 자간 후보를 em 단위로 바꾼 중간 실험도
실제 최종 `glyph_letter_spacing`의 glyph 폭 환산을 빠뜨린 잘못된 가정이었다. 이 실험의
42쪽 결과와 producer를 별도 진단에 보존하고 최종 보정으로 채택하지 않았다.

실제 소비는 `PDF 공백 원점 간 advance → 공백의 CharShape.ratios → ResolvedStyle →
glyph_letter_spacing → 측정/배치`다. 공백에는 보존할 윤곽이 없으므로 명목 폭 대신
**기록된 전체 advance**를 일반 CharShape 폭 비율로 옮긴다. 일반 글자의 원래 Tz·자간
계산은 유지한다. 범위 clamp·문서 조건이나 공백 추가/삭제 없이, 표현할 수 없는 범위는
그대로 보류한다. HWP 자간의 저장 범위(공식 스펙 언어별 자간)와 실제 소비의 최종 단위를
구분하고 동일 원본의 원점·간격·양 저장·전쪽 출력으로 다시 검증한다.

## 기존 대조군의 실제 소비 보정

공백의 전체 advance 보정 뒤 새 14개 × PDF/HWP/HWPX의 Native 42쪽은 모두 raw 100%였다.
그러나 기존 보고서 10–12쪽은 최상위 한 행 표를 수용하면서 6.20%p까지 하락했다. 후보의
실제 `spanIndices`를 재조회해 문자 소유가 있음을 확인했다. 장식/무내용으로 확정한 문제가
아니라 `family || preserve_paint`에 따라 다른 글줄·폭/paint 경로를 타던 소비 문제다.
모든 PDF 표는 같은 관측 글줄 메트릭을 사용하고, 이후 남은 0.07%p 하락은 legacy의
별도 균일 테두리·폭 재구성도 공통 물리 범위/paint 소유 경로로 통합해 다시 확인한다.

지역 후보가 한 tile뿐이면 기존 다중 셀 계약에 맞지 않으므로 후보를 추가하지 않는다.
보고서 5쪽에는 기존 18셀 표와 새로 해석 가능한 하단 19셀 후보가 있다. 하단의 source paint는
`unconsumed source boundaries`로 아직 발행할 수 없었고, 전역 paint 실패가 정상 상단 표까지
취소했다. 실패 target을 실제 root family에 연결해 그 family만 보류하고, 원본 내용·paint를
소비하지 않은 상태에서 남은 root를 계획한다. 재시도는 같은 1,000,000 work 예산을 공유하며
상한을 늘리지 않는다. Native 소비 보정 실행에서 상단 실제 Table 1개와 하단 보류를 확인했고
5쪽 PNG 점수는 B18과 같았다. 하단의 paint 소비 미지원은 B20에 남기며 복원 완료로 세지 않는다.

이전 producer의 검증과 Docker package는 `diagnostic/`에 보존했다. source가 바뀐 뒤에는
기존 package를 최종 fresh WASM으로 보고하지 않는다. 중단한 집중 검사와 중간 Native
검증도 별도 로그로 구분한다. 최종 제품이 고정된 뒤 영향받은 검증을 수행한다.


## 원점 누적 양자화와 최종 Native 결과

공통 물리 paint 경로만으로는 약 0.07%p 하락이 남았다. 원본 글자마다 전진폭을 독립적으로
반올림해 누적 오차가 생기는 것을 확인하고, 각 source glyph 원점·끝점과 이미 발행한 누적
전진폭의 차이를 합법적인 HWP 자간 값으로 표현한다. 다음 글자도 같은 관측 원점을 소비하므로
긴 줄에서 오차를 누적하지 않는다. 표현 불가능한 값은 기존 보류를 유지한다.

최종 Native 제품 manifest는 `source-runtime.json`의 2,448개 파일이며 SHA-256은
`80b7aff74d7a9cd6efe5bfea702244edf2e8c5222e22ed34f67589219a63b4a3`이다.
Native Skia 빌드 `native-build-origin-fit.log` exit 0. 전쪽 PDF/HWP/HWPX는
**95개 입력·285건·318쪽**, 최저 raw **97.47552%**, 90% 미달·오류·쪽수 차이 0이다.
같은 backend의 양 저장 재열기 PNG는 직접 PDF 열기와 모두 같다. 새 16개 입력의 48쪽은
모두 100%다. 이전 B18 공통 270쪽의 최대 하락은 **0.00346%p**로 기존 0.01%p 판정 범위
안이며, 보고서의 큰 하락이 났던 10–12쪽은 각각 +0.30584/+0.03987/+0.01918%p다.
`native/results.json`, `native-deltas.json`, `semantic-deltas.json`에 원문을 보존한다.

실제 Native API에서 정상 8개/보류 7개의 내용·주소/span·셀 포함과 글자/행열 편집·양 저장을
대조했다(`native-api-final.log`). 별도 root 반례도 위 18셀 Table과 37개 source label의
유일 보존·편집·양 저장을 통과했다(`ignored-test-result.log`). 빈 문자열과 실제 공백 U+0020을
구분하고, 닫힌 빈 셀의 공백은 양 저장 및 재열기에서도 그대로 유지한다.
보류 입력에 의미상 행 순서를 임의로 고정한 첫 검사 가정은 실제 source 순서와 달랐다.
제품을 바꾸지 않고 각 label의 유일 보존과 저장 전후 원문 순서를 검사하도록 바로잡았다.

새 입력은 정상 API+독립 Print 9개, graphics-state/외곽 반례 5개, 명시적 공백 대조군 1개,
두 root 실패 격리 대조군 1개다. 마지막 입력은 원본 geometry만 사용하고 모든 원문 text-show·
그림/Form draw·메타데이터·주석·marked content를 제거한 새 PDF다. C/L+1/2의 생성 label
37개만 남았음을 확인했다. 원본 자료나 연락처·font 바이너리는 등록하지 않는다.

Docker WASM·실제 Studio 및 최종 직접 판정은 계속 진행 중이다. 한컴의 이전 HWP 인쇄
작업 하나가 1,800초 제한으로 종료됐고 다른 작업도 지연되어, 최신 Native 편집 저장본
6개를 `E:\tmp\issue7998-local-region-review\edited\`에 별도 보존하고 직접 인쇄를 요청했다.
이 시점의 Native/API 통과를 편집 Print·fresh WASM·메인테이너 판정으로 대신하지 않는다.


## 직접 한컴 열기 실패와 검증 생성 경로 정정

메인테이너가 `local-l-shape-column.hwp`, `local-outer-only-row.hwp`의 손상 경고를
직접 확인했고, `merged-blank-edited.hwp`는 정상 열렸다. 세 HWPX의 인쇄 PDF를 제공했다.
파일을 만든 `cases.rs`는 행열 편집의 기준을 `from_bytes(export_hwpx_native())`로
다시 열면서, 이후 HWP 출력에 **어댑터 없는** `export_hwp_native()`를 사용했다.
이 API는 PDF 출처에서는 어댑터로 연결하지만 재열기 뒤의 HWPX 출처에는 저수준 writer다.
실제 Studio는 `wasm_api::export_hwp → export_hwp_with_adapter_snapshot →
prepare_hwp_export_snapshot`을 사용한다. 손상 둘은 압축 flag=0·PBF 1개이며, PDF 출처를
유지한 문자 편집 정상본은 flag=1·PBF 3개다. 이 차이는 이미 정식 HWP adapter 계약에 있다.

제품 serializer를 추측으로 바꾸지 않고 검증 생성기를 실제 어댑터 경로로 수정했다.
두 원래 실패 파일과 record 진단은 `diagnostic/invalid-raw-export-*`에 보존하고,
새 파일은 Windows 폴더의 `*-adapter-fixed.hwp`로 별도 준비했다. 한컴 직접 열기·Print
재확인을 요청한 상태이며 아직 성공으로 보고하지 않는다. 새 정식 검사도 실제 adapter를
사용한다. 제품 source manifest는 바뀌지 않았으므로 Native/WASM을 다시 빌드하지 않는다.


## 최종 제품·집중 검증과 직접 검토

제품 소스는 위 manifest와 같으며 fixture·회귀·문서 등록 후에도 2,448개 입력의 hash를
다시 대조했다. 새 시각 증거를 만든 뒤 제품 변경은 없으므로 이미 통과한 Native/Docker
빌드·캡처를 반복하지 않았다. Native CLI SHA-256은
`db98e4ff34275f75a0a4b7455aacbf25b488e32f0bc76400ff3ef1ff5730ea3d`,
fresh Docker WASM은 19,295,680 bytes·SHA-256
`0a7698e281d2caa09e96c7cdf3efd05d3d834aa0523f6ef1bc540635873d8088`다.
`wasm-build-final.log` exit 0이며 기존 rhwp Docker network/volume과 host 고정 target을
사용했다. Studio 7793에서 받은 실제 WASM의 bytes·hash가 같다(`studio-result.json`).
기본 Studio 7700의 JS/WASM은 시작 시 hash와 같아 영향을 주지 않았다.

| 범위 | 결과와 증거 |
|---|---|
| 전체 95개 원본 × PDF/HWP/HWPX | 각 backend **285건·318쪽**. 최저 Native **97.47552%**, fresh WASM **97.65094%**. 90% 미달·쪽수 차이·누락·실행 오류 0. 저장 재열기 PNG는 같은 backend의 직접 PDF 열기와 모두 같다. `native/results.json`, `wasm/results.json`. |
| 새 지역 축·빈 셀 입력 16개 | 각 backend **48쪽 전부 raw 100%**. 정상 API+Print 9개와 명시적 PDF 연산 대조군을 구분했다. `inputs.json`, `pdf/issue7998/local-regions.md`. |
| 이전 B18과 같은 쪽 비교 | Native 최대 하락 **0.00346%p**. WASM은 견적서 1쪽 **99.75780 → 99.68835%**, 최대 하락 **0.06945%p**. 다른 WASM 하락은 body-picture의 0.00002%p다. 견적서 전후 PNG와 수치를 검토 화면에 별도로 표시했다. 이 차이를 없는 것으로 보고하거나 90% 통과로 직접 판독을 대신하지 않는다. `*-final-deltas.json`, `quotation-before-after.png`. 최신 devel 제출 gate는 P5다. |
| 실제 Native/Studio API | 정상 8개·보류 7개 및 두 root 반례를 검증했다. 주소/span·공통 글줄 소비·내용 순서/보존·셀 포함, 문자/행열 삽입·삭제·양 저장·재열기 통과. 실제 공백 U+0020도 양 저장에서 보존했다. `native-api-final.log`, `studio-result.json`. |
| 독립 편집 Print | 메인테이너 제공 HWPX Print 3개 × 두 형식과 보정 HWP Print 2개로 각 backend **8쪽**. 최저 Native **99.73875%**, fresh WASM **99.03409%**. `*-extra/results.json`, `exact-print/results.json`, `manual-print-inputs.json`. |
| 한컴 상호운용 | 메인테이너가 어댑터 경로의 두 `*-adapter-fixed.hwp` 모두 **정상 열림**을 확인하고 PDF를 제공했다. 이전 `merged-blank-edited.hwp`도 정상 열림. 저수준 writer로 잘못 생성한 두 파일의 손상과 실제 저장 경로의 성공을 분리한다. |
| 정식 집중 회귀 | 기존 9개 case **41건**, B18 수정 기대 **5건**, 새 `issue_7998_pdf_local_regions` **5건**, 합계 **51건 통과**. `focused-final-results.json`·각 log. 첫 실행의 일부 0건은 suite 생성이 오래된 실행 누락이며 통과로 세지 않았다. `rust-test-suite-manifest.mjs --prepare` 뒤 해당 case만 다시 실행했다. 정상 통과 case는 재실행하지 않았다. |
| 신규 회귀의 원인 FAIL → PASS | 기준 B18의 import 성공 후 지역 셀 검사 실패와 소비 보정 전 producer의 정상 root 취소 실패를 같은 실제 Table 검사로 확인했다. 현재 정상 API 및 정식 case 통과. `before-after-regression.json`·두 before log. 빌드 실패·실행 누락은 원인 FAIL로 세지 않았다. |
| 형식·입력 재현 | `cargo fmt --all -- --check`, `git diff --check` 통과. 공개 scope 생성기의 전 사례 실행도 별도 출력 폴더에서 확인했다. 제품 전체 회귀·lint·Worker/package 제출 gate는 P5다. |

새 검사·fixture는 관련 전쪽 Native/fresh WASM 90% 이상과 에이전트의 동일 배율 직접
확대 판독 뒤 등록했다. 사방 선이 확인되는 기존 B18 `unwitnessed-cell`은 새 자격에서
실제 3셀을 가지므로 보류 기대를 양 저장·내용·주소 검사로 바꿨다. 애매한 빈 병합/숨긴 변
반례는 그대로 유지한다. 절대 px 핀·SVG hash·baseline 완화는 추가하지 않았다.

직접 판독은 실제 선의 존재/부재, 글자 원점·간격, 빈 셀·한 행/한 열, L자 paint, 두 root의
범위·내용 보존과 편집 후 경계를 대조했다. 서로 다른 PDF rasterizer의 가는 선 농도 차이는
남는다. 열 삽입본은 원래 좁은 PDF 용지/clip 범위를 벗어나는 내용이 같은 편집의 한컴
Print와 동일하게 잘리며 IR에는 보존된다. 용지 자동 확대나 원래 viewport clip을 바꾸는
편집 계약까지 완료한 주장이 아니다. `direct-review.json`과 대표 비교 PNG·overlay를 참조한다.

병렬 시각 실행은 다른 입력/출력 폴더만 사용하고 공유 빌드를 중복하지 않았다. 취합기가
새로 복제한 대용량 renderer scratch는 파일 목록·크기 및 TSV 동일성을 확인한 뒤 원래
검증 shard를 참조하는 링크로 바꿨다(`aggregate-deduplication.json`). 원본 shard·TSV·PNG·
저장본과 공유 target은 보존했으며, 이 정리는 기존 검증 증거를 삭제한 작업이 아니다.

## 메인테이너 검토 경로·남은 범위

`E:\tmp\issue7998-local-region-review\index.html`에 원본 PDF·실제 WASM 저장본·
실제 셀 주소/span/본문·Native/fresh WASM TSV와 대표 비교/overlay PNG를 상대 경로로
연결하고 모든 링크의 존재를 확인했다. 하단에 견적서 전후 하락과 실제 HWP 어댑터
보정·한컴 Print를 별도 연결했다. Studio 7793 새 탭은 다음과 같다.

- `?review=issue7998-local-local-outer-only`: 실제 2×2·4셀.
- `?review=issue7998-local-local-l-shape`: L자 paint여도 별도 4셀.
- `?review=issue7998-local-merged-blank`: 병합과 닫힌 빈 셀의 3셀.

실제 로드·1쪽·셀 주소를 `opened-review-tabs.json`에 기록했다.
**2026-10-19 메인테이너가 이번 비교 화면과 새 Studio 입력에 “시각 판정 통과입니다.”로
판정했다.** 이 판정은 견적서 1쪽의 WASM 하락 0.06945%p를 별도 표시한 같은 제품·증거에
대해 받은 직접 시각 판정이다. 실제 편집 구조·양 저장 보존은 위 Native/Studio API 검증을
함께 따른다. 앞서 받은 HWP 정상 열기 확인과 이번 시각 판정을 구분하며 B18의 기존 판정도 유지한다.

B03/B18의 이번 좁은 자격을 넘어서는 일반 무외곽/부분 지역, 빈 병합 구역의 의미 결정,
B20 하단 필수 paint 소비, 비표준 border의 일반 스타일·행열 상속, 기존 본문/clip 메트릭과
P5 최신 devel 통합·전체 회귀/lint/공개 패키지·전쪽 최종 gate는 남는다. 이번 결과를 P4
전체 완료·새 PR 준비 완료·원래 작성 객체의 유일한 복원·전체 보안 할당 경계 완료로
승격하지 않는다. 원격 push·댓글·PR 상태 변경은 수행하지 않았다.

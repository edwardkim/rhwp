# #8174 T03 — CFF 전진폭과 Type3 셀 줄바꿈

- 착수 source: `c089e778f2`(제품 code `9cabbff39d`). T01·T02 결과를 먼저 커밋했다.
- 승인: 2026-10-25 사용자의 남은 todo 순차 진행 지시. 기존 PDF·기준·실패 캡처를 보존한다.
- 상태: 원인 경계 확인·공통 보정 중. 시각 완료나 PR 제출 준비 완료가 아니다.

## CFF의 선행 원인과 보정 계약

S07의 같은 독립 `pdf/issue8174/fonts/cff-cid.pdf`는 실제 Nimbus Roman Regular 프로그램을
포함하지만 공통 메트릭 DB에는 이 face가 없다. 동일 프로그램의 올바른 이름 공급은 paint를
고쳐도 메트릭 DB에 없는 Greek·악센트·dash의 자연 전진을 제공하지 못한다. 소스의 em dash는
1em인데 추정 폭은 0.5em이어서, Tz=100%를 보존하면 합법 HWP 자간 ±50%로 표현할 수 없다.
따라서 문단 변환이 실패해 원래 span 전체를 positional fallback으로 남긴다. 이 원인은 S07의
실제 font trace·독립 PDF glyph 원점과 FontFile의 cmap/hmtx를 대조해 확인했다.

기존 Canvas2D supplemental API는 해당 backend와 세대의 측정 세션이며 PDF 가져오기 시점의
공통 source style 해소에 등록된 실제 프로그램이 아니다. 이를 다른 backend의 증거로 쓰지 않는다.
이미 사용하는 독립 프로그램 메트릭 모듈 경로로 Nimbus Roman의 실제 네 정체 cmap·hmtx를
추출해 공통 DB에 추가한다. 본문·기호마다 추정 비율을 보정하거나 Times 윤곽/폭으로 대체하지 않는다.
추출하는 것은 정수 전진폭과 출처이며 글꼴 bytes·outline을 추가 배포하지 않는다.

생산·소비는 `독립 SFNT cmap/hmtx → FontMetric::get_width → PDF SourceStyles/GlyphWidths의
공통 측정 → 합법 CHARSHAPE 자간·장평 → 저장/재열기의 같은 측정 → 실제 paint 원점`이다.
기존 등록 순서·다른 family 메트릭은 유지하고 새 표는 뒤에 연결한다. family·정체·PostScript
이름과 입력 파일 SHA-256을 함께 기록하며 설치 여부나 paint의 실제 공급까지 단정하지 않는다.
동일 실제 프로그램·환경을 공급한 Native/fresh WASM 전쪽 TSV와 확대 판독으로 마지막 소비를
검증한다. 새 렌더링 회귀는 양쪽 시각 선행 조건을 충족한 뒤 추가한다.

## Type3의 조사 범위

S07의 같은 12쪽 입력과 독립 기준을 사용해 11쪽 표의 `동작시험` 소유 셀·줄 구성·메트릭·가용
너비와 최종 배치를 추적한다. CFF 보정의 성공을 Type3 검증으로 대신하지 않는다. 원래 PDF
글꼴·원점·본문 순서와 다음 내용도 대조하며 문서 ID나 우연한 px 조건을 추가하지 않는다.

## Type3의 실제 소유·측정·배치 추적

원 PDF의 같은 입력 HWP `samples/issue5875/nested_table_text_caption.hwp`도 기존 CLI로
임시 HWPX에 변환해 구조를 읽었다. `동작시험`은 실제 별도 문단이다. PDF에는 11쪽의 같은
행에서 4개 글자가 같은 baseline에 있다. 반면 PDF 가져오기 IR은 앞의 가운데 정렬 줄들과
17글자 한 문단으로 묶고 source LineSeg 컷 0·5·10·13을 저장했다. 실제 render tree는
`의무부하동`과 `작시험`으로 배치했다. S07의 '추가 줄바꿈' 서술을 이 관측으로 구체화한다.
원인 확인 없이 마지막 글자의 좌표만 이동하지 않는다.

원 공통 문단 복원과 wrap witness는 #8120 통합 `80968497c2`에 있다. 본문은
`wrap_width::fit`을 호출하지만 셀 confirmed 분기는 이 검증 없이 `paragraph`만 추가했다.
또한 본문도 fit 실패를 기록만 하고 그대로 paragraph를 소비했다. 저장 LineSeg 존재와 실제
재배치의 일치를 혼동한 경계다. 같은 common reflow가 source 컷을 재현할 수 있는 문단만
`verified_paragraph`에서 발행하고 본문·셀이 이를 함께 사용하도록 보정했다.

불가능한 결합은 style·진단 append를 되돌린 뒤 본문은 기존 원본 positional owner, 셀은
기존 unresolved source-row owner 경로로 보존한다. 이 경로는 source의 실제 줄 원점·내용 순서를
사용하며 authored 문단 경계를 복원했다고 주장하지 않는다. 가짜 hard break·문서명 분기·
너비 clamp·paint 좌표 덮어쓰기를 추가하지 않는다. 고정 source rows가 아닌 기존 경로는 유지한다.
측정의 fit·예약 flow 높이·실제 배치는 같은 발행 문단을 소비한다. source를 따로 재추측하는
backend별 보정은 없다. 변경 후 같은 원본의 전쪽 출력과 normal soft-wrap·저장/편집을 검증한다.

## 현재 실행 범위

Nimbus 생성기를 같은 실제 네 정체에서 두 번 실행해 Rust 데이터·source JSON의 바이트 일치를
확인했다. 독립 CFF PDF 안의 실제 프로그램 전체 Unicode cmap/hmtx와 공통 메트릭을 대조한
Native 단위 검사 1건이 통과했다(854글자 이상·세 이름·space/en dash/em dash 포함).
이 결과는 layout 시각 통과가 아니다. 본문·셀의 공통 flow 보정 뒤 `cargo check --locked`도
통과했다. 다음 실행은 같은 고정 source의 최적화 Native·fresh WASM과 전쪽 시각/정상 회귀다.

## T03 절편 검증 완료 — 2026-10-25

제품 source는 `1e7282ad64fae37bbf9684c1e77008e132473075`다. 이후 새 회귀·기록만 추가했고
제품 코드는 바꾸지 않았다. [공개 증거](../pr/assets/issue8174/v2-s09/README.md)의 manifest가
제품 hash, 원문 TSV와 대표 review·overlay PNG를 고정한다. 실제 명령·실행 로그·개별 trace는
ignored `output/pr-review/issue8174/v2-s09-20261025/`에 보존했다.

| 독립 입력·대조군 | Native 전체 쪽 최저 raw | fresh WASM 전체 쪽 최저 raw | 내용·시각 판정 |
| --- | --- | --- | --- |
| CFF 1쪽 | 100% | 100% | Greek·악센트·en/em dash의 실제 전진폭·줄 원점과 세 원본 행을 보존 |
| Type3 12쪽 | 98.34664% | 98.79246% | 11쪽 셀의 네 줄과 앞뒤 행·그림·뒤 내용을 보존 |
| 본문·셀 11개·11쪽 | 모두 100% | 모두 100% | soft wrap·가운데 정렬·TJ·baseline·열·표·중첩·그림 혼재 보존 |
| T02 공개 clip 3개·10쪽 | 이미지·batch 100%, text 99.88074% | 모두 100% | 앞 절편의 내용·clip 출력 보존 |
| maintainer 문서 15쪽 | 99.13508% | 99.11167% | T02와 동일, 원본·경로·PNG는 공개하지 않음 |

각 환경 17개·49쪽이며 누락·90% 미달은 없다. CFF는 실제 Nimbus Roman 프로그램을 공급하고
동일 family 환경을 지정했다. TTF clip은 S08의 실제 DejaVu family 환경을 그대로 사용했다.
본문·셀 정상 대조군에는 실제 Arial 정체 네 개도 공급했다. 폰트 설치 여부를 보장하는 테스트나
내장 PDF 폰트의 완전 복구를 주장하지 않는다.

Type3 11쪽 raw는 Native `98.94580 → 99.97900`, WASM `98.93836 → 99.98717`이다.
다른 10쪽은 같은 backend의 변경 전후 픽셀이 동일했다. 1쪽은 일부 픽셀이 바뀌었지만 Native
raw는 99.13578%로 같고 WASM은 `99.19134 → 99.19147`이다. 1쪽의 본문·표 경계·그림·캡션,
3·7·8쪽의 중첩 표와 후속 내용, 11쪽 문제 셀·이웃 행·그림 경계를 독립 PDF와 review/overlay로
직접 판독했다. CFF의 Greek·dash·수치·괄호 간격도 직접 확인했다. 기존 글꼴 윤곽과 raster AA
차이는 남으며 좌표 clamp·텍스트 누락·추가 줄은 검출하지 않았다.

fresh WASM은 `wasm-pack-locked.sh --target web --out-dir pkg`의 release 빌드와 wasm-opt를
완료했고 Studio/public 복사본의 hash도 일치한다. 실제 Chromium에서 변경 전·후 각각
CFF와 Type3를 import → HWP/HWPX 저장 → 동기 재열기했다. 후보의 세 경로 모두 CFF의
검증된 메트릭 출처, Type3의 12쪽·동일 셀·네 줄을 확인했다. 변경 전의 추정 메트릭·잘못된
셀 행도 같은 harness에서 재현했다. 실제 Studio의 최종 통합 검증은 T08에서 새 source로 한다.

새 정식 `issue_8174_pdf_font_fidelity` 2건과 기존 clip·body soft wrap·body paragraph·table
composition·border editing의 Native Skia 회귀를 합쳐 **23 PASS**다. 새 검사는 전체/선택 쪽,
양쪽 저장 adapter의 내용·쪽·셀 소유·baseline 행을 검사하며 절대 px 핀을 추가하지 않았다.
처음 CFF 검사에서 `metricEntry`를 객체로 잘못 기대한 실패가 있었다. 실제 API는 숫자 ID이고
trace의 검증된 source가 정상이므로 숫자 타입으로 바로잡았다. 제품 보정으로 보고하지 않는다.
한 진단 호출은 준비된 suite와 재계산한 target이 달라 0 tests였다. 실제 생성 suite를 지정한
재실행과 위 23 PASS 로그를 근거로 쓰며 0 tests를 통과로 세지 않는다.

### 비용과 남은 통합 범위

동일 호스트·동일 최적화 Native 제품을 순차 실행해 입력마다 전후 3회 측정했다.
child median은 CFF `16 → 16 ms`, Type3 `338 → 368 ms`(약 8.9%, 30 ms 증가), maintainer
문서 `2133 → 2150 ms`(약 0.8%)다. 관측 최대 RSS는 각각 `26.83 → 26.69 MB`,
`46.47 → 47.82 MB`, `123.42 → 124.66 MB`였다. Type3의 source row 재현 가능성 검증에
비용이 추가됐음을 숨기지 않는다. 표본 3회 관측이며 일반 상한이나 peak heap 증명이 아니다.
WASM 비용과 전체 source 통합 회귀·시각·성능은 T07–T08에서 확정한다.

T03의 원인·공통 보정·내용/시각 선행 조건·집중 회귀는 완료했다. 다음은 승인된 T04의 잔여
font/cache deep copy와 frontend/Core/직렬화 예산이다. 전체 #8174 또는 PR 준비 완료는 아니다.

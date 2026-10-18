# #7998 실제 표 IR와 셀 소유 연결

- 지시: 화면의 높은 피델리티와 pdf2ir의 표 구조 재현을 구분하고 실제 표 복원을 진행한다.
- 승인 범위: 기존 [구현계획](../plans/task_m100_7998_impl.md) 4절의 제품 소비·내용 소유와
  B02/B03이다. 같은 범위의 구현 선택에 대해 재승인을 요청하지 않는다.
- 이전 source: `3c6fddb0b`, 기록 head `65a4aaa7c`. 기존 Native/WASM·전체 검증은 보존한다.
  새 구현의 통과 증거로 재사용하지 않는다. 원격 push/PR 등록은 계속 보류한다.

## 먼저 해결할 경계

실물 저장본에서 `<tbl>`는 두 입력 모두 0개였다. 5쪽 위쪽 표(6×3)와 견적서 본문(10×6)은
닫힌 셀 경계와 내용 소속을 검증할 수 있지만, periodic carrier를 paint 보존 불가로 분류해
recognition 전에 차단한다. 이 때문에 spanIndices도 비어 있다. 짧은 경계의 출처를 선택하는
`boundary_contributor`에도 길이 0.5pt 필터가 남아 점선 조각이 최종 소유 장부에서 빠졌다.

1·3쪽과 5쪽 아래의 무paint 병합 관계는 다른 경계다. 선 부재를 병합 확정으로 바꾸어 목표
개수를 맞추지 않는다. 닫힌 격자를 실제 표로 만드는 연결을 먼저 검증하고 판정 근거를 확장한다.

## 소비 계약과 반례

- 생산: 검출기의 완전한 격자·source 경계 참조 → recognition의 유일한 셀 내용 소유 →
  TableBuilder의 실제 Table/Cell 및 셀 문단 → 공통 renderer와 HWP/HWPX 저장·재열기.
- 임의 점선 패턴과 0.1mm 미만 선폭은 HWP 표 테두리의 고정 스타일/굵기 index로 정확히
  표현되지 않는다. 임의로 실선·표준 점선으로 치환하지 않고, 검증된 셀의 원본 paint를
  셀에 앵커된 공통 drawing으로 보존한다. 이는 셀 바깥의 선과 빈 표를 겹치는 우회가 아니다.
  본문은 실제 셀 문단이고, 선분/배경도 표의 셀 소속이며 페이지의 중복 paint는 제거한다.
- 경계 paint는 셀 frame의 물리 영역으로 분할한다. 원본 좌표·색·alpha·두께를 바꾸지 않는다.
  새 fragment ID는 유일해야 하며 소유하지 못하는 compound paint·외부 겹침은 보류한다.
- 셀 paint는 흐름을 전진시키지 않는다. 문단 앵커·좌우 여백·셀 padding과 상대 좌표를
  함께 계산하고 `table_layout.rs`의 `para_y_before_compose` 및
  `table_cell_content.rs::layout_cell_shape_with_parent_path`의 최종 원점 소비를 대조한다.
- 독립 2×2 점선 PDF와 정상 실선 대조군에서 수정 전 실제 의미 표 0/1개를 보존한다.
  새 경로에서는 실제 셀 본문 A/B/C/D의 단일 소유와 저장 후 Table 유지, 물리 paint 보존을
  검증한다. 시각 선행 조건 전에는 새 렌더링 회귀/golden을 정식 등록하지 않는다.

상태는 구현·집중 검증 중이다. 전쪽 Native/fresh Docker WASM·저장/편집·실물 셀 구조가
검증되기 전에는 이번 단계나 #7998을 완료했다고 보고하지 않는다.

## 첫 Native 실행에서 드러난 추가 경계

`semantic-stage3/`에 pass1 바이너리·source hash·저장본을 보존했다. 6·8쪽에는 실제 표가
각각 1개 생성됐으나 목표 5쪽은 paint 분할의 틈/중복 검사에서 보류됐다. 각 셀의 시작과
폭을 따로 반올림하면 같은 경계가 1 HWPUNIT(0.01pt) 달라질 수 있었다. 같은 축을 한 번
변환한 결과로 셀 크기와 paint 분할을 함께 소비하도록 수정한다.

견적서는 확정된 지역 격자 밖까지 이어지는 긴 선 때문에 기존 paint layer 검사가 정확히
보류했다. 이 검사를 면제하지 않는다. source paint를 페이지 전체의 선택된 셀 소유와
잔여 페이지 영역으로 한 번 분할해 원본 면적/길이를 보존하고 중복을 제거해야 한다.
독립 점선 대조군의 compound 외곽은 기존 경로가 object-local PNG로 보존한 source paint다.
그림 전체를 페이지로 flatten하지 않고 해당 crop을 셀별로 분할하며 같은 bitmap을 참조한다.

## 실행 중 보류와 수정 근거

pass2는 실제 저장본에 operators 7개 표, 견적서의 닫힌 내부 구역 1개 표와 셀 본문을
생성했다. 그러나 전쪽 Native에서 5–8쪽은 80.26/70.89/28.91/68.36%, 독립 점선
대조군은 29.84%였다. 실제 새 회귀이며 통과나 완료로 보고하지 않는다. 원본과 실패
바이너리·source hash·PNG·TSV는 외부 `semantic-stage3/pass2/`에 보존했다.

다음은 pass3에서 검증한 가설이다. 최종 원인 확정과 구분한다.

- 첫 글줄까지의 위치 차를 셀 padding으로 추정하면, 셀 크기의 paint와 padding이
  측정에서 중복 점유되어 행이 늘어났다. 위치 간격을 padding의 증거로 사용하지 않는다.
  원본 줄의 stored lead와 문단 간격으로 표현하며 실제 셀 크기와 공유 축은 유지한다.
- 셀 그림의 Para 수평 원점은 도형의 Para 원점과 달리 문단 margin을 소비하지 않았다.
  경계 paint는 본문 들여쓰기와 독립인 셀 Column frame에 앵커한다. `anchor`에서
  생산한 offset은 shape의 inner cell origin과 picture의 Column origin이 소비한다.
- Shape의 BehindText가 셀 TextLine 뒤에 방출됐고 마킹조차 없어 불투명 배경이 글을
  덮었다. 셀 조립 후 같은 셀의 배경 장식 → BehindText 개체 → 본문 → foreground
  순서로 정리한다. BehindText는 페이지 전체가 아닌 셀 본문에 대한 관계이므로
  정리한 개체는 표의 replay plane을 상속한다. 공통 renderer의 실제 경로를 수정하며
  PDF 객체만 따로 숨기거나 원본 배경색을 제거하지 않는다.

pass3에서 padding을 제거해도 행 높이 증가가 유지됐다. 따라서 padding 자체를 원인으로
확정하지 않는다. 저장본을 독립적으로 변형한 진단에서도 저장 줄 시작이 누적 예약에
잘못 더해진 것을 확인했다. `semantic-stage3/pass3/`에 실패 결과를 보존한다.

현재 pass4는 첫 내용까지의 거리를 원문 작성자의 padding으로 주장하지 않고, PDF의
내용 frame을 표현하는 유도 inset으로 사용한다. 배경은 이 frame 위로 올라가는 음수
변위를 갖는다. 공통 `cell_wrap_vertical_offset_hu`는 배경의 음수를 보존하나,
height_measurer와 table_layout의 소비 지점이 다시 `.max(0)`으로 지워 전체 개체 높이와
inset을 중복 계상했다. 두 소비 지점의 불일치를 제거해 같은 signed offset과 물리
끝점을 사용한다. 임의 상수·문서 ID·그림 숨김을 추가하지 않는다.

pass3의 실제 셀 편집과 HWP/HWPX 저장·재열기는 4개 입력에서 통과했다. 문자 수량도
operators 5,324자, 견적서 267자, 대조군 4/4자로 수정 전후 누락·추가가 없다.
이것은 위치·표 전체 구조의 정답 증거를 대신하지 않는다. pass4의 Native 방향과 기존
셀 foreground 계약을 먼저 확인한 뒤 fresh Docker WASM과 편집·저장 소비를 검증한다.

## 현재 구조 연결 결과 — pass5

제품 source hash는 외부 `semantic-stage3/pass5/source.json`에 고정했다. 최종 수정 후
Native와 표준 Docker WASM을 새로 빌드했으며 이후 변경은 정식 검사와 기록뿐이다.
WASM SHA-256: `ede5de68029b7502660757b1ee567726f8dc3e95f51eacd4f4e67135b6451f07`.

| 입력 | 실제 표/셀 | Native 최저 / fresh WASM 최저 | 구조·검증 범위 |
|---|---|---|---|
| 원래 보고서 12쪽 | 7개 / 128셀 | 91.04017 / 90.62487% | 5쪽 6×3, 6쪽 2개, 7쪽 1개, 8쪽 3개. 무paint 병합과 사진 표는 아직 보류 |
| 견적서 1쪽 | 내부 격자 1개 / 60셀 | 97.62909 / 97.46596% | 닫힌 내부 구역만. 공급자 영역과 원래 Excel 전체 셀 구조 복원의 완료가 아님 |
| 독립 native 점선 2×2 | 1개 / 4셀 | 99.94962 / 98.88438% | 0.4pt stroke와 0.6pt 공백, A/B/C/D 실제 셀 본문. 수정 전 producer는 실제 표 0개 |
| 정상 실선 2×2 | 1개 / 4셀 | 100 / 99.53052% | 기존 경로·단일 내용 소유 유지 |
| compound 외곽 점선 2×2 | 0개, 보류 | 99.93820 / 98.63014% | picture 경계의 셀별 crop에서 선 손상 확인. 원본 paint를 유지하며 확정 승격하지 않음 |

모든 입력의 전체 쪽수는 기준과 같다. 숫자 통과와 직접 판독 판정은 구분한다.
`pass5/native-visual-results.json`, `wasm-visual-results.json`, 각 `visual/*/*/silhouette.tsv`
원문을 참조한다. 이전 pass2–4의 실제 실패 자료는 삭제하지 않았다.

- 실제 저장본을 CLI `export-tables`로 다시 조회했다. 본문은 Table/Cell의 문단이며,
  같은 글자가 외부 TextBox로 남아 있지 않다. 원본의 비표준 세선·점선은 셀 소속 native
  drawing으로 보존한다. 표 경계의 모든 스타일 필드를 편집 가능한 border로 복원한 것은 아니다.
- 네 지원 입력의 `edit set-cell`로 첫 표 첫 셀만 AA로 바꿨고 다른 셀·병합 주소는
  보존됐다. HWP/HWPX 저장·재열기에서 Table/Cell과 본문이 유지됐다.
- Studio 7793이 실제 새 WASM을 제공하는지 HTTP 응답 hash로 확인했다. 실제 PDF를 열어
  Table/Cell 크기 질의, 셀 경계 조회, `insertTextInCell` 및 HWPX 저장을 실행했다.
  2×2·보고서는 저장 후 삽입 문자열과 다른 셀 본문이 일치했다. 견적서는 ASCII 접두어
  `IR:` 뒤에 공백 1개가 추가돼 정확한 문자열 검증은 미충족이다. 비공백 문자나 표 소속이
  보존됐다는 사실로 이를 통과 처리하지 않는다. `studio-semantic.json`, `wasm-edit-reopen.json`.
- 기존 #7998/6121/6192/6494 집중 10건은 통과했다. 새 픽셀 고정·fixture 추가는 자격을
  갖춘 독립 native 점선 대조군 1쪽으로 제한한다. 원본의 텍스트 잘림이 남는 페이지에는
  새 golden/절대 좌표 검사를 추가하지 않는다. 기존 전체 검증을 새 코드의 완료 근거로
  재사용하거나 반복 실행하지 않았다.

## 남은 실제 결함과 다음 구현 경계

5쪽 표에서 탭을 포함한 줄의 끝 글자가 셀 클립에 가려지는 차이가 직접 판독에서 남는다.
위치형 TextBox에서는 개별 span이 폭 보정을 소비하지만, 실제 셀 문단으로 합친 경로는
여러 span과 tab의 font advance를 같은 방식으로 보존하지 못한다. 원래 glyph·본문은
저장돼 있으므로 문자 존재 검사만으로 해결됐다고 판단하지 않는다. 공통 문단의 줄/탭
구성과 측정·paint의 동일한 advance 소비를 해결해야 한다.

진단에서 셀 SQUEEZE, source bbox를 재설정한 오른쪽 margin/segment 폭, tab 원점 변경,
수동 multi-segment 줄은 해결을 입증하지 못했다. 이 실험들은 `pass4/`에 보존했으며
제품 코드에 넣지 않았다. 새로운 조건·정렬·글꼴 비율로 목표 화면을 맞추지 않는다.

compound 경계 picture 보류, 무paint 병합/지역 후보 선택, 견적서의 ASCII 삽입 후 공백,
셀 border 스타일/행·열 크기 편집의 보존도 남아 있다. 같은 #7998 안의 미완료 경계이며
별도 이슈로 이관하거나 예외 수용을 요청한 것이 아니다. 현재 source는 구조 연결 검증
중인 WIP다. PR 생성·merge·이슈 종료와 전체 구현 완료를 계속 보류한다.

메인테이너 검토 파일은 `E:\tmp\issue7998-semantic-table-review\index.html`에 두었다.
실제 표가 있는 HWP/HWPX·셀 편집본과 CLI로 조회한 셀 본문을 함께 제공한다. Studio는
`http://localhost:7793/?review=issue7998-semantic-tables&lang=ko`다. 원래 7700과 사용자
피드백 파일은 보존했다. Native/fresh WASM source 및 원격 작업 범위는 변경하지 않았다.

### 추가한 정식 경계 검사

`pdf/issue7998/periodic-native-grid.pdf`(8,077 bytes,
SHA-256 `c0af24b25f739228438ca225ef1e17831fadfe472d96099b381723161372051f`)는
독립적으로 작성한 native 괘선 대조군이다. 기존 compound 원본과 그 실패는 유지하며
같은 그림이라고 바꿔 보고하지 않는다. 외곽을 네 개의 native stroke로 구성한 절차와
원본 PDF byte를 외부 진단에 보존했다.

`issue_7998_pdf_table_inference.rs`의 추가 검사는 실제 표 생성, 원본 A/B/C/D의 유일한
셀 문단 소속, 외부 중복 TextBox/괘선 제거, 양 저장 형식의 보존을 검사한다. 독립 PDF의
160pt 외곽을 96dpi로 변환한 높이/폭에 ±2px 조건을 두고 배경 때문에 행이 커지는
회귀를 보호하며, 글줄의 셀 내부 포함 관계도 유지한다. 이 1쪽은 양쪽 raw 최저가
98% 이상이고 직접 판독을 마친 범위다. 다른 원본의 미완료 영역에는 확장하지 않는다.

이전 producer의 CLI import 결과는 같은 입력에 실제 표 0개, 현재는 1개다. 이전
source에서 새 Rust 함수 자체를 재컴파일하지는 않았으며, 명령 수준의 동일 제품 경로
FAIL/PASS와 구분한다. `--prepare` 후 자동 suite 경로의 #7998 검사 6건이 통과했다
(`pass5/new-boundary-test-prepared.*`). 원격 변경과 전체 검증 완료 주장은 없다.

## 2026-10-18 문단 복원 범위 확장과 현재 매핑 감사

메인테이너가 문자 덩어리의 글줄·문단 인식과 IR 매핑 보완을 이번 범위에 추가했다.
[구현계획 4.1절](../plans/task_m100_7998_impl.md#41-이번-범위의-글줄문단-복원과-실제-ir-연결)에
참조 알고리즘·실제 호출 경로·구현 순서·반례·수용 조건을 반영했다. B10–B13은 이번 범위의
일감이며, 현재 표 인식 완료 후 나중에 처리할 별도 제안이 아니다.

현재 코드에서 T0와 recognition은 각각 glyph와 span을 별도로 묶는다. 본문은 인식된
`bodyRegions`를 소비하지 않고 source span마다 글상자를 만들며, 셀 `fixed_lines`는 제안된
문단을 줄별 Paragraph로 다시 쪼갠다. 글줄 사이 newline과 span 간 탭 생성도 실제 문단/탭
근거와 구분이 부족하다. 공통 문자/줄 결과와 지역화부터 정비하고 문단 판정·실제 소비를 연결한다.

pass5의 기존 저장본을 XML로 읽어 다음 상태를 확인했다. 새 import나 빌드를 실행한 결과가
아니며, 이 개수를 원본 작성 문단 수의 정답으로 사용하지 않는다. 셀의 빈 host 문단도 포함한다.

| 저장본 | 실제 표 | 셀 | 셀 직접 소속 문단 | LineSeg가 둘 이상인 문단 |
|---|---:|---:|---:|---:|
| 보고서 전쪽 | 7 | 128 | 210 | 0 |
| 견적서 내부 격자 | 1 | 60 | 60 | 0 |

감사 스크립트·입력 SHA·집계는 저장소 밖 기존 증적 폴더의
`semantic-stage3/paragraph-scope-20261018/`에 보존한다. 글줄/문단 알고리즘 검토는
pdfminer.six와 pdfplumber의 실제 코드, PdfPig의 XY Cut·Docstrum·읽기 순서 설명을 참조했다.
이는 설계 근거이며 신규 구현이나 실물의 정확성 검증을 대신하지 않는다.

이 범위 반영에서는 제품 코드·WASM을 바꾸지 않았다. pass5의 시각 캡처와 실제 표/편집
검증은 그 source의 결과로 보존한다. 새 문단 복원에 대한 검사·시각 판정은 아직 미실행이며,
다음 코드 변경이 영향을 주는 경계부터 새로 검증한다. 원격 push·PR 생성·이슈 상태 변경은 없다.

메인테이너의 후속 지시로 [계획의 우선순위](../plans/task_m100_7998_impl.md#작업의-우선순위-정확한-pdf--ir-변환)를
명시했다. PDF→IR의 내용·구조·배치·편집/저장 보존 정확성을 먼저 충족하고 성능을 개선한다.
알고리즘 비교에서 정확성과 시간/메모리를 별도 평가하며, 자원 제한에 도달한 처리는
미완료로 보고한다. 이 정책 기록에 따른 제품 코드 변경이나 빌드·시각 검사 재실행은 없다.

### 분할정복 설계와 다음 절편

메인테이너의 설계·구현계획 수정 지시에 따라 [계획 4.2–4.3절](../plans/task_m100_7998_impl.md#42-수정-설계-공통-관측과-소유-범위별-문단-구성)을
구체화했다. 원본 문자 관측을 공유하고, T0와 경계 적용 후 T1·소유·문단을 구분한다.
진단과 실제 lowerer는 같은 typed 결과를 참조하며, 셀/본문의 원점 변환과 source 줄의
수용/재조판 소비를 확인한다. 새 타입명은 설계안이며 현재 구현 완료로 표시하지 않는다.

실행 순서는 P1 셀 다중 글줄 문단, P2 혼합 run·공백/잘림, P3 본문 지역·복수 문단,
P4 표/문단 합성, P5 통합 검증이다. 첫 P1은 독립 대조군의 두 줄 한 문단/두 문단을 실제
셀 문단에 매핑하고 편집·양 저장 형식 재열기까지 연결한다. 이후 절편도 같은 제품 경로를
완주한 범위에서 수용한다. 기존 pass5 WIP·캐시·증거는 유지하며, 이번 설계 변경에서는
제품 코드·WASM·검증 대상 source가 바뀌지 않아 빌드·전체 검사를 반복하지 않았다.


## P1 공통 글줄·문단과 실제 셀 IR 연결 — 2026-10-18

메인테이너가 수정 계획 P1–P5를 승인해 P1을 구현했다. **문단 기능의 연결·편집은 충족,
시각 선행 조건과 P1 수용은 미충족**이다. P1 완료나 #7998 완료를 보고하지 않는다.
원격 작업은 승인 범위에 포함하지 않았으며 push·PR 생성·GitHub 게시를 수행하지 않았다.

### 위반 계약과 공통 생산·소비 경로

주석으로 독립 기대를 정한 2×2 native PDF에서 왼쪽 위 셀의 `ALPHA BETA` / `GAMMA`는
두 글줄 한 문단, 아래 `SECOND`는 별도 문단이다. 원본 PDF는 tagged 문서가 아니므로 이
주석은 이 대조군의 작성 의도이며 일반 PDF의 원래 편집 메타데이터를 입증하지 않는다.
기존 pass5 producer는 세 문단으로 생성했다. `p1/before/structure.json`에 FAIL을 보존했다.
새 검사를 옛 source에서 재컴파일한 증거와는 구분한다.

- `text_analysis.rs::ObservedText::observe/lines`가 source scalar/UTF-16 범위와 실제/근사
  geometry 품질을 보존한다. 상세 glyph 예산이 끝나도 coarse span 범위는 보존한다.
  `extract.rs`는 한 번 만든 Ledger의 text를 `recognition`에 전달하고 T0와 지역 글줄이
  같은 결과를 소비한다. 다른 셀의 같은 baseline을 하나의 셀 줄로 합치지 않는다.
- `recognition.rs::ParagraphProposal`은 typed 줄 목록·줄 연결 정책·소유 context·정렬/들여쓰기
  결과를 보존한다. `TableBuilder`는 JSON preview를 파싱해 문단을 재추측하지 않는다.
- `structured_lowering.rs::paragraph`가 확정 제안을 여러 `LineSeg`를 가진 한 Paragraph로
  만든다. 시각 wrap마다 hard newline을 만들지 않고 추론 단어 공백을 별도로 기록한다.
  문자열과 UTF-16 start·글자모양·source LineSeg가 같은 변환 결과를 소비한다.
- `paragraph_layout`의 기준 원점 → ParaShape의 `margin_left`/signed `indent`·정렬 → 공통
  style resolver/composer의 source 줄 수용 또는 재조판 → 실제 TableCell/TextLine으로
  소비한다. 첫 줄 간격은 Fixed leading으로 전달한다. 가운데 정렬과 내어쓰기의 후속 줄
  원점을 paint에서 덮어쓰거나 clamp하지 않는다. irregular 원점은 Deferred로 남긴다.
  Deferred의 줄 frame은 위치 보존이며 의미 문단 복원 완료로 세지 않는다.

첫 구현은 보고서 7쪽에서 86.35846%로 실제 회귀했다(`p1/attempt1/`). 처음의 세로 이동
가설은 SVG 원점 대조로 기각했다. 실제 원인은 후속 줄의 내어쓰기와 가운데 정렬 소실이었다.
source line 원점에서 공통 문단 layout을 만들고 margin/indent/정렬을 실제 스타일에 연결해
보정했다. 실패 출력과 소스는 보존했고 기존 renderer에 PDF 전용 보정을 추가하지 않았다.

### 같은 source의 제품 실행과 구조 검증

증거 root는 저장소 밖 `pdf-table-feedback-20261017/semantic-stage3/p1/`이다. `candidate/source.json`에
18개 runtime 파일의 SHA-256을 고정했고 빌드·캡처 후 일치도 확인했다. 후보는 아직 WIP이며
HEAD `65a4aaa7c984f9cd2d2cc57eaf99de0edf5bb6b2`만으로 소스 신원을 대신하지 않는다.
Native는 공용 `target/pr-review`, `--profile release-test --features native-skia --bin rhwp --locked`,
표준 Docker `wasm` service와 `rhwp_wasm-target` cache를 순차 사용했다. 실행 명령·시간·결과는
ignored `output/pr-review/issue7998/p1/{native-build-pass2,docker-wasm}.{json,log}`에 있다.
fresh WASM SHA-256은 `fedfa5bb9d3b21c85cf6d091f9edbf6f4ef218a85f4a936d45f15c9a1bbf0279`다.

| 독립 입력 | SHA-256 | 실제 문단·편집 결과 |
|---|---|---|
| multiline-cell.pdf | `9932fe6ab064921987fe03f771b4aa397344d72ed500cd86f96c50ce2acd3b3a` | (0,0) 셀 `ALPHA BETA GAMMA` 두 줄 한 문단 + `SECOND` 한 문단; 옆 셀 FIRST/SECOND는 각각 한 문단 |
| paragraph-origins.pdf | `ef812cddbce2a4ad6fc2b26ba2355a65cfb5a1b135f67170f22ee14d35e976d6` | (0,0) 두 줄 문단의 내어쓰기, (0,1) 가운데 정렬 두 줄 한 문단; 아래 셀의 별도 소유 유지 |

`verify_native.py`는 실제 CLI import·HWPX XML 문단/LineSeg 조회·문자 삽입과 HWP 재열기를
검사했다. HWP의 CLI table 조회는 문단을 평탄화하므로 그것만으로 문단 수를 증명하지 않는다.
그 경계는 실제 Studio의 `getCellParagraphCount/getCellParagraphLength/getTextInCell`로
두 형식 모두 다시 조회했다(`candidate/studio-result.json`).

Studio 7793에서 위 두 PDF 각각 `insertTextInCell` → `deleteTextInCell` →
`splitParagraphInCell` → `mergeParagraphInCell` → HWPX export/reload → HWP export/reload를
실행했다. 각 단계의 정확한 원문·공백·문단 수와 이웃 셀 내용을 확인했고 오류가 없었다.
HTTP의 WASM SHA도 후보 package와 같았다. 두 입력의 HWP/HWPX 저장본을 다시 그린
Native PNG는 PDF 직접 import의 PNG와 바이트가 같았다. 이것은 저장에 새 시각 손실이
없다는 증거이며 원본 PDF와 글자 폭이 일치한다는 증거로 사용하지 않는다.

### 전쪽 시각 결과와 수용 보류

모든 입력의 전체 쪽수는 독립 PDF와 같다. `candidate/visual/{native,wasm}/<key>/silhouette.tsv`
전체 원문과 `{native,wasm}-visual-results.json`, 저장본의 `saved-native-visual.json`을 보존했다.

| 입력 | 전쪽 수 | Native 최저 | fresh WASM 최저 | 판정 |
|---|---:|---:|---:|---|
| multiline-cell | 1 | 91.04550% | 89.01544% | 시각 미충족 |
| paragraph-origins | 1 | 90.19653% | 88.26223% | 시각 미충족 |
| 원래 보고서 | 12 | 91.04017% | 90.62487% | 수치 유지, 원래 5쪽 탭 뒤 잘림은 P2 미해결 |
| Excel 견적서 | 1 | 97.62909% | 97.46596% | 수치 유지, 원래 삽입 후 공백은 P2 미해결 |
| native 점선 대조군 | 1 | 99.94962% | 98.88438% | 기존 자격 유지 |
| 정상 실선 대조군 | 1 | 100% | 99.53052% | 기존 경로 유지 |

바로 앞 pass5와 같은 backend/입력/DPI/글꼴 조건으로 비교한 30개 페이지에서 0.01%p 이상
하락은 없다(`candidate/pass5-comparison.json`). 이는 현재 source의 직전 WIP 대비 영향 검사다.
최신 upstream/devel의 제출용 기준을 다시 검증했다는 주장은 아니며 최종 P5에서 구분한다.

대조군의 Helvetica를 `getFontDecisionTrace`로 추적하면 `layoutMetric.matchKind=none`,
`widthSource=heuristicHalfwidth`다. 원본/기준 PDF는 그대로 두고 `pdffonts -subst`가 확인한
NimbusSans-Regular 파일을 양 backend에 공급한 별도 대조도 수행했다. 실제 파일 SHA와
세션 설정은 `p1/font-evidence.json`, `font-environment.json`에 기록했다. 이때 Native는
91.34357/90.58986%, WASM은 89.26061/88.62069%로 여전히 미달이다. outline 공급만으로
미지원 메트릭이 해결된다고 가정하지 않는다. `p1/font-trace.json`에 기본/명시 환경을 보존했다.

새 두 PDF·문단 검사 초안은 저장소 밖에만 두며 정식 `tests/cases`/fixture로 등록하지 않았다.
기존 정식 6건은 현재 source로 실행하고 결과를 아래에 기록한다. 전체 회귀·lint를 중복 실행하거나
원래 미달을 상수/기준 변경으로 통과시키지 않았다. 후속 세부 보정은 승인된 P2의 정확한 glyph
advance/공백 계약과 연결하며 P1 시각 수용을 열린 상태로 유지한다.

검토 화면: `E:\tmp\issue7998-semantic-table-review\p1\index.html`.
독립 원본·Native/fresh WASM, 글줄 box/문단 묶음, 실제 저장 IR·편집본·원문 TSV·공급 대조를
함께 볼 수 있다. 이전 pass5 HTML은 보존하고 최신 P1 링크만 추가했다. 원래 Studio 7700과
사용자 피드백은 변경하지 않았다.

현재 source의 기존 정식 6건은 `node scripts/rust-test-suite-manifest.mjs --prepare` 후
`CARGO_BUILD_JOBS=4 node scripts/run-rust-test.mjs issue_7998_pdf_table_inference -- --cargo-profile
release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia`로
6/6 통과했다(`output/pr-review/issue7998/p1/focused-tests.log`). 이 결과는 새 문단 검사 등록이나
P1 시각 조건 충족을 뜻하지 않는다. 초안과 두 PDF는 계속 외부 경로에 보존한다.

## P2 수평 글줄의 전진폭·공백과 저장 좌표 — 2026-10-18

P1의 시각 보류를 원본 글자 전진폭의 공통 소비 문제로 보정했다. 메인테이너는
`p2/index.html`의 보고서 5쪽 끝 글자와 문단 대조군을 직접 확인해 **시각 판정 통과**를
주었다. 이 판정을 이후 별도로 발견한 저장본 7쪽이나 P2 전체의 완료로 확대하지 않는다.
현재까지의 독립 source와 실제 제품 실행은 아래 경계를 입증한다.

### 공통 계약과 생산·소비

- `text_analysis.rs::reconstruct_line`은 source scalar/UTF-16 참조, 실제 glyph 원점,
  nominal advance와 원점 간 advance를 구분한다. 같은 글줄의 span 분할이나 TJ 표현이
  달라져도 같은 단어 공백을 복원한다. 추론 공백은 source 문자 참조를 갖지 않으며 원래
  공백을 삭제·중복하지 않는다. 완전한 원점이 없거나 역방향·겹침 등이 표현되지 않으면
  그 경계를 미검증으로 기록하고 기존 positional/줄 frame을 보존한다.
- `structured_lowering.rs::observed_line_text`는 이 결과를 공통 CharShape 장평·자간과
  Paragraph의 글자모양 run으로 표현한다. `style_resolver`와 `estimate_text_width`의
  실제 측정을 사용하며, 공통 composer → TableCell/TextLine/TextRun이 같은 IR을 소비한다.
  화면에 맞춘 셀 margin·bbox clamp나 PDF 전용 paint 덮어쓰기를 추가하지 않았다.
- 원래 font가 없는 곳은 대체 글꼴을 명시하고 관측 전진폭을 표현한다. 원래 font outline을
  복구했다고 주장하지 않는다. HWP 정수 백분율의 양자화 오차는 진단에 남긴다. 보고서 5쪽
  24줄은 모두 관측 원점 경로이며 최대 오차는 0.35736751556396484pt다. 6쪽 2줄·7쪽
  14줄은 여전히 `unverified_legacy_line_frame`이다. 다른 줄의 성공으로 면제하지 않는다.
- `lowering.rs`의 font-space 선택은 실제 원점 간 공백 폭이 선언 font 메트릭과 일치할 때만
  기존 `use_font_space` 계약을 켠다. nominal `char_widths`만 비교한 첫 시도는 원래 보고서
  1–3쪽을 실제로 악화시켰고 기각했다. `p2/metrics-pass/` 실패와 source는 보존했다.
  TJ/Tw가 반영된 실제 원점으로 판단한 뒤 기존 출력이 회복됐다.
- Helvetica의 ASCII 메트릭은 Adobe 공개 AFM 네 파일에서 glyph 이름으로 생성했다.
  원본 AFM·저작권·MustRead 라이선스와 archive SHA를 `data/font-metrics/adobe-core14/`에
  보존한다. 생성기는 원본 600개 항목의 순서·인덱스를 바꾸지 않고 별도 메트릭을 연결한다.
  trace는 실제 AFM을 출처로 쓰며 과거 증거 원장의 미등록 항목을 등록된 근거로 꾸미지 않는다.
  재생성 결과는 현재 Rust 파일과 바이트가 같다(`p2/generator-reproducibility.json`).

### 검출 증거와 이전 진단의 정정

독립 `tj-cell.pdf`와 `split-cell.pdf`는 같은 첫 줄을 각각 TJ 간격과 별도 BT/Tj 연산으로
작성했다. 원래 주석 대조군과 함께 실제 첫 셀은 `ALPHA BETA GAMMA` 두 줄 한 문단과
`SECOND` 별도 문단이어야 한다. 이전 producer는 span을 나눈 입력에서 단어 사이에 탭을
만들었다. `p2/before-contract/`에 같은 CLI 제품 경로의 FAIL과 현재 PASS를 보존했다.
새 Rust 검사 함수 자체를 옛 source에서 재컴파일한 증거는 아니다.

보고서 5쪽의 ‘먼지목욕’·‘바닥’은 이전 실제 TextRun이 셀 오른쪽을 각각 약 9.7px·13.6px
넘어 clip에 가려졌다. 현재는 owning cell 내부에 있고 끝 글자가 실제 Native/WASM에
표시된다. 문자열 존재만 검사한 결과와 구분해 독립 PDF의 같은 영역을 직접 확대 판독했다.

**견적서 ‘삽입 후 추가 공백’ 진단을 정정한다.** 과거 `export-tables`의 trim된 문자열을
기대값으로 사용해 원래 있던 선행 공백을 새 결함으로 오인했다. 이전 pass5 저장 XML도
원문 14자 → 접두어 삽입본 17자이며 정확히 `IR:` + 원문이다. 현재 실제 Studio 삽입·저장
재열기도 같은 계약을 통과했다. 이 항목은 새 코드로 고친 편집 버그가 아니다.
`p2/quotation-historical-comparison.json`, `p2/candidate/quotation-studio-result.json`에
원본·실행 증거를 보존하며 연락처가 든 상세 자료는 로컬에만 둔다.

### 저장본에서 추가 검출한 탭 좌표 결함

직접 PDF 열기와 독립 PDF의 전쪽 90% 통과 뒤에 저장본을 별도로 검사했다. 네 문단 대조군과
견적서는 양 저장 형식에서 유지됐지만, 보고서 7쪽은 Native HWP/HWPX 87.99264/87.96357%,
WASM 88.50803/88.64197%로 실패했다. 직접 열기 통과나 메인테이너의 앞선 판정으로 이를
통과시키지 않았다(`p2/candidate/saved-visual-results.json`).

원인은 생성한 탭의 축이었다. Rust 문자열의 한 글자 탭은 HWP `PARA_TEXT`에서 8유닛인데,
새 문단의 char offset·CharShapeRef·LineSeg 시작을 텍스트 내용 UTF-16 축으로 기록했다.
저장 후 문자 offset만 확장되면서 후속 줄과 글자모양이 앞줄을 가리켰다. 예를 들어 7쪽
(행 1, 열 3)의 네 줄 중 두 번째가 ‘공간 확보, 공용’ 대신 ‘간’으로 시작했다.
원문·source 줄 장부는 남아 있으므로 텍스트 총량만으로 검출할 수 없는 결함이다.

`structured_lowering.rs::finalize_text_axis`에서 하나의 내용→stream 매핑으로 char offset·
CharShapeRef·LineSeg를 함께 발행하도록 보정했다. 탭 폭은 모델의 기존
`Paragraph::char_stream_len` 계약을 소비한다. positional 단일 줄도 같은 단위를 사용한다.
serializer의 수용 조건을 완화하지 않았다. 상세 진단의 `paragraphUtf16Start`는 floating
control 삽입 전의 **내용 축**이며, 실제 HWP stream 위치와 구분한다.

기존 source/producer·실패 PNG·실제 WASM 글줄 조회는 `p2/save-axis-fix/before-source/`,
`before-lines.json`, `before-line-contract.json`에 남겼다. 좌표만 바꾼 진단 저장본에서
7쪽이 98.87228%로 회복된 뒤 실제 코드에 적용했다. 현재 Native 제품의 8개 입력 직접 열기와
6개 입력×양 저장 형식 34쪽은 모두 90% 이상이다. 저장본 PNG는 직접 열기와 모두 같다.

### 최종 검증 상태

`output/pr-review/issue7998/p2`는 기존 저장소 밖 증적 디렉터리로 연결한다. 최종 producer와
37개 runtime 파일의 신원은 `save-axis-fix/candidate/source.json`에 고정했다. Native는 공용
host target의 `--profile release-test --features native-skia --bin rhwp --locked`, fresh WASM은
표준 `docker compose -p rhwp --env-file .env.docker run --rm --no-deps -e CARGO_BUILD_JOBS=6
-e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm`을 사용했다. 양 빌드 모두 exit 0이며
빌드 전후 source hash가 일치한다. 실행 로그·명령·시간은 `save-axis-fix/{native-build,docker-wasm}.{json,log}`다.
fresh WASM SHA-256은 `2f91abc71e01241554e1a13e66b8184b9654e008c9cfb40b87243382dfcc34a7`이다.

| 입력 | 쪽수 | Native 최저 | fresh WASM 최저 |
|---|---:|---:|---:|
| multiline-cell | 1 | 99.94837% | 99.94720% |
| paragraph-origins | 1 | 99.95058% | 99.94893% |
| operators | 12 | 91.04017% | 90.62487% |
| quotation | 1 | 97.69730% | 97.55112% |
| periodic-native-grid | 1 | 99.94962% | 99.79424% |
| solid-control | 1 | 100.00000% | 100.00000% |
| tj-cell | 1 | 99.41101% | 99.20990% |
| split-cell | 1 | 99.82070% | 99.73788% |

모든 입력의 전체 쪽수는 독립 원본 PDF와 같다. `candidate/visual/{native,wasm}/<key>/silhouette.tsv`
원문과 `{native,wasm}-visual-results.json`을 참조한다. 8개 입력×양 backend의 직접 열기 38쪽을
새로 캡처했고, 탭 축 보정 직전과 PNG가 바이트 동일하다. 앞선 5쪽·문단 대조군의 직접 판정을
새로운 저장 경로의 판정과 구분하며 해당 출력이 유지됐음을 확인했다.

네 문단 대조군·보고서·견적서의 양 저장 형식은 양 backend 합계 68쪽 모두 직접 PDF 열기의
PNG와 같다(`candidate/saved-visual-results.json`). 7쪽의 저장 전후 실제 WASM 글줄 조회도
독립 PDF의 네 줄 내용·순서를 유지한다(`save-axis-fix/after-lines.json`). 메인테이너가 7쪽의 새 저장본도
직접 확인해 **시각 판정 통과**를 주었다. 앞선 5쪽·문단 대조군의 판정과 별도로 기록한다.

실제 Studio 7793은 위 SHA의 새 WASM을 제공하며, 네 문단 대조군의 삽입/삭제·split/merge·
양 저장 형식 재열기와 이웃 셀 보존을 통과했다(`candidate/studio-result.json`). 견적서도
접두어+선행 공백 원문이 정확히 보존된다(`quotation-studio-result.json`). 이는 실제 제품의
편집·저장 검사이며 저장본의 한컴 편집기 열기/인쇄 증거를 대신하지 않는다.

정식 suite에 P1/P2 독립 네 대조군과 source 문자 참조·AFM 근거·5쪽 끝 글자 관계를 추가한
12건은 탭 축 보정 전 source에서 통과했다. 새 source에서는 7쪽 탭 반례를 시각 선행 조건
충족 뒤 추가해 집중 **13/13건**을 통과했다. 기존 font 메트릭도 새 source에서 **9/9건**을 통과했다
(`save-axis-fix/font-metric-tests.log`). 자동 suite가 선택한 module filter로 실행하며 CI에
개별 호출을 추가하지 않았다. 새 검사 추가 뒤 suite 재생성을 빠뜨린 첫 실행은 exit 4·0건
수집이었으며 성공으로 세지 않았다. `--prepare` 후 실제 13건 수집·통과를 확인했다
(`save-axis-fix/suite-prepare.log`, `focused-tests.log`). 예전 source의 새 Rust 함수 재컴파일과 제품 경로 FAIL/PASS를
구분한다. 대용량 전체 회귀·lint·package는 P5에서 수행하며 이전 결과를 현재 source의 완료
근거로 재인용하지 않는다.

직전 P1 WIP와 동일 backend/입력/조건의 공통 30쪽에서 0.01%p 이상 하락은 없다
(`candidate/p1-comparison.json`). 이는 최신 upstream/devel의 최종 제출용 비교가 아니다.
원래 7700 JS/WASM과 사용자 피드백 파일은 보존했다. 두 원본 입력·37개 runtime 파일·
7700 산출물의 SHA를 마지막에도 대조했다(`candidate/producers.json`). `git diff --check`와
`cargo fmt --all --check`도 통과했다.

검토 화면: `E:\tmp\issue7998-semantic-table-review\p2\index.html`. 실제 문단·줄 소속,
원문/편집본·양 저장 형식, 원본/Native/fresh WASM PNG와 전쪽 TSV를 제공한다. 7쪽 저장 축
수정 전 실패도 함께 보존했다. 원본 AFM·라이선스와 독립 PDF의 byte 보존은 `.gitattributes`에
제한된 경로로 지정했다. 라이선스 원문의 CR 개행·공백을 source whitespace 정리로 바꾸지 않는다.


font 메트릭의 집중 명령은 `CARGO_BUILD_JOBS=2 cargo nextest run --locked --lib
-E 'test(renderer::font_metrics_data::tests::)' --cargo-profile release-test --target-dir
/home/edward/mygithub/rhwp/target/pr-review --features native-skia`다. PDF 집중 검사는
`CARGO_BUILD_JOBS=4 node scripts/run-rust-test.mjs issue_7998_pdf_table_inference --
--cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review
--features native-skia`로 실행했다.

### 다음 P2 경계와 전체 작업의 미완료 범위

외부 `p2/next-mixed-baseline/`에 혼합 글꼴·위첨자 대조군을 준비했다. Native 초기 조회에서
둘 다 실제 셀 문단과 원문은 보존되지만, `H2O`의 작은 2가 원본처럼 올라가지 않는 결함이
보인다. 실루엣 99.66283%를 세로 배치 통과로 해석하지 않는다. 이 두 입력은 아직
fresh WASM·정식 회귀 자격을 부여하지 않은 진단본이다. 다음 P2에서는 source baseline
차이의 표현과 공통 측정·실제 배치의 소비를 먼저 연결한다.

P3 본문 지역·복수 문단과 P4의 표 계층/병합·compound paint·편집 계약, P5의 최신 devel
대조·통합 회귀·lint·패키지·저장본 한컴 열기/인쇄는 남아 있다. 보고서의 다중 줄 문단 42개는
현재 추론 결과이며 원래 작성 문단 수를 모두 독립 검증한 값이 아니다. 이번 절편의 통과를
전체 #7998 완료나 최종 PR 준비 완료로 표시하지 않는다. 원격 작업은 수행하지 않았다.


### P2 계속 — 혼합 기준선 (2026-10-18, 구현 착수)

위반 계약: PDF에 명시된 작은 글자 크기와 기준선 차이는 서로 다른 속성이다.
`H2O` 대조군의 2는 6pt·+4pt rise인데 기존 출력은 크기만 보존하고 기준선을
내렸다. 원인은 painted span 추출기의 reading-order용 `snap_superscript_baselines`와
PDF lowering의 글자 위치 누락이다. 일반 검색 추출은 유지하고 painted import만
원래 좌표를 보존한다. line owner의 일반 글자와 작은 첨자를 기하적으로 연결하되
원래 좌표를 덮어쓰지 않는다. 큰 글자 크기의 기준선을 줄 anchor로 사용하고
동일 거리의 경쟁 owner는 억지로 선택하지 않는다.

독립 근거: 외부 `p2/next-mixed-baseline/make_sources.py`가 작성한 PDF 연산자·
source annotation. HWP 5 표33과 한컴 공식 HWPML revision1.2의 CHAROFFSET은
기준 크기의 -100~100%, 양수 아래·음수 위로 정의한다:
https://cdn.hancom.com/link/docs/한글문서파일형식3.0_HWPML_revision1.2.pdf .
PDF의 이미 작은 글자에 superscript flag를 추가해 두 번 축소하지 않는다.

값 추적: source painted span bbox.y → ObservedText의 원래 baseline 및 owner-local
TextLine anchor → structured_lowering::observed_line_text의 기준선 차이/기준 크기
→ CharShape.char_offsets → style_resolver의 언어별 baseline_offsets(px)
→ resolved_to_text_style → TextStyle.script_draw_metrics → CanvasTextFont 및
SVG/HTML/Skia/paint_bounds의 동일 draw baseline. 가로 측정·선언 줄 상자는
글자 크기/저장 LineSeg 계약을 유지하며 글자 위치로 텍스트 흐름을 전진시키지 않는다.
소스 줄 상자·글리프 점유와 실제 앞뒤 줄/셀 포함은 출력으로 별도 확인한다.
별도 horizontal-shaping/세로쓰기 최적화가 이 필드를 무시하지 않도록 해당 비지원
경로는 일반 공통 text replay로 보낸다. 스타일 범위를 넘는 위치를 clamp하지 않고
기존 명시적 보류 경로와 원문을 보존한다.

반례는 위/아래 위치, 첨자 먼저/나중, 혼합 글꼴·크기, 분리되어야 할 다음 줄을
포함한다. 변경 전 진단본·실제 PNG는 외부 증적에 보존하며 점수만으로 판정하지 않는다.

구현 중 실제 셀 검증에서는 점수가 높아도 의미 표가 0개가 되는 반례를 확인했다.
`diagnosis-1/`은 H/O reading-order 병합 때문에 `H O`와 떨어진 2가 생겨 셀 flow가
겹쳤다. painted import는 span을 원문 paint 단위로 유지하며 문단 adapter가 줄을
묶도록 보정했다. `diagnosis-2/`는 source Ts 좌표는 맞지만 가로 원점의 비교 대상이
검색용 char cache여서 6pt·4pt rise의 앵커가 사라졌다. 해당 source metadata를
완화하거나 한 글자만 예외 처리하지 않고, painted char 추출과 span이 같은 Ts/Tm/CTM
원점을 소비하도록 했다. `Ts`가 바뀔 때와 `Tm` 기준선이 바뀔 때의 buffer 경계도
보존한다. 검색 추출의 snap/문자 치환/병합·cache는 기존 정책을 유지한다.

backend 진단 `extractor-probe-final.log`에서는 단일/복수 숫자, 위/아래, 선두 첨자,
같은 글꼴 Ts/Tm 7개 원본의 원래 문자·Y·크기와 완전한 가로 원점 배열을 확인했다.
이 진단만으로 제품 조판·시각 판정이나 새 fixture 자격을 부여하지 않는다.

### P2 혼합 기준선 — 호출 경계와 독립적인 보존 계약

메인테이너는 `p2-baselines/index.html`의 혼합 기준선 대조군 10개에 **시각 통과**를
판정했다. 이후 호출 경로 감사에서 수평 메트릭 복원이 실패하는 보고서 6쪽 `m²`와
일반 실선 표 경로가 원래 글자모양 ID를 사용해 세로 위치를 버리는 별도 결함을 확인했다.
시각 통과를 이 미검증 경로의 통과로 확대하지 않았다.

`source_baseline_position`은 source baseline과 line anchor의 차이만 계산한다.
`observed_line_text`의 장평·자간 복원이 성공한 경로뿐 아니라 `text_with_tabs`의
보류 frame과 `text_with_spaces`의 일반 문단도 `positioned_span_style`로 같은 글자
위치를 소비한다. 폭이 표현되지 않는다는 이유로 독립적인 세로 속성을 버리지 않으며,
폭의 장평 범위를 넓히거나 원점을 clamp하지 않는다. style cache의 rollback 이후에도
원래 크기·폭 속성을 보존한 위치 스타일을 새로 적용한다.

독립 기대값은 보고서 PDF의 실제 glyph 원점이다. 6쪽 `/m`은 11.0279483795pt·
baseline 612.8892822266pt, `2`는 7.0722708702pt·617.68408203125pt다. 일반 글자 대비
크기 비율과 기준선 차이를 실제 RenderTree의 공통 draw 결과에서 검사하고, 양 저장
형식으로 재열어 같은 관계를 확인한다. 수평 보류 상태 자체를 영구 기대값으로 고정하지
않는다. 작은 숫자에 붙은 배치 탭은 원문 숫자 검사와 구분한다.

두 경로의 동일 회귀 함수는 보정 전 library에서 기준선 차이 0으로 **FAIL**, 보정 후
library에서는 **PASS**다(`fallback-before.log`, `fallback-after.log`,
`solid-before.log`, `solid-after.log`).
보정 전 library·실행 파일·source fingerprint·사용자 판정은 외부
`p2-baselines/pre-deferred-position/`에 해시와 함께 보존했다. 이 실패는 빌드 오류나
원문 공백을 제거한 기대값 때문이 아니다.

저장 원문과 편집 후 재조판은 구분한다. 무편집 source/양 저장본은 원래 두 줄을 유지한다.
실제 삽입·삭제·문단 분할·병합 뒤에는 셀 가용 폭에 따라 `GAMMA`가 첫 줄에 재배치될 수
있으므로 과거 LineSeg를 고정하지 않는다. 그 경로에서는 원문·이웃 셀 소유와 크기·기준선
관계를 검증한다. 글자 위치는 ink를 이동하고 줄의 논리 흐름 상자를 이동하지 않는다.
paint bounds는 원래 상자와 위치 이동 상자를 합쳐 clip/cache 누락을 방지하며 회전
원점은 원래 상자를 사용한다.

### P2 혼합 기준선 — 최종 출력·편집 검증

증적 루트는 저장소 밖
`/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p2-baselines/`다.
task의 ignored `output/pr-review/issue7998/p2-baselines`도 같은 위치를 가리킨다.
source는 `9a1fd0406b44d5c023d1b85af40fa3b670e90216` 위 이번 절편이며
`source-final-runtime.json`의 src/vendor/Cargo 1,653개 해시를 빌드 전후 대조했다.
그 manifest의 SHA-256은 `0d4846da56af11d7ce38ae8da04a93b16ec570f564292cb8141b3925a621219c`다.
테스트/보고서 등록 뒤에도 runtime은 동일하며 시각 빌드·캡처를 반복하지 않는다.

| 실행 | 실제 범위와 결과 |
|---|---|
| Native | `cargo build --locked --profile release-test --features native-skia --bin rhwp --target-dir /home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`; 종료 0, 4분. 실행 파일 SHA-256 `10ba28ee0e5e3cde466d96c3faeb57fac6204f60ed756c864b0c61a31a5ab7b6`. |
| fresh Docker WASM | `docker compose -p rhwp --env-file .env.docker run --rm --no-deps -e CARGO_BUILD_JOBS=6 -e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm`. root 컴파일 2분55초, wasm-opt 및 package 생성 7분55초 완료 메시지. Docker wrapper 종료는 **143**으로 남아 정상 종료로 보고하지 않는다. 완성된 pkg/public의 바이트 일치와 실제 로드·실행을 별도 확인했다. WASM 19,149,031바이트, SHA-256 `f89dc4e41d6ff89d0ff4de5fc79b610dbf2fefae3ef0c762550de247cadab6e0`. |
| 전쪽 raw TSV | 독립 기준선 11개·기존 표/문단 대조군 6개·실물 보고서 12쪽·견적서 1쪽, **원본 19개/30쪽**. 각각 직접 PDF와 HWP/HWPX 저장본을 비교해 backend마다 57실행/90쪽이다. Native 최저 **97.45743%**, WASM 최저 **97.41558%**. 180쪽 모두 90% 이상·쪽수 일치·누락 없음. |
| 무편집 저장본 | 19개 원본의 HWP/HWPX 재열기 PNG는 각 backend에서 직접 PDF import PNG와 전부 바이트 동일하다. 이 검사는 Native CLI가 저장한 파일의 재열기이며 실제 WASM export는 아래 Studio 검사로 구분한다. |
| 기존 P2 비교 | 양쪽에서 열리는 기존 원본 19쪽씩을 같은 backend의 이전 P2 TSV와 비교해 0.01pp 이상 하락 없음. `final-comparison.json`에 38쪽의 전후 값을 보존했다. |
| 사용자 판정 보존 | 사용자 통과 10개 대조군의 최종 직접/양 저장본 PNG 30개씩, **총 60개**가 판정본과 바이트 동일하다. 동일 시각 판정을 다시 요청하지 않았다. 추가 실선 표와 보고서 6쪽의 판정은 구현자 직접 판독으로 따로 남긴다. |
| 실제 Studio | 7793에서 10개 대조군의 PDF 열기 → 삽입/삭제 → 문단 분할/병합 → 실제 WASM HWPX/HWP export·재열기, 70개 상태를 확인했다. 원문·공백·이웃 셀/문단 소속 일치, page error 0. instantiateStreaming이 실제로 받은 `/@fs/.../pkg/rhwp_bg.wasm` 응답을 해시해 위 WASM과 일치했다. main Studio 7700의 JS/WASM 해시는 변경 전과 같다. |

명령 원문·exit·페이지별 TSV는 `native/results.json`, `wasm/results.json`과
각 `visual/<입력>-<형식>/silhouette.tsv`에 있다. `validate.py native/wasm`은 96dpi,
compat 2022, Native Skia PNG와 실제 WASM PrintCanvas를 사용했다. 올바른 Batang 공급은
기존 `output-fixes/batang-only`를 재사용했다. 전체 overlay를 중복 생성하지 않고
실제 영향 영역을 확대 비교했다.

새 PDF 11개는 양 backend의 직접/양 저장본 시각 조건을 확인한 후에만
`pdf/issue7998/`에 회귀 입력으로 편입했다. 원래 PDF 연산자와 주석의 해시는 외부
`make_sources.py`, `*-source-annotation.json`에 보존했다. 일반 실선 표는 별도 생성본이며
source와 Native/WASM을 직접 비교해 99.97721/99.97686%와 올바른 위치를 확인했다.
이는 한컴 작성 문서의 원래 문단 메타데이터를 입증한 것이 아니다.

구현자는 보고서 3·5·7쪽의 이전 P2/현재 Native와 현재 WASM을 원본과 확대 대조했다.
5쪽 `먼지목욕`·`바닥` 끝 글자, 7쪽 네 줄 탭 구성·인접 셀·후속 내용과 6쪽 위첨자를 확인했다.
이전 P2 대비 WASM 4쪽은 99.73004→99.72187%(0.00817pp), 6쪽은
99.17763→99.17744%(0.00019pp)로 조금 낮다. 십진수로 canonical 0.01pp 경계를 비교했으며
이를 하락 0쪽으로 뭉뚱그리지 않는다. 4쪽도 원본/이전/현재를 확대 판독해 새 내용 누락이나
배치 결함은 확인되지 않았다(`public-p4-wasm-review.png`). 6쪽은 위첨자 위치 보정을
직접 대조했다. 가는 괘선의 농도와 원래 글꼴 대체의 모양/간격 차이는 남으며, 사용자 과거 판정을 새 SHA의
포괄 판정으로 바꾸지 않았다. 검토 화면은
`E:\tmp\issue7998-semantic-table-review\p2-baselines\index.html`이다. 모든 링크·그림은
폴더를 복사해도 열리도록 상대 경로로 제공한다.

이번 완료 범위는 **혼합 기준선 절편**이다. source typography의 184줄은 관측 원점과
전진폭을 소비하고 16줄은 원래 글꼴의 장평 범위 문제로 수평 frame에 남는다. 이번에는 그
줄의 세로 위치도 보존했지만 수평 메트릭 복원을 완료했다고 보고하지 않는다. 보고서의
다중 줄 문단 42개도 작성 당시 문단 수의 독립 정답이 아니다. B12·P4에서 남은 표현을
추적하며 다음 실행 절편은 P3 본문 지역·실제 문단 연결이다. P3·P4·P5와 원래 계획의
전체 회귀·lint·Worker/package·저장본 한컴 열기/인쇄는 미완료이며 원격 반영은 하지 않았다.

정식 집중 회귀는 위에 기록한 `scripts/run-rust-test.mjs issue_7998_pdf_table_inference`
명령으로 **17건 전부 PASS**다(`focused-final-tests.log`, 209건은 filter로 미실행).
`--prepare` 뒤 이번 모듈은 `regression_suite_021`에 자동 배정됐으며 기존 자동 수집
manifest 검사를 통과했다. generated suite/manifest나 개별 CI 호출은 source에 추가하지
않았다. 새 함수는 실제 셀의 혼합 원문·기준선·크기·편집/저장, 혼합 글꼴 문단 소속,
분리되어야 할 작은 줄, 보고서의 수평 보류 줄 보존을 확인한다. 일반 실선 표는 같은
실제 편집·양 저장 helper의 대조군이다. 절대 px/golden/과거 페이지 수 핀은 추가하지 않았다.

`cargo fmt --all -- --check`와 `git diff --check`도 통과했다. formal test/문서 추가 후
runtime fingerprint와 fixture 11개의 source byte 해시가 그대로인지 마지막으로 확인했다.
`fixture-admission.json`은 외부 증적으로만 보존하며 로그·중간 JSON·사적 견적서는 커밋하지 않는다.

---
kind: plan
status: active
canonical: mydocs/working/task_m100_7642_stage2.md
last_verified: 2026-10-08
---

# 이슈 #7642 시험지 16개 전체 시각 검증·관계 회귀 보정

이슈: [#7642](https://github.com/edwardkim/rhwp/issues/7642)
이전 단계: [수식·글꼴 원인 보정](task_m100_7642_stage1.md)

## 승인된 확장 범위

사용자가 HWP 11개와 HWPX 5개의 Visual Sweep·실제 출력 개선 및 px 고정 회귀의 관계 검사
변경을 요청했다. 기존 수식 96개·원본 11월·#1139 파일 전체 범위도 유지한다. 재저장본 문제는
별도 #7640이다. 원본을 새로 저장한 입력으로 바꾸거나 대조군 통과로 원본 미달을 면제하지 않는다.

| ID | 입력 | 대응 Print engine |
| --- | --- | --- |
| exam-01 | `samples/21_언어_기출_편집가능본.hwp` | 2020 |
| exam-02 | `samples/exam_eng.hwp` | 2020 |
| exam-03 | `samples/exam_kor.hwp` | 2020 |
| exam-04 | `samples/exam_math.hwp` | 2020 |
| exam-05 | `samples/exam_math_8.hwp` | 2020 |
| exam-06 | `samples/exam_math_no.hwp` | 2020 |
| exam-07 | `samples/exam_science.hwp` | 2020 |
| exam-08 | `samples/exam_social.hwp` | 2020 |
| exam-09 | `samples/exam-kor-2p.hwp` | 2020 |
| exam-10 | `samples/exam-kor-3p.hwp` | 2020 |
| exam-11 | `samples/exam-kor-4p.hwp` | 2020 |
| exam-12 | `samples/hwpx/exam_kor.hwpx` | 2020 |
| exam-13 | `samples/hwpx/exam_social.hwpx` | 2020 |
| exam-14 | `samples/hwpx/exam-kor-2p.hwpx` | 2020 |
| exam-15 | `samples/hwpx/exam-kor-3p.hwpx` | 2020 |
| exam-16 | `samples/hwpx/exam-kor-4p.hwpx` | 2020 |

## 순서와 증거

1. 앞 단계 변경을 `302be66a1` 일반 커밋으로 보존했다. 같은 source의 fresh WASM은 새로 만든다.
   root `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`를
   사용하고 pkg/Studio public SHA 일치와 실제 브라우저 실행을 확인한다.
2. 원본 파일 해시·저장 제품/버전·Print engine을 먼저 고정했다. 독립 MCP Print는
   매뉴얼의 `start → status → download`, 명시 engine·파일명·1800초 timeout을 따른다.
   출력마다 실제 `engine`, `hancom_version`, `pdf_print_method=0`, one-up mode와 쪽수를 검증한다.
   endpoint·token은 환경 파일로만 전달하고 공개 기록에 남기지 않는다.
3. 전체 페이지 Native/fresh WASM TSV를 산출하고 최저값·미달/누락 쪽·전체 쪽수 차이를 기록한다.
   미달 쪽과 대표 수식·표·그림·문단 경계의 compare·overlay·review를 직접 판독한다.
4. 실제 호출 경로에서 첫 발산의 원인·측정/배치 소비·반례를 확인한 뒤 공통 규칙을 보정한다.
   파일명·문서 ID·좌표 상수·clamp·숨김을 사용한 화면 맞춤은 하지 않는다. 영향 입력과 정상
   대조군을 새 code head에서 다시 출력한다. 세부 수식·간격은 90% 수치 통과 뒤에도 판독한다.
5. 검증 범위 전체 페이지·출력 경로가 90% 이상이고 쪽수가 같을 때만 해당 입력의 기존 px 검사를
   문단·쪽·개체 소속, 앞뒤 순서, 셀/열 내부 포함, 겹침·누락·중복 관계로 변경한다.
   의도한 결함의 수정 전 FAIL/수정 후 PASS와 독립 기대 근거를 연결한다. 기존 검사를 미달 상태에서
   삭제하거나 허용치·래칫을 완화하지 않는다. 합성 계약 통과를 실제 출력 일치로 보고하지 않는다.
6. 최종 source/base와 정확한 명령으로 전체 Rust lint·회귀·Native Skia·fresh WASM 게이트를 마치고,
   오늘 할 일·최종 판정·대표 PR 본문 이미지를 포함한 새 PR에서 CI→merge→후속 처리를 수행한다.
   PR #7600은 이미 통합됐으며 신규 작업의 현재 PR로 사용하지 않는다.

## 현재 실행 위치

- 입력·해시·저장 버전: ignored `output/pr-review/pr7600/issue7642-exam-print/inputs.json`.
- Print 실행: 같은 디렉터리의 `run.py`, `run.log`, `results.json`과 입력별 start/status/download JSON.
  아직 전체 terminal 결과를 확인하지 않았으며 PDF 생성 완료나 시각 통과로 보고하지 않는다.
- 관련 회귀 소스와 절대 좌표 검사 후보:
  `output/pr-review/pr7600/issue7642-exam-regression-inventory.json`.
  원문 경로 매칭 결과이며 실제 검사 의미를 대조한 뒤 변경 대상을 정한다.
- 원본 HWP/HWPX는 기존 추적 파일을 사용한다. 새 대응 Print PDF와 대표 PNG는 사용자 요청대로
  증적 커밋에 포함한다. TSV·실행 로그·중간 JSON·진단 스크립트는 ignored output에만 보존한다.

## 검증 시작 시점의 실제 확인

- 입력 16개가 모두 저장소에 있으며 원본 SHA를 고정했다. HWP 11개와 HWPX 5개 모두
  저장 메타데이터가 2024 이전이어서 MCP engine 2020을 명시했다. 각 job의 실제 한컴
  11.0.0.9136·Print method 0·one-up 결과를 개별 확인한다.
- 관련 테스트 소스는 58개이며 절대 좌표 검사 후보가 있는 파일은 26개다.
  후보에는 클릭 입력·속성 API 검사도 섞이므로 실제 배치의 의미를 확인한 뒤 관계 검사로 바꾼다.
- 첫 Native `21_언어_기출_편집가능본.hwp`는 RHWP/PDF 모두 15쪽이다.
  최저 60.58931%이며 미달은 1쪽 한 곳이다. 다른 14쪽은 90% 이상이지만
  문서 전체 통과나 기존 회귀 변경의 선행 조건 충족으로 보고하지 않는다.
  `issue7642-exam-native/exam-01/exam-01/silhouette.tsv` 및 1쪽 신규 직접 비교 패널로 추적한다.
- 첫 결과를 근거로 회귀 허용치를 바꾸지 않았다. fresh WASM은 `302be66a1`의 새 build가 진행 중이다.

## Native 최초 전체 비교 완료

- source `302be66a1`, Native 실행 바이너리 SHA-256
  `35e1f9a798d0e3c20ad947b1f26c6f9264898e74d044f80199c14be9ac314abc`.
- Print 16개와 Native 16개·134쪽이 종료했다. source/PDF 해시와 Print 계약을 개별 확인했고
  대응 PDF를 [공개 증적 폴더](../../pdf/issue7642-exam-print/README.md)에 보존했다.
  HWP/HWPX 원본 16개는 이미 Git에서 추적 중이며 그대로 유지한다.
- Native는 10개 문서 전체 페이지가 90% 이상이고, 6개(`exam-01/02/04/05/06/07`)는 미달이다.
  각 최저값·미달 쪽은 위 README 표와 ignored TSV/`issue7642-exam-native/results.json`에 연결했다.
- 첫 시험지 1쪽과 `exam_math_8.hwp` 1쪽의 신규 compare를 직접 판독했다.
  첫 시험지의 각 안내문 bullet 기준선 차이는 첫 문단 약 7.8px, 뒤 문단 약 15.4px이며
  source 문단 앞 간격은 570HU다. 진단 관측값이며 회귀 좌표·보정 상수로 사용하지 않는다.
  종속 분기는 `height_cursor::vpos_adjust` → table item의 문단 앞 간격 →
  `layout_partial_paragraph`의 호스트 글줄 표시이며 실제 앞 간격 소비를 계속 확인한다.
- `exam_math_8.hwp`는 수식 CASES·분수·기준선과 표 외곽/뒤 문단 간격 차이를 직접 확인했다.
  PDF의 HYhwpEQ·Haansoft Batang·Type3 본문 face와 Native의 실제 표시를 구분한다.
  글꼴 문제라는 추정으로 표 외곽 차이를 면제하지 않는다.
- 중단 때 첫 fresh WASM 최적화가 종료돼 wrapper의 public 동기화가 완료되지 않았다.
  process handle 부재와 `Terminated` 로그를 확인한 뒤 같은 source에서 재시작했다.
  Rust 캐시 재사용 build는 0.22초 exit 0이며 현재 `wasm-opt` 완료를 기다린다.
  완료된 Native 12개는 보존하고 13~16만 이어 실행했다. 전체 Native runner exit는 따로 확인한다.

## 문단 테두리 규칙의 독립 대조 준비

- `exam_math_8.hwp`의 연결 문단은 같은 왼쪽/오른쪽 여백 4540/2260HU와
  테두리 간격 1133HU를 저장한다. Native는 단 전체 폭을 테두리로 사용하지만,
  Print의 왼쪽 외곽은 해당 문단 여백·테두리 간격과 대응하며 오른쪽은 단 우단과 대응한다.
  source `paragraph_layout`의 일률적인 full-column 박스 선택을 먼저 검토한다.
- `borderIgnoreMargin`은 bit 29에 파싱되지만 ResolvedParaStyle의 소비 경로가 없다.
  플래그 이름만으로 의미를 추정해 분기를 추가하지 않는다. 기존 템플릿에서 왼쪽/오른쪽 여백,
  테두리 간격, ignoreMargin 0/1의 12개 조합을 만들고 저장 LineSeg 없이 MCP Print로 대조한다.
  진단 입력·파라미터·job·PDF는 ignored `paragraph-border-mcp-probe/`에 보존한다.
  해당 진단이나 원본의 검증이 끝나기 전에 새 회귀 기대값을 추가하지 않는다.

## fresh WASM 최초 전체 비교 완료와 독립 테두리 관측

- `302be66a1`의 재시작 wrapper는 exit 0으로 완료됐다. pkg/Studio public의 JS와 WASM
  SHA-256이 각각 일치한다. 브라우저 exporter는 고정한 `wasm-302be66a1/`을 사용했다.
  `issue7642-fresh-wasm/results.json`의 18개 입력·158쪽이 모두 종료했고,
  시험지 16개·134쪽의 쪽수·최저값·미달 쪽은 Native 결과와 같다.
  나머지는 수식 진단 3쪽 최저 94.00834%, 원본 11월 21쪽 최저 41.15967%다.
  원본 11월은 18~21쪽이 미달이므로 완료나 회귀 변경을 선언하지 않는다.
- 12조합 진단의 MCP Print job은 `a0a3536c-5b7c-4666-86ef-4fa7df87261e`이며,
  한컴 `11.0.0.9136`, engine 2020, Print method 0, one-up, 1쪽이다.
  실제 Print는 ignoreMargin=0에서 문단 할당 영역(단 폭·앞뒤 간격 포함),
  ignoreMargin=1에서 문단 좌우 여백과 마지막 줄 뒤 간격을 제외한 영역에 테두리를 표시한다.
  두 경로 모두 border offset만큼 외곽을 확장한다. 플래그 이름에서 추측한 규칙이 아니다.
- 생산/소비 경로는 `ParaShape.attr1 bit 29 → ResolvedParaStyle → paragraph_layout의
  실제 줄 점유·문단 흐름 끝 → para_border_ranges → border_rendering`이다.
  현재 resolver는 bit 29를 전달하지 않으며 paragraph_layout은 단 전체 폭을 일률 적용한다.
  보정은 실제 줄 점유와 문단 할당 영역을 구분하고 Square wrap override·연결 그룹·쪽 분할의
  소유 범위를 유지한다. 본문 흐름 높이를 paint에 맞춰 덮어쓰지 않는다.
- 수동 작성한 HWPX 진단은 Native와 Print의 전체 원점에 별도 차이가 있어 최초 실루엣이
  9.64511%다. 해당 입력과 실패를 보존하며 실제 정상 저장 대조군을 추가 확인한다.
  12조합의 상대 변화 관측을 원점까지 통과한 증거로 확대하지 않는다. 공식 fixture나 기대값은
  원본과 대조군의 시각 선행 조건 충족 전 추가하지 않는다.

## 문단 테두리 공통 결과 보정의 첫 Native 검증

- bit 29를 resolver에서 보존하고 `paragraph_border_box`가 실제 첫/마지막 줄 상자와 문단 흐름 끝을
  소비하도록 변경했다. 가로 여백·테두리 offset은 해당 플래그에 따라 적용한다. 문단 뒤 간격은
  동일한 `paragraph_flow_bottom`을 테두리와 반환 커서에서 소비한다. 연결 그룹·쪽 분할의 위아래
  offset 적용과 Square wrap의 개체 포함 override는 기존 소유 범위를 유지한다.
- Native build는 exit 0, 3분 47초다. 고정 바이너리 SHA-256은
  `e9458993d931357a155e7467b7833dfd6f67a5fc738d44aaec90ff542d9c9177`이다.
  source patch·추가 파일 SHA는 ignored `issue7642-border-build-provenance.json`에 기록했다.
- 수동 HWPX를 한컴 정상 저장 HWP로 만든 독립 대조군 job은
  `b8aa08ba-0f1d-41ba-9097-de863a311950`, 해당 HWP의 Print job은
  `d6094aa6-2c3a-45d9-ad52-c418fc92018e`다. 실제 저장된 용지/여백은 수동 입력과 달라
  두 입력을 구분한다. 정상 저장 대조군은 같은 HWP·Print에서 수정 전 61.00107% →
  수정 후 99.89713%이며 12개 frame·줄 시작·뒤 간격을 compare로 직접 확인했다.
  이는 원점이 어긋난 수동 HWPX 원본의 통과 증거가 아니다.
- `exam_math_8.hwp` 전체 1쪽은 67.21742% → 82.03312%다. 테두리 외곽과 마지막 문단의
  아래 간격은 Print와 대응하지만 CASES·첨자·기호 간격 차이가 남았다. 90% 선행 조건을
  충족하지 못했으므로 새 회귀나 기존 기대값 변경을 하지 않았다.
- 시험지 16개 전체를 이 고정 바이너리로 재실행 중이다. fresh WASM·최종 lint·관계 회귀·PR은
  아직 완료 전이다. 최초 원본·실패 TSV는 그대로 보존한다.

## 수식·호스트 간격·머리말의 후속 보정

- 테두리 보정은 `30fa6e963dd4218f4292367ccf5e985bd4c6618b`로 보존했다.
  이후 코드는 이 커밋 위의 변경이며 source patch·새 파일·바이너리 SHA를
  ignored `issue7642-header-and-consumption-provenance.json`에 고정했다.
  최신 Native 바이너리 SHA-256은
  `a22099c767c727659d0d3a4fb499a091848a4ece6ed2569fab8feb94c091bdba`다.
- 원본 과학 시험지의 `rmCH_3}COOH`에서 소유하지 않은 닫는 중괄호가 뒤의 `COOH`를
  파싱에서 누락시켰다. 그룹 소유가 없는 닫는 중괄호만 제외하고 뒤 내용을 보존한다.
  `LEFT/RIGHT`의 표시 구분자는 보호한다. 과학 4쪽은 89.91304% → 90.42185%이며,
  수식 96개·3쪽 대조군은 98.52323/94.00834/96.98509%로 변화가 없었다.
- 과학 문단 117의 명시적 Center 정렬이 본문 수식 단독 줄의 일률적인 offset 0에 의해
  무시됐다. Center의 가용 폭 정렬을 실제 수식 배치에 적용해 4쪽은 91.04705%가 됐다.
  Distribute 경로의 기존 조건은 이번 보정에 포함하지 않았으며 eq-002 HWP 대조군은
  92.51376%로 유지됐다. 저장 위치가 수동 지정된 합성 입력의 별도 계약은 미검증이다.
- 같은 문단의 PartialParagraph와 비글자 표가 문단 앞뒤 간격·마지막 줄 간격을 두 번
  소비했다. 처음 시도한 `PageItem이 존재하면 이미 소비했다`는 조건은 Square wrap의
  흐름을 전진시키지 않는 PartialParagraph를 오판해 언어 14쪽과 국어 14/17쪽에 회귀를
  만들었다. 이 시도와 실패 출력은 `issue7642-exam-host-spacing/`에 보존했고 조건을 제거했다.
  실제 paragraph 배치가 소비한 앞 간격·뒤 간격·마지막 줄 간격을 각각 기록하며,
  같은 구역·문단·단에서 표 배치가 해당 결과를 소비하도록 변경했다. 셀/머리말의 표시를
  본문 흐름 소비로 취급하지 않는다. 두 상태를 구분한 첫 보정에서 언어 14쪽 98.58707%,
  국어 14/17쪽 96.49499/90.61634%가 복구됐고 수학 1쪽은 84.09015%다.
  뒤 간격과 마지막 줄 간격을 분리한 최신 결과는 전체 재실행으로 확인한다.
- 머리말 한 문단에 표와 비글자 도형이 함께 있으면 기존 배타 분기가 도형을 누락시켰다.
  같은 문단의 원점을 보존해 도형을 배치하고, 실제 호출 메서드를
  `header_footer_layout.rs`로 분리했다. 혼재한 글자처럼 취급 도형은 이번 변경 대상이 아니다.
  과학 4쪽의 `4 (화학 I)` 표시가 복구됐으며 최신 Native 전쪽은
  93.89083/95.04297/95.23493/91.54535%다. 같은 캡처의
  `science-header-panels/review/review_004.png`를 사용자에게 직접 표시했다.
- 90% 이상인 입력의 기존 px 검사도 관계 검사로 바꾸라는 사용자 지시를 재확인했다.
  최신 Native/fresh WASM 전체 페이지·확대 판독을 먼저 확인한 뒤 원본 문단·표·셀 소속으로
  대상을 찾고 포함·정렬·앞뒤 순서·누락·중복을 검사한다. 고정 폭으로 그림을 선택하는 코드도
  변경 대상이다. 미달 입력의 원본 검사나 기대값을 먼저 바꾸지 않는다.
- 최신 Native 시험지 16개·134쪽과 root wrapper의 fresh WASM을 실행 중이다.
  현재 fresh WASM·관계 검사 변경·최종 lint·전체 회귀·새 PR은 완료 전이다.

## 수치 통과 뒤 직접 판독에서 찾은 그림 정렬 결함

- 최신 Native 16개·134쪽 runner는 exit 0으로 종료했다. 입력별 gate는 별도이며
  11개 문서가 전쪽 90% 이상, 5개(`exam-01/02/04/05/06`)는 미달이다.
  세부 결과는 ignored `issue7642-exam-header-and-consumption/results.json`에 있다.
  전체 runner의 정상 종료를 PR 승인으로 해석하지 않는다.
- 국어 원본 전체 20쪽은 최저 90.61634%, 5쪽은 94.54030%였지만, 사용자에게 표시한
  `kor-after-panels/review/review_005.png`에서 문단 145 셀[5]의 그림 세 장이 Print보다
  위에 놓인 결함을 직접 확인했다. 기존 #6632의 정확한 px 검사를 새 좌표로 갱신하지 않았다.
- 같은 내용의 HWPX 원문은 그림 문단 `paraPr 38`의 `align vertical="CENTER"`,
  해당 저장 LineSeg의 height/textheight 5952HU와 baseline 2976HU를 선언한다.
  문단 앞 도형 줄도 `paraPr 37`, CENTER, height 1984HU와 baseline 992HU다.
  [한컴 HWP 5.0 사양 표 44](https://cdn.hancom.com/link/docs/한글문서파일형식_5.0_revision1.3.pdf)는
  attr1 bits 20..21을 글꼴 기준/위/가운데/아래 정렬로 정의한다.
- 기존 `inline_picture_top`은 모든 그림 상자에 글꼴 기준 어센트 0.85를 적용했다.
  CENTER의 저장 기준선은 상자의 중앙이므로 그림이 상자 높이의 0.35만큼 위로 이동했다.
  이는 단순히 문단의 다음 줄이 당겨진 #6632의 원래 원인과 구분되는 paint 정렬 결함이다.
  resolver가 세로 정렬을 보존하고 일반 run·미배치 TAC·빈 run 세 호출 경로가 같은
  helper에서 명시적인 정렬점을 소비하도록 보정한다. 줄 높이·저장 LineSeg를 그림 좌표에
  맞춰 덮어쓰거나 출력 원점을 clamp하지 않는다. 새 보정의 Native/fresh WASM은 재실행한다.
- 독립 정렬 대조군은 동일 줄의 서로 다른 높이 그림을 글꼴 기준/위/가운데/아래로 지정해
  LineSeg 없는 HWPX를 만든 뒤 한컴 정상 저장 HWP와 그 Print를 별도로 산출한다.
  ignored `inline-picture-valign-mcp-probe/`에 입력 생성 방식·해시·원본 BinData·job을 보존한다.
  첫 진단의 삭제된 section 참조 오류는 원본 파일과 실패 로그로 남겼고,
  section count와 manifest/spine 참조를 정리한 package는 Native info에서 1쪽으로 열렸다.
  수동 진단 자체의 정상 저장본 대조 결과를 국어 원본 통과로 대신하지 않는다.

## 명시적 세로 정렬 보정의 Native 재확인

- 새 Native build는 exit 0, 4분 23초다. 고정 바이너리 SHA-256은
  `23963dde075a55654e4c4d86e6a19b3984c6352d409fdc34daded9066daca115`이며
  source는 ignored `issue7642-picture-valign-provenance.json`에 연결했다.
- 국어 5쪽은 94.54030% → 98.49149%이며, 그림 세 장이 자신의 그림 줄과 셀 안에 놓이고
  앞 도형 줄 뒤에 배치됨을 최신 review로 확인했다. 전쪽 20쪽 최저는 90.61634%다.
  `kor-picture-valign-panels/review/review_005.png`를 사용자에게 직접 표시했다.
  동일 원본의 수정 전/후 실제 렌더 트리로 실행한 줄 내부 포함 진단은 그림 세 장 모두
  FAIL → PASS다(`kor-picture-line-relation-diagnostic.json`). 정식 Rust 검사 실행과는 구분한다.
- 같은 세로 정렬 누락이 영어 첫쪽 그림도 위로 당겼다. 영어 전쪽 8쪽은
  최저 81.58687% → 95.63340%이며, 1쪽은 98.05928%다. 해당 최신 review도 직접 표시했다.
  이 보정은 파일명·폭·원점 상수 없이 원본 문단 정렬을 같은 그림 배치 helper에 전달한다.
- 국어 #6632와 사회 #6621의 px 선택/기대값을 줄·셀 소속, 내용 순서·보존·포함·정렬,
  원본 상자 선언과의 상대 관계로 바꿀 초안을 ignored output에 보존했다.
  현재 생산 코드를 바꾼 최신 fresh WASM의 전쪽·확대 판독이 끝나기 전 정식 회귀에는 반영하지 않는다.
  영어 #6630·#6569·#775의 같은 px 선택도 이제 전쪽 수치 통과 후보라 관계 검사 대상에 포함한다.
- 첫 머리말/소비 보정의 fresh WASM과 세로 정렬 보정의 fresh WASM은 다른 package다.
  전자는 `wasm-header-and-consumption/`이며 WASM SHA
  `ac14400bd9a929d8ddb4f87443f554086899b5b931795a7210f0de96dcebfa0c`다.
  후자는 세로 정렬·마지막 줄 간격 소비 보정까지 포함해 새로 빌드 중이다. 전자의 점수를
  후자에 적용하지 않는다. 최종 package의 pkg/Studio public 동기화와 브라우저 비교를 다시 확인한다.

## 전체 회귀 실행과 컴파일 누락 보정 — 2026-10-08

- 새 PR 번호는 아직 없다. 전체 검증을 우선하라는 지시에 따라 아래 명령을 끝까지 실행했다.
  `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`.
- 첫 실행은 기존 `equation_command_goldens::dump_layout`이 추가된 `FontGlyph`를 처리하지 못해
  E0004, exit 101로 컴파일 실패했다. helper가 글리프의 텍스트·선택 글꼴·메트릭을 기록하도록
  명시적인 match arm을 추가했다. golden·assertion·허용치 변경은 없다.
- 재실행은 컴파일 통과 후 **10,524건 실행, 10,464건 통과, 60건 실패, 50건 skipped**,
  실행 1,226.193초, exit 100으로 종료했다. 느린 검사도 종료 결과를 확인했다.
- 실행 시작 HEAD는 `a08f27f86a582a623907be779e9d71ddc8cfc653`, 고정 base는
  `b3c3047db575d146dff9d39098e6de98c4630b3e`다. 빌드 중 문서-only commit `3d3418221`이
  추가됐고 재실행에는 위 helper 보정이 포함됐다. 생산 Rust와 fixture는 변경되지 않았다.
- 실패 로그와 실제 assertion 대상·fixture 선언을 대조한 입력은 **HWP 18개·HWPX 6개**다.
  합성 수식·정렬 검사 6건은 직접 원본 파일이 없으므로 실물 문서로 추정하지 않는다.
  완료 로그·입력 대응 TSV·해시는 ignored
  `output/pr-review/issue7642/full-regression-20261008/`에 보존한다.
- 실패 입력의 최신 Native/fresh WASM 전쪽과 독립 한컴 Print를 다시 대조한다. 90% 미만은
  출력을 먼저 개선하고, 90% 이상 및 확대 판독 후 필요한 px 검사만 관계 검사로 바꾼다.
  본문 넘침·겹침·쪽수의 실패도 같은 원본으로 확인하며 baseline을 일괄 갱신하지 않는다.
- self PR 절차는 `3d3418221`에서 **PR 생성 → 정확한 code head CI 성공 → PR review·오늘할일
  문서-only trailing commit → trailing head CI 확인 → merge**로 수정했다. PR 번호 기반 기록과
  오늘할일은 앞으로 이 CI 완료 순서를 따른다. 전체 회귀 60건 실패는 준비 완료가 아니다.
- 최신 fresh WASM build는 exit 0, 17분 08초다. 고정 package는 ignored `wasm-a08f27f86/`,
  WASM SHA-256 `4e86188cfd4eceede8dd4a45ba4bc70b4aced39ccbbc0733174354e9d295b429`다.
  root pkg와 Studio public의 JS·WASM 해시가 각각 일치한다. package 생성과 브라우저 Sweep 통과를
  구분하며, 현재 실패 입력 전쪽 Sweep은 다음 보정 단계에서 실행한다.

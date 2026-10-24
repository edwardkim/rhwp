# 보정30 — 저장 문단의 로컬 간격과 출력 축 끝점 소유

Issue: #7445, 관련 #7009/#7751/#7390. 부모 `9542e4dcdf7ceb9fd733a873493afcd84a2a838a` + 아래3개 제품 파일의 후보1088/fresh1091이다. 이 증거는 단계별 공통 보정이며 전체 브랜치 제출·full nextest·모든 소비 경로의 완료 선언이 아니다.

## 원 해결과 이번 보정

기존 `d18ef1200cd64446f5232cb7d79956f804bc1841`은 닫힌 저장 문단 끝으로 후속 표 예산과 배치를 연결했고, `a45df0caf40fa432231ef1d083d96a7e206ddae0`은 TAC 개체 저장 줄까지 연결했다. 현행 코드는 절대 base가 없을 때 모든 닫힌 후행 간격을 거절했고, plain/empty 문단 뒤 일반 본문·TAC 표는 연결하지 않았다.

독립 한컴2024 편람 원본/Print383쪽과 원 등록 #7009 `e5984786bc`의 job을 재확인했다. 구역7 p410은 LH1250+LS748HU 및 뒤1000HU가 다음 vpos14044에서 닫힌다. 실제 전진은748HU를 빠뜨리고 뒤에서 lazybase748을 만들었다. p413 빈 줄과 p414 TAC 표의 원문 차이는1600HU(21.333333CSSpx)이나 실제 차이는852HU(11.36px)였다. 제목 표와 후속 본문이 약9.973333px 앞당겨진 실제242쪽 출력과 일치한다. 독립 PDF·저장 값을 바꾸거나 좌표를 clamp하지 않았다.

`StoredParagraphSuccessorFlow`의 인증된 로컬 구간을 현재 수용 커서에서 소비한다. 확정 base가 있는 경우 기존 절대 축 변환·되감기 금지를 유지한다. `StoredParagraphFlowProjection`은 content_bottom/advance_end와 paint_end를 분리한다. base가 없는 로컬 전진은 절대 paint 끝점으로 발행하지 않으며, 전역 vpos 사다리가 dirty이면 저장 축을 재사용하지 않는다. 원문 분할·합성/재조판·간격 닫힘 불일치 guard도 유지한다.

생산 `float_placement::stored_paragraph_successor_flow`→같은 project→`typeset/paragraph/content::stored_successor_flow`→일반 fit/atomic overflow의 content_bottom·advance_end 소비→확정 paint_end만 `record_stored_paragraph_flow_end`→`layout.rs`의 col_area.y+end 소비다. None 로컬 전진을 이 마지막 절대 좌표로 덮어쓰지 않는다. 다단의 기존 인증된 TAC 결과는 같은 반환형의 Some(paint_end)로 유지한다. 표 continuation이 별도로 기록하는 수용 끝점과 내용 컷/rowspan 계산은 변경하지 않았다.

## 전쪽으로 발견한 중간 후보 회귀와 재복구

초기1080은 편람 간격을 복구했지만1083의 Native 전640쪽에서 PrEP70(84.85921%)·RowBreak HWP14(83.34699%)가 새로 미달했다. 같은 source-axis로 둘을 해석한 오류다. PrEP는 dirty 사다리인데 저장 절대 끝을 게시했고, RowBreak는 base=None 로컬 끝이 paint의 절대 끝으로 넘어갔다. 실제 advance 길이는 이전과 같아도 마지막 소비자가 달라졌다. 1087의 전후 실제 flow/paint 로그로 확인하고 위 타입·유효성 계약을 보정했다. 1083 WASM은144쪽 중단으로 전쪽 통과 근거가 아니다.

최종1088/fresh1091의 PrEP70은 양쪽100%, RowBreak HWP14는99.90872/99.90810%다. Native 두 PNG는 기존1055/1058의 정상 PNG와 바이트 동일하다. 검사 제거·문서별 예외·기준 변경으로 감추지 않았다.

## 전쪽 TSV와 전후 감사

96DPI/Print/같은 compat, canonical PDFium smooth_rgb_v1, raw2px90%로 Native/fresh 각각 primary637쪽을 다시 출력했다. 수도시설3쪽은 기존 미달 대조군으로 별도 표시했다. batch16/capture총8/배타 lock과 source SHA를 유지하고 완료 생성SVG만 hash 기록 후 정리했다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 90% 미만 |
|---|---:|---:|---:|---|
| handbook | 383 | 92.33793% | 92.35482% | [] / [] |
| science | 4 | 96.14438% | 96.06833% | [] / [] |
| legal | 74 | 96.05600% | 96.01774% | [] / [] |
| prep | 140 | 94.75150% | 94.22354% | [] / [] |
| rowbreak-hwpx | 18 | 92.85921% | 92.82532% | [] / [] |
| rowbreak-hwp | 18 | 93.07060% | 92.96483% | [] / [] |
| water-control | 3 | 71.85405% | 72.05077% | [3] / [3] |


`after/<backend>/*.tsv`와 `before/<backend>/*.tsv`는 정본 원문을 바이트 그대로 보존했다. 모든 쪽의 순번·쪽수·누락/중복0·PDF PNG SHA·후보 PNG SHA를 확인했다. 1280쌍에서0.1%p 이상 후퇴0, 최저 delta는 편람 Native -0.02980%p다. 편람 before1070→after1092에는 Stage29 프로그램 선택 개선도 포함되므로 Stage30 단독 효과로 전부 부르지 않는다. 편람·나머지5조건은 설치 글꼴 hash 집합도 같다. 과학의 과거772/775에는 오픈소스 폴더5파일이 원장상 없지만 기존 파일 변경/제거0이며 실제8개 출력 PNG가 모두 현재와 바이트 동일하다.

법령·과학·RowBreak HWP·수도시설 출력 PNG는 이전과 같다. PrEP38·RowBreak HWPX15는 양 backend에서 각1쪽 PNG가 바뀌고 raw값은 후퇴하지 않았다. review로 표/뒤 본문·번호·내용 순서가 유지됨을 확인했다. 해당 출력을 완전 픽셀 동일로 보고하지 않는다. 법령2/13의 #8064 원 하한 미달은 이전과 같으며 이번 전후 무후퇴가 그 하한의 통과를 대신하지 않는다.


| 문서 | Native 이전 | Native 후보 | WASM 이전 | WASM 후보 |
|---|---|---|---|---|
| handbook | [TSV](before/native/handbook.tsv) | [TSV](after/native/handbook.tsv) | [TSV](before/wasm/handbook.tsv) | [TSV](after/wasm/handbook.tsv) |
| science | [TSV](before/native/science.tsv) | [TSV](after/native/science.tsv) | [TSV](before/wasm/science.tsv) | [TSV](after/wasm/science.tsv) |
| legal | [TSV](before/native/legal.tsv) | [TSV](after/native/legal.tsv) | [TSV](before/wasm/legal.tsv) | [TSV](after/wasm/legal.tsv) |
| prep | [TSV](before/native/prep.tsv) | [TSV](after/native/prep.tsv) | [TSV](before/wasm/prep.tsv) | [TSV](after/wasm/prep.tsv) |
| rowbreak-hwpx | [TSV](before/native/rowbreak-hwpx.tsv) | [TSV](after/native/rowbreak-hwpx.tsv) | [TSV](before/wasm/rowbreak-hwpx.tsv) | [TSV](after/wasm/rowbreak-hwpx.tsv) |
| rowbreak-hwp | [TSV](before/native/rowbreak-hwp.tsv) | [TSV](after/native/rowbreak-hwp.tsv) | [TSV](before/wasm/rowbreak-hwp.tsv) | [TSV](after/wasm/rowbreak-hwp.tsv) |
| water-control | [TSV](before/native/water-control.tsv) | [TSV](after/native/water-control.tsv) | [TSV](before/wasm/water-control.tsv) | [TSV](after/wasm/water-control.tsv) |

## 정식 실제 출력 회귀

`issue_7445_stored_paragraph_flow_projection`은4개 검사다. 동일 최종 source SHA `9a29381d09204ea3b7867b161492f79c03425ba635d8fdd250296cf0f2bab4ab`: Stage29 frozen1075에서2PASS/2FAIL, 잘못된 중간1080에서2PASS/2FAIL, 최종1088에서4PASS다. 서로 다른 실패 계급을 같은 검사로 확인했다.

- 편람 p413빈 줄→p414표→p415본문의 저장 원문 간격·소유·383쪽을 관계로 검사한다.
- 편람242 Native Print의 두 제목 소유자가 유일하고 독립 PDF F4/cm0.12/tm(812,3301)/(1085,3301)에 따른 x129.92/173.6 및 baseline528.16CSSpx ±2를 검사한다.
- PrEP70의 표 다음 원문 문단38 텍스트 순서·누락·중복 및 첫 어절 소유, 독립 PDF cm0.12/tm659,5118의 x105.44/baseline818.88 ±2를 검사한다.
- RowBreak HWP14의 표 인용 문단을 실제 textLine 소유자로 찾고 원문 전체 순서·누락·중복을 검사한다. 중첩 표에서 같은 stableSourceKey가 반복될 수 있으므로 문자열 prefix만으로 전체를 합치지 않는다. 독립 PDF cm0.12/tm2346,3317의 x375.36/baseline530.72 ±2를 검사한다.

전쪽90% 선행 조건을 충족했고, 고정하는242/70/14쪽은 각각 Native/fresh98.87729/98.89112%,100/100%,99.90872/99.90810%다. 실제 review/overlay를 열어 제목·표·뒤 문단·부호와 간격을 확인했다. 기준 PDF에서 얻은 기대값이며 후보 bbox에서 역산하지 않는다. 별도 source-unit test attrs는 늘리지 않았다.

기존 #7751 Print 검사는 crop이 자원에 소비된 뒤 원본 crop 조건을 다시 요구하는 잘못된 선택자를 교정했다. 원 해결 `8291d5cdd2`와 원본 source355 전체 그림/같은 자원의 잘린 머리말을 대조했다. source_node_id로 전체 바탕 그림 하나만 연결하며 종이 경계·본문 WMF·동적 머리말 assertion을 유지한다. 수정한2개 검사는 frozen1075/현재1088 모두2PASS라서 제품 FAIL→PASS로 과장하지 않는다. 대응130쪽98.96134/98.79901%, 전383쪽90% 및 직접 판독을 확인하고 교정했다.

## 빌드와 실제 초기 소비

Native1088 CLI `1a1fc725f3b19c9225bf93a04405146caaf3c4d6b9da76829eb30414ba1b5f6a`, lib `e0ce98b948cedc1f2c792e3f01a32ccc3a0dadb4f5d96c367798f1a3dd41f3f5`. fresh1091은 locked wrapper `--target web --out-dir pkg --no-opt` jobs8의 Mac 로컬 대체 빌드이며 Docker 최적화 통과가 아니다. JS `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`, WASM `686ef1dcfd4b4a0c98cd7c4ba4f654e20c4ae893297de8316b4b9fcbc7bc511e`. root pkg/rhwp-studio public/frozen pkg hash가 같다. 실제 Studio7736 초기 JS/WASM 응답200과 frozen build 일치, wasm 초기화true/errors0을1098에서 확인했다. 이 결과는 문서 paint·CanvasKit 준비·Actual36 수용을 대신하지 않는다. 기존 서버를 보존했다.

- `src/renderer/float_placement.rs`: `eaa55bf7e588ad2d1d90895916db4b4975de3f21ba01ccd661ad173bf5bcdb43`
- `src/renderer/typeset/paragraph.rs`: `7cfbafada63e9a189370001c4d8b0426767b1b7365bd944174a76247a124e95b`
- `src/renderer/typeset/paragraph/content.rs`: `6fd7aa95dfc535a04a3f44893a523340e620127abeaed826c8af6e47ee47fefb`

정식 자동 suite002에서 새4검사4PASS(실행1.233초), suite012에서 기존#7751의2검사2PASS(1.238초)다. 필터로 제외된216/248은 전체 회귀 skip으로 합산하지 않는다. --prepare/manifest·unit-tier(base2e3)·fmt/diff·workspace build·workspace all-target strict Clippy가 모두 exit0이다. 제품 Native/WASM strict Clippy도1096에서 각각 exit0이다. 기존 직접 계약41PASS/0FAIL(1093)과 최신 full nextest는 구분한다. Native Skia3종도1104에서 모두 exit0이다. --lib는 root3859PASS/12ignored + 다른 libs182PASS, 실제 PNG placeholder2PASS, 직접 PDF export14PASS다. 이 단계의 full integration nextest는 아직 실행하지 않았으며 Native Skia lib·focused suite와 구분한다. `.log`·JSON·진단 Rust는 ignored output에만 보존한다.

## 대표 그림

### handbook 242쪽

중간/변경 전 Native:

![before review](handbook-242-before-native-review.png)
![before overlay](handbook-242-before-native-overlay.png)

최종 Native:

![after Native review](handbook-242-after-native-review.png)
![after Native overlay](handbook-242-after-native-overlay.png)

최종 fresh WASM:

![after WASM review](handbook-242-after-wasm-review.png)
![after WASM overlay](handbook-242-after-wasm-overlay.png)

### prep 70쪽

중간/변경 전 Native:

![before review](prep-070-before-native-review.png)
![before overlay](prep-070-before-native-overlay.png)

최종 Native:

![after Native review](prep-070-after-native-review.png)
![after Native overlay](prep-070-after-native-overlay.png)

최종 fresh WASM:

![after WASM review](prep-070-after-wasm-review.png)
![after WASM overlay](prep-070-after-wasm-overlay.png)

### rowbreak-hwp 14쪽

중간/변경 전 Native:

![before review](rowbreak-hwp-014-before-native-review.png)
![before overlay](rowbreak-hwp-014-before-native-overlay.png)

최종 Native:

![after Native review](rowbreak-hwp-014-after-native-review.png)
![after Native overlay](rowbreak-hwp-014-after-native-overlay.png)

최종 fresh WASM:

![after WASM review](rowbreak-hwp-014-after-wasm-review.png)
![after WASM overlay](rowbreak-hwp-014-after-wasm-overlay.png)

### #7751 원본 그림 소유 — 편람130쪽

![Native130 review](handbook-130-after-native-review.png)
![Native130 overlay](handbook-130-after-native-overlay.png)
![WASM130 review](handbook-130-after-wasm-review.png)
![WASM130 overlay](handbook-130-after-wasm-overlay.png)

## 정확한 입력·기준

| 키 | 원본 / SHA-256 | 독립 Print PDF / SHA-256 |
|---|---|---|
| handbook | `samples/2025 행정업무운영 편람(최종).hwp`<br>`40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f` | `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`<br>`3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593` |
| science | `samples/exam_science.hwp`<br>`22d29786a80d68a9b2ad9294c2dab4915e0eced941e790e37390b14312b8b6a8` | `pdf/issue7642-exam-print/exam-07-exam_science-hwp-print-2020.pdf`<br>`1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14` |
| legal | `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx`<br>`5090e9d20ce75814febeb4b79e106ea6abfd2546e511f229900d137d42623617` | `pdf/issue7642-legal-table-origin/78494-virtual-convergence-industry-decree-2020.pdf`<br>`674ef63b51adc926c7c89f530bd142a45bb64cb655a7117bc20285d7a6e5a8d2` |
| prep | `samples/issue2006/1790387_prep_final_report.hwpx`<br>`c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f` | `pdf/issue2006/1790387_prep_final_report-current-print-2024.pdf`<br>`9580bb332813b70f27a367e79ad735b238bcd716ff51b849e716f7ddd1e60af0` |
| rowbreak-hwpx | `samples/rowbreak-problem-pages.hwpx`<br>`4ccc321e7eb146f28b768654502a128ba1ae408238fd1ed37d51e963732eb5b0` | `pdf/rowbreak-problem-pages-hwpx-2020.pdf`<br>`e3dc7bb21707f92e42e83808023bf155f62475fc4eb0e0498709d03986c5afe5` |
| rowbreak-hwp | `samples/rowbreak-problem-pages.hwp`<br>`10b6ab6548610e18c82ba78a1c844a00107fedbb28c195cb05e6fd20626d33ed` | `pdf/rowbreak-problem-pages-hwp-2024.pdf`<br>`2c49bda9cc21dc8b93b554d2607da241dd7e894657eb308ed823e7d44654e85c` |
| water-control | `samples/issue6776/36367506-water-facility-approval.hwpx`<br>`62ee54efc7fd9f5fd06cbfdb5843edcdbf84074b05ede5571c15078d4f1b14ab` | `pdf/36367506-water-facility-approval-2020.pdf`<br>`fe82e237046659fdd770811dbd30c8f199d155d9737f1a8d0732f19f2049deda` |

## 남은 제출 필수

법령 #8064 Native-only 하한·필요한 전체 registry 보존, 과학 본문/수식 기준선 잔여, 현재 브랜치 전체 범위/fixed-devel TSV·full nextest·Native Skia3종 등 나머지 필수 gate·Actual36을 계속한다. 이번637쪽 수치를 전체 브랜치 완료로 쓰지 않는다. PR 전 upstream head 확인·필요한 안전 리베이스 및 재검증, 사용자 새 PR 승인 절차도 남아 있다. 지침·원격 변경은 하지 않았다.

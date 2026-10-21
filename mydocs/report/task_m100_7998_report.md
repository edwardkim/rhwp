# #7998 PDF 의미 표·문단 구현 결과

상태: **표 인식·속도 수용, 추가 글꼴 자원 보정 로컬 검증 완료 — 전체 P5·PR 제출 보류**.

2026-10-21 작업지시자가 “이제 기다릴만하네요”로 속도를 수용했다.
이어 보고한4쪽 `font_metric.pdf`는 기준 devel과 보정 전 후보 모두 기본10만 font metric
한도로 거절했다. 정상 CJK3개 face의 CID 폭/TrueType 매핑 때문에 첫 쪽에서 발생했으며,
제품 요청의 유한 상한만100만으로 조정하고 raw decoder·선예약·decoded/scan·호스트 명시
저한도는 유지했다. 정식 자원124건 PASS, 새 Native4쪽 최저99.88105%이며
기존162쪽 문서의 HWPX bytes도 보존됐다. 새 fresh WASM4쪽 최저99.80979%, 실제 Studio
열기2.146초·HWP/HWPX 저장/재열기4쪽·표시 문자 digest 보존을 확인했다.
기존162쪽은17.707초·첫274셀을 유지했다. 최신 WASM은 Studio7793에 반영했고,
비교 자료는 `E:\tmp\issue7998-font-metric-20261021\review\index.html`이다.
이 원본·식별 가능한 이미지·실제 저장본은 개인 정보가 있어 로컬에만 보존한다.

2026-10-21 메인테이너는 표 인식 해결을 확인했고 속도 개선을 요청했다. 같은 원본에서
표/셀·문자/저장 계약을 유지하며 정확한 메트릭 재사용과 중복 인식 결과 복사를 줄였다.
같은162쪽 원본의 Native 가져오기·저장은24.13/25.08초 →19.90/18.45초,
동일 headless Chrome152 첫 표시는28.27초 →17.82초였다. 실제 Table162/Cell54,938과
HWPX·진단, 실제 WASM의 양 저장 파일이 이전 복구본과 같고 실제 셀 입력·undo·재열기도 통과했다.
앞선 성능 보정 제품의 집중47건·Native/fresh WASM 각180쪽을 확인했으며 원본162쪽 PNG는 양쪽 모두 이전과 같다.
재시험: [Studio7793](http://localhost:7793/?review=issue7998-performance-20261021&lang=ko),
원본 `/mnt/e/tmp/reousrce_limit.pdf`, 자료 `E:\tmp\issue7998-perf-20261021\review\index.html`.

추가 기회 승인 뒤 같은 `reousrce_limit.pdf`에서 실제 Table162/Cell54,938을 생성하고,
Native 전체 가져오기·저장을76.4→24.1초로 보정했다. 첫 복구본 WASM의 Windows 첫 표시는25.1초였다.
글자 속성 중복과 저장 후 뒤쪽 스타일 유실도 보정했다. 전쪽 Native/fresh WASM 최저는
99.56743%/99.56279%이며 실제 셀 키보드 입력·undo·양 저장·재열기와 집중 회귀47건을 확인했다.
표 인식 복구는 수용됐지만 devel5.7초보다 느리다(해당 devel은 의미 표0개). 전체 P5와 새
162쪽 저장본의 한컴 열기·인쇄는 미완료다. 정확한 dirty source·runtime·명령·측정 경계와
잘못된 초기 진단의 제외 사유는 [stage37 재시험 기록](../working/task_m100_7998_stage37.md#재시험용-복구본--메인테이너-수용-대기)을 따른다.

2026-10-20 메인테이너는 이전 devel 대비 종합 개선을 확인하지 못했고,
`reousrce_limit.pdf`의 과도한 지연과 전체 표 인식 실패를 보고했다. 아래169쪽/기존 대조군
결과를 실사용 개선이나 전체 목표 달성의 근거로 승격하지 않는다. 이 실제 입력을 같은 조건의
devel/후보에서 재현하며, 최종 전체 검증은 중단했다. 후속 이슈 등록 사실도 이 결함의 해결을 대신하지 않는다.

이전 마감 지시와 범위 기록: 2026-10-20 작업지시자가 현재 구현을
마감하고 남은 기능은 후속 이슈로 처리하도록 결정했다. PR 생성·병합·이슈 종료는 아직 수행하지 않았다.

## 완료 범위와 후속

근거가 확인된 PDF 지역을 실제 Table/Cell과 Paragraph/LineSeg로 소비하고, 셀 테두리·채우기·
검증한 병합/중첩·행열 편집·문자 편집과 HWP/HWPX 왕복을 연결했다. 원본 공백과 추론한 위치 이동을
구분한다. 불확정 구조·표현할 수 없는 메트릭은 원문을 단일 소유로 보존하고 이유를 진단한다.
전체 PDF 작성 객체·원본 Excel의 정확한 셀 수를 복원했다고 주장하지 않는다.

미완료 구현은 [백로그](../plans/task_m100_7998_backlog.md)에 보존하고 등록했다.

| 후속 | 남은 문제 |
|---|---|
| [#8106](https://github.com/edwardkim/rhwp/issues/8106) | 내용 증가에 맞춘 보존 테두리 격자 |
| [#8107](https://github.com/edwardkim/rhwp/issues/8107) | 보고서6쪽 glyph 폭과 문단 승격 |
| [#8108](https://github.com/edwardkim/rhwp/issues/8108) | 보고서1·4쪽 자간과 문단 승격 |
| [#8109](https://github.com/edwardkim/rhwp/issues/8109) | Helvetica 저장·재열기·한컴 대체 글꼴 일관성 |
| [#8110](https://github.com/edwardkim/rhwp/issues/8110) | 의미 표의 가는 선·작은 글자 농도와 PDFium coverage |

기존 승인된 5pt 표준 셀 전체 채움, 비사각 clip의 rhwp 왕복 범위와 B19/B21–B23/B25의
후속·예외는 유지한다. 이를 해결이나 정상 시각 통과로 바꾸지 않는다. 이전 단계의 “필수 미완료”는
당시 범위이며 현재 사용자 지시로 범위만 갱신했다. 세부 경로·반례·직접 수용은 기존 stage 보고서를 따른다.

## 고정 제품과 통합

- code head: `bd707a7af055ebc047e8d360f079f215af5a6a09`
- 통합 devel: `5e2b9bfbf` (최종 통합 착수 시 fetch한 기준).
- Native/fresh WASM과 기준 producer의 source·파일 SHA: [producers.tsv](../pr/assets/issue_7998/final-p5/producers.tsv).
- 병합16파일 충돌의 보정과 셀 줄 폭4 HWPUNIT 규칙은 [stage37](../working/task_m100_7998_stage37.md)에 연결했다.
  초기/편집 재조판은 같은 소유자 폭을 같은 줄 격자로 발행하고 실제 문단 원점을 보존한다.
  행/쪽 분할·이어받기 높이는 이 통합 보정에서 바꾸지 않았다.

## 시각·구조·저장 검증

| 경로 | 고정 범위 | 결과 |
|---|---|---|
| Native | 158입력169쪽 | 누락·쪽수 차이0; 일반163쪽 최저92.17391%; 글꼴 예외6쪽 raw최저89.54211% |
| fresh Docker WASM | 같은169쪽 | 모두90% 이상; 최저90.70673%; 누락·쪽수 차이0 |
| devel 공통 대조군 | 11입력22쪽/backend | Native 비공개 견적서1쪽만0.1%p 이상 하락:99.79020→95.16159(-4.62861%p); #8110 후속 |
| 실제 Studio | PDF/HWP/HWPX15경로 | 삽입·직접 삭제·Snapshot undo/redo·양 저장 통과; PDF route 저장은 Native와 bytes 동일 |
| 실제 Worker | 7 Worker/123검사 | 렌더·저장·정책·오류 복구 통과 |
| 실제 문서2개 IR 왕복 | 12쪽/1쪽 | 양 형식의 표·셀 중첩 순서/문자 순서/쪽수 보존 |

실물A는 현재 IR에서15표161셀·공백 제외5324문자, 실물B는1표60셀·267문자를 관측했다.
이 수치는 작성 당시 구조의 독립 정답이 아니다. 비공개 실물B의 원본·식별 가능한 이미지는 로컬에만 보존한다.

4 HU 보정 전후 새169쪽 PNG는 Native/WASM 각각 모두 동일하다. 실제 편집 저장30개도
동일해 확보한 한컴2020 Print를 재사용했다. 전체 TSV는 새 producer에서 산출했다.
Native Helvetica6건은 원본 glyph program 부재·한컴 Batang 대체와 Native Noto Sans KR 선택을
실제 observer/글꼴 공급·동일 geometry로 구분한 정식 `font_mismatch_exception`이다.
raw미달을 그대로 유지하며 일반90% 통과나 새 정상 fixture로 등록하지 않았다.

[증거·입력/PDF 대응·원문 TSV](../pr/assets/issue_7998/final-p5/README.md)와
[페이지별 devel 비교](../pr/assets/issue_7998/final-p5/devel-comparison.tsv)를 보존했다.
독립 PDF와 같은 영역의 직접 확대 판독은 표/중첩·문단·탭·글줄·저장 경계와 알려진 글꼴/선 농도를 포함한다.
0.1%p 하락이나 보이는 차이를 평균값으로 상쇄하지 않는다. #8110은 사용자의 범위 종료 지시에 따른
후속이며 전체 devel 무회귀를 주장하지 않는다. 앞선 사용자 시각 판정과 이번 작성자의 직접 판독은 구분한다.

최종 로컬 비교 화면: `E:\tmp\issue7998-final-grid-review\index.html`.
실행 로그·원본 중간 자료는 ignored `output/pr-review/issue7998/final-integration/`과
로컬 `E:\tmp\issue7998-final-grid\`에 보존했다. 공개 자료는 원문 TSV·대표 PNG·정식 글꼴 예외와 실제 입력/Print 증거다.

## 전체 로컬 검사

release 빌드만 통과했다. 메인테이너 실사용 결함 보고 후 release lib 컴파일을 중단했으며,
남은 nextest·Native Skia·lint·doc test는 실행하지 않았다. 전체 검증 통과나 제출 준비 완료가 아니다.
통과한 fmt·suite manifest·unit-tier 정책과 변경 없는 TypeScript·Studio Node2342 PASS/2 SKIP,
package 소비·Visual Sweep 계약173 PASS는 source 해시 대조와 함께 stage37에서 연결했다.
원격 push·PR 생성은 별도 승인 이후에 수행한다.


## 후속 실사용 회귀 보정

stage37의162쪽 표 인식·가져오기 성능과 font_metric4쪽 보정은 메인테이너가 확인했다.
추가34쪽 `tb_ownership_inf.pdf`는 본문 구역과 인식 검사의 클립 공백 소유 범위 불일치로
실패했다. 같은 소유 결과를 소비하도록 보정했고 실제 Studio34쪽 열기·양 저장/재열기와
쪽별 문자 digest를 확인했다. Native/fresh WASM 각37쪽(실물34+대조3)의 미달·누락·쪽수 차이0,
실물 최저92.89699%/92.38421%다. 고정 devel Native 대비 하락0쪽이다.
기존162쪽·font_metric4쪽의 Native HWPX는 이전 수용본과 각각 바이트가 같으며, 실제 새
WASM에서도 정상 열렸다. 원문 표 의미 복원 완료나 기존 글꼴 차이 해소를 주장하지 않는다.

새 산출물은 `E:\tmp\issue7998-ownership-20261021\`과 ignored
`output/pr-review/issue7998/tb-ownership-20261021/`에 보존한다. source/실행 해시·전체 TSV·
재현·정상 대조·집중 회귀의 세부 결과는 [stage37](../working/task_m100_7998_stage37.md)을 따른다.
공개 PR asset의 옛 P5 제품과 새 uncommitted 보정 제품을 혼용하지 않는다. 전체 P5·한컴
실사용 저장본 인쇄·원격 제출은 아직 미완료이며 기존 백로그 분리와 승인 범위는 유지한다.

추가 소유권 보정의 정식 집중32건·fmt·diff 검사는 모두 PASS했다. 전체 P5 회귀와 구분한다.


추가 마스크 입력5쪽의 기존 unsupported도 보정했다. StreamMask의 단일Flate 허용을
색상 키/stencil과 같은 제한된 lossless sample decoder로 연결했다. 원본4개 차트의 RGB/alpha는
Native·WASM 및 양 저장 재소비에서 독립 해석 결과와 정확히 같다. 실제 Studio5쪽 열기·
3/4쪽 차트 표시와 HWP/HWPX 재열기를 확인했다. 자원/마스크 집중133건은 PASS했으며,
Native/fresh WASM 각10쪽(실물5+기존대조5)은 최저90.25971%/90.46384%·미달/누락/쪽수 차이0이다.
원본 마스크 없는1/2/5쪽과 기존 대조군5개의 저장 바이트는 수정 전후 같다. 기존 글꼴·미세
간격 차이, 전체 P5·한컴 인쇄와 원격 반영 미완료를 유지한다. 재시험/전쪽 자료는
`E:\tmp\issue7998-image-codec02-20261021\review\index.html`이며 source/실행/남은 차이는 stage37을 따른다.

추가 XObject 입력은 18쪽 Native 열기에 성공했고 실제 201,834회 호출을 삭제하지 않았다.
동일 stencil 연속 배치의 decode/저장 투영과 선택 후 traversal 실효 한도의 소비 연결을
보정하고 있다. 2쪽 기존 글꼴 굵기 차이(Native raw 88.80969%, 고정 devel과 같은 PNG)와
새 배경의 미세 무늬 차이는 직접 비교에 공개했으며 예외 수용으로 기록하지 않는다.
`E:\tmp\issue7998-xobject-20261021\review\index.html`과 stage37에 근거를 연결했다.
fresh WASM·실제 양 저장 재열기를 완료한 뒤 결과를 추가하며 전체 P5/PR은 미완료로
유지한다.

XObject 후보의 fresh WASM·실제 Studio 검증을 마쳤다. 원본18쪽은 약4.7초에 열리고
HWP/HWPX 저장·재열기에서18쪽/표시 문자를 유지했다. 최종 집중143건은 PASS이며 전체
Native/fresh WASM 각19쪽 TSV의 누락/쪽수 차이는 없다. 다만2쪽 최저88.80969%/88.39467%와
반복 배경 무늬 차이가 남아 시각 gate는 보류다. 범위 예외를 임의로 수용하지 않았고
백로그 분리/추가 보정 선택은 대기 중이다. 새 테스트 탭은 Studio7793에 열었으며 비교는
`E:\tmp\issue7998-xobject-20261021\review\index.html`이다. 소스·실행 지문과 단일 측정의
범위는 stage37을 따른다. 전체 P5·한컴 저장 호환·원격 등록의 완료 주장은 하지 않는다.

2026-10-21 작업지시자의 **“열기 개선으로 한정·출력 차이는 백로그”** 판정으로 위의
범위 대기를 종료했다. `xo_visits.pdf`의18쪽 열기·반복 처리 보정은 이번 로컬 범위에서
수용됐으며, 2쪽 글꼴 굵기와 반복 배경 무늬는 [후속 백로그](../plans/task_m100_7998_backlog.md)에
남겼다. raw 수치·자동 `re_review_required`는 그대로 보존하고 시각 완료로 바꾸지 않는다.
제품·검사 소스와 산출물 지문을 확인해 기존143건·Native/fresh WASM 각19쪽·실제 양 저장
증거를 재사용했다. 전체 P5·한컴 호환·PR/원격 반영은 여전히 미완료다.

추가 `extraction-002.pdf`1쪽의 `PDF_PARSE_FAILED`는 일반 DeviceGray Mask를1비트
stencil로만 판정한 image_decode 경계였다. 알파/극성·Matte 처리 경계를 구별해 보정했다.
수정 전 같은 정식 decoded-resource 검사 FAIL, 수정 후 자원132 + 마스크8 =140 PASS다.
Native/fresh WASM 원본1쪽 raw99.45899%/99.46414%, 별도 자원6쪽은 모두100%이며
실제 새 WASM1.726초 열기·서명 표시·HWP/HWPX 저장/재열기를 확인했다. 제목 face·
가는 표선 차이와 이 원본 Table IR0개는 미완료로 명시한다. 이번 결과를 전체 시각·표
인식·P5 통과로 확대하지 않으며, 한컴의 새 저장본 호환성은 미검증이다.
비교는 `E:\tmp\issue7998-extraction002-20261021\review\index.html`, 최종 source/빌드·
실행 근거는 [stage37](../working/task_m100_7998_stage37.md)에 연결했다. 원격은 변경하지 않았다.


추가 `coll_entries.pdf`의 collection entries 오류는 선택 쪽수로 문서 공통 구조 예산을
재축소한 경계였다. 기본 전역400,000·raw100,000·낮은 host cap과 실제 계상을 유지해
실소비180,625개·69쪽을 수용했다. 집중134건 PASS·fresh Docker WASM 실제 Studio 열기
단일 계측4.626초이며 HWP/HWPX 저장·재열기69쪽에서 전쪽 문자 digest/표 셀 배열을
유지했다. Native/fresh WASM 각69쪽의 누락/쪽수 불일치는 없지만 최저49.46762%/
94.55309%·미달45쪽/0이다. Native의 일부 한글 미표시와 제목 face/간격·영문 표제/표선
농도 차이는 남는다. 작업지시자의 **“열기 개선으로 한정·출력은 백로그”** 판정으로
이번 로컬 열기 보정만 수용하고 [후속 출력 백로그](../plans/task_m100_7998_backlog.md)에
남겼다. 자동 보류를 통과로 바꾸지 않으며 전체 P5/PR·한컴의 새 저장본 호환은 미완료다.
비교는 `E:\tmp\issue7998-collection-20261021\review\index.html`, 실제 재시험 탭은
`http://localhost:7793/?review=issue7998-collection&lang=ko`다. 최종 source/실행·TSV·직접
판독/남은 범위는 stage37을 따르며 원격 변경은 하지 않았다.


## 추가 입력 회전 글자 — 2026-10-21

text_not_supported.pdf의 7쪽 열기와 4쪽 45도 연도 레이블 30개의 editable TextBox·
paint subtree 회전을 보정했다. 최종 raw는 4쪽 Native 97.14196%·fresh WASM
97.06169%다. Studio 7쪽·HWP/HWPX 왕복의 문자/표/회전 보존과 회전 문자 삽입·삭제,
독립 회전 대조군 9개 전쪽 및 정식 자원 134건·표/문단 30건의 집중 범위를 확인했다.
기존 정상 입력의 저장 바이트와 원본 7쪽의 Native 픽셀은 변하지 않았다.

작업지시자의 “열기·회전 보정으로 한정·기존 출력은 백로그”에 따라 기존 7쪽 출력
차이는 별도 기록했다(Native 80.62888%·WASM 80.69685%). raw 미달·자동 보류와
전체 P5/PR 미완료를 유지한다. CanvasKit 실제 replay와 새 한컴 저장 호환 등은
미검증이며 근거·한계는 stage37을 따른다. 새 commit·원격 변경은 수행하지 않았다.
메인테이너 탭은 Studio 7793 `?review=issue7998-text-rotation&lang=ko`, 비교 자료는
`E:\tmp\issue7998-text-transform-20261021\review\index.html`이다.

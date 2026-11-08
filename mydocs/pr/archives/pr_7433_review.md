---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-16
---

# PR #7433 검토 — 저장 줄 없는 TAC 표의 성장·줄 소속·중첩 이월

## 최종 판정

**승인 — 메인터너 보정 후 승인.** 승인 대상은 [통합 PR #7951](https://github.com/edwardkim/rhwp/pull/7951)의
code candidate `6614bb3353d706f18a9beffb5885f3d4c7702e3a`다. 원 PR7433의 기능·입력과 메인터너 보정을 구분해 통합했으며,
현재 code candidate의 실제 Full CI·보조 workflow 성공을 확인했다. 아래 과거 보류는 작성 당시 기록이다.
trailing head의 required check·MERGEABLE/CLEAN을 다시 확인한 뒤 사용자 승인 범위에서 정상 merge한다.

- 실제 GitHub 검증: [CI](https://github.com/edwardkim/rhwp/actions/runs/38907419334) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38907418632) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38907418782) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38907418883) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38907418974). 같은 PR/repository/branch/head의 실제 성공 결과다.
- 로컬 renderer source `f5a4774049fa10cc661f7220de0127d61fa081ba`: 전체10,454 PASS·실패0·45 skipped,
  집중14+5 PASS, lint6종·Native4,043+2+4 PASS. 이후 code/tests/scripts/Cargo와 원본 입력은 불변이다.
- Native/host fresh WASM13문서19쪽씩 최저100%·독립 Print 쪽수 일치·직접 확대 판독을 확인했다.
  새 MCP Print3개7쪽도 기존 기준의96dpi RGB 픽셀과 전부 동일하고 양 경로 전쪽100%다.
- 작은 용지/partial 폭 부족/wrapper의 devel2/1/1쪽은 기준 결함이며 현재/Print3/2/2쪽으로 복구됐다.
  사용자 재검토에 따라 이3건을 수용했다. 기준의 없는6행은 계속NA이며 전체 일대일 통과로 꾸미지 않는다.
  같은 쪽이 있는10문서12쪽/경로와 추가 공통8쪽의0.01%p 이상 하락0이다. 다른 입력·쪽의 기준은 유지한다.
- 정식 회귀14개는 신규12+기존2보완이다. 관계 검사13개와 완료18쪽의 독립 Print±2px 검사1개를 포함한다.
- PR 본문에 전체 Native/host WASM TSV26개38행과 exact head의 대표 review/overlay4개를 실제 표시했고
  API 원문 및 raw 이미지 바이트 일치를 확인했다. 원시 TSV·로그는 Git에 커밋하지 않았다.
- Docker 표준 WASM은 사용자 선택대로 미검증이다. host dev/no-opt를 Docker 성공으로 보고하지 않는다.
  엄격 픽셀/Studio report-only 차이와 알려진 glyph raster 차이는 관용100%와 구분한다.
- 근거: [Stage6](../../working/task_m100_7419_stage6.md), [새 MCP Print](../../working/task_m100_7419_stage7.md),
  [사용자 판정 정정](../../working/task_m100_7419_stage8.md). #7416의 별도 문제는 해결 범위에 포함하지 않는다.

## 첫 기여자와 merge 후 contributor PR comment 계획

작성자imsebeom은 GitHub FIRST_TIME_CONTRIBUTOR이며 reviewer는jangster77이다. 원 PR head와 fork ref의
`d895e51ac5fd1621abb99f0a20aa52034e01e083` 일치를 확인했다. 사용자 지시의 devel 기반 체리픽 통합 경로이며
contributor의 author/date·원본 commit 대응과 fork를 보존한다. fork에 rebase/amend/force-push/삭제하지 않는다.

merge·devel 반영과 실제 이슈 상태를 확인한 뒤 원 PR7433에 한국어 존댓말로 rhwp 첫 기여 감사·환영을
명시한다. 원 기여인 NO_LS 셀 성장 보존을 먼저 설명하고, 선언 높이 추정 제거·중첩 줄 소속/분할/이월·
내용 보존의 메인터너 보정이 필요했던 이유를 독립 Print·수정 전후 회귀 근거로 구분한다.
실제 통합PR/merge SHA·위CI URL·로컬 결과·13문서19쪽 최저100%와 엄격 raster 차이를 기록한다.
시각 검증 방법은 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을
연결하고, 현재 tracked Native/host WASM 저장 줄 혼재2쪽 review/overlay4개를 merge SHA로 고정한
실제 raw Markdown 이미지로 포함한다. 사용자 직접 확인을 새로 받았다고 꾸미지 않는다.
통합 반영 안내 뒤 원PR은 superseded close하고, 이슈7419만 실제 종료 여부를 확인한다.
기여자 fork는 유지하며 다음 기여 환영을 적는다.7416은 닫지 않는다.

필수 원시 증거는 기본 작업공간의 `pdf/pr-review-evidence/pr7433-20261016/`에 해시 검증 후 보존한다.
post_merge.md 순서로 duration metadata 확인·devel sync·이슈/원PR 후속 안내·임시 head/worktree/branch와
output 루트 전체 정리를 마친다. 공유 target/pr-review·사용자 Studio JS와 contributor fork는 보존한다.

## CI 이전의 로컬 검토 기록

<details>
<summary>과거 보류·재검토 기록 — 현재 판정 아님</summary>

### 당시 판정

**현재 판정: 머지 보류 — 0953598 대비3문서의 전쪽 일대일 TSV 대응 미검증.** 기준은
`0953598d8e08af5e9149b3e4cdbbbf2f5f2bae64`, 검증 source는
`f5a4774049fa10cc661f7220de0127d61fa081ba`다. 현재 Native/host fresh WASM 각각13문서19쪽의
최저100%, Print 쪽수 일치와 같은19쪽의 직접 판독을 확인했다. 최신 PNG38개도7fe 검증본과
바이트 동일하다. 비교 가능한10문서·backend별12쪽의0.01%p 이상 하락0이며, 기존3문서의
기준 쪽수 오류와 전쪽 일대일 TSV 보류를 그대로 유지한다.

새 source의 Studio TypeScript·의존성 계약은 exit0, 단위2,021 PASS·실패0·2 skipped다.
실제 Studio15캡처의 backend/profile·초기 WASM 해시 일치와 실행 오류0, 내용 보존을 확인했다.
엄격 backend 픽셀10건의 report-only 미달은 남는다. 정식 집중14개·Square5개는 모두 통과했고,
현재 source의 전체 회귀도10,454 PASS·실패0·13 slow·45 skipped·exit0으로 끝났다.
lint6종과 Native Skia library4,043 PASS·12 ignored·exit0까지 확인했다.
Native 그림 placeholder2/2·직접 PDF4/4도 PASS·exit0이다. 승인된 로컬 검증은 완료했고,
전쪽 일대일 TSV3문서 보류는 남았다. 표준 Docker WASM·GitHub CI는 통과로 표시하지 않는다.
아래 cdef/7fe의 전체 통과는 이전 source의 완료 기록이며 새 실행으로
간주하지 않는다. [Stage6](../../working/task_m100_7419_stage6.md)에 명령과 실제 종료 결과를
연결한다. 사용자 답변에 따라 Docker는 설치하지 않고 표준 Docker
WASM은 미검증으로 유지한다. 기존3문서의 일대일 TSV 보류와 PR/push 별도 승인 조건도 유지한다.

### 3문서 기준 쪽수 오류의 추가 진단

2026-10-16 재확인한 원격 devel은0953598 그대로다. 현재 source·입력·Print·글꼴을
바꾸지 않고 기준 PNG의 존재하는 쪽을 같은 Visual Sweep 지표로 추가 측정했다.
전체 회귀 실패나 후보 출력 미달로 분류하지 않는다.

| 입력 | 기준/후보/Print 쪽수 | 기준 Native 공통 쪽 % | 기준 host WASM 공통 쪽 % | 후보 양 경로 전쪽 % |
| --- | --- | --- | --- | --- |
| page-height-28000 | 2/3/3 | p1 10.49571, p2 21.71547 | p1 9.29248, p2 23.50084 | p1–3 100 |
| partial-nested-width-wrap | 1/2/2 | p1 58.58906 | p1 56.69763 | p1–2 100 |
| partial-nested-wrapper | 1/2/2 | p1 90.93783 | p1 91.23852 | p1–2 100 |

진단 결과는 기본 작업공간의 ignored
`output/pr-review/pr7433-current-20261016/rebased-0953598/base-mismatch-audit/`에 보존한다.
`page-comparison.tsv`는 전14행(7쪽×2경로)에 존재하는 기준8행과 누락6행을 기록하며,
누락값을0이나100으로 대체하지 않고`NA`로 표시한다.6개`silhouette.tsv`는 공통 쪽만의
진단이며 전쪽 일대일 통과 증거가 아니다. 상세는[Stage6](../../working/task_m100_7419_stage6.md)
마지막 절에 연결한다.

공통8쪽의0.01%p 이상 하락은0이며 후보의 독립 Print 일치는 유지된다. 고정된 기준 SHA에
존재하지 않는 쪽을 후보의 추가 코드 수정으로 만들 수는 없다. 기준 출력의 잘못된 쪽수로
후보를 되돌리면 Print 일치가 깨진다. **현재 후보 출력 보정은 완료, 엄격한 기준 일대일 조건은
미충족**으로 구분하며 PR/승인/통합 보류를 해제하지 않는다.

### 사용자 요청에 따른 MCP Print PDF 재산출

세 HWPX를 매뉴얼의 start/status/download로 다시 변환했다. 새 job 모두 성공했으며 실제
한컴13.0.0.3901·PrintMethod0·frame_print_to_pdf_ex_one_up을 확인했다. 새 PDF는3/2/2쪽이고
전체7쪽의96dpi 픽셀이 기존 기준과 동일하다. 현재 Native/host fresh WASM 출력 PNG를
재사용해 새 PDF와 전쪽 TSV를 다시 산출했고 양 경로 모두 최저100%다. 새 대표 review와
standalone overlay도 생성했다. 새 기준 PDF3개는 기존 추적 경로에 반영했고, 이전 PDF와
기존 source의 검증 기록은 보존했다. job·새 PDF SHA·실제 검증 범위는
[Stage7](../../working/task_m100_7419_stage7.md)에 연결한다. 순수 devel의2/1/1쪽은 그대로여서
기존 일대일 보류를 예외 처리하거나 PR/push를 수행하지 않았다.

### 직전 cdef source의 판정

**머지 보류 — devel 대비3문서의 전쪽 일대일 TSV 대응 미검증. 전체 회귀·lint·Native 통과.**
추가 devel `cdef0073bb24512daf0622dec5051ac62cb8d57e`(#7948) 위로 리베이스를 완료했다.
당시 검증 source는 `7fe3b2ce45b996c550397ab588f4e3903cbd9508`이다. 새 source의
13문서19쪽 Native/fresh WASM은 모두100%, 독립 Print 쪽수 일치와 전쪽 직접 판독을 확인했다.
새 결과 PNG38개가 리베이스 전과 바이트 동일하다. 집중14+5개와 전체10,454개가 통과했고
lint6종과 Native Skia3종도 모두 통과했다. 아래10,449건 성공은 리베이스 전 증거로 구분한다.
fresh WASM은 host `--dev --no-opt` 진단 빌드이며 표준 Docker 최적화 빌드는 미검증이다.
기존 저장 줄 혼재 반례의 겹침은 보정했고, 양쪽 전쪽100% 및 수정 전 FAIL/후 PASS를 확인했다.
독립 Print와 일치하는 쪽수 복구를 base TSV 누락의 자동 예외로 처리하지 않는다.
PR/push는 사용자의 별도 승인 전까지 수행하지 않는다.

- 원 PR: #7433, `imsebeom`, 원 head `d895e51ac5fd1621abb99f0a20aa52034e01e083`.
- 관련 이슈: #7419. GitHub 이슈 종료·원 PR 종료는 하지 않았다.
- 기여자 기능 `52a51c0a97d0df7baab1e815ed820c6dff7ae2bb`, 검사/입력
  `4905a31d0746fa86e23fe03655f4538079e2d0a1`을 순서대로 통합했다.
  과거 보류 문서만 담은 마지막 commit은 새 검토 기록과 섞지 않았다.
  현재 대응 commit은 각각 `177650456abc896e11e1014bdaf0f88cdafcd595`,
  `ae67b089c99febd1874ba839ec7ac5f51144592e`다. 두 commit 모두 원 author·author date를
  보존하지만 출처 trailer는 없어 이전 `-x` 표기를 정정한다. 검사 commit의 stable patch-id는
  원본과 같고, 기능 port는 당시 devel의 overlay 제외·stored_rows 우선 분기를 보존해
  적용했으므로 원본 patch-id와 다르다. 실제 patch 대조는 Stage6에 연결한다.
- 리베이스 전 기준: `4662c069e557e928ed7968d061121fde90b1f027`.
- 리베이스 전 렌더링 code: `dca310c06036c7f3fa7ca30cce74c1704f1169f2`.
- 리베이스 전 새 검사/입력 commit 및 전체 검증 시작 head: `9f464aa2c`.
- branch: `review/imsebeom-7433-20261016`.
- 최초 확인에서는 기준이 같아 rebase하지 않았고, 이후 devel이cdef로 바뀐 것을 확인해
  실제 리베이스했다. 당시 검증 source는 위의7fe이며 과거4662 결과와 구분한다.
- collaborator 외부 PR 체리픽 통합 경로다. 원 fork를 rewrite하지 않으며, 원 contributor head를
  직접 merge 대상으로 삼지 않는다. 통합 PR 채번용 별도 review 또는 docs-only PR을 만들지 않는다.

## 기여자 수정과 메인터너 보정

기여자는 NO_LS HWPX TAC 표의 셀 성장분을 보존했다. 메인터너는 선언 높이 합의0.5px
일치로 입력 생성 출처를 추정하던 조건을 제거했다. 중첩 표의 실제 본체 높이·바깥여백과
UTF-16 제어문 슬롯의 줄 소속을 현재 프레임이 함께 발행한다. 측정·셀 유닛·whole 배치·
partial eager/lazy paint가 같은 결과를 소비하며, 같은 줄과 여러 줄의 높이를 구분한다.

1×1 wrapper는 배치 소유자가 단일 자식 표와 폭 중립 마커일 때만 해제한다. 두 자식을
첫 표 하나로 축소하지 않는다. wrapper가 통째로 다음 쪽으로 가는 독립 Print와 대조했다.
원본 Table/Cell 참조를 유지해 주소 기반 캐시에 임시 복제 모델을 넣지 않는다.

실측 높이를 전달한 현대 NO_LS 셀만 새 프레임을 수용한다. 일반/Native 문단의 기존
보이는 글자+단일 TAC 계약은 유지한다. 새 프레임을 만들 때 저장 줄이 있는 TAC 형제도
실측 높이/소유 목록에 포함하여 셀 컷과 paint의 별도 예약·겹침을 해결했다.
자식 내부의 유효 저장 줄은 다시 나누지 않으며, 모두 저장 줄인 host는 새 경로로 넓히지 않는다.
문서 ID 분기·좌표 clamp·숨김·허용치 완화는 추가하지 않았다.

실제 생산→측정→예약/컷→배치 호출과 적용/비적용 경계는
[Stage1](../../working/task_m100_7419_stage1.md),
[Stage3](../../working/task_m100_7419_stage3.md),
[Stage4](../../working/task_m100_7419_stage4.md),
[리베이스 후 Stage5](../../working/task_m100_7419_stage5.md)에 연결한다.

## cdef 리베이스 후 검증

- 기준 `cdef0073bb24512daf0622dec5051ac62cb8d57e`, source `7fe3b2ce45b996c550397ab588f4e3903cbd9508`.
- 같은 입력·Print·글꼴·96dpi·Print profile·compat2024의 Native/fresh WASM13문서19쪽씩
  최저100%, 미달·누락0, Print 쪽수 일치. 같은19쪽을 양쪽과 Print에서 직접 열어
  줄바꿈·쪽 소속·내용 보존·여백·후속 본문을 대조했다. 수식·각주가 없는 입력의 한계는 유지한다.
- 기준 Native와 바이트 동일한 최초 후보 실행 파일을 검출해 해당 build/측정을 무효화했다.
  공용 target은 보존하고 source 진입점 mtime 갱신 후 실제 재컴파일했다.
  현재 producer 해시·명령·203.7초 Native/209.1초 WASM build/exit0는 Stage5에 연결한다.
- 새 devel과 비교 가능한10문서·backend별12쪽은0.01%p 이상 하락0·조건 불일치0이다.
  작은 용지·partial 폭 부족·wrapper3문서는 base 쪽수 불일치로 전쪽 TSV가 없어
  현행 일대일 gate의 `re_review_required`를 유지한다. 첫 불일치에서 중단된 배치 뒤의
  입력도 개별 실행으로 완료했고, 미실행을 문서 결함이나 통과로 대신하지 않았다.
- 새 source의 정식 집중14개와 devel Square5개, 전체10,454개가 모두 통과했다.
  전체는 실패0·45 skipped·31 slow·exit0, 컴파일7분50초·테스트1230.674초였다.
  이번14개와 새 Square5개 및 신규 sample2개를 명시 전달한 보안 검사도 실제 전체에서 PASS다.
  lint6종 및 Native Skia3종도 모두 exit0이다. Native library 합계4043 PASS/12 ignored,
  그림2/2와 직접 PDF4/4 PASS다.
  원시 증거: ignored `output/pr-review/pr7433-current-20261016/rebased-cdef/`의
  `producer-builds.json`, `base-completion.json`, `direct-read.json`,
  `rebase-png-comparison.json`, `final-devel-comparison.json`, `focused-validation.json`,
  `final-pipeline.log`, `full-summary.json`, `final-verification.json`,
  `{lint,native}-validation.json`. 표준 Docker WASM과 GitHub CI는 미검증이고 PR 생성·push는 실행하지 않았다.
- 집중14/14와 Square5/5가 통과해 전체 회귀를 시작했다. 현재 Studio에서도 원본2개와
  혼재/NO_LS 경계5입력·쪽 조합을 Canvas2D·CanvasKit compat/default software로15캡처했다.
  초기 WASM15응답의 현재 package 해시 일치, 실제 backend/profile 일치, 캡처 오류·replay
  hard violation·text fallback·unsupported0을 확인하고 실제 줄바꿈·소유·후속 본문을 직접 대조했다.
  backend 엄격 픽셀 비교10건은 report-only 미달(최대 차이율0.02082074)이다.
  이를 parity gate 또는 독립 Print 일치로 승격하지 않는다. Stage5와 자산 기록에 연결한다.
- 완료 감사에서 원 이슈의8457바이트 재현 입력 MD5 일치를 확인했다. 실제 한컴 저장
  HWP/HWPX 정상 대조군도 각각2쪽이 cdef/7fe에서 바이트 동일하며, 과거 실패3개가
  현재 전체 로그에서 모두 PASS다. 독립 Print 정확도 주장과 저장 host 무변화를 구분한다.
  Stage5의 `completion-control-audit.json`에 연결한다. #7416은 원 이슈의 제외 범위다.

## 리베이스 전 독립 출력과 직접 판독 기록

- 원본2개 및 선언 높이·작은 용지·동일 줄·명시적 개행·폭 부족·문맥·여백·분할·wrapper
  반례9개는 수동 XML이다. 실제 한컴 저장 출처로 주장하지 않는다.
- 새2개는 실제 한컴 HWP→HWPX 저장본의 LineSeg 일부만 제거한 혼재 입력과 대조군이다.
  생성 job·남긴 저장 줄·SHA는 [입력 기록](../../../tests/fixtures/issue7419-boundaries/README.md)에 있다.
- 독립 기준은 같은 입력의 한컴2024 13.0.0.3901 PrintMethod0,
  `frame_print_to_pdf_ex_one_up` PDF다.
- dca source의 **13문서19쪽 Native/fresh WASM 양쪽 최저100%, 미달·누락0, Print 쪽수 일치**.
  2px 관용 실루엣이며 엄격 픽셀100%가 아니다.
- 양쪽과 독립 Print의 같은19쪽 영역을 직접 대조했다. 줄바꿈·표 외곽/점유·자식 쪽 소속·
  뒤 본문·여백·누락·중복을 확인했고 겹침이나 추가 빈 쪽은 관측하지 않았다.
  이 입력에 없는 수식·각주 조합은 새로 검증했다고 주장하지 않는다.
- Native는 `native-skia` PNG, fresh WASM도 PNG 경로다. 같은96dpi·Print profile·compat2024·
  한컴 글꼴을 사용하고 source SHA와 실제 생산자 해시를 기록했다.
- Docker 미설치 호스트여서 locked wrapper의 host `--dev --no-opt` 진단 경로를 사용했다.
  Docker 표준 빌드나 GitHub CI 성공으로 표시하지 않는다.

## 정식 회귀와 수정 전후

[정식 검사](../../../tests/cases/issue_7419_no_ls_tac_row_growth.rs)는14개다.
원본2개의 고정 영역 선택을 셀 내용·순서·포함·비겹침 관계로 교체했다. 관계 검사13개가
같은 줄/개행/폭 부족/문맥/여백/이월/조각 종료/저장 줄 혼재를 보호한다.
완료 페이지18쪽의 핵심 제목·후속 본문 원점은 독립 PDF span.origin을96dpi로 환산한
±2px 검사1개로 함께 보호한다. 작은 용지1쪽은 제목만 있어 추가 위치 핀을 만들지 않았다.
Rust 좌표 검사는 실루엣98% 유지 판정의 대체가 아니다. 위18쪽은 이후에도 Native와 fresh
WASM 각각98% 이상이어야 하며, 좌표 검사가 통과해도 한쪽이 미달하면 회귀로 처리한다.
현재f5a에서는 해당18쪽을 포함한 전19쪽의 양쪽 최저100%와 직접 판독을 다시 확인했다.

| source/생산자 | 검사와 실제 결과 |
| --- | --- |
| 초기10개 | devel0pass/10fail, 원 체리픽3pass/7fail, 최초 보정10pass |
| 분할2개를 추가한12개 | 여백 앵커 오류10pass/2fail → 보정12pass |
| 59d의 기존 실패3개+당시12개 | 실제 Cargo15pass. 시행령74쪽 render tree가 devel과 바이트 동일 |
| dca의 현재14개 | 준비한 suite에서 실제 Cargo14pass/실패0 |
| 같은14개 실행 파일, 출력 CLI59d→dca | 12pass/2fail →14pass/0fail. 혼재 자식의 쪽 소유와 독립 좌표가 의도한 원인으로 실패 |

전부NO_LS인 새 대조군은59d/dca 모두 통과한다. 실제 저장 host 정상 대조군2쪽 render tree도
두 source에서 바이트 동일하다. 이 비교만으로 해당 정상 대조군의 독립 Print 정확도를 주장하지 않는다.
초기 stale generated suite 실행0건/exit4는 통과로 세지 않았다. `--prepare` 후 runner로14건을
실제 실행했다. generated harness/manifest는 커밋하지 않는다.

## 리베이스 전 전체 게이트 기록

- 첫 source `fafd0d378` 전체는10,447건 중10,444pass/3fail/45skip이었다. 일반/Native까지
  수용을 넓힌 원인을 고쳤으며 기존 기대값·baseline·허용치를 바꾸지 않았다.
- 59d의 lint6종과 집중15개 통과는 과거 source 증거다. 새 혼재 실패를 확인해59d 전체 빌드는
  테스트 시작 전에 SIGINT로 중단했다(pipeline exit130, test summary 없음).
- **9f464aa2c의 release-test 전체10,449pass/실패0/45skip을 확인했다**(실행713.108초, 빌드11분18초).
  이전 실패3개와 이번14개도 모두 통과했다. 이어 fmt·Native/WASM Clippy는 통과했으나
  새 devel 확인 후 workspace build 중 pipeline exit130으로 중단하고 기록을 고정했다.
  당시 workspace Clippy·suite check·Native Skia3종은 완료하지 않았다.
  집중14개와 이전 source 결과를 합산해 리베이스 후 전체 통과로 표시하지 않는다.
- `CARGO_BUILD_JOBS=8`, `/home/tsjang/rhwp/target/pr-review`를 사용한다. 다른 Cargo를 동시에 실행하지 않는다.
- Studio 원본2개의 Canvas2D/CanvasKit compat/default software6캡처는59d 당시의 추가 증거다.
  초기 WASM 응답 해시6개 일치, replay hard violation0. 엄격 픽셀4건은 report-only 불일치
  (최대0.01088193)였다. 이를 dca의 Studio 재실행 또는 parity gate 성공으로 승격하지 않는다.

## 리베이스 전 devel 전쪽 대응 기록

같은 입력·독립 Print·글꼴·DPI·profile로 비교 가능한10문서의 backend별12쪽에서
0.01%p 이상 하락0, 조건 불일치0이다. 새 혼재2문서는 base도2쪽이라 페이지별 비교가 가능하다.
아래3문서는 정상 쪽수 복구로 base 전체 TSV가 없어 전쪽 일대일 대응이 아직 미검증이다.

| 입력 | base Native/WASM | 현재 후보/독립 Print |
| --- | --- | --- |
| page-height-28000 | 2/2쪽 | 3/3쪽 |
| partial-nested-width-wrap | 1/1쪽 | 2/2쪽 |
| partial-nested-wrapper | 1/1쪽 | 2/2쪽 |

사용자의 현행 기준 유지 지시에 따라 `re_review_required`를 유지한다. 이 문서를 검증에서
제외하거나 Print/쪽수 기대값을 바꿔 보류를 숨기지 않는다. 전체 테스트가 통과해도 이 보류와
사용자 PR 승인 조건을 별도로 충족해야 한다.

## 현재 시각 자산 — cdef 리베이스 후 재출력

[자산·SHA 목록](../assets/pr_7433_maintainer/README.md). 사용자 판독을 위해 review와 overlay를 함께 표시한다.

![Native 혼재2쪽 review](../assets/pr_7433_maintainer/native-mixed-storage-review-002.png)

![Native 혼재2쪽 overlay](../assets/pr_7433_maintainer/native-mixed-storage-overlay-002.png)

![fresh WASM 혼재2쪽 review](../assets/pr_7433_maintainer/wasm-mixed-storage-review-002.png)

![fresh WASM 혼재2쪽 overlay](../assets/pr_7433_maintainer/wasm-mixed-storage-overlay-002.png)

## 원시 증거와 원격 상태

리베이스 전 증거는 ignored `output/pr-review/pr7433-current-20261016/mixed-storage-fix/`에 있다.
`native-build.json`, `wasm-build.json`, `focused.json`, `before-after.json`,
`mixed-visual-evidence.json`, `final-direct-read.json`, `final-devel-comparison.json`,
`all-saved-comparison.json`, `review-assets.json`, `full-summary.json`, `final-pipeline.log`,
`{full,lint,native}-validation.json`을 사용한다. 파일 존재 자체를 완료 근거로 쓰지 않는다.
이전 source 증거는 `regression-fix/`에서 보존한다. TSV·로그·중간 JSON은 커밋하지 않는다.
주 작업공간 Studio JS의 사용자 dirty 변경은 보존했다.
PR 생성·push·CI·merge·원 PR 종료·post_merge 정리는 아직 수행하지 않았다.

</details>

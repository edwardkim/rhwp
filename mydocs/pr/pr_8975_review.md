# PR #8975 리뷰 — 비TAC CellBreak 행 이월 회귀

## 최종 판정

**메인테이너 보정 후 수용 가능 — 필수 로컬 검증 완료.** 사용자가 간격도 보정하도록 승인했다.
공통 보정 `e5a6a1ff9f` 이후 목표 전 4쪽은 Native/fresh WASM 모두 100.00%다.
전체 회귀 13,432건·Native 공식 4,073건과 필수 lint가 통과했고, 보호 전 3,542쪽의 공식값 하락은 0건이다.
원격 처리는 사용자 승인과 새 통합 head의 CI가 필요하다.
기존 간격 차이에 예외를 적용하거나 원격 변경을 수행하지 않았다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 작성자·base | sacru2red / devel |
| 원 head | `286eb32e0877d2b1fccc0753b60d9604221c156f` |
| 최신 기준 | `180a4ec97205b67c4dc4f90416063c01ecfdf396` |
| 누적 검토 source | 제품 보정 `e5a6a1ff9f`, 회귀 보완 `b4b8c27260` |
| reviewer | edwardkim, 2026-11-13 지정 확인 |
| 원 head CI | 재조회 success 32·정상 skip 5 |
| 병합 시뮬레이션 | `samples/issue7620/README.md` 1곳 충돌. 양쪽 설명을 보존한 로컬 merge `d06e8861fe` |
| 관련 이슈 | #7620, OPEN 유지. 생성 대조군을 한컴 저장본 일반으로 확대하지 않음 |

## 변경과 독립 검증

원 기여는 제품 코드를 바꾸지 않고 생성기·HWP 2개·웹 인쇄 PDF·관계형 회귀 2건을 추가한다.
최신 devel의 합성 LineSeg 설명과 새 생성 대조군 설명의 README 충돌만 로컬에서 결합했다.
원 기여의 모든 commit은 보존했다. 이후 승인된 공통 보정과 독립 근거의 검사 보완을 추가했다.

원 PR의 웹 인쇄 PDF(Producer PDFium)는 데스크톱 Print 판정에서 제외했다. 동일 HWP의 저장
metadata는 2020이지만 실제 rhwp 생성 대조군이다. MCP `engine=2020`을 명시해 Windows
Hancom 2020 `11.0.0.9136`, `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`으로 출력했다.
각 PDF 2쪽을 Native Skia·96dpi·`--compat 2022`로 대조한 결과:

| 입력 | 1쪽 / 2쪽 raw | 직접 확인 |
| --- | --- | --- |
| e-cellbreak | 100.00 / 94.62% | 1행은 1쪽, 2·3행은 2쪽으로 통째 이월 |
| f-rowbreak | 100.00 / 93.46% | 2행 1–5줄은 1쪽, 6줄은 2쪽, 이후 3행 |

모든 셀 글줄과 뒤 본문은 보존된다. 다만 두 입력 모두 2쪽 뒤 문단의 잉크 윗변이 Native에서
8px 아래다. fresh WASM도 각각 전쪽 최저 94.90%·93.79%이며 뒤 문단의 잉크 윗변은 7px 아래다.
[간격 차이와 입력·PDF 해시](sacru2red_20261113_spacing_finding.md)에 근거를 분리했다.
[보정 전 TSV·대표 PNG·실행 조건](assets/sacru2red-20261113/README.md)을 보존했다.

## 승인된 메인테이너 보정과 결과

실제 수용된 줄 하단과 순차 전진이 이미 소비한 600HU를 증명하는데도, 기존 단 원점이 없으면
공통 `HeightCursor::vpos_adjust`가 이를 무시하고 상대 원점을 다시 역산했다.
[선행 해결 PR·실제 측정/배치 호출 연결·반례](sacru2red_20261113_review_impl.md)를 추적한 뒤,
원점이 없을 때도 연속 저장 줄과 실제 소비가 일치하면 그 근거를 사용하게 했다. 기존 원점이
있는 경우의 축 일치 검사, 합성·비연속·편집 경로의 bridge는 유지했다.

보정 후 e·f의 Native/fresh WASM 각각 전 4쪽 raw는 모두 100.00%다. 실제 빈 줄→뒤 본문
전진도 원본 1600HU와 일치하고, 새 관계 검사의 음성 대조는 수정 전 실제 트리에서 모두 FAIL이다.
정식 Rust 집중 8건은 보정 후 모두 PASS다. SVG도 Chromium 래스터 전 4쪽이 100.00%다.
원 이슈의 기존 a-RowBreak·None 입력도 Desktop Print와 보정 전후 Native/WASM이 각각 100.00%다.
[최종 전쪽 증거·직접 판독](assets/sacru2red-20261113/final/README.md)을 보존했다.

## 조판 원칙·기존 검사·검증 입력 검토

신규 검사는 고정 입력의 비TAC·TopAndBottom·Para·offset 0·3×2·pageBreak를 확인한 뒤
36개 셀 글줄의 쪽 소속·순서·누락/중복, 표 조각, 뒤 문단의 포함·앞뒤 관계를 검사한다.
쪽수 2 고정에는 전체 PDF와 내용 대응을 함께 확인하며 실물 절대 px 좌표는 고정하지 않는다.
기존 TAC 검사 `issue_7288_cell_unit_row_is_atomic`와 #7620의 RowBreak 예산 검사를 함께 검증한다.
원 기여는 분할 알고리즘을 바꾸지 않는다. 메인테이너 보정은 위 공통 상대 원점 계산에 한정한다.
독립 e·f Print 이후 기존 `lazy_path_applied_and_base_set`의 원점 부재 반례를 재검토해
중복 소비를 기대하던 한 경우를 고쳤고 다른 부정 대조군은 보존했다. 새 회귀 파일의 첫머리에
측정 시각·전쪽 수치·직전 Git 변경 정보를 기록했다.
원본 HWP의 SHA-256과 실제 MCP 입력 bytes의 동일성을 확인했다. 새 Desktop Print는 최종 검토
증적에 정식 보존하며 원 기여의 웹 PDF를 같은 파일로 간주해 삭제하지 않는다.

[공통 계획](sacru2red_20261113_review_plan.md)에 PR별 원격 승인 경계와 남은 절차를 기록했다.

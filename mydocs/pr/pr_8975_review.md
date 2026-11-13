# PR #8975 리뷰 — 비TAC CellBreak 행 이월 회귀

## 최종 판정

**머지 보류 — 기존 8px 간격 차이의 처리 범위와 로컬 검증을 확인 중.**
행 이월·내용 보존의 주장은 데스크톱 Print에서도 유효하다. 전쪽 수치가 90% 이상이라는
이유로 표 뒤 문단의 기존 차이를 면제하지 않는다. 원격 변경은 하지 않았다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 작성자·base | sacru2red / devel |
| 원 head | `286eb32e0877d2b1fccc0753b60d9604221c156f` |
| 최신 기준 | `180a4ec97205b67c4dc4f90416063c01ecfdf396` |
| 누적 검토 source | `ea627d6017` |
| reviewer | edwardkim, 2026-11-13 지정 확인 |
| 원 head CI | 재조회 success 32·정상 skip 5 |
| 병합 시뮬레이션 | `samples/issue7620/README.md` 1곳 충돌. 양쪽 설명을 보존한 로컬 merge `d06e8861fe` |
| 관련 이슈 | #7620, OPEN 유지. 생성 대조군을 한컴 저장본 일반으로 확대하지 않음 |

## 변경과 독립 검증

제품 코드는 바꾸지 않고 생성기·HWP 2개·웹 인쇄 PDF·관계형 회귀 2건을 추가한다.
최신 devel의 합성 LineSeg 설명과 새 생성 대조군 설명의 README 충돌만 로컬에서 결합했다.
원 기여의 모든 commit은 보존했으며 기대값·제품 코드는 보정하지 않았다.

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
[전쪽 TSV·대표 PNG·실행 조건](assets/sacru2red-20261113/README.md)을 보존했다.

## 조판 원칙·기존 검사·검증 입력 검토

신규 검사는 고정 입력의 비TAC·TopAndBottom·Para·offset 0·3×2·pageBreak를 확인한 뒤
36개 셀 글줄의 쪽 소속·순서·누락/중복, 표 조각, 뒤 문단의 포함·앞뒤 관계를 검사한다.
쪽수 2 고정에는 전체 PDF와 내용 대응을 함께 확인하며 실물 절대 px 좌표는 고정하지 않는다.
기존 TAC 검사 `issue_7288_cell_unit_row_is_atomic`와 #7620의 RowBreak 예산 검사를 함께 검증한다.
측정·배치·paint 자체는 변경하지 않으므로 새 일반성 분기·분할 알고리즘의 도입은 비해당이다.
원본 HWP의 SHA-256과 실제 MCP 입력 bytes의 동일성을 확인했다. 새 Desktop Print는 최종 검토
증적에 정식 보존하며 원 기여의 웹 PDF를 같은 파일로 간주해 삭제하지 않는다.

[공통 계획](sacru2red_20261113_review_plan.md)에 PR별 원격 승인 경계와 남은 절차를 기록했다.

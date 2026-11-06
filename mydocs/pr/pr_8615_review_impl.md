# PR #8615 누적 검토 구현 계획

## 대상과 단계

[원 PR #8615](https://github.com/edwardkim/rhwp/pull/8615)의 원 head는 `7d1d9b51f2f2cc9f92aef46eefe101a6d4b433a5`다.
[공통 계획](assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `7d1d9b51f2f2cc9f92aef46eefe101a6d4b433a5` | `dd6d1dc056a01a605191d991cd607eb4d797f5b1` | fix(layout): 칸 안 빈 host 의 글 위아래 표 띠가 뒤 문단 앞 간격을 흡수한다 (#8610) |

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.

## 자동 분할 경계선의 메인터너 보정 계획

2026-11-07 KST, 같은 원본의 R11 Native/fresh WASM review/overlay에서 2021년
보도자료 1쪽 아래, 2쪽 위·아래, 3쪽 위에 Print에 없는 가로선이 확인됐다.
원 PR의 빈 host 뒤 간격 보정과 구분한다. 98%대의 페이지 점수로 면제하지 않는다.

- 원본 `samples/issue8610/36849-nars-press-release-20211116.hwp`의 DocInfo를
  renderer 없이 CFB·레코드 단위로 읽었다. 표가 참조하는 borderFill 15의 attr은
  `0x0400`, 일반 본문 셀의 borderFill 5는 attr 0과 0.4mm 네 변이다.
- 원본을 실제 한컴 2020 11.0.0.9136 MCP로 HWPX 저장한 관찰본에서 표의
  `borderFillIDRef=15`, 해당 `breakCellSeparateLine=1`, `diagonal@type=NONE`을
  확인했다. PDF는 재산출하거나 교체하지 않았다.
- 정상 대조군 #5885는 같은 TopAndBottom·RowBreak지만 표 참조 borderFill 3의
  attr 0, 한컴 저장 HWPX의 `breakCellSeparateLine=0`이다. 이 표의 닫힌 경계를
  그대로 유지해야 하므로 wrap 종류만으로 모든 선을 지우는 가설은 채택하지 않는다.
- [한컴 공식 자동화 설명](https://forum.developer.hancom.com/t/topic/1357)은 표의
  TableBorderFill에서 BreakCellSeparateLine을 켜고 DiagonalType·Width·Color로
  자동 분할 경계선을 설정한다. 이 문맥의 diagonal은 셀의 대각선 그리기와 다르다.
- 기존 독립 Print의 벡터 선을 직접 조회했다. 1쪽의 본문 마지막 밑과 2쪽 전체에는
  전폭 수평선이 없고 3쪽 실제 종료선은 남는다. 무조건 숨김이나 좌표 clamp가 아니라
  원본의 별도 경계선 NONE을 같은 부분 표 경계에 적용해야 한다.

소비 경로는 `BorderFill 원본/한컴 HWPX → ResolvedBorderStyle → 부분 표의 원본
셀·zone 테두리 수집 → 실제 자동 분할 경계 덮어쓰기 → 공통 render tree`다.
현재 HWPX parser는 속성을 버리고 serializer는 0을 하드코딩하며, 부분 표는
셀의 굵은 종료선을 모든 조각에 복제한다. 속성을 보존하고 별도 선을 최종 경계에
적용한다. 비분할 표·설정이 꺼진 표·실제 마지막 종료선은 유지한다. 설정 bit와
중심선/대각선 문맥을 혼동하지 않는다.

원본·독립 Print는 유지하고 NARS 2019/2021·정상 #5885의 실제 경계, 전쪽 TSV와
같은 소비 경로의 Native/fresh WASM을 대조한다. 새 렌더 회귀는 시각 선행 조건 후
추가하며 검증 전 통과로 기록하지 않는다. ignored output의
`nars-5885-source-border-attrs.json`, `mcp-border-*-hwpx-hwpx.json`,
`nars-independent-pdf-horizontal-lines.json`에 원본/독립 관찰 근거를 보존했다.

### R26b Native 전쪽 결과

첫 빌드는 ResolvedBorderStyle의 명시적 storage clone 목록에 새 필드가 빠져
exit101이었다. 필드 목록을 보완하고 같은 후보를 재실행해 exit0(7분56초)을
확인했다. 기대값·임계값은 변경하지 않았다.

같은 원본·기존 독립 Print·96 DPI의 Native 실제 비교에서 NARS2021은
1쪽99.85%,2·3쪽100.00%다(R11은98.47/97.89/96.50%). 전3쪽 review와
overlay를 직접 판독해 1쪽 하단·2쪽 상하·3쪽 상단의 불필요한 가로선 해소와
실제 마지막 종료선·문장 보존을 확인했다. NARS2019 전1쪽과 정상 #5885 전7쪽은
모두100.00%다. 각 문서의 쪽수는 Print와 일치한다.

producer SHA-256은86ecc0b5ebb64ff0aa4c6fd928799f3b7724c88d0a5548ff11071a969a1d8d0c다.
R26b source receipt로 소스를 고정했다. 별도로 HWPX의 꺾인 BackSlash와 표 경계선
문맥을 혼동하지 않도록 저장측의 bit10 해제 조건을 검토했으며, 이 저장 전용 델타를
반영한 R26c에서 집중 저장 검사·fresh WASM을 수행한다. 이 델타를 R26b producer의
시각 결과로 소급 표기하지 않는다. 최종 보호 문서·전체 회귀·lint·브라우저 검증 전이다.

## R26c fresh WASM과 회귀 등록의 선행 조건

2026-11-07, 최적화 fresh WASM 빌드 exit0이다. 동일 원본·기존 Print에서 NARS2021은
전3쪽99.87/100/100%, NARS2019 전1쪽과 일반 경계선 대조군5885 전7쪽도 모두100%다.
Native R26b의99.85/100/100% 및 두 정상 대조군100%와 함께 독립 Print 쪽수를 유지했다.
양쪽 전3쪽 review/overlay에서 불필요한 컷 선 제거와 실제 종료선·본문·연락처 보존을 판독했다.
[양쪽 전쪽 TSV·실제 source/producer·정상 대조군](assets/planet6897_20261106/pr8615/r26/native-provenance.json)을 보존했다.

이 선행 조건 뒤 새 tests/cases의 automatic_break_boundary를 추가한다. 절대 위치를 핀하지 않고
표 전체 폭의 내부 컷·실제 종료선·연락처 소속을 검사한다. 별도 모델의 HWP5/HWPX 저장도 실제
재파싱해 보존하며, 합성 비트 계약으로 꺾인 BackSlash와 중심선의 독립 해석을 대조한다.
집중 검사·최종 통합 전쪽 비교·전체 회귀·lint는 별도 실제 종료 결과를 확인한다.

최소 글꼴 환경의 정식 집중 nextest에서 새 경계선3건 모두 PASS, 실제 실행 exit0을 확인했다.
함께 실행한 보고서 수식·꼬리 소속2건도 PASS다. 전체 Rust 회귀는 이후 별도 실행 중이다.

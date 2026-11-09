# PR #8615 통합 적용·메인터너 보정 기록

## 통합 PR의 exact code head CI

- [통합 PR #8857](https://github.com/edwardkim/rhwp/pull/8857), code head `29c74b69851bc6578f10a2c78e23592a05b69a5d`.
- [CI 실행](https://github.com/edwardkim/rhwp/actions/runs/42621490405)과 같은 code head의 CodeQL·Render Diff·Adapter·Proptest·정책 검사를 실제 확인했습니다. 기록 확인 시각: `2026-11-09T22:58:24.689803+09:00`.
- 기본 기능 worker A–E의 실제 nextest는 **12,934 PASS / 0 FAIL / 45 skip**입니다. 기존 로컬 13,105건과 실행 범위·HEAD가 다르므로 합산하지 않습니다.
- source·test·workflow를 추가로 변경하지 않고 이 검토·오늘할일만 단일 부모 trailing commit으로 보완합니다. trailing head의 실제 CI/fast-pass와 mergeability는 push 뒤 별도로 확인하며, 문서만 바뀌었다는 이유로 통과를 미리 가정하지 않습니다.
- [잔여 이슈 #8856](https://github.com/edwardkim/rhwp/issues/8856)은 9문서군의 글리프·간격·도형 및 clip5587 Native 4쪽의 엄격 0.1%p 미충족을 계속 추적합니다. 부분 해결 #8635·#8636과 부모 #7445는 일괄 종료하지 않습니다.


## 최종 source와 재사용 범위

제출 code head `2e262d0d4b2885b130e2db917b86e894710425e4`, base `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`입니다. 아래 전체 회귀·전쪽 수치는 원 실측 HEAD의 증거를 재사용한 기록입니다. 최신 집중 검사와 페이지별 producer, 엄격 0.1%p 잔여 한 건은 [재사용·최종 판정](../assets/planet6897_20261106/final-b768bee1-20261109/validation-reuse.md)을 따릅니다. 보호 TSV 표시값 하락은 0입니다.

## 최종 구현·검증 결과

검증 code head `eb876d1416fb558c874f9efec65c5722a6769751`, base `b768bee1c4c0023087687f79a0a95f12108743e9`에서 원 기여와 메인터너 보정을 함께 검증했습니다. 최소 글꼴 전체 회귀 13,105 PASS / 0 FAIL / 45 skip, Native 전용 73 PASS이며, Native/fresh WASM 각각 94조건·2,995쪽의 미달·누락·보호 표시값 하락은 없습니다. 원 PR의 개별 입력·페이지 판독과 보정 내용은 [최종 review](pr_8615_review.md)에 연결합니다. 원격 통합 PR의 최종 CI와 병합은 별도 단계입니다.

## 단계별 기록의 시점

아래 R 단계와 과거 실행의 진행 중·실패·추가 검증 표현은 해당 시점의 원인 추적 기록입니다. 과거 결과를 현재 code head의 통과로 합산하지 않으며, 현재 상태는 위 최종 결과와 연결된 review를 기준으로 판단합니다.

## 대상과 단계

[원 PR #8615](https://github.com/edwardkim/rhwp/pull/8615)의 원 head는 `7d1d9b51f2f2cc9f92aef46eefe101a6d4b433a5`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `7d1d9b51f2f2cc9f92aef46eefe101a6d4b433a5` | `387dc39efe764805e02d879c77a551a7767ab31c` | fix(layout): 칸 안 빈 host 의 글 위아래 표 띠가 뒤 문단 앞 간격을 흡수한다 (#8610) |

## 최종 리베이스 후 적용 상태

검증 base `b768bee1c4c0023087687f79a0a95f12108743e9`, 검증 제품 head
`eb876d1416fb558c874f9efec65c5722a6769751`의 실제 Git 이력을 대조해 위 적용 SHA를 갱신했다.
[고유 56개 최종 적용표](../assets/planet6897_20261106/contributor-commit-map-final.tsv)는 원 기여 commit의
cherry-picked footer와 현재 적용 commit을 일대일로 확인한 결과다. 중복 patch·bridge의 기존 처리는 유지한다.
이번 b768bee1 리베이스에는 실제 파일 충돌이 없었습니다. index.lock 경합이 끝나고 lock이 없는 상태에서 계속했으며 lock을 삭제하거나 todo를 바꾸지 않았습니다. 제품 src/crates/tests/scripts와 지침의 바이트는 이전 후보와 같고 의존성 변경에 대한 새 검증을 수행합니다.
이력 확인과 현재 제품의 기능·회귀·시각 검증 판정은 구분하며, 필수 검증 완료 전에는 승인하지 않는다.

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
[양쪽 전쪽 TSV·실제 source/producer·정상 대조군](../assets/planet6897_20261106/pr8615/r26/native-provenance.json)을 보존했다.

이 선행 조건 뒤 새 tests/cases의 automatic_break_boundary를 추가한다. 절대 위치를 핀하지 않고
표 전체 폭의 내부 컷·실제 종료선·연락처 소속을 검사한다. 별도 모델의 HWP5/HWPX 저장도 실제
재파싱해 보존하며, 합성 비트 계약으로 꺾인 BackSlash와 중심선의 독립 해석을 대조한다.
집중 검사·최종 통합 전쪽 비교·전체 회귀·lint는 별도 실제 종료 결과를 확인한다.

최소 글꼴 환경의 정식 집중 nextest에서 새 경계선3건 모두 PASS, 실제 실행 exit0을 확인했다.
함께 실행한 보고서 수식·꼬리 소속2건도 PASS다. 전체 Rust 회귀는 이후 별도 실행 중이다.

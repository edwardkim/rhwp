# PR #8604 리뷰 — fix(typeset): #8596 앞 TAC 표 host 줄 안에 저장된 자리차지 표 프레임은 다음 쪽에서 시작한다

## 최종 판정

**메인터너 보정 후 수용 가능 — 원 기여와 추가 보정을 포함한 통합 code head의 주장 범위를 검증했습니다. 원 contributor head의 직접 병합을 승인하는 판정은 아닙니다.**

원 PR head와 메인터너 보정을 포함한 통합 code head를 구분합니다. 위 exact code head의 필수 CI를 확인했습니다. 문서 trailing head의 실제 검사·충돌 여부를 다시 확인하며, 원 PR은 통합 병합을 확인한 뒤 종료합니다.

## 최종 source와 재사용 범위

제출 code head `2e262d0d4b2885b130e2db917b86e894710425e4`, base `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`입니다. 아래 전체 회귀·전쪽 수치는 원 실측 HEAD의 증거를 재사용한 기록입니다. 최신 집중 검사와 페이지별 producer, 엄격 0.1%p 잔여 한 건은 [재사용·최종 판정](../assets/planet6897_20261106/final-b768bee1-20261109/validation-reuse.md)을 따릅니다. 보호 TSV 표시값 하락은 0입니다.

## 통합 PR의 exact code head CI

- [통합 PR #8857](https://github.com/edwardkim/rhwp/pull/8857), code head `29c74b69851bc6578f10a2c78e23592a05b69a5d`.
- [CI 실행](https://github.com/edwardkim/rhwp/actions/runs/42621490405)과 같은 code head의 CodeQL·Render Diff·Adapter·Proptest·정책 검사를 실제 확인했습니다. 기록 확인 시각: `2026-11-09T22:58:24.689803+09:00`.
- 기본 기능 worker A–E의 실제 nextest는 **12,934 PASS / 0 FAIL / 45 skip**입니다. 기존 로컬 13,105건과 실행 범위·HEAD가 다르므로 합산하지 않습니다.
- source·test·workflow를 추가로 변경하지 않고 이 검토·오늘할일만 단일 부모 trailing commit으로 보완합니다. trailing head의 실제 CI/fast-pass와 mergeability는 push 뒤 별도로 확인하며, 문서만 바뀌었다는 이유로 통과를 미리 가정하지 않습니다.
- [잔여 이슈 #8856](https://github.com/edwardkim/rhwp/issues/8856)은 9문서군의 글리프·간격·도형 및 clip5587 Native 4쪽의 엄격 0.1%p 미충족을 계속 추적합니다. 부분 해결 #8635·#8636과 부모 #7445는 일괄 종료하지 않습니다.

## 접수 정보

- 원 PR: [#8604](https://github.com/edwardkim/rhwp/pull/8604), 작성자 `planet6897`, reviewer `jangster77`.
- 원 head: `013459bff4ff5313af53ece5646e79c48647a3aa`. 마지막 실시간 확인과 상태는 [조회 기록](../assets/planet6897_20261106/final-b768bee1-20261109/original-pr-heads.json)을 따릅니다.
- 통합 base: `b768bee1c4c0023087687f79a0a95f12108743e9`, 검증 code head: `eb876d1416fb558c874f9efec65c5722a6769751`.
- 브랜치: `review/integration-20261106-planet6897-open`. 통합 PR #8857을 제출했고 위 code head CI를 완료했습니다. 최종 trailing head 검사·병합·원 PR 후속 종료는 별도로 확인합니다.

## 변경과 검토 범위

fix(typeset): #8596 앞 TAC 표 host 줄 안에 저장된 자리차지 표 프레임은 다음 쪽에서 시작한다

[이 PR의 구현·메인터너 보정 계획](pr_8604_review_impl.md) · [통합 범위와 적용 순서](../assets/planet6897_20261106/integration-plan.md) · [원본→실제 적용 커밋](../assets/planet6897_20261106/contributor-commit-map-final.tsv).

## 검증 입력과 결과

code head `eb876d1416fb558c874f9efec65c5722a6769751`의 최소 글꼴 환경 기본 전체 회귀는 **13,105 PASS / 0 FAIL / 45 skipped**, 실행 570.421초입니다. Native 전쪽은 **94조건·2,995쪽**, 최저 **92.24%**, 90% 미만·쪽수 불일치·누락은 0입니다. 보호 TSV 1,771쪽의 공식 두 자리 표시값 하락은 0입니다. 이는 전체 범위의 요약이며 각 PR 주장은 아래 해당 입력·페이지로 대조합니다.

Native HWPX 편람 최소 글꼴/공개 웹폰트 383쪽 gate도 통과했습니다. 공식 두 자리 표시값은 130쪽 99.96%, 나머지 382쪽 100.00%입니다. 이는 보정 전 2px 내용 실루엣 값이며 엄격 RGB 동일성을 뜻하지 않습니다.

일반 base 대비 하락 허용폭은 이번 통합에 사용자가 지정한 **0.1%p**를 유지합니다. 별도 정본 보호 TSV의 공식 두 자리 표시값은 하락을 허용하지 않습니다. 최신 base `b768bee1`와 리베이스 전 후보를 같은 입력·PDF·글꼴·DPI·호환 버전으로 전쪽 대조했습니다. [Native 비교](../assets/planet6897_20261106/final-b768bee1-20261109/comparisons/latest-base-native-comparison.tsv) · [fresh WASM 비교](../assets/planet6897_20261106/final-b768bee1-20261109/comparisons/latest-base-wasm-comparison.tsv). 완료 쪽의98% 미만 후퇴·일반2%p 초과 하락은 없습니다. 엄격0.1%p 비교의 Native 뒤배경 표4쪽 잔여0.14171%p는 사용자 지시에 따라 신규 이슈로 분리하며 무회귀로 표기하지 않습니다.

관련 회귀 source: [tests/cases/issue_8596_tac_host_stored_frame_rewind.rs](../../../tests/cases/issue_8596_tac_host_stored_frame_rewind.rs)

## 시각 증적과 남은 차이

fresh WASM도 **94조건·2,995쪽**, 최저 **92.29%**, 90% 미만·누락·쪽수 불일치 0이며 보호 1,771쪽의 공식 표시값 하락은 0입니다. 추가 Native 전용 11개 모듈은 **73 PASS / 0 FAIL**입니다. 기본 전체 회귀의 45 skipped와 선택 밖 검사는 구분합니다.

### certificate

- 입력: [samples/issue8596/certificate_table.hwpx](../../../samples/issue8596/certificate_table.hwpx)
- 독립 Print: [pdf/issue8596_certificate_table-2020.pdf](../../../pdf/issue8596_certificate_table-2020.pdf)
- 전체 9쪽 Native 최저 100.00%. [TSV](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native.tsv) · [입력/PDF·producer 해시](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/provenance.json).
- 전체 9쪽 fresh WASM 최저 100.00%. [TSV](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm.tsv).

2쪽 Native **100.00%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/raw-002.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/pdf-002.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/review-002.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/overlay-002.png)

2쪽 fresh WASM **100.00%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/raw-002.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/pdf-002.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/review-002.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/overlay-002.png)

자격증 표 둘째 쪽의 전자·원자력·금속·섬유·일반화공·자원·농업·축산·생명유전 행과 각 세부 자격증의 줄 소속·병합·테두리를 확인했다. 내용 잘림과 행 중복이 없다.

| 경로 | raw | review | overlay |
| --- | --- | --- | --- |
| Native | ![certificate 2쪽 Native raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/raw-002.png) | ![certificate 2쪽 Native review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/review-002.png) | ![certificate 2쪽 Native overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/overlay-002.png) |
| fresh WASM | ![certificate 2쪽 fresh WASM raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/raw-002.png) | ![certificate 2쪽 fresh WASM review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/review-002.png) | ![certificate 2쪽 fresh WASM overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/overlay-002.png) |

머리행 직렬/직류·대상 자격증부터 첫 전자 행과 마지막 생명유전 행까지 표의 열·병합·행 경계와 텍스트 소속을 독립 Print와 대조했다. 앞 TAC host의 프레임에 다음 쪽 표가 끌려 들어오지 않고 이 쪽 첫·끝 행 및 줄 수가 보존된다. 획의 래스터 차이는 남는다.

3쪽 Native **100.00%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/raw-003.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/pdf-003.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/review-003.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/overlay-003.png)

3쪽 fresh WASM **100.00%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/raw-003.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/pdf-003.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/review-003.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/overlay-003.png)

자격증 표 셋째 쪽의 녹지조경·산림·해양수산·선박·해양교통시설 행과 긴 세부 항목의 줄바꿈·최하단 행 보존을 확인했다. 열별 본문이 다른 셀로 넘어가지 않는다.

| 경로 | raw | review | overlay |
| --- | --- | --- | --- |
| Native | ![certificate 3쪽 Native raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/raw-003.png) | ![certificate 3쪽 Native review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/review-003.png) | ![certificate 3쪽 Native overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/native/overlay-003.png) |
| fresh WASM | ![certificate 3쪽 fresh WASM raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/raw-003.png) | ![certificate 3쪽 fresh WASM review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/review-003.png) | ![certificate 3쪽 fresh WASM overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/certificate/wasm/overlay-003.png) |

녹지조경으로 시작해 해양교통시설로 끝나는 표의 머리행·좌측 분류·자격증 문단·마지막 셀과 아래 경계를 독립 Print와 대조했다. 앞2쪽과의 내용 이어받기·각 쪽 소속이 유지되고 끝 행이 잘리거나 중복되는 모습은 없다.

### policy-agenda

- 입력: [samples/issue8596/policy_committee_agenda.hwp](../../../samples/issue8596/policy_committee_agenda.hwp)
- 독립 Print: [pdf/issue8596_policy_committee_agenda-2020.pdf](../../../pdf/issue8596_policy_committee_agenda-2020.pdf)
- 전체 14쪽 Native 최저 98.40%. [TSV](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/native.tsv) · [입력/PDF·producer 해시](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/provenance.json).
- 전체 14쪽 fresh WASM 최저 98.40%. [TSV](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/wasm.tsv).

11쪽 Native **100.00%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/native/raw-011.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/native/pdf-011.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/native/review-011.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/native/overlay-011.png)

11쪽 fresh WASM **100.00%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/wasm/raw-011.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/wasm/pdf-011.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/wasm/review-011.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/wasm/overlay-011.png)

연구자 선정 심의결과서 상단 선택 항목과 수의계약 사유, 연구자·선정 사유·심의 결과·비고·하단 주석의 소속을 대조했다. 큰 셀의 빈 여백을 보존하며 일부 본문 래스터와 괘선 차이는 남는다.

| 경로 | raw | review | overlay |
| --- | --- | --- | --- |
| Native | ![policy-agenda 11쪽 Native raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/native/raw-011.png) | ![policy-agenda 11쪽 Native review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/native/review-011.png) | ![policy-agenda 11쪽 Native overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/native/overlay-011.png) |
| fresh WASM | ![policy-agenda 11쪽 fresh WASM raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/wasm/raw-011.png) | ![policy-agenda 11쪽 fresh WASM review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/wasm/review-011.png) | ![policy-agenda 11쪽 fresh WASM overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/policy-agenda/wasm/overlay-011.png) |

연구자 선정 심의결과서 제목, 위 정책연구과제/연구방식 표, 아래 연구자·선정사유·심의결과·비고 표와 마지막 유의사항을 같은 Print 쪽과 대조했다. 제목·두 표·비고·끝 줄의 순서와 분리, 체크 기호와 줄 기준선이 유지된다. 글자 윤곽·AA 차이를 raw100%와 구분한다.

### idol-counterexample

- 입력: [samples/issue8596/idol_goods_trimmed.hwpx](../../../samples/issue8596/idol_goods_trimmed.hwpx)
- 독립 Print: [pdf/issue8596_idol_goods_trimmed-2020.pdf](../../../pdf/issue8596_idol_goods_trimmed-2020.pdf)
- 전체 2쪽 Native 최저 99.76%. [TSV](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native.tsv) · [입력/PDF·producer 해시](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/provenance.json).
- 전체 2쪽 fresh WASM 최저 99.76%. [TSV](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm.tsv).

1쪽 Native **99.76%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/raw-001.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/pdf-001.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/review-001.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/overlay-001.png)

1쪽 fresh WASM **99.76%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/raw-001.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/pdf-001.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/review-001.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/overlay-001.png)

YG플러스 상품 설명의 앞뒤 본문, 두 이미지와 중앙 관련법리 상자를 대조했다. 내용/줄 소속은 유지되나 중앙 상자 테두리가 후보는 점선·Print는 실선으로 보이는 차이와 이미지 필터링 차이를 남긴다.

| 경로 | raw | review | overlay |
| --- | --- | --- | --- |
| Native | ![idol-counterexample 1쪽 Native raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/raw-001.png) | ![idol-counterexample 1쪽 Native review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/review-001.png) | ![idol-counterexample 1쪽 Native overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/overlay-001.png) |
| fresh WASM | ![idol-counterexample 1쪽 fresh WASM raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/raw-001.png) | ![idol-counterexample 1쪽 fresh WASM review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/review-001.png) | ![idol-counterexample 1쪽 fresh WASM overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/overlay-001.png) |

상단 응모 취소 문구·YG SELECT 이미지 두 개·가운데 관련 법리 상자가 원래 순서로 보존된다. 관련 법리 테두리 점선/실선 및 이미지 래스터 잔차는 남는다.

2쪽 Native **100.00%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/raw-002.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/pdf-002.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/review-002.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/overlay-002.png)

2쪽 fresh WASM **100.00%**: [raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/raw-002.png) · [PDF raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/pdf-002.png) · [review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/review-002.png) · [overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/overlay-002.png)

둘째 쪽의 마. 임의적인 하자 기준을 고지하여 청약철회 방해 제목·배경과 손가락 기호를 확인했다. 앞 쪽 그림/법리 상자가 이 쪽에 중복되지 않는다.

| 경로 | raw | review | overlay |
| --- | --- | --- | --- |
| Native | ![idol-counterexample 2쪽 Native raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/raw-002.png) | ![idol-counterexample 2쪽 Native review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/review-002.png) | ![idol-counterexample 2쪽 Native overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/native/overlay-002.png) |
| fresh WASM | ![idol-counterexample 2쪽 fresh WASM raw](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/raw-002.png) | ![idol-counterexample 2쪽 fresh WASM review](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/review-002.png) | ![idol-counterexample 2쪽 fresh WASM overlay](../assets/planet6897_20261106/final-b768bee1-20261109/cases/idol-counterexample/wasm/overlay-002.png) |

다음 항목인 임의적 하자 기준 제목이 독립된 다음 쪽 상단에 있으며 앞쪽 이미지를 중복하지 않는다. 실제 본문·테두리 반례 보존은 1쪽과 함께 판정한다.

## 최종 제출과 원격 조건

기존 전쪽 TSV·직접 판독을 유지하고 리베이스 후 영향 2조건 8쪽과 앞선 HWP3 3조건 10쪽과 필수 집중·lint를 추가 검증했습니다. 엄격0.1%p 비교 잔여1건은 새 이슈로 분리하며 완전 무회귀로 표기하지 않습니다. 원격 게시 직전에 원 PR head·upstream/devel·제출 diff와 병합 시뮬레이션을 재확인합니다. 통합 PR의 실제 CI 성공과 최종 head의 mergeability를 확인한 뒤 승인된 병합·후속처리를 수행합니다.

## Merge 후 contributor PR comment 계획

통합 병합이 완료되면 최종 merge SHA·CI URL과 이 PR이 주장한 입력/페이지의 raw 이미지를 고정 SHA 링크로 연결합니다. 원 기여와 메인터너 보정 이유·범위를 한국어 존댓말로 구분하며, 별도 검증하지 않은 문제까지 해결했다고 쓰지 않습니다. 원 PR의 종료는 통합 병합 후 실제 해결 범위를 확인하여 처리합니다.

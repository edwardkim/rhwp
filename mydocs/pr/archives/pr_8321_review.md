# [PR #8321](https://github.com/edwardkim/rhwp/pull/8321) 리뷰 — fix(text): 기호 슬롯 글자는 기호 슬롯 글꼴의 실제 전진폭으로 잰다 — 폭 표 보충·‥ … 반각 규칙 ([이슈 #8315](https://github.com/edwardkim/rhwp/issues/8315), [PR #8316](https://github.com/edwardkim/rhwp/pull/8316) 위)

## 최종 판정

**통합 PR에 반영됨 — 원 PR별 최종 검토 진행 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 머지됐다. 이 원 PR의 주장·변경 범위와 독립 증거를 대조한 뒤 원 PR 종료 여부를 판정한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `fa80cd8e7f` → `a9febd969c`, `f9c5d9af3c` → `223d27551c`, `11b41f459d` → `a3151b018b`, `ae417174ef` → `e262eeecbf`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_4962_font_metric_coverage` 8건, `issue_8315_symbol_slot_advance` 3건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

### 리베이스 차이 대조

원본 `fa80cd8e7f`와 통합 `a9febd969c`의 range-diff는 주변 글꼴 모듈 배치와 체리픽 메타데이터 차이를 표시한다. 기호 슬롯 폭 측정 변경과 회귀 본문은 대응한다. 전체 최종 기능과 시각 판정은 별도로 확인한다.

## 최종 코드의 공통 기호 슬롯 입력

같은 `lang-slot-ranges.hwpx` 두 쪽을 최종 제품 코드 `d536455b4e`의 Native·fresh WASM과 독립 한컴 2024 Print로 비교했다. 전쪽 최저는 각각 98.96%, 98.37%이고 전쪽 쪽수가 일치한다. 상세 실행·확대 판독 근거와 [Native/WASM 전쪽 TSV](pr_8316_review.md#최종-코드의-기호-슬롯곡선-따옴표-대조)를 참고한다. 이 결과와 원 PR의 다른 영향 범위 확인은 구분한다.

이 PR의 기호 슬롯 전진폭과 관련된 같은 입력의 자료를 직접 연결한다.
[Native 전쪽 TSV](../assets/issue_8136/final-integration-8438/native-lang-slot.tsv) ·
[WASM 전쪽 TSV](../assets/issue_8136/final-integration-8438/wasm-lang-slot.tsv) ·
[2쪽 review](../assets/issue_8136/integration-planet6897/native-review-002.png) ·
[2쪽 overlay](../assets/issue_8136/integration-planet6897/native-overlay-002.png).
이 자료는 위에 적은 통합 제품 코드의 결과이며 현재 보정 후보의 재측정 결과는 아니다.

2026-11-03 현재 제품 코드 `dd284b5cda`의 Native 실행 파일
(SHA-256 `b5c8835e5e3da6b55d5ef4e403ec66819f6a38224dbd91509ee5b732715dd006`)로
동일 입력과 한컴 2024 Print를 다시 대조했다. [전쪽 TSV](../assets/issue_8315/followup8438/native-current.tsv)의
1·2쪽은 각각 99.90%·98.96%로 위 통합 제품 코드의 표시값과 같다. 2쪽의
[비교](../assets/issue_8315/followup8438/native-review-002.png)와
[overlay](../assets/issue_8315/followup8438/native-overlay-002.png)에서 기호와
앞뒤 글자의 위치를 확인했다. 이는 Native 결과이며 현재 코드의 fresh WASM
재측정과 이 PR 전체의 최종 판정을 대신하지 않는다.

같은 제품 코드로 새로 빌드한 WASM(SHA-256 `cfe1e98fc6c4d7b2a4ac2fa23c6565fc3b32afe2d4ecf2ce71880d22318e86d8`)에서도 같은 두 쪽을 독립 Print와 비교했다. [fresh WASM 전쪽 TSV](../assets/issue_8315/followup8438/wasm-current.tsv)의 1·2쪽은 각각 99.86%·98.37%이며, 기존 통합 결과의 표시값과 같다. [2쪽 비교](../assets/issue_8315/followup8438/wasm-review-002.png)와 [overlay](../assets/issue_8315/followup8438/wasm-overlay-002.png)를 함께 보존한다. 2쪽 `00AD` 행은 한컴 Print에 하이픈이 보이지만 RHWP에는 보이지 않는다. HWPX에는 리터럴 U+00AD가 있고 Print의 텍스트 추출은 하이픈이다. 이전 [통합 후보의 같은 WASM 2쪽](../assets/issue_8136/integration-planet6897/wasm-review-002.png)과 현재 후보의 왼쪽 출력 영역을 픽셀 대조한 결과 같아 이번 저장 줄 보정에서 새로 생긴 차이는 아니다. 원 PR은 Latin-1 문자를 기존대로 유지한다고 명시한다. 따라서 이 관측을 기호 슬롯 개선의 증거로 쓰지 않으며, 문자 표시의 독립 원인과 수정 여부는 계속 확인한다. 이 입력의 전쪽 측정 결과이며 PR 전체의 최종 판정은 별도로 진행한다.

소프트 하이픈의 선행 [이슈 #7993](https://github.com/edwardkim/rhwp/issues/7993)과 [PR #8026](https://github.com/edwardkim/rhwp/pull/8026)을 대조했다. 한컴은 제어 표기는 줄 끝에서만 표시하고 리터럴 U+00AD는 일반 글자로 표시한다. 기존 `issue_7993_hwpx_hypen_element`의 리터럴 검사는 display 문자열에 U+00AD가 남는지만 확인하므로, 실제 Native·WASM 출력에서 글리프가 보인다는 증거는 아니다. 제어 표기의 동작과 원문 보존을 유지하면서 표시·폭을 검증해야 한다.

2026-11-03 00시경 KST에도 원 PR head와 열린 상태를 재확인했다. [일괄 조회 결과](../assets/review_8438_followup/original-pr-heads-20261103.md)는 기능·시각 최종 판정을 대신하지 않는다.

## 접수와 적용

- 원 PR: [PR #8321](https://github.com/edwardkim/rhwp/pull/8321), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `ae417174ef01e6033cf6af6388c1df0d568df05e`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `ae417174ef01e6033cf6af6388c1df0d568df05e`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #8315](https://github.com/edwardkim/rhwp/issues/8315).
- 선행 PR: [PR #8316](https://github.com/edwardkim/rhwp/pull/8316).

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

기호 슬롯 문자의 확인된 글꼴 전진폭을 사용하고 일부 문장부호의 반각 처리를 교정한다.
선행 [PR #8316](https://github.com/edwardkim/rhwp/pull/8316)의 슬롯 선택과 폭 계산이 같은 글자에 적용되는지 확인한다.

## 체리픽과 보정

- 원본 `fa80cd8e7f07f1f6d9daac5710c9dd4580cd7d3f` → 통합 `2685b526dced6ae26850be21bbefcee4682ac1bd`.
- 원본 `f9c5d9af3c6b3910712d53bc9075589d15e5afad` → 통합 `89f575a9be88290f26cd3fe66f4ca1ee9f19819d`.
- 원본 `11b41f459dba74563ef5e86008a6ca21fbd0d2cf` → 통합 `d6b5b19b30e14d913d123651df75dbc01efc456c`.
- 원본 `ae417174ef01e6033cf6af6388c1df0d568df05e` → 통합 `281f2af3b16fdbe20df47a21c61b0779cd928ca1`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_4962_font_metric_coverage.rs`](../../../tests/cases/issue_4962_font_metric_coverage.rs): 이전 실행 8건 통과, 0건 실패, 0건 건너뜀.
- [`issue_8315_symbol_slot_advance.rs`](../../../tests/cases/issue_8315_symbol_slot_advance.rs): 이전 실행 3건 통과, 0건 실패, 0건 건너뜀.

당시 코드에서 `issue_8315_symbol_slot_advance` 3건을 최소 글꼴 환경으로 다시 실행해 모두 통과했다. 같은 실행은 [이슈 #8079](https://github.com/edwardkim/rhwp/issues/8079)·[이슈 #8136](https://github.com/edwardkim/rhwp/issues/8136)을 포함한 10건이며, 실행 로그 SHA-256은 `91abb68a1a9efb4e796fd1bc12b5bd073a2c5759d4624128ea629bdf8447fc85`다. [PR #8321](https://github.com/edwardkim/rhwp/pull/8321)의 나머지 시각·회귀 검증 전체를 대신하지 않는다.

기여자가 제출한 기준 자료: [`ab_changed_pages_native.tsv`](../../../mydocs/pr/assets/issue_8315/ab_changed_pages_native.tsv). 해당 자료만으로 새 통합 후보의 시각 검증을 통과 처리하지 않는다.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`의 실행 파일에서 `issue_8315_symbol_slot_advance`
3건이 최소 글꼴 환경에서 모두 통과했다. 이 검사는 위에서 확인한 문서의 전체
페이지 시각 검증을 대체하지 않는다.

### 2026-11-02 의존성 리베이스 후 전체 Rust 회귀

최소 글꼴 환경에서 전체 선택 12,369건을 두 실행으로 완료해 모두 통과했다.
첫 실행의 중단과 나머지 307건의 재실행, 정확한 목록 대조는
[공통 실행 근거](../assets/review_8438_followup/rebase-de2295-rust.md)에 기록했다.
이는 이 PR의 시각 검증이나 최종 판정을 대신하지 않는다.

### 2026-11-03 리터럴 소프트 하이픈 대조

후속 후보에서 본문 리터럴 U+00AD와 줄 끝에서만 표시하는 소프트 하이픈
제어 표기를 구분했다. 한컴 2024 Print를 기준으로 기존 회귀 검사에서 모델 문자
보존과 출력 하이픈을 각각 확인하고, 최소 글꼴 환경에서 관련 회귀 2건이 모두
통과했다. 같은 실행의 총 7건도 통과했으며 로그 SHA-256은
`edf38a412efb58baa537cfcc40f5036c87c4951faf065f2b83998db8826d927e`다.
이는 수정 중인 후보의 집중 검사이며 최종 Native·fresh WASM 시각 검증은 진행 중이다.

WMFC 폴백과 리터럴 소프트 하이픈을 함께 보정한 후보로 동일한 원본과 한컴
2024 Print를 96 DPI에서 비교했다. Native는 전 2쪽 중 1쪽 99.90447%, 2쪽
98.94495%이며 누락 쪽과 90% 미만 쪽은 없다. 2쪽의 U+00AD 리터럴은 한컴처럼
하이픈으로 보인다. [2쪽 review](../assets/issue_8315/followup8438/native-hyphen-review-002.png)와
[overlay](../assets/issue_8315/followup8438/native-hyphen-overlay-002.png)를 직접 확인했다.
[후보 Native 전체 TSV](../assets/issue_8315/followup8438/native-hyphen-candidate.tsv)를
산출했으며 fresh WASM은 별도로 진행 중이다.

같은 보정이 [PR #8026](https://github.com/edwardkim/rhwp/pull/8026)의 별도 HWP
리터럴 대조에도 적용되는지 확인했더니 마지막 줄의 하이픈 뒤 간격이 벌어진다.
독립 Print에서 `와`의 시작은 174.505425pt(96 DPI에서 약 232.67px)인데,
보정 전 렌더 트리는 232.7px, 현재 후보는 238.7px이다. `K­MOOC ` 런의 폭이
61.7px에서 67.8px로 바뀌었고, 표시 문자열 `-`가 공통 폭 측정에도 사용되는
것을 확인했다. 문자 모양만 바꾸고 원문 메트릭을 유지해야 하는 리터럴과 실제
줄 끝 하이픈의 전진을 구분하는 공통 계약이 필요하다. 현재 후보는 이 차이의
보정과 재검증을 완료하기 전이므로 최종 통과로 판정하지 않는다.

독립 PDF의 글자별 bounding box도 확인했다. `K­MOOC`에서 리터럴 U+00AD의
잉크는 x=136.383–138.922pt이고 다음 `M`은 x=139.882pt부터 시작한다.
따라서 리터럴은 보이는 글자이면서 독립 PDF의 전진 관계를 유지해야 한다.
단순히 글자를 감추거나 `-`의 기본 전진을 더하는 방식은 이 근거와 다르다.

리터럴의 원문 메트릭과 표시 글리프를 공통 런·스타일 경로에서 분리했다.
최소 글꼴 집중 회귀 8건은 모두 통과했다
(`output/pr-review/planet6897-ordered-20261031/logs/book-validation/literal-hyphen-metrics-focused-rerun.log`).
새 Native 실행 파일 SHA-256은
`7d35357fe91cf213bb4143675a7c148bf3f6a9e4facbf93fea765081661b25c5`다.
같은 독립 Print PDF를 96 DPI로 다시 대조한 결과 #7993 HWP 1쪽 99.99638%,
HWPX 1쪽 99.40819%, #8136 HWPX 1·2쪽 99.90447%·98.95550%다.
HWP의 마지막 줄 `와` 시작은 현재 렌더 트리 x=232.7px로, 독립 Print의
174.505425pt(약 232.67px)와 일치한다. [#7993 review](../../../output/pr-review/planet6897-ordered-20261031/book-validation/literal-hyphen-final-native/issue7993-hwp/review/review_001.png)·
[overlay](../../../output/pr-review/planet6897-ordered-20261031/book-validation/literal-hyphen-final-native/issue7993-hwp/overlay/overlay_001.png),
[#8136 2쪽 review](../../../output/pr-review/planet6897-ordered-20261031/book-validation/literal-hyphen-final-native/issue8136-hwpx/review/review_002.png)·
[overlay](../../../output/pr-review/planet6897-ordered-20261031/book-validation/literal-hyphen-final-native/issue8136-hwpx/overlay/overlay_002.png)를 직접 확인했다.
페이지별 표시값 TSV는 `output/pr-review/planet6897-ordered-20261031/book-validation/literal-hyphen-final-native-tsv/`에 있다.
이 단계의 수치는 Native 결과이며 fresh WASM과 #8389·#8397 최종 전쪽 대조를
완료한 뒤 최종 판정을 갱신한다.

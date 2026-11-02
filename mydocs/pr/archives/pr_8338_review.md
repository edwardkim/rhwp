# PR #8338 리뷰 — fix(serializer): HWP↔HWPX 저장의 개체 번호 범주 비트를 numberingType에서 만든다 — 그림 캡션 번호 보존 (#8335)

## 최종 판정

**메인터너 검토 완료 — 저장 왕복의 캡션 범주와 번호 확인.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에 반영됐고 원 PR head가 검토한 head와 일치한다. 미완료 #7445 하위 이슈의 문서는 작업지시자 지시에 따라 이 판정에서 제외했다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `707465a995` → `1903cdf1d2`, `0ac95d8c95` → `444b20c126`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8335_caption_category_roundtrip` 4건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

### 리베이스 차이 대조

원본 `707465a995`와 통합 `1903cdf1d2`의 range-diff에서 범용 Storage 인자와 기존 직렬화 함수 이름 등 새 base 문맥 차이를 확인했다. 번호 범주 속성 비트 갱신과 회귀 변경은 대응한다. 전체 최종 기능과 시각 판정은 별도로 확인한다.

## 접수와 적용

### 통합 후 저장 왕복과 캡션 대조

제품 코드 `d536455b4e`의 기존 전체 최소 글꼴 회귀에서 이 PR의 `issue_8335_caption_category_roundtrip` 네 검사가 통과했다. 다음 입력과 같은 한컴 2020 Print PDF를 96 DPI, 호환 모드 2022로 전쪽 비교했다. 실제 저장본을 함께 보존하며 페이지 수는 모든 행에서 PDF와 같다.

| 입력·저장 경로 | Native 전쪽 TSV·최저 | WASM 전쪽 TSV·최저 |
| --- | --- | --- |
| 정책 HWPX→[HWP](../assets/issue_8335/followup8438/policy-saved.hwp), 64쪽 | [TSV](../assets/issue_8335/followup8438/native-policy.tsv), 92.61% | [TSV](../assets/issue_8335/followup8438/wasm-policy.tsv), 92.41% |
| 정책 HWP→HWPX→[HWP](../assets/issue_8335/followup8438/policy-roundtrip.hwp), 64쪽 | [TSV](../assets/issue_8335/followup8438/native-policy-roundtrip.tsv), 92.61% | [TSV](../assets/issue_8335/followup8438/wasm-policy-roundtrip.tsv), 92.41% |
| 국어 시험지 원본 HWP, 20쪽 | [TSV](../assets/issue_8335/followup8438/native-exam-hwp.tsv), 96.44% | [TSV](../assets/issue_8335/followup8438/wasm-exam-hwp.tsv), 96.93% |
| 국어 시험지 HWP→[HWPX](../assets/issue_8335/followup8438/exam-converted.hwpx), 20쪽 | [TSV](../assets/issue_8335/followup8438/native-exam-hwpx.tsv), 96.44% | [TSV](../assets/issue_8335/followup8438/wasm-exam-hwpx.tsv), 96.93% |
| HWP3 글꼴 문서 HWP→HWPX→[HWP](../assets/issue_8335/followup8438/h3native-roundtrip.hwp), 16쪽 | [TSV](../assets/issue_8335/followup8438/native-h3native-roundtrip.tsv), 99.68% | [TSV](../assets/issue_8335/followup8438/wasm-h3native-roundtrip.tsv), 99.95% |

정책 원본은 [`1790387_policy_progress.hwpx`](../../../samples/issue4599/1790387_policy_progress.hwpx), PDF는 [`1790387_policy_progress-hwpx-2020.pdf`](../../../pdf/issue4599/1790387_policy_progress-hwpx-2020.pdf)다. 29쪽의 `[그림 2]`가 두 HWP 저장 경로와 PDF에서 같으며 [Native review](../assets/issue_8335/followup8438/native-policy-p029-review.png)·[overlay](../assets/issue_8335/followup8438/native-policy-p029-overlay.png), [WASM review](../assets/issue_8335/followup8438/wasm-policy-p029-review.png)·[overlay](../assets/issue_8335/followup8438/wasm-policy-p029-overlay.png)를 확인했다. 두 정책 HWP 결과는 Native와 WASM 각각 64쪽 PNG가 바이트 단위로 같았고, 시험지 HWP와 HWPX도 각 경로의 20쪽 PNG가 같았다.

HWP3 글꼴 문서의 네 캡션은 HWP 왕복 후에도 독립 PDF처럼 모두 `그림 1.`이다. 8쪽 [review](../assets/issue_8335/followup8438/native-h3native-p008-review.png)·[overlay](../assets/issue_8335/followup8438/native-h3native-p008-overlay.png)를 대조했다. [항공교통관제사 문서 #7727](https://github.com/edwardkim/rhwp/issues/7727)은 미완료 #7445 하위 이슈라 이번 판정에 넣지 않는다. 범위가 남은 이슈 #8335 전체를 닫았다고 주장하지 않는다.

- 원 PR: [#8338](https://github.com/edwardkim/rhwp/pull/8338), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `0ac95d8c9599ad98b8f7601faf62243e58814d03`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `0ac95d8c9599ad98b8f7601faf62243e58814d03`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#8335](https://github.com/edwardkim/rhwp/issues/8335).
- 선행 PR: #8297.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

개체 번호 범주를 numberingType에서 생성해 저장 왕복 후 캡션 번호를 보존한다.
선행 #8297의 범주 파싱과 저장 비트가 같은 계약을 따르는지 확인한다.

## 체리픽과 보정

- 원본 `707465a9958c8ad65821eafb8df19ab0a1771ce4` → 통합 `01858daab44ead057b12f8acf0932fca86ce33de`.
- 원본 `0ac95d8c9599ad98b8f7601faf62243e58814d03` → 통합 `ea52d7741885250b5a41a3cd98ee6eac3808e7fc`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8335_caption_category_roundtrip.rs`](../../../tests/cases/issue_8335_caption_category_roundtrip.rs): 이전 실행 4건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`hwp3-native-font-program-final-hwp5-20261017-2020.pdf`](../../../pdf/hwp3-native-font-program-final-hwp5-20261017-2020.pdf) — SHA-256 `579648569f6b2ef7d16b4b094525407c9b5b737c5139d8d405d67ee42bd70fca`.
- [`1790387_policy_progress-hwpx-2020.pdf`](../../../pdf/issue4599/1790387_policy_progress-hwpx-2020.pdf) — SHA-256 `4c0d9d0876410766d99f4589916971f97b751a3d71901068637c7e2cc1061f95`.
- [`exam_kor-hwp-2020.pdf`](../../../pdf/exam_kor-hwp-2020.pdf) — SHA-256 `d67a5a0c759dcc8609226f89ef96de5339e3b41c43c2655be3824dcde1732f22`.

기여자가 제출한 기준 자료: [`ek_hwpx_silhouette.tsv`](../../../mydocs/pr/assets/issue_8335/native/ek_hwpx_silhouette.tsv). 해당 자료만으로 새 통합 후보의 시각 검증을 통과 처리하지 않는다.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

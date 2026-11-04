# PR #8404 리뷰 — fix(pdf-import): 0.5em 올린 위 첨자를 부동소수 오차와 무관하게 자기 줄에 묶는다 (#8381 일부)

## 최종 판정

**메인터너 검토 완료 — 위 첨자의 편집 줄 소속 확인.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에 반영됐고 원 PR head가 검토한 head와 일치한다. #8381의 나머지 범위는 이 PR로 해결했다고 판정하지 않는다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `4d0dff6404` → `baf51260b3`, `f22136ca82` → `2a4155f0e0`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8381_pdf_superscript_raise` 1건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 접수와 적용

- 원 PR: [#8404](https://github.com/edwardkim/rhwp/pull/8404), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `f22136ca8291757e2b542603016bc16e32411db7`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `f22136ca8291757e2b542603016bc16e32411db7`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#8381](https://github.com/edwardkim/rhwp/issues/8381).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

PDF 가져오기에서 0.5em 위 첨자를 부동소수 오차로 별도 줄에 분리하지 않는다.
동일 줄 소속과 인접 줄을 잘못 합치지 않는 경계를 확인한다. #8381 전체 해결은 아니다.

### 14쪽 위 첨자와 전쪽 대조

제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀에서 `raised_superscript_stays_in_its_editable_row`가 통과했다. 이 검사는 원 PR의 원본 PDF 14쪽을 **직접 편집 가능 모드로 가져온 직후** `NAD+는 줄어든다.`의 줄 소속·출현 횟수와 26번의 `NAD+와` 대조군을 확인한다. 원 PR에서 지적한 효과는 편집 본문의 줄 소속이며 화면 실루엣만으로는 증명하지 않는다.

보조 시각 검증은 동일 PDF를 각각 editable·fidelity로 가져와 HWP로 저장하고 다시 연 결과다. 이는 직접 가져온 직후 화면의 대체 증거가 아닌 별도 저장본 비교다. 기준은 HWP의 한컴 Print가 아니라 **가져오기의 원본 PDF**다. Native와 fresh WASM 모두 96 DPI·2024 호환, 16/16쪽이며 90% 미만은 없다.

| 저장본 | 입력 | Native 전쪽 | WASM 전쪽 |
| --- | --- | --- | --- |
| editable | [HWP](../assets/issue_8381/followup8438/editable-saved.hwp) | [TSV](../assets/issue_8381/followup8438/native-editable.tsv), 최저 97.80% | [TSV](../assets/issue_8381/followup8438/wasm-editable.tsv), 최저 98.08% |
| fidelity | [HWP](../assets/issue_8381/followup8438/fidelity-saved.hwp) | [TSV](../assets/issue_8381/followup8438/native-fidelity.tsv), 최저 98.90% | [TSV](../assets/issue_8381/followup8438/wasm-fidelity.tsv), 최저 98.85% |

원 PR이 지목한 14쪽 [Native review](../assets/issue_8381/followup8438/native-editable-p014-review.png)·[overlay](../assets/issue_8381/followup8438/native-editable-p014-overlay.png)와 [WASM review](../assets/issue_8381/followup8438/wasm-editable-p014-review.png)·[overlay](../assets/issue_8381/followup8438/wasm-editable-p014-overlay.png)에서 대상 문항과 주변 문단을 확인했다. 저장본과 원 PDF의 14쪽 raw는 Native 98.93%, WASM 98.91%다.

## 체리픽과 보정

- 원본 `4d0dff6404ef23891038187e2cee42789f37bf68` → 통합 `d596bdff8a9d05a66d3460c10df2be180aa9a043`.
- 원본 `f22136ca8291757e2b542603016bc16e32411db7` → 통합 `de5a7a6c8fdb80c5b0c908fef5940ded476fcea8`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8381_pdf_superscript_raise.rs`](../../../tests/cases/issue_8381_pdf_superscript_raise.rs): 이전 실행 1건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`21_언어_기출_편집가능본-2010.pdf`](../../../samples/21_언어_기출_편집가능본-2010.pdf) — SHA-256 `609d56421009ccab5a50cb2a8f4e78af4d0961de2e7ae85d811905e8434ac251`.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

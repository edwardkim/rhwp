# PR #8357 리뷰 — fix(serializer): 문단 모양 줄간격의 5.0.2.5 필드(attr3·line_spacing_v2)를 모델 줄간격으로 다시 만든다 (#8352)

## 최종 판정

**메인터너 보정 후 수용 가능 — #8352 줄간격 필드와 한컴 2020 출력 검증 완료.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에 반영됐고 원 PR head가 검토한 head와 일치한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `9af918cff1` → `9cd3c467be`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8352_para_shape_line_spacing_fields` 3건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 한컴 2020 Print 추가 확인

원 PR이 남긴 2020 출력 차이를 확인하기 위해 저장소의 원본 PDF `samples/21_언어_기출_편집가능본-2010.pdf`(SHA-256 `609d56421009ccab5a50cb2a8f4e78af4d0961de2e7ae85d811905e8434ac251`)를 최종 제품 코드 `d536455b4e`의 `import-pdf --layout editable`로 HWP에 저장했다. 이 HWP를 다시 열고 `edit insert-text --section 0 --para 59 --offset 0 --text 'IR: '`로 편집한 [HWP](../../../samples/issue8352/editable-import-ir-edited.hwp)(SHA-256 `7beb002732304560d80e0befa002eae4e4d84c9298c96e9b3ead9efff327bae1`)를 한컴 2020 MCP의 `PrintToPDFEx`, PrintMethod 0으로 출력했다. [16쪽 Print PDF](../../../pdf/issue8352/editable-import-ir-edited-2020.pdf)의 SHA-256은 `b9e6fed95fa7cfa89507e9cc6845b3780888ead90bb8e8ef92830b150c4af022`, job은 `d0c6cfd6-d785-4c30-a060-0b0ef37036cf`다.

동일 입력의 전체 페이지 raw 최저는 Native 97.24%, fresh WASM 97.27%이고 모두 16쪽으로 일치한다. [Native TSV](../assets/issue_8352/followup-8438/native.tsv) · [WASM TSV](../assets/issue_8352/followup-8438/wasm.tsv). 편집 문단은 4쪽에서 확인되며 해당 문단의 Native 줄 원점 간격은 25.6 CSSpx, Print PDF의 연속 글자 경계 간격은 19~20pt(약 25.3~26.7 CSSpx)로 관측된다. [Native review](../assets/issue_8352/followup-8438/native-p004-review.png) · [Native overlay](../assets/issue_8352/followup-8438/native-p004-overlay.png) · [WASM review](../assets/issue_8352/followup-8438/wasm-p004-review.png) · [WASM overlay](../assets/issue_8352/followup-8438/wasm-p004-overlay.png).

추가로 PDF를 `DocumentCore::from_pdf_bytes`로 가져온 직후 메모리에서 `insert_text_native(0, 59, 0, "IR: ")`를 적용하고 바로 HWP로 저장하는 [직접 재현 코드](../assets/issue_8352/followup-8438/reproduce-direct.rs.txt)를 실행했다. 이 직접 경로의 HWP SHA-256이 위 HWP와 완전히 같은 `7beb002732304560d80e0befa002eae4e4d84c9298c96e9b3ead9efff327bae1`임을 확인했으므로 같은 한컴 Print·TSV·페이지 검토를 직접 경로에도 적용할 수 있다. 별도 저장·재열기를 거친 실행과 직접 실행을 동일한 바이트 결과로 대조한 것이다.

## 접수와 적용

- 원 PR: [#8357](https://github.com/edwardkim/rhwp/pull/8357), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `9af918cff1d62b59a2926c71529282d158f43dfc`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `9af918cff1d62b59a2926c71529282d158f43dfc`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#8352](https://github.com/edwardkim/rhwp/issues/8352).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

HWP5 문단 모양의 새 줄간격 필드와 기존 모델 표현을 저장·읽기에서 일관되게 유지한다.
줄간격 유형별 필드와 버전별 왕복 보존을 확인한다.

## 체리픽과 보정

- 원본 `9af918cff1d62b59a2926c71529282d158f43dfc` → 통합 `b2fbb23efb6231f04cbc8de7aabf1c2aeacc2931`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8352_para_shape_line_spacing_fields.rs`](../../../tests/cases/issue_8352_para_shape_line_spacing_fields.rs): 이전 실행 3건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`pr8227-editable-import-edited-2020.pdf`](../../../pdf/pr8227-editable-import-edited-2020.pdf) — SHA-256 `95709baa066aaf9547e1e2d1f3eef99d8bf5b661b025ef68c2ea03b760451bcc`.
- [`21_언어_기출_편집가능본-2010.pdf`](../../../samples/21_언어_기출_편집가능본-2010.pdf) — SHA-256 `609d56421009ccab5a50cb2a8f4e78af4d0961de2e7ae85d811905e8434ac251`.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR #8438의 merge `b8338ba38be288b6e9385f5372d184649323aa9f`, 코드 head CI와 해당 원 PR의 기여·메인터너 보정을 구분해 존댓말로 안내한다. 메모리에서 직접 편집한 동일 바이트 HWP와 한컴 2020 Print, 전체 Native/fresh WASM TSV 및 4쪽 review·overlay를 연결한다. 전체 최저는 97.24% / 97.27%다. 게시 시 명시적 지시를 확인하고, 게시 뒤 API로 본문과 링크를 재조회한다.

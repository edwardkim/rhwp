# 정본 시각 보호 기준

이 자산은 수용된 최종 전쪽 검증을 통합한 품질 하한이다. 현재 개발 후보의 검증 결과나 통과 선언이 아니다.
적용 역할과 판정은 [시각 검증 거버넌스](../../../manual/verification/visual_verification_governance.md#정본-보호-tsv의-공식-표시값-보존)를 따른다.
**통합 담당 collaborator·maintainer만 이 특별 보호 검증을 수행한다.** 일반 사용자·외부 contributor에게
비공개 원본 OTF 보유·설치나 이 기준 전체의 재현을 제출 조건으로 요구하지 않는다.
조판 영향 PR의 `--handbook-gate` 실행·통과는 일반 기여자도 준수하며, 필요한 글꼴은 지원 웹폰트를 사용할 수 있다.
기존 편람 gate도 이 TSV에서 편람 HWPX·기준 PDF·backend가 일치하는 383쪽만 선택한다.
이 연결은 다른 문서 전체의 자동 강제 실행을 추가하지 않는다.

## 정본 파일

- [final.tsv](final.tsv): 입력·기준 PDF·backend·페이지별 공식 두 자리 raw 보호값.
- [manifest.json](manifest.json): 입력/PDF SHA-256·크기·측정 조건·전쪽 수·출처 commit/TSV/열/해시와 중복 제거 내역.
- TSV SHA-256: `7eb001e3e30d6892d8bc03ba3e16456a4dd758cccb0e9aba073671366a9a5a1b`.
- 입력 18개, Native/fresh WASM 각 18개 데이터셋·1,771쪽, 총 36개 데이터셋·3,542행.
- 최초 통합 당시 수용 자료 56개 데이터셋 중 중복 20개를 정리했다. 이전 자료보다 공식 표시값이 낮아진 페이지는 0쪽이다.

## 문서별 범위

HWP와 HWPX는 같은 제목이어도 별도 입력이다. 아래 최저값은 요약이며 판정은 전쪽 각 행으로 한다.
파일명은 탐색용이며 실제 비교는 manifest의 입력/PDF SHA·조건과 TSV의 document_id·backend·page를 사용한다.

| 입력 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| [angle-2400.hwpx](<../issue_8371/angle-narrow/angle-2400.hwpx>) | 1 | 100.00% | 100.00% |
| [jongbu_law.hwpx](<../legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx>) | 6 | 98.86% | 98.86% |
| [2025 행정업무운영 편람(최종).hwp](<../../../../samples/2025 행정업무운영 편람(최종).hwp>) | 383 | 99.97% | 99.96% |
| [2025 행정업무운영 편람(최종).hwpx](<../../../../samples/2025 행정업무운영 편람(최종).hwpx>) | 383 | 99.97% | 99.96% |
| [issue2007_nested_cell_pagination_42065.hwp](<../../../../samples/basic/issue2007_nested_cell_pagination_42065.hwp>) | 17 | 99.98% | 99.98% |
| [hwp3-sample10.hwp](<../../../../samples/hwp3-sample10.hwp>) | 763 | 99.57% | 99.57% |
| [hwp3-sample16-hwp5-2022.hwp](<../../../../samples/hwp3-sample16-hwp5-2022.hwp>) | 64 | 100.00% | 100.00% |
| [hwp3-sample16-hwp5-2024.hwp](<../../../../samples/hwp3-sample16-hwp5-2024.hwp>) | 64 | 99.99% | 99.99% |
| [hwp3-sample16-hwp5.hwpx](<../../../../samples/hwp3-sample16-hwp5.hwpx>) | 64 | 99.99% | 99.99% |
| [20099369_yeongwol_forms.hwp](<../../../../samples/issue5699/20099369_yeongwol_forms.hwp>) | 4 | 100.00% | 100.00% |
| [pic-crop-01.hwp](<../../../../samples/pic-crop-01.hwp>) | 1 | 100.00% | 100.00% |
| [exam-social-insert-delete.hwp](<../../../../samples/pr8040/exam-social-insert-delete.hwp>) | 4 | 99.99% | 99.99% |
| [exam-social-insert.hwp](<../../../../samples/pr8040/exam-social-insert.hwp>) | 4 | 99.99% | 99.99% |
| [re-03-latin-only-hancom.hwp](<../../../../samples/re-03-latin-only-hancom.hwp>) | 1 | 100.00% | 100.00% |
| [shape-group-02.hwp](<../../../../samples/shape-group-02.hwp>) | 1 | 100.00% | 100.00% |
| [issue3307_outline_number.hwpx](<../../../../samples/task3307/issue3307_outline_number.hwpx>) | 9 | 100.00% | 100.00% |
| [body-list-typed-hwp.hwp](<../../../../tests/fixtures/issue_8109_saved_fonts/body-list-typed-hwp.hwp>) | 1 | 100.00% | 100.00% |
| [control-styles-hwp.hwp](<../../../../tests/fixtures/issue_8109_saved_fonts/control-styles-hwp.hwp>) | 1 | 100.00% | 100.00% |

## 2026-11-15 PR #9075 검토의 상승분 반영

동일한 최종 제품 소스 `43adf3f7fe412e71797456887e2be44b8ef8d14f`의 Native/fresh WASM
36개 전쪽 데이터셋·3,542행을 수용했다. 기존 하한보다 Native 15쪽·WASM 15쪽이 상승했고,
하락·누락은 0쪽이다. [최종 수용 조건](../issue_9060/final/protected-validation.json)과
[전체 TSV](../issue_9060/final/protected/README.md)를 먼저 `44ec0fa4ec5d3332100406f6b4779a299a022ee0`에 고정했다.
[갱신 영수증](updates/pr9075-20261115.json)에 이전/새 TSV 해시와 상승 30행의 값을 남긴다.
이전 source·previous_source는 source_history와 acceptance_sources에도 보존한다.
페이지별 최댓값을 조합하지 않았고 기존 입력·독립 PDF·공급 조건·2px raw 비교 방법은 유지한다.
편람 HWPX Native 383쪽의 보호값은 이전과 같으며, gate의 정본 해시 연결만 함께 갱신했다.
로컬 수용 완료 기록이며 원격 CI·병합 완료를 뜻하지 않는다.

## 2026-11-14 상승 버전 반영

사용자 지시에 따라 [최종 전쪽 검증](../planet6897_integration_20261114/protected-validation.json)의 36개 데이터셋 전체를 새 보호값으로 수용했다. Native58쪽·fresh WASM56쪽이 상승했고 하락은0쪽이다. 입력·PDF·측정 조건과 2px raw·공식 두 자리 비교는 그대로다. 페이지별 최대값을 조합하지 않았으며 [갱신 영수증](../planet6897_integration_20261114/protected-baseline-update.json)에 이전/현재 TSV 해시와 상승114행을 기록했다. 이전 출처는 manifest의 previous_source와 acceptance_sources에 보존한다. CI·병합 완료를 뜻하지 않는다.

## 출처 통합과 판정

최초 통합은 PR #8389와 #8397의 수용된 병합 commit을 사용했다. 현재 보호값은 위 2026-11-15 최종 전쪽 결과로 갱신했으며, 두 PR 번호는 과거 출처다. 새 검증의 선택 기준은 이 정본 TSV와 manifest이고 원래 자료는 삭제하거나 덮어쓰지 않는다.

1. 같은 입력 SHA·독립 Print PDF SHA·compat·DPI·인쇄 프로필·계산 조건·backend가 겹칠 때 최신 수용본을 선택한다.
2. 같은 수용 시점의 중간 표와 최종 재출력이 겹치면 명시된 최종 전쪽 표를 선택한다. 반복된 동등 대조 표는 하나로 정리한다.
   기존 편람·sample16·편집 입력은 최종 전쪽 재검증을 사용하고, 같은 법령 입력의 별도 표와 편람의 SEO 대조 표는 중복으로 기록했다.
   sample10·기존 회귀 입력·각도 대조군·나머지 SEO 입력은 각각 최종 수용 자료를 보존했다.
3. 원본 TSV에 공식 두 자리 값이 있으면 그대로 사용한다. 정밀 최종 재검증 표만 있는 경우 한 번 두 자리로 표시하며,
   기존 TSV 작성기의 `.2f`와 동일함을 확인했다. 원본 TSV SHA·페이지/값 열·문서 필터를 manifest에 남겼다.
4. 각 문서·backend·페이지의 후보 공식 표시값은 정본 이상이어야 한다. 0.01%p 하락도 허용하지 않는다.
   일반 90%·98% 기준과 devel 대비 하락 판정에는 기존 정밀값을 사용한다. 다른 쪽의 개선·평균·CI 성공으로 면제하지 않는다.
5. 이 표는 글꼴 프로그램을 제공하지 않는다. 출처 commit의 condition_evidence와 그 SHA를 통해 수용 당시 조건·생산 기록을 확인한다.
   현재 설치 글꼴이나 임의 대체 글꼴이 원래 조건과 같다고 가정하지 않는다. 통합 담당자가 원본 공급 조건을 확인하고 맞춘다.
6. 검증을 완료해 수용한 최종 전쪽 결과에 상승한 공식 표시값이 있으면 **보호 하한도 반드시 상향한다.**
   새 결과와 입력/PDF·생산·글꼴 증거를 확보하고, 갱신 전 정본 대비 하락 0쪽·누락 0쪽을 확인한다.
   같은 최종 검증본의 전쪽 결과에서 상승한 행은 상향하고 동등한 행은 유지한다.
   TSV·manifest·갱신 영수증·위 문서별 요약을 함께 검토·커밋한다. 영수증에는 이전/새 TSV 해시·
   출처 및 생산 SHA·상승한 문서/backend/페이지별 이전값·새 값과 상승 행 수를 기록한다.
   이후 검증은 상향된 하한을 사용한다. 중간 진단·일부 페이지 출력이나 실행별 페이지 최댓값을
   이어 붙이지 않으며, 현재 미달 후보로 보호값을 낮추지 않는다.

## 자료 무결성 확인

통합 시 선택한 전 3,542행을 수용 commit의 원본 TSV와 대조한다. 중복 키·쪽 누락·범위 밖 값이 없어야 하며,
입력 18개와 대응 PDF의 SHA·전체 쪽수, 양 backend의 페이지 범위를 확인한다. 선택 전의 중복 표에도 같은 검사를
적용하여 원래 보호값이 낮아지지 않았는지 확인한다. 실행 로그·임시 생성/검사 스크립트는 ignored output에만 보관한다.
이 무결성 확인은 제품의 최신 head 재출력·회귀 검사·시각 판독을 대체하지 않는다.

## PR #9075 최종 source의 수용 출처

최종 제품 `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`의 36개 데이터셋·3,542행도 상향된 하한을 모두 유지했다. 추가 상승·하락·누락은 각각 0행이다. 보호 값과 표 해시 `8f073dddd67cf8f44bea7c0315c3f9950ba73641e9393747cdbc52d24b33f9a1`은 바꾸지 않고, 최신 [완료 증적](../issue_9060/final/protected-validation.json)을 수용 출처로 연결했다. [최종 검증 영수증](updates/pr9075-20261115-final-validation.json)과 앞선 [30행 상향 영수증](updates/pr9075-20261115.json)을 구분한다. 원격 CI·병합 상태는 별도 PR review를 따른다.

## 2026-11-15 PR #9070–#9072 검토의 상승분 반영

최종 제품 `8ad1c5c08aa4cd68accbd2af6b9c60bc458d8992`의 Native/fresh WASM 각 1,771쪽·36개 데이터셋을 수용했다. 같은 전쪽 실행에서 상승한 19행만 상향했고 하락·누락은 0쪽이다. [검증 원본](../jinho-von-choi-20261114/final-source-8ad/protected-validation.json)과 [갱신 영수증](updates/pr9070-9072-20261115.json)에 생산자·이전/새 해시·개별 상승값을 기록했다. 중간 실행의 페이지별 최댓값을 합치지 않았으며 이전 출처도 보존했다. CI·원격 병합 완료를 뜻하지 않는다.

## #7732 후속의 이전 상승 기록 보존

리베이스 전 제품 `2c90fa288bb4d48764eb82dbb025df4a12637ea9`의 상승 6행은
[이전 검증 영수증](updates/issue7732-20261115.json)에 보존한다. 최신 devel의
보호 표는 이전 표의 모든 3,542행 이상이며, 이 브랜치의 이전 6개 상승값도 이미 포함한다.
따라서 현행 표·manifest·검사 해시는 최신 devel의 값을 그대로 유지하고, 이전 값으로 내리지 않는다.
[충돌 해소 보고서](../issue7732_split_span_20261115/rebase-4e0b20d2/conflict-resolution.md)를 따른다.

## 2026-11-16 분할 셀 후속 보정의 상승분 반영

같은 조건의 최종 36개 데이터셋·3,542행에서 하락·누락 0쪽을 확인했습니다. 상승이 있는
중첩 표 Native/fresh WASM 각 전17쪽 TSV만 수용하여 14행을 상향했고, 다른 34개 데이터셋의
출처는 유지했습니다. [갱신 영수증](updates/issue7732-20261116.json)에 이전/새 해시와 생산 소스,
전쪽 증거 및 상승값을 기록했습니다. 페이지별 최댓값을 조합하지 않았습니다.

## 2026-11-17 #9104 후속의 말미 공백 보완

최종 제품 `1da7afbd64bbc59d9338a187ebcd21860ddc0842`의 Native/fresh WASM 각 1,771쪽,
총 3,542행과 정확한 devel 전쪽을 대조했다. 하락·누락은 0행이며 중첩 표 10쪽은 양쪽 모두
100.00%다. 같은 전 17쪽 생산본의 상승 2행만 하한에 반영하고 다른 34개 데이터셋 출처와
모든 편람 행은 유지했다. [전쪽 검증](../../../working/assets/issue9104/final/visual-validation.json)과
[갱신 영수증](updates/issue9104-20261117.json)에 source·고정 증거 commit·이전/새 해시를 남긴다.
페이지별 최댓값을 조합하거나 별도 수용 예외를 적용하지 않았다. 편람/기여자 gate는 고정
표 해시 연결만 갱신하며 비교 규칙·대상·한도는 같다. 로컬 검증의 수용 기록이고 원격 CI·병합
완료를 뜻하지 않는다.

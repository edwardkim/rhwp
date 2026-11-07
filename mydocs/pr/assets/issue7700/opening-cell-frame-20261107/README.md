# 저장된 첫 일반 셀 조각의 물리 상자 보존

부모 `0a7adeef5c617dc6677117e19dba7ac16ca87fd3`, base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`에서 완료한 병합 라벨 아래 일반 셀의 첫 저장 조각을 보정했다. 문서·쪽 번호와 고정 픽셀 예외로 분기하지 않는다. 51쪽은 양쪽 모두 99.51%대로 개선됐지만 다른 네 쪽이 98% 미만이므로 **이슈·PR 전체 완료가 아니다**.

## 독립 원본과 공통 원인

- 입력: `samples/issue5679/10857_delegation_rules.hwp`, SHA-256 `de43607f05072840097b19a294b4455905210402cc42823545125e03567df881`.
- 기준: `pdf/semanticist21-20261008/issue5679-10857_delegation_rules-de43607f0507-2020.pdf`, 독립 한컴2020 Print 전103쪽, SHA-256 `90eda86b2b4dfeabc601cd9b583408c6528a9efabf4e626fd6f2e28e3bcb2b97`.
- 현재 정본 `visual_sweep.py`, PDFium96DPI·compat2022·색상 경계 보정 전2px raw·capture 총8. 실제 로컬 글꼴 공급 조건이며 의미 검사의 macOS 최소 글꼴 sandbox와 구분한다.

원본의 완료한 병합 라벨은 8566HU로 두 행의 전체 선언3832+4734HU를 인증한다. 일반 미완료 칸에는 첫 조각3280HU와 다음 저장 줄의 vpos0 재시작이 별도로 있다. 이전 내용 컷은 2134HU만 소비해 빈 띠를 버렸다. 첫 선언의 전체 블록94.82667px가 남은 예산94.58667px보다0.24px 커서 상자를 전부 기각하는 문제도 확인했다.

공통 조회는 원본 줄·완료한 병합 라벨·다음 저장 프레임·완전 행합을 함께 확인한다. 실제 방출은 같은 쪽 예산에 맞춰 마지막 행의 빈 띠만 닫고, 앞 행·내용 컷·다음 조각을 유지한다. 재조판·편집·중첩 개체·미확인 저장 구조에는 이 선언을 승격하지 않는다. 동일한 수용 행 상자를 예약과 paint에 전달한다. 임의의 허용 픽셀을 추가하지 않았다.

초기 진단에서3280HU를 쪽 예산과 무관한 절대 출력 높이로 가정한 것은 바로잡았다. 독립 PDF의 실제 첫 조각 높이는 약43.47px이고 다음 조각은 약45.71px다. 정식 검사는 원본 선언 상한·내용 소속과 독립 PDF 하단±2px를 확인한다. 원본과 PDF는 바꾸지 않았다.

## 실제 검증

| 범위 | 이전 Native → 보정 | 이전 fresh WASM → 보정 |
| --- | ---: | ---: |
| 51쪽 | 94.75432 → 99.51311% | 94.75587 → 99.51281% |
| 전103쪽 최저 | 94.75432 → 96.64273% | 94.75587 → 96.64099% |

각103쪽·누락0·90% 미달0이며 변경 PNG는51쪽 하나다. 나머지102쪽은 각 backend에서 바이트가 같다. 9·26·44·86쪽은 여전히98% 미만이므로 보정 대상이다.

[이전 Native TSV](before-native-silhouette.tsv) · [이전 WASM TSV](before-wasm-silhouette.tsv) · [보정 Native TSV](after-native-silhouette.tsv) · [보정 WASM TSV](after-wasm-silhouette.tsv)

이전 전쪽 캡처는 실제 생산자b82d0843c로 표시한다. 직접 부모0a7adeef5까지의 유일한 runtime 차이는 근호 배치이고, 이 원본에는 수식 컨트롤이0개다. 표·본문 구현이 같다는 근거로 재사용했으며 직접 부모의 새 전쪽 캡처로 바꿔 부르지 않는다. 수정 전 정식 검사는 직접 부모의 frozen 라이브러리에서 별도로 실행했다.

[정식 검사](../../../../../tests/cases/issue_7700_source_owned_table_fragments.rs)는 기존6개를 유지하고51쪽 독립 하단 고정형·원본 상한·첫 줄/뒤 두 줄 소속을 추가했다. Native/fresh WASM51쪽이 각각98% 이상인 실제 출력과 직접 판독을 근거로96DPI±2px를 사용한다. 실제6 PASS/1 FAIL→7 PASS다. 기존 수식·글상자·저장 프레임·중첩 표를 포함한 집중153 PASS에는 이7개와 출력용 진단1개가 포함된다. 초기 단독 묶음 컴파일의 Cargo bin 환경변수 누락은 실제 frozen CLI를 지정해 복구하고 해당 묶음만 다시 실행했다.

fmt·Native/WASM/전체 target/Skia Clippy·workspace build·manifest·unit tier가 통과했다. fresh WASM은 Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 결과가 아니다. root pkg/Studio public 파일 해시는 일치하지만 실제 브라우저 초기화 응답 검증은 최종 단계에 남아 있다. source/artifact·명령·요약·로그 해시는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/opening-cell-frame-20261107/validation.json`)에 있으며 로그·중간 스크립트·raw tree는 ignored output에만 보존한다.

## 대표 출력과 직접 판독

- 이전 Native51쪽: [review](before-native-p051-review.png) · [overlay](before-native-p051-overlay.png)
- 보정 Native51쪽: [review](after-native-p051-review.png) · [overlay](after-native-p051-overlay.png)
- 이전 fresh WASM51쪽: [review](before-wasm-p051-review.png) · [overlay](before-wasm-p051-overlay.png)
- 보정 fresh WASM51쪽: [review](after-wasm-p051-review.png) · [overlay](after-wasm-p051-overlay.png)

하단 첫 조각의 괘선과 가운데 정렬 본문을 대조했다. 다음 조각의 두 줄·다른 행/쪽의 소속을 보존하며 일부 기존 미세한 행 높이·글자 간격 차이는 남는다. 전103쪽 개선과 별개로 다른 문서의98% 보정·최종 전체 nextest·편람·실제 소비 검증·PR 제출은 아직 끝나지 않았다.

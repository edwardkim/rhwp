# 저장 이어받기 문단의 앞 간격 보존

부모 `49778f49d8`, 고정 base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`에서 개인정보 문서 32쪽의 원본 앞 간격 누락을 공통 배치 경로에서 보정했다. Native/fresh WASM 전 66쪽이 모두 98% 이상이다. 다른 문서나 최종 PR 준비 완료를 뜻하지 않는다.

## 원본·독립 근거·선행 해결

원본은 `samples/task2097/75544_pii_bunseok.hwpx`, 독립 기준은 [한컴 2020 Print PDF](../../../../../pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf) 66쪽이다. 원본·PDF·문턱·내용은 바꾸지 않았다.

원 해결 #5846과 통합 #5912는 혼재 문단의 중첩 표 꼬리 행 소유를 보존했다. 후속 원점 보정 `3a9fc5a1d63`과 #8191의 `6097253468`을 확인했다. 이들 검사·정상 첫 조각은 유지한다. 이번 문제는 중첩 행 소유가 아니라 새 저장 프레임에서 시작하는 첫 문단의 앞 간격이다.

원본 32쪽 조항의 저장 첫 줄은 `vertical_pos=300HU`이며 HWPX paraPr174의 `prev=300HWPUNIT`와 일치한다. 현재 프레임의 원점은 이미 0으로 인증됐지만, 스냅이 차감한 4px를 compose에 넘기는 조건이 셀 전체의 문단0만 인정했다. 이어받기 문단은 column-top trim에서도 그 간격을 잃어 0px가 됐다.

원본 셀의 문단0과 인증된 재시작 프레임의 첫 가시 문단을 구분한다. 이미 선차감한 간격만 같은 공통 compose 호출에 돌려주며 뒤 문단의 저장 원점·내용 소유·수용 높이는 바꾸지 않는다. 문서/쪽 분기나 임의 4px 가산은 없다.

## 실제 결과

정본 PDFium 96DPI·compat2022·2px 색상 경계 보정 전 raw·capture 총8, 동일한 기존 참조 글꼴 환경에서 부모와 후보를 각각 전쪽 출력했다. 글꼴은 새로 설치하거나 커밋하지 않았다.

| 범위 | 이전 Native → 보정 | 이전 fresh WASM → 보정 |
| --- | ---: | ---: |
| 32쪽 | 96.87702 → 100.00000% | 96.87040 → 100.00000% |
| 전 66쪽 최저 | 96.87702 → 99.21550% | 96.87040 → 99.21467% |

각 66쪽에서 98% 미달·누락·부모 대비0.1%p 초과 하락0이다. 바뀐 PNG는32쪽 하나이며 각 나머지65쪽은 바이트 동일하다. raw100%는2px 이웃 판정이며 RGB 전체 바이트 동일을 뜻하지 않는다.

[이전 Native TSV](before-native-silhouette.tsv) · [이전 WASM TSV](before-wasm-silhouette.tsv) · [보정 Native TSV](after-native-silhouette.tsv) · [보정 WASM TSV](after-wasm-silhouette.tsv)

## 회귀 검사와 제한

[정식 검사](../../../../../tests/cases/issue_5846_continuation_stored_before.rs)는 관계형1개와 고정형1개다. 관계형은 원본 앞 간격·뒤 문단의 같은 저장 원점·내용/순서/단일 소유를 확인한다. 고정형은 양 backend32쪽100%를 확인한 뒤 등록했고 독립 PDF의 세 기준선 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue5846/continuation-before-space-20261107/independent-pdf-baselines.json`) 97.119995/123.519999/149.919993CSSpx에 ±2px를 적용한다. 후보 좌표에서 기대값을 복사하지 않았다.

실제 수정 전2 FAIL→수정 후2 PASS, 기존 #5846·#7700·#8060의15개도 PASS다. 중간 관계형1개 실행을 중복 집계하지 않아 집중 합계17 PASS다. 정식 자동 suite와 macOS 최소 글꼴 sandbox·threads8·Cargo 순차로 실행했다. Native/WASM/all-target Clippy·fmt·manifest·unit tier도 통과했다.

픽셀 검사 등록 뒤 바뀐 것은 테스트 파일뿐이며 runtime·빌드·스크립트 source hash는 빌드 때와 같다. 현재 source의 전체 nextest·전 코퍼스·Skia 전체unit·최종 편람 gate·실제 소비를 모두 완료했다는 의미는 아니다. Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 결과가 아니다. root pkg/Studio public hash는 일치하고 실제 Studio 초기화 응답 확인은 남아 있다. 명령·해시·결과는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue5846/continuation-before-space-20261107/validation.json`), 로그·임시 도구는 ignored output에 보존한다.

## 대표 32쪽

- 이전 Native: [review](before-native-p032-review.png) · [overlay](before-native-p032-overlay.png) · [raw](before-native-p032-raw.png)
- 보정 Native: [review](after-native-p032-review.png) · [overlay](after-native-p032-overlay.png) · [raw](after-native-p032-raw.png)
- 이전 fresh WASM: [review](before-wasm-p032-review.png) · [overlay](before-wasm-p032-overlay.png) · [raw](before-wasm-p032-raw.png)
- 보정 fresh WASM: [review](after-wasm-p032-review.png) · [overlay](after-wasm-p032-overlay.png) · [raw](after-wasm-p032-raw.png)
- 독립 한컴 Print: [raw](hancom-print-p032-raw.png)

보정 Native/WASM review와 standalone overlay를 직접 판독했다. 첫 문단 세 줄의 수직 위치가 복구됐고 뒤 조항·아래 표·쪽번호는 기존 위치를 유지한다.

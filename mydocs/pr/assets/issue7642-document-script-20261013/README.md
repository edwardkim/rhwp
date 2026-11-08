# 문서 수식의 명령 경계 Native 진단

생산 SHA: `cbd9d4676b648bee9696e19f4456f3cb8d999762`. 수정 전 SHA: `edbe159fa54f73472853522c1c969c84493b5e1e`.
동일 원본 `pdf/issue7642-equation-catalog-print/equation-catalog-hancom-2020.hwp`와 독립 한컴 Print PDF 전체 18쪽을 각각 실제 바이너리로 재출력했다.
입력 SHA-256: `5821913b3311590a0a724f93a8f0c5d84966c06bbf42cdd3d8e30fe5664ad3b5`. PDF SHA-256: `376ccde19b057c1dddd8b1548d7e5ac048e703c28d94c250467e8844d32680dc`.

3쪽은 74.00045% → 93.06517%, 17쪽은 33.20773% → 40.76399%다. 전체 최저는 33.20773% → 39.95090%, 90% 미만은 10쪽 → 9쪽이다. **시각 완료·PR 제출 가능으로 판정하지 않는다.** 3쪽의 공백 없는 덧셈 식에서 분자·분모 소속이 잘못된 것이 직접 보인다. 17쪽은 잘못 소비하던 이름을 보존했지만 비정상 구조 명령의 중단 경계·세로 배치가 남아 있다.

Native 전체 TSV·빌드/출력 receipt·원문 PNG는 ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage177-{before-catalog-whole-native,catalog-whole-native,catalog-native-review}/`에 있다. 대표 PNG는 해당 producer의 새 실제 출력이다. fresh WASM은 진행 중이며 이 그림이 그 결과를 대신하지 않는다.

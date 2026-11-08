# PR #8397 검토

- PR: https://github.com/edwardkim/rhwp/pull/8397
- 검토한 code head: `8b0bcde745f7b78a5b1fbe2122238a57696b1feb`
- 검토 기준: 한컴 Print 출력, 최소 글꼴 회귀, 최신 devel 병합 결과

## 검토 내용

HWP3 HFT에 대응하는 설치 글꼴의 선택과 원본 폭 fallback, Studio의 로컬 글꼴 감지·선택과
문서 재조판을 확인했다. 글꼴 공급이 없는 환경에서 실행되는 회귀와 명시적인 테스트 글꼴을
사용하는 검사를 구분했다. 상용 글꼴 파일은 이 PR에 포함하지 않는다.

sample10의 원본 및 한컴 2020 Print PDF 각 763쪽을 대조했다. Native 최저 99.41135%, fresh
WASM 최저 99.39720%이며 90% 미만은 없다. PDF의 그림과 본문이 겹치는 757쪽의 기존 차이는
[상세 기록](../assets/issue_8371/sample10-validation.md)에 구분했다.

#8389의 기존 8개 문서 972쪽은 Native와 fresh WASM 모두 원 검증 대비 하락이 없다. #7905
법령 문서 6쪽은 양쪽 모두 98% 이상이다. 편람과 sample16은 이번 보정 대상이 아닌 기능 보존
대조군이다. 원본·Print PDF·측정 조건과 대표 review·overlay는 [검증 자료](../assets/issue_8371/rebase-55f40/README.md)에 기록했다.

최신 devel과 code head를 병합한 코드 결과를 후속 리베이스 후보와 비교했으며, 제품 코드의
차이는 없었다. #8395의 탭 원본·편집본은 Native와 WASM에서 새 기준 대비 전쪽 raw가 동일했고,
관련 최소 글꼴 회귀도 통과했다.

## 검증과 판정

- 해당 code head의 [GitHub CI](https://github.com/edwardkim/rhwp/actions/runs/41149623203), CodeQL,
  Render Diff, Adapter 및 Proptest가 성공했다.
- 최소 글꼴 환경 전체 Rust nextest는 12,227건 통과, 45건 건너뜀이다. Studio 단위 검사
  2,702건과 브라우저 글꼴 선택 검사를 통과했다. Native Skia 집중 검사는 427건 통과했다.
- **최종 판정: 조건부 통합 가능.** 문서 trailing head의 CI와 병합 직전 head·병합 가능 상태를
  확인한 뒤 작업지시자가 요청한 admin merge를 수행한다.

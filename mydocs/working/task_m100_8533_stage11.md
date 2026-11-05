# #8533 시험 글꼴 고지 목록 보완

첫 제출 head `dc8e9b533bf5bb1dad0492016ae86d9c543a90b5`의 Frontend package gates는
`Verify font asset contracts`에서 7 PASS/1 FAIL이었다. 새 합성 시험 글꼴
`tests/fixtures/issue8533/RHWPPrintFallbackContext.ttf`를 저장소 전체 공개 글꼴 고지 원장의
`repositoryOnlyFonts`에 등록하지 않은 누락이다. 제품 글꼴 목록·재배포 허용치를 바꾸거나
검사를 제외해서 해결하지 않는다.

기존 `RHWPHostFixture-Regular.ttf`의 MIT 소유 합성 윤곽을 재사용한 파일이므로 실제
SHA-256, 원본 저작권, 저장소 LICENSE와 소유 fixture의 라이선스 설명 경로, 생성 출처를
같은 원장에 추가했다. 제품 `fonts` 목록에는 넣지 않으며 설치·웹 배포용 face로 바꾸지 않는다.

- 같은 계약의 로컬 음성 대조는 실제 1 FAIL, 원장 보완 후 1 PASS다.
- Studio·Chrome·Firefox build와 VS Code compile이 각각 exit 0이다.
- 새 고지를 포함한 배포 폴더에서 font asset 계약 전체 8 PASS/0 FAIL이다.
- font/WASM/JS 프로그램 바이트, Rust·Studio 제품 소스, 입력·PDF·baseline·시각 문턱은
  바꾸지 않았다. 기존 전383쪽 출력과 제품 회귀는 동일 소스 증거로 유지한다.
- workflow와 CI 실행·재사용 판정 정책은 바꾸지 않는다. 새 code head의 GitHub 검증은
  별도로 완료한 뒤에만 review·오늘할일 trailing commit을 만든다.

실제 실패 job: https://github.com/edwardkim/rhwp/actions/runs/41799023145/job/126471655414.
상세 로그와 패키징 결과는 ignored output의 `ci-frontend-failure.log`,
`font-notice-before.log`, `font-notice-after.log`, `font-notice-all-contracts.log`,
`font-notice-packaging-results.json`으로 구분한다.

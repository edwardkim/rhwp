# SO-SUEOP 첫쪽 일반 글꼴 대조군 입력 보존

이 파일은 기존 v416에서 만든 일반 글꼴 대조군의 실행 입력을 **동일 바이트로 보존**한다.
기존 실행 위치: `output/pr-review/semanticist21-20261008/issue7807-font-control-v416-retry1/SO-SUEOP-p1-normal-font-control.hwp`.
입력 SHA-256: `a230e82db180b6d01f9eb18e127df8a65cfcb740a2f4780c7a1abd4fcfa2e57e`.
독립 한컴 Print PDF: `pdf/semanticist21-20261008/issue7807-font-control-v416-retry1/SO-SUEOP-normal-control-a230e82db180-2020.pdf`.
PDF SHA-256: `830b1920a4dd0b9dd8c6f03ffc9b848eb0737ac768ccc8a1031b1fd57b3b5dbf`.

최신 생산 코드 `7f04c48cd395086ed59c08565f343d3f9e1b013d`, Native CLI SHA
`2552cecd315dea914451af6c52c77757c9dc6418cf066135bdece4ae08c10e7a`, fresh WASM SHA
`460a928d8423b17b150eee98f4408af330682512f0d0806a78d4cc709316586a`에서
**첫쪽 Native/fresh 각99.84371%**다. 검증 범위는 사용자가 지정한 첫쪽 대조다.
문서/Print의 전체 쪽수는46/46이지만 나머지45쪽의 시각 통과를 주장하지 않는다.
현재 두 첫쪽 캡처는 decoded RGBA가 같고, 기존에 직접 판독한 첫쪽과도 비교한다.
전체 #7807 해결 또는 신규 전쪽 회귀 대상으로 등록하는 기록이 아니다.

실제 argv·입력/Print/소스/패키지 해시·TSV·PNG는 ignored v451의 ole-so 계획/결과에 보존한다.
이 commit은 입력 보존만 추가하며 원본 SO-SUEOP나 이미 검증한 실행 입력을 바꾸지 않는다.

# #5874 재현 입력

- 출처: [이슈 #5874](https://github.com/edwardkim/rhwp/issues/5874), 작성자 `kjh0523`의 최소 재현 문서.
- 원본: [italic-repro.hwpx](https://github.com/kjh0523/rhwp/blob/4435a2f20e8c2166b57c570fb641ff959041ee70/italic-repro.hwpx).
- 입력 SHA-256: `cc03b646e5ba294575d8d254de00368b7a085bb51f1abbbdedcd3b2608f81dc5`.
- 대상: 1페이지의 부분 기울임과 문단 전체 기울임. `<hh:italic/>` 1개가 여러 run에 적용된다.
- 작성자 한컴 화면은 `mydocs/pr/assets/issue_5874/reporter-hancom.png`에 보존한다.
  동일 commit의 `italic-hancom.png`이며, SHA-256은
  `0c70a494a0658ee9efff4403c86ba6aae2eef58d1b0a72167b80a83c749e6043`이다.
- 작성자 화면은 기울임 유무를 판단하는 근거다. 폰트/플랫폼이 다른 PDF와 픽셀 일치율을
  계산하는 독립 기준 PDF로 간주하지 않는다.

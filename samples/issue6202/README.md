# samples/issue6202 - 용지 기준 Square 그림 재투영 회귀 입력

이 폴더는 [#6202](https://github.com/edwardkim/rhwp/issues/6202)의 정식 회귀
fixture다. 그림을 옮긴 뒤에도 본문이 예전 배제 밴드에 남지 않고 새 그림 위치를 피해
다시 조판되는지 `issue_6202_paper_relative_float_exclusion` 테스트가 확인한다.

- 원본 문서 식별자: `156483689`, 국내산 강황 제조기술 표준화 보고서
- 회귀 계약: 용지 기준 `Square` 그림의 수평 위치를 바꾸면 본문 배제 밴드도 새 위치로
  재투영되어야 한다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 한컴 기준 PDF와 통합 head 시각 증적은 별도 검토 단계에서 `pdf/` 및
  `mydocs/pr/assets/`에 기록한다.

# samples/issue5057 - HWP5/HWPX 첫 조각 초과 허용치 회귀 입력

이 폴더는 [#5057](https://github.com/edwardkim/rhwp/issues/5057)의 정식 회귀
fixture다. `21484591` 원본 HWP를 보존하며, `issue_5057_profile_agnostic_source_frame_allowance`
테스트는 이 파일을 HWP5로 읽고 `export_hwpx` 산출물에서 출처 표식만 제거한 direct-HWPX
경로를 다시 읽는다.

- 원본 문서 식별자: `21484591`, 김천시 하수도 사용 조례 시행규칙 별지 서식
- 회귀 계약: HWP5와 표식 없는 direct-HWPX가 동일한 페이지 수를 가져야 한다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 한컴 기준 PDF와 통합 head 시각 증적은 별도 검토 단계에서 `pdf/` 및
  `mydocs/pr/assets/`에 기록한다.

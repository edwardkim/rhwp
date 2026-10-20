# HWP3 묶음 상대 위치 보정의 Native 증거

- 코드 source: `d4b756bb4d775c2c6bb0bac9429ccfb3482492e8`.
- 원본: `samples/issue1892_hwp3_drawing_group_roundtrip.hwp`, SHA-256 `ab6bb4e80c4b0f4228e62bbb3d180229e5cef5f52b20fddee8849bf1c7b4c438`.
- 독립 Print PDF: `pdf/issue7642-roundtrip-contract-print-20261014/issue1892_hwp3_drawing_group_roundtrip-2020.pdf`, SHA-256 `bde778b26c91635f14bf14272eca8b9199da7be7f9d4e0750b7402768b640bc9`.
- 동일 원본·독립 PDF의 전체 2쪽 Native 최저 일치율: 95.50501%. 실제와 기준 모두 2쪽이다.
- 대표 그림은 1쪽의 비교와 overlay다. 변경 전 source6f77의 83.82851%와 별도로 기록한다.
- 실행 명령·전체 TSV·로그·입력 해시 receipt는 ignored output의 stage703/704에 보존한다. 이 디렉터리에는 TSV를 커밋하지 않는다.
- 직접 판독에서 수평 치수선의 형제 위치와 중첩된 세로 치수선의 부모 위치 상속을 확인했다. 원과 일부 글자 폭 차이는 남아 있어 시각 완료 판정이 아니다. fresh WASM과 정식 회귀 검증도 진행 중이다.

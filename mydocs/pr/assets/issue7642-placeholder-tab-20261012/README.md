# #7642 생략된 탭 폭의 Native 1차 보정

이 asset과 함께 커밋한 생산 코드의 Native 출력이다. 빌드 전후 생산 파일 해시가 일치한다.
바이너리 SHA-256: `be990687ed7ca512c393d8d1e53f0ecad957193e6bf00e4d1d6f732ce4127192`.

- 원본: `samples/issue1892_hwp3_tab_roundtrip.hwp`
- 독립 한컴 2020 Print: `pdf/issue7642-hwp3-realign-controls-20261012/issue1892_hwp3_tab_roundtrip-hancom-2020.pdf`
- 전체 1/1쪽, 인쇄 프로필, 2022 조판 세대, 전체 글꼴 포함, 96 DPI.
- Native TSV: 93.91816%. 이진화 원값도 93.91816%, 색상 경계 보정은 0픽셀.
- 이전 고정 source `68d8bf777f`의 같은 원본 점수는 82.43006%다.

경찰서장·교도소장·구치소장의 가로 시작은 개선됐다. `귀하`·하단 `(인)`의 위치와
원형 글리프 차이는 미해결이며 점수 통과만으로 시각 완료를 판정하지 않는다.
fresh WASM과 최종 전체 검증도 미완료다. 새 회귀·골든은 아직 추가하지 않았다.

[review](tab-p001-native-review.png) / [overlay](tab-p001-native-overlay.png)

review는 RHWP / Print / overlay 순이다. 파랑=RHWP만, 빨강=PDF만,
주황=양쪽 차이, 회색=일치다. 입력/PDF 해시·명령·실제 exit·대조군 5쪽 TSV는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage144-tab-preflight/receipt.json`에 보존한다.
상세 분석은 [Stage 144](../../../working/task_m100_7642_stage144.md)에 연결한다.

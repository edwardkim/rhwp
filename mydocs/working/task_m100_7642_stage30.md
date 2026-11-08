# 이슈 #7642 — 공개 근호 윤곽을 세 출력 경로에서 함께 소비

시작 커밋: `005231fb3`. 현재 회귀 실패 전체를 이 브랜치에서 보정한다.

## 독립 관측과 원인

근호 예약 폭과 몸체의 이탤릭 전진을 독립 정상 한컴 Print와 맞춘 뒤에도
154건 대조 5쪽은 82.69128%다. 확대 판독에서 올라가는 선이 너무 수직이고
두꺼운 시작 획·세로 높이도 다르다. 현재 SVG·Canvas·Skia는 공개 수학 글꼴의
실제 근호 대신 서로 복제한 5점 스케치를 그린다. 올라가는 선의 가로 폭은
0.15em으로 고정되어 높이를 늘릴수록 모양이 왜곡된다.

저장 폭이나 검사 허용치를 바꾸어 이 차이를 숨기지 않는다. 배포된 공개
Latin Modern Math의 U+221A 윤곽(UPM 1000, 전진 833)을 사용해 그 자체의
선과 곡선을 보존하고, 근호의 몸체 원점·점유 높이에 연결한다. GUST Font
License와 파일 SHA를 코드에 기록한다. 사설 폰트 윤곽을 복제하지 않는다.

## 소비 경로와 검증 범위

`layout_sqrt의 body.x/height/width → 공통 윤곽·윗줄 기하 → SVG/Canvas/Skia`
를 연결한다. 글리프의 원점은 공개 전진폭으로 몸체 왼쪽에 연결하며 근
지수는 기존 소속을 보존한다. 윗줄은 같은 점유 결과의 끝을 소비한다.
다른 face·중첩 근호·분수·근 지수도 같은 공통 결과로 그린다.

진단 28건·154건과 원본 문제 페이지의 Native 직접 비교·확대를 먼저 수행한다.
세로 프레임 차이가 남으면 별도로 독립 메트릭과 대조하고 보정한다. 이
소단계는 전체 회귀·fresh WASM·최종 PR 게이트를 대체하지 않는다. 최저
90% 미만이나 미세 차이가 남으면 새 회귀·기대값 갱신·PR 제출을 보류한다.


## 소단계 결과와 판정

Native build는 exit 0, 바이너리 SHA-256은
`46e7d63646ec2a3f7322211439ee5cb06783e0d579b91c3b1caaeadd7b6699e5`다.
Native Clippy는 exit 0(52.09초), 포맷·diff 검사도 통과했다.
소스 해시·명령·로그는 ignored `output/pr-review/issue7642/failed-inputs-20261008/
stage15-current-validation/`의 `public-radical-outline-native-provenance.json`과
`public-radical-outline-native-clippy.json`으로 고정한다.

| Native 대조 | 수정 전 | 수정 후 | 판정 |
| --- | --- | --- | --- |
| 근호·함수 28건, 1쪽 | 94.57617% | 95.94651% | 자동값 통과, 함수 잉크·근호 높이 확대 차이 잔존 |
| 근호 154건, 4쪽 | 89.64494% | 92.19460% | 자동값 통과, 직접 판독 별도 |
| 근호 154건, 5쪽 | 82.69128% | 85.94595% | 미충족, 중첩 높이·복합식 간격 보정 계속 |

추가 대조 전5쪽의 수정 후 값은 98.56587 / 97.34244 / 95.77897 /
92.19460 / 85.94595%다. `public-radical-outline-diagnostic-native/`에
summary와 representative review PNG를 보존하고 1·5쪽 그림을 사용자에게
직접 표시했다. SVG·Canvas·Skia의 공통 구현과 Native 실행을 구분한다.
Canvas/fresh WASM·Skia 실행·전체 회귀는 아직 이 코드로 검증하지 않았다.
기존 테스트·기대값·golden을 바꾸지 않았으며 PR 제출 완료가 아니다.

## 별도 확인: 기준 출력과 조판 세대

2024 Print를 가진 2022년 11월 원본 HWPX의 17쪽을 같은 Native 바이너리
`rhwp-root-italic-advance`에서 `export-svg --compat 2024 --profile print`로
다시 출력했다. 30.90632%로, 이전 기본 2022 비교의 약 30.8%와 같이 미달이다.
`nov2022-compat2024-direct/command.json`·overlay·review를 보존하고 직접
표시했다. 기준 PDF의 앞 문항 꼬리가 현재 쪽에서 누락되고 다음 문항이 일찍
시작한다. 조판 세대 불일치만으로 해결됐다고 보고하지 않는다.

검증 도구가 조판 세대 인자를 받지 않는 점은 별도로 고친다. 원본 저장 연도나
확장자를 추정해 고르지 않고 실제 Print engine과 일치시킨다. 2024 원본의
독립 Save job `dedf4679-c2df-4b76-bdd5-a679e364ca9c`는 성공했고
한컴 `13.0.0.3901`의 정상 HWP를 ignored 진단 경로로 받았다. 이 저장 작업은
Print 작업이 아니다. 원본과의 IR 요약은 diff 0이며 기존 PDF 기준은 유지한다.

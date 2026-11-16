# PR #9137 리뷰 — 저장 줄이 있는 글상자 표의 세로 정렬

## 최종 판정

**머지 보류 — 데스크톱 Print와 저장 앵커·표 바깥 여백 경계를 검증 중이다.**

## 접수 정보와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/9137), 작성자 `hyunmyungJaneLee`.
- 원 head `125f8c80c2eec043f604b1fc6d9815e3e3e7dcaf`, base `53774cc1508591b6ac794950f0063452fbaf6e87`.
- reviewer `jangster77`, milestone `v1.0.0`을 API로 확인했다.
- #9108의 저장 줄이 있는 경로를 다루는 #9109 후속 PR이며 공통 커밋 6개는 중복 적용하지 않았다.

## 검증과 조판 원칙

`samples/textbox_stored_table/`의 TopBottom·앞으로 배치 표, 오프셋 0/2000, 위/가운데/아래 정렬,
띠 아래로 밀린 저장 앵커를 독립 Print와 대조한다. stored 단위와 실제 개체 원점을 구분하고
측정 높이·정렬·paint가 같은 표의 소속과 바깥 상자를 소비하는지 확인한다.
저장 줄 없는 정상 대조군과 HWP/HWPX 양쪽 저장 경로도 포함한다.

원 PR의 **웹한글기안기 PDF는 공식 대조 자료가 아니다.** 같은 입력의 한컴 2020 데스크톱
Print를 확보했다. `textbox_nostored_header_front_tall.hwpx`는 이 독립 Print에서도 base raw
33.04%이므로 정상 시각 대조군으로 취급하지 않으며, 후보·주장과 기존 결함의 경계를 별도로 확인한다.
`_pushed`의 원 PR 잔여 간격도 공식 Print로 다시 확인한다. 기존 점수를 복사해 수용하지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR·merge SHA·CI와 실제 Print 제품을 한국어 존댓말로 알린다. 저장 줄이 있는 이 PR의
각 주장에 대응하는 같은 쪽 raw·review·overlay를 본문에 표시한다. #9109 공유 범위와
메인터너 보정 범위가 있으면 구분하며 #9108의 완전 해결 여부를 확인한 뒤 종료한다.

# 제한 개체의 편집·저장 후 이월과 부동 앵커 앞 공백 처리

PR #9063의 원 기여 여섯 commit을 보존해 수용합니다. 문단 기준으로 쪽 영역이 제한된 그림·도형을
다음 쪽에 배치하고, 부동 앵커 앞의 꼬리 공백이 글줄 위치를 바꾸던 문제를 보정합니다.

메인테이너 검토에서 첫 문단에 한 글자를 입력하면 개체가 앞쪽으로 돌아오는 경계를 발견했습니다.
편집 후의 실제 흐름 원점을 공통 이월 판정에 연결하고, 독립적으로 이월하는 그림 높이를
호스트 문단에 중복 예약하지 않도록 보완했습니다. HWP/HWPX 반복 저장·재열기에서는 합성 LineSeg를
저장 쪽 좌표로 잘못 재사용하지 않습니다. 문서별 상수나 시각 기준의 예외는 추가하지 않았습니다.

- 독립 한컴 2020 Desktop Print 기준, 원본·실제 편집 저장본 39조합·58쪽씩 Native/fresh WASM raw 100.00%.
- 실제 Native 편집·반복 저장 60상태·100쪽 raw 100.00%, WASM live 20쪽 raw 100.00% 및 반복 저장 48상태 보존.
- 실제 Studio 키보드 편집·Canvas2D와 SVG 출력 확인, 최신 devel의 #9042·#9066 대조군 유지.
- 정본 보호 3,542쪽의 공식 두 자리 값 하락 0건, 편람 HWPX Native 383쪽 gate 통과.
- 최소 글꼴 전체 회귀 13,524건·Native 공식 4,073건 통과. 모든 Rust lint/build·suite 정책과 새 샘플 보안·쪽수 회귀도 통과했다. 실제 명령·exit는 실행 원장을 따른다.
- 신규 원본 17개는 canonical Print PDF와 쪽수 보호 원장에 등록했다. 입력·저장본 43개는 보안 탐지기 3종 모두 clean이다.

원 PR: https://github.com/edwardkim/rhwp/pull/9063

Closes #8660
Closes #9059

원 기여 head: `b43050954c10512705d714114f5af23f1ee98ac6`. 공통 보정 source: `e2d7fb08b5`, 정식 회귀 보강: `5b028b3f03`.

[검토 기록](https://github.com/edwardkim/rhwp/blob/review/pr-9063-restricted-object-next-page/mydocs/pr/pr_9063_review.md) · [실행 원장](https://github.com/edwardkim/rhwp/blob/review/pr-9063-restricted-object-next-page/mydocs/pr/assets/pr9063/evidence.md) · [Native 비교](https://github.com/edwardkim/rhwp/blob/review/pr-9063-restricted-object-next-page/mydocs/pr/assets/pr9063/correction/native-detail.png) · [fresh WASM 비교](https://github.com/edwardkim/rhwp/blob/review/pr-9063-restricted-object-next-page/mydocs/pr/assets/pr9063/correction/wasm-detail.png).

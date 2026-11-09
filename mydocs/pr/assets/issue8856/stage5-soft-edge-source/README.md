# 그림 soft-edge의 원본 반경·저장 소유

이번 단계는 효과 도색 전의 모델·저장 손실을 복구한다. 실제 효과 PNG의 완료나
최종 PR 검증 완료를 뜻하지 않는다. [작업 기록](../../../../working/task_m100_8856_stage5.md)에
원 해결 #8656, 통합 보정, 원시 바이트·사양·반례를 구분했다.

원본 `interview.hwp`의 첫 그림은 단독 효과 flag 4·radius f32 1000 HU를 가진다.
독립 한컴 2020 변환도 같은 값을 `hp:softEdge`에 기록한다.
[source-raw-picture-effects.json](source-raw-picture-effects.json)과
[independent-source-conversion.json](independent-source-conversion.json)을 대조한다.
다른 합성 그림의 반경 300/500 HU, 크기·crop·투명도는 기존 #8593 계약을 유지한다.

새 관계 검사에서 원본 반경 소실은 실제 1 PASS / 1 FAIL이었다. 보정 후 Native Skia의
새 3건·기존 크기/자르기 2건·연속 그림자 7건·PDF 로그 2건, 합계 14 PASS였다.
최종 주석 변경 후 새 3건도 다시 실행했다. 최소 글꼴 프로필·명령·로그 해시·실제 소스 해시는
[validation.json](validation.json)에 기록했다. 초기 컴파일 오류와 0 tests는 통과로 세지 않는다.

별도 [독립 효과 대조군](independent-controls.json)은 0/300/1000/2000 HU 각각을
2020과 2024 Print method 0으로 출력한 실제 8건·각 1쪽이다. 입력과 PDF는
`pdf/issue8856/flat-soft-edge-*`에 보존했다. 원본에서 첫 그림과 구역·단 설정만 남기고,
단색 RGB(64,128,192) JPEG, 30000×20000 HU 크기와 전체 crop·identity 행렬,
5000×5000 HU 위치를 지정했다. 다른 구역은 manifest/spine과 secCnt에서 제거했다.
실제 그림의 단위·효과를 원본 확대·crop과 분리해 확인할 수 있다.

현재 parser/serializer는 알려진 단독 효과를 보존하며 다른 가변 복합 효과의 내부 위치를
추정하지 않는다. 원본·PDF·문턱·기존 검사를 바꾸지 않았다. 실제 가장자리 도색은 다음 단계에서
이 대조군과 원본을 함께 검증하고, 최종 head의 Native/fresh WASM 전쪽 TSV를 다시 산출한다.


## 자르기 기준 후속 보강

첫 단계 커밋 `787ad43f5e` 뒤에, 반경·alpha 편집 후 실제 그림의 crop 기준도 비교했다.
원본 크기가 없는 첫 그림이 crop 끝값을 전체 크기로 오인하여 달라지는 실패를 확인했다
(2 PASS / 1 FAIL). 알려진 단독 효과의 새 alpha 꼬리에 원래의 미확인 imgDim을 0으로
유지하여 그림의 암묵 기준을 보존했다. 새 검사·기존 대조군 14건을 현재 소스로 모두
다시 실행해 통과했다. [후속 기록](crop-frame-followup.json)은 앞 단계 기록과 구분한다.
이 보강도 실제 soft-edge 도색·전쪽 수용 완료를 주장하지 않는다.

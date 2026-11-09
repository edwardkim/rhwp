# Flutter GPU 뷰와 데스크톱 창의 수명

`NativeGpuLifecycle(controller: controller, child: viewport)`는 기존 GPU 뷰를
mount한 채 앱의 hidden·paused·detached 상태에서 admission과 polling을 멈춘다.
resumed와 inactive에서는 현재 source·geometry를 새 presentation generation으로
게시한다. focus를 잃었지만 여전히 보이는 inactive 창을 최소화로 취급하지 않는다.
일시 중지는 기존 `NativeTextureController.pause/resume`와 Native cover 확인을 사용한다.
문서·입력 owner를 닫거나 새 GPU owner를 자동 생성하지 않는다.

physical size 또는 DPR이 바뀌면 이전 frame을 즉시 가리고 다음 Flutter layout이
현재 geometry를 적용한 뒤 재개한다. 여러 metric 이벤트는 마지막 frame으로 합친다.
관측한 `FlutterView`의 size/DPR이 같은 전역 알림은 다른 창의 이벤트이므로 무시한다.
controller를 교체하면 현재 창의 숨김 상태를 새 controller에도 적용한다.

이 wrapper는 child를 Offstage로 옮기거나 Texture를 조기에 제거하지 않는다.
문서 교체·앱 종료 때에는 기존 lifecycle의 확정 GPU retirement 경계를 사용한다.
장치 오류의 fallback을 재개만으로 복구하지 않으며 명시적 `NativeGpuSession` 복구와
분리한다. 실제 OS의 최소화·다중 모니터 이동·화면 배율 전환은 플랫폼 실행에서
추가 확인하며 widget event 계약 검사와 구분한다.

검증: Flutter 3.47.9에서 숨김 3종·inactive·DPR 변경·다른 창의 전역 알림·연속 metric 변경의
widget 계약 7개와 package analyze가 통과했다. 실제 OS 창 조작이나 물리 GPU의 실행 결과로
표현하지 않는다. 앞선 단일·다중 페이지 수명 검사는 stack 통합 뒤 다시 대조한다.

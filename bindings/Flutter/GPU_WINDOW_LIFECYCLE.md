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
처음부터 숨겨진 상태로 mount하거나 숨김 중 controller를 교체해도 재개 전에는
첫 create·snapshot capture를 시작하지 않는다. 이전 controller의 늦은 pause 오류는
새 controller의 오류로 전달하지 않는다.

이 wrapper는 child를 Offstage로 옮기거나 Texture를 조기에 제거하지 않는다.
문서 교체·앱 종료 때에는 기존 lifecycle의 확정 GPU retirement 경계를 사용한다.
장치 오류의 fallback을 재개만으로 복구하지 않으며 명시적 `NativeGpuSession` 복구와
분리한다. 실제 OS의 최소화·다중 모니터 이동·화면 배율 전환은 플랫폼 실행에서
추가 확인하며 widget event 계약 검사와 구분한다.

검증: Flutter 3.47.9에서 widget 계약 16개가 통과했다. 기존 숨김 3종·inactive·DPR·다른
창의 전역 알림·연속 metric 검사에 초기 숨김 3종, 숨김 중 controller 교체, physical size만
바뀐 경우, 숨김과 metric callback의 경합을 추가했다. AppKit mock 채널에서는 native hide
응답 전 재개해도 uncover하지 않는지, 현재 hide 실패의 호스트 전달과 CPU fallback,
교체된 controller로 이전 pause 오류가 전달되지 않는지를 확인했다.

AppKit 검사는 실제 controller와 mock 플랫폼 뷰·cover 응답을 사용한다. 실제 OS 창 조작,
native layer의 물리적 가림이나 물리 GPU의 실행 결과로 표현하지 않는다. 이 wrapper는 앱의
lifecycle 이벤트를 받으므로 한 창만 최소화하고 다른 창을 표시하는 OS 이벤트도 별도 확인이
필요하다. 앞선 단일·다중 페이지와 tile-group 수명 검사는 stack 통합 뒤 다시 대조한다.

참조 example은 GPU session이 있는 동안 `NativeGpuLifecycle`을 페이지 뷰의 같은
위치에 유지한다. 복구로 controller가 바뀌면 observer는 새 controller를 보고,
확정 종료 뒤 CPU fallback에서도 문서·editor 자식의 widget 수명을 유지한다.
pause 오류는 호스트 진단에 전달하고 문서와 renderer 종료는 기존 owner가 맡는다.

## 취소 가능한 창 종료 (P122)

참조 앱의 `NativeWindowClose`는 Flutter의 `didRequestAppExit` 요청을 기존
`NativeDocumentLifecycle.close()`에 연결한다. 저장 확인·입력 정리·GPU retirement·
문서 owner 종료가 끝날 때까지 viewport를 mount한 상태로 유지하고, 완료된 경우에만
`AppExitResponse.exit`를 반환한다. 초기 연결 또는 다른 문서 작업 중에는 종료를 취소한다.

중복 요청은 진행 중인 close 결과를 공유한다. 저장 취소, 확정되지 않은 retirement,
owner 종료 오류에서는 창을 유지하며 재요청할 수 있다. close callback이 교체되거나
widget이 제거된 뒤 도착한 결과는 새 창의 종료를 승인하지 않는다. 오류 표시 callback의
실패 역시 종료 승인으로 바뀌지 않는다. 강제 프로세스 종료는 이 계약에 포함되지 않는다.

`native_window_close_test.dart`는 실제 Flutter binding의 exit 요청을 통해 대기 중인
viewport 유지·중복·취소 후 재시도·정리 오류·소유자 교체·unmount를 확인한다. 기존
문서 lifecycle 검사는 저장/편집 경합과 GPU retirement 뒤 owner 종료 순서를 보호한다.
OS 창 관리자와 실제 플랫폼 GPU 종료 검증은 이 widget 회귀 결과와 구분한다.

Linux 로컬 실행에서는 Flutter 3.47.9 release 참조 앱을 Xvfb/Mesa llvmpipe로
열고 실제 `WM_DELETE_WINDOW` 요청을 보냈다. 저장 확인을 취소한 뒤 창이 유지됐고,
다시 요청해 폐기를 선택한 뒤 프로세스가 exit 0으로 종료됐다. 강제 종료 신호를
정상 종료의 근거로 사용하지 않았다. 이 실행은 기존 Native 라이브러리
`b5064fae1eefff92d60464c51844a38debec09f9`를 사용한 Linux 소프트웨어 GPU 확인이며,
물리 GPU·macOS·Windows 및 최종 문서 자료 검증은 포함하지 않는다.

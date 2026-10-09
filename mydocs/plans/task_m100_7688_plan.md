---
kind: plan
status: active
last_verified: 2026-10-09
---

# #7688 Canvas·SVG·PNG 출력 정합성 조사 계획

Issue: [#7688](https://github.com/edwardkim/rhwp/issues/7688). 관련 멀티 렌더러 트래킹은 #536이다.

## 범위와 기준

사용자는 이슈 등록에 이어 조사를 승인했다. 기준은 최신 `upstream/devel`의
`6f66932a73ee6fd05a74a83bdc438c9499a89a12`이며, 별도 작업 브랜치
`work/issue7688-renderer-audit-20261009`에서 진행한다.

현재 제품의 Canvas2D·SVG·Native Skia PNG 소비 경로와 최종 화면을 조사한다.
CanvasKit은 기존 baseline의 대조군으로 유지한다. #536의 미병합 개발 범위를 대신하지 않는다.

## 실행 순서

1. 동일 트리 생성 본체, Canvas 보충 메트릭 선택, portable export 가드와 복원 경로를 추적한다.
2. Docker에서 최종 source SHA의 최적화 WASM을 빌드하고 root `pkg/`와 Studio/public의 해시를 확인한다.
   공용 target은 기본 작업공간의 `/home/edward/mygithub/rhwp/target/pr-review`를 재사용한다.
3. Native Skia 기능을 포함한 동일 바이너리로 SVG·PNG를 출력한다. 기존 baseline manifest의
   문단·탭·표·이미지 crop·수식·그룹 도형 6개 항목에서 screen/print 프로필을 비교한다.
4. 같은 Chrome 실행 파일로 SVG raster와 Canvas 캡처를 확보한다. 실제 Studio 화면에서는
   메트릭 모드 전환 전후의 트리·쪽수·내보내기 후 복원 결과를 별도로 대조한다.
5. 대표 이미지의 직접 판독과 구조 차이로 실행 결함·출력 계약 차이·글꼴 공급 차이·미검증을 구분한다.
   수정 필요성이 확인되면 독립 한컴 출력과 적용 경계를 먼저 확정한다.

## 증적과 판정

입력/source/산출 해시·profile·메트릭 모드·browser/backend/surface·DPI/DPR를 보존한다.
진단 산출물은 기본 작업공간의 ignored
`output/pr-review/renderer-backend-audit-20261009/`에 둔다.
기존 baseline 허용치는 바꾸지 않는다. backend 간 비교 통과를 한컴 출력 일치로 보고하지 않으며,
기준 PDF가 없는 부분은 미검증으로 남긴다. 원격 게시·push·PR·이슈 종료는 이번 조사 결과와 구분한다.

## 1단계 실행 결과

대표 6개 입력·12쪽의 메트릭 조사와 Docker fresh WASM, Native Skia, 독립 한컴 Print PDF 비교를 수행했다.
[1단계 결과](../working/task_m100_7688_stage1.md)에 실행 결함·계약 차이·미검증과 재현 증적을 연결했다.
Native 수식의 한글 네모가 우선 수정 대상이며 공통 crop·화살촉 차이와 구분한다.
zoom/DPR/resize 15조건의 트리 유지와 3개 원래 조건 복원 PNG 일치를 확인했다.
제품 변경·새 baseline·원격 push는 없으며, 이슈는 후속 구현·검증이 남아 active다.

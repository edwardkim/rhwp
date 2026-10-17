# #7974 — 전체 검증과 로컬 PR 준비

2026-10-17 사용자가 **“전체 검증·PR 준비 승인”**으로 시각 통과 후 실행안을 승인했다.
원격 push·PR 생성·ready 전환·merge·이슈 종료는 이번 승인에 포함하지 않는다.

- 기본 경로: `collaborator_self_merge`.
- 추가 적용: `intake_and_review`, `local_validation`, `visual_fixture_evidence`.
- 읽은 권위 문서: `pr_review_workflow.md`, `pr_review/README.md`와 위 네 자식 문서.
- 제품 commit: `6b072955e9b83d8f7729133eda5e93536ee453f5`.
- 시작 HEAD: `88535330fdce00379c0f6284d182f2a116d4efb3` (제품 검증 후 문서만 추가).
- 새 fetch의 upstream/devel: `515bd72dcde0eecdeece333666b675354b7fa91d`.
  HEAD가 이를 포함하며 추가 통합은 필요 없다.
- 공유 target: `/home/edward/mygithub/rhwp/target/pr-review`; Cargo 작업은 순차 실행한다.
- 기존 Native/Docker WASM·집중 검사·전쪽 시각 TSV와 사용자 판정은 stage1의 동일 소스 증거를 재사용한다.
  새 제품 소스 변경이 없으므로 동일 WASM을 반복 빌드하지 않는다.

## 실행 중

`output/pr-review/backlog-7949/validation-next/plan-ko.md`의 순서로 미완료 lint·전체 Rust·Studio·
공개 PDF API/실제 Worker·npm 소비 경계를 검증한다. 개별 명령·시각·종료값·로그는 같은 경로의
`validation-results.json`에 저장하며, 미실행 검사를 통과로 기록하지 않는다.

exact-base Native/fresh WASM의 공통 정상 대조군 전쪽 하락 비교도 남아 있다. 새 지원 입력은 base에서
거절된 기존 증거와 독립 PDF 비교를 연결하고 기준 TSV가 있는 것으로 보고하지 않는다.

기본 Studio 7700, 다른 작업의 WIP와 공유 target은 보존한다. task Studio 7795의 생성 glue는
로컬 실행 산출물로 남기고 source commit에는 포함하지 않는다.

## exact-base 비교에서 검출한 회귀와 보정

Native·WASM32·workspace all-target Clippy와 workspace build는 통과했다. 이후 release lib 컴파일 중
exact-base fresh WASM의 공통 대조군 27쪽 비교에서 개인 정상 문서의 9쪽이 0.01%p 이상 하락했다.
최대 하락은 9쪽의 99.85616 → 98.40687% (1.44929%p)다. 90% 통과·앞선 사용자 시각 판정을
하락 기준의 예외로 해석하지 않았다. 후속 통합 검사를 실행하기 전에 이 작업 소유의 전체 검증
runner를 중단했으며, 아직 끝나지 않은 release lib 검사도 통과로 세지 않았다.

하락 영역은 source보다 작은 출력에 배치된 이미지다. 기존 코드가 확대의 Interpolate 선택을
축소의 coverage/prefilter까지 강제한 가정을 제거한다. PDF 32000-1 §8.9.5.3의 확대 정의와 원본
PDF의 실제 축소 출력을 근거로, 원색 선언은 PNG에 계속 보존하되 최종 draw의 source rectangle과
device rectangle을 비교한다. 축소에서는 기존 backend coverage/profile을 유지한다.
Native·Canvas의 실제 변환 행렬과 crop을 소비하고, CanvasKit도 draw 시점의 행렬·source 창을
사용한다. DOM은 실제 source 창·화면 배율·devicePixelRatio를, SVG는 intrinsic 96-DPI frame을
사용한다. 문서 ID·목표 점수·배율 보정 상수는 넣지 않는다.

근거: [PDF 32000-1:2008 §8.9.5.3](https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf).
작은 checkerboard 축소 입력을 별도로 생성하여 수정 전/후를 확인한다. 이 절은 보정 방향과
검증 계획이며 새 Native/fresh WASM 검증 완료를 뜻하지 않는다. 새 코드에 영향을 받는 빌드·
시각·전체 회귀는 새 결과로 갱신하고 기존 stage1 결과와 구분한다.

## PR 준비에 남은 판정

- 새 지원 입력의 기준 TSV 부재는 2026-10-17 사용자가 **“신규 지원 예외·공통 대조군 비교 승인”**으로
  판정했다. devel이 거절하는 13개만 동일 입력의 미지원 오류 → 정상 열기·독립 PDF 전쪽 90% 이상으로
  평가한다. 양쪽에서 열리는 정상 대조군의 Native/fresh WASM 페이지별 하락 비교는 그대로 수행한다.
  원래 90%·쪽수·직접 판독 기준이나 공통 대조군의 하락 허용치를 완화한 승인이 아니다.
- 개인 type002 PDF는 저장소 밖에만 있다. `intake_and_review.md` 2.8의 50 MiB 미만 입력 보존 조건에
  해당하므로, 미준수 상태를 감추지 않고 공개 가능한 입력/별도 범위 판정과 구분한다.
- PR 번호별 self-review·오늘할일은 PR 등록 뒤 정확한 code head CI 성공 후 작성한다.
- 검증과 본문·대표 이미지·생성 명령까지 로컬로 준비한 다음 원격 반영 승인을 요청한다.

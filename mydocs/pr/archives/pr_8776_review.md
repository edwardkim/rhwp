# PR #8776 self-review — 릴리즈 렌더 검사와 Linux 실행 환경 보정

## 최종 판정

승인. code head `c09e8c9f5e0389776f1fc29c6fe9ca75779a6d56`의 필수 CI와 실제 산출물을
확인한 뒤 2026-11-08 16:13:38 KST에 `9409e7f6ebd26daaef8e68a73ecf9d102cce0e20`로
정상 병합했다. [공개 self-review](https://github.com/edwardkim/rhwp/pull/8776#pullrequestreview-5817008041).

collaborator self-merge와 CI 변경 후속 기록 옵션 2 경로다. 작업지시자가 두 CI 결함 보정,
제한된 글꼴 폴백 판정·90분 상한, PR·CI·self-review·merge와 릴리즈 계속 진행을 승인했다.
원 PR에 trailing commit을 추가하지 않고 이 별도 문서 PR로 검토·오늘할일을 보존한다.

## 변경과 검토

이전 Full Renderer Sweep은 반복 CLI 재링크 때문에 60분 상한에 도달했다. 배포 Linux x86_64
archive는 GLIBC_2.43 요구로 Ubuntu 24.04에서 실행되지 않았다. CLI feature별 executable
재사용과 Ubuntu 24.04 runner 고정을 적용했다. 원인·선행 계약·승인·로컬 실행은
[구현 기록](../../working/release_0_9_0_ci_runtime_corrections.md)에 연결했다.

전체 브라우저 실행에서 제품 계약에 따른 글꼴 폴백을 CanvasKit 실패로만 계산하던 검사와
portable metrics 밖의 진단 API 호출도 보정했다. 폴백은 독립 font plan·실제 선택 오류가
일치할 때만 별도 inventory에 넣고 명시적 Canvas2D와 정확히 같은 픽셀·비어 있지 않은
내용을 요구한다. 필수 CanvasKit readiness 8건은 폴백을 허용하지 않는다. 알 수 없는 오류,
GPU 초기화·이미지 실패는 실패로 남긴다. CanvasKit 성공·성능 집계에 폴백을 포함하지 않는다.

제품 Rust·Studio renderer·manifest·fixture·golden은 변경하지 않았다. 조판 구현 변경과
새 독립 Print 시각 회귀는 비해당이다. 기존 public manifest 입력을 재사용했다. CLI cache key는
feature·작업 경로·환경을 포함하고 build/export 오류를 보존한다. runner와 60→90분 상한 외
workflow event·권한·게시 조건·문서/profile/surface 수·시각/성능 기준은 유지했다.

## 완료한 검증

- 로컬 Python 계약 46건, renderer backend 계약, 메트릭 복구 6건, parity provenance self-test 통과.
- Cargo dev 빌드 2회로 Native export 210개를 완료했다. SVG·PNG 189쌍은 제품 기준 `1b78dc23fd8`과
  동일하며 PDF 21개를 생성했다. 전체 browser matrix 378개·집중/readiness 40개를 완료했다.
  필수 readiness·parity 각각 8건과 별도 폴백 검사는 통과했다.
- 제품 source가 그대로여서 같은 source의 최적화 CI WASM을 재사용했고 실제 소비 해시를 확인했다.
  새 후보 WASM을 로컬에서 다시 빌드했다는 뜻은 아니다. 14개 Cargo 패키지의 파일 목록·내용도
  [동일](../assets/pr8776/unchanged-package-inputs.json)하다.
- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/42455796623) attempt 1 성공:
  Archive A–E 12,803건, 선택 Native 237건, Studio 2,778건 통과·기존 skip 2건.
  Lint·Frontend 후속 검사·Build & Test 집계도 성공했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42455796445) 네 언어,
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42455796286),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42455796280),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42455796613),
  [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42457763671)가 성공했다.
  수동 취소·재실행·경고 재분류는 없었다. 실제 API의 단계 정보도 완료를 확인했다.
- 렌더 artifact의 Canvas 3건·필수 readiness 8건·직접 PDF 3건이 통과했고 WASM 초기화 응답
  18건의 해시가 빌드와 일치했다. 기존 PDF report-only 차이 4건을 성공으로 세지 않았다.
- 병합 직전 최신 base `1b78dc23fd8`와 후보의 실제 merge tree
  `05b1be50ba870f1f922ad25fcf33981c8a9d0aa6`를 GitHub 합성 병합 tree와 대조했다.
  최신 check·mergeable을 확인해 `--merge --match-head-commit`으로 병합했으며 admin 우회는 없었다.

[실제 CI·로그 집계·병합 증거](../assets/pr8776/ci-results.json) ·
[실제 렌더 artifact 증거](../assets/pr8776/render-evidence.json) ·
[원격 변경 범위](../assets/pr8776/remote-scope-verification.json).

## 남은 릴리즈 절차

이 기록 PR의 실제 review-only preflight·required check를 확인하고 병합한 최종 devel에서
승격 workflow 13개를 실행한다. 새 Linux archive를 Ubuntu 24.04에서 version·PNG까지
실행하고 실제 Full Renderer 완료를 확인한다. 이전 12개 성공·1개 시간 초과 결과를 새 head의
성공으로 대신하지 않는다. 제품 범위는 `1b78dc23fd8`에서 고정하며 다른 제품 PR을 추가하지 않는다.

최종 승격·main PR·태그·crates.io 및 기존 채널의 실제 게시·공개 소비 검증은 아직 남아 있다.
이 운영 기록이 배포 완료를 뜻하지 않는다. 릴리즈가 사용 중인 output·target은 완료 후 정리한다.

# PR #9191 검토 — 릴리즈 태그의 Native Skia 회귀 재사용

## 최종 판정

**승인.** 작성자 `edwardkim`의 self-review다. 검증한 head `5951f0bf41177c87106f93eed0285dcfdd57b584`의 CI와 최신 devel 병합 시뮬레이션을 확인했다. 작업지시자 승인 후 2026-11-19 17:35:56 KST에 squash 병합했다. [원 PR](https://github.com/edwardkim/rhwp/pull/9191) · [병합 커밋](https://github.com/edwardkim/rhwp/commit/36ae26d6f60de452a161afd14500ba9080794cac)

## 변경과 검토 범위

승격 PR에서 Native Skia 성공 후 실제로 검증한 병합 트리를 기록한다. 태그 CI는 같은 저장소의 `devel → main` 승격 PR, 해당 실행 회차의 병합 전 검사 성공, 태그와 검사한 트리의 일치를 확인한 뒤 Native Skia 결과를 재사용한다. 증거가 부족하면 `release-test` 전체 검사를 실행한다. 배포 압축 파일의 `release` 바이너리로 PNG를 출력하는 경로는 Release Binary에 있다.

조판 원칙은 비해당이다. 변경 범위는 `.github/workflows/ci.yml`, workflow 계약 테스트와 배포 매뉴얼이다.

## 검증 입력과 결과

- 검증 base는 `af53659e19f70c18d0d3591a719c238235007a05`다. 병합 시뮬레이션, 후보, 원격 병합 커밋의 tree는 모두 `067dc0c59cc8f41d9effb64cf0f03a613f5997b3`이다.
- 로컬 workflow·정책 계약 549건, actionlint, 배포 매뉴얼 링크 검사와 `git diff --check`를 통과했다. 다른 트리, 다른 실행 회차, 실패, skip, 증거 누락과 만료에서 전체 Native 회귀로 전환했다.
- 검증 입력 커밋 확인은 충족이다. v0.9.1 Linux x86_64 공개 배포 압축 파일로 `samples/issue2527_empty_linesegs.hwpx`를 열어 794×1123 PNG를 출력했다. 실제 입력과 검토한 head의 샘플 SHA-256은 모두 `43b3090f33370cc678da8ed3e55e32901ccb379b50153114ae41fa3984006f10`이다.
- [Native Skia](https://github.com/edwardkim/rhwp/actions/runs/44483756731/job/135422242606)는 `release-test`에서 248건 통과, 실패 0건이었다. [CI](https://github.com/edwardkim/rhwp/actions/runs/44483756731)의 Lint, Frontend, 회귀 A–E와 최종 Build & Test, [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44483756496), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/44483756464), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/44483756524), [최종 CI 정책](https://github.com/edwardkim/rhwp/actions/runs/44488124162)이 같은 head에서 통과했다.

## 병합 후 확인

[실행 시간 데이터 갱신](https://github.com/edwardkim/rhwp/actions/runs/44488965691)이 PR의 성공한 B–E worker 자료를 게시했다. 게시 커밋은 `a70c99a395607e8df1ee4dd17df8f80efec91fa8`이다. [#9154](https://github.com/edwardkim/rhwp/issues/9154)는 Ubuntu 러너 전환 추적을 위해 열린 상태다. 다음 main 승격 PR과 실제 태그 실행에서 같은 트리의 Native Skia 성공 증거가 전달되는 경로를 확인한다.

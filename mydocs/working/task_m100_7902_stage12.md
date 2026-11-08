# #7902 12단계 — PR #7941의 browser 계약 실행 환경 보완

Issue: #7902. PR: #7941. 검토 head: `9e3fce70048607b11525ddcecd030d961a25134c`.
[11단계](task_m100_7902_stage11.md)의 로컬 전체 검증 뒤 원격 CI를 확인하던 중 발견했다.

## 실패 근거와 보완 계획

[Render Diff run](https://github.com/edwardkim/rhwp/actions/runs/38833024839)의
`Validate Visual Sweep Canvas font registration`에서 실제 Chrome 실행이
`No usable sandbox`로 종료됐다. 첫 계약1개 FAIL·정리 음성 대조1개 PASS다.
새 검사에 전달한 다운로드 Chromium은 Ubuntu의 설치 Chrome용 AppArmor 경로에 포함되지 않는다.
[Chromium 공식 설명](https://chromium.googlesource.com/chromium/src/+/main/docs/security/apparmor-userns-restrictions.md)에
같은 실행 제한이 기록돼 있다. 캡처나 글꼴 assertion까지 진행하지 못한 실행 환경 실패다.

- 글꼴 등록 계약을 Chromium 다운로드 전으로 옮겨 설치된 Chrome에서 실행한다.
  같은 head의 frontend package에서는 이미 이 설치 Chrome과 실제 HFT face 선택 검사가 통과했다.
- 실제 Render Diff는 기존 고정 Chromium `1660786`과 측정 명령을 유지한다.
- browser sandbox를 끄거나 전역 커널/AppArmor 설정을 바꾸지 않는다.
- workflow 배선 검사·해당 실제 browser 검사·actionlint를 수행한 뒤 보완 commit을 push한다.
  Rust·제품 JS·글꼴·기준 출력은 바꾸지 않으므로 완료한 전체 Rust와835쪽×2를 재실행하지 않는다.
  새 head의 실제 CI를 다시 확인하며 최초 실패를 성공으로 바꾸어 기록하지 않는다.

## 공개 fixture 원장 누락

같은 code head의 frontend package는 실제 HFT browser·undo gate2666개까지 통과했으나
`every public repository font has matching bytes and redistribution notices`에서 실패했다.
새 `RHWPHftOutline.otf`·`RHWPHftOutlineJamo.otf`의 바이트는 커밋했지만 공개 글꼴 원장의
`repositoryOnlyFonts` 등록을 누락했다. 저장소에서 만든 독창적인 사각형 fixture 두 개의 실제
SHA와 기존 MIT 전문 `LICENSE`·저작권을 등록한다. 글꼴 bytes·생성기·배포 글꼴36개는 유지한다.
선행 라이선스 문서만으로 이 엄격한 inventory gate를 충족했다고 판단한 검증 누락을 수정한다.

## 실행 결과

- 보완 뒤 workflow 계약42 PASS, 실제 글꼴 등록/Chrome 실패 정리2 PASS, actionlint exit0.
- 공개 font inventory·원본 SHA/전문 notice 계약2 PASS. 검사나 허용치를 완화하지 않았다.
- 첫 제출 head의 원격 CI는 종료됐다. 실제 Rust archive/worker와 Native Skia·CodeQL·
  adapter·roundtrip은 성공했고, 위2개 실패가 frontend/Canvas 및 최종 집계를 막았다.
  새 head 전체 CI 성공으로 바꾸어 보고하지 않는다.
- 이번 변경은 workflow 배선·Python 계약·원본 글꼴 원장·작업 기록이다. Rust source/test와
  제품 JS·fixture bytes·시각 산출물은 그대로이므로 기존 검증과 최종 PNG를 재사용한다.
  PR 본문의 이미지 URL을 새 head SHA로 갱신하고 새 CI에서 두 실패의 해소를 확인한다.

원시 로그는 ignored `output/pr-review/issue7902-render-repair-20261014/`의
`stage11-render-diff-ci-failed.log`, `stage12-frontend-package-ci-failed.log`,
`stage12-validation.json`, `stage12-font-notice-contract.log`에 보존한다.

# 승인한 글꼴 복사와 기존 PR CI 보정

사용자가 win10-ted에서 글꼴을 복사해 진행하고 #9293의 준비한 고지 2파일을 push하도록
명시적으로 승인했다. 9개 이슈의 기존 검증은 계속하며, 다른 PR의 merge 승인을 추가로
추정하지 않는다. 이전 분석은 stage8과 source39 증거에서 잇는다.

## #9293의 승인 범위 보정

원격 PR의 head `b4faa9e6d6633ce0fed56cd0540eea5ea01c3645`와 reviewer 미지정을
다시 확인하고, 그 head에 검토 전용 sparse worktree를 만들었다. Git에 등록된 글꼴
74개를 모두 실제 경로에 노출해 부분 checkout이 검사를 숨기지 않도록 확인했다.
공개 글꼴 inventory와 재배포 고지 검사는 수정 전 1 PASS / 1 FAIL이었다.
미등록 fixture 글꼴 때문에 실패한 검사를 기준값 변경 없이 보정 후 2 PASS로 확인했다.
초기 sparse 환경의 `FONTS.md` 누락은 환경을 보완한 뒤 양쪽을 다시 실행해 제거했다.

변경 파일은 `assets/fonts/licenses/font-notices.json`과
`tests/fixtures/hft_mapped_symbol_program_tracking/LICENSE.txt` 두 개다.
기존 합성 fixture 바이트·renderer·검사 기대값·CI 정의는 바꾸지 않았다.
커밋 `83b671c2a63a87b0946d2d2aa3f512ecf38945d2`를 기존 upstream head 브랜치에
일반 fast-forward push했고 GitHub API로 PR의 새 head를 확인했다. 새 head의 CI는 별도로
확인하며, 원격 반영만으로 CI 성공 또는 통합 완료로 표시하지 않는다.

## Windows 글꼴의 실제 복사

SSH의 PowerShell에서 원격 크기·SHA를 읽고 SCP로 `C:\Windows\Fonts`의 Arial 4종과
`HANBatang.ttf`를 작업 전용 private 폴더에 복사했다. 다섯 파일 모두 원격 SHA·크기와
일치한다. 폴더는 0700, 파일은 0600이며 시스템 설치·교체와 Git/배포물 추가는 하지 않았다.
Arial regular의 실제 SHA는 `c9b76220a5be42ead4733611e417cd65c5fd8aeaa33eb56576ac378a37d130a1`,
함초롬바탕은 `0dbeb7b129fcec63f8d6b7f1d3b5bb991c1d599e725286518ccc598de643f014`다.
이들은 과거 정본의 `5f9f…`·`ba2a…` 버전과 다르므로 같은 파일을 복원했다고 기록하지 않는다.

새 복사본을 원래 CFF 프로세스 namespace와 함께 명시적으로 공급하여 chemical의 알려진
영향 10쪽을 frozen base와 실제 후보 Native로 대조하는 진단을 시작했다. 원본·PDF·
명시 환경·producer SHA와 실제 명령을 고정하고, 진단 결과를 정본 조건 충족과 구분한다.
글꼴 우선순위는 원본 TTF → 확인된 TTC → CFF를 유지한다.

정식 영수증과 재현한 전후 로그는 `mydocs/pr/assets/issues7754_7765/source40/`에 있다.
전체 Rust nextest, fresh WASM 원본의 남은 쪽과 편집·저장·재열기·Studio·전체 보호 조건을
계속 검증한다. Native 583쪽·최소 글꼴 집중 검사·Clippy와 workspace build의 완료 결과는
stage8에 남긴 판정 그대로 유지한다.

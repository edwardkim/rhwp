# PR #8896 메인테이너 완료 처리 계획

## 범위와 승인

2026-11-11 작업지시자가 “8896 PR 은 메인테이너쪽에서 완료 처리로 진행합니다”라고 지시했다.
기존 원 head `51a960e392a3d79a4d28f78fa90a2aa88ad8c0d7`와 공개 3개 입력·독립 Print PDF를 보존한다.
현재 devel은 `4c876654d86d071d342fc78706c51cfff6ddbde4`, 충돌 없는 검토 병합 source는 `db4a808937260a46bbddf6ff7af113f28bec5733`다.
기존 4건 통합 worktree와 미해결 #8897은 변경하지 않으며 #7620의 종료로 확대하지 않는다.

## 남은 처리와 완료 기준

1. 기존 검토의 보류 사유와 실제 source·테스트·독립 Print를 대조한다.
2. 최신 devel 병합 결과에서 Native/fresh 최적화 WASM의 3개 입력 전 6쪽과 실제 빈 host 글줄 대조군을 검증한다.
   #8945의 8쪽 첫 조각·아래 여백 계약도 같은 후보에서 유지되는지 확인한다.
3. 정본 보호 18개 문서·Native/WASM 36개 데이터셋과 필수 편람 gate를 완료한다.
   직전 #8945에서 정합한 글꼴·연결 이미지·source-metrics 환경을 재사용하고, 현재 devel의 기록된 PNG 해시와 후보 실제 출력을 비교한다.
   과거 정본 미재현을 새 회귀와 구분하며 이전 PR 한정 예외를 자동 적용하지 않는다.
4. 집중 회귀와 변경 범위의 필수 검증을 완료한다. source/test/fixture 보정이 없으면 원 exact-head 녹색 Full CI의 광범위 회귀 재사용 조건을 확인한다.
   보정하면 새 head의 로컬 필수 묶음과 CI를 완료한다.
5. 최종 승인·병합 및 #8693의 실제 해결 범위를 확인하고 기록·댓글·동기화·전용 산출물 정리를 수행한다.
   fork `kidsnote/rhwp`는 push 권한이 없으므로 보정 commit이 필요한 경우 원 기여 SHA를 보존한 upstream 수용 PR을 사용한다.

## 기존 근거

- 원 PR: https://github.com/edwardkim/rhwp/pull/8896
- 기존 보류 설명: https://github.com/edwardkim/rhwp/pull/8896#issuecomment-6784903830
- 원 문제: https://github.com/edwardkim/rhwp/issues/8693
- 이전 로컬 기록: 다른 작업의 `review-integration-20261110-hyunmyungjanelee/mydocs/pr/pr_8896_review.md`를 읽기 전용으로 확인했다.

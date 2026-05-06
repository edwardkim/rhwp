# Task #598 Stage 4-6 완료보고서 — PR open 전 검증 보강

## 작업 범위

- #598 이슈 본문의 `검증` 항목과 PR #642 검증 항목 재대조
- rhwp-studio e2e에 본문 각주 마커 좌/우 커서 이동 검증 추가
- rhwp-studio e2e에 본문 각주 마커 클릭 후 각주 편집 모드 진입 검증 추가

## 구현 내용

`rhwp-studio/e2e/footnote-delete-confirm.test.mjs` 에 다음 검증을 추가했다.

- `ArrowRight`: 본문 각주 마커 왼쪽 `charOffset=7` 에서 오른쪽 `charOffset=8` 로 한 칸 이동
- `ArrowLeft`: 본문 각주 마커 오른쪽 `charOffset=8` 에서 왼쪽 `charOffset=7` 로 한 칸 이동
- 본문 첫 번째 각주 마커 좌표 클릭 후 각주 편집 모드 진입 확인
- 클릭 후 원본 본문 문단/컨트롤/각주 인덱스 연결 확인

기존 e2e 검증은 유지했다.

- 각주 앞 Backspace 일반 텍스트 삭제 및 Undo anchor 복원
- Delete 경로 확인창/취소
- Backspace 경로 동일 확인창/확인 삭제
- 후속 각주 번호 재계산
- Ctrl+Z 복원

## 검증

실행 명령:

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/footnote-delete-confirm.test.mjs --mode=headless
```

결과:

- 본문 각주 마커 좌/우 방향키 이동: 통과
- 본문 각주 마커 클릭 후 각주 편집 모드 진입: 통과
- 각주 앞 Backspace 일반 텍스트 삭제/Undo: 통과
- Delete 확인창/취소: 통과
- Backspace 확인창/삭제: 통과
- Ctrl+Z 복원: 통과

## 판단

이슈 본문의 e2e 검증 항목 중 PR 본문에서 약하게 표현되던 `본문 각주 마커 클릭` 과 `좌우 화살표 이동 단위` 검증을 자동화했다. 한컴 직접 비교는 컨트리뷰터 환경상 수행하지 못했으나, 이슈의 진행 안내에 따라 PR 본문과 이슈 코멘트에 메인테이너 시각 판정 포인트로 남긴다.

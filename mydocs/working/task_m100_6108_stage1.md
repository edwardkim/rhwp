# Task M100 #6108 Stage 1 완료 보고 — 쪽 배치별 맞춤 계산 계약 단일화

- **이슈**: [#6108](https://github.com/edwardkim/rhwp/issues/6108)
- **브랜치**: `codex/issue-6108-zoom-fit`
- **기준 commit**: `upstream/devel` `2166f4065`
- **Stage 범위**: 순수 맞춤 계산과 상태 모델만 변경. 명령·UI 진입점 및 무사용 API 정리는 Stage 2에 유지

## 구현 결과

- 자동·한 쪽은 1×1, 두 쪽·맞쪽은 2×1, 여러 쪽은 정규화된 `columns×rows`로 해석한다.
- 폭 맞춤은 선택한 배치의 한 행 전체, 쪽 맞춤은 가로×세로 블록 전체와 내부 page gap을 계산한다.
- 프레임 여백은 기존 화면 계약인 좌우 40px, 상하 20px를 유지하고 page gap은 페이지 사이에만 한 번씩
  차감한다.
- 폭·쪽 맞춤 모두 `MIN_DOCUMENT_ZOOM`~`MAX_DOCUMENT_ZOOM`, 즉 10~500%를 공유한다.
- `resolveZoomFitZoom()`이 `fitWidth`와 `fitPage` 모두 현재 `PageArrangement`를 사용한다.
- 확대/축소 대화상자 상태 계산도 같은 resolver를 사용하고, 여러 쪽 전용 중복 계산기를 삭제했다.
- legacy 한 페이지 helper는 자동 1×1 배치의 공용 계산기로 위임해 기존 소비처와 수치를 보존했다.

## 테스트 우선 확인

구현 전 새 계약 테스트를 먼저 실행해 다음 RED를 확인했다.

- `calculateArrangementFitPageZoom` export 부재
- 여러 쪽 4×1의 기존 외곽 gap 공식 차이: 실제 `0.484375`, 새 계약 `0.478125`
- 저장된 `fitPage`가 4×1 배치를 무시: 실제 `0.88`, 새 계약 `0.478125`

구현 후 같은 focused suite는 28/28 통과했다.

```text
node --test \
  tests/page-arrangement.test.ts \
  tests/zoom-fit.test.ts \
  tests/zoom-dialog.test.ts \
  tests/zoom-fit-mode-persistence.test.ts

tests 28, pass 28, fail 0
```

검증한 주요 매트릭스는 다음과 같다.

| 배치 | 쪽 맞춤 기준 | 1600×900 뷰포트, 800×1000 쪽, gap 10 결과 |
| --- | --- | ---: |
| 자동·한 쪽 | 1×1 | 0.88 |
| 두 쪽·맞쪽 | 2×1 | 0.88 |
| 여러 쪽 2×2 | 전체 2×2 | 0.435 |
| 여러 쪽 4×1 | 전체 4×1 | 0.478125 |
| 여러 쪽 8×8, 작은 창 | 전체 8×8 | 0.1 하한 |
| 한 쪽, 큰 창 | 1×1 | 5.0 상한 |

## 정적 검증

- 삭제한 `calculateMultiplePagesZoom`·`MultiplePagesZoomInput` 잔여 참조: 0건
- `npm exec tsc -- --noEmit`: 통과
  - 새 worktree에는 생성형 `pkg/rhwp.js`가 없어 최초 검사가 해당 모듈 탐색에서 중단됐다.
  - 원본 작업공간의 동일 WASM 산출물을 로컬 symlink로 연결해 검사한 뒤 즉시 제거했다. source와 Git
    변경에는 포함되지 않는다.
- `git diff --check`: 통과

## 다음 게이트

작업지시자가 Stage 1 결과를 승인하면 다음 단계에서만 아래 작업을 진행한다.

1. 메뉴·상태 표시줄·대화상자·저장 복원의 fit metrics/명령 경로 단일화
2. 여러 쪽에서 적용되지 않는 비율 선택 UI 비활성화
3. 소비처가 없는 이벤트·CanvasView wrapper·`is-neutral` class 토글 정리

#6109의 invalid 입력 표시와 원자 view-settings transaction은 이 Stage와 #6108 브랜치에 포함하지 않는다.

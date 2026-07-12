# PR #1973 리뷰 - raw IME/iOS page-local refresh 힌트 보강

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1973 |
| 제목 | task 1964: raw IME page-local refresh 힌트 보강 |
| 작성자 | jangster77 |
| base | `devel` |
| head | `task_m100_1964_raw_ime_refresh_guard` |
| 문서 작성 시점 head SHA | `f8b47af12b8b4be9b7b618741360b3041558b5fa` |
| 규모 | 문서 작성 시점 참고값: 5 files, +121 / -4 |
| mergeable | 문서 작성 시점 참고값: `MERGEABLE` |
| CI | merge 전 최신 PR head 기준 GitHub Actions 통과 필요 |
| 관련 이슈 | Closes #1964 |

## 변경 범위

- `rhwp-studio/src/engine/input-handler.ts`
  - raw IME/iOS 입력 후처리 라우터 `afterTextInputEdit()`가 `PageLocalTextEditOptions`를 받도록 확장했다.
  - command 기반 입력 경로와 같은 `shouldUsePageLocalRefresh()` 계약을 사용한다.
- `rhwp-studio/src/engine/input-handler-text.ts`
  - IME 조합 입력에서 실제 `insertedText`, 편집 전 page index, 편집 후 page index를 전달한다.
  - iOS fallback 입력에서 첫 입력 앵커 기준 `beforePageIndex`를 보존하고, 디바운스 렌더링 시 마지막 입력 결과와 함께 전달한다.
- `rhwp-studio/tests/input-edit-invalidation.test.ts`
  - raw IME/iOS 입력 경로가 command 입력과 같은 page-local 판정 힌트를 전달하는지 소스 계약 테스트를 추가했다.
- `mydocs/plans/task_m100_1964.md`, `mydocs/working/task_m100_1964_stage1.md`
  - 이슈 원인, 수정 방향, 검증 결과를 기록했다.

## 렌더 영향 및 visual sweep 판정

이 PR은 Studio의 raw IME/iOS 입력 후 refresh 라우팅 안전 가드를 보강한다. 정적 HWP/HWPX/PDF 렌더 결과를 바꾸는
레이아웃, serializer, parser, paint 변경이 아니라 입력 이벤트 이후 stale static layer 재사용을 막는 동작 보정이다.

따라서 `visual_sweep_guide.md` 기반 PDF/SVG 비교 대신, 입력 invalidation 계약 테스트와 Studio build로 검증한다.
시각 검증 asset은 생성하지 않는다.

## 로컬 검증

```bash
cd rhwp-studio
node --test tests/input-edit-invalidation.test.ts
npm run build
cd ..
git diff --check
```

결과:

- `node --test tests/input-edit-invalidation.test.ts`: 7개 통과
- `npm run build`: 통과
- `git diff --check`: 통과

보조 확인:

- `npm test -- input-edit-invalidation.test.ts`: 166개 통과

## 검토 결과

### 1. command 입력과 raw 입력의 page-local 판정 정보가 맞춰졌다

기존 command 기반 텍스트 입력은 `insertedText`, `beforePageIndex`, `afterPageIndex`를 통해 줄바꿈, 탭,
긴 입력, page 이동을 full refresh로 되돌릴 수 있었다. 이번 변경으로 raw IME/iOS 입력도 같은 판정 정보를 전달한다.

### 2. iOS fallback은 첫 입력 앵커 기준 page index를 보존한다

iOS fallback은 연속 input을 디바운스해 마지막에 렌더링한다. 이때 `_iosAnchor`가 첫 입력 위치를 유지하므로,
`beforePageIndex`도 같은 시점 값으로 보존하도록 `_iosBeforePageIndex`를 추가한 점이 적절하다.

### 3. 별도 하드코딩 분기 없이 기존 정책을 재사용한다

줄바꿈, 탭, 긴 입력, page 이동 여부는 기존 `isPageLocalTextEditCommand()` 정책에서 판단한다. raw 경로가 별도
임의 분기를 만들지 않고 같은 정책으로 들어가므로 유지보수 리스크가 낮다.

## 최종 권고

merge 후보로 판단한다.

merge 전 최종 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과
- 작업지시자 승인

merge 후 확인:

- #1964 auto-close 여부 확인
- auto-close 여부와 관계없이 #1964에 처리 완료 코멘트 남김
- PR 브랜치와 로컬 작업 브랜치 정리

# Task M100 #2392 Stage 1 완료 보고 - apply 계약 기준선 고정

- 이슈: #2392
- 브랜치: `issue-2392-picture-props-apply-pipeline`
- source 기준: `upstream/devel@af5902b659be9a4d86ad458d79c63353dba88167`
- 계획 commit: `293c764020488963113630994689c3dac3614860`
- Stage 1 commit: `8773d6b4310eb4057f13bd8d7af60c8e3c701831`
- 작성일: 2026-07-19

## 1. 완료 요약

제품 source와 test를 변경하지 않고 `PicturePropsDialog.handleOk`의 리팩터링 전 계약을 고정했다.

- schema v2 pre metrics를 같은 source에서 재생성했다.
- 대상 파일의 모든 35개 reported function과 aggregate를 기록했다.
- common, shape/line/group/OLE, image field의 conversion/default/diff/always-send 정책을 분류했다.
- 5개 setter와 header/footer 우선순위, undo/fallback/defensive empty patch 의미를 고정했다.
- unit, production build, 실제 Chrome headless undo 계약을 다시 실행했다.
- dialog instance 재사용에서 line patch가 과거 type에 의존할 수 있는 stale-control 위험을 발견했다.

상세 표는
`mydocs/tech/investigations/issue-2392/task_m100_2392_picture_props_apply_contract.md`에 기록했다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `mydocs/tech/investigations/README.md` | #2392 조사 묶음 링크 추가 |
| `mydocs/tech/investigations/issue-2392/README.md` | issue investigation index |
| `mydocs/tech/investigations/issue-2392/task_m100_2392_picture_props_apply_contract.md` | metrics, field, target, undo characterization |
| `mydocs/working/task_m100_2392_stage1.md` | Stage 1 gate 결과 |
| `mydocs/orders/20260719.md` | Stage 1 완료와 Stage 2 승인 대기 반영 |
| `mydocs/plans/task_m100_2392.md` | latest base와 Stage 1 관문 상태 갱신 |
| `mydocs/plans/task_m100_2392_impl.md` | unchanged/empty fixture 가정 보정 |

제품 source, test, package, lockfile, generated WASM은 변경하지 않았다.

## 3. metrics 결과

| 지표 | frontend 전체 | 대상 파일 |
|------|--------------:|----------:|
| files / reported functions | 215 / 2,386 | 1 / 35 |
| Total CC | 12,369 | 647 |
| Top 20 합 | 2,660 | - |
| CC>25 개수 / 합 | 70 / 4,297 | 2 / 560 |
| CC>100 개수 | 7 | 2 |
| Max CC | 453 | 348 |

- `handleOk`: CC 348 / 381 LOC
- `populateFromProps`: CC 212 / 286 LOC, 이번 구현 범위 제외
- 대상 파일: 2,825 physical LOC / 2,562 code LOC
- measured source clean: true
- metrics snapshot SHA-256:
  `02ab67076683a091b1c77f1c9c9889867af42f100dc7fc6ef6092485a59f5a93`
- git clean / measured source clean: true / true
- dirty path: 0

metrics output은 ignore된 `output/frontend-metrics/task2392/pre/`에 두고 commit하지 않는다.

## 4. characterization 결론

Stage 2/3 구현에서 다음 불변 조건을 사용한다.

1. pure builder는 DOM과 runtime service를 받지 않고 raw group snapshot과 기존 props만 받는다.
2. diff-only와 always-send key를 fixture에서 구분한다.
3. image scale이 common size를 덮는 현재 순서를 보존한다.
4. target resolver는 shape cell/body와 image header-footer/cell/body 5개 결과를 반환한다.
5. image에 두 marker가 있으면 header/footer를 우선한다.
6. 정상 unchanged UI는 type별 always-send key를 유지한다. control group이 없는 방어적 empty patch만
   setter/history/event를 만들지 않는다.
7. snapshot은 `objectProps`와 cursor 반환을, fallback은 `document-changed` 1회를 유지한다.
8. actual WASM setter 5회는 기존 dialog에 남긴다.

## 5. upstream 통합

Stage 1 도중 #2395가 merge되어 `upstream/devel`이 `eb9c7f1f`에서 `1cfb4273`으로 이동했다. 사용자에게
통합 초안을 제시하고 승인 후 계획 commit을 rebase했다.

- upstream 변화: `.github/workflows/ci.yml`과 #2393 orders/plan/report만 변경
- frontend 제품 모집단 diff: 0 files
- orders 충돌: upstream #2393 행·이월 3건을 유지하고 #2392 행을 추가
- metrics: 최신 head/upstream metadata로 재생성, aggregate 변화 0
- unit/build/E2E: 제품 tree가 byte-identical하므로 앞서 통과한 결과를 유지

#2370은 `insert.ts`의 회전/대칭 중복 emit 제거만 포함하고 dialog 생성 경로와 겹치지 않는다. #2394는
대상 dialog를 변경하지 않지만 merge되면 전역 metrics를 바꿀 수 있어 Stage 2 시작 전에 다시 확인한다.

Stage 2 시작 전에 사용자 승인으로 최신 `upstream/devel@af5902b6`에 다시 rebase했다. `1cfb4273..af5902b6`는
#2393 보고서 7줄만 변경해 frontend 제품 tree와 위 aggregate가 동일했다. 위 commit·snapshot hash는 이
최신 이력을 기준으로 보정한 값이다.

## 6. 추가 발견

`line` type은 textbox와 fill 탭을 만들지 않지만 non-OLE shape apply branch에서 해당 값을 읽는다.
fresh dialog에서는 없는 control을 0/`Top`/`none`으로 계산하고, context-menu singleton이 과거에 shape/group을
열었다면 detached control을 재사용할 수 있다. 현재 E2E는 매번 새 instance를 만드는 툴바 image body 경로라
이 차이를 검증하지 않는다.

이번 리팩터링은 이 결함 후보를 함께 수정하지 않는다. grouped snapshot에서 control presence를 유지하고,
후속 기능 이슈는 Stage 2 결과와 별도로 초안을 제시한다.

초기 구현 계획의 "unchanged image/shape는 empty patch" 가정도 현재 source와 달랐다. image/OLE은
`hasCaption`, shape/line/group은 shadow key를 always-send한다. 구현 계획의 fixture를 현재 동작에 맞게
보정했으며, #2392에서 이를 diff-only로 바꾸지 않는다.

## 7. 검증

| Gate | 결과 |
|------|------|
| `npm --prefix scripts/frontend-metrics ci` | PASS, 93 packages installed |
| metrics snapshot | PASS, aggregate가 계획 기준선과 일치 |
| `npm --prefix rhwp-studio test` | PASS, 362/362 |
| `npm --prefix rhwp-studio run build` | PASS |
| headless Chrome `e2e:undo` | PASS, case 1/2/2b/3/4/5 |
| `git diff --check` | PASS |

build에는 기존 CanvasKit `fs`/`path` browser externalize와 500 kB chunk 경고가 있었으나 실패는 없었다.
첫 headless 실행은 sandbox 안에서 Chrome DevTools WebSocket endpoint 생성 전에 timeout이 났고, 동일 명령을
sandbox 밖에서 재실행해 전체 PASS를 확인했다. 제품 실패로 분류하지 않는다.

picture 관련 browser evidence:

- `restrictInPage` apply 후 undo stack 정확히 1건 증가
- `performUndo` 후 원래 값 복원
- 실제 Escape로 object selection 해제 후 Ctrl+Z 복원
- command 실행 오류 0건
- `Through` object에서 확인만 눌러도 `textWrap` 유지

## 8. Stage 2 관문

- [x] 최신 upstream과 target overlap 없음 확인
- [x] 계획 문서 commit 고정
- [x] pre metrics와 characterization 완료
- [x] baseline gate PASS
- [x] 범위 밖 결함 후보 분리 기록
- [x] 작업지시자의 Stage 2 승인

Stage 2 결과는 `mydocs/working/task_m100_2392_stage2.md`에서 이어서 추적한다.

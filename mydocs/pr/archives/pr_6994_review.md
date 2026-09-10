# PR #6994 — #6950 제출 검증 기록

- PR: https://github.com/edwardkim/rhwp/pull/6994
- Issue: #6950
- 작성일: 2026-09-10
- 작성자·담당자: edwardkim (메인테이너 자체 PR)
- base: devel / source: task_m100_6950 / milestone: v1.0.0
- 상태: Open 제출. 로컬 검증 완료, **GitHub CI·정식 self-review 확정·병합 승인 대기**.
- 제출 후보: `e4f2b1a38`.
- 검증 제품: `a653d23ddfb08a65e88569a9f85b37389f41d500`.
- 최신 base 통합: `61eb331b0a92a7c19e46e36d273ac1e1e4af2746`,
  base `ec822767ae52926479e8fe58bc7003b4e6c82cba`.
- 결과: [최종 보고서](../../report/task_m100_6950_report.md),
  [Stage 3 §31](../../working/task_m100_6950_stage3.md#31-최신-devel-통합-및-제출-전-검증).

## 변경 계약과 보호 범위

텍스트 끝의 자리차지 표가 잔여 너비에 들어가지 않으면 다음 줄의 확정 원점과 점유 범위를
fit·pagination·일반/조각 출력에 전달한다. 다음 줄 점유와 떠 있는 표의 배제 영역을 구분해
명시적 개행·문단 종료 진행량·후속 빈 문단을 보존한다. 표 속성 조회는 raw 부재에 영향받지
않도록 IR 기하를 사용하며 조회 과정의 문서 불변성을 검사했다.

최신 base의 고정 글상자 배제·래퍼 표 여백 처리를 보존했다. #6972 신규 fixture에는
추가 필드의 빈 초기값만 보완했다. 후속 #6991 병합은 CI·문서이며 Rust·Studio·샘플·렌더
검사 입력은 동일하다. 해당 CI delta의 Python56/56·Node291/291 검사도 통과했다.

## 완료한 로컬 검증

| 게이트 | 결과 |
| --- | --- |
| 별도 review worktree prepare·fmt·native/WASM/workspace Clippy·workspace build | PASS |
| manifest·source-side unit tier | PASS |
| 전체 nextest | 9,444 passed / 0 failed / 46 skipped |
| Native Skia lib | 4,112 passed / 0 failed / 13 ignored |
| Native Skia placeholder / direct PDF | 2/2 / 4/4 PASS |
| 새 Docker dev WASM·Native Skia CLI | PASS |
| 렌더 계약·Canvas·Direct PDF·CanvasKit readiness | PASS / 3/3 / 3/3 / 8/8 |
| 제출 source·test·샘플·PDF와 검증 사본 동일성·diff check | PASS |

새 integration test는 `tests/cases/` 원본만 제출했다. generated suite·manifest·Cargo 파생
target·로그·중간 PNG·WASM 산출물은 제외했다. 메인테이너의7700 서버는 재시작하지 않았다.

Canvas 최대0.01761%(허용0.05%), Direct PDF 최대1.15895%(허용2%).
보고 전용 Browser Canvas/PDF4warn과 readiness 초기 폰트·context·커서 경고는 남았다.
nextest 버전·CI 설정 키 경고도 Stage 3에 기록했다. 최종 gate 통과를 무경고와 혼동하지 않는다.

## 시각 증거와 판정 경계

메인테이너가 원본 문단 끝 표·후속 문단, synam30쪽, #6025 1쪽, #1510의 두 형식을
SVG·WASM 및 한컴 편집기에서 확인했다. 최신 조판 통합 뒤 원본3쪽·#1510 HWP1쪽·HWPX2쪽의
SVG6개는 같은 `--font-style` 옵션의 승인본과 바이트 동일했다.
HWP1쪽/HWPX2쪽은 한컴에서도 서로 다르며 기존 쪽수 래칫을 유지했다.

![#1510 HWP 실제 1쪽의 rhwp·한컴 PDF·overlay](../assets/pr_6994_20260910/issue1510-hwp-physical-p1.png)

이 대표 PNG를 직접 열어 표 위 본문과 표 아래 재개 흐름을 확인했다. 패널 제목의 `p1510`은
생성기가 파일명 번호를 읽은 표기이며 **실제 문서1쪽**이다. pixel_match96.316%,
ink_match8.435%는 보조 지표로, 글꼴·텍스트의 픽셀 차이를 포함한다. 전체 fidelity 합격 점수나
한컴과의 완전 일치 주장이 아니다. 원본의 작은 용지 높이 합성 실험은 메인테이너 결정으로
이번 범위에서 제외했으며 모든 표 분할 정책을 해결했다고 주장하지 않는다.

## 다음 조건과 merge 후 comment 계획

- 최신 PR head의 GitHub required checks를 확인한다. 로컬 성공으로 원격 CI를 대체하지 않는다.
- CI 성공 뒤 메인테이너 승인에 따라 정식 self-review를 확정한다.
- 병합 방식·병합·이슈 close는 후속 승인 전 실행하지 않는다.
- 병합 후 승인된 comment에는 #6950의 해결 규칙과 #1510 HWP1쪽/HWPX2쪽 보호 결과,
  위 시각 지표의 한계, 실제 merge SHA의 `mydocs/pr/assets/pr_6994_20260910/issue1510-hwp-physical-p1.png`
  raw 이미지 링크를 넣는다. 현재는 계획만 기록하며 comment를 게시하지 않는다.

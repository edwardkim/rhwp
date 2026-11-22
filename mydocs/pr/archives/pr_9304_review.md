# PR #9304 그림 개체의 CTRL_DATA 저장 보존

## 최종 판정

**승인.** 메인테이너 `edwardkim`이 2026년 11월 22일 셀프 리뷰했다. 네 원본을 편집하고 두 번 저장해도 각 그림 헤더와 구성 요소가 소유한 레코드의 순서와 전체 바이트가 보존됐다. 수정 커밋은 `f2039b89ac4f`, GitHub에서 검증한 제출 커밋은 `99f1d153ff69`다. 이 기록을 추가한 커밋의 필수 검사와 최신 `devel` 병합 결과는 머지 직전에 확인한다. [PR #9304](https://github.com/edwardkim/rhwp/pull/9304) · [이슈 #8225](https://github.com/edwardkim/rhwp/issues/8225)

## 저장 경로 검토

[기여 PR #8233](https://github.com/edwardkim/rhwp/pull/8233)과 [통합 PR #8395](https://github.com/edwardkim/rhwp/pull/8395)는 그룹 자식의 레코드를 복원했다. 네 원본의 편집본에는 그룹이나 중첩 그림 자체의 헤더에서 데이터가 사라지는 문제가 남아 있었다. 이번 파서와 저장기는 HWP5 레코드의 깊이로 소유자를 구분한다. 헤더와 구성 요소에 같은 값이 있더라도 각각 원래 위치에 쓰며, 값이 다를 때도 두 값을 유지한다. 첫 글자 장식을 편집하면 해당 ParameterSet이 갱신된다. 캡션과 그룹 자식의 독립적인 데이터, 변환기가 생성한 그림 하이퍼링크도 회귀 검사에서 확인했다. [구현 계획](../../plans/task_m100_8225.md) · [회귀 검사](../../../tests/cases/issue_8225_group_child_ctrl_data.rs)

## 로컬 검증

- `sungeo.hwp`의 CTRL_DATA는 최초 편집 저장에서 60개가 48개로 줄었으나 수정 후 두 번 모두 60개였다. 나머지 세 원본도 각각 2개가 1개로 줄던 현상이 해소되어 두 번 모두 2개였다. 독립적으로 구역의 압축을 풀어 소유 경로·순서·원문 바이트를 검증했다.
- 공개 HWP5 원본 33개 경로에서 그림 레코드 119개를 Native CLI와 새로 빌드한 최적화 WASM으로 보존했다. Chromium에서 네 원본과 네 대조군을 각각 두 번 편집·저장한 결과도 일치했다. 첫 글자 장식을 변경한 HWP·HWPX 교차 저장본 8개에서 장식과 삽입 문자를 다시 읽었다.
- 최소 글꼴 환경의 Rust 전체 회귀는 실행 13,948개가 모두 통과했고 nextest가 별도로 집계한 건너뛴 검사는 45개였다. 집중 검사 10개, Native·WASM32·workspace Clippy, workspace 빌드, 포맷과 suite manifest가 통과했다. 이번 변경은 불투명한 저장 레코드와 편집 가능한 장식 속성을 다루므로 원본 바이트와 공개 API 저장·재열기를 판정 근거로 사용했다. [실행 조건과 원본별 결과](../../report/task_m100_8225_report.md)

## 최신 devel과 GitHub 검사

리베이스로 반영된 [#9297](https://github.com/edwardkim/rhwp/pull/9297)의 CI 요약·정책 검사 42개가 기준과 후보에서 모두 통과했다. 제출 전에는 리뷰어가 미지정인 같은 계정의 Draft #9299·#9301과 각각 병합을 시뮬레이션하고 두 PR의 CI 성공을 확인했다.

CI 대기 중 [#9302](https://github.com/edwardkim/rhwp/pull/9302)가 병합됐다. 새 `devel` `99a81bd7f0`과 제출 커밋의 병합 트리는 `8182baae7c`였다. #9302가 바꾼 51개 경로는 새 기준과 병합 결과에서 동일했다. 두 트리에서 에디터 암호 저장과 전송 계약 17개를 각각 실행해 모두 통과했다.

제출 커밋의 Build & Test, 회귀 A–E, Native Skia, Clippy, Render Diff, Adapter, Proptest와 최종 CI 정책 감사가 성공했다. CodeQL의 Rust 분석 결과는 0건이었다. GitHub 통합 보안 표시는 경로 판정에서 선택하지 않은 Actions·JavaScript·Python 세 구성을 사유로 `neutral`을 반환했다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/44942653132/job/136972182005) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44942652971) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/44942653089) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/44942653030) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44942653066) · [통합 보안 표시](https://github.com/edwardkim/rhwp/runs/136969806857) · [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/44945651821)

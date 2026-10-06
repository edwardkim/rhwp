# PR #6894 self-review — 소유 구조 기반 사각형·글상자 식별

- 관련 이슈: [#6856](https://github.com/edwardkim/rhwp/issues/6856).
- 작성자/검토 역할: `edwardkim` 본인 PR의 메인테이너 self-review. 외부 reviewer를 지정하지 않는다.
- base `devel`, head `task_m100_6856_baseline`, code candidate `98a2d403d43b784f00e22adcf4f011c777a91dcf`.
- 접수 규모: 27파일, +1,826/-51. 대형 PR 경로로 source 검토·통합 검증·사람 판정을 분리했다.
- 적용 절차: collaborator_self_merge + intake_and_review + local_validation + visual_fixture_evidence +
  rework_and_exceptions + review_only_fast_pass + post_merge.
- 2026-09-08 메인테이너가 병합과 이슈 close를 승인했다. CI를 우회하는 승인이 아니다.

## 검토 범위와 결과

1. `RectangleShape::control_kind()`는 기존 `Option<TextBox>`만 질의한다. 사각형 기하와 소유 문단
   영역을 별도 축으로 유지하며 글자 수·배경·선 스타일로 재분류하지 않는다.
2. `RenderNode`의 원본 컨트롤 부호를 SVG·HTML·Canvas·paint가 공동 소비한다. 내부 TextBox 노드의
   중복 부호만 숨기며 실제 내부 그림·표의 부호를 일괄 제거하지 않는다. 조판부호 OFF는 유지된다.
3. HWPX 자기닫힘 `<p/>`는 속성·빈 문단을 보존한다. 뒤 형제를 소비하지 않는 #5797 보호도 유지했다.
   기존 시험의 선두 개행 기대값 정정은 빈 문단을 버리지 않는 승인 계약과 일치한다.
4. HWP 검사기는 이미 해석한 레코드에 깊이 스택을 적용해 사각형이 직접 소유한 목록만 검사한다.
   선언 개수 기반 할당·재귀 전수 재파싱이 없으며 캡션·그룹 형제·중첩 소유권 시험을 확인했다.
5. 한컴에서 손상 판정받은 HWP 소유 0문단 목록을 구조 오류로 거부한다. 일반/lenient HWP와
   HWPX section/master-page 경로의 해당 오류가 빈 section 복구로 가려지지 않는다.
   다른 오류 정책과 HWPX·표·캡션의 0 개수 필드 전체를 변경하지 않는다.
6. 선 억제 A 정책, geometry, Studio source, workflow, Cargo 입력, 원본 sample·PDF·golden·baseline은
   이번 PR에서 바꾸지 않았다. 신규 integration 원본만 제출하고 generated suite·manifest는 제외했다.

검토 범위에서 새 병합 차단 결함을 발견하지 않았다. 선언 개수보다 많은 문단을 포함한 입력 등
모든 손상 패턴의 완전한 검출이나 모든 한컴 문서의 시각 동일성을 주장하지 않는다.

## 로컬 검증과 시각 근거

명령·로그·SHA의 정본은 [Stage 6](../../working/task_m100_6856_restart_stage6.md)와
[결과보고서](../../report/task_m100_6856_report.md)다.

- 독립 후보: 전체 nextest 9,248 통과·46 skip, fmt·세 Clippy·workspace build·manifest 통과.
- 최신 base `a7de17ff8` 통합 tree `e68d40647229f6a57a3471a984be20a0461afcdd`:
  전체 nextest 9,266 통과·0 실패·46 skip, Native Skia lib 4,112·picture 2·PDF 4개 통과,
  fmt·세 Clippy·workspace build·manifest·배정 규칙 21개 통과.
- Docker 통합 WASM 최적화 빌드 6분 48초 통과. self-review에서도 해당 WASM으로 A4 HWP/HWPX
  1쪽·사각형 1개·글상자 2개·표시 OFF와 실제 손상 파일 거부 스모크를 재실행해 통과했다.
- 통합 WASM SHA256 `3e4b1e60112933cc3c0f70b18981301ef2e0c2927bd0d9a15c4a687bde1efcdc`.
- 새 source 수정이 없고 검증한 base가 유지되어 광범위 Cargo 재검증은 반복하지 않았다.
- A4 원본 HWP SHA256 `0483bda7e884143faf9cc19003897b8421623bd3e4c3446bdfbc6f90388b766c`,
  HWPX `809f31be6b7cd6a87fe38c5d0d2b8d194c4b842bef05557692ecaae6b69599ac`.
- 메인테이너가 A4 조판부호 식별을 통과 판정했다. 통합본 HWP legacy/HWPX layer SVG가 각각
  선행 판정·검증본과 바이트 동일했고 self-review에서 대표 PNG를 직접 열어 한글 부호와 실선을 확인했다.
- 대표 PNG: [A4 식별](../assets/issue6856_a4_control_codes.png), SHA256
  `c92adc23dacf6bb36b9d1f48d44afea369b2522a0f4abde1009ea955554d868a`.
- 0문단 손상 원본 SHA256 `075b6fe46ddf5926505e98e49d92c14692fb3130e08db182f3b4de82a1b87e76`.
- PDF pixel sweep, flagged 후보 수, pixel_match, visual_accuracy_proxy_percent는 미계측이다.
  조판부호 식별의 동등한 직접 판정이며 문서 전체 fidelity 통과로 확대하지 않는다.

## GitHub CI와 최종 판정

- candidate `98a2d403d`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/34215242255) 및
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34215242211) workflow 성공을 확인했다.
  Build & Test, 모든 실행된 lint/native/archive job과 언어별 Analyze는 SUCCESS,
  종합 CodeQL check는 NEUTRAL이다. 이를 alert 0건으로 해석하지 않는다.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34215242008),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34215242260),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34215242252), CI Impact Policy도 성공했다.
  확인 시점 MERGEABLE/CLEAN이고 pending/failed check가 없다. 최신 head는 merge 직전 재조회한다.
- 판정: **승인** — 승인된 구조 식별·제한적 손상 거부 범위의 코드·로컬·사람 판정 근거를 충족했다.
- 병합 전 조건: code candidate 전체 CI 성공 → review-only 기록 push → 최신 head CI 성공 및
  mergeability 재조회. 기본 merge commit 방식, `--admin` 우회·force push·자동 issue close 문구 없음.
- #6856은 초기 A 선 억제 수정까지 완료한 것으로 닫지 않는다. 승인된 재착수 범위를 본문에 명시하고
  원래 범위는 역사로 보존한 뒤, 그 범위의 완료를 근거로 메인테이너 승인에 따라 종료한다.
- 초기 범위 밖 A 정책의 정당성, 포맷 간 전체 시각 동일성, 전후 성능은 미확정/미계측으로 남긴다.

## Merge 후 contributor PR comment 계획

- 문서 비교 경계의 정본: [Visual Sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment).
- 실제 판정은 위 A4 1쪽의 사각형 1개·글상자 2개와 손상 원본 거부다. PDF 자동 수치를 만들지 않는다.
- merge SHA와 CI run, 위 로컬 결과, 메인테이너 시각 판정 및 A 정책 범위 밖을 issue 종료 기록에 남긴다.
- PNG가 devel에 포함된 뒤
  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue6856_a4_control_codes.png`를 사용한다.
- UTF-8 `--body-file` 게시 후 API의 본문을 재조회한다. merge 전 완료 사실을 게시하지 않는다.

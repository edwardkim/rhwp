# PR #9099 리뷰 — frontend Canvas와 Native PDF CI 분리

## 최종 판정

**승인 — CI 분류·실행 계약과 실제 최적화 WASM 브라우저 검증을 완료했다.**
최종 source head `7053e375deec4703f0095bf278df9a800aefaa89`의 Full CI와 정책 감사가
완료됐으며, 2026-11-16 00:10:10 +09:00에 일반 squash로 병합했다.
merge SHA는 `5ab44e8315d3f85057dd09f700a0c1a11386fcbb`다. 관리자 우회를 사용하지 않았다.

## 접수와 범위

| 항목 | 확인 결과 |
| --- | --- |
| PR·작성자·base | [#9099](https://github.com/edwardkim/rhwp/pull/9099) / `jangster77` / devel |
| 이슈 | [#9097](https://github.com/edwardkim/rhwp/issues/9097), `github-actions[bot]`가 자동 CLOSED 처리 |
| 역할 | collaborator self-review, reviewer 미지정, Open PR |
| 최종 base | `c5022cbe8db2efe24a27150539af6525a15b082f` |
| 실제 CI checkout | `23fa8d2660a4a2f8d69b0a2bca9bd90fc0ad44ea` |
| CI·병합 tree | 양쪽 모두 `856701d0c2a7d22663672c6bbc098683397eb89e` |
| 후속 기록 | 사용자 지정 옵션 2. 원 PR에는 review·오늘할일 trailing commit을 추가하지 않음 |

#9077의 frontend 수정에서도 Native/PDF를 포함한 전체 Render Diff와 별도 frontend WASM이
실행된 문제를 다뤘다. #3790의 trusted-base 분류와 #7473/#7474의 standard release·wasm-opt·
실제 WASM 응답 hash 계약을 유지했다. [구현과 선행 분석](../../working/task_m100_9097_stage1.md),
[최신 기준 검증](../../working/task_m100_9097_stage2.md)을 대조했다.

알려진 Studio 화면 소스와 #9077 직접 검사는 `frontend_canvas_required=true`로 package와
브라우저 검사를 선택한다. Rust renderer·PDF·글꼴·공개 자원·빌드 설정·미분류 입력과 혼합
변경은 기존 전체 축을 유지한다. command·unit의 기존 선택도 보존한다.

실행기는 동일한 최적화 package를 검사한 뒤 #9077 setter 대조, legacy/layer 비교와 CanvasKit
readiness를 실행한다. Native CLI·PDF 명령은 없다. 정책 v9는 Canvas 빌드·실행 step 각각의
정확한 completed/success 증거를 요구하며, 누락·중복·skip·실패를 green package job으로
대체하지 못한다. 정책·classifier·세 소비 workflow의 version과 `fc` 필드를 함께 확인했다.

## 검증 결과

| 구분 | 실제 결과 |
| --- | --- |
| Windows 로컬 계약 | Node 343 PASS·0 FAIL·0 SKIP, Python workflow 250 PASS |
| 로컬 구문·문서 | workflow 3개 actionlint, 링크·diff·병합 시뮬레이션 PASS |
| 로컬 WASM·browser | standard release + 실제 wasm-opt, 선언·embed 3 PASS, #9077 12 PASS·legacy/layer 3 PASS·readiness 8 PASS |
| GitHub 최신 head | 39개 완료 또는 정상 skip, pending 0·failure 0, merge 직전 CLEAN |
| CI frontend WASM | dev 빌드 skipped, 최적화 빌드 success 12분 13초 |
| CI 새 Canvas gate | success 84초, 실제 WASM 응답 18개 hash 일치 |
| CI 브라우저 결과 | 고정 revision `1660786`, 실제 Chrome 152.0.7946.0, legacy/layer 3 PASS·readiness 8 PASS |
| CI 기존 전체 검사 | Rust archive·lint·Native Skia, CodeQL, 전체 Render Diff의 WASM·Native CLI·PDF·readiness, Adapter·Proptest success |

실제 API·job step·artifact·hash와 정책 결과는 [CI 원장](../assets/issue_9097/code-ci-7053e375de.json)에
보존했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/43725569569),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43725569479),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43725569582),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/43725569555),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/43725570034),
[최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/43728173864)가 모두 성공했다.

첫 정책 변경 PR은 trusted base의 v8로 **Full 실행**했다. 새 Canvas gate는 실제로 실행됐지만,
이 전체 실행의 시간을 향후 frontend-only 선택 경로의 비용 개선 실측으로 해석하지 않는다.
분류 축의 변경 전후와 정상·음성 경계는 Node 동작 검사와 workflow 계약으로 확인했다.

## 공통 검토

- **조판 원칙: 비해당.** 제품 renderer·Studio 소스, baseline·허용치·입력·글꼴은 바꾸지 않았다.
  새 검사는 artifact 계약이며 시각 좌표나 golden을 새로 고정하지 않는다. CI 연결을 위한 기존
  browser 검사 실행을 한컴 Print와의 일치율·전쪽 TSV 측정으로 주장하지 않는다.
- **검증 입력 commit: 충족.** 기존 HWP/HWPX 11개의 byte/hash와 검증 commit 일치를
  [입력 원장](../../working/assets/issue_9097/local-validation.json)에 보존했다. Vite의 실제
  `samples/` 우선 경로와 browser document digest를 대조했다. PDF를 새 기준으로 사용하지 않았다.
- **반영 PR 보존: 충족.** #9096 반영 후 새 엔진을 다시 빌드해 같은 browser 검사를 실행했다.
  새 devel과 후보의 제품·입력 파일은 동일하고, 이후 base 전진은 #9096 종료 문서뿐이었다.
  코드·실행 입력 차이가 없는 영역의 parser·저장·Print를 다시 수정하지 않았다.
- **폰트 변경: 없음.** Windows browser 검증의 임시 `public/fonts` 연결은 원래 파일을 동일
  hash로 복원했으며 임시 junction도 제거했다. public 생성 선언 파일도 원복했다.

[duration 자동 갱신](https://github.com/edwardkim/rhwp/actions/runs/43728359687)은 성공했고
44개 target의 실측을 metrics commit `0c4712cfb085f17eda3635f0bb40ff2fbc2e45af`로 게시했다.
병합 SHA의 push run은 duration 갱신과 issue close뿐이며 검증 CI를 다시 실행하지 않았다.

원 코드의 검증과 병합을 완료한 뒤 이 review·CI 원장·2026-11-16 오늘할일을 별도 기록 PR로
반영한다. 해당 기록 PR은 실제 review-only preflight·required check·mergeability를 확인한다.

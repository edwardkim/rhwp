# PR #6984 작성자 self-review

- 검토일: 2026-09-10. GitHub 상태는 최초 제출 head 조회 시점의 참고값이다.
- base route: `collaborator_self_merge.md`.
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `rework_and_exceptions.md`(1,000줄 초과), `review_only_fast_pass.md`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서,
  `dev_environment_guide.md`, `edit_command_review_checklist.md`, 문서·Git 절차.
- 작성자 self PR로 reviewer는 지정하지 않았다. GitHub approve event는 아니다.

## 2026-09-10 후속 UI·표시 문자열 갱신 검증

이하 최초 제출 기록 이후의 제품·테스트 검증 후보는 `fd06747f6`이다.
사용자 요청에 따라 설명 문자열은 이번 PR에서 제외하고, 표시 문자열 편집·웹 탭·미리보기·
방문 표시·컨텍스트 메뉴·주소 전용 호버를 포함한다.
전체 Rust nextest 9,405개, Studio Node 1,502개가 통과했다.
필수 fmt/3종 Clippy/workspace build/manifest, native-skia lib(주 lib 3,930개),
placeholder 2개·직접 PDF 4개, TypeScript/Vite build와 unit tier 정책 검사도 통과했다.
검증은 별도 review worktree에서 수행했고 generated suite는 source PR에 포함하지 않았다.
동일 HWPX를 Studio·macOS 한글 Viewer에서 열고 그 문서의 PDF를 미리보기에서 확인했다.
Viewer는 사용자 승인 후 한 번 허용하여 실제 한컴 홈페이지 이동까지 확인했다.
[단계 10 증거 및 재현](../../working/task_m100_6963_stage10.md)을 따른다.
최신 devel과 merge-tree 충돌 없음. 이 기록은 GitHub approve·merge 승인이 아니다.

## 최초 제출 대상과 범위

| 항목 | 확인값 |
| --- | --- |
| PR / Issue | [#6984](https://github.com/edwardkim/rhwp/pull/6984) / [#6963](https://github.com/edwardkim/rhwp/issues/6963) |
| 작성자·담당 | postmelee / postmelee |
| milestone / labels | v1.0.0 / enhancement, rhwp-studio, serialization, rendering |
| base / source | devel / codex/issue-6963-hyperlinks, upstream 같은 저장소 |
| 로컬 검증 제품·테스트 후보 | `b05cadb0e` |
| 최초 PR 제출 head | `4013da8a2e87f08543f6eb8f062396f8b94a8cca` |
| 조회한 base | `37bd46a72f9fd9ffd709e35244df79c00e789780` |
| 최초 제출 규모 | 47 files, +3,615 / −49 |
| 최초 제출 상태 | OPEN, non-Draft, MERGEABLE, BLOCKED, GitHub 검사 진행 중 |

Studio의 HTTP/HTTPS 텍스트 링크 삽입·수정·해제, undo/redo와 HWP/HWPX 저장 왕복을
연결했다. PDF에는 실제 텍스트 run의 경계·clip으로 계산한 줄·페이지별 Link/URI를 넣는다.
브라우저 인쇄 SVG, CLI SVG PDF, 직접 Skia PDF를 각각 검증했다.

## 코드 및 편집 계약 검토

1. URL·Command codec은 공유하고 Unicode scalar offset으로 필드 범위를 다룬다.
   본문·중첩 셀·글상자 경로를 보존하며 미지원 위치를 명시적으로 제한한다.
2. `executeOperation()` snapshot 경로로 글자·필드 변경을 함께 처리한다. 실패는 원자적으로
   복원하며 취소·동일 주소는 history를 추가하지 않는다. dirty·caret·focus 및 undo/redo,
   모달을 연 뒤 문서 세대·읽기 전용 상태가 바뀌는 경우를 검사했다.
3. 저장 전에 원본 section stream과 cache를 무효화한다. 전체 회귀에서 새 API 3개의
   passthrough guard 분류 누락을 발견해 실제 공통 helper 위임으로 등록했다. 검사 해제나
   Pending 상한 변경 없이 가드 5개 및 전체 9,401개 회귀를 다시 통과했다.
4. 링크 영역은 문자 수 비례 추정 대신 렌더 경계와 clip을 사용한다. 페이지 재배열·변환 실패
   격리·px/pt 및 Y축 변환을 검사했다. 회전·세로쓰기 등 부정확한 영역은 오류로 반환한다.
5. 실제 저장 메뉴의 serializer 바이트를 다시 열고 실제 인쇄 iframe을 Chromium PDF로
   출력했다. OS picker의 write와 native print 호출만 캡처한다. 실제 PDF 뷰어의 글자 위치를
   클릭하고 한글 query·fragment 이동 주소를 확인했다. 목적지 외부 요청은 테스트 응답으로 대체했다.
6. 새 integration 원본은 `tests/cases/`에 있고 generated suite·manifest는 review worktree에서만
   준비했다. source-side unit test, sample, npm/editor API, CI workflow 변경은 없다.

새 코드 보정이 필요한 추가 문제는 발견하지 못했다. 기존 [구현 계획](../../plans/task_m100_6963_impl.md)과
6단계 기록이 구현·검증·제출 순서를 담고 있어 별도 `review_impl`은 만들지 않는다.
1,000줄 초과 PR이므로 이번 제출을 즉시 admin merge 근거로 사용하지 않는다.

## 로컬 검증과 GitHub 상태

[최종 통합 검증](../../working/task_m100_6963_stage6.md)에 명령·종료 결과·바이너리 SHA-256을 기록했다.
전용 `/tmp/rhwp-issue-6963-review`의 `b05cadb0e`에서 같은 `target/pr-review`로 순차 실행했다.
최초 제출 head까지의 후속 차이는 문서·중간 증적 정리뿐이다.

| 검증 | 완료 결과 |
| --- | --- |
| Rust 필수 lint 전체 | suite prepare, fmt, native/WASM32/workspace all-target Clippy, workspace build, manifest check 통과 |
| 전체 release-test nextest | 9,401 pass / 46 skip / 0 fail |
| Native Skia | library 4,112 pass / 13 ignored, placeholder 2/2, direct PDF 4/4, hyperlink PDF 9/9 |
| fresh WASM + Studio | native wrapper `--no-opt`, TypeScript, production bundle, Node 1,501 pass / 2 skip, 편집 시나리오 10종 통과 |
| 새 suite 정책 | 23/23 통과 |
| 실제 Chrome 저장·재열기·PDF | 5종 34페이지의 81개 URI 주석 및 뷰어 클릭 통과 |
| merge simulation | 최초 제출 head + 위 base의 무충돌 tree `69e14830c4ec157a698ca862f36203b73c263e6a` |

Docker 데몬 연결이 불가해 매뉴얼이 허용한 native WASM 진단 경로를 사용했다. 최적화 배포 빌드나
최신 base의 merge tree 전체를 로컬 컴파일한 증거로 확대하지 않는다. 원시 로그는
`/tmp/issue6963-gates/final/`, 브라우저 산출은 review worktree의 `output/pdf/issue6963-stage5/`다.

최초 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/34445352565),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34445352540),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34445352340),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/34445352578),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/34445352589)는 최초 조회 시 진행·대기 중이었다.
녹색 code candidate의 GitHub 결과를 아직 확보하지 못했으므로 이번 기록 commit의 fast-pass를
보장하지 않는다. 후속 head에서 Full CI가 실행되면 그 결과를 확인한다.

## 시각·원본 증적과 한계

[PDF/SVG visual sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)의
직접 이미지 확인 원칙에 따라, 이번 기능에는 링크 주석 파싱·DOM 영역 대조·동일 잉크 픽셀 비교를
동등한 범위 검증으로 사용했다. 일반 visual_sweep.py 전수 후보 분류는 실행하지 않았다.
따라서 자동 후보 수와 `visual_accuracy_proxy_percent`는 미측정이다.

- 원본 HWP/HWPX와 기존 한컴 PDF 경로·SHA-256·Creator/Producer는
  [단계 1](../../working/task_m100_6963_stage1.md)에 보존했다. 기존 저장소 PDF를 재사용했고 새 MCP PDF를 만들지 않았다.
- CLI 대표 Textmail p1·LH p8은 [단계 3](../../working/task_m100_6963_stage3.md)의
  레이어→주석 차이 0.001pt 미만, 주석 추가 전후 변경 픽셀 0이다.
- Chrome 5종 PDF의 34페이지·81개 주석을 독립 파서로 검사했다. Textmail p1·LH p8·긴 링크 p2와
  실제 뷰어 화면은 직접 이미지로 확인했다. DOM→주석 최대 오차는 0.381pt 미만이다.
- 최종 후보에서도 새 문서 p1·Textmail p1의 링크 유무 PDF를 Poppler 144dpi와 Pillow로
  다시 비교해 변경 픽셀 0(pixel match 100%)을 확인했다. 한컴 원본과의 100% 일치 수치가 아니다.
- 아래 비교 패널 2개를 이번 self-review에서 다시 열었다. 영문 도구 라벨과 빨간 링크 영역은
  판독 가능하며 본문 잉크와 구분된다. Textmail X 약 6.5pt, LH p8 Y 약 29pt의 기존 배치 차이와
  기존 텍스트 clipping이 남아 있다. 한컴 전체 조판 일치나 해당 잔여 해결로 판정하지 않는다.

원래 비교 파일은 `mydocs/working/assets/issue6963/stage3/`, 생성 PDF 임시 경로는
`output/pdf/issue6963-stage3/`이며 PR 고정 대표 파일은 다음 두 개다.

![Textmail 링크 범위](../assets/pr_6984_textmail_annotation.png)

![LH 링크 범위와 기존 배치 차이](../assets/pr_6984_lh_annotation.png)

신규 mailto·파일·내부 책갈피 편집 UI, 편집 화면의 URL 이동, 다른 브라우저·OS 인쇄창은
검증 범위 밖이다. bare `createEmpty()` 최소 IR의 저장 제약은 실제 Studio의
`createBlankDocument()` 경로와 구분해 API 가이드에 기록했다.

## 최종 판정

**승인** — 명시한 HTTP/HTTPS 편집→저장 왕복→PDF URI·클릭 영역 보존의 로컬 검증 범위다.
이는 GitHub approve나 작업지시자의 시각·merge 승인과 다르다. 최신 후속 head의 required checks,
mergeable 상태와 작업지시자의 병합 판단이 남았다. 원격 push·PR 생성 승인은 받았고,
merge·issue close·GitHub comment는 실행하지 않았다.

## Merge 후 contributor PR comment 계획

self PR에도 증적 계획을 남긴다. merge 및 댓글 게시 승인을 받은 뒤에만 실제 결과로 갱신해 게시한다.

- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 인용한다.
- 실제 확인한 Textmail p1·LH p8·긴 링크 p2 및 PDF 뷰어, 34페이지·81주석 파싱,
  최대 0.381pt, 별도 2페이지 픽셀 일치 100%를 각각의 범위와 함께 적는다.
- 자동 visual sweep 후보 수·proxy 지표는 미측정으로 남기고, 한컴 배치·clipping 잔여를 명시한다.
  에이전트가 이미지를 직접 확인한 결과와 작업지시자의 승인 여부를 구분한다.
- 최종 대표 이미지 2개를 다음 merge SHA 고정 형식으로 표시한다.

```text
https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6984_textmail_annotation.png
https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6984_lh_annotation.png
```

- 실제 merge SHA에 asset이 존재하는지 확인하고 UTF-8 without BOM 본문 파일을 `--body-file`로
  게시한다. API로 한글·이미지 링크·BOM·문자 치환 여부를 재조회한다. 이 문서는 게시 예약이나
  댓글·issue close의 선승인이 아니다.

## 제출 후 오늘할일 충돌 해소

검토 문서 추가 뒤 `mydocs/orders/20260910.md`의 add/add 충돌을 확인했다.
base `37bd46a72`에는 #6979·#6962 기록이 이미 있고 이 작업의 이전 분기에는 없었다.
같은 base를 병합해 기존 두 작업의 기록 전체와 새 #6963 항목을 함께 보존했다.
수동 충돌 해소는 이 Markdown 한 파일뿐이며 제품·테스트 충돌은 없었다.
로컬 전체 검증은 앞서 명시한 `b05cadb0e` 결과이고, base 병합 후 통합 결과는 최신 PR CI가 검증한다.
이후 head의 required checks와 mergeable 상태를 다시 확인하며 아직 원격 CI 성공·병합을 주장하지 않는다.

## 링크 끝 이어 쓰기 후속 수정

사용자 재현에 따라 링크 끝 삽입 시 범위를 늘리지 않고 링크 밖의 원래 모양을 유지하도록
수정했다. 링크 내부 삽입은 계속 범위를 확장한다. native 링크 회귀 14개, 실제 WASM 11그룹,
Chrome UI 실제 이어 쓰기·고치기·지우기·undo/redo, PDF 저장 왕복과 주석 검증을 통과했다.
전체 Rust 9,407개, 필수 fmt/3종 Clippy/workspace build/manifest,
native-skia 주 lib 3,930개·보조 lib 182개, placeholder 2개·직접 PDF 4개를 통과했다.
[단계 11의 재현 화면과 검증 범위](../../working/task_m100_6963_stage11.md)를 따른다.

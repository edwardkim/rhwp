---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7108 — 일반 face PDF 합성 굵게 self-review

**최종 판정: 승인.** 기본 SVG→PDF에서 Bold face 없는 글꼴의 굵기 소실과 glyph 중복 방출을
해결하는 범위다. 최종 trailing head의 CI 및 mergeability 확인은 병합 직전 별도 게이트다.
작업지시자의 CI 모니터링 후 후속 처리 지시에 따라 문서 trailing commit·병합·종료 절차를 진행한다.

## 1. 대상과 코드 검토

| 항목 | 확인값 |
| --- | --- |
| PR / issue | [#7108](https://github.com/edwardkim/rhwp/pull/7108) / [#6936](https://github.com/edwardkim/rhwp/issues/6936) |
| 작성자 / 경로 | jangster77 / collaborator self-review, reviewer 미지정 |
| source branch / base | `fix/6936-pdf-synthetic-bold-20260913` / `devel` |
| 로컬 코드 검증 HEAD | `3ba6d6d3e`; 이후 code/test/fixture 변경 없음 |
| GitHub Full CI 후보 HEAD | `ac1e8ec9e878f4b4a6a138f486e1dd301f219b01` |
| 기준 devel | `70bf40af2a2818e72bd58b4fa66e2d4c06de2b51` |
| 기본 / 보조 절차 | collaborator_self_merge / intake_and_review, local_validation, visual_fixture_evidence, rework_and_exceptions, review_only_fast_pass, post_merge |

- [PDF 준비 단계](../../../src/renderer/pdf_synthetic_bold.rs)는 usvg의 실제 선택 glyph face weight와
  균일한 paint를 검사한다. 일반 face의 굵게에만 한컴 실측 `font-size × 0.02` 획을 추가한다.
- [PDF 호출 경로](../../../src/renderer/pdf.rs)는 기존 기울임 처리 앞에 합성 굵게를 적용한다.
  브라우저 SVG에 PDF 전용 stroke를 섞지 않는다.
- [svg2pdf text](../../../vendor/svg2pdf/src/render/text.rs)와
  [paint](../../../vendor/svg2pdf/src/render/path.rs)는 단색 fill+stroke를 `Tr 2` 한 번으로 방출한다.
  실제 Bold·normal, 서로 다른 역순 paint 및 반투명 역순 paint의 기존 동작을 보존한다.
- [새굴림 change set](../../../assets/font-rules/changes/issue-6936-new-gulim-face-identity.json)은
  기존 HCR Dotum 치환을 New Gulim으로 정정한다. retired/successor 이력과 public trace ID 계약을
  보존하고 나머지 네 projection semantic hash는 같다. 봉인 v1를 다시 쓰지 않는다.
- 신규 회귀 9개와 기존 projection 회귀 3개에서 actual font fallback, glyph transform,
  단일 text show, 결정성, opacity·italic·mixed span·child stroke 반례를 확인했다.

## 2. 공통 조판 원칙

| 검토 항목 | 근거 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 한컴의 `w/Tf=0.02`와 PDF Tr 2를 독립 측정했다. 실제 선택 face weight와 균일 paint를 조건으로 사용하며 파일명·문서별 분기는 없다. 실제 Bold, 혼합 span, fallback, paint 반례를 실행했다. | 충족 |
| 측정·배치 일관성 | PDF 준비 단계는 usvg의 실제 glyph face를 조회하고 원본 문자·transform을 유지한다. 새굴림은 공통 이름 규칙과 기존 New Gulim metrics를 함께 사용한다. 다른 10개 행의 glyph 원점 변화는 0이다. | 충족 |
| 줄 소속과 점유 높이 | 줄 소속·LineSeg·점유 높이 알고리즘을 변경하지 않는다. 새굴림 face 복원으로 해당 두 행의 advance만 달라지며 1페이지를 유지한다. | 비해당 |
| 사례와 증거의 독립성 | 합성 계약 테스트와 별개로 한컴 저장 HWP 및 한컴 PDF를 생성하고 12개 행을 직접 대조했다. 비공개 사내 원본 코퍼스를 재현했다고 주장하지 않는다. | 충족 |
| 기준값 변경 | 한컴 PDF와 직접 대응하는 새 HWP의 쪽수 원장 1행만 추가했다. 글꼴 교체는 명시적 change set으로 기록하며 봉인 v1와 과거 W7 baseline은 보존했다. | 충족 |
| 주장과 검증 범위 | 코드 SHA·명령·결과 및 입력/산출 SHA를 기록한다. Windows 네이티브 실행, 사내 코퍼스 및 대규모 PDF 성능은 미검증으로 남긴다. | 충족 |

## 3. 검증 입력과 독립 시각 증거

**입력 커밋 확인: 충족.** [증적 manifest](../assets/issue_6936/README.md)에 실제 사용한 HWP/HWPX 및
PDF 4개의 SHA-256·출처·한컴 변환 job ID를 기록했다. 코드 검증 commit의 Git blob과 실행 입력이
같고, 최종 코드 재출력 PDF도 커밋된 최종 PDF와 바이트 동일하다. 기존 파일의 이름만 바꾼 중복 추가는 없다.
라이선스 글꼴 파일은 Windows 설치본을 읽기 전용으로 제공했으며 Git에는 포함하지 않았다.

한컴 12.0.0.4605 engine 2020의 기준 PDF와 같은 HWP의 수정 전/획만 추가/최종 결과를 비교했다.
[비교 패널](../assets/issue_6936/comparison.png)의 1페이지 12개 행을 직접 확인했다.
돋움·바탕·굴림·새굴림의 합성 굵기는 복원되고 함초롬 두 계열의 실제 Bold와 일반 행은 유지된다.

- 공백 제외 추출 문자: 한컴 158, 수정 전 158, 획만 추가한 대조군 207, 최종 158.
- 최종 PDF는 glyph 158개를 각각 한 번 기록하며 합성 굵게 49개에 `Tr 2`를 사용한다.
- 새굴림 두 행은 원래 face의 advance로 복원되어 최대 원점 차이 5.200012pt가 있다.
  나머지 10개 행의 원점 변화는 0이다. 전체 페이지 픽셀 동일성 또는 사내 비공개 코퍼스 재현은 주장하지 않는다.
- 기준 PDF와 직접 대응하는 새 HWP의 쪽수 원장 `1 / 1` 한 행만 추가했다.
  전체 suite의 기존 SVG golden 및 코퍼스 래칫은 통과하여 기존 baseline 재생성은 필요하지 않았다.

## 4. 실행 검증과 한계

[상세 명령·결과](../../report/task_m100_6936_report.md)를 함께 보존한다.

| 검사 | 결과 |
| --- | --- |
| Rust fmt / native·WASM32·workspace all-target Clippy / workspace build / suite manifest | 모두 PASS |
| 로컬 full release-test nextest | 9,749 PASS / 51 skipped / 실패 0 |
| Native Skia lib / 그림 placeholder / 직접 PDF export | 4,112 PASS·13 ignored / 2 PASS / 4 PASS |
| #6936 / #5874 / #3772 / #7077 focused | 9 / 7 / 2 / 2 PASS |
| 글꼴 reducer·projection·mutation / 기존 Rust projection | 45 / 3 PASS |
| 실제 HWP public font trace | 새 rule 참조 34개, dangling/retired/replaced/traceSourceDrift 0 |
| fresh WASM 진단 build + 실제 native/WASM SVG | HWP/HWPX 각각 1페이지 바이트 동일 |
| PDF glyph·추출·원점·시각 비교 | 위 독립 기준과 범위에서 PASS |

추가 Node 전수 검사는 97개 중 93 PASS·4 FAIL이다. 과거 population/lifecycle 및 W7 snapshot 검사
네 건은 기준 devel에서도 실패하며 상세 보고서에 이름과 대조 결과를 기록했다. 기존 실패를 PASS로
표시하거나 봉인 기준을 완화하지 않았다. 첫 full Rust 실행의 새 rule ID/retired 개수 검사 실패 2개는
보정 후 전체를 다시 실행해 해소했다.

WASM은 공통 새굴림 이름 규칙의 회귀 검사다. Docker daemon 부재로 허용된 `--no-opt` 진단 빌드를
사용했고 Node에서 실제 WASM을 실행했다. 최적화된 Docker 배포 빌드·Studio UI·Windows 네이티브
CLI·사내 코퍼스·대규모 PDF export 성능은 미검증이다. 관련 없는 HML Bold 파싱 의문은 해결 범위에 넣지 않는다.

## 5. 코드 후보 GitHub CI

아래는 모두 `ac1e8ec9e`의 completed/success 결과다. 작성 직전 `MERGEABLE / CLEAN`과
CI Impact Policy SUCCESS도 확인했다.

- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/34759398196): `fast_pass=false`,
  `reason=no-green-build-candidate`; Lint·Native Skia·Frontend package 및 A/B/C/D worker 실제 실행.
  Linux 회귀 A 3,873 + B 1,578 + C 2,320 + D 1,785 = **9,556 PASS / 51 skipped**, Build & Test SUCCESS.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34759398221): JavaScript/TypeScript·Python·Rust 분석 SUCCESS.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34759398140): Canvas visual diff SUCCESS.
- [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34759398210),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34759398212): SUCCESS.
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/34759397747): SUCCESS;
  exact head의 CI Impact Policy status도 SUCCESS.

별도 WASM Build 및 Frontend unit gates는 SKIPPED이며 실행 성공으로 세지 않았다.
WASM cfg는 CI Lint에 포함되며 fresh WASM 실행 근거는 위 로컬 검증으로 구분한다.

## 6. trailing commit과 병합 후 처리

검증 후보 뒤에 이 self-review와 오늘할일을 single-parent 문서 commit으로 추가한다.
코드·시험·fixture·workflow는 바꾸지 않는다. 최신 base/head merge-tree, 공백, 문서 링크와
기존 오늘할일 보존을 확인하고 push한다. 최신 head의 preflight가 실제로 후보를 재사용하는지,
heavy worker skip과 required aggregate 완료 상태를 확인한 후 병합한다.

### Merge 후 contributor PR comment 계획

[PR #7108](https://github.com/edwardkim/rhwp/pull/7108)에 실제 merge SHA와 최종 CI 링크,
위 로컬 검증 수치 및 문서·시각 비교 결과를 게시한다. devel에 존재하는
`mydocs/pr/assets/issue_6936/comparison.png`를 merge SHA raw URL 이미지로 포함하고
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을
연결한다. p.1·12행, 추출 문자 207→158, 합성 glyph 49개·단일 방출 및 새굴림 원점 차이를
사람의 시각 결론과 함께 적는다. 측정하지 않은 pixel match 백분율을 만들어 넣지 않는다.

### Issue 종료와 정리 계획

운영 문서는 PR에 포함되어 병합되므로 별도 문서 PR은 필요하지 않다. primary devel을 fast-forward한 뒤
[이슈 #6936](https://github.com/edwardkim/rhwp/issues/6936)를 다시 조회한다. 본문 `Closes #6936`에도
기본 branch가 main이라 OPEN이면 수동 종료하고 실제 merge SHA·검증·기준 PDF/비교 패널을 comment한다.
동일 merge의 comment가 있으면 중복 게시하지 않는다. HML 별건 또는 비공개 사내 코퍼스까지 해결했다고 쓰지 않는다.

병합 뒤 검증 workflow를 실행하거나 재실행하지 않고 `Refresh nextest target duration data`의
성공 또는 증거 부족으로 인한 갱신 보류를 확인한다. 이 작업 소유의 clean worktree
`/Users/tsjang/rhwp-review-6936-20260913`, branch 및 전용 target
`/Users/tsjang/rhwp/target/issue6936-20260913`을 활성 Cargo 작업과 공유 여부 확인 뒤 정리한다.
원격 head는 최종 SHA를 lease로 확인한 뒤 제거하며 primary와 다른 작업 소유 산출물은 보존한다.

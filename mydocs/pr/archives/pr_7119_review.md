---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7119 — Canvas 보충 진행폭 self-review

검토 판정: **승인**. 최신 원격 `6434189bc`의 CI 완료 후 정식 self-review를 진행했다.
검토한 변경 범위에서 병합 차단 결함을 발견하지 못했다. 이는 GitHub approve나 병합 승인이 아니다.
기존 로컬 검증과 메인테이너 시각 통과는 유지하며 아래 지원 한계는 남긴다.

## 대상과 승인 경계

- [PR #7119](https://github.com/edwardkim/rhwp/pull/7119), [issue #7084](https://github.com/edwardkim/rhwp/issues/7084).
- 작성자/assignee `edwardkim`, milestone `v1.0.0`, labels `bug`, `rhwp-studio`, `rendering`, `font`.
- `task_m100_7084` → `devel`, Open(non-Draft), reviewer 지정 없음.
- 최초 게시 head `78f372a5b020ab3d7f23c79c1a7ba409f7c75d49`.
- 검증 source/test/fixture `d3a189d5d964e4e6eb3b376062df846744c4c58d`.
- 재조회한 base `11860a9f4186438a3cb9c355e3cd8d2257162d3a`; 최초 merge-tree exit 0.
- 기본 경로 `collaborator_self_merge`; modifier `intake_and_review`, `local_validation`,
  `visual_fixture_evidence`, `rework_and_exceptions`, `review_only_fast_pass`를 읽고 적용했다.
- 최초 승인은 작업 branch push·Open PR 생성·게시 준비 기록이었다. 이어서 CI 확인 후 self-review
  진행 승인을 받았다. 이번 검토에서는 GitHub approve/comment, merge·issue close를 수행하지 않는다.
  [후속 검토 순서](pr_7119_review_impl.md)를 따른다.

## 최신 CI 확인

2026-09-14 KST, 아래 모두 `6434189bc3564be34e6fe7021fe07568eaffa194`에 귀속된다.

| 검증 | 결과 | 근거 |
| --- | --- | --- |
| CI | SUCCESS | [34798218652](https://github.com/edwardkim/rhwp/actions/runs/34798218652) |
| CodeQL (Rust/JS·TS/Python) | SUCCESS | [34798218743](https://github.com/edwardkim/rhwp/actions/runs/34798218743) |
| Render Diff | SUCCESS | [34798218651](https://github.com/edwardkim/rhwp/actions/runs/34798218651) |
| Adapter inter-diff | SUCCESS | [34798218770](https://github.com/edwardkim/rhwp/actions/runs/34798218770) |
| Proptest roundtrip | SUCCESS | [34798218716](https://github.com/edwardkim/rhwp/actions/runs/34798218716) |

Build & Test aggregate, Lint, Native Skia, Archive A/B/C/D tests, Frontend package gates의 실제
성공을 확인했다. Render Diff의 Canvas visual diff와 CodeQL 언어별 Analyze가 실행됐다.
전체 작업이 fast-pass로 생략됐다고 해석하지 않는다. 별도 GHAS `CodeQL` check와
최신 `CI Impact Policy` status도 성공이며 PR은 Open/MERGEABLE이다.
후행 검토 문서를 push하면 그 새 SHA의 required check는 다시 확인해야 한다.

## 완료된 검증

[Stage 7](../../working/task_m100_7084_stage7.md) 6절과
[결과보고서](../../report/task_m100_7084_report.md)의 동일 후보 검증을 재사용한다.
9,814 nextest PASS/51 skipped/0 FAIL, 별도 lib 4,055 PASS/13 ignored,
집중 84 PASS, Native Skia 4,112 PASS/13 ignored 및 그림 2·PDF 4 PASS,
세 Clippy·workspace build·fmt·doctest·manifest·unit-tier가 완료됐다.
이 수치는 중복 포함되므로 합산하지 않는다.
Docker WASM·TypeScript·production build, Studio 1,692 PASS/2 skipped,
실제 Chrome HWP/HWPX·서식·portable 복원·결합열 경계·오류 복구 12조건,
renderer/Undo 계약, Render Diff 3/3·직접 PDF gate 3/3이 완료됐다.
원격 CI 성공을 이 로컬 결과로 대신하지 않는다.

제출 시 검증 후보 뒤 변경은 결과 문서 2개뿐이었다. diff 공백 검사와 변경 Markdown 링크 검사 통과.
원본 HWP/HWPX 두 개와 기준 PDF 두 개 모두 실제 검증 bytes가 위 commit의 Git blob과 일치했다.
새 HWP/HWPX sample 추가·변경은 없으므로 신규 sample 보안 검사 입력 대상은 없다.
PDF는 기존 MCP 산출물을 재사용했으며 새로 변환하지 않았다. 상세 출처·SHA-256은 Stage 7 5절을 따른다.

## 조판 원칙 — 정식 self-review

검증 후보와 원격 head 사이 제품·테스트 차이가 없음을 확인하고 실제 자료 흐름과 회귀 assertion을 대조했다.

| 항목 | 현재 근거와 남은 확인 | 판정 |
| --- | --- | --- |
| 구현 근거·일반성 | `char_width_decision`의 기존 `heuristicHalfwidth` 결정에만 보충 폭을 적용한다. DB·공백·PUA·dash leader 제외, 비이모지 𝄞 계약도 확인했다. 문서명/페이지/이모지 정사각형 상수에 의존하지 않는다. | 충족 |
| 측정·배치 일관성 | `CanvasTextFont`가 CSS 크기·장평·첨자 설정을 provider/paint에 공유하고 `natural_advance`에서 한 번 환산한다. 같은 `char_width_decision`을 총폭·문자 위치가 사용하며 Canvas는 등록 descriptor 검증 후 이중 fit을 생략한다. | 충족 |
| 줄 구성·점유 높이 | `ParagraphMetricScope`와 `ComposedTextRun::text_style`이 원문 grapheme 경계를 조판·줄바꿈·caret·표/글상자 측정에 전달한다. 표의 줄 소속/높이 상수나 clamp는 추가하지 않는다. portable 경로의 기존 구성과 비관련 shaping run 유지 assertion을 확인했다. | 충족 |
| 독립 사례·시각 근거 | 기존 실제 HWP/HWPX·한컴 PDF·메인테이너 시각 통과, 실제 Chrome 서식 및 결합열 검사. 합성 계약을 한컴 출력으로 간주하지 않는다. | 충족 |
| baseline 변경 | 쪽수 원장에 독립 PDF를 근거로 두 입력의 2/2 행만 추가. 기존 기대값·허용치 변경 없음. | 충족 |
| 주장과 검증 범위 | 동일 검증 SHA의 전체 로컬 검사와 최신 PR CI를 구분했다. 측정 세션·복구·출력 경로를 직접 읽었으며 아래 미지원/비용 한계를 개선 성공에 포함하지 않는다. | 충족 |

## 세션·실패·출력 경계 검토

- `SupplementalMetricStore::replace`는 전체 batch를 검사한 뒤 snapshot을 교체한다.
  실패는 기존 snapshot을 보존하고 교체/owner drop은 retained style까지 무효화한다.
  4,096개/키 4MiB, 개별 font 32MiB/합계 64MiB 한도를 확인했다.
- 요청은 context·ticket·document/section revision·DPI에 묶인다. 응답의 누락/중복/descriptor
  불일치·비정상 수치를 거부하며 batch edit 중 상태 변경을 거부한다.
- 문서 교체에서 owner를 버리고, 서식 ID 변경에서는 활성 snapshot을 다시 연결한다.
  JS epoch와 view/document/font generation 검사가 await 뒤 오래된 WASM handle 접근을 막는다.
- VS16/ZWJ 등 다중 scalar는 문단 원문에서 먼저 식별한다. run/언어/저장 줄 경계를 넘어도
  구성원 전체가 기존 폭 경로에 남으며, 다른 standalone 이모지의 cache가 부분 적용되지 않는
  HWP/HWPX assertion을 확인했다. 전체 결합열 shaping 구현을 완료했다는 주장은 아니다.
- portable 출력 진입은 활성 Canvas 폭을 거부한다. Studio의 동기 transaction은 출력 페이지
  목록까지 portable 문맥에서 수집하고 finally에서 화면 문맥을 복원한다. 중첩·예외 검사도 있다.
- descriptor 오류는 정확한 오류 문자열에만 반응하고 동일 문서/폰트 세대당 한 번 복구한다.
  재paint가 예산을 충전하지 않으며 일반 오류나 오래된 실패를 새 화면의 복구로 오인하지 않는다.

남는 한계: 브라우저의 실제 fallback face를 식별/보증하지 않으며 OS 간 동일 폭/모양을 보장하지 않는다.
native/CanvasKit/SVG 보충 폭 활성화와 복합 이모지 shaping은 범위 밖이다.
문서 재검사·동기 print SVG 수집 비용은 결과보고서의 큰 문서 warm 약 50ms/print 약 575ms 관측을
유지한다. 무비용 또는 모든 문서에서 실시간이라는 주장을 하지 않는다. 이 검토에서 성능 A/B를 새로 측정하지 않았다.

추가 집중 검증: `node --test rhwp-studio/tests/supplemental-text-metrics.test.ts
rhwp-studio/tests/canvas-metric-session.test.ts rhwp-studio/tests/canvas-metric-recovery.test.ts`
25 PASS/0 FAIL. unavailable Canvas 경고는 실패 경로를 검증하는 기대된 진단이다.
Rust는 기존 review worktree의 검증 후보 `d3a189d5d`에서 다음 집중 명령을 실행했다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --tests --no-fail-fast \
  -E 'test(/issue_7084/)'
```

32 PASS/0 FAIL, 9,833개는 필터에 따라 실행하지 않았다. 이는 전체 회귀 재실행이 아니다.
캐시 재컴파일 3분 45초, 실제 집중 실행 0.075초, exit 0.
nextest run `9a7a4259-c410-4c9f-850a-7c7b52786f74`이며 기존 nextest 버전/JUnit 경고는 유지한다.
제품 코드 변경이 없고 같은 후보의 전체 검증·원격 Full 실행이 성공했으므로 전체 Cargo/WASM/시각
재생성은 반복하지 않는다. 새 시각 판정을 만들어 이전 메인테이너 판정을 대체하지 않는다.

## 직접 시각 증적

[표준 비교 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)의
compare/overlay/review 패널을 사용했다. 실행 방법은 Stage 7 7절을 따른다.
HWP/HWPX 각각 2쪽 한 장씩, 총 2페이지를 비교했다. 구조 자동 후보 검사는 실행하지 않았으며
후보 0건이라고 주장하지 않는다. 각각 pixel match 90.78317%, 내용 픽셀 보조값 6.68869%다.
메인테이너는 두 형식의 이모지 렌더링을 통과로 판정했다. 통합 화면은 이전 승인 유지 화면과
byte-identical이며 활성 paint 배율 1/1, 비활성 대조 약 0.364245/1이다.
이 수치는 전체 조판 fidelity 또는 이모지 정확도를 뜻하지 않는다. PDF와의 색상/글꼴·표 간격 차이는 남는다.

- 임시 패널: `output/7084/stage7/visual/{hwp,hwpx}/{compare,overlay,review}/`의 `*_002.png`.
- 영구 대표: [HWP 2쪽 비교](../assets/pr_7119_7084_hwp_review_002.png).
- 대표 PNG를 직접 열어 본문·도구 라벨·한글·지표·footer가 판독 가능함을 확인했다.
  두 형식의 Canvas 본문이 같으므로 대표 하나만 보존한다. 중간 로그·JSON·PNG는 커밋하지 않는다.
- PDF SHA-1: HWP `b143b6154f036b37903801168ef56b6eba0c21f5`, HWPX `420f1785b2e70e7a2ce91a87b9e3f3160c60f8eb`.
  두 PDF 모두 1.6, 2쪽, 595×841pt, Creator/Producer `Hancom PDF 1.3.0.550`.
  MCP engine/profile 2020, 실제 응답 Hancom 12.0.0.4605를 구분한다.

## Merge 후 contributor PR comment 계획

현재 게시하지 않는다. 정식 self-review·최신 CI·병합 승인 후 실제 merge와 asset의 devel 포함을 확인한 뒤,
별도 승인 범위에서 UTF-8 LF 본문 파일을 `gh pr comment --body-file`로 게시하고 API로 재조회한다.
문서 비교 정본 링크, 위 2페이지/자동 후보 미실행/지표, 사람의 이모지 통과와 잔여 차이를 함께 쓴다.
대표 이미지 URL은 실제 merge SHA로 다음 형식을 채운다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7119_7084_hwp_review_002.png`

이 PR 생성 또는 로컬 검증만으로 #7084를 close하지 않는다.

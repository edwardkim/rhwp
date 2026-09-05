# PR #6739 검토

## 최종 판정: 승인

정적 검토, 통합 후보의 Studio 테스트·타입 검사·빌드와 직접 Chrome CDP 검증을 완료했다.
정상 줄 경계의 조합 표시, 부적합 좌표에서 일반 캐럿으로의 복귀와 조합 확정 후 텍스트 보존을
확인해 로컬 보류를 해제했다. 이 판정은 통합 PR 생성·push·merge 승인을 대신하지 않는다.

## 검토 대상

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#6739](https://github.com/edwardkim/rhwp/pull/6739) |
| 제목 | Task #6738: 조합 오버레이가 성립하지 않는 기하를 폴백 폭으로 덮지 않는다 |
| 작성자 / 리뷰어 | `lpaiu-cs` / `jangster77` |
| 원 PR 기준 브랜치 | `devel` |
| 확인한 원 head | `facfdf93ea5ef1b5e43a30121251bce0365a0775` |
| 통합 기준 | `upstream/devel`의 `1f861362ab372f9fa26e38f9a534a89286f641c4` |
| 로컬 검토 브랜치 | `review/lpaiu-6739-6770-20260905` |
| 통합 코드 후보 | `5bc8e87f5879d3308ae006bcc8e53410477ee580` + #6770 미커밋 메인터너 보정 |
| 변경 규모 | Studio 구현 2개, TypeScript 테스트 1개 파일 |
| 작성 시점 mergeable 참고값 | 이 검토에서 수집하지 않음. 원 CI 성공과 merge 가능 여부는 별개 |

작성일은 2026-09-05다. head와 CI는 조회 시점의 참고값이며, 실제 PR 제출·merge 전에
최신 상태를 별도로 확인해야 한다. 두 원 PR의 reviewer 지정은 로컬 fetch·체리픽 전에 완료했다.

## 관련 이슈와 범위

[이슈 #6738](https://github.com/edwardkim/rhwp/issues/6738)은 잘못된 시작 좌표와 현재 캐럿으로
계산한 음수 폭이 최소 폭 대체 로직에 가려지는 문제의 안전장치를 다룬다.
본문의 실제 줄 경계 잔상은 선행 #6736에서 고쳤다는 이슈의 범위를 유지한다.
이번 PR을 다섯 문맥의 줄 경계 좌표 API 통일이나 모든 다중 줄 조합의 해결로 해석하지 않는다.

## 정적 검토 결과

- `input-handler.ts`는 조합 시작 좌표를 보정한 뒤 페이지 불일치 또는 역전된 가로 좌표를 검사한다. 조건을 만족하지 않으면 오버레이를 숨기고 일반 캐럿으로 복귀한다.
- `line-start-affinity.ts`는 줄 정보 조회 실패를 `null`로 취급하고 기존 좌표로 돌아간다. 실제 호출부도 WASM 예외를 이 계약으로 변환한다.
- 다른 페이지의 줄 좌표를 반환할 때 이전 페이지의 셀 경계 상자와 `cellOverflowed`를 재사용하지 않는다. 같은 페이지에서는 기존 셀 경계를 유지한다.
- `isCompositionBoxRepresentable`의 같은 페이지·가로 좌표 순서 검사는 필요조건이다. 서로 다른 줄이 항상 검출된다고 주장하지 않는다. 글꼴 크기가 섞인 줄에서 단순 세로 좌표 비교를 추가하는 보정도 하지 않았다.
- 기존 `CaretRenderer.hideComposition()`과 `update()` 연결은 일반 캐럿 복귀 의도와 맞는다. Chrome에서 오버레이 숨김과 현재 캐럿 표시, 텍스트 보존을 직접 확인했다. 셀 경계 전달은 통제된 조회값과 단위 테스트로 확인했으며 실제 모든 셀·페이지 조합의 전수 시각 검증으로 확대하지 않는다.
- 이 PR 자체에는 메인터너 보정 코드가 필요하지 않았다. 같은 통합 후보의 #6770 보정은 해당 개별 리뷰에 구분했다. 체리픽 충돌은 없었다.

## 원 PR CI 확인 결과

아래 결과는 원 head의 GitHub 결과이지, 최신 `devel` 위 로컬 통합 후보를 실행한 결과가 아니다.

| 검사 | 조회 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/33962087275) | Build & Test 및 실행된 프런트엔드 작업 성공 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33962087208) | 집계 및 실행된 언어 작업 성공 |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/33962087216) | 성공 |
| [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33962087200) | 성공 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33962087135) | 실행된 Canvas 검사 성공 |

Rust 고비용 작업의 생략을 Rust 전체 회귀 성공으로 기록하지 않는다.
별도 GitHub 코드 스캔 `CodeQL` check의 `NEUTRAL`도 언어 분석 작업의 `SUCCESS`와 구분한다.

## 완료한 검증과 한계

1. 보정 전 Studio 테스트는 1,414 통과·1 건너뜀·실패 0, #6770 보정 후에는 1,424 통과·1 건너뜀·실패 0이었다. `tsc --noEmit`, `npm run build`와 검토 전용 target의 최적화 WASM 빌드가 통과했다.
2. `samples/143E433F503322BD33.hwp`를 실제 Studio에 열고 Chrome CDP로 조합 이벤트를 전달했다. 최초 편집 뒤 재조판된 본문 줄 경계는 offset 26이었다. 원 좌표 `(381.3, 125.8)`를 다음 줄의 `(75.6, 146.5)`로 보정해 폭 `13.3 px`의 정상 조합 박스를 표시했다.
3. 동일 조합 상태에서 이전 줄의 원 좌표를 주입했다. 가드를 우회한 대조 표시의 폭은 `7.98 px`, top은 `135.8 px`였다. 실제 가드 경로에서는 오버레이를 숨기고 top `157.1 px`에 일반 캐럿을 표시했다. 페이지 불일치 주입도 같은 복귀 경로를 통과했다.
4. 같은 페이지 셀 경계 유지, 다른 페이지의 경계·overflow 표식 제거, 조회 실패 시 원 좌표 유지와 서로 다른 run 높이·y의 허용을 실제 브라우저 모듈에 통제된 조회값을 전달해 확인했다.
5. 조합을 확정한 뒤 내용이 보존됐다. 초기 안내창이 겹친 첫 캡처는 제외하고 안내 완료 후 핵심 시나리오를 다시 실행·캡처했다. 최종 PNG를 직접 열어 비교했다.

직접 증적은 [공통 시각 검증 기록](pr_6739_6770_visual_sweep.md)에 있다. 정상 본문 조합은 실제
WASM 좌표를 사용했고, 가드 발동은 좌표 주입 대조임을 분리했다. OS IME·머리말·각주·거대 중첩 셀의
전수 검증이나 다중 줄 조합 지원을 주장하지 않는다. Rust 소스를 바꾸지 않아 전체 Rust 회귀는 생략했다.
최종 미커밋 보정 diff의 SHA-256은 `e00d6c035ce675020915a2c00a089e86792ba70a6a668d0c46589132ff74fd67`이다.
새 통합 PR의 CI는 아직 없으므로 원 PR CI와 구분해 이후 확인해야 한다.

## Merge 후 contributor PR comment 계획

- 아직 게시 승인을 요청하거나 댓글을 게시하지 않았다. 통합 PR도 생성하지 않았다.
- 대표 PNG `6739-normal-wrap.png`, `6739-guard-bypassed.png`, `6739-guard-fallback.png`를 `mydocs/pr/assets/pr_6739_6770_20260905/`에 보존했다. 공통 시각 검증 기록 링크와 함께 세 이미지를 댓글에서 직접 표시할 계획이다.
- 최종 댓글에는 실제 통합 PR·merge SHA, 원 head 및 통합 PR/devel CI, 자연 입력과 주입 검사의 구분, 확인한 문맥과 제한을 적는다.
- 그림은 확정된 SHA를 사용한 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6739_6770_20260905/<대표-PNG>`를 Markdown 이미지로 삽입한다.
- 실행 로그와 임시 PNG·SVG·JSON은 저장소 증적에서 제외했다. 본 검토의 PNG는 직접 캡처한 화면이며 기여자의 이미지를 재측정 결과로 바꾸지 않았다.
- 승인과 merge 후 `post_merge.md`에 따라 UTF-8 본문 파일로 한 번만 게시하고 API로 본문을 확인한다. #6738의 실제 closing reference와 기존 댓글을 확인해 중복 조치하지 않는다.

통합 출처와 단계별 승인 조건은 [통합 처리 계획](pr_6739_review_impl.md)에 기록했다.

## 2026-09-05 PR 게시 승인 후 확정 기록

- 작업지시자가 메인터너 보정과 검토 기록의 커밋 및 통합 PR 생성을 승인했다. 앞의 미커밋/승인 대기 표기는 로컬 검증 당시 상태이며, 게시 준비 단계에서는 이 절을 따른다.
- #6739의 최신 source head는 `facfdf93ea5ef1b5e43a30121251bce0365a0775`로 유지됐다.
- #6770의 최신 source head는 `d8d8828f5ea6acf7506db20f541ac1186f00755d`다. 기여자가 선행 #6763의 통합을 반영하여 `1f861362ab372f9fa26e38f9a534a89286f641c4` 위로 재배치했다.
- `git range-diff`에서 이전 `d07f721fade0e2397284b5b1119898d1c42f54e0`과 최신 `d8d8828f5ea6acf7506db20f541ac1186f00755d`의 기능 패치는 `=`로 확인됐고, `git cherry HEAD <최신 source head>`도 이미 적용된 패치(`-`)로 판정했다. 로컬 `5bc8e87f5879d3308ae006bcc8e53410477ee580`의 기존 provenance를 보존하며 같은 기능을 중복 체리픽하지 않았다.
- 검토 branch의 기준 `upstream/devel`도 `1f861362ab372f9fa26e38f9a534a89286f641c4`로 동일하다. 검증한 메인터너 보정 diff의 SHA-256 `e00d6c035ce675020915a2c00a089e86792ba70a6a668d0c46589132ff74fd67`과 커밋 직전 diff가 일치했다.
- 실제 소스 변경이 없으므로 기존 Studio 1,424 통과/실패 0/skip 1, TypeScript 검사, Studio 및 WASM 빌드, CDP 검증 결과를 재사용한다. 이번 게시 단계에서 테스트를 다시 실행했다고 기록하지 않는다. 통합 PR 최신 head의 GitHub CI 결과는 별도로 기다려야 한다.
- 최종 판정은 #6739 **승인**, #6770 **메인터너 보정 완료, 수용 가능**이다. 원 PR/관련 이슈 comment와 close, 통합 PR merge, branch/target 정리는 이번 게시 작업에서 수행하지 않는다.

- 메인터너 code/test 보정 커밋: `f7c53792ca966521567a79078247fb28a2b1111a`. 검토 문서, 오늘할일 및 코멘트용 PNG 5개는 이 커밋 뒤의 별도 기록 커밋으로 포함한다.

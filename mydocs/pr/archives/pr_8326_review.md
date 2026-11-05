# PR #8326 검토 — fix(hwpx): 내보낸 HWPX의 칸 배경 그림과 HWP 그림 글머리표가 다른 그림을 가리키지 않게 한다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8326, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `32c2a1e9febb303ac0662021279cf9171b346cb8`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:44.159666+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: CONFLICT (content): Merge conflict in src/model/style.rs; CONFLICT (content): Merge conflict in src/parser/hwpx/mod.rs; CONFLICT (content): Merge conflict in src/serializer/doc_info.rs; CONFLICT (content): Merge conflict in src/serializer/hwpx/header.rs.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](../assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 기존 해결과 실제 저장 계약

HWP 그림 글머리표의 기본 참조는 #8438 → #8517과 `c97709` 계열에서 이미 보정됐다. 최신 `Bullet::image_reference`와 누락 참조 오류·그림 효과·Storage 예산을 유지하며, 이번 후보의 새 기능은 HWPX `header.xml`의 `img`/`image` 참조를 본문과 동일하게 manifest ID에서 정규화하는 것이다. 글꼴 참조 ID는 변경하지 않는다. 원 PR의 오래된 HWP 모델 표현을 복구하지 않았다.

실제 HWP → HWPX 내보내기에서 `--verify --verify-pages`를 base/후보 양쪽의 17쪽·3쪽 문서에 실행해 모두 성공했다. 저장본도 같은 독립 Print의 전체 20쪽과 비교했다. 칸 배경 참조와 서로 다른 원본 그림 바이트의 보존은 `hwpx_export_header_image_refs`의 세 검사로 확인한다. NewYear 그림 글머리표는 기존 해결 출력이 유지된다. 추가 byte-guard 입력은 데이터 참조 검사이며 별도 전체 문서 시각 일치를 주장하지 않는다.

원본 17쪽의 N/W 최저는 각각 99.62%, 3쪽은 98.43%다. 저장 재열기 17쪽은 base의 N/W 96.68/96.75%에서 후보 99.62/99.62%로 개선됐고 3쪽은 양쪽 98.43%를 유지했다. 실제 사용한 원본·PDF·저장본의 Git blob은 [입력 검증](../assets/semanticist21-20261105/input-commit-verification.json)에 있다.

최종 WASM의 실제 `exportHwpx` → 재열기 → 재저장도 실행했다. 첫 저장은 정식 증거에 포함된 Native 저장본과 두 파일 모두 바이트가 같다. 두 번째 저장은 ID가 다시 정규화돼 파일 hash가 달라져도 header/body의 참조 10개·2개가 해소하는 그림 바이트는 각각 모두 같다. [실제 binding 결과](../assets/semanticist21-20261105/final-wasm-image-save.json).

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](../assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](../assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](../assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·Open 통합 PR·해당 head CI·self-review·merge·후속 댓글·정리를 작업지시자가 승인했다. 실제 실행과 성공을 확인한 뒤 원격 결과를 기록한다.

## Merge 후 contributor PR comment 계획

HWPX로 저장한 뒤 머리말 그림과 새해 문서의 그림 글머리표 참조가 유지된다. 본문·머리말 그림의 소속과 크기 및 저장 재열기를 확인했다.

실제 통합 PR·merge SHA·최신 head CI URL을 확정한 뒤 아래 원 PR별 증적과 함께 한국어 존댓말로 감사 댓글을 게시한다. 그림은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 아래 경로를 붙인다. merge SHA의 HTTP Content-Type과 SHA-256을 검증하고 게시 후 API 본문과 GitHub 화면 표시를 확인한다.

| 입력·쪽 | 경로 | raw 실루엣(2px) | pixel match | 내용 픽셀 보조값 | review / overlay |
| --- | --- | --- | --- | --- | --- |
| `roundtrip-header-image-refs` 4쪽 | native | 99.86496% | 95.42749% | 50.44102% | [review](../assets/semanticist21-20261105/comment-evidence/pr8326-roundtrip-header-image-refs/native/review/review_004.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8326-roundtrip-header-image-refs/native/overlay/overlay_004.png) |
| `roundtrip-header-image-refs` 4쪽 | wasm | 99.94992% | 94.59617% | 41.19537% | [review](../assets/semanticist21-20261105/comment-evidence/pr8326-roundtrip-header-image-refs/wasm/review/review_004.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8326-roundtrip-header-image-refs/wasm/overlay/overlay_004.png) |
| `roundtrip-image-bullet-newyear` 1쪽 | native | 98.79866% | 95.08479% | 92.70295% | [review](../assets/semanticist21-20261105/comment-evidence/pr8326-roundtrip-image-bullet-newyear/native/review/review_001.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8326-roundtrip-image-bullet-newyear/native/overlay/overlay_001.png) |
| `roundtrip-image-bullet-newyear` 1쪽 | wasm | 98.79678% | 96.74037% | 95.15122% | [review](../assets/semanticist21-20261105/comment-evidence/pr8326-roundtrip-image-bullet-newyear/wasm/review/review_001.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8326-roundtrip-image-bullet-newyear/wasm/overlay/overlay_001.png) |

자동 geometry 후보 추출은 수행하지 않았으므로 후보 수를 0으로 쓰지 않는다. TSV-only 전쪽 raw 측정과 직접 판독을 구분한다. 해당 기능 검증 문서의 Native/fresh WASM 전쪽 raw 90% 미만은 0쪽이다. 내용 픽셀 보조값은 사람의 판정 정확도가 아니며 승인 gate를 대신하지 않는다. [Visual Sweep 댓글 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [전쪽 TSV manifest](../assets/semanticist21-20261105/visual-manifest.json)를 연결한다.

## GitHub CI와 self-review

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)의 code head `da0074f41ee01d7fa79e6bd4ffd5cadf7c69665d`에서 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41937077195)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41937077068)·Adapter·Proptest와 최종 CI 정책 집계의 성공을 확인했다. self-review의 추가 차단 사항은 없으며 통합 후보는 승인이다. [실제 check·job 결과](../assets/semanticist21-20261105/github-ci-code-head.json), [self-review 근거](../assets/semanticist21-20261105/self-review.json).

이 기록은 녹색 code head 뒤의 문서 전용 single-parent trailing commit이다. 새 trailing head의 실제 fast-pass·필수 집계·최신 base 충돌 상태를 확인한 뒤 승인된 merge를 진행한다.

공통 보안 후속으로 기존 행 높이 진단 출력 두 그룹과 같은 채널의 span 진단만 제거했다. 보정 source `d75809517f`에서 전체 회귀 12,499개·Native CI 22개 명령 192개·Native 전체 라이브러리 4,040개를 통과했고, 실제 암호 Native CLI의 SVG 24쪽은 이전과 같다. 새 code head `da0074f41e`의 독립 GHAS CodeQL도 성공했다. 진단 외 조판 코드와 기존 시각 producer를 구분한 [실제 근거](../assets/semanticist21-20261105/ci-security-followup.md)를 후속 댓글과 함께 연결한다.

## Merge 확인

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)을 merge commit `e9d4a23be4523a97c29ce6f5e407aba6c4cfb26c`로 통합했다. 원 PR 10건 모두 간접 MERGED를 확인했으며 원 기여 commit·fork branch를 보존했다. [code CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[문서 head CI](https://github.com/edwardkim/rhwp/actions/runs/41941236661)와 실제 fast-pass·최종 정책 status·fresh-base merge-tree를 확인했고, [COMMENT self-review](https://github.com/edwardkim/rhwp/pull/8609#pullrequestreview-5778015173)를 최종 head `1557f70b03b92f0af88fb0e6f4ef280df4b8683a`에 게시했다. 이 원 PR도 GitHub `MERGED`다. [원 PR별 확정 상태](../assets/semanticist21-20261105/remote-completion.json). 위 `Merge 후 contributor PR comment 계획`의 해당 문서·페이지 review/overlay를 실제 merge SHA에 고정해 감사 댓글을 게시한다.

후속 처리 완료: [해당 원 PR 감사 댓글](https://github.com/edwardkim/rhwp/pull/8326#issuecomment-6646330645)을 게시하고 API 본문 및 GitHub 화면의 8개 review/overlay 표시를 확인했다. 통합 PR의 전용 branch/worktree·root output·검토 debug 종료 정리도 완료했다.

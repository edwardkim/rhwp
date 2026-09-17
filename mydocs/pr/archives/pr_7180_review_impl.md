---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7180_review_impl.md
last_verified: 2026-09-16
---

# PR #7180 적용·검증 기록

[검토 판정](pr_7180_review.md) / [최종 실행·제한 사항](../../working/task_m100_6970_open_pr_stage2.md).

## 2회차 메인터너 보정과 최종 검증

- 검증 코드 commit: `f2e21164c029320d1e9b42a8c8ad1751c69fcb1d`, branch `codex/open-pr-review-20260916`.
- 판정: #7118·#7184의 기존 보류 사유 해소. 개별 변경 범위 승인; 전체 이슈 해결과 원격 CI 승인은 별도다.
- focused 66/66, 전체 nextest 9933/9933(기존 skip 51), fmt·Clippy 3종·workspace build·base 고정 정책 통과.
- Native Skia lib 4112 pass/13 ignore, 누락 그림 2/2, 직접 PDF 4/4 통과.
- 새 WASM host `--no-opt` 통과. Docker 데몬 부재로 배포 최적화 경로는 미실행이다.
- Native/fresh WASM 각각 6문서 22쪽 직접 Visual Sweep. backend 간 본문 픽셀 22/22 동일.
- 기존 PDF 차이, 합성 경계 테스트 범위, 실제 호출 경로는 [2회차 보고](../../working/task_m100_6970_open_pr_stage2.md)에 기록했다.
- 원 PR 8개 head는 종료 전 재조회해 불변·OPEN을 확인했다. 기존 Git 입력/PDF 12개 재사용, 중복 파일 추가 없음.
- 새 영문 코드 주석을 한국어로 수정했다. RED용 임시 worktree는 Cargo 종료 후 제거했고 검토 target은 유지했다.

[명령/exit code](../assets/pr7118_7187_review_stage2/gate-results.json) · [RED/GREEN](../assets/pr7118_7187_review_stage2/red-green-results.json) ·
[소스/CLI 해시](../assets/pr7118_7187_review_stage2/validation-source.json) · [source head](../assets/pr7118_7187_review_stage2/source-heads-stage2.json) ·
[Visual Sweep](../assets/pr7118_7187_review_stage2/visual-results.json).

## 원본 적용 계보

| 원 commit | 누적 cherry-pick commit |
| --- | --- |
| `6989418dee9998d7044bfd807d34f4b6f907fadb` | `f8678abf3755552b0b1de1a0c7318c7ef48b938a` |

## 1회차 기록 — 보정 전

아래 53/54 실패와 최종 게이트 미실행 기록은 보정 전의 이력이다. 현재 판정은 위 2회차를 따른다.

- 원 `refs/pull/7180/head`를 fetch하고 PR API의 head SHA와 일치함을 확인했다.
- 최신 devel `8d45f242baa1a565357aaa38e9f459595b1e756c` 위의 누적 적용이다. 텍스트 충돌 없음.
- code 검증: `2a2089bf71e7a42286e0643a3fb517b7047c7320`. 별도 보정은 #7118의 동일 바이트 fixture 참조 경로와 중복 복사본 제거뿐이다.
- macOS arm64, Rust 1.93.1, `DEVELOPER_DIR=/Library/Developer/CommandLineTools`.
  처음 선택된 Xcode의 라이선스 오류는 테스트 실패로 세지 않았다. 실제 CLT 빌드 exit 0.
- 독립 target: `/Users/tsjang/rhwp/target/pr7118-7187-20260916`; shared target 삭제/이동 없음.
- `cargo build --locked --profile release-test --target-dir target/pr7118-7187-20260916`: CLT에서 성공.
- focused는 [suite/필터 명령 배열](../assets/pr7118_7187_review/focused-command.json)대로 실행했다.
  **54 run / 53 pass / 1 fail**. 실패와 제거 대조군은 #7118 검토에 기록했다.
- source 각 PR의 green CI를 누적 code head의 green CI로 간주하지 않는다.
- fmt check, base 고정 unit tier 검사 통과. fixture 경로 변경으로 생긴 generated suite drift는
  prepare 재실행 후 manifest base 검사 통과; 파생 파일은 commit하지 않는다.
- fresh dev WASM: `DEVELOPER_DIR=/Library/Developer/CommandLineTools CARGO_TARGET_DIR=target/pr7118-7187-20260916 scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev` 성공.
- renderer 비교 명령: `venv/bin/python scripts/visual_sweep.py --file-target <key> <원본> <PDF>
  --rhwp-bin <누적 CLI> [--wasm-pkg pkg] --pages <페이지> --dpi 96 --out <임시 경로>`.
  입력·페이지·hash·실제 CLI/WASM은 [manifest/metrics](../assets/pr7118_7187_review/visual-metrics.json)와 같은 폴더의
  `*-manifest.json`에 보존한다. 임시 root는 `/private/tmp/rhwp-review-open-20260916`이다.
- HWP3 저장: `rhwp convert tests/fixtures/issue_4680/german-legislative-system.hwp
  tests/fixtures/issue_4680/german-legislative-system-open-pr-review.hwp --json`.
  최초 임시 출력과 최종 Git 경로 파일의 SHA-256은 동일하다.
- 전체 release-test / Native Skia 3종 / 새 head의 Clippy 3종은 **미실행**이다.
  선행 focused에서 blocker가 확인되어 최종 PR 제출 게이트를 완료했다고 주장하지 않는다.
  해당 회귀 해결 후 최종 head에서 실행해야 한다.
- #7186: 일반/CI-unit tsc, npm unit 1740 pass/2 skipped, production build,
  command-palette E2E, 영어/한국어 Chrome 실제 실행 통과. 다른 PR의 Rust 검증 대체가 아니다.

## 통합 PR CI 후속 기록

[PR #7197](https://github.com/edwardkim/rhwp/pull/7197)의 `7665912859016c9d46a326e86f000c1a394e4862`에서 [CI Full](https://github.com/edwardkim/rhwp/actions/runs/35071582466) 및 CodeQL·Render Diff·Adapter·Proptest가 통과했다.
코드 변경 없이 최종 검토·오늘할일을 trailing commit으로 보완한다. [개별 검토의 contributor comment 계획](pr_7180_review.md#merge-후-contributor-pr-comment-계획)에 따라 merge 후 확정값과 이슈 상태를 게시한다.

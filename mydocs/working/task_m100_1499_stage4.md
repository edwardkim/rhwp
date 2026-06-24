# 단계별 완료보고서 — Task #1499 단계 4: 임계 확정 + 문서화 + 회귀

- **이슈**: #1499 · **브랜치**: `local/task1499`

## 작업 내용

- **임계 확정**: `--batch samples/hwpx` 실측 → PASS 36건 매칭 노드 변위 **전부 0.0px**.
  부동소수 여유로 `DEFAULT_THRESHOLD_PX = 0.5` 확정 (이슈 제시값 일치).
- **문서**: `mydocs/manual/render_diff_command.md` 신설, `CLAUDE.md` 에 명령 섹션 추가.
- **최종 보고서**: `mydocs/report/task_m100_1499_report.md`.

## 검증

- 전체 `cargo test` 통과 (exit 0, FAILED 없음) — 메모리 룰 `feedback_full_cargo_test_before_pr` 준수.
- `cargo test --lib render_geom_diff` 5건, `cargo test --test visual_roundtrip_baseline` 3건 PASS.

## 범위 밖 (후속)

- XFAIL 20건 개별 렌더 회귀 수정 (별도 이슈).
- 픽셀/SSIM 보조 비교(P1) — 작업트리 `compare_svg_pdf.sh` 자산으로 보존(미커밋).

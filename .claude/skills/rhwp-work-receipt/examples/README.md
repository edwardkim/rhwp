# rhwp-work-receipt 워크스루

실사용 에이전트가 기존 `replay` / `audit` / `lineage` 만으로 노동을
증명하는 표본이다. 새 CLI 는 없다. gym 은 없다.

레퍼런스:

- [replay-attest.md](../references/replay-attest.md)
- [capsule-chain.md](../references/capsule-chain.md)
- [audit-accounting.md](../references/audit-accounting.md)
- [lineage-chronicle.md](../references/lineage-chronicle.md)
- [exit-codes.md](../references/exit-codes.md)
- [pitfalls.md](../references/pitfalls.md)

픽스처 목록은 [../fixtures/catalog.json](../fixtures/catalog.json).

## 목록

| # | 파일 | 단 | 보여주는 것 |
|---|------|----|-------------|
| 01 | [01_attest_three_hashes.md](01_attest_three_hashes.md) | 영수증 | attest 3해시 발급. 사용자 경로 무생성 |
| 02 | [02_verify_expect_output.md](02_verify_expect_output.md) | 영수증 | --expect-output-sha256 제3자 검증 |
| 03 | [03_verify_mismatch_exit3.md](03_verify_mismatch_exit3.md) | 영수증 | reproduced:false = exit 3 판정 |
| 04 | [04_plan_file_vs_inline.md](04_plan_file_vs_inline.md) | 영수증 | 위치 인자 계획 파일 vs --plan-json |
| 05 | [05_capsule_issue.md](05_capsule_issue.md) | 캡슐 | workCapsule 자기완결 교환 |
| 06 | [06_parent_same_folder.md](06_parent_same_folder.md) | 캡슐 | 같은 폴더 --parent 상대 이름 |
| 07 | [07_parent_relative_subdir.md](07_parent_relative_subdir.md) | 캡슐 | 자식 파일 기준 ../root/a.capsule.json |
| 08 | [08_immutability.md](08_immutability.md) | 캡슐 | 포맷터 저장이 parentOk 를 깬다 |
| 09 | [09_same_file_rejected.md](09_same_file_rejected.md) | 캡슐 | --capsule == --parent 거부 |
| 10 | [10_run_then_chain.md](10_run_then_chain.md) | 캡슐 | 실산출은 run, 증명은 replay |
| 11 | [11_audit_all_ok.md](11_audit_all_ok.md) | 감사 | reproducedRate 1.0 |
| 12 | [12_audit_mixed_rate.md](12_audit_mixed_rate.md) | 감사 | 2/3 회계 + exit 3 |
| 13 | [13_audit_non_recursive.md](13_audit_non_recursive.md) | 감사 | 하위 폴더 캡슐 무시 |
| 14 | [14_audit_empty_exit2.md](14_audit_empty_exit2.md) | 감사 | 0개 = 사용법 |
| 15 | [15_lineage_root.md](15_lineage_root.md) | 계보 | 뿌리 3축 null |
| 16 | [16_lineage_two_link.md](16_lineage_two_link.md) | 계보 | parentOk·lineageOk |
| 17 | [17_lineage_deep.md](17_lineage_deep.md) | 계보 | --deep 재실행 |
| 18 | [18_lineage_broken_at.md](18_lineage_broken_at.md) | 계보 | brokenAt 명세 |
| 19 | [19_toolversion_pitfall.md](19_toolversion_pitfall.md) | 함정 | 버전 불일치 선대조 |
| 20 | [20_no_attribution.md](20_no_attribution.md) | 함정 | 누가 했는지는 증명하지 않는다 |

## 공통 규칙

1. 입력은 공개 샘플 경로 또는 워크스루가 밝히는 가명이다. 새 HWP 바이너리를 두지 않는다.
2. 명령 머리는 `replay` / `audit` / `lineage` / `run` 뿐이다. `run` 은 실산출이 필요할 때만.
3. exit 3 은 판정 데이터다. 크래시로 재시도하지 않는다.
4. 캡슐은 발급 후 불변이다. 고치려면 재발급한다.
5. 귀속(누가)과 서명은 이 스킬 1부의 범위가 아니다.

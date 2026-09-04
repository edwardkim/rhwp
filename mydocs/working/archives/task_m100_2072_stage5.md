# Task M100 #2072 Stage 5 - 프론트엔드 이슈 조사 문서 분리

## 목표

`mydocs/tech/` 루트에서 이슈 한정 진단과 시점 의존 스냅샷을 분리한다. 장기 설계·계약 결론이
섞인 문서는 이번 단계에서 이동하지 않는다.

## 이동 대상과 근거

### Issue #2023

- `task_m100_2023_frontend_contract_guardrails.md`
- `task_m100_2023_frontend_diagnosis.md`

두 문서는 프론트 리팩터링 계획의 당시 공개 계약 목록과 휴리스틱 재진단값이다. 장기 canonical
문서가 아니라 계획·보고서가 참조하는 이슈별 근거이므로 `tech/investigations/issue-2023/`에 둔다.

### Issue #2124

- `task_m100_2124_baseline_manifest.md`
- `task_m100_2124_extension_security_snapshot.md`
- `task_m100_2124_font_inventory.md`
- `task_m100_2124_frontend_metrics_scope.md`
- `task_m100_2124_frontend_solid_anchors.md`
- `task_m100_2124_public_contract_snapshot.md`
- `task_m100_2124_smoke_manifest.md`
- `task_m100_2124_wasm_json_schema_snapshot.md`

이 문서들은 특정 기준 커밋·환경·측정 범위에 묶인 baseline, inventory, snapshot, smoke 근거다.
일반 운영 지침이나 장기 기술 계약이 아니므로 `tech/investigations/issue-2124/`에 둔다.

## Redirect 판단

각 기존 파일명을 GitHub issue/PR 본문과 코멘트에서 검색했고 외부 이력 참조는 0건이었다. 따라서
옛 경로 redirect stub은 만들지 않는다. 저장소 내부 참조는 같은 커밋에서 새 경로로 직접 갱신하고,
문서 링크 CI의 `--forbid-path`로 옛 경로의 재유입을 막는다.

## 제외 범위

- `task_m100_1658_*`: 장기 pagination 설계·용량 결론과 조사 기록이 섞여 있어 별도 현행성 감사가 필요하다.
- `task_m100_1772_*`와 그 밖의 root-cause 문서: 재현·원인·대응이 확정됐는지 확인한 뒤 investigations 또는
  troubleshootings로 분류한다.
- 문서 본문의 기술 사실, 프론트 코드, 테스트 동작 변경

## 검증 계획

- 기본 Markdown 링크 검사
- Stage 4와 이번 이동 대상의 이전 경로를 모두 지정한 `--forbid-path` 검사
- Documentation Link Check YAML 문법과 `git diff --check`

## 결과

- #2023의 프론트 공개 계약 목록과 재진단 2개 문서를 `investigations/issue-2023/`으로 이동했다.
- #2124의 환경·범위 의존 frontend snapshot 8개 문서를 `investigations/issue-2124/`으로 이동했다.
- 두 이슈의 기존 파일명은 GitHub 이력 참조가 없으므로 redirect stub을 남기지 않았다. 저장소 내부의
  절대 경로 참조는 새 위치로 직접 갱신했다.
- tech 문서 지도, canonical manifest, Documentation Link Check가 새 이슈별 조사 진입점과 이전 경로
  금지를 반영한다.
- `python3 scripts/check_markdown_links.py`와 Stage 4·5의 이전 경로 19개를 지정한 `--forbid-path`
  검사가 각각 269개 문서에서 통과했다.
- `ruby` YAML 파싱과 `git diff --check`도 통과했다.

## 다음 단계

Stage 6에서 `task_m100_1658_*`처럼 장기 설계와 이슈 조사 기록이 섞인 문서를 개별 내용 감사한다.
확정 원인과 재현·대응이 갖춰진 문서는 `troubleshootings/` 후보로도 함께 판단한다.

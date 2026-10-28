# Task M100 #2072 Stage 6 - 원인 조사와 트러블슈팅 분류

## 목표

이슈별 원인 분석 문서를 단순히 한 디렉터리에 모으지 않고, 확정 원인·재현·대응·검증을 갖춘 지식은
`mydocs/troubleshootings/`으로 승격한다. 그 외의 잔여 분석, 설계 탐색, 당시 기준선은
`mydocs/tech/investigations/issue-####/`에 둔다.

## 분류 결과

### Troubleshootings

- `task_1858_manifestation2_investigation.md`
  - 선언 높이와 실측 높이의 차이로 생기는 하단 앵커 블록 오프셋, PR #1894 수정과 회귀 테스트가 기록되어 있다.
- `task_m100_1772_root_cause.md`
  - HWPX 표 outMargin과 `common.margin` 동기화 결함의 재현 경로·수정·검증이 확정되어 있다.
- `task_m100_1785_root_cause.md`
  - micro-grid 셀의 `apply_inner_margin` 왕복 불일치의 재현 경로·수정·검증이 확정되어 있다.

### Investigations

- `task_m100_1370_divergence_catalog.md`: A3 미주 발산군의 당시 조사·가설 정정 기록
- `task_m100_1772_residual_over28.md`: 일부 원인만 확정되고 잔여 유형은 미조사인 OVER 분류
- `task_m100_1773_record_only_encoding.md`: HWP3 관측으로 재판별된 record-only 인코딩 설계 조사
- `task_m100_1883_diagnosis.md`: 특정 기준 커밋의 SOLID·복잡도 재진단 snapshot

## Redirect allowlist

GitHub issue/PR 검색에서 `task_m100_1772_residual_over28`만 외부 이력 참조 4건을 확인했다.
해당 이전 경로에는 새 위치로만 연결하는 redirect stub을 남긴다. 나머지 6개 이전 경로는 외부 참조가
없어 stub 없이 제거하고 저장소 내부 참조를 새 위치로 직접 갱신한다.

## 제외 범위

- `hwpx_residual_ir_diff_10.md`: #1584 이후 여러 잔여 분류를 함께 담아, 별도 canonical 승격 여부를 감사한다.
- `task_m100_1658_*`: 장기 pagination 설계와 조사 기록이 섞여 있어 내용별 현행성 감사가 필요하다.
- `task_m100_1772_residual_over28.md`의 미조사 유형에 대한 구현 변경
- 문서 본문의 기술 결론 변경

## 검증 계획

- 기본 Markdown 링크 검사
- Stage 4~6의 이전 경로를 지정한 `--forbid-path` 검사
- Documentation Link Check YAML 문법과 `git diff --check`

## 결과

- 확정 원인·대응·검증을 갖춘 3개 문서를 `troubleshootings/`으로 승격하고, 해당 디렉터리의
  진입점과 최근 레이아웃·왕복 충실도 항목을 정리했다.
- 이슈 한정 조사·스냅샷 4개 문서를 각각 `investigations/issue-1370/`, `issue-1772/`,
  `issue-1773/`, `issue-1883/`으로 이동하고 이슈별 README를 추가했다.
- GitHub 외부 참조 4건이 있는 `task_m100_1772_residual_over28.md`에만 redirect stub을 남겼다.
  다른 6개 이전 경로의 내부 참조는 새 경로로 직접 갱신했다.
- 기본 링크 검사와 Stage 4~6의 이전 경로 26개를 지정한 `--forbid-path` 검사가 각각 271개
  문서에서 통과했다. YAML 파싱과 `git diff --check`도 통과했다.

## 다음 단계

Stage 7에서 `task_m100_1658_*`와 `hwpx_residual_ir_diff_10.md`의 장기 설계·스펙 정정·조사
기록을 내용 단위로 감사한다. historical 계획 문서의 `tech/archive/` 후보도 별도 판단한다.

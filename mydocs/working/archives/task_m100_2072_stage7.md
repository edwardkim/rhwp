# Task M100 #2072 Stage 7 - 페이지네이션과 HWPX 잔여 조사 분리

## 목표

`mydocs/tech/` 루트에 남은 #1658 페이지네이션 연구와 #1584 이후 HWPX 잔여 분석을 이슈별
조사로 분리한다. 장기 설계로 보이는 제목이더라도 현재 canonical 문서가 아닌 당시 가설·실험·오라클
기록이면 조사 문서로 보존한다.

## 이동 대상과 근거

### Issue #1658

- `task_m100_1658_capacity.md`
- `task_m100_1658_clipping_capacity_unified.md`
- `task_m100_1658_com_rowheight.md`
- `task_m100_1658_design.md`
- `task_m100_1658_giantcell_residual.md`
- `task_m100_1658_rca_rowsplit.md`
- `task_m100_1658_reset_discriminator.md`

문서군은 특정 법령 별표 오라클, 당시 `devel` 수치, 기각된 가설, 실험 결과, 안전 ceiling을 기록한다.
현재 장기 pagination 계약의 권위 문서로 지정된 곳은 없고, 외부 GitHub 이력 파일명 검색도 0건이다.
따라서 `tech/investigations/issue-1658/`으로 이동한다.

### Issue #1584 이후 HWPX residual

- `hwpx_residual_ir_diff_10.md`

이 문서는 #1584 이후 corpus에서 남은 네 가지 IR_DIFF 유형의 조사와 수정 후보를 묶은 기록이다.
여러 결함의 후속 조사는 필요하지만 단일 장기 스펙이 아니므로 `tech/investigations/issue-1584/`으로 이동한다.
GitHub issue/PR 검색에서 기존 파일명 외부 참조 5건이 확인돼 이전 경로에는 redirect stub을 남긴다.

## 제외 범위

- #1658 조사에서 도출된 현재 코드의 pagination 계약을 새 canonical 문서로 재정의하지 않는다.
- `dev_roadmap_v1_backup.md`의 archive 이동: 별도 historical 참조·외부 링크 감사를 거친다.
- HWPX residual 네 유형의 코드 수정과 이슈 분리

## 검증 계획

- 기본 Markdown 링크 검사
- Stage 4~7의 이전 경로를 지정한 `--forbid-path` 검사
- Documentation Link Check YAML 문법과 `git diff --check`

## 결과

- #1658의 페이지네이션 가설·계측·실험 7개 문서를 `investigations/issue-1658/`으로 이동했다.
- #1584 이후 HWPX residual 분석을 `investigations/issue-1584/`으로 이동하고, 외부 이력 참조 5건을
  보존하도록 기존 경로에 redirect stub을 남겼다.
- 저장소 내부의 기존 경로 참조는 새 위치로 직접 갱신했고, tech 문서 지도와 이슈 조사 지도를 보완했다.
- 기본 링크 검사와 Stage 4~7의 이전 경로 34개를 지정한 `--forbid-path` 검사가 각각 274개 문서에서
  통과했다. YAML 파싱과 `git diff --check`도 통과했다.

## 다음 단계

남은 `tech` 루트의 task 문서는 하나씩 현행성·외부 참조·canonical 관계를 감사한다. historical roadmap의
`tech/archive/` 이동은 이슈 조사 분리와 섞지 않고 독립 스테이지에서 처리한다.

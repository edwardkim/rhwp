# Task M100 #2072 Stage 4 - 시각 검증 클러스터 이동

## 목표

Stage 0~3에서 만든 문서 지도와 링크 안전장치를 실제 경로 이동에 적용한다. 첫 클러스터는 반복적인
검증 절차를 설명하는 `manual/verification/`으로 한정한다.

## 이동 대상

- `visual_verification_governance.md`
- `visual_sweep_guide.md`
- `object_visual_regression.md`
- `roundtrip_fidelity_harness.md`
- `svg_regression_diff.md`
- `visual_clipping_detector.md`
- `hangul_pdf_baseline.md`
- `hangul_page_oracle.md`
- `verify_pi_page_vs_hangul.md`

각 문서는 `mydocs/manual/verification/`으로 `git mv`한다. 새 디렉터리에는 역할과 우선 진입점을
설명하는 `README.md`를 둔다.

## Redirect allowlist

GitHub issue/PR 검색에서 위 9개 기존 경로 모두 외부 이력 참조가 확인됐다. 따라서 이전 경로에는
새 위치로만 연결하는 짧은 redirect stub을 남긴다. 이 목록 밖의 이전 경로 stub은 만들지 않는다.

저장소 내부 Markdown 링크는 이전 경로에 의존하지 않도록 모두 새 경로로 바꾼다. 이동 후 링크 검사와
`--forbid-path`로 루트 안내 문서·`manual`·`tech` 범위의 새 이전 경로 참조를 막는다.

## 분류 경계

- `tech/investigations/`: 특정 이슈의 가설·실험·관찰·미확정 또는 기각 결론을 보존한다. 반복 작업의
  지침이나 장기 계약의 권위 문서가 아니다.
- `troubleshootings/`: 재현 가능한 증상, 확정 원인, 적용 가능한 대응과 검증 방법을 보존한다. 이후
  작업의 사전 점검 자료로 사용한다.
- 조사 결과가 장기 계약·스펙 정정·운영 절차로 확정되면 각각의 canonical 문서에 반영하고, 원 조사나
  트러블슈팅 문서는 근거 링크로 남긴다.

## 제외 범위

- `tech/investigations/`, `tech/archive/`의 실제 생성 및 이슈별 조사 문서 이동
- visual 문서 본문의 기술 사실 변경
- 그 밖의 `manual` 문서 클러스터 이동

## 검증

- 저장소 전체 Markdown의 이동 대상 기존 경로 참조를 새 경로로 갱신했는지 확인
- `python3 scripts/check_markdown_links.py`
- 9개 이전 경로 각각에 대한 `--forbid-path` 검사
- 문서 링크 CI YAML과 `git diff --check`

## 결과

- 9개 시각 검증 문서를 `manual/verification/`으로 이동하고, 역할별 진입점을 `README.md`에 정리했다.
- GitHub issue/PR 검색에서 기존 9개 경로 모두 외부 이력 참조가 확인되어, allowlist를 이 9개 redirect stub으로
  한정했다. 저장소 내부의 기존 경로 참조 47건은 새 경로로 갱신했다.
- `tech/investigations/`와 `troubleshootings/`의 산출물·권위 문서 승격 기준을 manual/tech 문서 지도에 명시했다.
- `Documentation Link Check`는 기본 검사 뒤 9개 이전 경로의 새 Markdown 링크를 `--forbid-path`로 거부한다.
- 기본 검사와 CI 동일 금지 경로 검사는 266개 문서에서 통과했다. 저장소 전체 `mydocs` 확장 검사는 기존
  archive/eng 문서의 494건 오류를 보고하며, 이번 이동이 추가한 오류는 없다.

## 다음 단계

Stage 5에서 `tech` 루트의 이슈 조사 문서를 내용 기준으로 소수 선정해 `tech/investigations/issue-####/`로
이동하고, historical 계획 문서의 `tech/archive/` 후보를 별도 감사한다.

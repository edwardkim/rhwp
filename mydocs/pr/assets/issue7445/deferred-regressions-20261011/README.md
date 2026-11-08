# 회귀 검사 이관 기록 — 2026-10-11

사용자 요청에 따라 아래 문서의 렌더링 회귀를 #7445의 서브 이슈로 이관합니다.
이관은 출력 보정 완료 또는 시각 검증 통과를 의미하지 않습니다.

| 원본 | 서브 이슈 | 처리 범위 |
| --- | --- | --- |
| `samples/table_giant_cell_overfill.hwpx` | [#7812](https://github.com/edwardkim/rhwp/issues/7812) | 마지막 조각·쪽수·행 높이·넘침 및 해당 원본 의존 CLI 진단 검사 |
| `samples/issue5792/2700727_animal_facility_standards.hwpx` | [#7811](https://github.com/edwardkim/rhwp/issues/7811) | 표 분할·행 보존·쪽수·본문/셀 넘침·용지 밖 검사 |
| `samples/2025 행정업무운영 편람(최종).hwpx` | [#7752](https://github.com/edwardkim/rhwp/issues/7752) | HWPX 쪽수·이어받기·문단/표/바탕쪽 배치 |
| `samples/issue1949_giant_cell_nested_tables_perf.hwp`·`.hwpx` | [#7813](https://github.com/edwardkim/rhwp/issues/7813) | 실패한 실물 거대 셀 이어받기4검사의 이관·새 컷 간극 후속 분석 |

## 보존과 실행 대상 분리

- 앞 세 문서는 다른 브랜치가 보정한다는 사용자 지정에 따릅니다. 최신 devel 전쪽 점수를 확보했다는 판정이 아닙니다.
- 모든 원본·독립 PDF는 공유 검사/다른 작업의 입력이므로 기존 경로에 보존합니다.
- `source-before/`는 변경 전 commit `6d55a02401a9be42488fcb7aa7519900aebcecb2`의 검사 원문12개를 바이트 그대로 보존합니다. `.rs.txt`는 Cargo와 통합 suite 자동 수집 대상이 아닙니다.
- 전용 검사 파일은 이 자산 경로로 이동하고, 혼합 파일은 해당 HWPX의 렌더링 assertion/입력만 분리합니다. 저장 모델·parser·serializer·보안 계약과 별도 HWP 입력은 유지합니다.
- `transfer-inventory.json`에 원본/검사 SHA·이전/새 경로·검사 목록을 기록했습니다. 기존20함수와 HWPX E2E 입력1개가 앞 세 문서의 이관 범위입니다. #3930의 혼합1함수는 모델 저장 계약으로 좁힙니다.
- `issue1949/`의 검사4함수는 정확한 원문을 별도로 보존합니다. 실패4함수와 전용 컷/helper 코드를 활성 단위 검사에서 분리했습니다. 상태와 이후 검증은 `transfer-plan.json`을 따릅니다.
- 새 컷 간극을 기존 base 시각 미달과 혼동하지 않습니다. 이관으로 새 회귀가 해결되었다고 보고하지 않습니다.

## 복원 조건

해당 서브 이슈에서 보정한 뒤 동일 원본·독립 Print PDF의 전쪽 Native/fresh WASM 최저90% 이상,
쪽수 일치와 표/문단/내용 소속·누락/중복을 직접 확인하고 관계형 검사로 복원합니다.
검사 원문은 재현에 사용하며 기존 절대 px·쪽수 기대값을 현재 출력으로 단순 완화하지 않습니다.

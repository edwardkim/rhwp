# #6040 최종 검증 증거

최종 판단은 [결과보고서](../../../report/task_m100_6040_report.md)와
[개발 측정 패널 안내](../../../manual/studio_scroll_probe_guide.md)를 따른다.

## PR에 남긴 자료

[final-summary.json](final-summary.json) 한 파일에 다음을 보존한다.

- 같은 환경의 devel before, 보정 전 후보, 최종 제품 후보: 각 2회, 7개 체크포인트 전부.
- 소유 surface pixel·최고 관찰값·visible/focus/offscreen 구성과 오류 상태.
- 4쪽 실문서의 100% 및 34→100% 뒤 20회 이동: 두 변형의 raster/cache 집계.
- 비교 source SHA, WASM hash·빌드 조건, 브라우저·viewport·DPR·측정 한계.

메모리 표본은 시간 성능 비교용이 아니다. 최고 관찰값은 소유 surface peak의 관찰 하한이며
RSS/GPU/native 임시 surface를 포함하지 않는다. warm cache 수치는 누적이고 raster 수는
기록한 20회 이동 구간만 센다. 최종 요약에서 두 반복과 불리한 읽기 화질 비용도 제외하지 않았다.
새 실행은 위 패널로 재현한다. 테스트용 fixture·golden·제품 회귀 테스트는 제거하지 않았다.

## 중간 기록의 로컬 보존

2026-09-10 사용자 승인에 따라 중간 JSON 19개와 기각 실험의 재현 스크립트 1개를 제출 tree에서
제외했다. 계획·단계별 보고서의 판단·실패 기록은 하이퍼워터폴 이력으로 유지했다.
그 문서의 이전 JSON 링크는 이 보존 안내로 바꾸었다. 과거 명령·SHA는 당시 실행 이력이며
현재 checkout에 원시 파일이 존재하거나 해당 실험이 제품에 포함됐다는 뜻이 아니다.

전체 원본은 로컬 백업 branch `codex/issue-6040-evidence-backup-20260910`의
`98b58c356`에 같은 저장소 상대 경로로 남아 있다. 이 branch는 push하지 않는다.
이 백업은 공개 재현 의존성이 아니며, 로컬 저장소를 지우면 함께 잃을 수 있다.

```bash
# 백업이 있는 개발자 로컬 저장소에서만 사용. 작업 tree를 변경하지 않고 한 파일을 읽는다.
git show codex/issue-6040-evidence-backup-20260910:mydocs/working/assets/issue6040-post-stack/baseline.json
```

제출 branch는 원시가 들어 있는 개발 이력을 부모로 삼지 않고 검증한 devel 위에 정리된 tree를
새 commit으로 올린다. 따라서 최종 diff에서만 숨기는 것이 아니라 중간 원시 blob도 새 이력으로
전송하지 않는다. 원 개발·백업 branch는 유지한다.

---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage16_square_table_anchor.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 16단계: Square 중첩 표의 문단 앵커

Issue: #6712. 시작 HEAD: `6968f076d`.

## 분석과 계획

- 한국어 1쪽 Square 중첩 표는 host 줄의 1920HU 전진 뒤에 그려지지만,
  원본 `vertical_offset`은 1602HU다. 2쪽 표도 1920HU 대신 1660HU를 저장한다.
  텍스트 완료 커서가 아니라 문단 시작점을 기준으로 해야 한다.
- 실제 저장 host 줄, 단일 non-TAC Square/Para 표, 양의 크기와 정상 offset,
  다음 저장 본문/컨트롤 앞에 표가 들어간다는 증거를 요구한다. 일반/분할 셀에
  같은 predicate를 적용한다. fixture 이름이나 특정 페이지 오프셋을 사용하지 않는다.
- 원본 2종의 두 쪽에서 Table 노드와 host TextLine 위치를 비교하는 계약을 추가한다.
  focused, 관련 제어군, 새 시각 비교 후 결과를 기록하고 일반 커밋한다.
- 중국어 마지막 테두리는 별도 문제이며 이 단계 성공으로 해결 완료 처리하지 않는다.

## 구현과 검증 결과

- `stored_square_table_anchor_offset`을 추가해 일반/분할 셀의 non-TAC Square 표를
  호스트 문단 시작점 + 저장 offset에 놓는다. synthetic LINE_SEG, 부적절한 anchor/offset,
  다음 문단 영역을 침범하는 표는 기존 경로를 유지한다.
- 실제 두 원본의 1/2쪽, 총 네 중첩 표를 확인한 신규 계약 포함 focused **17 passed,
  148 skipped, 0.246s, exit 0**. 처음에는 suite 번호를 잘못 지정해 0 tests/exit 4였고,
  generated registry의 실제 suite_019를 확인한 뒤 실행했다. 0 tests를 성공으로 집계하지 않는다.
- 제어군 **86 passed, 997 skipped, 89.496s, exit 0**. focused와 중복 합산하지 않는다.
- 한국어 2쪽 그림 표의 y 오차는 약 +3.4px에서 -0.05~-0.07px로,
  footer 그림은 -0.05~+0.01px로 감소했다. 두 원본 모두 2/2쪽이다.
- 중국어 1/2쪽 새 visual sweep을 직접 확인했다. 폰트 및 저장 위치 차이는 남고,
  마지막 테두리가 footer를 가로지르는 현상도 남아 있다. 이 단계에서는 #6712를 닫지 않는다.
- `cargo fmt --all`, `git diff --check` 통과. 전체 회귀/필수 lint는 최종 수정 뒤 실행한다.
  검증 로그와 중간 SVG/PNG/JSON은 `/tmp`에만 보관한다.

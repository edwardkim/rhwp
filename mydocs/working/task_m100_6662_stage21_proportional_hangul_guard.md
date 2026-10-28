---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage21_proportional_hangul_guard.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 21단계: 비례폭 한글 보정 범위 제한

## 분석과 계획

20단계 `4dcdad3e5`에 전체 회귀의 2개 SVG snapshot 실패를 고정했다.
기존 등폭 `함초롬바탕`의 970/1000em 폭을 비례폭으로 취급한 것이 원인이다.

1. 현재 글자의 양수 폭이 em보다 작고, 한글 metric에 다른 양수 폭이 존재할 때만 보정한다.
   특정 문서/폰트 이름이나 임의의 폭 비율 임계값으로 분기하지 않는다.
2. `함초롬바탕`/`HCR Batang` regular/bold 제어군을 추가하고 기존 제목 6개 계약을 유지한다.
3. fmt 후 suite prepare, 제목 계약과 기존 SVG snapshot/lib 제어군을 순차 실행한다.
4. 보고서와 코드를 일반 커밋한 뒤 다음 단계에서 전체 검증과 최종 이미지 재산출을 수행한다.

## 결과

- 비례폭 판정에 다른 양수 한글 폭의 존재를 요구하도록 수정했다. 등폭 한글은 기존 출력 유지.
- 제목 계약 7 passed / 163 skipped / 0.046초. wrapper가 현재 suite_023을 선택했다.
  앞선 직접 suite_013/015 실행은 제목을 포함하지 않아 아래 snapshot 8개 결과로만 센다.
- 기존 SVG snapshot 8 passed / 374 skipped / 0.278초. 실패했던 두 golden도 수정 없이 통과했다.
- SVG lib 제어군 52 passed / 3,834 skipped / 0.101초. fmt/diff check 통과.
- 실제 한국어 원본 export-svg: 2쪽, page 0 overflowCellLines=0, exit 0.
  기존 PartialTable frame의 3.8px LAYOUT_OVERFLOW 진단은 남으며 문자 겹침과는 별도다.
- 새 CLI SHA-256: `65aba36c3f8ce312f046047ff1f051e492b693b1e486ae3fdc552ef74e5c55ec`.
  Chrome 제목 확대를 직접 확인했다. 같은 fallback face의 2배 ink 간격은
  `5, 2, 39, 5, 2, 36, 1, 8, 41, 7`px로 19단계 보정 효과가 유지된다.
  NumPy가 없는 환경에서 첫 픽셀 분석 시도는 실패했으며 Pillow만 사용하는 재실행은 exit 0이다.
- 원본 golden과 기존 레이아웃/페이지 배치 코드는 변경하지 않았다. 이전 실패가 남긴 actual SVG는
  임시 파일이므로 제거한다. 다음 단계에서 최종 코드 전체 회귀와 잔여 필수 게이트를 실행한다.

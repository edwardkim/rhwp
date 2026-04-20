---
name: 트러블슈팅 폴더 사전 검색 의무
description: HWP 직렬화·한컴 호환·파일 손상 관련 작업 전에 mydocs/troubleshootings/ 전수 검색 필수
type: feedback
originSessionId: 67d1cb8f-86d4-4672-b831-a8d028a1cfcf
---
직렬화·한컴 호환·"파일이 손상되었습니다" 류 작업 시작 전에 `mydocs/troubleshootings/` 폴더를 먼저 검색.

**Why**: 같은 함정에 반복 빠짐. #178 두 번째 시도가 한컴 거부를 만난 후에야 `picture_save_hancom_compatibility.md`, `cell_split_save_corruption.md`, `table_paste_file_corruption.md` 의 패턴 (필드 누락 시 오프셋 밀림 / control_mask vs controls 불일치 / char_count vs PARA_TEXT code unit 불일치 / 한컴 정상 파일 바이트 비교 필수) 이 본 시도와 정확히 같은 영역임을 발견. 사전 검색했다면 한컴 호환 한계를 더 빨리 인지했을 것.

**How to apply**: 직렬화·한컴 호환·파일 손상 관련 신규 작업 시작 시 첫 단계로 `Grep` 또는 `Glob` 으로 트러블슈팅 폴더 전수 검색 (예: `Grep pattern="저장|export_hwp|한컴|손상" path=mydocs/troubleshootings`). 관련 문서 1건이라도 나오면 본 작업 진행 전에 모두 정독.

상세: `mydocs/troubleshootings/task178_second_attempt_hancom_rejection.md` §8.5

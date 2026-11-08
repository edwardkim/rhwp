# Task #7957 1단계 — PDF 오류 등록과 공동 처리 준비

- Issue: #7957; 공동 #7958·#7959·#7960·#7961·#7962·#7963
- 작성일: 2026-10-17 (Asia/Seoul)
- 판정: **등록·착수 준비 완료, 계획 승인 대기**

## 완료한 작업

- 동일 오류와 PDF import 관련 열린 issue/PR을 검색했다. 직접 대응 작업은 찾지 못했다.
  닫힌 #7950은 앞선 Mask 지원 사례이며 새 입력의 해결 증거가 아니므로 재개하지 않았다.
- 최초 여섯 유형을 #7957–#7962로 등록했다. 추가 decoded bytes 제보는 #7963으로 등록하고
  대표 #7957의 공동 목록을 갱신했다. 일곱 건의 한국어 본문·assignee edwardkim·v1.0.0·labels와
  OPEN 상태를 GitHub API로 재조회해 확인했다. 실제 PDF는 업로드하지 않았다.
- 현재 Docker bundle을 독립 Worker에서 읽어 실물 6개와 16 MiB+1바이트 합성 거절 입력의
  오류를 모두 재현했다. 전체 elapsed 관측은 단일 진단 호출 시간이며 성능 개선 결과가 아니다.
- 최신 upstream/devel b7a5388034를 fetch하고 `task_m100_7957` worktree를 생성했다.
  기본 작업공간의 미커밋 타입 선언 2개와 Studio 7700은 보존했다.
- [수행계획](../plans/task_m100_7957.md)과 [구현계획](../plans/task_m100_7957_impl.md)에
  실제 소비 경로·절편 순서·반례·시각/최종 게이트와 미검증 범위를 정리했다.

## 실행 증거와 한계

ignored `output/pr-review/issue7957/registration/`의 `inputs.json`, `wasm-reproduction.json`,
`decoded-input.json`, `decoded-wasm-reproduction.json`, `registered-issues.json`,
`registration-verified.json`에 입력/소스/산출물 hash와 실제 결과를 보존했다.
원본 쪽수는 Poppler, extraction 오류 입력의 컨테이너 검사는 qpdf로 대조했다.

새 제품 source·의존성·WASM 변경/재빌드와 수정 후 검사·전쪽 sweep은 수행하지 않았다.
image의 Unsupported 객체, parse 내부 원인, operators/decoded/path의 최초 소진 지점과
정확한 중복 비용은 아직 미검증이다. [구현계획](../plans/task_m100_7957_impl.md)의 F1에서 시작한다.

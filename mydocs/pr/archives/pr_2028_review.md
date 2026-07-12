# PR #2028 검토 — Task #2027: 그림 배치 변경 undo 기록 + tac 해제 line_segs 재계산

- 작성일: 2026-07-07 / **lpaiu-cs — rhwp 첫 PR** (사이클 검증: 누적 1건) / MERGEABLE
- 연결 이슈: #2027 / 출처: HOP(rhwp 기반 다운스트림 앱) 실사용 보고
- CI: 첫 기여자 워크플로 승인 대기였음 → 메인테이너 승인 처리, 재실행 중

## 요지 (결함 2건, 커밋 분리)

1. **studio — 그림 배치 변경·삽입 undo 미기록** (#1320 편집 라우터 계약 위반): 대화상자
   `handleOk`가 wasm 직접 호출 → CommandHistory에 기록 없음. 배치모드/드래그드롭 삽입도
   동일. 수정: 기존 적용 로직을 클로저로 보존한 채 `executeOperation({kind:'snapshot'})`
   라우팅 (services 미주입 환경 직접 적용 fallback), 삽입 2경로는 `insertPicture`
   snapshot — 기존 `pasteImage` 패턴 준용.
2. **core — tac true→false 시 텍스트 문단 line_segs 미복원**: false→true가 키워 놓은
   `line_segs[0]`(그림 높이)가 역방향에서 잔존(저장 파일에도). 수정: 텍스트 보유 본문
   문단에 한해 `reflow_paragraph` + `recalculate_section_vpos` (그림 삭제 경로와 동일
   원리). **기존 else의 내부 가드 동일성 검증 완료** — empty-migrate 함수가 같은 조건으로
   no-op이라 분기 분리 후에도 비대상 케이스 동작 불변.

## 검토 평가

- 첫 PR임에도 진단의 층위 귀속이 정확: studio 기록 누락 vs core 마이그레이션 비대칭을
  분리하고, core snapshot 대조 실험으로 studio 결함을 격리. 회귀 테스트도 양층 모두
  (Rust 4건 + studio 소스 검사 4건).
- 수정 폭 절제: 빈 문단/컨트롤-문자 문단/endnote 가상 문단 기존 동작 유지 명시.
- 참고(비차단): studio 테스트가 "소스 검사 스타일"(문자열 검사)이라 동작 검증은 아님 —
  차후 e2e 승격 후보로 기록.

## 게이트 (devel + PR)

| 게이트 | 결과 |
|---|---|
| GitHub CI | **11 pass / 1 skip** (첫 기여자 워크플로 메인테이너 승인 후) |
| Rust fmt/clippy/tests | 통과 / 0 / **2,929 통과·실패 0** (신규 핀 4건 포함) |
| OVR baseline 5샘플 | **추가 변동 0** (기지 #1936발 3건 동일) |
| Studio tsc / npm test | 0 에러 / **175/0** (신규 4건 포함) |

## 판단

머지 권고 — 첫 PR로서 이례적으로 완성도 높음(층위 분리 진단 + 양층 회귀 테스트 + 절제된
수정 폭). 환영 코멘트에 rhwp 첫 PR 감사 + fork base 동기화 안내 포함 예정.

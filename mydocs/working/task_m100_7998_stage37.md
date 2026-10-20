# #7998 stage37 — 범위 종료·최신 devel 통합과 최종 검증

2026-10-20 작업지시자가 남은 구현을 후속 이슈로 분리하고 P5로 마무리하도록 지시했다.
완료 범위는 현재까지 구현한 의미 표/문단·편집/저장 계약이다. 모든 PDF 작성 구조의 복원이나
후속 #8106·#8107·#8108의 해결을 주장하지 않는다. 이 문서는 진행 중 기록이다.

## 고정 기준과 범위

- 직전 로컬 구현: `77f46b19e` (stage36, 동일 바이트 후보 정식 등록 포함).
- fetch한 원격 devel: `5e2b9bfbf`.
- root devel은 사용자 untracked 피드백 두 파일을 보존한 채 해당 remote로 fast-forward했다.
- 남은 B24 내용 성장 격자 → #8106, 보고서6쪽 glyph 폭 → #8107, 본문1/4쪽 자간 → #8108.
- stage36의158입력169쪽/각 backend, Print32저장본·Studio15경로는 병합 전 증거다.
  병합 후 제품이 바뀐 경로의 검증을 대신하지 않는다. 같은 입력/PDF는 재사용한다.

## 병합 보정

16개 충돌의 양쪽 구현을 대조했다. 제품 경로는 PDF source stroke의 path 투영과 새 metafile
image payload를 함께 유지하고, 명시적 physical dash와 새 HWP dash 표현을 함께 소비한다.
글자 위치는 최신 devel의 언어별 percent 속성으로 통일해 pixel offset과 이중 적용하지 않으며,
paint bounds/fast-lane도 같은 위치를 고려한다. 공통 TAC 셀 projection·source row 소유를 보존한다.
독립 Base14 AFM과 새 한컴 font overlays를 함께 유지한다. CFB의 border/text-clip 보존 stream과
HWP3 재정렬 stream은 공존한다. Layer minor27 충돌은 두 계약을 합친28로 구분한다.

최신 devel의 PDFium 기준 rasterizer를 채택한다. 과거 Poppler TSV는 그대로 보존하고 새 최종
시각 검증과 혼동하지 않는다. stroke 상태가 같은 fill을 바꾸지 않는 독립 계약은 유지한다.
상세 충돌 목록은 ignored `output/pr-review/issue7998/final-integration/merge-resolution.json`이다.

디스크 여유7GiB로 필수 빌드가 어려워, 실행 중 Cargo/rustc/wasm 도구가 없음을 확인한 뒤
24시간 이상 미사용한 debug incremental12개만 정리해12.36GiB를 확보했다. 공유 target의
실행 파일·deps·fingerprint·다른 증거는 유지했다. 경로/시간/전후 여유는 같은 폴더
`incremental-cleanup.json`에 기록했다.

## 최종 게이트

진행 순서는 병합 제품의 컴파일·집중/시각 방향 확인 → 필수 전체 Rust/lint/Native Skia·
Worker/package·실제 Studio와 전쪽 fresh WASM → 기록/본문 준비다. 후속 구현을 시작하지 않는다.
아직 이 단계의 통과·PR 준비 완료를 선언하지 않는다. 원격 push·PR 생성·merge는 별도 절차다.

# PR #2219 검토 — HWPML(HML) 문서 열기 + 의미 보존 저장 (#1157)

- 작성자: cskwork (과거 5건 미머지 CLOSED — 본 건이 실질 첫 머지 후보) / 검토일: 2026-07-12
- base: devel / 70파일 +8,994/−368 / MERGEABLE, CI 12항목 green
- 판정: **B안 — 문서 수정 2건 요청(CHANGES_REQUESTED) 후 merge 수용**, 렌더
  차이 2건은 머지 후 후속 이슈 분리 (작업지시자 옵션 결정)

## 검증 (결합 로컬 게이트)

- Rust 전수 3,121/0 (HML 신규 61건: CLI 12/파서 26/직렬화 23) / fmt·clippy 0
- Studio npm 225/0 + production build / OVR 3샘플 0건
- 구조 격리: parser/hml·serializer/hml 분리, 렌더러·공통 모듈 비접촉
  (document_core에 HmlImportMetadata Option 1개), 파서 구조 규칙 정합
- 저장 계약: 손실 시 구조화 거부(차단 코드+XML 경로), TAIL/SCRIPTCODE 원본
  조각 보존, atomic write + 하드링크 거부, Studio 첫 저장 '다른 이름으로' 강제
- 샘플: ohah-hwpjs(MIT)/osk_filter(UNLICENSE) 라이선스 동봉 — 재배포 적법성 처리

## 실측 차이 2건 (후속 이슈 대상)

1. formatting_table.hml 표 폭 — IR 단계부터 41956HU(148mm 전체폭) 파싱,
   뷰어 참고(약 절반 폭)와 상이.
2. "abc[표]efg" 인라인 병합 — 뷰어는 abc/표/efg 세로 분리.

## 수정 요청 2건 (문서 수준, 반영 후 approve)

1. "정답지" 용어 → 보조 대조 자료 (한컴 뷰어 macOS 출력은 권위 등급 미달).
2. 지원 범위와 제한에 표 폭·표 배치 해석 차이 명시.

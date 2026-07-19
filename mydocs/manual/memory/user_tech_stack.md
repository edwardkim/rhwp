---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: 기술 스택 + 숙련도 + 협업 분배
description: 작업지시자 본인 보유 영역 / Claude 위임 영역 / 본인 직접 결정 영역
type: user
originSessionId: 4861649d-834a-43c6-a262-9f08333360e8
---
## 본인 보유 영역 (Claude 위임 가볍게)
- **Rust** 핵심 — parser/renderer/serializer + WASM bindgen, HWP 5.0 binary + HWPX OOXML 양방향
- **WebAssembly + JS/TS** — rhwp-studio (Vite + TS), rhwp-chrome/Edge/Firefox/Safari 4 브라우저 확장
- **Git workflow** — `local/devel` + `local/task{N}` 패턴, 외부 PR cherry-pick 옵션 분류 (A/B/C/D)
- **빌드 도구**: Docker (WASM 전용), Node.js (nvm v20.20.2 / v22.18.0)

## Claude (Opus 4.7 1M context) 위임 영역
- 코드 분석, 회귀 진단 (bisect, IR dump, SVG 비교)
- 문서 작성: 수행/구현 계획서, 단계 보고서, 최종 보고서, 트러블슈팅, 위키
- 외부 PR cherry-pick 옵션 분류 + 후보 제시 (A/B/C 세분화)
- PR 처리 후속 보고서 일괄 작성 (하루 9건 위임 사례)
- CHANGELOG / README 동기화
- 메모리 ↔ `mydocs/manual/memory/` 백업 동기화
- 페르소나 덤프 작성 (자기 인식 보조)
- 자기검열 grep + 자동 검증 게이트

## 본인 직접 결정 영역 (Claude 회피)
- **시각 판정** (SVG / 브라우저 / 한컴 2022 정답지)
- PR cherry-pick 후보 결정 (옵션 선택)
- 회귀 본질 통찰
- PR base skew 처리 정책 (close + 분리 PR + base 동기화 권장)
- 양 컨트리뷰터 협업 권유 결정
- 외부 정책 (Skia 별도 브랜치, 한컴 2020 정답지)
- 스토어 제출 + 인증 (Chrome / Edge / AMO 직접 업로드)

## Claude → 본인 의존 영역
- 한컴 환경 + Windows 환경 외부 영역 검증
- "이건 회귀가 아니라 원래부터 미완성" 같은 본질 통찰
- PDF는 정답지 아님 (한컴 편집기가 정답지)

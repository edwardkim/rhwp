# Task #1253 Stage1

## 목적

한컴오피스 2024 기준으로 RHWP-studio `주석 모양 > 미주 모양` UI를 먼저 맞춘다. 이번 단계는 렌더 공통 간격 정책을 바꾸지 않고, 사용자가 같은 설정인지 눈으로 확인할 수 있도록 구분선 종류/굵기/색 UI를 보강한다.

## 작업 항목

1. 구분선 종류 선택을 한컴처럼 선 미리보기가 보이는 드롭다운으로 변경한다.
2. 구분선 굵기 선택을 한컴처럼 `0.1mm`부터 `5mm`까지 선 미리보기가 보이는 드롭다운으로 변경한다.
3. 구분선 색 선택을 한컴처럼 현재 색상 칩과 기본 팔레트로 선택할 수 있게 변경한다.
4. 기존 내부 저장값(`separatorLineType`, `separatorLineWidth`, `separatorColor`)과 API 계약은 유지한다.

## 검증 예정

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- TypeScript 변경이므로 필요 시 dev 서버 UI 시각 확인

## 작업 기록

- 2026-06-02: `rhwp-studio/src/ui/endnote-shape-dialog.ts`의 구분선 종류/굵기/색 선택 UI를 미리보기 버튼과 팝업 메뉴로 보강했다.
- 2026-06-02: 구분선 굵기 옵션을 전체 테두리 굵기 코드와 같은 0~15 범위로 맞췄다. 기존 UI는 `5`를 `0.7mm`로 표시했지만, 렌더/저장 매핑 기준 `0.7mm`는 `9`이다.
- 2026-06-02: 내부 저장 계약은 유지했다. Studio JSON의 `noteSpacing`은 내부 `raw_unknown`(`미주 사이`), `separatorMarginBottom`은 내부 `note_spacing`(`구분선 아래`)으로 계속 전달된다.

## 검증 결과

- `npm run build --prefix rhwp-studio`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 43개 통과
- 자동 브라우저 시각 확인은 내부 Browser/Chrome 세션 연결 실패와 데스크톱 상태 조회 타임아웃으로 수행하지 못했다. 한컴 UI와의 최종 시각 비교는 작업지시자 확인이 필요하다.

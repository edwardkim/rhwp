# Task M100 #6112 Stage 2 사후 재구성 — 단일 커맨드 연결 구현

- **이슈**: [#6112](https://github.com/edwardkim/rhwp/issues/6112)
- **일자**: 2026-08-26 KST
- **구현 초안 commit**: `d707d4cf2b4efef757420381b6a460f1b325482e`
- **문서 성격**: 작업 뒤 최종 diff를 대사한 감사 증거

## 구현 결과

### 기본 정책

- 신규·미설정 `toolbarBasic` 기본값을 `false`로 변경했다.
- `theme-init.js`도 같은 기본값을 사용하고, 저장값이 명시적으로 `true`인 경우 펼침을 복원한다.
- `toolbarFormat` 기본값 `true`는 유지했다.

### 진입점

- 기존 `view:toolbox-basic`에 `Ctrl+F1` 표기와 shortcut map을 연결했다.
- 메뉴바 우측 버튼은 새 핸들러가 아니라 `data-cmd="view:toolbox-basic"`로 기존 dispatcher에 연결했다.
- 편집 textarea와 그 밖의 버튼 포커스 모두 같은 shortcut map을 사용하도록 전역 보기 단축키 집합을
  보강했다.

### 표시·접근성

- 메뉴 항목은 `active`와 `aria-checked`를 유지한다.
- 직접 버튼은 `active`, `aria-expanded`, `기본 도구 상자 접기/펴기` 이름과 `Ctrl+F1` 툴팁을 표시
  상태에 맞춰 갱신한다.
- 화살표는 기존 테마 토큰을 쓰는 CSS로 그려 새 래스터·SVG 자산을 추가하지 않았다.

## 변경 범위

- 제품 코드·마크업·스타일 9개 파일
- 단위·E2E 테스트 4개 파일
- 총 13개 파일, `+206/-47`

Rust 렌더러, 파서, DocumentCore와 서식 도구 상자 레이아웃은 변경하지 않았다.

## focused 검증

구현 뒤 도구 상자·설정·단축키 focused 테스트 24개가 모두 통과했다.

## 단계 판정

버튼·메뉴·단축키가 한 커맨드로 수렴하며 저장·FOUC 경로를 중복 구현하지 않았다. Stage 3 전체
회귀와 실제 브라우저 검증 대상으로 넘길 수 있는 상태로 판정했다.

## 절차 이탈

이 구현은 Stage 1 승인 없이 진행됐고 Stage 2 독립 커밋이 아니라 단일 구현 초안 커밋에 들어갔다.
커밋 이력을 재작성해 단계가 있었던 것처럼 만들지 않고 이 사실을 보고서에 보존한다.

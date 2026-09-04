# Task M100 #1445 최종 보고서 — npm README *Ex 안내 + 소비자 편집 API 매뉴얼

- 이슈: #1445 "[docs] npm @rhwp/core README에 *Ex(options) 안내 + 소비자 편집 API 매뉴얼 (#1413 후속)"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1445`
- 작성일: 2026-06-20

## 1. 개요

#1413 의 `*Ex`(options object) API 는 소비자(npm `@rhwp/core` 개발자)용 변경인데, 소비자
문서에 안내가 없어 발견되기 어려웠다. README 보강 + 소비자 편집 API 매뉴얼로 해소.
코드 변경 없음(문서 전용).

## 2. 변경

### `npm/README.md` (= @rhwp/core 패키지 README) API 섹션 보강
- **편집 API** 개요(insertText/createTable 예시 + d.ts 참조 안내).
- **options object 변형(`*Ex`)** 안내: positional vs *Ex 예시(insertPictureEx 하이브리드),
  camelCase 키·선택 키 기본값·바이너리 별도 인자·`rhwp.d.ts` 에서 `Ex(options` 찾기, 0.x
  변경은 CHANGELOG `### API` 기록.

### `mydocs/manual/consumer_edit_api_guide.md` (신규, 소비자용)
- 초기화/문서 객체(createEmpty·로드), 편집 API 분류표, **래퍼(Builder) 패턴 권장**,
  `*Ex` 사용법(셀 텍스트·서식 예시), **0.x 버전 변경 대응**(*Ex·래퍼·CHANGELOG·tsc),
  저장(exportHwp). HwpDocumentBuilder 류 래퍼 개발자 대상.

## 3. 검증

- **자기검열 체크리스트**: 비교/최상급/공공기관 오인 표현 없음 확인.
- **시그니처 일치**: README·매뉴얼 예시(`insertPictureEx`/`insertTextInCellEx`/
  `applyCharFormatInCellEx`/`createEmpty`/`exportHwp`)가 `pkg/rhwp.d.ts` 와 일치.
- 코드 변경 없음 → 빌드/테스트 영향 없음.

## 4. 의의

- 소비자가 README 만으로 `*Ex` 존재·사용법을 발견 가능.
- 편집 API 커스터마이징(래퍼 작성) 개발자가 참조할 매뉴얼 확보.
- #1413(메인테이너 가이드)와 짝을 이뤄 내부/소비자 양쪽 문서 정비.

## 5. 산출물

- 수행계획서: `mydocs/plans/task_m100_1445.md`
- 최종 보고서: 본 문서
- `npm/README.md`(편집 API + *Ex 안내), `mydocs/manual/consumer_edit_api_guide.md`(신규)

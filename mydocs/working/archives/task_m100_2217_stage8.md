# Task M100 #2217 Stage 8 - 저장 로컬 글꼴 조회 캐시

## 목표

`samples/issue2217/20200830.hwp`를 Chrome에서 연 뒤 문서가 표시되었어도
"응답 없는 페이지"가 나타나는 경로를 제거한다.

## 재현 및 분석

- 실제 Chrome의 저장 snapshot에는 685개 face와 8,726개 alias가 있으며 JSON 크기는
  약 288KB다.
- CPU profile에서 `normalizeFontAlias()`가 초기 로드 중 약 653ms를 점유했다.
- `getLocalFontRecords()`는 호출마다 `snapshotRecords()`를 통해 전체 snapshot을 다시
  정규화한다. `analyzeDocumentFonts()`와 CanvasKit/CSS font chain은 문서 글꼴마다
  `resolveLocalFont()`를 호출하므로 전체 alias 재정규화와 전수 탐색이 반복된다.
- 화면은 먼저 표시될 수 있지만, 그 뒤의 반복 동기 작업이 길어지면 Chrome이
  메인 스레드 비응답으로 판단할 수 있다.

## 변경 범위

1. 저장 snapshot을 메모리에 적재할 때 face 목록을 한 번만 정규화한다.
2. alias, PostScript name, full name, family 기반 인덱스를 만들어 `resolveLocalFont()`의
   반복 전수 탐색을 제거한다.
3. snapshot 교체/초기화 시 모든 파생 캐시를 함께 무효화한다.
4. 대형 v2 snapshot에서 반복 alias 해석이 전체 alias 정규화를 반복하지 않는 회귀 테스트를
   추가한다.

## 검증 계획

- local-fonts focused 테스트와 Studio build를 수행한다.
- 이번 변경은 Studio TypeScript 범위이므로 새 Vite 탭에서 `20200830.hwp`의 Chrome 로드
  프로파일을 확인해 font alias 경로가 반복 장기 작업을 만들지 않는지 검증한다.

## 변경 결과

- 저장 snapshot을 적재할 때 `cachedFontRecords`와 alias/PostScript/full name/family-style/family
  lookup을 한 번만 만든다.
- `getLocalFontRecords()`는 캐시된 face 목록을 사용하고, `resolveLocalFont()`는 lookup으로
  정확한 face를 찾는다. 이전처럼 각 문서 글꼴마다 전체 685 face와 8,726 alias를 다시
  정규화하고 전수 탐색하지 않는다.
- snapshot 교체와 초기화는 lookup을 함께 교체해 오래된 face가 남지 않도록 했다.

## 검증 결과

- `node --test tests/local-fonts.test.ts tests/document-initialization-order.test.ts tests/toolbar-local-font-options.test.ts`:
  15 passed.
- `npm run build`: passed.
- 동일한 685 face/8,726 alias 저장 snapshot에서 Chrome CPU profile의
  `normalizeFontAlias()` 점유는 653ms에서 50ms로 줄었다. alias와
  PostScript/full/family key의 중복 정규화도 lookup 생성 중 재사용한다.
- 수정 후 새 Chrome 탭에서 `20200830.hwp` 4쪽을 열고 30초 유휴 상태를 관찰했다.
  10초 CPU profile은 약 9.96초가 idle이었고 rhwp의 반복 장기 작업은 보이지 않았으며,
  Chrome "응답 없는 페이지" 대화상자는 재현되지 않았다.

# #665 재개 — 시험지 ingest 이미지 구현 계획

Issue: #665 (https://github.com/edwardkim/rhwp/issues/665)

## 지시와 범위

작업지시자는 이미지 구현 가능성 보고 뒤 “이슈로 등록한 후 구현을 시작합니다”라고 지시했고,
이어서 기존 이슈 중복을 먼저 확인하도록 했다. 동일 범위 #665를 확인해 신규 등록 없이 재개했다.
계획 작성 후 같은 구현 범위를 재승인받지 않고 지시한 구현을 진행한다. PR·push·merge·재종결은 제외한다.
브랜치: `task/665-ingest-images`, 기준 devel `ea86816b5316dfeeb47ecf20df97abfd4b02da73`.
담당자에 edwardkim을 추가하고 기존 assignee를 보존했다.

## 선행 분석

- 원 PR #668은 closed/미머지이며 실제 반영은 `80535a3be1c21d6a082c9e02d876bcfc4f9ea3d1`이다.
  이미지 placeholder를 #665 후속 과제로 명시했다.
- #666 관련 `f708a3cb9f58e5598a56eef86957a65d890aa9b1`은 DocInfo·페이지·HWPX 직렬화를 보정했다.
- #667의 공유 지문·boxed·머리말/꼬리말, #3355의 무테두리 기본값 및 #3358의 엄격한 입력 검증을 유지한다.
- #665는 completed로 닫혔지만 현재 CLI는 media 디렉터리 존재만 확인하고 image 블록을 텍스트로 만든다.
- #8421은 안내 정정이다. #5319는 스킬 고도화, #8232는 회귀 검사 정비이므로 이번 주 트랙으로 쓰지 않는다.
- 열린 이슈·PR을 build-from-ingest/ingest/이미지/Picture-BinData/exam_paper로 검색했고 별도 진행 중 구현은 없었다.

## 구현 계약

1. CLI에서 제한된 로컬 미디어를 읽는다. 경로 이탈·손상/미지원 형식·픽셀/바이트 상한·메타데이터 불일치·참조 누락을 진단한다.
2. 코어에는 이미지 바이트와 실제 치수를 전달하는 새 fallible API를 추가한다. 기존 텍스트 빌더 API와 입력은 유지한다.
3. 공통 BinData 등록 및 Picture 크기·crop·imgDim 구성 규칙을 공유한다. 파일 시스템 의존성은 코어에 넣지 않는다.
4. above/between/below는 문항의 발문·선택지 순서에 맞게 삽입하고 같은 이미지를 중복 방출하지 않는다.
   공유 지문·보기 내부는 컨테이너의 내용·서식을 유지한다. 이미지 참조의 소유·중복 규칙을 명시한다.
5. 실제 문장 중간 inline은 `paragraph` 블록의 순서 있는 text/image `runs`로 표현하는 호환 확장으로 만든다.
   기존 text/image/boxed는 유지한다. 미지 필드 거부와 정상 텍스트 입력의 무회귀를 검증한다.
6. 이미지 byte 자원은 같은 파일 참조에서 재사용하고 조판은 전체 조립 후 수행한다.
7. 스킬·CLI 매뉴얼·예제·생성기를 실제 구현 계약에 맞춰 갱신한다. 검증된 범위에서만 F18 제한을 해제한다.

## 사양·좌표 근거와 소비 경로

HWPUNIT는 1/7200 inch이다. 기존 schema의 mm 출력 폭과 실제 이미지 종횡비로 크기를 정한다.
기존 등록 helper의 position id/storage id 구분과 Picture의 원본 crop·imgDim 계약(px × 75)을 보존한다.
`해석한 asset/placement → 공통 Picture·문단 줄 소유 → 기존 측정/조판 → backend 출력`을 추적한다.
문장 중간 이미지는 동일 문단의 실제 제어문자 위치와 글자 좌표를 기록하며 저장 LineSeg를 임의 생성하지 않는다.
본문 흐름에 참여하는 그림은 글자처럼 취급하고, 원본 순서·줄 점유·후속 문단을 확인한다.

## 한 단위 완료 게이트

- 기존 텍스트 구조/API·스키마 strict 검증과 새 loader 오류·참조·중복·저장 재열기 검증.
- 실제 CLI → HWPX → 그림/바이트/텍스트 보존 확인.
- 독립 한컴 Print와 Native/fresh WASM 전쪽 TSV·쪽수·내용 보존·확대 판독. 신규 렌더링 회귀는 90% 선행 gate 뒤에만 등록.
- 4종 배치·공유 지문·보기·반복 이미지·정상 텍스트 대조군과 간단한 이미지 다수 성능 확인.
- Rust lint 묶음·전체 release-test·Native Skia 및 fresh WASM 검증은 정본의 변경 범위 게이트를 따른다.
- 단계 결과·실제 실행 명령·source/producer SHA와 남은 검증은 working 기록에 남긴다. 미검증을 완료로 쓰지 않는다.

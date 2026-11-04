# planet6897 통합 검토 — PR #8565 반영 후 PostScript 전용 글꼴 공급

2026-11-04, 기준 `upstream/devel` `1924d371d6757392d063b8dc2e9bbe9a298be817`.

## 원인과 보정

PR #8565(`5551091085418c17d8321c0fee2626365911481b`)는 동일 프로그램의 PostScript 별칭을
Native와 WASM에 공급한다. 그 변경에서 family가 없는 face의 `names[0]`을 별칭을 합치기 전에
읽어 `font_catalog()`가 `IndexError`로 종료됐다. 시스템 Fontconfig의 2,070개 파일 중
KBIZ 한마음명조·나눔바른펜·나눔스퀘어라운드의 일부 설치본이 family ID 1·4·16 없이 실제
PostScript ID 6만 제공해 재현됐다. 다른 정상 설치본까지 제거하거나 제공 목록을 줄이지 않는다.

`font_program_identity()`가 해독한 실제 PostScript 이름을 기존 family 뒤에 합친 다음
`primaryFamily`를 선택한다. 기존 family 우선순위와 정상 ID 1 선택은 유지한다.
글꼴 프로그램을 수정하거나 파일명으로 family를 만들어 내지 않는다. family와 PostScript 이름이
모두 없으면 입력 경로와 face index가 있는 명확한 오류를 낸다.

생산·소비는 `font_program_identity → font_catalog의 cold 캐시 생성 → cached_font_faces의
schema 3 검증 → Canvas 글꼴 공급`이다. 캐시 복원에서도 실제 별칭과 원본 바이트를 유지한다.
제품 Rust 조판이나 Visual Sweep의 실루엣 계산·임계값은 변경하지 않는다.

## 검증

기존 라이선스 확인 테스트 글꼴에서 family 레코드만 제거한 임시 입력을 생성해 검사했다.
시스템 글꼴이나 OTF 파일을 저장소에 추가하지 않았다.

- 새 검사 2건을 수정 전 함수에 실행: 2건 모두 `IndexError`로 실패.
- 수정 후 `ProgramPostScriptAliasTests`: 3건 통과. cold/warm 캐시, 실제 별칭,
  프로그램 바이트 보존과 이름이 전혀 없는 입력의 오류를 확인했다.
- `scripts.tests.test_visual_sweep_png` 전체: 38건 통과.
- 제품 base Native/WASM은 다시 빌드하지 않고 같은 정확한 바이너리·package로 비교를 재개했다.
  이 단위 검사 결과를 전체 문서 시각 통과로 대신하지 않으며, 전쪽 비교는 별도로 완료한다.

로그는 `output/pr-review/planet6897-20261103/logs/`의
`postscript-only-font-catalog-before.log`, `postscript-only-font-catalog-full-tests.log`,
`base-1924-wasm-direct-font-catalog.log`에 있다. 최종 제출에는 해당 입력·페이지의 정식 TSV와
review·overlay를 개별 PR 리뷰에 연결한다.

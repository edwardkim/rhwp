# #8687: 원본 대체 관측 요청의 단일 글꼴 이름 검증

## 문제와 선행 근거

- Issue: https://github.com/edwardkim/rhwp/issues/8687
- 기준: `upstream/devel` = `08963f6825956984d38b0d9392f44777db7107ab`.
- 작업 브랜치: `task/8687-font-fallback-name-validation`.
- #8565의 실제 통합은 `5551091085418c17d8321c0fee2626365911481b`이다.
  `mydocs/pr/archives/pr_8565_review.md`의 공개 공급·기존 선택 보존 계약을 확인했다.
  당시와 현재 `host_font_environment.rs`의 SHA-256은
  `2a7f5c1b7de14f79e78a6de55196330fff5b87bd143fdb72ef825501b8beca9e`로 같다.
- 원본 대체 관측이 `산돌고딕 L\,한컴돋움`을 환경의 치환 key로 넣으면
  단일 face만 허용하는 `FontEnvironment::validate`가 전체 환경을 거부한다.
  문서 선택 생산자인 `document_requests_with_slots`는 이미 같은 이름을 제외한다.
- 열린 PR 검색과 #8687 타임라인을 확인했으며 이 이름 문제를 해결하는 기존 PR은 없다.
  타임라인의 #8692는 유효한 이름의 자기 참조 glyph 대체·남은 선택 규칙 문제다.
  메시지가 같아도 원인과 변경 지점이 다르며 이 작업의 완료 범위로 포함하지 않는다.

## 수정 범위와 소비 경로

`FontFace.name/subst_font.face/alt_name → declared_font_fallback_requests →
prepare_declared_font_fallbacks_with → with_observed_substitutions →
resolved_font_environment → 공통 조판·paint` 경로에서 요청 단계의 불일치를 고친다.
Native PNG 준비와 WASM `getDeclaredFontFallbackRequests`가 같은 요청 목록을 소비한다.

`valid_name`을 crate 내부에 공유하고 원본 이름과 trim한 대체 이름에 적용한다.
이름을 분해하거나 임의의 다른 글꼴로 치환하지 않는다. 정상 요청의 중복 제거·style·
내장 프로그램 우선·명시적 환경 우선·256개 제한과 환경 검증 계약을 보존한다.
원본 FontFace·직렬화·줄 구성 알고리즘은 수정하지 않는다.

## 검증 계획

1. 설치 글꼴·네트워크를 사용하지 않는 API 계약으로 잘못된 원본/대체 이름과
   정상 요청의 공존을 검사한다. 같은 검사로 수정 전 실패와 수정 후 통과를 확인한다.
2. 공개 `header-footer-paper-frame.hwpx`에서 글꼴 선언만 바꾼 진단 입력으로
   실제 Native `export-png --profile screen --scale 1.5` 실패/성공을 대조한다.
3. Native/fresh WASM, 독립 한컴 Print, 정상 대조군, 전쪽 TSV·직접 판독과
   편람 gate를 실행한다. API 계약 검사를 실물 배치의 증거로 사용하지 않는다.
4. 최소 글꼴 격리에서 집중 회귀·release-test 전체·Native Skia 3종을 순차 실행하고,
   fmt·Native/WASM/workspace all-target Clippy·workspace build·manifest를 확인한다.
5. 결과와 SHA·명령을 최종 보고서에 기록하고 로컬 커밋을 만든다.
   원격 push·PR 생성은 저장소의 명시 승인 절차에 따른다.

## 독립 입력과 인쇄 확인

재현 입력은 공개 `samples/header-footer-paper-frame.hwpx`의 ZIP 항목 중
`Contents/header.xml`만 수정했다. 언어별 3번 글꼴을
`산돌고딕 L\,한컴돋움`과 문서 지정 TTF `함초롬돋움`으로 바꾸고,
5번 글꼴 이름을 `함초롬돋움`으로 바꿨다. 본문·줄 정보·개체·용지는 그대로다.

| 입력 | 바이트 수 | SHA-256 |
| --- | ---: | --- |
| 공개 원본 | 15,531 | `8589147dc4a70140d9ccd0c83b899932432541c9db9e714cc59cfdd4c16c67b6` |
| 재현 HWPX | 15,547 | `b84f9c8815b9093135a1e3e630dd8b55aac62916275edac0dc0e8fc61520120e` |
| 독립 Print PDF | 12,546 | `38656e23c496930d4b80405eb71c044ffd8056a930eae82549bd7e2af8723d96` |

HWPX `version.xml`의 저장 제품은 Hancom Office 13.x이며 2024 engine을 명시했다.
2026-11-06에 MCP `start → status → download`를 실행했다.
job `62fcb88a-a642-4462-b593-ec03a329c7ab`의 실제 제품은 `13.0.0.3901`,
`pdf_print_method=0`, `pdf_output_mode=frame_print_to_pdf_ex_one_up`, 1쪽이다.
독립 PDF는 `pdf/issue8687/font_list_name_fallback-2024.pdf`에 보존한다.
PDF의 내장 글꼴은 CID TrueType `HCRDotum`이며 저장 환경을 추측한 값이 아니다.

회귀 검증의 최소 글꼴 격리는 `bwrap`의 별도 filesystem을 사용한다.
`/opt`, 사용자 글꼴, 시스템 글꼴 디렉터리를 숨기고 `RHWP_FONT_PATH`와
`XDG_DATA_HOME`을 해제한다. Fontconfig에는 저장소의 `ttfs/opensource`만 보인다.
기존 검사가 명시적으로 공급하는 라이선스 확인 fixture와
`ttfs/redistributable/happiness-sans/HappinessSansVF.ttf`는 허용한다.
이 fixture의 SHA-256은 `3bbd254dcc5780f7524f9d07af4aa981ba5e3e84cf32d7d4e04301b3943e8694`다.
실물 Print 대조는 별도의 실제 글꼴 공급 환경에서 수행한다.

## 수정 전 재현 결과

기준 제품 코드에서 새 API 계약 4건 중 정상 단일 face 대조군 1건은 통과하고,
잘못된 원본 이름·잘못된 대체 이름·부적합한 목록만 있는 환경 보존의 3건은 실패했다.
최소 글꼴 격리에서 같은 실행 파일을 다시 실행해 1 PASS / 3 FAIL을 확인했다.
원본 이름 사례의 실제 오류는 이슈와 같은
`font environment requires a valid id, single-face substitutions, and bounded distinct glyph fallback faces`다.
대체 이름과 목록 전용 사례는 부적합한 요청이 공급자까지 도달하여 실패했다.
빌드 실패를 결함 재현으로 사용하지 않았다.

실제 기준 Native CLI도 동일 재현 HWPX에서 `--profile screen --scale 1.5`로
종료 코드 1과 같은 환경 검증 오류를 냈으며 PNG는 0개였다.
정상 대조군은 Native 1쪽 raw `91.10204%`다. 기존 #8165 수용 기록과 같은
BOX 높이·기준선 차이가 보이며 이 수정이 만든 차이로 분류하지 않는다.

작업지시자의 순서 보정에 따라 수정 전 WASM 빌드는 중단했다. 그 산출물은
완료된 fresh WASM이나 기준 증거로 사용하지 않는다. 이름 검사를 공통 생산자에
먼저 적용했고, 수정 후 집중 회귀와 같은 Native 빌드의 실제 PNG 확인을 우선한다.
추가 기준 빌드를 위해 구현을 지연하지 않는다.

# #7902 4단계 — 원본 HFT 글꼴과 소수 글리프 원점의 검증

Issue: #7902. 원래 19문서·835쪽 및 양쪽 backend의 전체 무회귀 범위를 유지한다.
[3단계](task_m100_7902_stage3.md)의 보정·문서별 완료 쪽 검사는 `4e2c554b3`에 커밋했다.
기준 `upstream/devel`은 마지막 fetch에서도 `2efefc6a1`로 같아 리베이스하지 않았다.

## 원인과 독립 근거

- 남아 있던 nested HWPX104쪽은 WASM99.93539→99.87312%로0.06227%p 하락했다.
  원본 표의 위 여백140HU를 한 번 예약한 현재 후속 내용은 독립 Print의 실제 위치에 더 가깝다.
  이 이유나98% 유지로0.01%p 하락을 면제하지 않는다.
- Print의0.119935/0.119869 좌표 행렬과595×841pt 용지를 확인했다. 비교 래스터의 위치·배율·DPI를
  바꾸지 않는다. 모든19문서에서 같은 Print 좌표 계약을 확인했지만 검증 도구의 사후 맞춤에는 쓰지 않는다.
- 원본 요청은 HFT `한양신명조`인데 기존 공급은 다른 윤곽의 TTF `H2MJSM.TTF`였다.
  예를 들어 `크`의 HFT 원본 곡선과 대체 TTF의 아래 획이 다르다. 문단의 LineSeg 기준선은
  유지하고 실제 글꼴 윤곽과 raster 원점의 소비를 분리해 검증한다.

## 대상 PDF와 독립적인 원본 글꼴 준비

별도 빈 HWPX에서 HFT 한양신명조로 **U+AC00–U+D7A3 전체11,172자**를 순서대로 작성했다.
원본 대상 문서·대상 정답 PDF의 글자나 윤곽을 이 표본 생성에 사용하지 않았다.
저장 LineSeg를 넣지 않은 이 독립 표본의 실제 한컴 Print 결과는7쪽이다.

- 입력 SHA-256: `8dfd16631ac3cbbc0c56f667f15dd7863f51d7bd62673df3b48e661de7b0f801`.
- Print PDF SHA-256: `86be09a49b9744eb3c1bf38d9f6ae7d86dfda8744f4e5e8cc3e12ed5cc69bc70`.
- MCP job: `f17d491b-6c5b-4026-b594-8623f5a971d3`; engine2020, 한컴11.0.0.9136,
  PrintMethod0 / `hancom2020_pdf_driver_one_up`. PDF의 추출 문자도 전체 알파벳과 순서까지 일치한다.
- 표본 Type3의 원본1000em 윤곽·전진폭을 이동/스케일/획 수정 없이 CFF로 옮겼다.
  ASCII는 넣지 않아 기존 공급 사다리를 계속 사용한다. 결과 글꼴 SHA-256은
  `54763122727f08c3fc83fe9bfd63e1b3dc2a63110ecefcbab0fe266bd4e4360f`다.
- **준비를 마친 뒤** 기존 대상 Print의435개 한글 원본 glyph program과 대조했다.435/435가
  원본 program 바이트까지 동일했다. 이 대조에서 대상 PDF의 데이터를 글꼴에 반영하지 않았다.
- PDF는 로컬 `pdf/issue7902-hft-diagnostic/`에 보존하고 정확한 경로를 `.git/info/exclude`에
  등록했다. 생성 글꼴·전체 원장은 ignored 증적에만 있으며 시스템 글꼴로 설치하지 않았다.

증적 루트는 `output/pr-review/issue7902-render-repair-20261014/hft-independent-specimen/`이다.
`complete-alphabet-provenance.json`, `complete-status.json`,
`original-font/HGSMJ-original.provenance.json`, `independent-source-target-glyph-identity.json`에
입력·원본 윤곽·생성 바이트·Print 실행을 연결한다. 인증 정보는 기록하지 않는다.

## 같은 글꼴을 공급한 선행 진단

아래는104쪽 **선택 쪽 진단**이다. 정식 새 source build 또는 전쪽 무회귀 판정이 아니다.
독립 PDF·DPI·측정/마스크는 그대로이며 base와 candidate에 같은 원본 글꼴을 공급했다.

| 경로 | devel | 글꼴만 복원한 현재 코드 | 소수 원점 보존 진단 |
| --- | ---: | ---: | ---: |
| Native | 99.95853% | 99.94062% | 99.96471% |
| WASM | 99.94545% | 99.92986% | 99.95176% |

Native 진단은 소유한 CLI 복사본의 `SkFont::setSubpixel`에서 baseline-snap 비트를 지우는1바이트
변경이다. 원본 producer·소스·공용 Cargo target은 수정하지 않았다. 이 복사본은 최종 검증에 쓰지 않는다.
WASM 진단은 소유한 export script 복사본에서 Canvas `textRendering=geometricPrecision`을 지정했다.
기존 WASM 바이트는 그대로다. 양쪽 모두 진단 전후 render tree는 동일하다.
정식 구현에서는 paint가 받은 소수 원점을 뒤에서 반올림하지 않는 일반 계약으로 수정한다.

각 producer·변경 바이트·실제 PNG·명령은 `diagnostic-native-no-baseline-snap-provenance.json`,
`original-hft-baseline-snap-diagnostic/`, `original-hft-geometric-diagnostic/`에 보존한다.
최종 source의 Native/fresh WASM을 새로 빌드한 뒤 전체19문서·835쪽을 같은 원본 글꼴 조건의
새 devel TSV와 다시 비교한다. 기존 TTF 대체 공급 결과를 삭제하거나 새 조건의 결과와 섞지 않는다.

## 실행 중인 검증과 남은 작업

- `4e2c554b3`의 전체 release-test nextest는 **10,377 PASS·45 skipped**, exit0이다.
  jobs8·고정 target·실행12 threads, 빌드15분12초·실행1130.340초·전체2045.391초다.
  `rebase-7923-final-full-nextest-result.json`과 해당 log에 보존했다. 이 뒤 backend 변경의 최종
  결과로 바꾸어 보고하지 않는다.
- 진단 결과를 정식 source build 통과로 보고하지 않는다. 원점 소비 수정, 원본 글꼴 준비 절차,
  해당 계약의 수정 전 FAIL/후 PASS 및 정상 대조군, 전체 시각 재검증·직접 판독·필수 lint가 남아 있다.
- 모든 gate를 충족한 뒤 PR 직전 devel head를 다시 확인한다. 변경됐을 때만 리베이스와 재검증을
  수행하고, 사용자가 승인한 PR→CI→merge→옵션2 기록 PR·후속 처리를 진행한다.

## 정식 원점 보존과 선행 회귀 결과

- Native는 글리프를 만드는 두 갈래 모두 subpixel과 baseline-snap(false)를 소비한다.
  생성 번호만 따로 소수 기준선을 보존하던 제한을 제거했다. 저장 LineSeg/배치 원점은 바꾸지 않는다.
- WASM positioned text는 `CanvasTextFont::apply_to`에서 같은 CSS 글꼴과 geometricPrecision hint를
  사용한다. 이 hint가 없는 브라우저는 지원 여부를 검사한다. Studio의 opt-in Canvas metric provider도
  같은 hint로 측정한다. 입력/레이아웃 좌표나 비교 스크립트의 이동·허용치를 바꾸지 않는다.
- 순수 raster 계약은 같은 glyph의1/4px 이동이 잉크 무게중심에도 보존되는지,1px 이동 대조군과
  가로 위치를 함께 검사한다. 독립적인 입력 이동량이 기대값이며 실물 문서 좌표 핀은 아니다.
  기존 Native 라이브러리에 정식 테스트 소스를 직접 링크한 실행은 **이동0으로 의도한 FAIL**,
  소스 수정 후 정식 Native Cargo 검사에서는 **PASS**다. 컴파일 실패를 결함 검출로 세지 않았다.
- `fractional-origin-before-result.json`/`fractional-origin-before-test.log`와
  `fractional-origin-after-native-unit.log`에 연결한다. 기존 공유 deps는 읽기만 했고 standalone
  증거의 모든 새 출력은 ignored 증적 아래에 있다. 최종 Cargo gate는 별도로 진행한다.
- Studio 전체 Node1875개 중1873 PASS·2 skipped, TypeScript noEmit exit0이다.
  원본 글꼴 도구9개 단위 계약도 모두 PASS다. 사용법을 정식 검증 매뉴얼에 연결했다.
- 공개 준비 도구가 만든 HWPX는 실제 한컴 Print에 보낸 입력과 **바이트 동일**하다.
  공개 restore의 CFF/cmap/hmtx/name/OS2 테이블도 선행 공급 글꼴과 모두 동일하다.
  head timestamp/checksum 차이만 있으며 실행 중인 전쪽 비교는 기존 고정 글꼴 바이트를 유지한다.
  `hft-independent-specimen/public-helper-font-table-identity.json`에 실제 해시를 기록했다.
- 현재 source의 Native CLI와 fresh WASM을 순차 빌드 중이다. source 파일·기존 사용자 Studio JS 해시는
  `fractional-origin-source.json`, 실제 종료/producer는 `fractional-origin-build-result.json`에 기록한다.
  원본 HFT를 같은 조건으로 공급한 고정 devel 전쪽 Native/WASM도 각각 시작했다.
  이 실행의 완료·하락0쪽·직접 판독을 확인하기 전에는 PR 준비 완료가 아니다.

소스 측 단위 테스트 총량 정책은 새 raster 계약을 source-side에 추가한4104>4103을 검출했다.
기준선을 올리지 않고 파일/폰트 I/O를 사용하는 이 계약을 `tests/cases/native_fractional_glyph_origin.rs`의
정식 통합 계층으로 옮겼다. source-side는4103으로 유지된다. 파생 suite는 `--prepare`로 다시 만들고
커밋하지 않는다. 정식 case 위치에서 Native 검사와 필수 gate를 다시 확인한다.

정식 통합 위치로 옮긴 뒤 Native renderer·대상 문서 회귀 **129/129 PASS**(실행138.253초)다.
순수 raster 계약도 `regression_suite_019 native_fractional_glyph_origin`에서 실제 실행됐다.
기존코드의 이동0 FAIL과 같은 검사를 유지하며 source-side quota를 상향하지 않았다.
현재 원본 HFT 조건의 devel Native/fresh WASM은 각각19문서835쪽 완료다.
소스에서 빌드한104쪽도 Native99.96471%·fresh WASM99.95176%로 진단과 같은 값이다.
이 선택 쪽 결과로 다른834쪽의 무회귀를 대신하지 않는다.

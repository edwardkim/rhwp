# #8160 10단계: #8231 반영 후 리베이스와 raw100 보존

## 사용자 지시와 고정 기준

- 현재 브랜치 `task/8160-sample16-raw100`을 새 `upstream/devel` 위로 리베이스한다.
- 리베이스 전 head: `9ebf2b3490ee6bb93ee3b7aab91732370fa4940a`, 코드: `404c0de4be62f14542497ddf94f7f2b6af4eaa44`.
- 이전 base: `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`.
- 새 base: `238de6de41a208648f4ade6b494fe433a81c1fce` (#8231), 기능 통합: `3e02fe2749866a8453af3eb3901b9c1a674cccaf` (#8228).
- 충돌은 현재 브랜치의 해결 의도를 우선한다. 리베이스의 ours/theirs 명칭으로 방향을 추정하지 않는다.
  새 base의 비충돌 변경을 보존하고, 충돌 보정 뒤 양쪽 실제 출력을 검증한다.
- 편람 HWP/HWPX 두 파일은 기존 제외 범위를 유지한다. 사양 정오표는 이번에 수정하지 않는다.

## 수행과 통과 조건

1. 리베이스 전 head·source·86개 검사·일반 대조군 261개 인벤토리를 고정한다.
2. 리베이스 충돌을 해소하고 #8160 검사와 잘못 제거된 일반 대조군을 대조한다.
   편람·sample16 공용 제외 정책은 새 devel의 canonical helper와 통합한다.
3. 새 base와 후보에서 실제 Native·fresh 최적화 WASM을 생성한다.
   목표 세 파일 각각 전 64쪽의 raw100·정수 미대응 0/0을 유지한다.
4. #8228의 공통 조판·글꼴·표·원본 프레임 및 편집·저장·Studio 변경은 실제 호출 경로로 보존 검증한다.
   같은 입력/PDF/환경의 새 base와 후보를 대조한다. 조판은 전쪽 TSV·직접 판독으로 확인하며
   페이지별 0.1%p 이상 하락·90% 미만·내용 누락은 보정한다. 목표 세 파일은 더 엄격한 100%를 적용한다.
5. source별 시각 보정 후 정확한 회귀 기대값을 유지하고 필수 fmt·Clippy·workspace·전체 nextest·관련 Native/브라우저 검증을 완료한다.
6. 단계별로 코드와 검증 기록을 일반 커밋한다. 기존 PR에 후속 제출할 자료를 준비하며,
   CI 이후 머지·이슈 종료·후속 처리는 기존 승인 대기를 유지한다.

임시 로그·스크립트·렌더 중간 산출물은 `output/pr-review/issue8160-20261022/rebase8231/`에 보존한다.
리베이스 전 실행은 새 후보의 검증 결과로 표시하지 않는다.

## 리베이스와 1차 보정

- 리베이스 전 계획 포함 head는 `a2b8ee4a2`이며 `refs/backup/issue8160-before-8231`에 보존했다.
- 새 base `238de6de41a208648f4ade6b494fe433a81c1fce` 위로 50개 커밋을 재적용했다.
  리베이스 직후 head는 `6ece02cdf73c8d521dc84bcbcdf8ed245820202e`다.
- 탭은 #8160의 공통 블록 폭과 새 devel의 실제 줄 원점을 함께 소비한다.
  글꼴 프로그램 투영 슬롯과 쪽번호 HFT 판정, WMF 클립과 원본 재생 계약도 함께 보존했다.
- 저장 줄 갱신은 선택 프로그램 경로와 새 base의 본문·목록·이전 여백 처리 경로를 연결했다.
  내부 재귀의 정책 인자는 한 구조체로 전달한다. 실제 보존 여부는 후속 출력 검증으로 판정한다.
- #8160 40개 모듈은 리베이스 전과 바이트가 모두 일치한다. 일반 대조군 261개 중
  자동 적용으로 빠진 `export_structure_envelope_matches_fixture_keys`를 복원했다.
  기존 원본 글꼴 대조군의 가운뎃점 기대값 보강은 새 devel의 독립 Print 근거와 함께 유지했다.
- 자동 수집의 제외 목록은 새 devel의 `deferred_document_regressions.txt`로 통합했다.
  #8160의 명시적 전용 검사는 실행하며, 편람 두 파일은 계속 제외한다.
- 새 devel 시각 보존 범위 35조건/2,041쪽의 입력·독립 PDF 70개 해시는 모두 일치했다.
- 1차 Native `cargo check --features native-skia --bin rhwp`는 exit 0이다. 이후 정책 인자 정리까지
  포함한 최종 빌드·테스트·Native/fresh WASM 출력 검증은 아직 진행 중이다.
  처음 fmt는 오래된 generated suite의 삭제 모듈 참조로 실패했다. 검토 worktree에서 suite를
  다시 준비해 검사하며, 실패한 실행을 통과로 표시하지 않는다.

### 정책 검사 보정

- 새 base의 신규 integration source 정책은 복원한 루트 `tests/diag_1042_version_check.rs`를 거부했다.
  K-water 두 대조군의 기존 검사 본문을 바꾸지 않고
  `tests/cases/issue_1042_k_water_version_read.rs`로 이동했다. sample16 검사를 다시 넣지 않았다.
- 파생 suite를 새 후보로 준비한 검토 worktree의 fmt와 source unit 정책은 통과했다.
  Node 테스트 등록·잠금 검사는 25/25 통과했다. 경로 이동 후 suite 정책은 다시 확인한다.
- raw100 이름으로 한 Python discover는 실제 테스트 0건이었다. 통과 증거로 세지 않는다.

## HWP 2022 18쪽의 WMF 재생 후퇴 분석

- 1차 Native 정수 비교: HWPX 64/64, HWP 2024 64/64 통과. HWP 2022는 18쪽만
  raw 99.99395%, 후보 미대응 15px·기준 미대응 2px였다. 전체 100% 완료로 판정하지 않았다.
- 독립 Print와 리베이스 전 producer·새 producer의 확대 이미지에서 같은 그림의 내부
  글자·선이 흐려졌다. `pi=394, ci=1` 그림 상자와 원점은 양쪽에서 동일했다.
- 원 해결 `4f9ab2b46`의 공통 WMF 인쇄 재생기와 #8228의 장치 클립 변경을 추적했다.
  새 변환기의 상위 `g` 클립 108개 때문에 `images::placement`가 중첩 그림 배치를 거부했고,
  공통 인쇄 자료 전체가 일반 SVG로 되돌아갔다. 실제 SVG의 인쇄 frame·image 표식이
  리베이스 전에는 존재하고 새 후보에서는 사라지는 것을 확인했다.
- 상위 변환이 없는 그룹 클립은 그 그룹에 그대로 남으며 재생기는 자식 image/use만
  교체한다. 이 경로에서는 클립과 JPEG 전달 격자를 각각 보존하도록 배치 인증을 보정한다.
  자체 클립·상위 변환 등 기존 미지원 좌표 계약의 거부 조건은 유지한다.
- 수정 후 실제 Native/fresh WASM 전쪽·새 base 보존 범위를 다시 검증한다.
  문서명·좌표에 따른 예외나 raw100 공차 변경을 추가하지 않는다.

### 보정 전 경계 재현과 보조 검증

- 같은 18쪽의 보정 전 fresh 최적화 WASM에서도 후보 15px·기준 6px 미대응을 확인했다.
  Native의 후보 15px·기준 2px와 함께 공통 WMF 자료의 후퇴를 입증한다.
- 공개 DocumentCore API를 소비하는 합성 WMF 대조군에서, 클립 없음은 인쇄 표식 2개를
  유지하고 상위 클립을 추가하면 인쇄 표식이 사라졌다. 회귀 초안도 수정 전 실제 재생기로
  빌드·실행해 해당 의미 단언에서 실패했다(exit 101). 이 초안은 ignored output에 두며,
  전체 출력 보정 후 정식 등록 여부를 확정한다.
- 새 base에서 유지한 저장 제품 메타데이터 검사의 CLI helper가 자동 적용으로 빠진 것을
  추가 대조에서 확인했다. upstream의 runtime `CARGO_BIN_EXE_rhwp` 우선 helper를 복원했다.
- 시각 검증 Python 191/191, 등록·잠금 Node 25/25, Studio TypeScript 검사를 통과했다.
  Studio 단위 검사는 최초 프로세스 exit 143과 테스트 요약을 구분했고, 독립 프로세스 그룹에서
  재실행해 exit 0·2,572 PASS·2 skip을 확인했다.
- 보정 전 표준 WASM 빌드는 최적화까지 exit 0으로 완료했다. 이 자료를 보정 후 결과로
  표시하지 않는다. 보정 후 Native 빌드와 전쪽 검사로 진행한다.

## Native 전쪽 raw100 고정

- 보정 producer `e01e7540fd4c8a67ce5d1d1e5802170973acb998`에서 세 목표 문서 각각
  전 64쪽을 다시 출력했다. 192/192쪽의 정수 미대응이 양방향 모두 0이며 raw100이다.
- [HWPX TSV](../pr/assets/issue_8160/rebase-8231/native/hwpx.tsv),
  [HWP 2024 TSV](../pr/assets/issue_8160/rebase-8231/native/hwp2024.tsv),
  [HWP 2022 TSV](../pr/assets/issue_8160/rebase-8231/native/hwp2022.tsv)와
  [실행·입력/PDF/PNG 해시 및 정수 비교](../pr/assets/issue_8160/rebase-8231/native/proof.json)를 고정한다.
  이 단계는 Native 결과이며 최종 WASM 완료로 표시하지 않는다.
- 새 devel과의 Native 대조 791쪽까지 0.1%p 이상 하락을 확인하지 않았다.
  전체 2,041쪽과 본문 소유 대조는 진행 중이다.
- 보정 후 fmt·Native/WASM/workspace all-targets Clippy·workspace build·두 정책 검사가
  통과했다. #8160 및 새 devel의 신규 경계 검사들은 release-test로 컴파일·실행 중이다.
- 실행 관찰이 중단됐던 작업은 실제 프로세스 종료를 확인한 뒤 미완료 명령만 재개했다.
  완료된 HWPX/HWP 2024 출력은 다시 실행하지 않고 보존했다.

## 새 devel의 Native 전쪽 보존 확인

- 고정 base와 보정 producer의 35조건·2,041쪽을 각각 실제 산출했다. 전쪽 최저는
  91.66238%이며 0.1%p 이상 하락·90% 미만·쪽수 불일치가 없다.
- paint의 페이지·가용 소유키별 본문 문자 순서는 전 35조건에서 같다. 그림 내부 문자와
  IR 전체 동일성을 뜻하지 않는다. 주요 기호·표·본문 프레임 17쪽을 독립 Print와 직접 판독했다.
- 온새미로 변환본은 기존 #7881의 명시적 Print 글꼴 환경을 최초 대조 명령에서 빠뜨렸다.
  기본 환경의 46쪽 87.14432% 실패를 양쪽 모두 보존하고 기존 환경을 복원해 전 46쪽을 재실행했다.
  후보 최저 94.80197%, base 최저 94.80229%, 46쪽 TSV는 양쪽 100.00000%다.
  설정 파일의 해시까지 비교하며 문서·PDF·문턱과 제품 코드를 바꾸지 않았다.
- Studio의 AI·설정·SFNT·소스 계약은 base·후보 각각 203 PASS다. 글상자 생성·붙이기·제거·
  HWP 저장/다시 열기 3건도 고정한 각 Native 라이브러리에서 모두 통과했다.
- Windows 체크아웃 설정으로 LF 소스·PDF 원본 바이트·심볼릭 링크를 12경로씩 대조했다.
  총 24건 통과이며 실제 Windows OS 실행으로 표시하지 않는다.
- #8160 기본 85건과 새 devel 보존 51건, 총 136건이 release-test에서 실제 통과했다.
  필터 미선택 11,481건은 전체 테스트의 실행·통과로 세지 않는다.
- [전쪽 TSV·입력/producer/명령·직접 판독·비시각 증거](../pr/assets/issue_8160/rebase-8231/preservation/README.md)를 고정한다.
  보정 producer의 표준 최적화 WASM 빌드를 시작했으며, 최종 WASM·전체 Rust·Studio 검증은 계속한다.

## Native·최적화 WASM 전쪽 raw100과 경계 검사 등록

- 보정 producer의 표준 release WASM 빌드는 `wasm-opt -O`까지 exit 0으로 완료했다.
  source dirty=false, 패키지·Studio public 해시 일치와 `WebAssembly.compile`을 확인했다.
  WASM SHA-256은 `c767cd8a3d1550d1310130bf5a5abcc2cefe8714e257896a7b94b8053b12b45c`다.
- 세 목표 문서를 fresh WASM으로 각각 전 64쪽 다시 출력했다. Native와 합친 여섯 실행의
  총 384개 PNG를 정식 검증기가 다시 읽었고, 384/384의 양방향 정수 미대응이 0/0이다.
- [문서별 전체 TSV·빌드·정수 증거](../pr/assets/issue_8160/rebase-8231/README.md)를 고정한다.
  다른 문서의 평균이나 반올림으로 100%를 판정하지 않았다.
- 이 결과를 확보한 뒤 `tests/cases/issue_8160_wmf_clipped_print.rs`를 정식 등록했다.
  기존 #8160 40개 모듈의 내용은 유지하고, 중복되지 않는 상위 장치 클립 경계 1건을 추가했다.
  합계 41개 모듈·87개 검사(기본 86개, Native Skia 전용 1개)다.
- 정식 본문은 최종 검사에서 같은 내용으로 수정 전/후 frozen 라이브러리에 연결해
  결함 단언의 실제 FAIL/PASS를 재확인한다. 최종 fmt·세 Clippy·workspace·전체 nextest·
  Native Skia 3종과 관련 Native 전용 검사도 실행한다.
- 새 devel의 WASM 전쪽 대조와 실제 Studio/CanvasKit 소비 검증은 계속 진행 중이다.
  임시 스크립트·진단 파일은 ignored output에 두며 새 테스트 원본만 정식 경로에 등록한다.

## 실제 브라우저 소비와 최종 WMF 경계 재현

- 최종 검사 본문 SHA `4a156823671a3ff2c7bc3bbaade187abfd59f5cb17b427accef4c957b7e68037`를
  보정 전과 보정 후 frozen Native 라이브러리에 각각 연결했다. 두 컴파일은 exit 0이며,
  같은 본문의 보정 전 실행은 의도한 인쇄 frame 단언에서 exit 101, 보정 후는 exit 0·1 PASS다.
- 목표 문서 주요 8쪽을 Native·최적화 WASM·독립 Print로 직접 판독했다. BCP 2022 문단의
  원본 두 줄, 2024 원본의 해당 한 줄, 수식의 분수·괄호를 확대 대조했다.
- Canvas 브라우저 계약 8건과 CanvasKit crop 네 모드를 통과했다. 실제 WMF 18쪽·35쪽의
  네 영역은 최적화 WASM의 CanvasKit software backend에서 정수 미대응 0/0이다.
  readiness·image 실패 없음·fallback 미사용 및 실제 초기화 응답 WASM SHA를 확인했다.
- Studio 후보 5종과 새 base 3종을 실제 브라우저에서 통과했다. 초기화 응답 자체의 SHA를
  확인했으며 별도 파일 재요청으로 실행 신원을 대신하지 않았다.
- [실제 소비·이미지·해시·종료 코드](../pr/assets/issue_8160/rebase-8231/consumer/README.md)를 고정한다.
  CanvasKit 판정 범위는 두 그림 영역이다. 전체 Rust·Native Skia와 새 devel의 WASM 전쪽
  보존은 계속 진행하며, 이 단계의 통과를 나머지 필수 항목 완료로 표시하지 않는다.

## 새 devel의 WASM 전쪽 보존 확인

- 새 base와 보정 producer의 표준 최적화 fresh WASM으로 35조건·2,041쪽을 각각 산출했다.
  후보 최저 91.70207%, 최대 하락 0.09506%p이며 0.1%p 이상 하락·90% 미만·누락·쪽수 불일치가 없다.
  Native의 대응 최대 하락은 0.08523%p다. 목표 sample16 세 문서의 각각 64쪽은 별도의 더 엄격한
  raw100·정수 미대응 0/0 계약을 계속 만족한다.
- WASM의 전쪽 PNG export receipt 해시·render tree 수를 확인했고, 실제 본문 소유키가 있는
  전 35조건의 페이지·가용 소유키별 본문 문자 순서가 같다. 주요 17쪽도 새 base·후보·독립 Print로
  직접 판독했다. 그림 내부 문자와 IR 전체 동일성을 뜻하지 않는다.
- 415쪽 교육과정 문서는 기본 CDP timeout 실패 후 지원 옵션 `protocolTimeoutMs=3600000`을
  양쪽에 적용했다. 임시 wrapper의 multiprocessing 재진입 오류는 `__main__` 보호로 고쳤고,
  실제 2쪽 실행의 exit 0을 확인한 뒤 양쪽 전 415쪽을 재산출했다. 두 실패 시도의 자료는 보존하고
  통과에 합산하지 않았다. 제품·문턱·비교기를 바꾸거나 임시 호출 스크립트를 커밋하지 않았다.
- 온새미로 변환본의 WASM도 기존 #7881 Print 글꼴 환경을 양쪽에 동일하게 적용했다.
  양쪽 최저 94.60320%, 46쪽은 양쪽 100.00000%이며 제목·프레임·바닥글을 확대 대조했다.
- [전쪽 TSV·신원·실제 페이지 대조·직접 판독](../pr/assets/issue_8160/rebase-8231/preservation/README.md)을 고정한다.
  최종 전체 nextest는 exit 0·11,573 PASS·45 skip이며 #8160 기본 86건도 모두 포함한다.
  Native Skia Clippy는 통과했고 나머지 Native Skia 필수 검증을 계속한다.

## 최종 로컬 필수 검증 완료

- 최종 검사 원본은 `949195dba653cdef8d8fa1506131f5db2f669e78`이다. 보정 producer
  `e01e7540fd4c8a67ce5d1d1e5802170973acb998`와의 런타임 동일성 및 이후 커밋이 검증 증거·문서만
  추가했음을 확인했다. fmt·Native/WASM/workspace Clippy·workspace build·두 정책 검사와
  전체 nextest·Native Skia 필수 검증, 총 14개 단계가 모두 exit 0이다.
- 전체 nextest는 11,573 PASS·45 skip이며 #8160 기본 86건을 모두 실제 실행했다.
  Native Skia 라이브러리 4개 target은 합계 4,040 PASS·12 ignore다.
- Native Skia 공식 누락 그림 2건, 직접 PDF 출력 14건, #8160 문단·이중 테두리 3건을 각각
  정식 suite wrapper로 실행해 모두 통과했다. #8160은 기본 86개와 Native 전용 1개, 총 87개다.
  해당 Native suite의 3건을 모두 새로운 고유 검사로 합산하지 않는다.
- [실제 명령·종료 코드·로그 해시·범위·전체 결과](../pr/assets/issue_8160/rebase-8231/validation.json)를 고정한다.
  후속 문서 커밋 때문에 완료한 검증을 다시 실행하거나 producer 이름을 바꾸지 않았다.
- 원격 `devel`은 검증 base `238de6de41a208648f4ade6b494fe433a81c1fce`와 같음을 다시 확인했다.
  기존 PR #8188의 원격 head는 `021c486d1eaee17920a2b3e82ae896115b847669`이며 아직 갱신하지 않았다.
  승인받은 뒤 같은 원격 작업 브랜치에 lease를 지정해 갱신하고 정확한 새 head CI를 확인한다.
  본인 PR 검토 기록·오늘할일의 확정은 새 code head CI 성공 뒤에 진행하며 머지·이슈 종료는
  기존 승인 대기를 유지한다.

# #8160 단계 5 — 최종 커밋 전쪽 재검증과 PR 제출 증거

## 기준과 승인

- 작업 브랜치: `task/8160-sample16-raw100`.
- 검증 source: `0c3148ab54b3a4ca1bdc969e4d9283265625feec`.
- 재확인한 `upstream/devel`: `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`.
- 사용자 지시의 최신 순서는 전쪽 TSV 재검증, 잘못된 회귀 정정, 전체 검사 완료 후 PR이다.
  사용자는 검사 완료를 조건으로 자동 승인과 머지 후속 처리까지 승인했다. 이전의 별도 PR 승인
  대기 기록은 당시 상태이며 현재 제출을 다시 승인받아야 하는 조건으로 사용하지 않는다.
- 정오표와 브랜치 명명 규칙 설명 문서는 다른 PR의 작업으로 보존했다.

## 최종 384쪽의 실제 재산출

세 원본과 새 MCP Print PDF는 [생성 영수증](../pr/assets/issue_8160/mcp-reference-refresh.json)의
동일 바이트를 사용했다. 각 문서는 64쪽이며 HWPX·2024 저장본은 2024 compat,
2022 저장본은 2022 compat와 실제 Hancom 2020 Print 기준이다.

| 원본 | Native | fresh 표준 WASM | 양방향 미대응 |
| --- | --- | --- | --- |
| HWPX | 64/64쪽 raw 100% | 64/64쪽 raw 100% | 모든 쪽 각각 0/0 |
| 2024 HWP | 64/64쪽 raw 100% | 64/64쪽 raw 100% | 모든 쪽 각각 0/0 |
| 2022 HWP | 64/64쪽 raw 100% | 64/64쪽 raw 100% | 모든 쪽 각각 0/0 |

- [384쪽 정수 비교 원문](../pr/assets/issue_8160/final-384-raw100.json)은 TSV를 읽지 않고 실제
  Native/Canvas2D PNG 384개와 PDF PNG를 다시 열어 양방향 미대응 픽셀 수를 계산했다.
  결과는 **384/384 통과, 실패 0, 모든 쪽의 두 정수 0**이며 exit 0, 17.89초다.
- DPI 96, RGB 최소 채널 임계값 232, 대칭 2px 대응 반경, 흰색 padding을 고정했다.
  색상 경계 보정이나 허용 반경을 바꾸지 않았다.
- [Native TSV](../pr/assets/issue_8160/tsv/final/native/)와
  [WASM TSV](../pr/assets/issue_8160/tsv/final/wasm/)는 각각 세 파일·192행이다.
  각 파일의 실제 페이지 목록은 1~64, raw 최저값은 100%다.
- [동일 글꼴 base TSV](../pr/assets/issue_8160/tsv/same-font-base/)는 생산 source `605ccfd92`의
  실제 보존 출력을 사용한다. 원본 글꼴·PUA 공급 조건을 맞춘 대조이며 착수 시 다른 글꼴 조건의
  기존 `tsv/base` 기록을 덮어쓰지 않았다. 384개 쪽 모두 개선 또는 유지, 하락 0이다.
  base의 관련 PNG 생산 코드와 `d9fd85ed5` 사이에 렌더링 변경이 없다는 선행 분석을 유지한다.
  이 과거 생산 SHA를 새 devel에서 빌드한 SHA로 바꾸지 않았다.

최종 실행 파일과 패키지의 실제 해시는 다음과 같다.

| 산출물 | SHA-256 |
| --- | --- |
| Native `rhwp` | `9466f68ece8e197d96259f16bc6085f1c23b35ac421a88ad15a135340f96bc3b` |
| Native 실제 링크 rlib | `1655395b512c4f3d17f779857e7f3084410a24e8176fd537b4cb3e838eb98b42` |
| 표준 web WASM | `a814fb7a95f29f6d9de397f72a756ec19ce0fa2b7c151e00e879d58c80db06d4` |
| web glue | `00dff351e81fe1cd05d108e928d5f8ab49127ed8aeac9498b1f128c26065a666` |

[Native 빌드](../pr/assets/issue_8160/final-native-build.json)는 실제 Cargo artifact의
`deps/librhwp.rlib`와 실행 파일을 동결했다. 빌드·복사는 exit 0, 592.861초다.
[WASM 빌드](../pr/assets/issue_8160/final-standard-wasm.json)는 release·standard·rlib+cdylib·
manifest LTO와 실제 `wasm-opt -O` 실행을 기록한다. 빌드·최적화는 exit 0, 2,080.277초이며
실험용 cdylib-only 또는 최적화 생략 경로를 사용하지 않았다.

동결 pkg·빌드 worktree pkg·Studio public의 두 산출물 해시를 비교하고 `WebAssembly.compile`을
실행했다. wrapper가 갱신한 검증 worktree의 tracked `public/rhwp.js`는 동일 해시의 생성 출력이다.
이 파일을 source 변경으로 커밋하거나 주 저장소의 개발 서버에 복사하지 않았다.

## 자동 회귀의 수와 범위

- 집중 검사는 **25개**다. 새 전쪽 검사 3개와 기존 계약·대조 검사 22개를 실제 실행했다.
- 새 [전쪽 case](../../tests/cases/issue_8160_all_pages.rs)의 세 함수는 각각 64쪽을 순회해
  세 문서 **192쪽 전체**의 독립 내용 앵커 572곳, 원본 소유·문자 범위·순서·중복·쪽 소속을 검사한다.
- 모든 쪽 첫 앵커의 가로 원점과 각 독립 기준선을 ±2px로 보호한다. 중요 쪽의 셀 624개와
  그림 6개, 합계 630영역의 네 변을 **2,520개 ±2px assertion**으로 보호한다.
- 6쪽 산식의 원본 스크립트·첫 가시 글자 원점·분수선 높이와 길이, 38쪽 꼬리말 `- 36 -`의
  단일 소유·내용·좌표를 검사한다. 기본 글꼴 환경의 수식 내부 분수선 양 끝까지 고정했다는
  주장으로 확대하지 않는다. 원본 글꼴을 공급한 실제 paint는 위 384쪽 PNG 검사로 확인했다.
- 기존 BCP 검사 안에서 원본 LineSeg `[0,64]`, 두 줄 내용과 소유·후속 문단·비겹침·독립 좌표를
  보호한다. 2022 원본의 두 번째 줄 `립`을 한 줄로 합치는 제품 예외는 제거된 상태다.
- 기존 HWPX 18쪽 picture-row 검사는 공통 helper로 병합해 세 문서에 적용했다. 별도 중복 함수
  하나를 제거해 등록 함수 수의 순증은 **2개**다. 다른 바이트의 기본 HWP5 및 parser/save/corpus
  검사는 다른 보호 범위이므로 유지했다.
- 세 JSON은 `include_str!`로 읽는 정식 독립 기대값이다. 좌표 추출 임시 도구는 private 증거에
  보존하며 source PR에 넣지 않는다. 자동 수집의 28 suite+20 예외, 48 target 예산도 유지했다.

CI의 자동 보호는 위 내용·관계·좌표 검사다. 원본 글꼴 공급과 실제 PNG가 필요한 384쪽 paint
검사는 커밋한 `scripts/verify_sample16_raw100.py`와 보존 산출물로 실행하는 별도 검증이다.

## 전체 검증 결과와 실행 순서 보완

[최종 검증 원문](../pr/assets/issue_8160/final-validation.json)에 명령·경로·시간·종료 코드와
실제 summary를 보존했다. 모든 Cargo 실행은 주 저장소의 공용 `target/pr-review`를 순차 사용했다.

| 검사 | 실제 결과 |
| --- | --- |
| fmt, Native/WASM/workspace Clippy, workspace build | 모두 통과 |
| suite·source-side 정책 | 통과; source-side 4,101개·296모듈, 통합 target 48/48 |
| 관련 집중 검사 | 25/25 통과; 선택한 case별 실행 수까지 대사 |
| 전체 nextest 첫 실행 | 11,365 통과, 기존 제외 45; 실행 1,688.354초 |
| release build | 통과; 1,904.954초 |
| release lib | 3,984 통과, 기존 ignore 12; 빌드 포함 2,078.524초 |
| 정본 순서의 전체 nextest 재실행 | 11,365 통과, 기존 제외 45; 실행 2,459.365초 |
| Native Skia lib 재실행 | 4,041 통과, 기존 ignore 12 |
| Skia missing picture / direct PDF | 각각 2개 / 14개 통과 |
| doc test | 8 통과, 기존 ignore 3; 재실행도 통과 |
| TypeScript / Studio Node | 통과 / 2,361 통과·기존 skip 2; 정본 순서 재실행도 통과 |
| source archive 사전 검사 / Python | 9개 후보 source 검사 통과 / 18개 통과 |
| 실제 font-metric-gen Node | 주 저장소에서 5/5 통과 |

큰 변경의 정본 순서를 다시 확인해 release build/lib 뒤에 전체 nextest·Skia 3종·fmt·diff·
Clippy·doc·TypeScript·npm을 순차로 재실행했다. 앞서 통과한 실행도 과거 결과로 보존했다.
Studio의 두 skip은 기존 `pkg-node` 부재 검사이며 실제 fresh web WASM 실행과 구분한다.
source archive의 `publish_ready=false`는 비게시·미래 버전의 기존 의도된 blocker다.
실제 registry 게시를 수행하지 않았다.

검증 도구의 출력 경계와 checkout 기준도 바로잡았다. source 사전 검사는 주 저장소를 명시해
해당 저장소의 `output/` 아래에 생성했다. 공유 target의 실행 파일을 worktree 밖에서 호출하면
주 저장소를 기준으로 찾는 font generator 계약은 주 저장소에서 정식 5개를 실행했다.
초기 worktree 시도가 바꾼 검증 전용 생성 파일은 보존 후 commit 바이트로 정확히 복원했다.
[worktree 내부의 canonical 출력 보호](../pr/assets/issue_8160/final-fontgen-worktree-control.json)도
595-entry guard의 실제 거부·전후 해시 동일·metadata 미생성으로 확인했다. 실패 로그는 private
증거에 남기고 제품 기대값을 변경하지 않았다.

## 브라우저 실제 소비와 확대 판독

[실제 WASM 소비](../pr/assets/issue_8160/final-wasm-consumption.json)는 초기화 중 받은 응답의
바이트를 CDP `loadingFinished` 시점에 읽어 보존했다. 별도 진단 fetch가 아니며 관측한 모든 응답의
상태 200·해시가 빌드와 일치한다. 초기 시도의 DevTools cache eviction 실패는 별도 보존했고,
실패 응답을 무시하거나 성공으로 바꾸지 않았다.

[CanvasKit 결과](../pr/assets/issue_8160/final-canvaskit-regions.json)는 software backend의
실제 18·35쪽 WMF 영역에서 양방향 미대응 0이다. 준비 전후 image failure·렌더 오류·예상 밖
unsupported op·숨은 Canvas2D overlay가 없고 readiness와 완료 상태도 통과했다.

[직접 판독 기록](../pr/assets/issue_8160/final-direct-review.json)과
[Print/Native/WASM 확대 이미지](../pr/assets/issue_8160/final-review/)에서 BCP 두 줄,
원본 글꼴·문단 간격, 평가표·두 산식, 18·35쪽 WMF와 앞뒤 제목·본문, 38쪽 표·꼬리말,
58쪽 서식 격자를 직접 확인했다. 내용 누락·새 겹침·잘못된 원본 줄 소유가 없었다.
14개 표준 review/overlay의 쪽 번호도 실제 입력 쪽과 대사했다.

일부 표선·밑줄의 래스터 명암과 배경 해칭·안티앨리어싱 차이는 확대 이미지에 남아 있다.
이번 100%는 고정한 232 임계값·2px 내용 실루엣 계약이며 위 차이를 RGB 동일성으로 표현하지 않는다.
기본 글꼴을 쓰는 CI 좌표 검사와 원본 글꼴 공급 환경의 paint 검증도 구분한다.

[영향 대조군](../pr/assets/issue_8160/final-controls.json)은 Native/WASM 합계 12개 모두
source33 실제 출력 이상을 유지했다. 팔레트 네 문서·기본 HWP5 목차는 양쪽 100%이며,
별도 KTX 2쪽은 종전 Native 99.70954%·WASM 99.68345%로 동일하다. 세 목표 문서의
384쪽 100% 결과와 이 별도 대조군의 결과를 섞지 않았다.

## 보존과 제출 순서

- 원본·새 PDF·글꼴 프로그램과 필요한 설치 글꼴, 동일 글꼴 base 출력, source31 결함 검출과
  source33 대조 출력, 최종 실행 파일·WASM·384쪽 PNG·명령·로그·해시를
  `/home/tsjang/rhwp-local-verification/issue8160`에 SHA 대사 후 보존했다. 디렉터리는 0700이다.
  원본 글꼴 바이너리를 Git에 넣지 않았다. 임시 출력 경로와 영구 보존 경로를 구분한다.
- 제출용 증거·계획·이 기록만 일반 commit으로 고정한다. 검증한 source `0c3148ab5` 이후
  제품·검사 코드가 바뀌지 않았음을 diff로 확인한다.
- 승인된 원격 순서는 upstream 작업 head push → Open PR → 정확한 code head CI 성공 →
  review·오늘할일 trailing 문서 → 정확한 trailing head CI 확인 → 일반 merge → 후속 처리다.
  이 기록 작성 시점에는 PR 생성·원격 CI·merge를 완료했다고 기록하지 않는다.
- 후속 정리에서는 사용자 개발 서버와 공유 target을 보존한다. `output/`의 다른 작업
  `7445-ubuntu-ted2-assessment-20261024`와 공유 font cache는 소유·사용 여부를 확인해 처리하며,
  확인되지 않은 자료를 삭제해 전체 정리를 완료한 것으로 보고하지 않는다.

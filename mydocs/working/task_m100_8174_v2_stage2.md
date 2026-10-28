# #8174 v2-S02 — 기본 기능·Native

- 승인: 2026-10-24 사용자 「다음 절편 진행을 승인합니다.」.
- 계획: [v2 구현·검증 계약](../plans/task_m100_8174_impl.md#범위-v2--기본-기능과-환경별-자원-정책).
- 시작: S01 `472e38d4f4e4fba201c2ccaa8422d9796a3e9536`의 로컬 커밋·검증 완료를 확인했다.
- 최신 기준: `upstream/devel` `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`.
  기존 로컬 작업을 rebase한 S02 시작 source는 `f9d12b3a17`이다.
- 상태: V2-S02 로컬 구현·필수 검증·기록 완료. GitHub 게시와 v2 전체 완료는 구분한다.

## 승인된 절편

| 항목 | 구현·검증 범위 |
| --- | --- |
| F01·F02 | 기본 JBIG2·JPX, 기존 요청 예산 보존과 Indexed 내장 mask·CMYK alpha 조합 확대. 명시적인 개발용 제외 빌드도 유지한다. |
| F03 | ICC 실제 색 변환의 요청 예산·profile 검증·할당·cache·alpha 보존. Native/WASM에서 같은 CMM을 사용할 기반을 둔다. |
| F04 | 기존 무제한 legacy scan을 재활성화하지 않고 요청 예산을 받는 xref 복구를 constructor에 연결한다. 문자열·주석·stream의 가짜 object와 incremental update를 대조한다. |
| F05 | SVG 글리프의 공통 유효 artwork·gradient·clip·opacity·내부 참조·내장 이미지 표현을 확장한다. 외부 자원의 resolver/host 경계를 구분한다. |
| F06 | 공통 링크 목적지 분류·원본 보존·편집·Native PDF 출력을 연결한다. 웹·메일·내부 목적지·파일·등록 앱을 구분하며 자동 실행하지 않는다. |

브라우저 Worker·Studio의 실제 적용은 S03·S04에서 검증한다. F05·F06의 공통 결과는 이번에
준비하되 해당 제품 적용을 완료로 앞당기지 않는다. MD05의 남은 감사·Native의 다른 진입점과
다른 OS, S05의 실제 포장·독립 소비도 승인 범위에서 계속 추적한다.

## 새 base의 선행 분석과 보존

| PR → 실제 반영 | 이번 대조·검증 범위 |
| --- | --- |
| #8178 → `7f1787708e64433a7c853b9fdda5fc94b27033a1` | Chrome 선택 실행 보고서의 원본 상태·증적 검증·공유 경계다. 제품 문서 엔진 수정과 분리하며 기존 보고서 계약을 보존한다. |
| #8191 → `60972534680a0c5e51e54ec189b92e252e81a867` | 기존 완료 문서의 공통 조판·그림/글꼴·웹폰트·Native 출력과 JPEG encoder fork를 보존한다. 원 PR·정식 리뷰의 source·독립 Print·실제 소비 범위를 읽었으며 새 base와 후보의 영향 경로를 실제 대조한다. |
| #8195 → `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a` | Studio 명령 실패·복구·호스트 통지의 계약이다. parser/renderer의 새 실패도 이전 문서·편집 상태를 성공으로 오인하지 않게 보존한다. |

rebase에서 `Cargo.toml`의 새 Native 의존성과 S01의 Unix 감독 의존성을 모두 보존했다.
`agent_knowledge_map.md`는 새 base의 내용과 S01의 필드 사전·확인일을 함께 유지했다.
착수 시점에는 새 base의 실제 보존 검증을 미완료로 구분했다. 최종 보존 결과는 아래
「새 base 기능 보존」에 기록한다.

## 선행 구현과 원인 경계

- #7911 → #7913 `4662c069e5`: PDF frontend·제한 constructor와 기존 font/cache 격리.
  [S01 분석](task_m100_8174_v2_stage1.md#7911의-남은-할당-경계-대조)의 B01–B05와 구분한다.
- #8183 → #8190 `e7a0dec55c`: 개별 코덱·실제 decoder 예약과 alpha. 두 작은 alpha 사례의
  [메인테이너 정답 결정](task_m100_8183_alpha_visual_review.md#메인테이너-결정--2026-10-24)을 유지한다.
- 현재 요청 경로의 PNG borrowed fast path와 RGB/JPEG 경로는 ICC transform을 일관되게
  소비하지 않는다. ICC feature만 켜서는 F03이 완료되지 않는다.
- 제한 constructor는 xref 복구를 명시적으로 거부한다. legacy 복구의 전체 읽기·정규식 scan을
  그대로 호출하지 않으며, 복구 결과가 같은 요청 예산과 실제 object load를 소비해야 한다.
- SVG의 현재 공통 판정은 정상 gradient·clip 등을 거부한다. Native replay가 이미 갖춘
  resvg 소비와 공통 판정·후속 브라우저 소비를 같은 유효 artwork 결과로 연결한다.
- 현재 링크 읽기는 폭넓지만 새 command 생성과 PDF 출력의 scheme 필터는 좁다.
  목적지의 원본·분류·host 실행을 분리하고 PDF의 실제 action/destination을 검사한다.

## 최종 제품과 빌드

제품 source는 `5a77054519216f9f2dae6a779e7e334f3894466d`, 회귀·시각 증거를 고정한
검증 checkout은 최종 자동 검증의 source 항목에 기록한다. 제품 source 이후에는
`src/`·`vendor/`·`crates/`·Cargo manifest/lock의 변경은 없다. 이후 작업 기록만 추가한다.

| 고정 산출물 | source | SHA-256 / 크기 |
| --- | --- | --- |
| 기준 Native Skia | `9418ed0c7c` | `5b5d20ded5f7c509e0cccc9af3008aae731ddfaecb5006654efba736730b69ae` |
| 최종 Native Skia | `5a77054519` | `5156299d25ae896b25f5affe7dd369a9f84f452354ef6341eb4a128a2ffacf5a`, 74,145,368 bytes |
| 기준 최적화 WASM | `9418ed0c7c` | `27f3b1248f11d6035e47e1069ebfbd0e7afbfee0937c4dfcc7a51aa419ff0686`, 22,273,224 bytes |
| 최종 최적화 WASM | `5a77054519` | `ad5ab226585925cb00833762925eec8319999ecacb218866ef0dc7a610a00607`, 23,297,205 bytes |

Cargo 작업은 같은 절대 공용 target에서 순차 실행했다. 정상 producer를 별도 경로에 복사해
테스트의 dev-dependency graph나 후속 제외 빌드가 덮어쓰지 않게 했다.

```bash
CARGO_BUILD_JOBS=4 cargo build --locked --profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --bin rhwp
CARGO_BUILD_JOBS=4 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review \
  scripts/wasm-pack-locked.sh --target web --out-dir pkg
```

Native는 5분 10초, wasm-pack·wasm-opt는 11분 29초에 종료했다. wrapper가 성공한 뒤
`pkg`와 `rhwp-studio/public`을 동기화했다. tracked JS의 변경은 새 base의 `getTextLayout`
주석 동기화이며 ignored pkg·WASM은 커밋하지 않는다. 공식 배포·remote push는 수행하지 않았다.

## 실제 소비와 전쪽 시각 증거

| 실제 실행 | 결과 |
| --- | --- |
| 새 PDF의 Native CLI·fresh WASM 가져오기/저장/재열기 | 각각 24건 성공, 모두 1쪽 |
| 새 입력에서 저장한 그림의 Native/WASM 픽셀 대조 | 20건 전체 채널 차이 0. 나머지 네 입력은 path paint |
| RGB·Indexed·간접 Filter ICC와 독립 Pillow/LittleCMS | 3건 전체 픽셀 차이 0 |
| 새 24건 + #8183 기존 17건의 Native Skia | 41조건·41쪽, raw 최저 100%, 미달·누락·쪽수 불일치 0 |
| 같은 입력의 최적화 fresh WASM Canvas | 41조건·41쪽, raw 최저 100%, 미달·누락·쪽수 불일치 0 |
| 대표 compare·standalone overlay·review | 11조건×2 backend를 새로 산출. 색·연속 alpha·palette·복구한 paint·200 DPI scan의 내용/순서를 직접 판독 |
| standalone SVG artwork Native ↔ 독립 Chrome | gradient·clip/opacity·내부 use·제한 script·내장 PNG/JPEG의 6건 모두 raw 100% |
| Native PDF 링크·내부 책갈피 | 6종 action과 SVG 호환/Skia 직접 PDF의 GoTo가 읽힌다. PDF 좌표를 reader 좌표로 변환한 목적지가 실제 대상 단어 안에 있으며 OpenAction 없음 |

전쪽 TSV를 먼저 생성했다. source·producer hash·TSV 원문·대표 PNG는
[영구 증적](../pr/assets/issue8174/v2-s02/manifest.json), 입력·독립 기준·profile hash와
생성 조건은 [입력 설명](../../pdf/issue8174/features/README.md)에 보존했다.
24개 원본 PDF를 독립 generator에서 다시 생성해 바이트 일치를 확인했다.

실행 argument 배열은 ignored `output/pr-review/issue8174/v2-s02-20261024/`의
`native-sweep-command.json`·`wasm-sweep-command.json`에 있다. 명령은
`venv/bin/python scripts/visual_sweep.py --silhouette-only --compat 2024 --dpi 96
--page-workers 2 --rhwp-bin <고정 Native>`에 입력별 `--file-target KEY DOC PDF`를 쓴다.
WASM은 `--wasm-pkg <고정 최적화 package>`를 더했다. 두 가져오기가 각각 저장한 HWPX를
소비했다. 대표 PNG는 같은 명령에서 `--silhouette-only`를 제거해 생산했다.
새 대표 22장 중 16장은 이전 후보의 PNG와 바이트도 같았다. 보정한 Indexed preblend
두 장과 정상 RGB·ICC palette 대조군 네 장을 직접 확대 판독했다. 이전 PNG를 새 출력 대신 제출하지 않았다.

ICC 기준은 원본 PDF의 Ghostscript 정규화와 독립 LittleCMS다. 원 JPX에 대한 Ghostscript의
일부 빈 출력·Indexed palette 차이는 일치 근거로 쓰지 않았다. 독립 encoder에 넣은 샘플과
PDF의 colour/opacity 계약에서 외부 SMask 기준을 만들었다. **RHWP 출력은 기준 생성에
사용하지 않았다.** #8183의 두 작은 alpha Print 결함에 대한 메인테이너 결정을 새 입력에 확장하지 않았다.

standalone SVG의 gradient 차이는 최대 1 channel, clip의 가장자리 AA 차이는 최대 28이며
내부 use·script 무시·내장 PNG/JPEG는 차이 0이다. 원 SVG에서 독립 Chrome raster를 생산했다.
이는 HWP/HWPX 한컴 Print나 RHWP WASM 제품의 SVG 글리프 검증이 아니며 F05의 브라우저
소비는 S03·S04다. [SVG 증적](../pr/assets/issue8174/v2-s02/svg-native/review.png)을 구분한다.
최종 `5a77054519`의 정상 Native producer에서도 여섯 출력을 다시 생성해 이전 대표 PNG와
바이트가 같음을 확인했다. 책갈피 reader는 PDF의 아래쪽 원점을 위쪽 원점으로 변환한 뒤
대상 단어와 대조했으며, 두 PDF 경로 모두 같은 실제 내용으로 연결됐다.

## 새 base 기능 보존

| 경계 | 실제 최종 실행과 판정 |
| --- | --- |
| #8178 선택 실행 보고서 | Chrome 진단·PNG integrity·report/run Node 계약 34건 통과. 해당 source 변경 없음 |
| #8191 일반 HWP·TAC 경계 | `hwp_table_test.hwp`와 `tac_cell_leftover_fits.hwpx`의 각 전 3쪽을 base/최종 Native·WASM으로 비교. 후보와 base의 RGB 픽셀 차이 0, 쪽수·내용 소속 동일 |
| 독립 Print 최소 raw | 일반 표 Native 96.61580%·WASM 96.11700%, TAC Native 99.29369%·WASM 99.30776%. 90% 미달·누락·2%p 이상 하락 0 |
| #8195 명령 실패·복구·통지 | 변경 없는 Studio/공통 JS의 전체 Node 2,438 PASS·2 skip·0 FAIL, 새 WASM의 실제 Worker 여정과 함께 대조 |

[전쪽 보존 TSV](../pr/assets/issue8174/v2-s02/preservation/results.json)를 보존했다.
TAC의 앞 문단·큰 행·끝 행·뒤 문단의 실제 배치도 독립 Print와 판독했다. 표 선색·글꼴의
기존 미세 차이와 새 변경에 의한 후퇴를 구분했다. #8191의 과거 49조건·1,772쪽을 이번에
전부 재캡처했다고 주장하지 않는다. 현재 변경은 HWP 조판을 재작성하지 않는다.

## 실제 한도·cache·실패 뒤 복구

- CLI는 `{ "policy": { "backend": { ... } } }` 봉투를 쓴다. 최초 스크립트가 Rust 내부 JSON만
  전달한 설정 오류는 제외하고, 공식 형식으로 수정한 뒤 **실제 `PDF_RESOURCE_LIMIT`**을 확인했다.
- ICC path의 `codecWorkUnits=1`, CMYK ICC의 `codecAllocationBytes=128`, xref scan의
  `codecWorkUnits=1`은 실제 한도로 실패했다. stdout은 비었고 기존 출력 bytes를 보존했으며
  매번 다음 정상 ICC 요청이 성공했다. CMM 예산 실패를 대체 색의 성공으로 숨기지 않는다.
- 독립 시각 gate 뒤 source sample·ICC 픽셀·alpha·raw/boxed 동등성·그림/쪽 소유와
  HWP/HWPX 저장 보존 회귀를 추가했다. profile 참조/compiled cache의 재사용·수명 해제,
  복구의 최신 객체·Info·모호한 길이/잘린 trailer 거부, 압축 profile의 stream 한도도 검사한다.
- 기존 mask 실패는 `80968497c2c`/PR #8120의 손상 filter 거부와 #8183/PR #8190의 실제
  decoder 연결을 추적했다. 미활성 codec의 `Unsupported`만 허용하던 기대를 활성 코덱의
  `Decode`·`Image`·`UnsupportedFilter` 거부로 확장했다. 거짓 filter를 붙인 raw data의 성공이나
  예산 오류는 허용하지 않는다. 픽셀 좌표·golden을 완화하지 않았다.
- `StreamData`가 `Object::decode_stream_data()`에도 예산을 전달함을 확인해, 처음 제기한
  무제한 압축 해제 가정은 철회했다. 실제 보정은 **profile의 간접 Filter 해소**다.
  이전 후보 `248f6b0435`는 `(128,64,32)`로 대체 출력했지만 최종 source는 독립 CMM의
  `(191,53,29)`와 일치한다. 공통 document decoder를 cache·해소된 profile·OutputIntent가 소비한다.

## 성능·메모리 관측

고정 정상 Native CLI를 12회 교대로 실행했다. 첫 회와 이후 11회 중앙값을 나누고
`/usr/bin/time`의 peak RSS를 관측했다. 호스트 작업·JIT·캐시 영향을 받는 로컬 관측이며
보증 상한이나 통계적인 성능 개선 주장으로 확대하지 않는다.

| 공통 입력 | base warm → 최종 | base peak RSS → 최종 |
| --- | ---: | ---: |
| JPEG PDF 가져오기·HWPX 저장 | 31.83 → 41.85 ms | 25,148 → 25,768 KiB |
| 일반 표 HWP info/조판 | 12.93 → 13.49 ms | 22,272 → 23,296 KiB |

base `9418ed0c7c`에는 S01 감독 작업자가 없으므로 약 10.0 ms 증가는 **S01+S02 전체**의 비용이다.
S02 기본 코덱만의 증가로 분류하지 않는다. 감독을 빼거나 기본 기능 결정을 되돌리지 않았다.

Chrome의 새 탭 네 번씩에서 각 package를 초기화하고 입력별 12회 직접 실행했다.

| 최적화 fresh WASM | base → 최종 |
| --- | ---: |
| module import/초기화 중앙값 | 100.15 → 107.55 ms |
| JPEG PDF 첫 가져오기 중앙값 | 217.05 → 222.60 ms |
| JPEG PDF warm 중앙값 | 33.50 → 33.05 ms |
| 일반 표 HWP warm 열기 중앙값 | 13.95 → 14.15 ms |
| 선형 메모리 high-water | 양쪽 9,371,648 bytes |
| 최적화한 `.wasm` 크기 | +1,023,981 bytes, 약 4.60% |

최초 실행의 변동과 일반 표의 +0.20 ms를 숨기지 않는다. 위 숫자로 모든 입력의 영향이 없다고
단정하지 않는다. ICC/CLUT의 요청 사용량과 일반 입력을 구분하며 선형 메모리는 JS heap·
브라우저 전체 RSS가 아니다. `performance/`와 producer JSON에 원시 관측을 보존했다.

## 최종 자동 검증 — 완료

최종 회귀·자료 commit `7d85ef57a036a0725f406c64cbb664ae77e764ff`에서 필수 lint·전체 release-test/Native Skia·
개발용 제외 구성을 순차 실행해 모두 통과했다. 명령·exit·시간·검증 source는 ignored
`output/pr-review/issue8174/v2-s02-20261024/final-gates.json`에 기록한다.
정상 producer source는 `5a77054519216f9f2dae6a779e7e334f3894466d`이며 두 revision 사이의 제품 코드 변경은 없다.

새 최적화 WASM의 기존 codec/runtime 계약은 19건·158 checks, 추가 일반 import/실제 Worker는
138 assertions·12쪽·7 Workers가 모두 통과했다. 취소·deadline·decoder 실패 뒤 재생성과
다음 정상 요청을 실제 실행했다. S03의 새 자원 프로필 연결 완료를 뜻하지 않는다.

qcms는 원본 `COPYING`과 바이트가 같은 `LICENSE`도 함께 포함해 source 검사 계약을 충족했다.
`5a77054519`의 **14개 source 후보 사전 검사 통과**, 도구 테스트 18건 통과다.
원본 PDF xref의 규격상 공백과 upstream 고지·C bindings 주석 네 줄의 whitespace는 보존했다.
주석은 해시를 확인한 원본 crate와 줄 단위로 대조했으며 다른 변경의 `git diff --check`는 통과했다.
문서 메타데이터의 base에도 있던 20개 오류는 동일함을 대조했으며
이번에 바꾼 canonical 문서의 오류는 0이다.

S03·S04의 제품 연결, F07/F08, MD05의 B01–B05·B07·다른 OS·다른 CLI 진입점과
파일/네트워크 경계, S05의 실제 14개 graph 포장·registry 소비는 남아 있다.
`parserAllocationBoundsProven=false`·`filesystemNetworkSandbox=false`·`publish_ready=false`를
유지한다. source 사전 검사를 Cargo package/실제 게시 성공으로 바꾸어 쓰지 않는다.

## 최종 사양 대조의 보정 — Indexed preblend

PDF 32000-1:2008 §11.6.5.3은 Indexed의 preblend 대상을 인덱스가 아닌 팔레트 색으로 명시한다.
기존 S02 구현과 합성 기준이 모두 인덱스를 alpha로 나누어 잘못된 정답을 공유한 것을 확인했다.
첫 90% gate의 이 부분은 독립 해석의 증거가 아니었으므로 그대로 최종 통과로 사용하지 않는다.

공통 Indexed expander에서 원 인덱스 lookup → base 색 성분의 preblend 역변환 → ICC/CMYK
변환 순서로 보정했다. 기존 원본 20개는 바이트 그대로 유지하고 잘못 해석한 기준만 사양으로
다시 만들었다. 이전 기준은 ignored 증적과 Git 이력에 보존했다. 별도로 정상 preblended RGB
palette·ICC palette 대조군을 생성해, 색 변환 뒤에 alpha를 나누는 오류도 검출했다.
새 Native/fresh WASM·전쪽 TSV·확대 판독을 마친 뒤 해당 회귀 기대값과 정식 자료를 갱신했다.


### 팔레트 보정의 실제 경계·비용

최종 Native·WASM에서 기존 20개 원본을 유지한 채 기준을 올바르게 해석한 결과와 새 RGB/ICC
palette raw·boxed 4개 대조군이 모두 raw 100%다. 원본의 index 64는 green을 선택한다.
팔레트 색에만 역 preblend를 적용하고 정상 범위로 제한하므로 첫 RGBA는 `(0,255,0,128)`이다.
기존 red assertion은 잘못된 인덱스 역변환을 요구했으므로 새 출력·독립 기준을 확인한 뒤
엄격한 green assertion으로 교정했다. 정상 대조군의 첫 색은 `(255,0,0,128)`, ICC 대조군은
독립 LittleCMS의 `(191,53,29,128)`와 같다. ICC 전에 복원해야 하는 비선형 변환 순서를 보호한다.

원 Lab 범위를 잃은 palette는 preblend를 추측하지 않고 `PDF_IMAGE_UNSUPPORTED`로 거부한다.
straight alpha는 이전 후보와 픽셀 차이 0이다. 이를 모든 JPX 조합의 완료로 표시하지 않는다.
기존 일반 100만 픽셀 Indexed 입력은 `e5addc7d47` → 최종의 12회 교대 실행에서 warm 중앙값
36.77 → 36.76 ms, peak RSS 24,672 → 24,676 KiB였다. 저장 그림 bytes도 같았다.
제품 코드와 직접 연결한 작은 계약 9건도 정상 producer에 연결한 진단 실행에서 통과했고,
정식 suite 수집·전체 실행도 별도로 완료했다. 진단 실행으로 CI 수집을 대신하지 않았다.


### 완료한 필수 자동 검증

| 검사 | 실제 결과 |
| --- | --- |
| Native Skia 전체 nextest | 11798 PASS, 0 FAIL. Summary [1175.233s] 11798 tests run: 11798 passed (8 slow), 45 skipped |
| Native Skia 본체 libtest | 3,863 PASS, 0 FAIL, 12 ignored. 함께 실행한 workspace 구성원도 15·165·2 PASS |
| `focused-default-features` | exit 0, 322.0초 |
| `focused-links` | exit 0, 13.76초 |
| `focused-mask` | exit 0, 13.18초 |
| `fmt-check` | exit 0, 20.47초 |
| `clippy-root` | exit 0, 84.55초 |
| `clippy-wasm` | exit 0, 80.29초 |
| `build-workspace` | exit 0, 312.24초 |
| `clippy-workspace` | exit 0, 110.44초 |
| `native-issue_2225_missing_picture_placeholder` | exit 0, 1.69초 |
| `native-render_p37_direct_pdf_export` | exit 0, 1.05초 |
| `native-skia_native_glyph_replay` | exit 0, 1.63초 |
| `reduced-native-build` | exit 0, 307.91초 |
| `reduced-wasm-check` | exit 0, 76.59초 |
| `manifest-final` | exit 0, 0.69초 |

개발용 제외 Native는 코덱 두 종류의 거부·기존 출력 보존, 일반 JPEG, 명시적 ICC fallback,
예산 있는 xref 복구를 실제 실행했다. 제외 WASM은 compile 검사이며 새 runtime matrix 완료로
표시하지 않는다. [최종 검증 원장](../pr/assets/issue8174/v2-s02/validation.json)에 명령·source·
exit·시간을 연결했다. 다음 절편은 승인된 계획의 S03(WASM 프로필·한도와 Worker 연결)이다.

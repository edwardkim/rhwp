# #8174 v2 S06 — MD05 보안 경계 조사와 실행 재현

- 승인: 2026-10-25 사용자 「다음 절편 진행을 승인합니다.」
- 조사 source: `695cea7b973fc54fc5ffc7f6d414f7ecf403b882`.
- 고정 `upstream/devel`: `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`. fetch 후 변경 없음을 확인했다.
- 환경: Linux/WSL2 x86_64, Rust/Cargo 1.93.1, 공용 `target/pr-review`, Cargo jobs 4, 순차 빌드.
- 상태: **이번 조사·재현 절편의 결과를 확보했다. 제품 보정과 MD05-B의 전체 감사는 진행 중이다.**

## 판정

S05의 실제 포장·독립 소비 이후에도 **폰트 요청 예산의 적용 누락이 존재한다.** 같은
`fontEntries: 1` 정책에서 Type1 사용자 encoding 3개, simple CFF byte/GID map 3개,
CID CFF map 1,001개가 만들어져도 가져오기가 성공했다. CLI와 브라우저 WASM에서 모두
재현했고, 실제 Studio에서도 CID CFF의 성공을 확인했다. 이미 제한되는 CID 폭 3개 대조군은
동일 한도에서 `PDF_RESOURCE_LIMIT`로 실패했다. 옵션 전달 실패와 할당 계상 누락을 구분한다.

Native backend 직접 소비에서는 Unicode memo 256개를 만들고 문자열을 복사하는 동안
누적 decoded 계상이 늘지 않았다. 요청이 sticky 실패한 뒤에도 memo hit가 `A`를 반환했다.
단, Document의 sticky 검사와 제품 추출 wrapper는 오류를 계속 확인한다. 이 결과를
제품 가져오기 전체가 sticky 오류를 무시하고 성공한다는 주장으로 확대하지 않는다.

현재 `parserAllocationBoundsProven=false`·`filesystemNetworkSandbox=false`를 유지해야 한다.
S01–S05의 정상 기능과 포장 검증은 보존되지만, 이 감사와 보정 전에 비신뢰 업로드의 공식
지원·PR 제출·게시 준비 전체 완료로 판정하지 않는다.

## 선행 이력과 현재 호출 경로

원 [#7911](https://github.com/edwardkim/rhwp/issues/7911)의 본문·전체 댓글,
[#7913](https://github.com/edwardkim/rhwp/pull/7913)의 본문·댓글·review 목록과
[로컬 self-review](../pr/archives/pr_7913_review.md)를 읽었다. 실제 통합은
`4662c069e557e928ed7968d061121fde90b1f027`이며, 이후 #7970·#8120·#8190과 이번
S01·S02의 자원/코덱 변경을 현재 파일 이력과 대조했다.

[#7911 3.21–3.22](task_m100_7911_stage2.md#322-폰트-전역-캐시-격리와-truetype-cmap-예약-경계-2026-10-16)의
객체 소유 payload 복사, 제한 요청의 전역 폰트 cache 우회, TrueType map 공유·예약은 이미
보완된 범위다. 이번 재현에서도 같은 문서의 font Arc는 공유되고, 다른 요청의 font/memo는
분리됐다. 같은 BaseFont에 서로 다른 프로그램을 넣으면 각각 `A`와 `B`를 반환했다.
기존 해결을 다시 미구현으로 분류하지 않는다.

실제 경로 24곳의 파일·line·Git blob·SHA-256과 관찰은
[source-boundaries.json](../pr/assets/issue8174/v2-s06/source-boundaries.json),
이력은 [lineage.json](../pr/assets/issue8174/v2-s06/lineage.json)에 고정했다.

| 항목 | 현재 관찰과 소비 경로 | 판정 |
| --- | --- | --- |
| B01 | `FontInfo::from_dict` → Type1/CFF encoding·INDEX·charset·simple/CID map. 작은 요청 한도에서 실제 확장이 성공 | **계상 누락 재현. 보정 필수** |
| B02 | glyph-name OnceLock의 Vec/String 생성, `char_to_unicode`의 cache hit clone·miss 정규화·memo insert. 256개 memo의 계상 누락과 직접 API의 sticky hit 재현 | **보정 필수. glyph-name 경계의 실행 확대도 남음** |
| B03 | `load_fonts_impl`의 이름·fingerprint 정렬·cache index, `get_font_set`, `Arc::make_mut`와 FontInfo payload clone | 요청 격리는 통과. 잔여 복사·index 예약의 보정·실제 경로별 검증은 남음 |
| B04 | bounded object copy 이후 cache index·문자·postprocess, frontend spans/clip/recognition/lowering/진단, Core 조판·저장 | 기존 제한을 보존했다. 전 경로 할당 선계상 증명은 미완료 |
| B05 | zune worker H resize의 실제 capacity 증가 분기 | 아래 Native/WASM 실행으로 **증거 부족 보완**. 별도 progressive 호환 실패는 유지 |
| B07 | Native 감독, 실제 CLI 환경 필터, WASM Worker, Studio 후보·활성 문서 분리와 동기 재열기 | 검증한 제한은 동작. 파일·네트워크 격리, 브라우저 전체 메모리와 재열기의 강제 중단은 미구현 |

## 폰트·memo와 제품 대조

입력은 [생성기](../pr/assets/issue8174/v2-s06/probes/prepare_inputs.py)로 만든 작은 합성
PDF이며 xref는 정확히 생성했다. 한컴 문서나 조판 golden이 아니다. 입력 크기·해시는
[inputs.json](../pr/assets/issue8174/v2-s06/inputs.json)에 있다.

| 입력 | backend에서 관찰한 map | CLI | WASM | Studio |
| --- | --- | --- | --- | --- |
| Type1 | custom encoding 3개 | cap 1에서 성공 | cap 1에서 성공 | 별도 실행하지 않음 |
| simple CFF | byte/GID 3개 | cap 1에서 성공 | cap 1에서 성공 | 별도 실행하지 않음 |
| CID CFF | CID/GID 1,001개 | cap 1에서 성공 | cap 1에서 성공 | cap 1에서 성공·1쪽·진단에 정책 보존 |
| CID 폭 대조군 | 폭 3개 | 거절·기존 출력 보존 | 거절 | 거절·기존 문서/draft/recents 보존 |

세 폰트의 직접 Unicode 조회 256회에서 각각 약 33.8/34.9/39.8 kB의 allocator 요청 합계를
관찰했으며 해당 구간의 decoded charge 증가는 0이었다. 이 값은 alloc/alloc_zeroed/realloc의
요청 바이트 합계다. 진단용 결과 수집 Vec를 만들지 않고 각 반환값을 즉시 해제해 측정했다.
ASCII memo hit 한 번의 String clone은 각각 1바이트를 요청했다. Vec 오버헤드·live heap·RSS의
상한으로 사용하지 않는다.
직접 FontInfo clone도 67/64/10,294바이트를 요청하고 decoded 계상은 늘리지 않았다.
이 clone 결과는 직접 backend API의 관측이며 모든 CFF clone이 제품의 TrueType donation
분기에 도달했다는 실행 증거는 아니다.

원본 결과:
[Native](../pr/assets/issue8174/v2-s06/native-audit.json),
[CLI](../pr/assets/issue8174/v2-s06/cli-audit.json),
[WASM/Worker](../pr/assets/issue8174/v2-s06/browser-audit.json),
[Studio](../pr/assets/issue8174/v2-s06/studio-audit.json).
이 입력의 수용 여부는 보안 계약 조사이며 시각 일치 판정이 아니다.

## B05 — JPEG resize 증가 분기의 실제 증거

처음의 일반 36개 입력은 H 분기를 92회 실행했지만 capacity 증가가 없었다. 이것만으로
도달 불가라고 판정하지 않고 `vertical_sample`과 초기 destination 예약식을 대조했다.
`cjpeg -sample 1x2,2x2,2x2`로 만든 성분별 샘플링 입력 16개를 추가했다. 모두 독립 `djpeg`가
정상 디코딩했다. 원본 decoder와 감사용 관측 사본을 같은 실행에서 비교했다.

- Native와 WASM 각각 52개 입력, H 호출 **106회**, capacity 증가 요구 **10회**.
- 46개는 정상 디코딩했다. 원본/관측 사본의 픽셀·계상 결과를 각 backend 안에서 대조했고,
  Native/WASM 사이에서도 픽셀·분기 결과가 일치했다.
- 실제 증가 예약의 `worker upsample destination`을 거절하면 오류로 종료했다. 정상 입력은
  총 예약량의 정확한 경계에서 성공하고 1바이트 부족하면 거절했으며 새 decoder로 복구했다.
- 최소 baseline/progressive 1×1 입력은 실제 증가 예약 **512바이트**를 소비한다. 독립
  `djpeg`의 RGB `[0, 0, 0]`과 원본/관측 사본의 3바이트가 모두 일치했다.
- **6개 mixed progressive 입력은 현재 원본 decoder도 `Invalid slice`로 거절한다.** 감사 사본도
  같은 결과이며 H 증가 전 실패했다. WASM trap은 없었고 같은 모듈의 정상 후속 요청은 통과했다.
  이 호환 실패를 정상 디코딩 통과에 포함하거나 B05 미도달의 근거로 사용하지 않는다.

Native/wasm32의 component 구조체 크기가 달라 예약 바이트는 다르다. 일반 1×1 입력은
각각 1,875/1,527바이트였다. 한 backend에서 잰 전체 예약량을 다른 backend의 정확한 경계로
고정하지 않는다. [집계](../pr/assets/issue8174/v2-s06/jpeg-summary.json),
[Native 전체 결과](../pr/assets/issue8174/v2-s06/jpeg-native.json),
[WASM 전체 결과](../pr/assets/issue8174/v2-s06/jpeg-wasm.json),
[관측 사본의 변경 해시](../pr/assets/issue8174/v2-s06/jpeg-observer-source.json)를 보존한다.

사본은 별도 ignored 경로에 만들고 H resize 직전의 관측과 진단 API만 추가했다. 원본
vendor는 수정하지 않았다. 새 WASM은 이 **JPEG 감사 모듈만** release 빌드·wasm-opt 후 실제
Chromium에서 실행했다. Studio 제품 bundle 재빌드나 제품 전체 보안 통과로 표시하지 않는다.

## 환경별 격리와 복구

- Native 감독: 100 ms wall 초과, 1초 CPU 초과, 64 MiB AS에서 200 MiB 예약 실패,
  출력 초과, 이후 정상 요청을 실행했다. 전용 시험 파일 읽기·소유 sentinel 환경 변수·loopback
  연결은 허용됐다. 이는 **일반 supervisor API의 OS 권한 경계**를 확인한 실행이다.
- 실제 CLI는 이 일반 API에 앞서 `env_clear`와 allowlist를 적용한다. 별도로 자식 `/proc`를
  관측해 **관련 없는 소유 sentinel은 제거**, 소유 font 경로와 worker nonce는 전달됨을 확인했다.
  CPU 20초·AS 1 GiB·core 0·`NoNewPrivs=1`도 확인했다. CLI가 임의 인증 환경을 그대로
  상속한다는 주장은 하지 않는다. [실제 CLI 환경](../pr/assets/issue8174/v2-s06/cli-environment.json).
- Linux 파일·네트워크 격리는 제품에 없다. 현재 호스트의 Landlock ABI 조회는 3이고,
  새 user/network namespace 생성과 bubblewrap 사용 가능성을 확인했다. **가능성 조사이며
  실제 제품 confinement 적용 증거는 아니다.**
  [호스트 조사](../pr/assets/issue8174/v2-s06/native-isolation-feasibility.json).
- 실제 Worker에서 CID 폭 한도 실패 뒤 활성 문서가 유지됐다. deadline은 후보를 종료하고
  다음 요청은 새 Worker로 성공했다. 생성/종료는 각각 3개였다. 일반 자원 오류 때 아직
  살아 있는 후보 Worker를 재사용할 수 있는 계약과 deadline 후 재생성을 구분한다.
- 실제 Studio UI에서 같은 CID 폭 실패 뒤 문서 세대·파일명·쪽수·draft·저장 draft·recents·handle이
  유지됐고 다음 CID CFF 요청은 성공했다. Worker의 HWPX 직렬화 뒤 **메인 스레드 재열기는
  여전히 동기식**이며 종료 시각 검사는 constructor 이후다. CLI 직렬화 출력/진단도 Vec 생성 뒤
  크기를 검사하므로 해당 길이 상한을 예약 전 heap 제한으로 해석하지 않는다.

Windows/macOS, 동시 다중 요청의 총 메모리, 전체 브라우저 RSS와 메인 스레드의 강제 중단은
이번 Linux/Chromium 결과로 입증하지 않았다. 남은 MD05 보정·지원 판정에 포함한다.

## 잠금 의존성 경고 조사

Native 기본+Skia와 WASM 기본 graph 모두 `chacha20poly1305 0.11.0` → `chacha20 0.10.1`이며
feature는 `cipher`·`xchacha`다. `legacy`·`rng`는 선택되지 않았다.
[upstream #580](https://github.com/RustCrypto/stream-ciphers/pull/580)의 실제 보정은
`075ccbd357ad7cbd9ec934a011b9a379966d42a2`이며,
[변경 기록](https://raw.githubusercontent.com/RustCrypto/stream-ciphers/master/chacha20/CHANGELOG.md)은
SSE2 backend의 RNG/legacy 64-bit counter 경로와 0.10.2를 연결한다.

crates.io의 0.10.2 archive를 받아 공식 checksum과 대조했다. 후보는 yank되지 않았고 MSRV는
1.85다. 0.10.1과 비교한 Rust source 변경은 `src/backends/sse2.rs`이며 normalized manifest도
버전 갱신으로 달라졌다. [검토](../pr/assets/issue8174/v2-s06/dependency/review.json)와
[현재 graph](../pr/assets/issue8174/v2-s06/dependency/current-feature-graphs.json)에 기록했다.
**root lock을 아직 바꾸거나 후보의 제품 빌드·암호 호환 검증을 실행한 것은 아니다.** 현재 graph에
대한 정적 조사로 외부 소비자의 feature 합산·구형 CPU·전체 의존성의 안전성을 보증하지 않는다.

## 보정 순서와 완료 조건

기존 MD05·MD06·MD07 선택은 유지한다. 이번 실행으로 아래 순서를 구체화했으며 전체 이슈의
필수 항목을 다른 이슈로 옮겨 완료 처리하지 않는다.

1. **P0 — B01·B02:** Type1/CFF public legacy API를 보존하면서 budget을 받는 공통 경로를 연결한다.
   INDEX/charset/encoding·단순/복합 map, glyph name·memo 생성/복사/insert 전에 선계상하고
   fallible reserve한다. CFF DICT의 48개 operand 제한과 INDEX offset·범위를
   [Adobe CFF 명세](https://adobe-type-tools.github.io/font-tech-notes/pdfs/5176.CFF.pdf)와 대조한다.
   cache hit/miss/fallback·실패 뒤 재사용·독립 정상 mapping을 Native/WASM에서 검사한다.
2. **P0 — B03·B04:** immutable payload 공유와 실제 deep copy를 분리한다. cache name/index,
   문자·postprocess·recognition/lowering·IR·진단을 같은 요청 수명에서 선계상한다. Core 조판·
   serialization의 선행 예약·출력 제한도 소비 지점까지 확인한다. 정상 출력·성능과 작은 한도의
   정확한 실패 지점을 함께 검증한다.
3. **P0 — B07:** CLI의 허용 파일/글꼴·임시 경로와 네트워크 권한을 좁히는 실제 worker confinement을
   구현·검증한다. WASM/Studio의 복사·직렬화·재열기 한도를 연결하고 중단 불가능한 소비 지점을
   보정한다. 환경별 강제 제한과 host 책임을 capability·가이드에 일치시킨다. 다른 OS의 지원
   주장은 해당 실행 증거와 함께 판정한다.
4. **P1 — 의존성과 정상 조합:** chacha20만 0.10.2로 좁혀 갱신하고 seal/trailer의 기존 자료·
   오암호·변조·Native/WASM 소비를 재검증한다. mixed progressive 6개의 최초 실패 경로와
   독립 decoder 출력을 대조해 같은 #8174 안에서 수용·내용 보존을 보정한다.

이후 제품 code head에서 해당 전체 lint·회귀·fresh 제품 WASM·시각·성능 및 바뀐 패키지 소비를
재검증한다. S06의 감사 모듈이나 기존 S04/S05 통과를 미래 보정 source의 결과로 재사용하지 않는다.
MD01·MD08·MD09와 최종 게시 구성의 결정도 별도로 남아 있다.

## 검증 범위와 보존

추적 제품 Rust/Studio 코드·root lock은 변경하지 않았다. S04 제품 source와 현재 source의
제품 트리 차이는 S05에서 고친 backend standalone lock 하나이며 Rust 파일은 같다.
제품 CLI/WASM·Studio 조사는 보존된 S04/S05 최적화 제품을 사용했다. Native 직접 backend
probe는 같은 source에서 `pdf-import`만 선택해 새로 빌드했다. producer 해시와 새 JPEG 감사
모듈의 해시는 [product-producers.json](../pr/assets/issue8174/v2-s06/product-producers.json)에 있다.

새 조판 회귀·golden·좌표 기대값은 추가하지 않았다. 제품 코드를 바꾸지 않은 이번 문서·감사
증적 변경에는 기존 전체 회귀나 Visual Sweep을 반복하지 않는다. 자원 합성 입력의 수용 결과를
한컴 시각 검증으로 바꾸어 보고하지 않는다. 실행 명령·exit·미완료 경계는
[validation.json](../pr/assets/issue8174/v2-s06/validation.json), 재현 소스는
[증적 안내](../pr/assets/issue8174/v2-s06/README.md)에 있다.

ignored 로그·전체 입력·중간 출력은 `output/pr-review/issue8174/v2-s06-20261025/`에 보존한다.
원격 push·PR·comment·registry 게시나 인증 설정은 수행하지 않았다.

최종 문서 검사에서는 변경한 Markdown 7개의 링크와 장기 문서 2개의 metadata를 통과했다.
전체 metadata 검사의 기존 오류 20건·5개 파일은 절편 시작 source와 파일 바이트가 같다.
실행 결과의 상호 대조와 실제 경로 24곳의 source 해시도 확인했다. 소유한 Vite 7704와
브라우저·Worker 검증 세션을 종료했으며 다른 작업의 7700 서버는 보존했다.

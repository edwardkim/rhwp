# Task M100 #6534 — 공공기관 XLSX 다운로드 HWP 오탐 제거 최종 보고서

- **이슈**: [#6534](https://github.com/edwardkim/rhwp/issues/6534)
- **브랜치**: `task_m100_6534`
- **착수 기준**: `upstream/devel@9edf3d82ba47c539f4a5e0aa7fe3716671619bba`
- **최신 통합 기준**: `upstream/devel@891e395bb962627333262f110fc79354f44a2dbd`
- **완료일**: 2026-09-01 KST
- **최종 판정**: `qualified-synthetic-real-browser`

## 1. 결론

XLSX 최종 파일명이 확정됐는데 URL·redirect·MIME 중 하나에 HWP 힌트가 남아 있으면 rhwp 뷰어 탭을
열던 문제를 두 방어 계층에서 해결했다.

1. Chrome/Firefox 다운로드 observer는 metadata 우선순위에 따라 `intercept/defer/ignore`를 결정한다.
   확정 `.xlsx` 파일명과 구체 XLSX MIME은 URL의 `.hwp` 힌트보다 우선하고, URL/MIME만 HWP인
   `onCreated` 후보는 확정 metadata까지 보류한다.
2. Studio는 ZIP 시그니처를 HWPX가 아닌 `zip` 후보로만 부르고, Rust core가
   `Contents/content.hpf`와 `Contents/header.xml`을 모두 확인한 뒤 HWPX를 최종 확정한다.

격리된 실제 Chrome에서 다운로드를 발생시킨 최종 결과는 다음과 같다.

| 응답 | 저장 | rhwp 뷰어 탭 |
| --- | ---: | ---: |
| 정상 `.xlsx` | 1 | 0 |
| `.hwp` URL + XLSX filename/MIME | 1 | 0 |
| 확정 `.hwp` | 1 | 1 |
| extensionless + HWP MIME/body | 1 | 1 |

따라서 일반 XLSX 다운로드는 유지하면서 자동 탭만 억제했고, 확정 HWP와 #198의 extensionless HWP
fallback은 보존했다.

## 2. 원인과 수정 계보

### 다운로드 계층

기존 `shouldInterceptDownload()`은 filename, URL, finalUrl과 MIME의 양성 신호를 OR로 결합했다.
`onCreated`에서 URL/MIME 힌트만으로 탭을 열면 이후 `.xlsx` filename이 확정돼도 되돌릴 수 없었다.

새 `classifyDownload()`은 다음 불변식을 고정한다.

- 재요청 불가 handler는 항상 무시한다.
- 확정 HWP filename은 generic·잘못된 MIME보다 우선한다.
- 확정 비-HWP filename과 구체 비-HWP MIME은 HWP URL 힌트보다 우선한다.
- URL/finalUrl/HWP MIME만 있는 미확정 후보는 `defer`한다.
- filename 확정 또는 terminal 뒤에도 HWP 보조 근거가 유지될 때만 extensionless fallback을 허용한다.
- `ignore`는 다운로드 취소가 아니라 자동 뷰어 탭 생성 거부다.

### 바이트 형식 계층

기존 Studio와 Rust 감지는 `PK\x03\x04`만으로 HWPX를 수용했다. XLSX도 같은 ZIP 컨테이너라 HWPX
parser까지 들어간 뒤 필수 파일 누락 오류로 늦게 실패했다.

Studio는 이제 ZIP을 후보로만 전달한다. Rust는 ZIP payload를 풀지 않고 중앙 디렉터리의 두 필수 entry를
exact lookup하며, 둘 다 있을 때만 `FileFormat::Hwpx`를 반환한다. XLSX·일반 ZIP·손상 ZIP·한 entry만
가진 ZIP은 `Unknown`으로 끝난다.

## 3. 단계별 결과

| 단계 | 결과 | 판정 |
| --- | --- | --- |
| Stage 1 | 공유 6·Chrome 3·Firefox 3·Studio 2·Rust 2 red 계약 | `qualified-red-contract` |
| Stage 2 | 공유 3상태 판정, 두 browser adapter, service-worker 131건 | `qualified-download-decision` |
| Stage 3 | Studio ZIP 후보, Rust 구조 확정, 실제 ZIP HWPX 525/525 | `qualified-hwpx-structure` |
| Stage 4 | packaged Chrome 4/4, Rust 8,881/8,881, WASM·frontend 전체 gate | `qualified-packaged-download-boundary` |
| Stage 5 | 최신 devel 통합, canonical manual·재현 절차·잔여 위험과 exact-head gate | `qualified-synthetic-real-browser` |

Stage 4 full 회귀가 드러낸 기존 threat-scan 합성 HWPX 5건의 필수 entry 누락은 제품 detector를
완화하지 않고 fixture를 유효한 HWPX package로 교정했다. 전체 병렬에서만 실패하던
`rhwp-contracts` 정책 문서 runtime lookup은 compile-time `include_str!` 연결로 바꿔 원래 불변식을
결정적으로 유지했다. Stage 4 세 번째 전건에서는 8,881건이 모두 통과했다.

Stage 5 시작 전 원격 `devel`이 `891e395bb`까지 전진해 로컬 merge를 수행했다. 유일한 content conflict인
`mydocs/orders/20260831.md`는 #6534 기록과 원격 #6513 기록을 모두 보존해 해결하고 merge checkpoint
`0e3c5b524`로 고정했다. 이 exact merged head에서 아래 전체 gate를 다시 통과했다.

## 4. 검증 증적

### Rust·WASM

- native Clippy, WASM lib Clippy, workspace build와 all-target Clippy: PASS
- integration manifest: 1,096 sources / 4,758 static attrs / 48 targets
- source unit tier: 4,223 tests / 299 modules
- full release-test: 8,908/8,908 PASS, 46 policy skip, 332.620초
- Docker WASM release build와 `wasm-opt`: PASS, 6분 09초

### Studio·확장

- 공유·Chrome·Firefox service-worker: 131/131 PASS
- Studio document signature: 17/17 PASS
- Studio 전체: 1,330 PASS, 1 policy skip, 실패 0
- Studio·Chrome·Firefox production build: PASS
- frontend-extension dist 계약과 packaged Chrome smoke: PASS
- packaged Chrome 실제 download E2E: 4/4 PASS

packaged E2E는 사용자 profile이나 외부 네트워크 없이 임시 profile/download와 loopback server만
사용했다. 실제 browser download GUID, 완료 event, 저장 파일, `chrome.downloads` id와 탭 수를 함께
확인하고 임시 경로를 정리했다.

## 5. 호환성·성능

추적된 `.hwpx` 534개를 감사한 결과 실제 ZIP HWPX 525개는 전부 새 구조 게이트를 통과했다. CFB
signature 6개는 HWP, placeholder·기타 3개는 Unknown을 유지해 기대 분류 불일치는 0개였다. 비밀번호
보호 HWPX 표본도 기존 parser 계약을 유지했다.

동일 WSL2 호스트에서 525개 ZIP HWPX 전체를 독립 release binary로 번갈아 9회 측정한 중앙값은 기준
934 ms, 후보 977 ms로 +43 ms(+4.60%)였다. 문서당 환산 증가는 약 0.082 ms다. CLI process 생성까지
포함한 기초값이며 CI hard gate나 다른 환경의 성능 보장을 뜻하지 않는다.

## 6. 실제 공공기관 응답 증적의 한계

이번 작업에는 문제를 보고한 공공기관 사이트의 원본 URL, 인증 세션, 실제 response header와 event
timeline이 제공되지 않았다. 따라서 특정 기관 응답을 재생했다거나 실제 현장 표본의 MIME 분포를
측정했다고 주장하지 않는다.

대신 보고된 충돌 shape를 자체 HTTP fixture로 재현하고 실제 packaged Chrome에서 검증했다. 정책은
도메인별 예외가 아니라 확정 filename·MIME·이벤트 단계에만 의존하므로 같은 metadata 충돌에는
결정적으로 적용된다. 추후 실제 표본을 확보하면 세션 token과 식별 query를 제거한
`filename/url path shape/finalUrl path shape/mime/event order`만 추가 감사한다. 이는 현재 일반 정책
수정의 차단 조건은 아니지만 실제 기관별 분포를 주장하기 위한 필수 근거다.

## 7. 잔여 위험과 범위 밖

- Safari는 downloads API가 없어 별도 fetch 선행 게이트를 사용한다. 공유
  `rhwp-shared/security/file-signature.js`가 아직 ZIP 시그니처를 `hwpx` 후보로 표현하지만, 공통 WASM
  core는 일반 ZIP을 최종 거부한다. Safari의 선행 명칭·거부 정합화는 별도 작업 범위다.
- 확정 비-HWP 확장자 목록에 없는 extensionless 비-HWP가 잘못된 HWP MIME/URL을 끝까지 유지하면
  #198 호환 terminal fallback에 따라 후보가 될 수 있다. 실제 HWP 감지율을 보존하기 위한 명시적
  trade-off다.
- 인증·POST·일회성 URL의 원본문서 재요청 문제는 이번 metadata 오탐 수정 범위가 아니다.
- 브라우저 스토어 버전 증가와 배포는 수행하지 않았다.

## 8. 제출 경계와 최종 판정

제품·test·문서 변경은 승인된 allowlist 안에 있고, private corpus·세션 정보·실사이트 payload는 포함하지
않았다. `dist`, `pkg`, generated integration suite·manifest와 진단용 임시 파일도 제출 범위에 없다.

최종 판정은 **`qualified-synthetic-real-browser`**다. 합성 응답을 사용했지만 실제 packaged Chrome의
download lifecycle 전체를 통과했으며, Chrome/Firefox 공유 정책과 Rust HWPX 구조 판정의 보호계약이
모두 green이다. 다음 단계는 code candidate의 exact diff와 최신 base 포함을 다시 확인한 뒤, 별도 승인
게이트에 따라 push와 PR 생성을 진행하는 것이다.

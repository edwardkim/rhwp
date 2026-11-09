# seo-rii 9건의 메인테이너 검증 원장

## 소스와 판정 경계

- 기준 devel: `06181802e574dbd91e62375536751dea8c270fcd`.
- 원 기여 9개 head: [접수 원장](intake.json). #8781에 P103–P110의 원 head 8개가 모두 포함되고,
  독립 #8749와 함께 로컬 merge했다. 원 기여 SHA를 변경하거나 squash하지 않았다.
- 통합 제품 소스: `be778bae9b3803050146dcc47a896940587a612e`.
- 측정 근거 주석·검증 증거 추가: `5bccdb7b90c4f447f523fc7ec4fa0931e761c3a7`.
  [본문 해시 확인](test-header-receipt.json)처럼 두 Rust 파일의 assertion·fixture·실행 코드는 같다.
- 최종 Rust 검증은 위 주석 보완 뒤 실행한다. 원 PR의 녹색 CI를 새 통합 head의 CI 성공으로
  간주하지 않는다. 원격 push·통합 PR·공개 review/comment·merge는 별도 승인 전이다.

**9건 모두 로컬 수용 권고다.** [최종 검증](final-local-matrix.json)에서 Rust lint·workspace·정책 검사,
전체 회귀 13,005건(기존 skip 45건), Native 공식 3종 합계 4,077건이 통과했다.
작업지시자의 원격 처리 승인을 받았다. 제출 직전 검토 문서의 최종 판정·원 PR별 댓글 증거 계획을 보완했으며 제품 소스·테스트는 같다.

## 기존 해결 여부와 공통 계약

[기존 해결 여부](already-resolved-check.json): 현재 devel에는 새 명시적 Canvas2D API,
파일 저장·문서 lifecycle·다중 페이지 controller와 연속 desktop consumer 기능이 없다.
기존 #8739의 Native 기반·수식 글리프 보정은 보존한다. #536·#8701 전체 로드맵은 OPEN을 유지한다.

| 변경 | 소비 경로와 확인 결과 |
| --- | --- |
| #8749 | 문서 소유 page-layer tree → 공통 text-variant 선택 → Canvas2D 전체 part 준비 → 실제 replay. 모든 part를 준비한 group만 TextRun을 생략하며 실패 시 전체 fallback을 유지한다. 32 MiB·4,096 parts·256 reports 상한이 있다. 기본 생성자의 strict 선택은 false이고 공개 preview API만 명시적으로 활성화한다. |
| #8761 | worker의 prepareSave → 파일 크기·source stamp·승인 확인 → 동일 파일 시스템 staging·flush·게시 → 정확한 저장 revision acknowledgement. 새 경로 충돌, 외부 변경, 취소, 손실 승인, 게시 후 실패, recovery와 dirty 보존을 검사했다. |
| #8764 | 입력 settle·저장 idle → 비동기 사용자 결정 → document/revision/font checkpoint 재확인 → 전환. 닫기는 viewport retirement 확인 뒤 owner를 닫으며 취소·실패 때 문서를 보존한다. |
| #8765 | 공통 Native page geometry → bounded viewport 요청 → 한 document/view generation의 CPU batch → stale 확인 → Flutter 이미지 게시. 실제 6쪽 문서·입력·resize·저장을 실행했다. |
| #8767·#8769 | 실제 Windows Flutter SDK consumer와 Apple SDK producer의 빌드·실행·원본 artifact를 대조했다. portable 검사만으로 해당 OS 실행을 주장하지 않는다. |
| #8777·#8779·#8781 | 실제 Linux GL·Windows immutable ownership transfer·AppKit/CAMetalLayer의 32회 표시·편집·resize·종료를 검증했다. 새 UI adapter는 기존 Native 결과를 소비하며 조판·줄 구성·pagination 생산자를 바꾸지 않는다. |

조판 규칙·측정·page ownership·분할·이어받기 생산자는 변경하지 않는다. 따라서 조판 변경에 적용하는
편람 383쪽 `--handbook-gate`는 비해당이다. 실제 렌더링·viewport 소비 검증과 아래 전쪽 시각 검증은 수행했다.
문서별 좌표 보정·clamp로 원문 배치를 맞추는 분기는 추가하지 않았다.

## 로컬 실행

- [최종 Rust 묶음](final-local-matrix.json): fmt, root/WASM32/workspace all-target Clippy,
  workspace build, 고정 base의 suite 정책, 전체 nextest, Native 공식 3종을 완료했다.
  Native lib는 workspace 합계 4,040건, missing-picture 2건, Direct PDF 35건이다.
  lib의 기존 ignore 12건과 focused filter로 제외된 항목은 실행한 검사 수와 구분한다.
  최초 Node 경로 오류와 Skia 다운로드 전 DNS 경로 누락은 환경 보완 뒤 재개했고 통과한 검사는 유지했다.
  격리 profile·실제 명령·종료 코드·로그 해시와 최종 성공을 함께 보존했다.
- [집중·portable·Flutter 결과](initial-local-results.json): 최소 글꼴 Rust 집중 8건,
  Python 최종 182건, Flutter unit/widget 183건 통과. Dart analyze와 example analyze 모두 문제 0건.
  Windows Python 검사의 최초 `-I` 실행은 sibling import 오류였고 정본 `python3 -B` 명령으로
  27건 모두 통과했다. 이를 제품 결함 보정으로 기록하지 않는다.
- [실제 Native 연동](actual-native-integration.json): 새 `native-skia-gl` FFI·owner로
  저장/재열기·recovery·lifecycle·다중 페이지·IME 5건 통과. 별도 설치 글꼴을 요구하지 않는 계약이다.
- [최소 글꼴 실제 Native 연동](actual-native-isolated-integration.json)은 위 범위와 새 desktop CPU oracle을
  함께 실행해 6건 통과·skip 0건이다. 실제 snapshot/lease byte 복사와 편집 뒤 stale 거절도 확인했다.
  Flutter shell의 Fontconfig cache 경고와 6건의 실제 assertion 통과를 구분해 기록했다.
- 실제 Linux Flutter SDK public header compile/link, texture-state 13 cases/226 checks,
  Mesa llvmpipe EGL 상태 handoff와 [GTK 실제 consumer](linux-actual/receipt.json)의 32회 표시 통과.
  종료 후 source/snapshot/frame/lease를 포함한 Native 계수 9개 모두 0이다.
- [최적화 fresh WASM](fresh-wasm-build.json): Rust release + wasm-opt 완료.
  첫 실행은 종료 코드 143으로 미완료였고 완료된 재실행을 증거로 사용한다.
  최종 빌드는 `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`이며 기본 배포본과의 크기 차이를 성능 효과로 해석하지 않는다.
- [실제 Chromium 회귀](wasm-browser-contract.json): 12건 통과. 이는 동일 committed source의
  dev-profile WASM 검사이며, 최적화 배포 패키지의 실행과 구분한다. 시스템·사용자 글꼴 경로를 차단한
  filesystem 격리에서도 동일 12건이 통과했다.
- [최적화 패키지 공개 API](optimized-preview-smoke.json): 실제 문서 producer의 glyphOutline 선택,
  가시 픽셀, 잘못된 scale 4종 정규화, 잘못된 page/profile 거절과 기존 Canvas 보존,
  명시 preview 전후 기본 API의 동일 PNG를 확인했다. [기본](optimized-default.png)·[preview](optimized-preview.png).
  최소 글꼴 격리에서도 Screen·Print를 실제 실행했다. 크기는 각 profile의 기존 API와 대조했으며
  서로 다른 profile의 rounding 결과가 같다고 가정하지 않았다.
  이 fixture의 API 계약을 독립 한컴 시각 일치 증거로 확대하지 않는다.
- Rust 집중·전체·Native 회귀의 최소 글꼴 격리는 별도 홈·시스템 글꼴 디렉터리 차단을 포함한다.
  [실제 글꼴 목록](minimal-font-files.txt)을 보존했다. 시각 비교의 공급 글꼴 환경과 구분한다.

## Native와 fresh WASM 전쪽 시각 검증

Print 96 DPI, 색상 경계 보정 전 2px `silhouette_raw_match_percent`다.

| 입력 / 독립 기준 | 전체 쪽수 | Native 최저 | fresh WASM 최저 | 원본 TSV |
| --- | ---: | ---: | ---: | --- |
| `samples/issue8160/bitmap-channel-transfer-independent.hwpx` / `pdf/issue8160/bitmap-channel-transfer-independent-2024.pdf`, compat 2024 | 1 | 100.00% | 100.00% | [Native](visual/bitmap-channel-native.tsv) · [WASM](visual/bitmap-channel-wasm.tsv) |
| `tests/fixtures/gradient/radial-corner-print-contract.hwpx` / 같은 디렉터리의 `radial-corner-print-contract-2020.pdf`, compat 2022 | 6 | 100.00% | 100.00% | [Native](visual/radial-native.tsv) · [WASM](visual/radial-wasm.tsv) |

Native/fresh WASM 각 7쪽의 쪽수는 독립 PDF와 일치한다. [배포 기준 source 확인](published-baseline.json)과
[14개 PNG 쌍 대조](base-comparison.json)처럼 동일 backend의 기준/후보 PNG는 모두 바이트 단위로 같다.
기준 바이너리는 실제 공개 v0.9.0/NPM 패키지를 사용했으며 현재 devel을 새로 빌드했다고 기록하지 않는다.
현재 devel과 배포 기준의 제품 소스는 같고 JPEG decoder의 description 메타데이터만 다르다.

직접 판독: 세 회색 계조 그림의 순서·경계·명암·간격과 전 6쪽 radial의 외곽·방향·종횡비·글자 위치를
같은 배율로 대조했다. 이 범위에 수식·첨자는 없다. PDF 대비 기존의 Native/브라우저 글자 래스터화
차이를 새 개선으로 주장하지 않으며 기준 대비 새 누락·중복·배치 악화는 없다.
[Native 비교](visual/bitmap-channel-native-review.png) · [WASM 비교](visual/bitmap-channel-wasm-review.png) ·
[1–3쪽 상세](visual/radial-pages-1-3-detail.png) · [4–6쪽 상세](visual/radial-pages-4-6-detail.png).
기본 출력 대조군이며 strict preview의 모든 글꼴 품질이 한컴과 일치한다는 뜻은 아니다.

## Windows·macOS 실제 SDK 증거의 범위

[실제 Actions run](https://github.com/seo-rii/rhwp/actions/runs/42496859868)의 Windows·macOS job은 success다.
[원본 ZIP 해시·source·영수증 대조](platform-artifact-audit.json)를 보존했다.
[화면 원본 대조](platform-visual-source-map.json)는 첫 번째·32번째 frame의 원본 ZIP member와
PNG/raw 해시를 연결하며 Windows RGBA의 PNG 변환도 무손실임을 확인한다.

- consumer source `e77a9da854fb78515f1564c3b057d04e13e3e3f1`와 #8781의 Dart/C++ 런타임 입력은 같다.
  차이는 qualification helper/test 5개이며 제품 기능을 새로 추가한 source가 아니다.
- #8767 exact head의 Windows 3-frame 별도 실행도 확인했다. Windows Native DLL은
  `9452099dc796ca5626f1c4b52396a9ca07dc445e`에서 제공된 입력이다.
- #8769 exact head의 Apple SDK compile/link·producer 실행은 실제 SDK에서 수행됐으며,
  Native 라이브러리는 `4f738e766ccbba8faf938f2421ca6b516e939019`에서 제공됐다.
  consumer 실행은 이 Native 입력을 재사용한다. 새 통합 head에서 macOS Native 자체를 재빌드한 증거로 바꾸지 않는다.
- [제공된 Native 입력과 현재 devel 대조](provided-native-input-check.json)에서 양쪽 source의 제품 차이는
  JPEG decoder의 description 한 줄뿐이며 Native ABI header는 같다. 이번 후보의 추가 Rust 변경은
  명시적 Canvas2D/WASM replay이고 OS별 Native producer·ABI·조판 생산자는 바꾸지 않는다.
- Windows continuous와 AppKit Metal은 각각 32회 표시·revision/resize 변화·종료 후 계수 0을 확인했다.
  raw RGBA 해시와 직접 판독한 [Windows 첫 화면](platform-visual/windows-continuous-01.png)·
  [마지막 화면](platform-visual/windows-continuous-32.png),
  [macOS 첫 화면](platform-visual/macos-platform-0.png)·[마지막 화면](platform-visual/macos-platform-1.png)을 연결한다.
- 비교는 opaque 내부 채널 ±1, 독립 source palette 13 + 경계 1px, 유일한 zero-shift 정렬 계약이다.
  CPU/GPU 전체 바이트 동일성을 주장하지 않는다. `consumerGpuCompletionVerified:false`,
  `hardwareAccelerationVerified:false`와 Windows `targetReuse:false`는 유지한다.
  Linux는 llvmpipe, Apple은 Paravirtual 환경이므로 실물 GPU 성능·장시간 표시·OS IME/접근성 완료를 의미하지 않는다.

과거 #8726의 실험 기반 예외를 새 P110의 SDK·실제 표시 증거 대신 사용하지 않았다.

## 원 PR별 공개 댓글 증거

[댓글 계획](contributor-comment-plans.json)과 각 review의 `Merge 후 contributor PR comment 계획`을 따른다.
[비교 지표](comment-visual-metrics.json)는 기존 검증 raster에 정본 overlay·review 도구를 적용한 결과다.
[실제 SDK 화면 원본](comment-visual-sources.json)과 [P106 exact head의 세 화면](windows-p106-exact.json)을
구분해 보존했다. 다른 PR의 대표 이미지를 대신 사용하지 않는다.

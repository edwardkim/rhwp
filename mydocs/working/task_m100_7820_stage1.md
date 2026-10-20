---
kind: working
status: completed
last_verified: 2026-10-11
---

# #7820 1단계 — 공통 래스터와 Sixel 출력

기준 source는 `790a6f8f388445e7bff2bcb165aa7b384f9ece80`이다.
[계획](../plans/task_m100_7820_sixel.md)의 단일 쪽 CLI 범위를 구현했다.

## 구현과 소비 경로

`DocumentCore::build_page_layer_tree_with_profile`의 같은 결과를 PNG와 Sixel이 소비한다.
`SkiaLayerRenderer::render_pixels_with_target_width`에서 기존 canvas replay·글꼴·배경 정책을
유지하고, N32 surface를 명시적인 RGBA8888 premultiplied 데이터로 읽는다.
PNG는 `encode_pixels_png`, Sixel은 `opaque_rgba_on_white → icy_sixel::encode_with`로
분기한다. 행 패딩·alpha·sRGB byte 공간·DPI를 공통 픽셀 계약으로 명시했다.
Sixel은 정확한 요청 너비와 같은 배율의 높이 올림값을 사용하며 최대 dimension·pixel 한도와
마지막 6줄 밴드 한도를 검사한다. 줄 구성·페이지 배치·PaintOp는 변경하지 않았다.

`export-sixel`은 기본 첫 쪽·너비 360px이며, stdout에 DCS 스트림 하나만 쓴다.
쪽·너비·profile·font-path·compat 옵션과 stderr 오류·feature 누락 안내를 제공한다.
추가 지시를 반영해 `-o, --output`으로 전체 쪽의 개별 파일 저장도 구현했다.
`-p`와 함께 쓰면 해당 쪽만 저장한다. 파일 이름은 기존 PNG의 단일/복수 쪽 규칙을 따르며
파일 모드 stdout은 비어 있다. 쪽 범위는 폴더 생성 전에 검사한다.
`native-skia`의 선택 의존성에 pure Rust `icy_sixel=0.7.0`을 고정했고, 기존 의존성 버전은
올리지 않았다. 새 `palette` 의존성의 배열 `AsRef` 구현으로 생긴 타입 모호성은
`security_trailer.rs`의 네 곳에서 명시적인 byte slice로 해소했다. 암호화 알고리즘은 그대로다.

## 실행 증거

ignored `output/pr-review/issue7820-20261011/`에 명령·바이너리·원문 결과를 보존한다.

| 검사 | 결과와 증거 |
| --- | --- |
| Native 빌드 | `cargo build --locked --profile release-test --features native-skia --bin rhwp` 통과, `build-native.log` |
| 합성 픽셀·CLI 계약 | 파일 옵션 반영 뒤 Native 7건·feature 없는 5건 통과, `focused-native-file-output.log`, `focused-default-file-output.log` |
| CLI 자기서술 | catalog 20건·scoped help 11건 통과, `focused-catalog-fixed.log`, `focused-help.log` |
| 기존 Skia renderer 검사 | 47건 통과, `focused-skia.log` |
| byte slice 보정의 보안 검사 | 19건 통과, `focused-security.log` |
| 독립 복원 | 공개 문서 4종의 첫 쪽 × 360/720px = 8건, libsixel 1.10.3에서 크기·RGB 복원 통과, `independent/results.json` |
| 전쪽 독립 복원 | 파일 출력 코드에서 4문서 전체 11쪽 × 360/720px = 22건 통과, `independent-pages-results.json` |
| 파일 출력 | 4문서 전체 11파일과 쪽별 stdout 바이트 동일, 마지막 쪽 선택 파일 동일, 잘못된 쪽·저장 경로 오류 검증, `file-output-results.json` |
| PNG 무회귀 | 같은 4문서 전체 11쪽에서 변경 전후 decoded RGBA·크기 동일, `native-all-results.json` |
| Native PNG Visual Sweep | 파일 옵션 반영 뒤 전쪽 11건 재출력·통과: 표 최저 96.18681%, 각주 96.77648%, crop 99.83013%, 복합 개체 99.87079%, `native-all/<key>/scores-file-output/` |
| 터미널 직접 판정 | 메인테이너가 Windows 11 WezTerm 및 Windows Terminal에서 SSH로 접속해 파일 출력 성공·시각 통과 판정, `terminal-verification.json` |

독립 기준은 `samples/hwp_table_test.hwp`와 `samples/issue7688/reference/table-core-2020.pdf`,
`samples/footnote-01.hwp`와 `pdf/footnote-01-hwp-2020.pdf`, `samples/pic-crop-01.hwp`와
`samples/issue7688/reference/image-crop-2020.pdf`,
`samples/issue7688/arrow-controls/control-compound.hwpx`와
`samples/issue7688/reference/compound-2020.pdf`다. 이전부터 출처가 확인된 같은 원문 Print PDF를 재사용했다.

Sixel RGB 평균 채널 오차는 0.0452~0.3182/255, 99% 채널 오차는 1~5/255였다.
사전 품질 예산 MAE 4/255·p99 24/255를 모두 충족했다. 양자화 손실의 수치이며 조판
누락 허용치로 사용하지 않았다. 표·각주·crop의 같은 배율 PNG 비교에서 누락·외곽 변형을
발견하지 않았다. 터미널의 최종 시각 판정은 메인테이너가 수행했다.

새 framing 테스트는 자체 decoder의 6줄 밴드 반환을 이미지 높이로 오인해 처음 실패했다.
같은 3×1 RGB 스트림을 독립 libsixel로 복원하면 정확한 3×1이었다. raster header의
`1;1;3;1`, RGB 내용·투명 패딩을 검사하도록 정정했다. `tiny.six`와
`tiny-independent.png`를 보존했다. 실제 문서 출력이 바뀐 것은 아니다.

기존 catalog 검사에서는 새 명령 추가에 따른 102→103 변화와 capabilities 순서 불일치를
검출했다. capabilities의 새 항목을 catalog와 같은 자리로 옮겼고, 명령 수 기대값은
정확히 새 명령 한 개만 반영했다. dispatch·catalog·capabilities 간 쌍방 일치 검사는 유지했다.
이미 터미널에서 판정한 8개 스트림과 파일 옵션 추가 뒤의 같은 쪽·너비 출력은 바이트가
동일하다(`terminal-evidence-reuse.json`). PDF/base/head의 각주·표 비교에서 기존 글꼴
표현 차이는 보이지만 이번 변경 전후 배치·내용과 PNG 픽셀의 추가 차이는 없었다.

## 남은 검증과 한계

feature 없는 CLI·catalog·help 검사를 완료했다. fresh Docker WASM 전쪽 Sweep과
전체 회귀·Rust lint는 아직 완료하지 않았다. Foot은 미실행이며 두 Windows 터미널 버전은
제공받지 않았다. SSH에서 저장 파일을 `cat`한 결과와 원격 CLI의 실시간 렌더링은 구분한다.
새 실물 문서 golden·배치 baseline은 추가하지 않았다. 합성 검사에서만 픽셀·프로토콜·기하 계약을 확인한다.

PNG 인코딩 전 공통 버퍼와 Skia Data 복사 비용이 추가됐다. 표 첫 쪽의 warm CLI 3회는
변경 전 PNG 0.0567~0.0640초, 변경 후 0.0591~0.0603초였다. Sixel은 360px
0.0346~0.0434초·31,142바이트, 720px 0.0540~0.0544초·71,441바이트였다.
`focused-timing.json`의 한 환경·한 쪽 실측이며 전체 성능이나 최대 메모리 보장은 아니다.
전체 CI 성격의 긴 검증은 focused·시각 결과를 공유한 후 별도 승인 절차를 따른다.

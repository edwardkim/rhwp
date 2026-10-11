# #7820 — CLI 단일 쪽 Sixel 출력 계획

- 착수 승인: 2026-10-11 작업지시자의 「이슈 처리 착수하세요」.
- 기준: `upstream/devel` `790a6f8f388445e7bff2bcb165aa7b384f9ece80`.
- 브랜치: `feat/issue7820-sixel-20261011`. 공용 target은 기본 작업공간의 `target/pr-review`.
- 범위: 읽기 전용 `export-sixel`, 0부터 시작하는 단일 쪽 선택, 픽셀 너비와 종횡비, stdout 스트림·stderr 오류, 사용·검증 안내.
- 추가 지시: 2026-10-11 「기존 export-png 처럼 출력 디렉토리와 페이지 지정」.
  `-o`로 전체 쪽의 개별 파일을 저장하며 `-p`로 한 쪽을 선택한다. stdout 기본은 첫 쪽 한 스트림을 유지한다.

## 공통 결과와 소비 경로

기존 `DocumentCore → PageLayerTree → SkiaLayerRenderer::render_raster_with_options`
경로에서 canvas replay와 크기·픽셀 수 검증을 유지한다. Skia N32 surface를 명시적인
RGBA8888 premultiplied 픽셀로 읽은 공통 결과를 만든다. PNG는 그 결과에서 기존 Skia
인코더를 사용하고, Sixel은 같은 결과를 흰색에 합성한 뒤 인코딩한다. 채널·alpha·stride·
색 공간을 타입 계약으로 명시한다. 조판·PaintOp·글꼴 선택은 수정하지 않는다.

## 인코더 선택

| 후보 | 의존성·라이선스 | 품질·성능·배포 판단 |
| --- | --- | --- |
| libsixel | C 라이브러리와 FFI, 여러 출처의 permissive 라이선스 | 성숙한 양자화와 독립 디코더. 5개 배포 플랫폼의 C 빌드·링크가 추가된다. 검증용 디코더로 사용한다. |
| icy_sixel 0.7.0 | MIT OR Apache-2.0, pure Rust, quantette·thiserror | Wu 양자화·최대 256색·반복 압축 지원. `native-skia`의 네이티브 선택 의존성으로 제한한다. 글자 윤곽 보존을 위해 diffusion=0을 명시한다. |
| 자체 인코더 | 외부 의존성 없음 | 팔레트·압축·프로토콜 유지보수를 새로 부담하므로 이번 범위에서 선택하지 않는다. |

선택: icy_sixel 0.7.0. [프로젝트](https://github.com/mkrueger/icy_sixel),
[libsixel](https://github.com/libsixel/libsixel). 문서화된 성능을 rhwp 실측값으로 주장하지 않는다.
lockfile 변경은 새 의존성에 한정하며 Rust compiler·플랫폼 빌드 호환성을 검증한다.

## 단계와 검증

1. 원 PNG의 대표 문서·옵션 출력과 decoded pixel을 보존하고 픽셀 계약을 구현한다.
2. CLI·Sixel 인코딩과 합성 계약, 범위·누락 feature·stdout 분리 검사를 구현한다.
3. 독립 libsixel 디코더로 실제 문서 출력을 복원해 dimensions·색 순서·내용 누락과 양자화 오차를 확인한다.
4. 기존 PNG 검사와 입력별 변경 전후 픽셀 비교, 필요한 Native/fresh Docker WASM Visual Sweep을 수행한다.
   새 실물 조판 회귀·golden은 90% 선행 게이트 없이 추가하지 않는다. 합성 픽셀·CLI 오류 계약과 구분한다.
5. Foot·WezTerm 실제 수동 판정과 SSH 검증은 각각 별도로 기록한다. 실행 환경이 없으면 미검증으로 남긴다.

긴 전체 CI 성격의 검증은 focused·시각 결과 공유 후 별도 승인으로 실행한다.
원격 push·PR 생성·merge·이슈 종료는 현재 착수 승인에 포함시키지 않는다.

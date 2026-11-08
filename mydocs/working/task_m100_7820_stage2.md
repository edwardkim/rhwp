---
kind: working
status: active
last_verified: 2026-10-11
---

# #7820 2단계 — Docker WASM 비교와 원격 CLI

구현 source는 `dc72a324923c404fb38a46667fc841304f3669e9`이며
[1단계](task_m100_7820_stage1.md)의 코드·합성 계약·Native PNG·Sixel 검증을 이어받는다.
현재 단계의 결과는 ignored `output/pr-review/issue7820-20261011/`에 보존한다.

## 원격 직접 사용 준비

사용자 지시에 따라 메인테이너의 Omarchy Linux x86_64(glibc 2.44) 개발 폴더에
Native Skia Linux CLI와 공개 HWP 3종·HWPX 1종을 업로드했다.
기존 PATH의 CLI는 그대로 두고 새 파일의 절대 경로 또는 `./rhwp`로 실행하게 안내했다.
SHA-256은 `9eda469d041e6dc8c6f3ce6cdc9e748aa7ec28f3ba2f7eb046b82f569699efe7`이다.
실행 파일은 `edward@192.168.2.146:/home/edward/mydev/rhwp`, 사용 가이드는
`/home/edward/mydev/README-issue7820-ko.md`, 공개 샘플은
`/home/edward/mydev/samples/issue7820/`에 있다. 버전 표시는 0.8.7이지만
아직 릴리즈하지 않은 #7820 개발 빌드다.

원격에서 version/help 실행, 표 문서 전체 3파일 저장, 파일 모드 stdout 공백,
두 번째 쪽 stdout과 파일 바이트 일치, 세 번째 쪽 선택 파일과 전체 출력의 해당 파일
일치를 확인했다. `remote-cli-verification.json`에 환경·명령 결과를 연결한다.
실시간 CLI의 터미널 화면 판정도 메인테이너가 성공으로 확인했다.
제공한 화면은 `/home/edward/mydev`에서 `./rhwp export-sixel samples/21a.hwp
-p 0 --width 320`과 `-p 1 --width 320`을 실행한 첫째·둘째 쪽이다.
두 페이지의 Sixel 직접 표시와 페이지 선택 동작을 확인한 증적이며,
해당 문서의 전체 페이지 한컴 일치 판정으로 확대하지 않는다.
첫 실행에서 보이는 `LAYOUT_OVERFLOW_DRAW`·`LAYOUT_OVERFLOW`(9.1px)는
조판 진단으로 별도 기록한다. stdout Sixel 스트림의 무결성은 앞의
바이트 비교·독립 디코더 검사로 검증했다. 이 화면만으로 진단의 원인이
이번 변경에서 발생했는지 또는 기존 문서 조판 문제인지는 판정하지 않는다.
이전 저장 스트림의 Windows 11 WezTerm·Windows Terminal SSH 시각 판정은 통과다.

## fresh Docker WASM

표준 `docker compose --env-file .env.docker ... run --rm wasm` service를 사용한다.
공용 host `target/pr-review`를 `/build-target`에 bind하고 기존 Rust 1.93.0 builder image와
프로젝트 캐시를 재사용한다. 새 기본 network 주소 pool이 고갈돼 기존 `rhwp` 프로젝트의
network·registry/cache를 사용했다. 다른 network·volume은 삭제하거나 초기화하지 않았다.
source는 같은 worktree `/app`에서 wrapper가 빌드하며 pkg·Studio public의 동기화까지 확인한다.
실제 컴파일은 프로젝트 toolchain 설정에 따른 Rust 1.93.1을 사용한다.
빌드 로그는 `docker-wasm.log`, 입력·독립 PDF는 1단계의 4문서 전체 11쪽과 같다.

Docker 빌드와 fresh WASM 전쪽 TSV 산출을 완료했다. `fresh-wasm-sweep/summary.json`의
전체 11쪽에 누락·쪽수 불일치·90% 미만은 없다. 문서별 최저값은 다음과 같다.

| 문서 | 전체 쪽수 | Native PNG 최저값 | fresh WASM 최저값 |
| --- | ---: | ---: | ---: |
| table-core | 3 | 96.18681% | 97.41234% |
| footnote | 6 | 96.77648% | 98.15558% |
| image-crop | 1 | 99.83013% | 99.83033% |
| compound | 1 | 99.87079% | 99.87578% |

WASM wrapper가 생성한 루트 pkg와 Studio public의 JS·WASM SHA-256이 각각 일치한다.
JS는 `57805b611572e0d2bf9d3a871e801698450d9a8785e7bc524ebc422f7ac924a7`,
WASM은 `2be5c801685a9af3b3048b103a5f1cbc367723de2df06108284831da9f1be16d`다.
실루엣 전용 실행의 자동 gate는 `not_evaluated`이므로 TSV 산출을 승인 판정으로
대신하지 않는다. Native PNG의 수정 전후 전체 픽셀 동일성과 독립 Sixel decode 검증은
1단계에 연결한다. `fresh-wasm-inspection/`의 PDF·Native PNG·fresh WASM 비교 PNG를
직접 확인하고 표 셀·본문 줄 간격 및 각주 구분선·각주 본문을 확대 판독했다.
표 괘선과 그림 경계·화살표의 주요 위치 및 내용 순서는 유지된다. 한컴 PDF와의
글꼴 굵기·정체/글꼴 형태, Native의 회색 괘선·각주선, compound 하단의 `id` 표시 차이는
남아 있다. Native 전체 11쪽은 고정 base와 decoded RGBA가 동일해 이번 공통 픽셀
분리로 새로 생긴 Native 출력 차이는 검출되지 않았다. WASM/Native의 글꼴 형태 차이는
동일 backend의 회귀나 한컴 완전 일치로 해석하지 않는다. 새 레이아웃 golden은 추가하지 않았다.
Sixel 기능의 메인테이너 터미널 판정과 전체 조판 일치·최종 PR 판정은 구분한다.

전체 release-test·Native Skia 공식 3종·Rust lint는 별도 검증 승인 전까지 미실행이다.
remote push·PR 생성·merge·이슈 close도 아직 수행하지 않았다.

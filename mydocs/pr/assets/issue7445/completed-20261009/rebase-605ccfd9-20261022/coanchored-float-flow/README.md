# #7687 공동 앵커 표의 흐름 소유 복구

기존 `samples/issue7687/empty-host.hwpx`와 같은 입력의 독립 한컴 2020 Print
`pdf/issue7687/empty-host-2020.pdf`, H1~H6 전 6쪽을 비교했다.
Native/fresh WASM Print/compat2022, 96DPI, PDFium153, raw 2px 실루엣이다.

| 출력 | 전체 쪽수 | 최저 raw | 90% 미만 |
| --- | --- | ---: | --- |
| 고정 base Native | 6/6 | 100% | 0쪽 |
| 고정 base WASM | 6/6 | 100% | 0쪽 |
| 변경 전 Native | 6/6 | 85.87541% | 1·6쪽 |
| 변경 전 WASM | 6/6 | 85.87541% | 1·6쪽 |
| 변경 후 Native | 6/6 | 100% | 0쪽 |
| 변경 후 fresh WASM | 6/6 | 100% | 0쪽 |

같은 backend의 base 대비 0.1%p 이상 후퇴는 전 6쪽 모두 없다. 각 쪽의 TextRun 내용·
소유·순서는 base/변경 전/변경 후와 두 backend에서 같다. 쪽 누락·중복은 없다.
양쪽 backend의 1·6쪽 review와 overlay를 직접 판독했다. 100%는 RGB byte 동일성이 아니다.
표 괘선 두께와 글자 래스터 색 차이는 남지만 본문 원점·표 사이 흐름·내용 소속은 보존된다.

## 원인과 검사

원 해결 `7bf6a6613992e42cd8bfb41a76edd7894e46d585`와 후속
`4a7e8f57aef57cd8d14e0441a5e10f1f7184ea97`을 대조했다. 양수 오프셋 표가
문단 종료를 진행시키지 않는 배제 띠 소유를 공통 placement에서 복구했다.
호스트 문단 뒤 간격이 개체 배제 띠에도 중복 포함된 2px 차이도 원본 바깥여백 소유로 보정했다.
기존 H1~H6 관계 검사를 제거·완화하지 않았다. v1은 전쪽 raw100%여도 H6 검사가 실패했으며,
v2에서 실제 통과했다. 집중 25건은 11 PASS / 14 FAIL, 5.294초이며 최신 전체 통과는 아니다.
기존 67개 입력의 재귀적 조건 경계 감사는 모두 읽기 성공, 해당 형상 6곳 모두 이 대조군이다.

## 생산 출처

- 입력 SHA256 `3c9a25d26e79de910f044a178997c0ad77a86546e8d76570dae206a252743317`
- 독립 Print PDF SHA256 `a4dc12ca09b05db1f8bbdeb98a30bcc017bd5c2dee87edf7b1268522553b8584`
- base `605ccfd92f2f12f18e14182548d729f9929fda00`
- 변경 전 HEAD `43ecc84eec229a2816ecd45fe3bdd2821030f0e9`
- 변경 후 source는 위 parent에 아래 세 파일 변경을 적용한 후보다.
- `src/renderer/float_placement.rs` SHA256 `acad36c4cfc7b24125b4b6ae5578410e263f33f4dd0ef2eb468d41c32d9c1de3`
- `src/renderer/typeset/table/block/entry.rs` SHA256 `a90f4eafb1eeb929cdf7b1b4a60e2a16c9f76599c0d97eb14e8a3c5579ccaab5`
- `src/renderer/typeset/controls/empty_float.rs` SHA256 `e1614737ce34109f93d2094fd0d4ae847ee89cc88fa85ae18e6507826f997bcc`
- Native SHA256 `e481c1c3d512700d9900cd66481b1b626deb24085412d0f7653346832cdc8875`
- fresh WASM SHA256 `626cfc8c6da915594e4e52e80621f1fd341c7e4cbad1836fa1344943328d4666`
- Mac locked wrapper `--target web --out-dir pkg --no-opt`, 로컬 대체 빌드다. Docker 최적화 통과가 아니다.
- root pkg·Studio public·frozen JS/WASM hash는 일치한다. 실제 Studio 초기화 응답 검증은 남아 있다.
- base WASM은 신규 자동 환경 API가 없어 실패한 첫 실행을 제외하고 canonical 명시적 null로
  산출했다. 후보의 실제 자동 환경도 null임을 확인해 같은 환경을 대조했다.
- 전체 실행·producer·글꼴 catalog·실패와 정정 로그는 ignored
  `rebase-8155-20261022/stage62-coanchored-float-flow/`에 보존한다.

## 대표 이미지

| 출력 | 쪽 | review | overlay |
| --- | --- | --- | --- |
| before-native | 1 | [review](before-native-p001-review.png) | [overlay](before-native-p001-overlay.png) |
| before-native | 6 | [review](before-native-p006-review.png) | [overlay](before-native-p006-overlay.png) |
| before-wasm | 1 | [review](before-wasm-p001-review.png) | [overlay](before-wasm-p001-overlay.png) |
| before-wasm | 6 | [review](before-wasm-p006-review.png) | [overlay](before-wasm-p006-overlay.png) |
| after-native | 1 | [review](after-native-p001-review.png) | [overlay](after-native-p001-overlay.png) |
| after-native | 6 | [review](after-native-p006-review.png) | [overlay](after-native-p006-overlay.png) |
| after-wasm | 1 | [review](after-wasm-p001-review.png) | [overlay](after-wasm-p001-overlay.png) |
| after-wasm | 6 | [review](after-wasm-p006-review.png) | [overlay](after-wasm-p006-overlay.png) |

모든 TSV는 원본 그대로 복사했다. 최신 전체 회귀·다른 잔여 14건·필수 게이트·실제 소비 경로는
계속 검증한다. 이 국소 보정의 통과를 PR 준비 완료로 보고하지 않는다.

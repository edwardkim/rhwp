# #8160 단계 1: 기준 고정과 현재 전쪽 출력 진단

## 계획

`task/8160-sample16-raw100`에서 base `605ccfd92f2f12f18e14182548d729f9929fda00`의
Native/fresh WASM을 실제 빌드한다. 새 MCP Print PDF 세 개를 같은 조건의 전쪽 출력과 비교하고,
페이지별 미대응 영역·기존 회귀 보호 범위를 확인한다. 전체 목표와 이후 단계는
`../plans/task_m100_8160.md`를 따른다.

## 착수 확인

- GitHub #8160은 OPEN, assignee `jangster77`이며 같은 이슈의 진행 PR은 발견하지 못했다.
- `upstream/devel` fetch 결과 HEAD와 0/0이다. 새 PDF 세 개와 생성 증적만 미커밋 상태였다.
- 실행 중인 Cargo/Rust/WASM/Visual Sweep 작업은 없었다. 공유 target을 그대로 재사용한다.
- GitHub 계정의 저장소 권한은 write다. collaborator self PR 절차로 진행한다.
- 기존 `verified_sample16_2024_print_boundaries.rs`가 실제 읽는 입력은
  `samples/hwp3-sample16-hwp5.hwp`다. 파일 제목만으로 이번 `-2024.hwp`의 보호로 세지 않는다.
- 사용자 참고 정보인 TABLE 정오표·브랜치 규칙의 문서 수정은 다른 PR에서 진행 중이다.
  이 작업의 실제 브랜치는 사용자 지정 형식을 적용했다.

## 진행 중인 검증

```bash
CARGO_BUILD_JOBS=14 cargo build --locked --profile release-test \
  --features native-skia --bin rhwp --target-dir target/pr-review
```

로그: `output/pr-review/issue8160-20261022/logs/base-native-build.log`.
Native 빌드는 exit 0, 10분 36초로 완료했다. 실행 파일과 Native rlib은
`output/pr-review/issue8160-20261022/base/native/`에 동결했고 해시는
`base/source-provenance.json`에 기록했다. 같은 source의 최적화 fresh WASM 빌드를 이어서 실행한다.

## 사용자 추가 순서

세 문서 전쪽 Native/fresh WASM raw 100%를 먼저 달성한 후 회귀 테스트를 추가한다.
기존 검사 중복은 원본 입력·보호 범위·독립 근거를 대조해 병합·제거하며 필요한 보호를 보존한다.
이번 단계에서는 새 회귀 테스트를 작성하지 않는다.

## Native 전쪽 측정 완료

실제 source는 `605ccfd92f2f12f18e14182548d729f9929fda00`이며 생산 파일 2,747개의
지문은 `a28c53ac28e6a0e7b73f4f38f3531a7a9964598793e8066be5cae9d2a393dc1c`다.
동결 CLI SHA-256은 `de6723319a105a43a54ed7d8505d1c7e6a634103f18f5562d0feabf9072b62fb`다.

| 원본 | 실제/Print 쪽수 | Native raw 최저 | fresh WASM raw 최저 | 각 backend raw 100% 쪽수 |
| --- | --- | ---: | ---: | ---: |
| HWPX | 64/64 | 96.24353% | 96.13140% | 4 |
| 2024 HWP | 64/64 | 97.73521% | 97.69323% | 4 |
| 2022 HWP | 64/64 | 95.03337% | 95.08029% | 0 |

최적화 fresh WASM 빌드도 exit 0, 35분 55초로 완료했다. WASM SHA-256은
`06fd40f347912964abe9b588c2a0e8a1d70fd5fa43a89de7f6ddea367dfec419`이며
root pkg와 Studio public의 glue·WASM 바이트 일치를 확인했다.
동결한 base package로 세 문서 각각 64쪽 브라우저 직접 PNG를 산출했고 모두 exit 0이다.
전쪽 384개 결과 중 90% 미만은 없지만 100% 목표는 미완료다.
전쪽 WASM TSV·생산 해시는 `../pr/assets/issue_8160/base-wasm-summary.json`에 연결한다.

모든 실행은 직접 Native Skia, print, 96 DPI와 실제 한컴 2020 TTF/KoPub 공급을 사용했다.
2024 Print 두 입력은 compat 2024, 2020 Print인 2022 저장본은 compat 2022다.
세 실행 모두 exit 0이다. 전쪽 TSV는 `../pr/assets/issue_8160/tsv/base/native/`에 보존했다.
공통 도구와 같은 임계값 232·반경 2px로 미대응 픽셀 수를 별도 계산했고 총192쪽 중
0픽셀인 쪽은8쪽이다. 표시값 반올림만으로100%를 판정하지 않는다.
`base-native-summary.json`과 `base-native-unmatched.tsv`에 출처·쪽별 결과를 연결한다.

## 직접 판독과 독립 폰트 진단

- 실제 전쪽 출력에서 생성한 HWPX18·41쪽, 2022 HWP40쪽 compare/overlay를 열었다.
  그림 내부 표현, 굵은 제목의 가로 전진, 2022 본문의 글꼴 차이가 남는다.
  높은 raw 값으로 실제 차이를 면제하지 않는다.
- 기존 완료 쪽 검사는 `d47ae4ff8`·`7f44525cd`에서 도입됐고 PR #8064의 실제 merge는
  `9b01d5d1447220b3a7345703e79ad42cf3f502f4`다. 기본 HWP5·2022 원본과 이번 별도 2024 원본을
  혼동하지 않는다. 이전 단계의 수치와 새 기준의 수치도 구분한다.
- 원시 CFB를 rhwp 모델을 거치지 않고 읽었다. 2022 저장본의 Section0 text record offset
  352530, `한국수자원공사는 우리나라`의 글자 모양19는 한글 font ID2·휴먼명조 TTF(`0x61`)다.
  2024 저장본의 같은 문구는 offset356984, 글자 모양19·한글 font ID7·휴먼명조 HFT(`0x62`)다.
  새 2020 Print는 이 본문을 Gulim으로, 새 2024 Print는 휴먼명조 TrueType으로 출력했다.
  `raw-source-font-evidence.json`에 원시 글꼴 레코드와 대체 정보를 보존했다.
- 2024 PDF41쪽의 휴먼명조 subset과 설치된 `HMKMM.TTF`를 읽기 전용으로 비교했다.
  한글91개의 em 정규화 윤곽이 모두 같았다. 원 TTF SHA-256은
  `d033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6`이다.
  `human-font-outline-comparison.json`에 결과를 보존하며 폰트 파일을 배포하지 않는다.
- 같은41쪽 제목의 `<` 다음 `별` 원점은 Native167.36px, 독립 PDF169.60px다.
  뒤 `기술제안서` 첫 글자는 Native252.6933px, PDF257.12px다. Native의 실제 run은
  HCI Poppy 메트릭·Palatino 표시, bold, HFT 원본이며 반각 강제 플래그는 false다.
  `<` 전진은10.6667px인데 PDF는12.96px다. 독립 ASCII 전체 표본의 regular/bold Print로
  원래 전진폭을 확인한 뒤 공통 메트릭 경로의 원인을 보정한다.

## 진행 중인 별도 실행

- 최적화 fresh WASM과 전쪽 측정은 위 기록대로 완료했다.
- 원본 HFT 공급은 `hft_original_font_supply.md`를 따른다. 대상 PDF에서 폰트를 만들지 않는다.
  별도 한양신명조 전체 한글11,172자 표본 job은 `aed935f9-41fd-498e-b21e-f39d6a9145a4`,
  HCI Poppy 전체 ASCII regular/bold 표본 job은 `e33c6bee-6cf4-44ff-a2ab-cc340cf1eab9`다.
  두 job 모두 실제 succeeded·download success·Print 0을 확인했다.
  한양신명조 표본은 11,172자 전체 순서 검증을 통과해 원 윤곽·전진폭을 옮겼다.
  대상 PDF의 두 Type3 subset 193자·51자를 읽기 전용으로 대조해 모두 원 프로그램이 같았다.
  공개 기록은 `hanyang-original-font-provenance.json`에 두며 글꼴·표본 PDF는 공개하지 않는다.
- 기존 엔진의 명시 font environment 진단은 HWPX41쪽97.73925%, 2022 HWP40쪽96.08471%다.
  원인은 글꼴 공급 하나로 해소되지 않았으며 이 선택 두 쪽을 전쪽 결과로 쓰지 않는다.

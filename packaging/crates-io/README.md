# crates.io 준비용 source 사전 검사

이 도구는 [#8174](https://github.com/edwardkim/rhwp/issues/8174)의 결정 비의존 공통 준비다.
[결정 대장](../../mydocs/plans/task_m100_8174_maintainer_decisions.md)의 선택을 기다리지 않고
현재 후보의 누락·의존 구조·재현성을 검사한다. 후보 graph를 입력으로 삼으며 게시 구성을 확정하지 않는다.

이후 승인된 [기술 보완 범위 v1](../../mydocs/working/task_m100_8174_stage2_corrections.md)에서는
기존 8개 패키지의 source 선별을 Cargo manifest에도 반영했다. 이후 #8183의 codec/resource
4개, #8191의 JPEG encoder 1개와 #8174의 qcms 1개가 추가돼 현재 `source-policy.json`의
후보는 14개다. 기존 8개의 Cargo/registry 소비, #8183 이후 12개의 source 검사와 현재
14개 후보의 실제 소비 검증을 구분한다. 본체의 현행 준비 버전은 0.9.0이다. 제품 릴리즈와 registry 게시 일정은 별도다. 사용자용 API·feature·설치 안내는
[패키지 README](README_PACKAGE.md)에 둔다. 이 사전 검사 자체는 실제 Cargo 검증을 실행하지 않는다.

#8160의 인쇄 JPEG encoder fork `rhwp-jpeg-encoder`도 실제 path 의존성과 source 사전 검사에 포함한다.

## 실행

Python 3.11 이상과 Git을 사용한다. 추가 Python package나 Cargo 빌드는 필요하지 않다.
저장소 루트에서 실행한다.

```bash
python3 scripts/crates_io_source_preflight.py \
  --output-dir output/pr-review/issue8174/source-preflight

python3 -m unittest scripts/tests/test_crates_io_source_preflight.py -v
```

`--ref <commit>`은 검사할 source를 고정한다. 기본은 `HEAD`다. `--policy`와 `--runtime-files`는
검사 설정 파일이며, 기본값은 이 디렉터리의 JSON 두 개다. 설정 파일의 제안과 Git source의 실제
name/version을 결과에서 구분한다. Git commit의 blob으로 archive를 만들므로 미커밋 source와
untracked 파일은 들어가지 않는다. 변경 source를 검사하려면 해당 변경을 커밋한 ref를 지정한다.

출력은 저장소의 ignored `output/` 아래에만 쓴다. 제품 source·manifest·lock·Cargo cache·
feature·인증·GitHub 설정을 수정하지 않고 registry에도 접속하지 않는다.

## 검사 범위

- 후보 파일 목록과 실제 source의 path 의존성·root patch를 대조하고 dependency 순서를 구한다.
  workspace에서 상속한 path 의존성도 확인한다.
- 패키지 내 라이선스·NOTICE·원본 추적 자료의 누락을 검사한다. 포함되는 글꼴은 기존 canonical
  font registry의 SHA-256, 라이선스 파일과 파생 provenance가 같은 패키지에 들어가는지 대조한다.
  저작권·재배포 권한의 법적 판단이나 SPDX 전수 해석을 수행하는 도구는 아니다.
- `runtime-files.json`에 등록한 외부 내장 파일의 literal include와 archive 안의 실제 상대 경로를
  확인한다. 현재 등록된 참조는 15개다. Rust cfg parser나 새 include의 자동 전수 수집이 아니므로
  소비 경로가 바뀌면 등록 자료를 갱신하고 실제 Cargo 소비 검증도 수행한다.
- 저장소의 `samples/`, `pdf/`, `output/`, `mydocs/privacy/`와 환경 파일이 선택되면 거부한다.
  이는 로컬 검증 corpus를 source 후보에 넣지 않기 위한 경계다. `saved/blank2010.hwp` 같은
  실제 내장 입력과 공식 라이선스 PDF는 이 자료와 구분한다.
- Git symlink/submodule와 archive 경로 이탈을 거부하고, Git의 실행 권한을 보존한다.
  재현 가능한 gzip tar를 만든 뒤 전 파일의 바이트·모드를 다시 읽어 원장과 대조한다.

로컬 작업 보호용으로 blob 하나 64 MiB·선택한 blob 합계 512 MiB·source archive 하나
10,000,000바이트 미만을 검사한다. 이 수치를 실제 registry의 최신 정책이나 제품 런타임의
메모리 한도로 설명하지 않는다.

## 결과 해석

exit 0은 **source 검사 통과**다. 실패는 exit 2이고 기존 성공 보고서도 `failed`로 바꾸므로
이전 실행의 통과를 새 실행 결과로 오인하지 않는다. `source-preflight.json`과 함께 package별
`*.files.json`, `*.source.tar.gz`를 생성한다. 출력 파일 symlink를 따라 source를 덮어쓰지 않는다.

보고서에는 source SHA·도구 파일의 SHA-256과 두 입력 JSON의 의미상 digest를 기록한다.
JSON digest는 `json.dumps(sort_keys=True, ensure_ascii=True)` 결과를 UTF-8로 인코딩해 계산한다.

`manifest_blockers`는 현재 설정에서의 준비 차단점이다. `publish_false`는 내부/fork의 현재
게시 제한, `path_version_missing`은 registry 버전 병기가 없는 의존성,
`root_patch_not_propagated`는 root patch가 하위 소비자에게 전달되지 않는 경계다.
이 목록을 보고 자동으로 version·publish flag·package 이름을 바꾸지 않는다.

모든 성공 보고서도 `publish_ready=false`, `cargo_normalized=false`, `cargo_verified=false`,
`registry_consumption_verified=false`다. 생성물은 **정규화 전 source 후보**이며 `.crate`나
`cargo package`/`cargo publish --dry-run`의 성공 증거가 아니다. 정책 선택 뒤 실제 package와
독립 소비를 검증하고 게시 승인을 따로 받는다.

이 절편의 테스트는 위 명령으로 로컬 실행한다. 기존 Python workflow 계약 검사는 특정 파일명과
명시적인 CI 호출을 사용하고 이 로컬 도구를 자동 수집하지 않는다. 이 변경에서 CI workflow를
추가·수정하지 않으며, 게시 경로가 정해질 때 검사 진입점과 필요한 최소 연결을 검토한다.

## 실제 Cargo 포장과 독립 소비

승인된 [v2 S05 실행 기록](../../mydocs/working/task_m100_8174_v2_stage5.md)에 현행 14개 후보의
실제 `.crate`와 독립 소비 결과를 연결한다. source 사전 검사의 결과를 이 실행의 결과로
바꾸지 않는다. `cargo package --no-verify --registry crates-io`는 정규화된 archive 생성이며,
이후 실제 내려받은 archive를 빌드·실행한 것과 Cargo 자체의 포장 후 verify를 구분한다.

검증 registry는 loopback 주소에서 index와 다운로드만 제공하며 게시 API는 제공하지 않는다.
작업별 Cargo `--config`와 독립 workspace의 설정으로 연결하고 사용자 전역 Cargo 설정·cache를
수동 수정하지 않는다. 외부 의존성은 공개 crates.io index를 사용한다. 각 후보의 다운로드된
archive 체크섬·source 경로와 Git의 실제 파일 목록·내용·실행 권한을 대조한다.
공용 target의 같은 이름/버전 출력 파일은 해당 실행의 이전 파일을 보존한 뒤 새로 생성한다.
단일 gzip 스트림의 CRC·종료·잔여 바이트까지 검사하는 절차는
[운영 가이드](../../mydocs/manual/crates_io_guide.md#기존-출력-파일과-gzip-무결성)를 따른다.

소비 앱에는 repository path·patch를 넣지 않는다. 기본·최소 Rust, 기본 Native CLI·PDF 제외
CLI, `wasm-api` 활성화·비활성화 앱을 구분한다. Native의 `release-test` 실행은 제품용
`release`의 성능 측정이 아니며, WASM은 wasm-opt를 완료한 실제 산출물로 브라우저·Worker와
전쪽 시각 검증을 한다. 원본 source와 이전 검증 bundle만 검사하는 것으로 대신하지 않는다.

S05 당시 실제 소비 버전은 0.8.7이었다. 현재 0.9.0의 게시 명령은
[사용자 README](README_PACKAGE.md)에 게시 후의 안내로 표시한다. 실제 등록과 소유권 변경은 별도 승인 뒤 수행한다. 확정된 14개 구성은
`publish = ["crates-io"]`로 제한하고 Rust 1.93.1을 선언한다. 최종 포장·소비·운영 인계는
[T07 기록](../../mydocs/working/task_m100_8174_t07_completion.md), 공개 전 점검은
[수동 운영 가이드](../../mydocs/manual/crates_io_guide.md)를 따른다.

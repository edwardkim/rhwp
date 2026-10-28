# Task M100 #4601 Stage 2 — v0.8.2 이후 릴리즈 범위 조사

- 이슈: [#4601](https://github.com/edwardkim/rhwp/issues/4601)
- 기준: `v0.8.2@9b16aa9e2..upstream/devel@572786d02`
- 조사일: 2026-08-11 KST
- 상태: CHANGELOG 작성 근거 확정

## 1. 범위 스냅샷

| 항목 | 실측 |
| --- | ---: |
| `devel`에 병합된 PR | 190건 |
| Git 커밋 | 2,095개 |
| 변경 파일 | 4,186개 |
| 변경 줄 | +737,740 / -22,154 |
| 준비 재개 시 열린 PR | 10건 — 이번 기준선에서 제외 |

문서·검토 증빙·테스트 자산이 변경량의 큰 부분을 차지하지만, 실제 제품 변경도 Rust 코어,
조판·렌더러, HWP3/HWP5/HWPX 파서와 저장기, Studio, 확장, CLI, Node/Python 바인딩,
배포 workflow 전반에 걸쳐 있다. 따라서 `0.8.2`처럼 단일 회귀를 고친 핫픽스로 소개하지
않고, 여러 사용자 표면이 함께 전진한 누적 패치 릴리즈로 기록한다.

## 2. 사용자 가시 변경

### 암호 문서와 저장 호환성

- HWP5 EncryptVersion 4, HWP3 암호 문서, 암호화 HWPX를 열 수 있고 Studio가 비밀번호
  입력 흐름을 제공한다.
- HWP3/HWP5/HWPX의 암호화 저장·재열기 계약이 추가됐다.
- HWP5 손상 저장 경로와 HWPX namespace·OLE·탭·차트 등 왕복 보존 범위를 넓혔다.

### 조판·렌더링

- 중첩 표의 페이지 분할, RowBreak, 셀 내부 빈 문단, 자식 표 흐름과 표 하단 테두리를
  연속 보정했다.
- PUA 글자 겹침의 사각 번호·삼각 기호를 실제 글리프로 복원하고 관련 회귀 테스트를
  추가했다.
- CanvasKit의 정확한 glyph run 재생, 페이지 번호 metric, 대형 표 편집 시 국소
  repaint와 핫패치 무효화 경계를 개선했다.

### Studio·확장·편집

- 중첩 표 안의 텍스트·표 선택과 복사 경로, 다중 셀 서식, 표 분할·결합, F5 셀 크기
  조절, VS Code outline 이동을 확장했다.
- HwpCtrl 호환 명령 표면을 단계적으로 늘리고 호환성 ledger를 제공한다.
- 대형 표의 커서 이동·선택·재렌더 비용과 중복 렌더를 줄였다.

### CLI·에이전트 표면

- `mcp-serve`가 stdio JSON-RPC 서버와 세션 핸들을 제공한다.
- IR·capabilities·plan schema, ontology, provenance, replay/lineage/audit, 서명·anchor·gate,
  검증·보안 진단과 같은 기계 소비 표면이 추가됐다.
- `run` 계획과 capsule의 입력·계획·산출 해시, CAS 전제조건, 계보·감사 계약을 통해
  에이전트 작업을 재현하고 검산할 수 있다.
- Node와 Python 공식 바인딩 및 생성 타입 드리프트 검사가 추가됐다.

## 3. 계약과 호환성 판단

`v0.8.2` 태그에는 현재의 `ir_schema.rs`, `capabilities_schema.rs`, `plan_schema.rs`,
`schema_registry.rs`가 없었다. 현재 기준선은 다음 축을 단일 레지스트리로 공개한다.

| 계약 축 | 현재 버전 | 판단 |
| --- | --- | --- |
| JSON 봉투 | 1.0 | 기존 봉투 major 유지 |
| IR | 1.0 | 신규 공개 schema 축 |
| capabilities | 1.3 | 세션·MCP annotations·schema registry 추가 이력 포함 |
| plan | 1.1 | 선택적 `preconditions.inputSha256` 추가 |
| 서명 파일 형식 | 1.0 | key/signature/keyring 교환 형식, registry 외 별도 축 |

공개 봉투의 major가 바뀌지 않았고 추가 기능은 새 명령·선택 필드 중심이다. 현재 semver와
schema 연동 정책도 제안 상태다. 작업지시자가 확정한 `0.8.3`을 유지하되, CHANGELOG에는
에이전트 소비자가 schema registry를 대조해야 한다는 점을 명시한다.

## 4. 배포 표면 변화

이번 태그는 종전의 확장·WASM 배포뿐 아니라 다음 자동화를 처음으로 함께 검증해야 한다.

- Python wheel/sdist와 `@rhwp/node`
- 4개 플랫폼 CLI archive와 `SHA256SUMS.txt`
- Debian/RPM/MSI 설치 프로그램
- `@rhwp/core`, `@rhwp/editor`, VS Code Marketplace, Open VSX
- GHCR container image와 tag/`latest`

Cargo에는 암호화·서명용 `aes`, `cbc`, `des`, `hmac`, `pbkdf2`, `sha1`, `sha2`,
`ed25519-dalek` 등이 추가됐고, workspace에는 Native FFI와 batch-convert가 편입됐다.
`THIRD_PARTY_LICENSES.md`는 단순 기준 버전 표기만 바꾸지 않고 실제 의존성 목록을 다시
생성·대조해야 한다.

Scoop·Homebrew·Winget은 릴리즈 자산의 해시가 나온 뒤 `tools/update_channel_manifests.py`로
후속 PR에서 갱신한다. AUR은 자동화 대상이 아니므로 별도 확인한다.

## 5. 기여자 범위

병합 PR 기준으로 16명이 참여했다. 기여 건수 상위는 `jangster77` 78건, `edwardkim` 37건,
`kevin9327` 19건, `postmelee` 17건, `planet6897` 16건이다. 그 밖에
`JamesPsh`, `humdrum00001010`, `enigma-jerry72`, `seo-rii`, `lpaiu-cs`, `yuyu04`,
`NacreousCloud`, `jeong-sik`, `johndoekim`, `scari`, `walnutkim`이 참여했다.

CHANGELOG의 기여자 절에는 개인별 변경을 임의로 축약하지 않고 이 병합 기록을 기준으로
전원을 표기한다.

## 6. Stage 3 작성 기준

- 한국어·영어 CHANGELOG는 암호 문서, 조판·렌더링, 편집, 에이전트·CLI, 보안·성능,
  배포·인프라 순으로 같은 사실을 대응시킨다.
- README 한국어·영어의 현행 버전 표기를 `0.8.3`으로 맞춘다.
- 9개 제품 version 필드와 Rust lockfile을 갱신한다. npm lockfile은 패키지 관리자의
  결과를 확인해 root package version이 실제로 관리되는 경우에만 함께 반영한다.
- `pkg/package.json`과 Node/Python 개발 package version은 직접 고정하지 않는다. 태그
  workflow의 `tools/set_package_version.py` 계약으로 검증한다.
- Chrome·Edge·Firefox 제출 문서는 새 권한·새 외부 endpoint가 없다는 사실을 코드 diff와
  manifest로 다시 확인한 뒤 작성한다.

## 7. 다음 단계

버전·CHANGELOG·README·라이선스·스토어 제출 문서를 `0.8.3`으로 갱신한다. 그 변경은 Stage 3
단독 커밋으로 만들고, 이후 전체 릴리즈 검증 단계와 섞지 않는다.

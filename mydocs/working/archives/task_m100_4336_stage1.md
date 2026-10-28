# Task M100 #4336 Stage 1 — R63 PyPI/npm 릴리스 파이프라인

- 이슈: [#4336](https://github.com/edwardkim/rhwp/issues/4336)
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `task_m100_4336`
- 작성일: 2026-08-09 KST
- 상태: 구현·로컬 실증 완료

## 핵심 판단

- **런타임 무변경**: `src/rhwp/_binary.py` 가 `RHWP_BIN` → 패키지 동봉(`_bin/`)
  → `PATH` 3단 탐색을 이미 계약으로 구현(§3, bindings_foundation.md) — 이번
  작업은 그 2순위 자리를 채우는 빌드 파이프라인만 만든다.
- **결정의 2단 분리**: 워크플로 머지 = 능력 채택(빌드·스모크만), 시크릿 등록 =
  실행 채택(실게시). PR 가 곧 결정 매체가 되도록 게시 단계는 시크릿 부재 시
  `::notice` 사유를 남기고 명시적으로 생략한다.
- R67(PR #4330) 개념 적층 — 버전 정합(§6)은 `tools/set_package_version.py`
  `--check` 가 게이트로 수행. 코드 의존이 없어 독립 브랜치로 제출.

## 산출물

- `bindings/python/hatch_build.py` — 환경변수 쌍(`RHWP_WHEEL_BUNDLE`·
  `RHWP_WHEEL_TAG`)이 있을 때만 동봉+플랫폼 태그. 하나만 있으면 즉시 실패.
  환경변수 없으면 완전 무개입(로컬 `-e`·sdist 종전과 동일).
- `bindings/python/pyproject.toml` — 훅 배선 5줄.
- `tools/set_package_version.py` — 태그↔Cargo 검증(`--check`)·바인딩 버전 정렬.
- `.github/workflows/release-packages.yml` — version-gate → 휠 4종(각 플랫폼
  설치 스모크 포함) + sdist → PyPI/npm 게시(시크릿 게이트). 액션은 저장소 관례의
  핀 SHA 재사용.
- `mydocs/manual/release_packages_guide.md` — 켜는 절차·한계·롤백.

## 로컬 실증 (Windows, 2026-08-09)

```
RHWP_WHEEL_BUNDLE=target/debug/rhwp.exe RHWP_WHEEL_TAG=py3-none-win_amd64 \
  python -m build --wheel → rhwp-0.1.0-py3-none-win_amd64.whl
깨끗한 venv 에 pip install → RHWP_BIN 없이:
  BUNDLED: …\site-packages\rhwp\_bin\rhwp.exe   (동봉 2순위로 발견)
  VERSION: rhwp v0.8.2
  INFO OK: pages= 17   (실물 문서 rhwp.info() 파싱)
```

- `tools/set_package_version.py 0.8.2 --check` → 일치(Cargo.toml 대조).
- 파이썬 스위트 재실행 결과는 아래 검증 절.

## 검증

- 휠 빌드→격리 설치→동봉 탐색→실행→실물 파싱 E2E (위).
- `python -m pytest` (RHWP_BIN=debug 바이너리) — 294/294 (pyproject 훅 배선이
  기존 경로에 무영향임을 확인).
- 문서 메타데이터·상대 링크 검사, `git diff --check`.
- Rust·렌더 변경 없음 — Cargo·WASM·시각 검증 해당 없음. 워크플로 실행 검증은
  머지 후 `workflow_dispatch` (기존 태그 지정) 1회를 권장(가이드 §3).

## 한계 (가이드 §5 와 동일)

manylinux_2_39 하한(구형 배포판은 sdist 경로), npm 바이너리 동봉 없음(후속),
서명 없음(version_policy §8 결정 후 부착).

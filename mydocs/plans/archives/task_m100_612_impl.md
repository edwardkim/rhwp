# Task #612 구현 계획서 — 자체 GitHub Actions matrix release-binary.yml

**Issue**: [#612 CLI 바이너리 릴리즈 파이프라인](https://github.com/edwardkim/rhwp/issues/612)
**브랜치**: `local/task612`
**Milestone**: M100 (v1.0.0)
**선행**: [수행 계획서](task_m100_612.md) 승인 후 작성
**구현 옵션**: **옵션 B (자체 GitHub Actions matrix)** — 작업지시자 결정 정합
**작성일**: 2026-05-06

## 1. 구현 영역 명세

### 1.1 신규 파일

`.github/workflows/release-binary.yml` — GitHub Actions matrix workflow

### 1.2 변경 영역

기존 파일 변경 0 (Cargo.toml 의 `[[bin]] rhwp` 이미 정의됨, `src/main.rs` 변경 0).

## 2. Workflow 설계

### 2.1 트리거

```yaml
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:  # 수동 실행 (테스트용)
```

### 2.2 Matrix 정의

```yaml
strategy:
  fail-fast: false  # 한 플랫폼 실패가 다른 플랫폼 영향 없도록
  matrix:
    include:
      - target: x86_64-unknown-linux-gnu
        runner: ubuntu-latest
        archive: tar.gz
        artifact_name: rhwp-${{ github.ref_name }}-linux-x86_64.tar.gz
      - target: x86_64-apple-darwin
        runner: macos-13
        archive: tar.gz
        artifact_name: rhwp-${{ github.ref_name }}-macos-x86_64.tar.gz
      - target: aarch64-apple-darwin
        runner: macos-14
        archive: tar.gz
        artifact_name: rhwp-${{ github.ref_name }}-macos-aarch64.tar.gz
      - target: x86_64-pc-windows-msvc
        runner: windows-latest
        archive: zip
        artifact_name: rhwp-${{ github.ref_name }}-windows-x86_64.zip
```

### 2.3 빌드 단계

```yaml
steps:
  - uses: actions/checkout@v4
  - name: Install Rust toolchain
    uses: dtolnay/rust-toolchain@stable
    with:
      targets: ${{ matrix.target }}
  - name: Build release binary
    run: cargo build --release --bin rhwp --target ${{ matrix.target }}
  - name: Package archive (Linux/macOS)
    if: matrix.archive == 'tar.gz'
    run: |
      mkdir -p dist/rhwp
      cp target/${{ matrix.target }}/release/rhwp dist/rhwp/
      cp LICENSE dist/rhwp/
      cp README.md dist/rhwp/
      cp README_EN.md dist/rhwp/
      cd dist && tar -czf ../${{ matrix.artifact_name }} rhwp/
  - name: Package archive (Windows)
    if: matrix.archive == 'zip'
    shell: pwsh
    run: |
      New-Item -ItemType Directory -Force -Path dist/rhwp
      Copy-Item target/${{ matrix.target }}/release/rhwp.exe dist/rhwp/
      Copy-Item LICENSE dist/rhwp/
      Copy-Item README.md dist/rhwp/
      Copy-Item README_EN.md dist/rhwp/
      Compress-Archive -Path dist/rhwp -DestinationPath ${{ matrix.artifact_name }}
  - uses: actions/upload-artifact@v4
    with:
      name: ${{ matrix.artifact_name }}
      path: ${{ matrix.artifact_name }}
```

### 2.4 체크섬 + Release 첨부 단계

```yaml
release:
  needs: build  # 모든 matrix 빌드 완료 대기
  runs-on: ubuntu-latest
  if: startsWith(github.ref, 'refs/tags/v')
  steps:
    - uses: actions/download-artifact@v4
      with:
        path: dist/
        merge-multiple: true
    - name: Generate SHA-256 checksums
      run: |
        cd dist
        sha256sum * > SHA256SUMS.txt
    - uses: softprops/action-gh-release@v2
      with:
        files: |
          dist/*.tar.gz
          dist/*.zip
          dist/SHA256SUMS.txt
        body_path: .github/release-template.md  # 또는 release notes 자동 생성
        draft: false
        prerelease: ${{ contains(github.ref_name, '-rc') || contains(github.ref_name, '-beta') }}
```

## 3. native-skia feature 영역 결정

### 3.1 옵션 분석

| 옵션 | 본 task 영향 | binary 크기 | 빌드 시간 | 사용자 영향 |
|---|---|---|---|---|
| **(A) native-skia 미포함 (기본 build)** | ✅ 단순 | 13 MB | 빠름 | PNG export 미지원 (SVG/PDF/Text/MD 만) |
| (B) native-skia 포함 | matrix 의 build step 변경 | ~30 MB+ | 길어짐 (skia 컴파일) | PNG 포함 |
| (C) 두 binary 병행 (rhwp + rhwp-skia) | 복잡 | 13 MB + 30 MB | 더 길어짐 | 사용자 선택 |

### 3.2 권장: 옵션 (A) — native-skia 미포함

이유:
- v0.7.10 patch release 의 본질은 "binary 발행" 자체 (이슈 #608 정합)
- PNG export 는 본 사이클 후속 영역 — 별도 release 또는 v0.8.0 에서 검토
- skia-safe 의 platform 별 빌드 영역 (Linux fontconfig / macOS / Windows) 잠재 결함 영역 회피
- 사용자 빠른 사용 시작 (다운로드 시간 + 압축 해제 시간 단축)

→ **작업지시자 결정 영역**.

## 4. 다른 결정 영역 (Stage 0 의 10.2)

### 4.1 추가 옵션 (후속)

권장: **본 task 영역 비포함**, 별도 task 후보:
- Homebrew tap — Task #616 후보
- `cargo install rhwp` (crates.io publish) — Task #617 후보
- Docker 이미지 (`ghcr.io/edwardkim/rhwp:vX.Y.Z`) — Task #618 후보
- WASM 자산 release 첨부 (npm 외 별도 다운로드) — 후속

→ 본 task 는 4 플랫폼 native binary 만으로 시작, 후속 task 분리.

### 4.2 macOS / Windows 코드 서명

권장: **본 task 영역 비포함**, README 안내 영역:
- macOS: 첫 실행 시 Gatekeeper 영역 — `xattr -d com.apple.quarantine rhwp` 안내
- Windows: SmartScreen 경고 — "추가 정보 → 실행" 안내
- Apple Developer ID / Windows EV 인증서 영역은 별도 비용 + 운영 영역 (별도 task 후보)

### 4.3 빌드 타깃

권장: **이슈 #612 본문 4 플랫폼만**:
- Linux x86_64
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

→ Linux aarch64 / musl / FreeBSD 등은 후속 task 영역.

## 5. 검증 영역

### 5.1 Stage 2 (구현) 검증

- ✅ workflow 문법 정합 (`actionlint` 또는 GitHub UI 점검)
- ✅ `workflow_dispatch` 로 dry-run 실행 (tag push 없이 빌드만)
- ✅ 4 플랫폼 모두 success
- ✅ 각 아카이브 size 정합 (~13 MB compressed 추정)

### 5.2 Stage 3 (v0.7.10-rc1 검증) 검증

- ✅ `git tag v0.7.10-rc1` push → 자동 release-binary workflow 실행
- ✅ 4 플랫폼 아카이브 첨부
- ✅ SHA256SUMS.txt 첨부
- ✅ Linux 환경에서 `rhwp --help` + `rhwp export-svg samples/exam_kor.hwp` 정합
- ✅ macOS / Windows 환경 — 작업지시자 환경 또는 GitHub Actions verification job

### 5.3 Stage 4 (정식 릴리즈) 검증

- ✅ devel → main PR (메모리 룰 `feedback_release_sync_check` 정합)
- ✅ `git tag v0.7.10` 정식 push
- ✅ 이슈 #608 / #612 close + release link 안내 (한글 + 영문)
- ✅ 컨트리뷰터 @almet 영문 알림 댓글

## 6. 빌드 시간 추정

| 단계 | 추정 시간 |
|---|---|
| Linux x86_64 빌드 | ~5-7 분 |
| macOS x86_64 빌드 | ~7-10 분 |
| macOS aarch64 빌드 | ~7-10 분 |
| Windows x86_64 빌드 | ~10-12 분 |
| 4 플랫폼 parallel 빌드 (matrix) | ~12 분 (max) |
| 체크섬 + release 첨부 | ~1 분 |
| **전체 워크플로우** | **~13 분** |

→ 작업지시자 v0.7.10 release 시점에 ~15 분 대기 영역.

## 7. 잠재 결함 영역

### 7.1 Linux 의 fontconfig / freetype 의존성

- `cargo build --release` 영역에서 system fontconfig 라이브러리 필요
- ubuntu-latest 는 기본 install 정합 — 별도 의존성 install 영역 검증 필요

### 7.2 macOS 의 Universal Binary

- 본 task 는 x86_64 / aarch64 별도 binary
- Universal Binary (`lipo` 합성) 는 후속 task 영역

### 7.3 Windows MSVC vs GNU

- `x86_64-pc-windows-msvc` 채택 (이슈 #612 정합)
- MinGW (`x86_64-pc-windows-gnu`) 는 별도 영역

### 7.4 Tag 형식 가이드

- 정식 release: `v0.7.10`, `v1.0.0`
- Release candidate: `v0.7.10-rc1`, `v1.0.0-rc1`
- Workflow 가 자동 prerelease 영역 정합 (`contains(ref_name, '-rc')`)

## 8. 본 task 의 산출물

### 8.1 코드 변경

- `.github/workflows/release-binary.yml` (신규, ~120 라인)

### 8.2 문서 변경

- `README.md` (한글) — 다운로드 영역 섹션 추가
- `README_EN.md` (영문) — Download section 추가
- `mydocs/manual/release_binary_guide.md` (신규, 한글)
- `mydocs/eng/manual/release_binary_guide.md` (신규, 영문)
- `mydocs/working/task_m100_612_stage{2,3}.md` (단계별 보고서)
- `mydocs/report/task_m100_612_report.md` (최종 보고서)

### 8.3 GitHub Releases 영역

- v0.7.10-rc1 (release candidate, prerelease)
- v0.7.10 (정식 릴리즈) — 본 task 마무리 시점

## 9. 작업지시자 승인 요청

### 9.1 결정 필요 영역

1. **native-skia feature** — 본 task 권장: 미포함 (옵션 A). 작업지시자 결정?
2. **추가 옵션** — 본 task 권장: 후속 task 분리. 작업지시자 결정?
3. **macOS / Windows 코드 서명** — 본 task 권장: 비포함 + README 안내. 작업지시자 결정?
4. **빌드 타깃** — 본 task 권장: 4 플랫폼만. 작업지시자 결정?

### 9.2 승인 후 진행 단계

- Stage 2: `.github/workflows/release-binary.yml` 작성 + workflow_dispatch dry-run
- Stage 3: `v0.7.10-rc1` tag push + 4 플랫폼 빌드 검증
- Stage 4: 정식 v0.7.10 release + 이슈 close + 컨트리뷰터 알림

## 10. 메모리 정합

- ✅ `feedback_release_sync_check` — Stage 4 의 devel → main PR 시점에 적용
- ✅ `feedback_release_manual_required` — Stage 4 의 release 작업 시 매뉴얼 정독
- ✅ `feedback_external_docs_self_censor` — README dual 갱신 시 자기검열
- ✅ `feedback_assign_issue_before_work` — 본 task 이미 edwardkim assignee 지정됨
- ✅ Issue #608 한글 / 영문 답신 정합 — 컨트리뷰터 @almet 영문 알림

## 11. 참고

- 수행 계획서: [task_m100_612.md](task_m100_612.md)
- 이슈 #612: https://github.com/edwardkim/rhwp/issues/612
- 이슈 #608: https://github.com/edwardkim/rhwp/issues/608 (@almet 의 요청)
- GitHub Actions matrix 패턴: https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs
- `softprops/action-gh-release`: https://github.com/softprops/action-gh-release

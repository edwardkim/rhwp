# #8733 고정 기준 비교 재현

기준 source는 `e4ffabea38062e716f6c333cc950fa31b42b4dd7`이다. 후보 렌더러의 code source는
`e551fcc76e0794790e25a565f2509a4891eac6cb`, 기존 검증 기록의 source는
`0352c9deee13130f374cfc8015384512ecc6256c`이다. 입력과 Print PDF는
[`../hancom-print-manifest.json`](../hancom-print-manifest.json)의 일곱 고정 쌍을 사용한다.
후보 입력을 동일하게 두 엔진으로 읽는 비교이며, 기준 코드로 신규 파일을 다시 저장하는 검사는 아니다.

## 빌드

후보 workspace를 `$CandidateRoot`, 분리한 기준 worktree를 `$BaseRoot`로 지정한다.
공용 절대 `$ReviewTarget`은 후보 workspace의 `target/pr-review`로 고정한다.
기존 producer를 ignored 증적 디렉터리 `$Evidence`에 복사하고 해시를 확인한 뒤,
Cargo 프로세스가 없는 상태에서 아래 빌드를 순차 실행했다. 후보 `pkg`는 덮어쓰지 않았다.

```powershell
git worktree add --detach $BaseRoot e4ffabea38062e716f6c333cc950fa31b42b4dd7
$env:CARGO_TARGET_DIR = $ReviewTarget
$env:CARGO_BUILD_JOBS = '4'
Set-Location $BaseRoot
cargo build --locked --features native-skia --bin rhwp
# 성공 직후 기준 exe를 보존하고 $Native에 그 보존 경로를 지정한다.
Copy-Item "$ReviewTarget/debug/rhwp.exe" $Native
$env:CARGO_PROFILE_RELEASE_CODEGEN_UNITS = '16'
.\scripts\wasm-pack-locked.ps1 --target web --out-dir pkg --release
```

Rust 1.93.1, wasm-pack 0.15.0을 사용했다. 두 빌드 모두 exit 0이며 WASM은 `wasm-opt -O`까지
완료했다. 기준 WASM의 실제 rustc 명령은 opt-level=3, codegen-units=16을 명시했다.
후보의 정확한 codegen partition은 보존 로그에서 확인되지 않는다. 따라서 빌드 옵션이 모두
동일하다고 주장하지 않는다. 문서·글꼴·출력 설정을 일치시킨 뒤 실제 PNG 픽셀도 별도로 비교했다.
각 producer 해시와 빌드 시간은 `comparison.json`에 기록했다.

## 전쪽 측정과 전체 리뷰

기준 worktree의 scripts를 사용했다. 후보와 이 세 스크립트의 Git 차이는 없었다:
`visual_sweep.py`, `visual_sweep_pdf.py`, `export-wasm-png-for-sweep.mjs`.
Node 24와 Python의 Pillow/pypdfium2/fonttools, PDF 도구를 PATH에 준비하고 기존 설치된
puppeteer-core를 사용한다. Chrome은 설치된 Google Chrome을 `VISUAL_SWEEP_CHROME`으로 지정했다.
아래 `$Evidence`는 재현 실행의 새 ignored 출력 디렉터리이며, `$Native`는 보존한 기준 Native exe다.

```powershell
$targets = @(
  @('cell-new-solid-hwp', 'cell-new-solid.hwp'),
  @('cell-new-solid-hwpx', 'cell-new-solid.hwpx'),
  @('table-new-solid-hwp', 'table-new-solid.hwp'),
  @('table-new-solid-hwpx', 'table-new-solid.hwpx'),
  @('imported-raw-zero-hwp', 'imported-raw-zero.hwp'),
  @('cell-cleared-zero-hwp', 'cell-cleared-zero.hwp'),
  @('table-cleared-zero-hwp', 'table-cleared-zero.hwp')
)
$inputs = @()
foreach ($t in $targets) {
  $inputs += @('--file-target', $t[0],
    "$CandidateRoot/mydocs/pr/assets/issue_8733/$($t[1])",
    "$CandidateRoot/pdf/issue_8733/observed_prints/$($t[0])-2020.pdf")
}
$common = @('scripts/visual_sweep.py', '--rhwp-bin', $Native,
  '--compat', '2022', '--dpi', '96', '--font-path', 'C:/Windows/Fonts', '--page-workers', '1')
foreach ($backend in @('native', 'wasm')) {
  $extra = @()
  if ($backend -eq 'wasm') { $extra = @('--wasm-pkg', "$BaseRoot/pkg") }
  python @common @inputs @extra --silhouette-only --out "$Evidence/$backend-scores"
  python @common @inputs @extra --out "$Evidence/$backend-review"
}
```

기본 PNG/print 프로필, 모든 페이지, scale=1.0을 사용했다. 페이지 선별·마스크·사후 확대는 없다.
Native 측정은 보존 producer를 사용해 WASM 빌드 중 진행했으며 두 Cargo 빌드는 겹치지 않았다.

## 비교와 판독

일곱 문서 × 두 backend × 전체 한 쪽 = 14개 page/backend 쌍이다.
기준과 후보 TSV의 `silhouette_raw_match_percent`를 Decimal로 읽고
`Decimal(base) - Decimal(candidate) > Decimal('2')`를 판정했다.
TSV는 변환 없이 바이트 복사했다. 원래 PNG는 RGB 픽셀 배열을 전쪽 비교했고,
대표 compare/overlay/review는 바이트 해시도 비교했다. 결과와 공개 asset 경로는 `comparison.json`에 있다.

raw0의 내부 수평 해치 두 줄은 기준과 후보 모두에서 누락된다. 실루엣 100%는 이 손실을 탐지하지
못하므로 해당 대조군을 시각 통과로 표현하지 않는다. 신규 solid와 explicit-none 대표는 균일 채움과
위치가 맞으며 Hancom과 경계 래스터 차이가 남는다.

일곱 PDF의 가로·세로 100%, 원래 용지, 전체 페이지, 한 쪽씩 설정은 출력자가 사후 확인했다.
직접 UI 관찰로 확인했다는 뜻은 아니다. 설치된 Xpdf pdftotext는 `-bbox-layout`을 지원하지 않아
exit 99로 PDF 문항 marker 추출을 생략했다. 이 제한은 래스터 비교와 구분한다.


## 프로필 및 기준 범위

최신 기준 Native는 저장소의 기본 dev 프로필, 후보 Native는 release 프로필이다. 빌드 프로필이
동일하다는 주장은 하지 않으며, 같은 CPU Skia 소비 경로에서 7개 실제 출력 PNG를 별도로 비교한다.
기준 WASM의 codegen16도 명시적으로 기록하고 후보의 미확인 partition을 추정하지 않는다.
이 자료는 실행 전에 확인한 devel e4ffabea38062e716f6c333cc950fa31b42b4dd7에 고정된다.
원격 branch가 이후 이동하면 별도 검토가 필요하다.

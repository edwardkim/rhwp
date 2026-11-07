# rhwp 0.9.0 — PDF 시작 안내

이 안내는 공식 CLI 배포 압축 파일에 포함됩니다. [English](PDF_QUICKSTART_EN.md).
압축을 푼 디렉터리에서 실행하거나 `rhwp`를 PATH에 추가하세요.
Windows에서는 `./rhwp` 대신 `.\rhwp.exe`를 사용합니다.

```sh
./rhwp --version
./rhwp capabilities
./rhwp import-pdf input.pdf output.hwpx --json
./rhwp info output.hwpx --json
```

PDF를 편집 가능한 문서로 가져옵니다. 원본 배치를 우선하는 `fidelity`가 기본입니다.
문단 재편집을 우선하려면 `--layout editable`을 추가합니다.

```sh
# 원본 1·3·4쪽만 가져오기: 쪽 인덱스는 0부터 시작합니다.
./rhwp import-pdf input.pdf selected.hwpx --source-pages 0,2-3 --json
./rhwp import-pdf input.pdf editable.hwpx --layout editable --json
# Linux/macOS: 열기 암호는 명령줄 인자가 아닌 표준 입력으로 전달합니다.
./rhwp import-pdf protected.pdf opened.hwpx --password-stdin < password.txt
# 실제 문서 글꼴이 설치된 디렉터리를 지정해 결과를 확인합니다.
./rhwp export-png output.hwpx -p 0 -o preview-png --font-path /path/to/fonts --profile print
./rhwp export-pdf output.hwpx -o preview.pdf --font-path /path/to/fonts
```

Windows PowerShell은 `< password.txt` 리다이렉션을 지원하지 않습니다. 대화형 입력을 지원하는
Studio를 사용하거나 PowerShell에서 표준 입력을 안전하게 공급하는 호스트 코드를 사용하세요.
암호와 원본 문서를 명령 이력·공개 로그·배포 패키지에 넣지 마세요.

- PDF 입력, JBIG2·JPEG 2000·ICC 색 처리는 기본 포함입니다. `capabilities`에서 실제 빌드를 확인합니다.
- Standard Security R2–R6 열기 암호를 지원합니다. 출력 HWP/HWPX는 새 문서이며 입력 암호를 자동 승계하지 않습니다.
- 표준 프로필은 입력 64 MiB, 선택 1,024쪽, content operators 8,000,000개, 단일 이미지 64,000,000픽셀 상한을 사용합니다. 다른 누적 한도도 적용됩니다.
- `PDF_RESOURCE_LIMIT`이면 기본 `standard` 프로필로 확인한 뒤 선택 쪽을 나눕니다. JSON 조정은 프로필 상한 안에서 낮추는 기능입니다.
- `PDF_IMAGE_UNSUPPORTED` 등 미지원 표현 오류는 한도를 올리는 것으로 해결되지 않습니다.
- 스캔 PDF의 글자를 OCR하지 않습니다. 복잡한 마스크·회전·표 추론 결과와 저장 후 모양은 원본 및 대상 편집기에서 확인하세요.

Rust·npm/WASM API, Worker 배치·취소·쪽별 진행률, Studio 임베딩은
[PDF 개발자 시작 안내](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pdf_developer_onboarding.md)를 참고하세요.
JS·WASM·Worker·Studio는 같은 릴리즈로 맞춥니다. `@rhwp/core`에는 저장소 Worker 모듈이
자동 포함되지 않으며 `@rhwp/editor`에는 호스팅할 Studio가 별도로 필요합니다.

[CLI 전체 명령](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/cli_commands.md) ·
[보안과 지원 경계](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/security_feature_limits.md) ·
[릴리즈·기여자](https://github.com/edwardkim/rhwp/releases/tag/v0.9.0)

서드파티 고지는 `notices/THIRD_PARTY_LICENSES.md`와 `notices/` 아래 원문 파일에서 확인합니다.

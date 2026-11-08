# #8174 CLI 입력 계약 자료

crates.io 준비 과정에서 base CLI와 포장한 registry CLI의 텍스트·오류·PDF 입력 계약을
같은 파일로 확인한다. 조판·렌더링 회귀나 한컴 출력 일치를 판정하는 fixture가 아니다.
제품 source 기준은 `def947c2149df5bc812d801e6e79be19a403b4d1`, 보완 code head는
`9df631769bd668d3e4b86f86d1359689a108759b`다.

| 입력 | 생성 방식·역할 | SHA-256 |
| --- | --- | --- |
| `invalid.hwp` | 24바이트의 잘못된 문서 입력. 열기 실패·exit 1을 비교한다. | `7beaea096136a120b75751c0a5137bbd2829a513f4ef062cadfbc8d8ce8b9df5` |
| `protected-test.hwpx` | 공개 `samples/hwpx/blank_hwpx.hwpx`에 아래 명령으로 합성 암호를 설정했다. 올바른 암호·오입력·미지정의 exit/stdout/stderr를 비교한다. | `d818e196e35a3c2ee8fab8e0a4594f9ac1b8220554d19287f8fd41eb8b528410` |
| [package-input.pdf](../../../pdf/issue8174/package-input.pdf) | 공개 `samples/hwp_table_test.hwp`의 0번 쪽을 기존 CLI의 SVG PDF backend로 출력했다. Producer `rhwp`, PDF 1.7, 1쪽, 12,599바이트. PDF 가져오기의 `info --json`을 비교한다. | `4d6fc1c51a36ad4e90a02d4f71834ef9c764a8098e6e3dba9395a0d6b8b01548` |

저장소 루트에서 기존 CLI로 생성한 암호 문서다. 아래 문자열은 공개 테스트용 암호다.
암호화의 난수 때문에 재생성한 파일의 해시는 달라질 수 있다. 최종 비교는 커밋된 동일 파일을 사용한다.

```sh
rhwp export-hwpx samples/hwpx/blank_hwpx.hwpx \
  tests/fixtures/issue8174/protected-test.hwpx --output-password rhwp-8174-public-test
rhwp export-text tests/fixtures/issue8174/protected-test.hwpx --json \
  --password rhwp-8174-public-test
rhwp export-text tests/fixtures/issue8174/protected-test.hwpx --json --password wrong
rhwp export-text tests/fixtures/issue8174/protected-test.hwpx --json
rhwp export-text tests/fixtures/issue8174/invalid.hwp --json
rhwp info pdf/issue8174/package-input.pdf --json
```

순서대로 올바른 암호는 exit 0, 오입력은 exit 1, 미지정은 exit 2다.

PDF 생성은 저장소의 bundle font를 상대 경로로 선택하지 않는 아래 cwd에서 실행했다.
PR 준비 때 같은 명령을 base와 registry 설치본에 각각 실행해 보존한 PDF와 바이트 일치를
재확인했다. 실제 실행 파일의 절대 경로·명령·cwd는 로컬 `pdf-output-parity.json`에 있다.

```sh
rhwp_source_dir="$(git rev-parse --show-toplevel)"
mkdir -p "$rhwp_source_dir/output/pr-review/issue8174/architecture-review/corrections"
(
  cd "$rhwp_source_dir/output/pr-review/issue8174/architecture-review/corrections"
  rhwp export-pdf "$rhwp_source_dir/samples/hwp_table_test.hwp" --backend svg -p 0 \
    -o "$rhwp_source_dir/output/pr-review/issue8174/recreated-package-input.pdf"
)
```

실행 증거와 기존 일반 입력 14개를 포함한 비교 결과는
[2단계 보완 기록](../../../mydocs/working/task_m100_8174_stage2_corrections.md)에 연결한다.

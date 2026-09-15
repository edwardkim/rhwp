# #6701 단독 해결 — synam-001 26쪽 세로 흐름

Issue: [#6701](https://github.com/edwardkim/rhwp/issues/6701).
시작 commit: `a95bae7fe` (#6706 단독 완료). 브랜치: `codex/issue-6701-flow-review`.

## 분석

작성자의 정정 댓글을 기준으로 본문·표의 누적 세로 흐름만 조사한다. 최초 본문의 그림
clamp 원인설과 셀 아래맞춤 원인설은 철회되었다. 그림과 같은 줄 글자의 관계를 유지하고,
문의처 → 웹 주소 → 날짜 → 부천시 → 인천지역본부 → 마지막 본문 순서로 기준선을 대조한다.

Git에 있는 `samples/synam-001.hwp`와 `pdf/synam-001-2022.pdf`를 재사용한다.
같은 26쪽과 앞뒤 25·27쪽을 Visual Sweep으로 직접 확인한다. PDF 점 좌표와 96dpi
픽셀 좌표, 페이지 크기 보정 여부를 구분하고 전역 이동으로 원인을 숨기지 않는다.

이미 완료·커밋한 #6706 이후 이 회차를 시작했다. 기존 누적 브랜치의 다른 이슈 변경을
한꺼번에 가져오지 않고 원인과 반례에 필요한 변경만 적용한다.

## 결과

**기존 수정으로 해소됨을 재검증했다. 이번 회차의 제품 코드 수정은 없다.**
과거 재현 버전 `1f861362ab372f9fa26e38f9a534a89286f641c4`를 별도 worktree에서 빌드해
같은 입력·PDF·현재 Visual Sweep·웹폰트로 대조했다. 작성자가 보고한 누적 오차가
그대로 재현되고, 기준 `upstream/devel`인 `769582fc856f162e57604b318d41414d7b026345`에서는
해소되어 있다. 이 사이의 최초 수정 commit은 별도 bisect로 확정하지 않았으므로 특정 PR의
효과로 귀속하지 않는다.

작성자의 [그림 clamp 원인설 철회](https://github.com/edwardkim/rhwp/issues/6701#issuecomment-5530354423)와
[본문 흐름으로 범위 정정](https://github.com/edwardkim/rhwp/issues/6701#issuecomment-5530854272)을
반영했다. 그림 clamp, 줄간격, golden, 허용치를 바꾸지 않았다.

### 같은 26쪽의 기준선 실측

SVG의 상위 transform을 합성한 절대 기준선과 PDF text span origin을 비교했다.
단위는 96dpi px이며 PDF 점 좌표에는 정확히 `4/3`을 곱했다. PDF MediaBox는
595×841pt, rhwp 페이지는 약 793.7×1122.5px이다. 아래 표에는 페이지 크기 정규화나
전역 이동을 적용하지 않았다. Visual Sweep 이미지는 도구의 기본 페이지 대응 방식을 사용한다.

| 대상 | 한컴 PDF | 과거 rhwp | 과거 오차 | 현재 rhwp | 현재 오차 |
|---|---:|---:|---:|---:|---:|
| 문의처 | 183.200 | 181.333 | −1.867 | 181.333 | −1.867 |
| `(www.lh.or.kr)` | 305.120 | 309.973 | +4.853 | 305.173 | +0.053 |
| `2022.07.27` | 470.880 | 480.600 | +9.720 | 471.000 | +0.120 |
| 부천시 | 747.520 | 755.360 | +7.840 | 745.760 | −1.760 |
| 한국토지주택공사 인천지역본부 | 856.000 | 871.507 | +15.507 | 854.187 | −1.813 |
| 마지막 `’` | 910.560 | 943.320 | +32.760 | 910.560 | 0.000 |

위에서 아래로 증가하던 본문 흐름 오차는 사라졌다. 문의처와 하단 기관명의 약 −1.8px
셀 내부 오차는 남아 있으나 다음 본문으로 누적되지 않는다. 현행 trace에서 날짜 pi180은
y=450.6→489.0, 마지막 본문 pi194는 y=895.8→921.9로 전진하며 마지막 기준선이 PDF와 맞는다.
그림 두 개의 현재 render tree y는 694.4/797.7px, PDF y는 695.400/798.647px이다.

### 실행·시각 증적

- 원본: [samples/synam-001.hwp](../../samples/synam-001.hwp), SHA-256
  `1dce9356ec316407b6c684d5a11190a44bb26da643a7749626763e781ab0c13b`.
- 기준: [pdf/synam-001-2022.pdf](../../pdf/synam-001-2022.pdf), SHA-256
  `2f430884f916f00e65796beeb524b65b0f0c4aac48c6283431c40a97a2325fc8`.
  PDF Creator `Hwp 2022 12.0.0.4426`, Producer `Hancom PDF 1.3.0.550`, 35쪽.
  기존 Git 파일을 재사용했으며 변환·복사본을 추가하지 않았다.
- 과거 CLI: 전용 target에서 `cargo build --locked --profile release-test --bin rhwp`
  완료, exit 0, 2분 16초. 과거 전체 회귀는 실행하지 않았다.
- 현재 CLI/WASM: #6706에서 빌드·검증한 `a95bae7fe63d0b2c26f9dfcac4287baf270f57cb`의
  산출물을 재사용했다. 이후 Rust·Cargo 파일 변경이 없음을 확인했다. 이번 회차에 fresh build를
  다시 했다고 주장하지 않는다. [앞 회차 provenance](../pr/assets/issue6706/visual_provenance.json).
- 현재 native Visual Sweep 25–27쪽, 현재 WASM 26쪽, 과거 native 26쪽 모두 exit 0.
  각 비교 이미지를 직접 열어 판독했다. 현재 native/WASM의 26쪽 rhwp PNG는 byte-identical.
- `769582fc8` native와 `a95bae7fe` native의 동일 원본 **35쪽 SVG 모두 byte-identical**.
  #6706이 #6701을 고친 것으로 계산하지 않는다.
- [현재 25쪽](../pr/assets/issue6701/current_p025.png),
  [현재 26쪽](../pr/assets/issue6701/current_p026.png),
  [현재 27쪽](../pr/assets/issue6701/current_p027.png),
  [WASM 26쪽](../pr/assets/issue6701/current_wasm_p026.png),
  [과거 26쪽](../pr/assets/issue6701/history_p026.png).
- [검증·해시·기준선 JSON](../pr/assets/issue6701/verification.json),
  [재현 스크립트·실행 로그·26쪽 SVG/tree·Sweep manifest](../pr/assets/issue6701/verification_logs.tar.gz).

재현 명령은 아래와 같다. 각 경로의 binary/package는 provenance의 SHA-256을 함께 확인한다.

```sh
# 작업 디렉터리: /Users/tsjang/rhwp-issue-6706
# R=/private/tmp/rhwp-issue-6701-20260915
# CLI=/private/tmp/rhwp-issue-6706-20260915/rhwp-final
# WASM=/private/tmp/rhwp-issue-6706-20260915/wasm-pkg
/Users/tsjang/rhwp/venv/bin/python scripts/visual_sweep.py \
  --file-target issue6701 samples/synam-001.hwp pdf/synam-001-2022.pdf \
  --rhwp-bin "$CLI" --pages 25-27 --dpi 96 --out "$R/current"
# WASM: 위 명령에 --wasm-pkg "$WASM"를 추가하고 --pages 26 --out "$R/current-wasm" 사용
# 과거: --rhwp-bin "$R/rhwp-history" --pages 26 --out "$R/history" 사용
```

### 판정 범위와 후속 상태

이슈가 보고한 26쪽 누적 세로 흐름은 해소되어 종료 근거를 마련했다. 25쪽의 수 px 위치 차이,
테두리 농도, 26·27쪽 제목/기관명 굵기와 자형 등은 여전히 PDF와 다르다. 따라서 문서 전체의
시각 완전 일치나 모든 셀 내부 정합을 완료했다고 주장하지 않는다. 코드 변경이 없는 재검증
회차이므로 전체 Rust 회귀·Clippy·Native Skia를 다시 실행하지 않았다.

GitHub 이슈는 아직 open이며 댓글·close·push·PR은 수행하지 않았다. 앞 회차 #6706은
신규 수정 1건, 이번 #6701은 기존 해소 확인 1건으로 구분한다. 이 단계와
[최종 보고서](../report/task_m100_6701_report.md), 증적을 함께 커밋한 뒤 다음 이슈로 넘어간다.

문서 2개의 내부 상대 링크 검사와 staged diff 공백 검사를 통과했다. 실행 중인 Cargo/Rust가
없음을 확인한 뒤 과거 비교 전용 worktree만 제거하고 전용 target 484MB를
`/Users/tsjang/.Trash/rhwp-issue-6701-history-target-20260915`로 이동했다. 본 작업 worktree,
#6706 검증 target, 사용자와 다른 작업의 산출물은 보존했다.

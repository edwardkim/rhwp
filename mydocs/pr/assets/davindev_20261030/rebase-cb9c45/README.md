# 최신 devel 반영 후 검증

시각 검증은 완료했다. 이전 기준의 결과는 상위 README에 보존하며 이 실행과 혼용하지 않는다. Rust 전체 회귀 12,198건과 Native Skia 필수 3종도 통과했다. 세부 결과는 PR별 검토 기록에 있다.

- 기준: `cb9c45cf774a03eddb5d56fdb1fd28919f10a969`.
- 후보 제품 소스: `d189352a1b3b0e09718022f7da47daaa5205fab9`.
- 세 PR의 원본 5개: compat 2022, 각 1쪽, 기존 한컴 웹기안기 Print PDF.
- 기능 보존 대조군: 편람 HWP/HWPX, compat 2024, 각 383쪽, 기존 한컴 2024 Print PDF.
- 모든 비교: 96dpi, PDFium `pdfium_smooth_rgb_exact_dpi_v2`, 색상 경계 보정 전 2px raw 실루엣.
- 글꼴: 원 PR 사례는 한컴바탕·굴림 원본, 편람은 KoPub 6종·HCR Batang·Wingdings 원본과 한컴바탕 subset.
  한컴바탕 subset은 원본 HWPX의 Unicode 집합에 해당하는 986개 문자의 윤곽·advance를 원본과 대조했다.
  private 글꼴은 Git에 추가하지 않는다.
- 한 번의 후보 빌드에서 공유 캐시가 기준 바이너리를 재사용한 것을 SHA 대조로 발견했다.
  그 결과는 무효로 분리하고 후보 root crate의 실제 재컴파일과 새 해시·원 사례 출력을 확인했다.
  무효 결과를 후보 통과 근거로 사용하지 않는다.

실제 입력·빌드·글꼴과 제출 자산의 SHA-256은 [provenance.json](provenance.json)에 기록했다.

## 원 PR 5개 사례의 전쪽 측정

각 원본은 1쪽이고 Native/fresh WASM 전체 쪽수는 독립 PDF와 일치한다.

| 원본 | 기준 Native | 후보 Native | 기준 WASM | 후보 WASM |
| --- | ---: | ---: | ---: | ---: |
| empty_para_tac_picture | 89.09% | 99.72% | 89.51% | 99.63% |
| textbox_valign_no_lineseg | 85.19% | 99.55% | 85.70% | 99.42% |
| form_checkbox_width_1100 | 88.77% | 96.21% | 88.15% | 95.98% |
| form_checkbox_width_3000 | 92.10% | 96.27% | 91.89% | 96.05% |
| form_checkbox_width_9921 | 96.76% | 96.62% | 96.55% | 96.40% |

기준·후보의 전쪽 TSV는 `base/{native,wasm}/<원본>/silhouette.tsv`,
`candidate/{native,wasm}/<원본>/silhouette.tsv`에 바이트 그대로 보존했다.

## 새 devel의 편람 기능 보존

HWP와 HWPX 각각 383쪽이며 Native/fresh WASM의 전체 쪽수는 독립 Print PDF와 같다.
기준·후보 모두 전쪽 raw 최저 99.96%(130쪽)다. 같은 형식·backend끼리 대조한
총 1,532쪽 PNG는 모두 바이트 단위로 동일하다. 점수만 같은 것으로 판정하지 않았다.
전쪽 TSV는 `base/handbook-{native,wasm}/handbook-{hwp,hwpx}/silhouette.tsv`와
`candidate/handbook-{native,wasm}/handbook-{hwp,hwpx}/silhouette.tsv`에 보존했다.

130쪽의 WMF 화면 그림과 205쪽의 흐름도·뒤 본문을 HWP/HWPX 각각 Native/WASM으로
직접 대조하고 같은 영역을 200%로 확대 판독했다. 외곽선·글상자·화살표·본문 소속은 유지된다.
130쪽의 작은 WMF 표현 차이는 기존 #8389와 동일하며 이번 후보에서 추가된 차이가 없다.
대표 compare/overlay/review PNG는 해당 candidate 하위 디렉터리의 130·205 파일이다.

## 원 사례 직접 판독과 한계

- 글상자의 제목 두 줄이 테두리 안에서 가운데 정렬된다. Native의 기준→후보 차이는
  제목 영역에 한정되고 그 밖의 본문·그림 픽셀은 동일하다.
- 빈 문단 그림은 기준에서도 한 번 표시된다. 누락 증상이 이미 해결되었다는 사실과
  선행 #8134의 제목 정렬 개선을 구분한다. 후보에서도 소유 줄 포함·후속 본문 순서를 유지한다.
- 양식 폭 1100/3000/9921HU는 각각 캡션 숨김/경계 잘림/전체 표시다. Native·WASM·Studio에서
  같은 영역을 확대 대조했다. clip이 다음 개체로 누출되지 않고 뒤 개체도 표시된다.
- 한컴 Print와 다른 기존 체크 상자 크기·라디오 외곽선 일부·글자 획 차이는 남는다.
  기준과 후보를 함께 확대 확인했고 새 회귀는 확인하지 않았다. 양식 appearance 전체의
  일치를 완료했다고 주장하지 않는다. 가장 넓은 대조군 하락은 Native 0.14%p/WASM 0.15%p다.
- SVG 5건도 전쪽 90% 이상이며 기존 임베딩 글꼴 2종의 Unicode cmap을 확인했다.
  private 글꼴 프로그램과 이를 포함한 SVG는 자산에 넣지 않았다.

확대 판독 이미지는 [zoom](zoom)에 있다. Studio는 Canvas2D·CanvasKit compat/default의
screen/print 18회 캡처와 backend 비교 12건을 통과했다. 런타임 오류와 hard gate 위반은 0건이다.
CanvasKit은 software 경로이며 GPU 검증으로 보고하지 않는다.

## 재현 명령과 출력 출처

- Native: 현재 후보의 `rhwp export-png`와 명시적 글꼴 공급, print 프로필, 위 compat 버전.
- WASM: 같은 후보의 최적화 WASM을 Studio에 동기화하고 Chromium에서 새로 실행.
- 공통: `scripts/visual_sweep.py`의 silhouette-only 전쪽 TSV 및 같은 출력의 compare/overlay/review.
- Native 바이너리 SHA-256: `8114f5e93b0c544a66c7dad8bfe2a533691a9a5d5568c49a71b55915e28503a0`.
- WASM SHA-256: `ce942e2d6e3d37dc4fd593cae55423b5b9303f510405d868a91c57bf39196f8d`.

제품 소스는 위 `d189352a1b`이며 이후 실측 주석·증거·검토 기록 커밋은 제품 코드와
assertion을 바꾸지 않았다. 코드 변경 뒤 과거 캡처를 재사용한 것이 아니다.

### 실제 전쪽 실행 명령

아래 output 경로는 실행 당시 경로이며 종료 정리 대상이다. 재현 시 동일 소스의 새 빌드와
위 글꼴 환경을 준비하고 출력 디렉터리를 지정한다. 제출 TSV·PNG는 이 문서 아래 정식 경로에 보존한다.

```bash
/home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96 --page-workers 1 --rhwp-bin /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/producers/latest-candidate/rhwp --embed-fonts full --font-path /home/edward/mygithub/ttfs/hwp/HBATANG.TTF --font-path /home/edward/mygithub/ttfs/windows/gulim.ttc --out /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/latest-candidate/native --file-target empty_para_tac_picture samples/issue8118/empty_para_tac_picture.hwp pdf/issue8118/empty_para_tac_picture-webhwp-print.pdf --file-target textbox_valign_no_lineseg samples/issue8122/textbox_valign_no_lineseg.hwp pdf/issue8122/textbox_valign_no_lineseg-webhwp-print.pdf --file-target form_checkbox_width_1100 samples/issue8166/form_checkbox_width_1100.hwp pdf/issue8166/form_checkbox_width_1100-webhwp-print.pdf --file-target form_checkbox_width_3000 samples/issue8166/form_checkbox_width_3000.hwp pdf/issue8166/form_checkbox_width_3000-webhwp-print.pdf --file-target form_checkbox_width_9921 samples/issue8166/form_checkbox_width_9921.hwp pdf/issue8166/form_checkbox_width_9921-webhwp-print.pdf
```

```bash
/home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96 --page-workers 1 --rhwp-bin /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/producers/latest-candidate/rhwp --embed-fonts full --font-path /home/edward/mygithub/ttfs/hwp/HBATANG.TTF --font-path /home/edward/mygithub/ttfs/windows/gulim.ttc --out /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/latest-candidate/wasm --wasm-pkg /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/producers/latest-candidate/pkg --file-target empty_para_tac_picture samples/issue8118/empty_para_tac_picture.hwp pdf/issue8118/empty_para_tac_picture-webhwp-print.pdf --file-target textbox_valign_no_lineseg samples/issue8122/textbox_valign_no_lineseg.hwp pdf/issue8122/textbox_valign_no_lineseg-webhwp-print.pdf --file-target form_checkbox_width_1100 samples/issue8166/form_checkbox_width_1100.hwp pdf/issue8166/form_checkbox_width_1100-webhwp-print.pdf --file-target form_checkbox_width_3000 samples/issue8166/form_checkbox_width_3000.hwp pdf/issue8166/form_checkbox_width_3000-webhwp-print.pdf --file-target form_checkbox_width_9921 samples/issue8166/form_checkbox_width_9921.hwp pdf/issue8166/form_checkbox_width_9921-webhwp-print.pdf
```

```bash
/home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py --silhouette-only --compat 2024 --dpi 96 --page-workers 1 --rhwp-bin /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/producers/latest-candidate/rhwp --embed-fonts full --out /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/latest-candidate/handbook-native --font-path /home/edward/mygithub/ttfs/kopub --font-path /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/handbook-fonts/HBATANG.TTF --font-path /mnt/c/Windows/Fonts/HANBatang.ttf --font-path /mnt/c/Windows/Fonts/wingding.ttf --file-target handbook-hwp 'samples/2025 행정업무운영 편람(최종).hwp' 'pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf' --file-target handbook-hwpx 'samples/2025 행정업무운영 편람(최종).hwpx' 'pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf'
```

```bash
/home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py --silhouette-only --compat 2024 --dpi 96 --page-workers 1 --rhwp-bin /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/producers/latest-candidate/rhwp --embed-fonts full --out /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/latest-candidate/handbook-wasm --font-path /home/edward/mygithub/ttfs/kopub --font-path /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/handbook-fonts/HBATANG.TTF --font-path /mnt/c/Windows/Fonts/HANBatang.ttf --font-path /mnt/c/Windows/Fonts/wingding.ttf --wasm-pkg /home/edward/mygithub/rhwp/output/pr-review/davindev-20261030/producers/latest-candidate/pkg --file-target handbook-hwp 'samples/2025 행정업무운영 편람(최종).hwp' 'pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf' --file-target handbook-hwpx 'samples/2025 행정업무운영 편람(최종).hwpx' 'pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf'
```

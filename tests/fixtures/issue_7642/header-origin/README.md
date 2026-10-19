# 헤더 PAPER 사각형의 독립 저장·Print fixture

- Issue: #7642
- 원본: `header-paper-shape-hancom-2020.hwp`, SHA-256 `3ae94ff8672e99cf96749db48c0dc2ec78ae993ad708cd423393354d5462607f`
- 독립 Print: `header-paper-shape-print-2020.pdf`, SHA-256 `3f4ec50265bbb47f0c2d85fdde834429dfbc7d30165e22648e6cb522724ecd85`
- 원본 생성: `samples/pic-in-head-02.hwp`의 HWPX 구조에서 본문 한 문단·헤더 사각형만 남긴 진단 입력을 한컴 engine2020으로 저장했다. 사각형 XML은 `samples/issue6121/156531618_police_press_header.hwpx`의 정상 구문을 사용하고, Header/colPr는 `hp:ctrl`로 포장했다. 저장본의 실제 도형과 본문을 다시 파싱·출력하여 확인했다. 원본 사용자 문서의 전쪽 개선 증거로 사용하지 않는다.
- 저장 job: `e1f4c46c-0963-46ba-abb3-8f52b8b80725`
- Print job: `32c4e620-85ff-4b8b-b070-83b190433d4f`; engine `2020`, 한컴 `11.0.0.9136`, `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`
- Print PDF는 한 쪽·원문 용지·100%·1-up이다. PDF 저장/내보내기 대조군이 아니다.
- 시각 검증 production: `015764c13c99ad3694b35ee57411728f83cb6579`, Native/fresh WASM 모두 `--compat 2022 --dpi 96 --embed-fonts=full --profile print`, 전1쪽 1/1, 최저 **100.00000%**. PNG를 직접 확대해 PAPER 사각형의 헤더 소유와 위치·크기·본문 전 순서를 판독했다. glyph·선의 raster 톤은 구분한다.
- source HWPUNIT offset `(1500,2250)`, size `(7500,3000)`은 원문 선언이며, 회귀는 화면의 절대 px가 아니라 헤더 프레임 원점·크기와의 비율·본문 전 소속 관계를 검사한다.
- 대표 증적: [Native review](../../../../mydocs/pr/assets/issue7642-header-shape-origin/native-p001-review.png), [fresh WASM review](../../../../mydocs/pr/assets/issue7642-header-shape-origin/wasm-p001-review.png), [확대 비교](../../../../mydocs/pr/assets/issue7642-header-shape-origin/header-shape-origin-detail-review.png)
- 전체 TSV·빌드 출처·명령·MCP 원시 상태·생성 입력·진단 스크립트는 ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage139-*`에 보존한다. 첫 생성 시도의 저장본은 Header가 사라져 부적합으로 분류했고 자동100%를 이 fixture의 검증으로 재사용하지 않았다.

이 저장본과 독립 PDF는 새로운 관계형 검사의 근거 자료다. 기존 합성 검사의 잘못된 물리 용지 원점 기대를 현재 좌표 상수로 갱신하지 않는다. 정식 회귀 추가·실행 및 전체 브랜치 검증 완료는 별도로 기록한다.

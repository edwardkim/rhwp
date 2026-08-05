# 이미지 발산 스윕 r2 — EMF 수정 후 잔여 33건 확정

- **표본**: `hwpdocs/` 전수 10,000건 (hwp+hwpx)
- **바이너리**: `fix/emf-image-conversion e6da48fdf` (devel `aebfcaa33` + WMF `bef011d22` +
  EMF `e6da48fdf`), release, native-skia 불필요
- **실행 환경**: `RHWP_FONT_PATH=ttfs/hwp:ttfs/windows`, 12갈래 xargs (n=25)
- **하네스**: `examples/audit_studio_image_parity` (3경로 mime 대조) +
  `examples/audit_image_magic` (매직 스니핑 2차) —
  사용법은 [이미지 발산 스윕 가이드](../manual/verification/image_divergence_sweep.md)
- **회차**: r1 = EMF 수정 전(같은 표본, flagged 131 · octet-stream 129op, PR #4060 커밋
  메시지에 요약), r2 = EMF 수정 후 재실행(이 문서)

## 1. 결과 — r1 대비

| 지표 | r1 (수정 전) | **r2 (수정 후)** |
|---|---|---|
| flagged 문서 | 131 | **33** |
| octet-stream op (layer 기준) | 129 | **20** |
| 파서 실패(`ok:false`) | 52 | 52 |
| 악화 | — | **0건** |

EMF 부류(16문서·109op)는 전량 해소. 잔여 33건은 아래 4부류로, **EMF 와 무관한 별도
결함**이라 #4060 범위에 넣지 않았다.

## 2. 잔여 33건 부류별 내역

bad mime 합산(문서×경로 중복 포함): studio flow `x-wmf 50 · octet 16 · tiff 6`,
studio layer `x-wmf 50 · octet 20 · tiff 6`, svg `x-wmf 50 · octet 20 · tiff 28 · bmp 5`.

### 2.1 octet-stream — 12문서 20op: 판별기에 매직이 없는 포맷

`audit_image_magic` 으로 정체 확정. PR #4060 커밋 메시지의 "미상 2"는 **DOS EPS
바이너리**로 판명됐다.

| 정체 | op | 비고 |
|---|---|---|
| PostScript/EPS 텍스트 (`%!PS-Adobe-3`) | 17 | 변환기 자체가 없음 |
| DOS EPS 바이너리 (`C5 D0 D3 C6` preamble) | 2 | TIFF/WMF 프리뷰 내장 가능성 — 헤더의 프리뷰 오프셋 활용 여지 |
| PCX v2.8 (버전바이트 `0x03`) | 1 | 판별기가 PCX v5(`0A 05`)만 인식 |

문서 목록 (경로는 `hwpdocs/` 기준):

- korea_policy_downloads/148703503_20101119 매일유업 남양유업 부당한 고객유인.hwp — PS 1
- korea_policy_downloads/148755751_20130214 불법 다단계 소비자 피해 주의보.hwp — PS 1
- korea_policy_downloads/148738070_20120829_무학대선건 보도자료.hwp — PS 1
- korea_policy_downloads/148715914_20110728_주식소유현황_참고자료.hwp — PS 1
- prism_downloads/해양수산부/1192000-201600027_…대중국수산물수출확대….hwp — DOS EPS 1
- prism_downloads/공정거래위원회/1130000-201200013_…2012년_6회_회의자료.hwp — PS 1
- prism_downloads/지식재산처/1430000-201500006_…자유학기제_전체회의_자료집.hwp — PS 2
- prism_downloads/성평등가족부/1382000-200900001_…성평등지표개발기초연구.hwp — PS 8
- prism_downloads/농림축산식품부/1541000-200800028_…2차중간보고회_프리즘등록.hwp — DOS EPS 1
- prism_downloads/해양수산부/1611000-200900116_…해양영재-최종보고서.hwp — PS 1
- prism_downloads/전남광주통합특별시 함평군/4960000-201400003_…제3장 5절 시추보고서.hwp — PCX v2.8 1
- prism_downloads/대전광역시/6300000-201400009_…조직진단최종보고서.hwp — PS 1

### 2.2 image/x-wmf — 6문서 50op: WMF 변환기가 실패하는 바이트

판별은 되는데 `convert_wmf_to_svg` 가 None 을 반환해 원본 WMF 가 그대로 나간다.
#4057 의 경로 문제가 아니라 **변환기 실패** 부류다.

- korea_downloads/질병관리청/156588640_[9.6.보도참고자료] 엠폭스 감염병 위기경보….hwpx
- korea_policy_downloads/156389312_[5.8.보도참고자료] 코로나바이러스감염증-19….hwp
- prism_downloads/재정경제부/1051000-201200058_…2012년_경제정책방향.hwp
- prism_downloads/국가데이터처/1240000-201200013_…소상공인_통계_생산방안_연구….hwp
- prism_downloads/기후에너지환경부/1480000-201500223_…매체통합 위해성평가(Ⅳ)….hwp
- prism_downloads/국토교통부/1611000-200900025_…일반항공(General Aviation)….hwp

### 2.3 image/tiff — 11문서: TIFF 변환 실패 (svg 경로 28op·studio 6op)

`tiff_bytes_to_png_bytes` 실패 부류(구식 압축 추정). svg 쪽 op 가 많은 것은 같은
이미지가 여러 페이지에 반복 등장하기 때문.

- korea_policy_downloads/148719004_11. 10월 재난종합상황 분석 및 전망….hwp
- ordin_downloads/인천광역시/17647009_[별표 3] 금연구역과 흡연구역….hwp
- ordin_downloads/도봉구/18318831_[별지 제2호서식] 부패영향평가 세부 평가서….hwp
- ordin_downloads/성동구/21828173_[별지 제1호서식]표 창 장….hwp
- ordin_downloads/성동구/21828175_[별지 제2호서식] 상장….hwp
- prism_downloads/보건복지부/1351000-201800393_…살아있는 간장 기증자….hwp
- prism_downloads/고용노동부/1490000-200700034_…ILO최종본071220.hwp
- prism_downloads/해양수산부/1611000-200800021_…항행통보(주의해역).hwp
- prism_downloads/고용노동부/1490000-201100008_…평가결과서(정치활동).hwp
- prism_downloads/대구광역시/6270000-202400018_…대구시 에너지산업 융복합단지….hwp
- prism_downloads/기후에너지환경부/1480000-201900698_…자생생물유래독성물질….hwp (BMP 도 포함)

### 2.4 image/bmp — 5문서 5op: SVG 경로 전용 실패

studio `<img>` 는 BMP 를 그리므로 studio 쪽은 정상이고, SVG `<image>` data URI 가 BMP
미지원이라 svg 경로만 flagged. `bmp_bytes_to_png_bytes` 가 실패하는 초대형/변형 BMP.

- korea_downloads/농림축산식품부/156536147_…그린카드로 지역농산물….hwp
- korea_downloads/과학기술정보통신부/156643049_240727 조간 (보도) 100년 전 에디슨….hwpx
- prism_downloads/기획재정부/1051000-200500001_…선진통상국가용역보고서-KIEP.hwp
- prism_downloads/산업통상부/1450000-201700178_…최종보고서_26(1119).hwp
- prism_downloads/기후에너지환경부/1480000-201900698_…자생생물유래독성물질….hwp (2.3 과 중복)

## 3. 후속 후보 (이슈 승격 판단은 작업지시자)

1. **PostScript/EPS 변환기 부재** (13문서 19op, 최다) — resvg 계열로는 안 되고 별도
   래스터라이저 또는 DOS EPS 내장 프리뷰(TIFF/WMF) 추출이 필요.
2. **WMF 변환 실패 50op** — 실패 레코드 종류를 `dump_source_images` 로 좁히면 변환기
   보강 범위가 나온다.
3. **TIFF 구식 압축 / 초대형 BMP** — 기존 변환기의 실패 케이스 보강.
4. **PCX v2.8 판별 추가** — 판별기 1줄(버전바이트 `0x00–0x05` 허용)로 끝날 가능성.

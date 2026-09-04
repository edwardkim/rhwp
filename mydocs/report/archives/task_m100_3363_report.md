# Task #3363 — HWP3 내장 OLE payload 추출 (최종 보고서)

- Issue: [#3363](https://github.com/edwardkim/rhwp/issues/3363) (#3313 서브 이슈)
- Branch: `task/3363-hwp3-ole-payload`
- 계획서: `mydocs/working/task_m100_3363_stage1.md` / `_stage2.md`

## 결론

SO-SUEOP.hwp 1쪽 "이미지"는 외부 연결 그림이 아니라 **추가 정보 블록 id=2(표 82)에
내장된 한컴 글맵시(HMapsi) OLE 개체**였다. payload 추출·재포장 배선으로 사이드카
파일 없이 HWP3에서 직접 렌더되며, HWPX와 렌더 결과가 동일해졌다. 검증 중 WMF
변환기의 POLYPOLYGON 구멍 소실 결함(작업지시자 시각 판정 발견)을 함께 정정했다.

## 구현

1. **`src/parser/hwp3/ole.rs` `extract_ole_payloads()`** — id=2 블록의 CFB에서 root
   서브 스토리지(개체명=그림 참조명)별로 스트림을 root 승격 재포장한 standalone CFB
   생성. 한컴 HWPX 변환과 동형(실측: 스트림 md5 완전 동일·크기 19,968 동일).
   wasm 함정 회피: `cfb::CompoundFile::create()`는 `SystemTime::now()`로 wasm panic
   → 자체 `mini_cfb` 빌더 사용.
2. **`mod.rs` 블록 순회 id=2 분기** — 기존 `Hwp3OleInfo`(인식 시그니처 0xF8995567/68
   검증, 미배선 상태였음) 재사용, `pic_name_to_id` 매칭 → ext "ole"
   BinDataContent + Embedding BinData 주입.
3. **`fixup_hwp3_ole_pictures()`** — payload 확보 그림만 `Control::Picture` →
   `ShapeObject::Ole` 변환. 렌더러의 기존 OLE 경로·#3319/#3321 선택 경로 상속,
   HWPX 저장 시 hp:ole 방출(한컴 동형). payload 미확보 pic_type=1 은 현행 유지 가드.
4. **`ole_container.rs` `preview_wmf` 확장** — 실측에서 OlePres000 이 EMF 가 아니라
   40바이트 헤더 + **표준 WMF**로 확인. EMF 부재 시 WMF 추출 추가
   (`strip_ole_presentation_header_wmf`).
5. **`shape_layout.rs` OLE 폴백에 WMF→SVG 분기** — 기존 `convert_wmf_to_svg` 재사용
   (포맷 일반 — HWP3 전용 분기 아님). HWPX SO-SUEOP도 PrvImage 클립 방식 대신 이
   경로로 전환(시각 판정 통과).
6. **WMF POLYPOLYGON 구멍 정정** (`src/wmf/converter/svg/mod.rs`) — 작업지시자
   시각 판정에서 'ㅇ'·'ㅂ' 폐영역이 검게 채워짐을 발견. MS-WMF 스펙상 POLYPOLYGON
   은 전체 윤곽을 polyfill 모드로 채우는 하나의 도형인데 윤곽별 `<polygon>` 분리
   방출이 원인. 단일 `<path>`(서브패스) 병합으로 정정 — 실측: SETPOLYFILLMODE=
   WINDING + 반대 방향 구멍 윤곽, 글맵시 영역 검정 픽셀 20.1%→17.4%.
7. **스펙 보완 주석** — 표 44 아래: pic_type=1/2 이름=내부 참조명(외부 파일 취급
   금지), 3중 실측 근거.
8. **테스트** — 단위(`task3363_hwp3_embedded_ole_payload_extraction`) + 구 테스트
   `issue_1692`의 "external_path 유지+사이드카 로드" 단언을 오분류 고정으로 판정,
   "내장 OLE 렌더·사이드카 불요"로 교체.

## 검증 결과

| 항목 | 결과 |
|---|---|
| 단위·통합 테스트 | 통과 (hwp3 38건, issue_1692 11건) |
| 전체 `cargo test --tests --profile release-test` | **323 스위트 전부 ok, 실패 0** |
| `cargo fmt --check` | 통과 |
| 1쪽 렌더 (CLI svg) | 글맵시 WMF 폴리곤 렌더, 위치·크기 52.3×174.7mm 정합, HWP3=HWPX 동일 |
| payload 등가성 | 한컴 `ole1.ole` 내부와 스트림 md5 완전 동일 |
| Link 소거 | `getExternalImageBasenames()` 빈 배열 (#3348 가드와 정합) |
| 코퍼스 스모크 | 271/271 크래시 0 (내장 OLE 보유는 SO-SUEOP 유일 — 스코핑 스윕) |
| **studio(wasm) 시각 판정** | **통과** — 글맵시 표시 + 구멍 흰색 (작업지시자, 2026-07-26) |
| **확장(rhwp-chrome dist) 판정** | **글맵시 정상 표시 확인** (작업지시자) |

## 후속 관찰 (범위 밖 기록)

1. **확장 첫 로딩 시 1·2쪽 세로축 어긋남** — 작업지시자 발견(확장 dist, 첫 로딩
   한정). **42쪽 이동 후 1쪽 복귀 시 소멸** — 재렌더로 해소되는 첫 페인트 과도
   상태(1쪽 캔버스가 이후 확정된 줌/배치로 재페인트되지 않고 stale 유지)로 추정.
   dev 서버 headless 20회 시계열 측정(0.3s 간격)에서는 재현 안 됨(1·2쪽 x=313/w=793
   완전 정렬). 별도 이슈 등록 대상.
2. **WMF 변환 공용 수정의 광역 영향** — POLYPOLYGON 병합은 모든 WMF 렌더에
   적용되므로(스펙 정합 방향) 다음 10k 서베이 라운드에서 시각 확인 권장.
3. 사이드카 공급 UX(#3313 잔여)는 진짜 pic_type=0 문서 한정 과제로 축소.
   `samples/00000000.OOO` 워크어라운드 파일의 존치 여부는 별도 판단.

## 릴리즈

0.8.1 포함 여부는 작업지시자 결정 대기 (#3303·#3348과 함께 또는 분리).

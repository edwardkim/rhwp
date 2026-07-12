# PR #1913 검토 — Task #1891: 외부 참조(BinData Link) 그림 HWPX 왕복 소실 수정 + 클러스터 판별

- 작성일: 2026-07-05 / 검토자: Claude (메인테이너 대행 검토)
- PR: planet6897 → devel / MERGEABLE, 충돌 없음 / CI 11 pass (1 skip)
- 연결 이슈: #1891 (규제영향분석서 hwpx 계열 라운드트립 쪽수 팽창)
- 시간순 처리 2번째 (#1912 → **#1913** → #1919 → #1922)

## 1. PR 요약

두 갈래 기여:

1. **클러스터 판별 (조사)**: 이슈의 "비표준 ZIP(EOCD)" 가설 기각 — 비-PASS 14건 중 10건은
   **HWP5(OLE)가 .hwpx 확장자로 업로드**된 부처 산출물(축 A, 쪽수 팽창 전건). 후속 설계
   사안으로 분리(LINE_SEG 부재 기계생성 HWP5의 로드 시멘틱 비대칭).
2. **축 B 수정**: 외부 참조(BinData Link, `isEmbeded="0"`) 그림의 HWPX 왕복 소실(73504) —
   3중 결함 체인(직렬화기 bin_data_map 임베디드 전용 → pic 드롭 / manifest 순번 명명의
   숫자 불변식 위반 / 파서 media-type 의존 수집) 수정.

변경: serializer/hwpx 5파일 + parser/hwpx/content.rs + 신규 픽스처
`samples/issue1891_external_bindata_link.hwpx` + 핀 2건(`tests/issue_1891.rs`).

## 2. 코드 검토

- **숫자 불변식 정립이 핵심 가치**: manifest id를 순번(i+1)이 아닌 `image{bin_data_id}`로
  통일 — 파서(section.rs)가 binaryItemIDRef 숫자를 그대로 bin_data_id로 읽는 기존 계약과
  정합. 기존 lib test(`img_uses_manifest_id`)가 위반을 고정하고 있었음을 식별하고 계약을
  정제한 판단이 옳다. 통상 문서는 id 조밀(1..n)이라 명명 불변 → big_hwpx 2,500 A/B 완전
  동일 주장과 부합.
- **Link 보존 설계 일관**: 등록(context) → ZIP 생략(mod) → isEmbeded 실값 방출(content) →
  3-way 단언·href 실재 검사 제외(mod/package_check) 전 구간 일관. package_check의
  href 추출을 속성 순서 비의존 태그 스캔으로 바꾼 것도 견고성 개선.
- **#1567 센티널(storage_id=0) 충돌 가드** + 기등록 키 skip — 기왕 이슈 인지.
- 관찰(비차단): **HWP5-origin Link** 항목은 참조가 레코드 순번 기반(트러블슈팅
  `bin_data_id_index_mapping.md`, 2026-02-17)이라 storage_id 키 등록이 안 닿을 수 있음 —
  이 경우 종전(드롭)과 동일 동작 + 무참조 manifest 항목 1개 잔존 수준으로 악화는 아님.
  HWP5 Link 왕복은 별도 커버리지 과제로 남김.
- 파서 수집 조건 `!is_embedded && media_type != "application/xml"` — 외부 참조를
  media-type 무관 수집. 2-round 안정성 결함(후속 항목 id 밀림)의 옳은 해소.

## 3. 게이트 결과 (devel `bf5228df` + PR 테스트 머지)

| 게이트 | 결과 |
|---|---|
| GitHub CI | 11 pass / 1 skip |
| cargo fmt --check | 통과 |
| cargo clippy --profile release-test --all-targets | 경고 0 |
| cargo test --profile release-test --tests (hwpx_roundtrip_baseline 신규 픽스처 자동 포함) | **2,871 통과 / 실패 0** |
| hwpx-roundtrip 단독 재현 (신규 픽스처) | **PASS diff=0 r2=0** |

- OVR 게이트는 미적용 — 직렬화기/파서 변경으로 원본 렌더 경로 무영향(왕복 검증은
  roundtrip baseline이 담당). 시각 판정 대상 아님(시각 검증 거버넌스 — 선택 적용).

## 4. 판단 (작업지시자 승인 대기)

- 수정 자체는 3중 결함 체인 해소 + 계약(숫자 불변식) 정제로 건전. #1912/#1919와 파일·도메인
  겹침 없음 — 독립 머지 가능.
- 축 A(HWP5-in-.hwpx 10건) 후속 설계 사안은 이슈 #1891에 기록 유지 — 이슈 close 여부는
  작업지시자 판단(기전 잔존).

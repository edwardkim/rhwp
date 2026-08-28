# Task M100 #4969 Stage W10-Q1 진입 — exact SFNT supply와 shaping oracle

- **기준**: `upstream/devel@6415047a4dfdb71fe96239eced0017559f699c81`
- **제품 동작 변경**: 0
- **private corpus 사용**: 0
- **runtime WOFF2 decoder 추가**: 0

## 결과

승인된 수정 A안에 따라 Source Han은 OFL WOFF2를 OTF로 결정적으로 해제했고, Happiness Sans는 공식 화면용
archive의 variable TTF와 라이선스를 byte 그대로 추적할 준비를 마쳤다. 두 SFNT는 rustybuzz에서 parse되며
기존 WOFF2와 glyph order·전체 default outline mismatch가 0이다.

기계 판독 정본은 다음 두 파일이다.

- [`w10_q1_source_supply.json`](../tech/investigations/issue-4969/w10_q1_source_supply.json)
- [`w10_q1_shaping_contract.json`](../tech/investigations/issue-4969/w10_q1_shaping_contract.json)

## 최초 shaping 인사이트

- Noto의 `office`는 `liga=0`에서 6 glyph, `liga=1`에서 4 glyph다. W9의 nominal glyph identity 유지 조건으로는
  GSUB를 지원할 수 없다는 점이 실제 source로 확인됐다.
- Source Han의 `ᄒᆞᆫ글` 결과는 여러 glyph가 같은 UTF-8 cluster offset을 공유한다. cluster를 문자와 1:1로
  되돌리는 구현을 금지해야 한다.
- Happiness의 두 axis boundary는 glyph ID를 유지하면서 advance를 바꾼다. variation vector가 cache key에서
  빠지면 같은 glyph ID 때문에 잘못된 측정값을 재사용할 수 있다.

## Q1 잔여 범위

이번 절편은 source supply와 oracle을 고정한 **Q1 진입 절편**이다. 다음 절편에서는 32 MiB font, 4,096 text·
glyph, feature 64, axis 16 상한을 실제 request validator에 연결하고 malformed tag·duplicate/out-of-range axis·
손상 SFNT 음성 fixture를 추가한다. 그 전까지 renderer의 기존 structured rejection과 W9 layout은 유지한다.

**후속 상태**: validator와 canonical identity·bounded glyph output까지 완료됐다. 현재 판정은
[`task_m100_4969_w10_q1_identity_output.md`](task_m100_4969_w10_q1_identity_output.md)를 따른다.

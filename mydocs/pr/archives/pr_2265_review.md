# PR #2265 검토 — BinData 지연 로딩 (planet6897, closes #2263)

- 검토일: 2026-07-14 / base: devel / 36파일 +688/−111 / CI 11 green / BEHIND
- 요지: `BinDataContent.data: Vec<u8>` → `BinDataBytes{Loaded|Lazy{resolver,key}}`.
  파서 측이 `BinDataResolver`(원본 ZIP/CFB 보유) 구현, 요청 시 압축 해제.
  배열 순서·길이 의미 보존(위치 기반 조회·직렬화기 전량 소비 호환).

## 구조 검토

- **model→parser 역의존 회피**: 트레이트를 model 에 두고 파서가 구현 — 적절.
- **캐시 비보유 설계**: 소비자가 바이트를 복사 보유하므로 리졸버 캐시는
  이중 상주 — 목적 정합. `len()` 의 Lazy 압축 해제 비용은 문서화됨.
- **기존 의미 보존 확인**: HWP5 skip(`has_stream` 사전 확인), OLE 4-byte
  prefix(#195), HWPX OLE 정규화, 로드 실패 placeholder(#1917) — 표대로.
- **#2230 접점 정확**: `register_embedded_bin_data` push 는 `.into()`(Loaded),
  테스트 `.load()` 적응 — 어제 merge 분 rebase 완료.
- **#2225 접점**: 손상/부재 엔트리는 빈 벡터 반환 → `is_empty` placeholder
  판정 의미 불변.
- 의도적 비지연(HWPX Chart XML, HWP5 lenient) 사유 타당.

## 검증 (로컬 재실증)

| 게이트 | 결과 |
|--------|------|
| 전수 `--tests --no-fail-fast` | **3,157 / 0** |
| fmt / clippy(all-targets) | 통과 / 0 |
| `hwpx_roundtrip_baseline` | 4/0 (저장 시 이미지 유실 없음) |
| 인접 표적 (#2230 6/0, #2225 1/0, #2258 parity) | 전부 통과 |
| **RSS 실측 (devel vs PR)** | 편람 hwpx 1쪽 export **132.5 → 67.4 MB**, treatise 24.8 → 21.1 MB |
| **SVG 바이트 동일성** | treatise 전 7쪽 + 편람 p0 — diff 0 (시각 무회귀 바이트 실증) |

## 판단

**approve → merge 수용 권고.** BEHIND — fork maintainer-edit 불허이므로
merged tree 전수 선검증 후 admin merge (#2257 방식). merge 시 #2263 은
devel push 워크플로로 close.

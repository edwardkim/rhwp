//! 조합형 텍스트 변환 로직
//!
//! `johab_map.rs`의 테이블을 활용하여 실제 조합형 텍스트를 유니코드(UTF-8) 문자로
//! 디코딩하는 함수(`decode_johab`)를 제공한다.

use crate::parser::hwp3::johab_map;

/// KSSM 조합형의 아래아(중성 인덱스 30)를 Unicode 옛한글 자모로 푼다.
///
/// HWP3은 이 음절을 한 개 hchar로 저장하지만, HWP5/HWPX 변환본은
/// 초성·아래아·종성 자모열로 보존한다. `decode_johab`의 완성형 반환 계약을
/// 바꾸지 않기 위해, 가변 길이 텍스트가 필요한 호출자만 이 함수를 사용한다.
pub fn decode_johab_araea_jamo(ch: u16) -> Option<(Option<char>, char, Option<char>)> {
    if ch < 0x8000 {
        return None;
    }

    let cho_idx = ((ch >> 10) & 0x1F) as usize;
    let jung_idx = (ch >> 5) & 0x1F;
    let jong_idx = (ch & 0x1F) as usize;
    if jung_idx != 30 {
        return None;
    }

    let cho_map: [i32; 32] = [
        -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1,
    ];
    let jong_map: [i32; 32] = [
        -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, -1, 17, 18, 19, 20, 21, 22,
        23, 24, 25, 26, 27, -1, -1,
    ];
    let cho = *cho_map.get(cho_idx)?;
    let jong = *jong_map.get(jong_idx)?;
    if jong < 0 {
        return None;
    }

    // [#6380] 초성 인덱스 1 은 '채움'(초성 없음)이라 무효가 아니다. cho_map 이 이를
    // -1 로 내는 바람에 채움 + 아래아 음절이 통째로 버려졌다 — hwp3-sample16 의
    // 0x87C1 이 그 경우로, 한컴 변환본은 같은 자리를 `석ᆞ박사급` 처럼 U+119E 한
    // 글자로 보존한다. 그 밖의 무효 초성(0·21~31)은 종전대로 None 이다.
    const CHO_FILL_INDEX: usize = 1;
    if cho < 0 && cho_idx != CHO_FILL_INDEX {
        return None;
    }

    let leading = if cho < 0 {
        None
    } else {
        Some(char::from_u32(0x1100 + cho as u32)?)
    };
    let araea = char::from_u32(0x119E)?;
    let trailing = if jong == 0 {
        None
    } else {
        char::from_u32(0x11A7 + jong as u32)
    };
    Some((leading, araea, trailing))
}

pub fn decode_johab(ch: u16) -> char {
    if ch < 0x80 {
        return ch as u8 as char;
    } else if ch >= 0x8000 {
        // 조합형 한글 (상위 비트 1)
        let cho_idx = (ch >> 10) & 0x1F;
        let jung_idx = (ch >> 5) & 0x1F;
        let jong_idx = ch & 0x1F;

        let cho_map: [i32; 32] = [
            -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ];
        let jung_map: [i32; 32] = [
            -1, -1, -1, 0, 1, 2, 3, 4, -1, -1, 5, 6, 7, 8, 9, 10, -1, -1, 11, 12, 13, 14, 15, 16,
            -1, -1, 17, 18, 19, 20, -1, -1,
        ];
        let jong_map: [i32; 32] = [
            -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, -1, 17, 18, 19, 20, 21,
            22, 23, 24, 25, 26, 27, -1, -1,
        ];

        let cho = cho_map[cho_idx as usize];
        let jung = jung_map[jung_idx as usize];
        let jong = jong_map[jong_idx as usize];

        // jong_idx == 0 은 예약된 미사용 값이며, jong_map[1] == 0 이 "받침 없음"을
        // 나타낸다. jong == -1 을 무조건 0(받침 없음)으로 치환하면 예약/무효
        // 조합(jong_idx 0, 18, 30, 31)이 유효한 완성형 음절로 잘못 디코딩된다.
        if cho != -1 && jung != -1 && jong != -1 {
            let uni_val = 0xAC00 + (cho * 21 * 28) + (jung * 28) + jong;
            if let Some(c) = std::char::from_u32(uni_val as u32) {
                return c;
            }
        }

        // 한자 및 기호 영역 (이진 탐색)
        if let Ok(idx) = johab_map::JOHAB_SYMBOLS.binary_search_by_key(&ch, |&(k, _)| k) {
            return johab_map::JOHAB_SYMBOLS[idx].1;
        }
    } else if ch >= 0x0080 {
        // [Task #741 Stage 5] HWP3 사적 graphic char 영역 (0x0080~0x7FFF).
        // 표준 KSSM 조합형 영역 (0x8000+) 외 한컴 사적 인코딩.
        // 매핑은 hwp3-sample10.hwp ↔ hwp3-sample10-hwp5.hwp cross-ref 로 도출.
        // Target: HWP5 변환본 IR 정합 (PUA 보존).
        if let Some(c) = decode_hwp3_extra(ch) {
            return c;
        }
    }

    // 매핑되지 않은 값
    '?'
}

/// KS C 5601(KS X 1001) 완성형 좌표 한 쌍을 유니코드 한 글자로 푼다.
///
/// `row`/`cell` 은 EUC-KR 고위 바이트 표기(0xA1..0xFE)다. 완성형에 배정되지 않은
/// 자리는 `None` 을 돌려 호출부가 다음 규칙으로 넘어가게 한다.
fn ksc5601_char(row: u8, cell: u8) -> Option<char> {
    if !(0xA1..=0xFE).contains(&row) || !(0xA1..=0xFE).contains(&cell) {
        return None;
    }
    let bytes = [row, cell];
    let (text, _, had_errors) = encoding_rs::EUC_KR.decode(&bytes);
    if had_errors {
        return None;
    }
    let mut it = text.chars();
    let c = it.next()?;
    if it.next().is_some() {
        return None;
    }
    Some(hancom_variant(c))
}

/// HWP3 기호 영역: KS C 5601 기호행(0xA1..0xAC)의 좌표를 **행 간격 96**으로 편 코드.
///
/// 실측 근거 — 기존 하드코딩 매핑이 이 식에서 그대로 유도된다:
/// `→`0x3446 · `■`0x3441 · `▷`0x3479 · `▶`0x347A · `─`0x35E1,
/// 로마숫자 0x3590..0x3599(Ⅰ..Ⅹ), 원문자 0x36E7..0x36F0(①..⑩) 전부 일치.
/// (한자 영역은 행 간격이 94 라 식이 다르다 — `decode_hwp3_ksc_hanja` 참고.)
fn decode_hwp3_ksc_symbol(ch: u16) -> Option<char> {
    const BASE: u16 = 0x3401;
    const ROW_STRIDE: u16 = 96;
    if ch < BASE {
        return None;
    }
    let idx = ch - BASE;
    let row = 0xA1u16.checked_add(idx / ROW_STRIDE)?;
    let cell = 0xA1u16.checked_add(idx % ROW_STRIDE)?;
    if row > 0xAC {
        return None;
    }
    ksc5601_char(u8::try_from(row).ok()?, u8::try_from(cell).ok()?)
}

/// HWP3 한자 영역: KS C 5601 한자행(0xCA..0xFD)의 좌표를 **행 간격 94**로 편 코드.
///
/// 실측 근거 — HWP3 원본에서 직접 확인한 두 글자가 정확히 맞는다:
/// `債` 0x4F5D → idx 3933 → 0xF3F0, `權` 0x4222 → idx 546 → 0xCFED.
fn decode_hwp3_ksc_hanja(ch: u16) -> Option<char> {
    const BASE: u16 = 0x4000;
    const ROW_STRIDE: u16 = 94;
    if ch < BASE {
        return None;
    }
    let idx = ch - BASE;
    let row = 0xCAu16.checked_add(idx / ROW_STRIDE)?;
    let cell = 0xA1u16.checked_add(idx % ROW_STRIDE)?;
    if row > 0xFD {
        return None;
    }
    ksc5601_char(u8::try_from(row).ok()?, u8::try_from(cell).ok()?)
}

/// HWP3 사적 graphic char (0x0080~0x7FFF 영역) → Unicode 매핑.
///
/// 한컴 변환본 (HWP3 → HWP5) 의 IR 과 정합. PUA (Private Use Area) 영역도
/// 변환본 정합 위해 그대로 보존.
///
/// 매핑 출처: hwp3-sample10.hwp ↔ hwp3-sample10-hwp5.hwp paragraph 별 cross-ref.
///
/// 하드코딩 표를 **먼저** 본 뒤 규칙(기호·한자 완성형 좌표)을 적용한다. 순서가 중요하다 —
/// 예컨대 회사명 graphic 0x37C0..0x37C5 는 기호 규칙으로는 가타카나가 되어 버린다.
fn decode_hwp3_extra(ch: u16) -> Option<char> {
    // [#5555] 라틴 확장(Latin-1 Supplement) — HWP3 은 이 구간 문자를 유니코드 값
    // 그대로(0x00A0..=0x00FF) hchar 에 담는다. 07615 원시 실측: ü(0x00FC)·ä(0x00E4)·
    // ö(0x00F6)·ß(0x00DF)·Ö(0x00D6)이 "Tübingen"·"Europäischer"·"Götz"·
    // "ausschließliche"·"DÖV" 문맥에서 원시값으로 확인되고, 한글 SaveAs 정답지의
    // 8문자 분포(ü·ö·ä·ß·Ö·Ü·Ä·é)와 정합한다 — 실측 8코드 전부 항등이라 구간
    // 항등 통과가 근거를 갖는다. 매핑이 없으면 '?' → 파서가 조용히 버려
    // "für"→"fr" 처럼 글자가 삭제된다. 사적 따옴표(0x0081/0x0082)는 구간 밖이라
    // 아래 하드코딩이 그대로 담당한다.
    if (0x00A0..=0x00FF).contains(&ch) {
        return char::from_u32(ch as u32);
    }
    // [Task #877 Stage 3] 로마숫자 대문자 Ⅰ~Ⅹ: 0x3590~0x3599 → U+2160~U+2169.
    // sample16 (hwp3-sample16.hwp) 의 cross-ref 로 도출. 한컴 HWP5 변환본의
    // paragraph 26/31/36/44 ("Ⅰ. 사업개요", "Ⅱ. 제안 일반사항", "Ⅲ ...", "Ⅳ ...") 정합.
    if (0x3590..=0x3599).contains(&ch) {
        return char::from_u32(0x2160 + (ch - 0x3590) as u32);
    }
    // HWP3 사적 원문자 계열. 연속 코드가 ①~⑩에 대응한다.
    if (0x36E7..=0x36F0).contains(&ch) {
        return char::from_u32(0x2460 + (ch - 0x36E7) as u32);
    }
    // `한글 97 안내문` HWP3 원본의 머리말 회사명 graphic char 여섯 글자.
    // HWP3 암호 fixture의 원시 값 0x37C0..=0x37C5와, 같은 문서의 HWPX
    // 변환본 U+F03EF..=U+F03F4 및 Hancom PDF의 "한글과컴퓨터"를 대조해
    // 확정했다. 렌더러의 공통 한컴 PUA 표가 표시 문자열을 담당하므로 여기서는
    // 해당 PUA를 보존한다.
    if (0x37C0..=0x37C5).contains(&ch) {
        return char::from_u32(0xF03EF + (ch - 0x37C0) as u32);
    }
    let codepoint: u32 = match ch {
        0x0081 => 0x201C, // 왼쪽 큰따옴표
        0x0082 => 0x201D, // 오른쪽 큰따옴표
        // [#5860] 작은따옴표 짝 — 큰따옴표(0x0081/0x0082) 바로 다음 코드다.
        // 07270 위치 정렬(303코드 ↔ 303삭제 1:1)에서 각각 ‘·’ 자리에 놓였다.
        0x0083 => 0x2018,  // ‘
        0x0084 => 0x2019,  // ’
        0x301E => 0xF0811, // 한컴 PUA - 관계도 가지 선문자
        0x301C => 0xF080F, // 한컴 PUA - 굵은 가로선 (94.5% 발생)
        0x3024 => 0xF0817, // 한컴 PUA - 관계도 하단 가지 선문자
        0x3027 => 0xF081A, // 한컴 PUA - 관계도 가로 선문자
        0x3404 => 0x2024,  // 한 점 리더
        0x3446 => 0x2192,  // 오른쪽 화살표
        0x35E1 => 0x2500,  // 상자 그리기 가로선
        0x303D => 0xF0827, // 한컴 PUA
        0x3479 => 0x25B7,  // ▷ WHITE RIGHT-POINTING TRIANGLE
        0x347A => 0x25B6,  // ▶ BLACK RIGHT-POINTING TRIANGLE
        0x3441 => 0x25A0,  // ■ BLACK SQUARE
        // `한글 97 안내문` HWP3 원본의 표 셀 글머리표. HWP5 변환본과
        // 한컴 PDF가 모두 U+25B8(▸)으로 표시하므로, HWP3 사적 코드도 같은
        // 표준 Unicode로 보존한다. 매핑이 없으면 decode_johab가 '?'를 반환하고
        // HWP3 parser가 미지원 사적 코드를 조용히 건너뛰어 글머리표가 사라진다.
        0x2F67 => 0x25B8, // ▸ BLACK RIGHT-POINTING SMALL TRIANGLE
        // [Task #1105] sample16 글머리 prefix.
        // HWP3 0x3366 은 한컴 HWP5 변환본에서 U+F03C5 로 보존되고, 렌더러가
        // 이를 한컴오피스 표시값인 □(U+25A1)로 확장한다. 여기서 ○로 직접
        // 낮추면 HWP3 원본만 정답지와 다른 bullet 로 보이므로 PUA를 보존한다.
        0x3366 => 0xF03C5,
        // 04442 실측: 이 문서는 한자를 완성형 좌표로 담으면서 `※` 만 유니코드 값
        // 그대로(0x203B) 담았다. 구간 전체를 유니코드로 통과시키면 안 된다 —
        // 같은 구간의 0x205A·0x2024·0x2058 은 한글이 각각 ○·・△ 로 표시하는
        // 사적 코드라, 통과시키면 ⁚·․·⁘ 라는 다른 글자가 된다(한글 대조 실측 89건).
        // 사적 코드 → 표시 문자의 일반식이 없으므로 실측된 값만 하나씩 싣는다.
        0x203B => 0x203B,
        // [#5860] 아래는 s36 load/save 전수(한글 2022 오라클)에서 **버려지던** 코드를
        // 계측으로 모으고 오라클 추출 텍스트와 위치 정렬해 짝지은 값이다. 코퍼스
        // HWP3 38문서 중 15문서에서 479자가 이 경로로 사라졌다(07270 은 40쪽→52쪽,
        // 04442 는 도장 기호 한 글자 때문에 1쪽 서식이 2쪽). 근거 문서(횟수)를 값마다
        // 적는다 — 새 값은 같은 방식의 실측으로만 추가한다.
        //
        // 항등으로 확인된 값. "구간 항등"이 아니라 값별 실측이다 — 바로 옆의
        // 0x2024·0x2058·0x205A 는 항등이 아니다.
        0x2010 => 0x2010, // ‐  07615(1)
        0x2013 => 0x2013, // –  07270(78)
        0x2103 => 0x2103, // ℃  07270(9)
        0x2113 => 0x2113, // ℓ  05434(5)
        0x2190 => 0x2190, // ←  07270(5)
        0x2192 => 0x2192, // →  07270(3)
        0x2193 => 0x2193, // ↓  07270(7)
        0x2219 => 0x2219, // ∙  07270(57)
        0x22C5 => 0x22C5, // ⋅  07615(3)
        // 항등이 아닌 값 — 위 주석이 "한글이 ○·・△ 로 표시한다"고 적어 두고도 매핑을
        // 넣지 않아, 측정해 놓고 버리던 자리다.
        0x2024 => 0x30FB, // ・  07270(4)
        0x2058 => 0x25B3, // △  05434(1)
        0x205A => 0x25CB, // ○  04405(2)·04566(3)·05555(5)
        0x2F08 => 0x25AA, // ▪  07270(16)
        0x2F11 => 0x25E6, // ◦  06109(3)
        0x2F14 => 0x25E6, // ◦  07270(49)·07615(60)
        0x3157 => 0x2027, // ‧  04640(1)
        0x0480 => 0x02D0, // ː  07270(71)
        // 일본어 글자·홑낫표. 0x1F2E·0x32B0 은 07270 위치 정렬 1:1 에서 나왔고,
        // 0x3067·0x3068·0x309B·0x309D 는 07615 의 잔여 86자가 정확히 맞물린다.
        0x1F2E => 0x306E, // の  07270(1)
        0x32B0 => 0xFF70, // ｰ (반각)  07270(1)
        0x3067 => 0x2018, // ‘  07615(2)
        0x3068 => 0x2019, // ’  07615(2)
        0x309B => 0xFF62, // ｢  07615(7)
        0x309D => 0xFF63, // ｣  07615(7)
        // 한컴 사설 영역 기호는 평면 15 PUA 로 보존한다 — 렌더러의 공통 한컴 PUA 표가
        // 표시 문자열을 담당한다(0x37C0..0x37C5·0x3366 과 같은 계약).
        // [#6380] 아래는 저장소 `samples/` 의 HWP3 원본을 같은 문서의 한컴 변환본과
        // 문자 멀티셋·문맥으로 대조해 얻은 값이다. 코드 출현 횟수가 변환본의 해당
        // 문자 개수와 정확히 맞는 것만 싣는다.
        //
        // 텍스트 다이어그램 괘선 조각 — 0x301C(→F080F) 와 같은 묶음이고 상수
        // 오프셋(0xC07F3)으로 이어진다. 렌더러 검증표가 ┌┬┐└┘│ 로 편다.
        // sample11 개수: 3·1·10·6·9·17 ↔ 변환본 F0806·F0807·F0808·F080C·F080E·F0810 동수.
        0x3013 => 0xF0806,
        0x3014 => 0xF0807,
        0x3015 => 0xF0808,
        0x3019 => 0xF080C,
        0x301B => 0xF080E,
        0x301D => 0xF0810,
        // 겹줄. 기호 좌표 규칙으로는 가타카나 `ネ` 가 되는 사적 graphic 코드다.
        // sample10(42) · sample11(12) 의 변환본이 같은 자리에 리터럴 U+2550 을 동수로
        // 갖는다 — 0x3048(→F0832) 과 달리 PUA 가 아니라 표준 문자로 저장된다.
        0x37ED => 0x2550,
        // 원문자·괄호문자 계열 (sample11).
        //   0x2E01~0x2E07 은 표준 ①~⑦ 로 저장되고(7코드 연속 문맥 일치),
        //   0x2E00·0x2E0A~0x2E12 는 hancom_pua.rs 가 근거로 적어 둔 p23 NVRAM 라벨 줄의
        //   별도 글리프(F0288~F0291)다. 두 계열이 같은 문서에서 함께 쓰인다.
        0x2E01 => 0x2460,
        0x2E02 => 0x2461,
        0x2E03 => 0x2462,
        0x2E04 => 0x2463,
        0x2E05 => 0x2464,
        0x2E06 => 0x2465,
        0x2E07 => 0x2466,
        0x2E00 => 0xF0288,
        0x2E0A => 0xF0289,
        0x2E0B => 0xF028A,
        0x2E0D => 0xF028C,
        0x2E0E => 0xF028D,
        0x2E0F => 0xF028E,
        0x2E10 => 0xF028F,
        0x2E11 => 0xF0290,
        0x2E12 => 0xF0291,
        // 0x2E0C(→F028B=③)는 근거 문서가 그 자리에 리터럴 ③ 을 써서 관측되지 않았다.
        0x2C21 => 0x24D0, // ⓐ  sample11 개수 1·1·2·2·2·3 ↔ 변환본 ⓐ~ⓕ 동수
        0x2C22 => 0x24D1,
        0x2C23 => 0x24D2,
        0x2C24 => 0x24D3,
        0x2C25 => 0x24D4,
        0x2C26 => 0x24D5,
        0x2C40 => 0x3260, // ㉠  sample11 각 1건
        0x2C41 => 0x3261,
        0x2C42 => 0x3262,
        // 글머리표·장식.
        0x2022 => 0x2022,  // •  sample(4) — 유니코드 값 그대로 담은 자리
        0x2F17 => 0x2022,  // •  sample10(3)
        0x2F06 => 0x25A0,  // ■  sample10 "제목차례" 좌우 장식(2)
        0x25F5 => 0xF0099, // 04759(1)
        // 관인·서명란 도장 기호. 00460·00465·04442·04640·04845·05428·05755 (7문서 8회).
        0x2BCE => 0xF012B,
        0x3048 => 0xF0832, // 06109(61)
        // 하드코딩에 없으면 완성형 좌표 규칙으로 넘어간다. 종전에는 여기서 None →
        // decode_johab 가 '?' → HWP3 파서가 그 문자를 **조용히 버렸다**(10k 스윕
        // A-개체치환 17문서: `채권(債權)조서` → `채권()조서`, ※·○·□ 등 전량 소실).
        _ => return decode_hwp3_ksc_symbol(ch).or_else(|| decode_hwp3_ksc_hanja(ch)),
    };
    char::from_u32(codepoint)
}

/// KS C 5601 완성형 → 한컴이 실제로 쓰는 코드포인트 보정.
///
/// 표준 매핑과 한컴 관행이 갈리는 자리가 있다. `encoding_rs::EUC_KR` 은 표준을 주지만
/// 한글은 전각형을 쓴다 — 한글 오라클 대조 실측(A군 17문서)에서 나온 차이만 담는다.
fn hancom_variant(c: char) -> char {
    match c {
        // 0xA1AD: 표준 U+223C(∼) ↔ 한컴 U+FF5E(～). 실측 80건.
        '\u{223C}' => '\u{FF5E}',
        // [#6380] 0xA2C1: 표준 U+2299(⊙) ↔ 한컴 U+25C9(◉).
        // hwp3-sample11 ↔ hwp3-sample11-hwpx 문맥 정렬 2건.
        '\u{2299}' => '\u{25C9}',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_johab_rejects_reserved_jong_index() {
        // cho_idx=2(초성 0), jung_idx=3(중성 0), jong_idx=0(예약/무효 값).
        // jong_idx=0 은 종성 매핑에서 -1(무효)이며, "받침 없음"은 jong_idx=1 이
        // 별도로 나타낸다. 무효 jong_idx 를 받침 없음으로 오인하면 존재하지
        // 않아야 할 완성형 음절 '가'(U+AC00)가 잘못 생성된다.
        let jong_idx = 0;
        let ch: u16 = 0x8000 | (2 << 10) | (3 << 5) | jong_idx;
        assert_ne!(decode_johab(ch), '가');
    }

    #[test]
    fn decode_johab_araea_preserves_legacy_jamo_sequence() {
        // HWP3 fixture의 첫 글자. 한컴 HWPX 변환본은 "ᄒᆞᆫ"으로 보존한다.
        assert_eq!(
            decode_johab_araea_jamo(0xD3C5),
            Some((Some('ᄒ'), 'ᆞ', Some('ᆫ')))
        );
    }

    #[test]
    fn decode_hwp3_company_graphics_to_the_common_hancom_pua() {
        let decoded: String = (0x37C0..=0x37C5).map(decode_johab).collect();
        assert_eq!(
            decoded,
            "\u{F03EF}\u{F03F0}\u{F03F1}\u{F03F2}\u{F03F3}\u{F03F4}"
        );
    }

    #[test]
    fn decode_hwp3_table_triangle_bullet() {
        assert_eq!(decode_johab(0x2F67), '▸');
    }

    #[test]
    fn ksc_hanja_rule_decodes_real_hwp3_codes() {
        // HWP3 원본(10k 스윕 04442 `[23-1호서식] 채권(債權)조서`)에서 직접 읽은 값.
        // 종전에는 매핑이 없어 '?' → 파서가 조용히 버려 `채권()조서`가 됐다.
        assert_eq!(decode_johab(0x4F5D), '債');
        assert_eq!(decode_johab(0x4222), '權');
    }

    #[test]
    fn ksc_symbol_rule_reproduces_hardcoded_mappings() {
        // 규칙(행 간격 96)이 기존 하드코딩 매핑을 그대로 유도한다 — 규칙이 옳다는 근거.
        assert_eq!(decode_johab(0x3446), '→');
        assert_eq!(decode_johab(0x3441), '■');
        assert_eq!(decode_johab(0x3479), '▷');
        assert_eq!(decode_johab(0x347A), '▶');
        assert_eq!(decode_johab(0x35E1), '─');
        // 범위 매핑도 동일하게 유도된다.
        let roman: String = (0x3590..=0x3599).map(decode_johab).collect();
        assert_eq!(roman, "ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩ");
        let circled: String = (0x36E7..=0x36F0).map(decode_johab).collect();
        assert_eq!(circled, "①②③④⑤⑥⑦⑧⑨⑩");
    }

    #[test]
    fn ksc_symbol_rule_recovers_lost_symbols() {
        // A-개체치환 군에서 사라지던 기호들 (※ 195회·○ 등 코퍼스 실측).
        assert_eq!(decode_johab(0x3438), '※');
        assert_eq!(decode_johab(0x343B), '○');
        assert_eq!(decode_johab(0x3440), '□');
        assert_eq!(decode_johab(0x341C), '【');
        assert_eq!(decode_johab(0x341D), '】');
    }

    #[test]
    fn hardcoded_mapping_wins_over_rule() {
        // 회사명 graphic 0x37C0..0x37C5 는 기호 규칙으로는 가타카나가 된다.
        // 하드코딩이 먼저여야 한컴 PUA 보존 계약이 깨지지 않는다.
        assert_eq!(decode_johab(0x37C1), '\u{F03F0}');
        assert_eq!(decode_johab(0x3366), '\u{F03C5}');
    }

    #[test]
    fn only_measured_private_codes_are_mapped() {
        // ※ 를 유니코드 값 그대로(0x203B) 담은 HWP3 이 있다 — 04442 실측.
        assert_eq!(decode_johab(0x203B), '※');
        // 같은 구간이라도 구간 통과는 안 된다. 아래 셋은 한글이 각각 ○·・△ 로
        // 표시하는 사적 코드라, 유니코드로 읽으면 ⁚·․·⁘ 라는 다른 글자가 된다.
        assert_eq!(decode_johab(0x205A), '○');
        assert_eq!(decode_johab(0x2024), '・');
        assert_eq!(decode_johab(0x2058), '△');
        // 실측 근거가 없는 값은 여전히 매핑하지 않는다.
        for ch in [0x0085_u16, 0x2059, 0x2F09] {
            assert_eq!(
                decode_johab(ch),
                '?',
                "0x{ch:04X} 는 근거 없이 매핑하면 안 된다"
            );
        }
    }

    #[test]
    fn hancom_tilde_variant_matches_hangul() {
        // KS X 1001 0xA1AD 는 표준 매핑이 U+223C(∼)지만 한글은 U+FF5E(～)를 쓴다.
        // 실측 80건 — 보정하지 않으면 되살린 문자가 다른 글자가 된다.
        assert_eq!(hancom_variant('\u{223C}'), '\u{FF5E}');
        assert_eq!(ksc5601_char(0xA1, 0xAD), Some('\u{FF5E}'));
    }
}

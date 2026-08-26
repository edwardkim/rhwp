//! 언어 7칸 — HWP5 CHAR_SHAPE 배열 첨자와 HWPX `hh:fontRef` 속성 이름.

/// HWPX 언어 속성 이름. 인덱스 = HWP5 `font_ids`/`ratios`/`spacings`/
/// `relative_sizes`/`char_offsets` 첨자.
pub const LANG_ATTRS: [&str; 7] = [
    "hangul", "latin", "hanja", "japanese", "other", "symbol", "user",
];

/// 한컴 스펙 표 26 언어 슬롯 설명.
pub const LANG_LABELS_KO: [&str; 7] = ["한글", "영어", "한자", "일어", "기타", "기호", "사용자"];

/// 언어 슬롯 한 칸.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LangSlot {
    pub index: u8,
    pub attr: &'static str,
    pub label_ko: &'static str,
}

/// 7칸 전체를 순서대로 순회할 때 쓰는 표.
pub const LANG_SLOTS: [LangSlot; 7] = [
    LangSlot {
        index: 0,
        attr: "hangul",
        label_ko: "한글",
    },
    LangSlot {
        index: 1,
        attr: "latin",
        label_ko: "영어",
    },
    LangSlot {
        index: 2,
        attr: "hanja",
        label_ko: "한자",
    },
    LangSlot {
        index: 3,
        attr: "japanese",
        label_ko: "일어",
    },
    LangSlot {
        index: 4,
        attr: "other",
        label_ko: "기타",
    },
    LangSlot {
        index: 5,
        attr: "symbol",
        label_ko: "기호",
    },
    LangSlot {
        index: 6,
        attr: "user",
        label_ko: "사용자",
    },
];

/// 속성 이름 → 첨자. 없으면 `None`.
pub fn lang_index(attr: &str) -> Option<usize> {
    LANG_ATTRS.iter().position(|name| *name == attr)
}

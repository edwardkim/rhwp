//! Kangxi Radical to CJK Unified Ideograph normalization.
//!
//! Maps Kangxi Radical characters (U+2F00–U+2FD5) and CJK Radicals Supplement
//! (U+2E80–U+2EFF) to their corresponding CJK Unified Ideographs.
//!
//! Some PDF fonts/CMaps emit Kangxi Radicals instead of standard ideographs.
//! While visually identical, they are different Unicode codepoints and will
//! break text search, string matching, and NLP processing.
//!
//! Reference: Unicode Standard, Chapter 12 (CJK Unified Ideographs)

/// Map a Kangxi Radical or CJK Radical Supplement character to its
/// corresponding CJK Unified Ideograph.
///
/// Returns `Some(ideograph)` if the input is a Kangxi Radical (U+2F00–U+2FD5)
/// or CJK Radical Supplement character with a known mapping, `None` otherwise.
pub fn kangxi_to_unified(c: char) -> Option<char> {
    let cp = c as u32;

    // Kangxi Radicals (U+2F00–U+2FD5): 214 radicals
    // Each radical maps to a specific CJK Unified Ideograph
    if (0x2F00..=0x2FD5).contains(&cp) {
        let idx = (cp - 0x2F00) as usize;
        if idx < KANGXI_TO_UNIFIED.len() {
            return char::from_u32(KANGXI_TO_UNIFIED[idx]);
        }
    }

    // CJK Radicals Supplement (U+2E80–U+2EFF)
    // These are variant forms; map the ones with clear unified equivalents
    if (0x2E80..=0x2EFF).contains(&cp) {
        return kangxi_supplement_to_unified(cp);
    }

    None
}

/// Kangxi Radical U+2F00–U+2FD5 → CJK Unified Ideograph mapping table.
/// Index 0 = U+2F00 (radical 1: 一), index 213 = U+2FD5 (radical 214: 龠).
static KANGXI_TO_UNIFIED: [u32; 214] = [
    0x4E00, // U+2F00 ⼀ → 一 (radical 1: one)
    0x4E28, // U+2F01 ⼁ → 丨 (radical 2: line)
    0x4E36, // U+2F02 ⼂ → 丶 (radical 3: dot)
    0x4E3F, // U+2F03 ⼃ → 丿 (radical 4: slash)
    0x4E59, // U+2F04 ⼄ → 乙 (radical 5: second)
    0x4E85, // U+2F05 ⼅ → 亅 (radical 6: hook)
    0x4E8C, // U+2F06 ⼆ → 二 (radical 7: two)
    0x4EA0, // U+2F07 ⼇ → 亠 (radical 8: lid)
    0x4EBA, // U+2F08 ⼈ → 人 (radical 9: man)
    0x513F, // U+2F09 ⼉ → 儿 (radical 10: legs)
    0x5165, // U+2F0A ⼊ → 入 (radical 11: enter)
    0x516B, // U+2F0B ⼋ → 八 (radical 12: eight)
    0x5182, // U+2F0C ⼌ → 冂 (radical 13: down box)
    0x5196, // U+2F0D ⼍ → 冖 (radical 14: cover)
    0x51AB, // U+2F0E ⼎ → 冫 (radical 15: ice)
    0x51E0, // U+2F0F ⼏ → 几 (radical 16: table)
    0x51F5, // U+2F10 ⼐ → 凵 (radical 17: open box)
    0x5200, // U+2F11 ⼑ → 刀 (radical 18: knife)
    0x529B, // U+2F12 ⼒ → 力 (radical 19: power)
    0x52F9, // U+2F13 ⼓ → 勹 (radical 20: wrap)
    0x5315, // U+2F14 ⼔ → 匕 (radical 21: spoon)
    0x531A, // U+2F15 ⼕ → 匚 (radical 22: right open box)
    0x5338, // U+2F16 ⼖ → 匸 (radical 23: hiding encl)
    0x5341, // U+2F17 ⼗ → 十 (radical 24: ten)
    0x535C, // U+2F18 ⼘ → 卜 (radical 25: divination)
    0x5369, // U+2F19 ⼙ → 卩 (radical 26: seal)
    0x5382, // U+2F1A ⼚ → 厂 (radical 27: cliff)
    0x53B6, // U+2F1B ⼛ → 厶 (radical 28: private)
    0x53C8, // U+2F1C ⼜ → 又 (radical 29: again)
    0x53E3, // U+2F1D ⼝ → 口 (radical 30: mouth)
    0x56D7, // U+2F1E ⼞ → 囗 (radical 31: enclosure)
    0x571F, // U+2F1F ⼟ → 土 (radical 32: earth)
    0x58EB, // U+2F20 ⼠ → 士 (radical 33: scholar)
    0x5902, // U+2F21 ⼡ → 夂 (radical 34: go)
    0x590A, // U+2F22 ⼢ → 夊 (radical 35: go slowly)
    0x5915, // U+2F23 ⼣ → 夕 (radical 36: evening)
    0x5927, // U+2F24 ⼤ → 大 (radical 37: big)
    0x5973, // U+2F25 ⼥ → 女 (radical 38: woman)
    0x5B50, // U+2F26 ⼦ → 子 (radical 39: child)
    0x5B80, // U+2F27 ⼧ → 宀 (radical 40: roof)
    0x5BF8, // U+2F28 ⼨ → 寸 (radical 41: inch)
    0x5C0F, // U+2F29 ⼩ → 小 (radical 42: small)
    0x5C22, // U+2F2A ⼪ → 尢 (radical 43: lame)
    0x5C38, // U+2F2B ⼫ → 尸 (radical 44: corpse)
    0x5C6E, // U+2F2C ⼬ → 屮 (radical 45: sprout)
    0x5C71, // U+2F2D ⼭ → 山 (radical 46: mountain)
    0x5DDB, // U+2F2E ⼮ → 巛 (radical 47: river)
    0x5DE5, // U+2F2F ⼯ → 工 (radical 48: work)
    0x5DF1, // U+2F30 ⼰ → 己 (radical 49: oneself)
    0x5DFE, // U+2F31 ⼱ → 巾 (radical 50: turban)
    0x5E72, // U+2F32 ⼲ → 干 (radical 51: dry)
    0x5E7A, // U+2F33 ⼳ → 幺 (radical 52: short thread)
    0x5E7F, // U+2F34 ⼴ → 广 (radical 53: dotted cliff)
    0x5EF4, // U+2F35 ⼵ → 廴 (radical 54: long stride)
    0x5EFE, // U+2F36 ⼶ → 廾 (radical 55: two hands)
    0x5F0B, // U+2F37 ⼷ → 弋 (radical 56: shoot)
    0x5F13, // U+2F38 ⼸ → 弓 (radical 57: bow)
    0x5F50, // U+2F39 ⼹ → 彐 (radical 58: snout)
    0x5F61, // U+2F3A ⼺ → 彡 (radical 59: bristle)
    0x5F73, // U+2F3B ⼻ → 彳 (radical 60: step)
    0x5FC3, // U+2F3C ⼼ → 心 (radical 61: heart)
    0x6208, // U+2F3D ⼽ → 戈 (radical 62: halberd)
    0x6236, // U+2F3E ⼾ → 戶 (radical 63: door)
    0x624B, // U+2F3F ⼿ → 手 (radical 64: hand)
    0x652F, // U+2F40 ⽀ → 支 (radical 65: branch)
    0x6534, // U+2F41 ⽁ → 攴 (radical 66: rap)
    0x6587, // U+2F42 ⽂ → 文 (radical 67: script)
    0x6597, // U+2F43 ⽃ → 斗 (radical 68: dipper)
    0x65A4, // U+2F44 ⽄ → 斤 (radical 69: axe)
    0x65B9, // U+2F45 ⽅ → 方 (radical 70: square)
    0x65E0, // U+2F46 ⽆ → 无 (radical 71: not)
    0x65E5, // U+2F47 ⽇ → 日 (radical 72: sun)
    0x66F0, // U+2F48 ⽈ → 曰 (radical 73: say)
    0x6708, // U+2F49 ⽉ → 月 (radical 74: moon)
    0x6728, // U+2F4A ⽊ → 木 (radical 75: tree)
    0x6B20, // U+2F4B ⽋ → 欠 (radical 76: lack)
    0x6B62, // U+2F4C ⽌ → 止 (radical 77: stop)
    0x6B79, // U+2F4D ⽍ → 歹 (radical 78: death)
    0x6BB3, // U+2F4E ⽎ → 殳 (radical 79: weapon)
    0x6BCB, // U+2F4F ⽏ → 毋 (radical 80: do not)
    0x6BD4, // U+2F50 ⽐ → 比 (radical 81: compare)
    0x6BDB, // U+2F51 ⽑ → 毛 (radical 82: fur)
    0x6C0F, // U+2F52 ⽒ → 氏 (radical 83: clan)
    0x6C14, // U+2F53 ⽓ → 气 (radical 84: steam)
    0x6C34, // U+2F54 ⽔ → 水 (radical 85: water)
    0x706B, // U+2F55 ⽕ → 火 (radical 86: fire)
    0x722A, // U+2F56 ⽖ → 爪 (radical 87: claw)
    0x7236, // U+2F57 ⽗ → 父 (radical 88: father)
    0x723B, // U+2F58 ⽘ → 爻 (radical 89: double x)
    0x723F, // U+2F59 ⽙ → 爿 (radical 90: half tree trunk)
    0x7247, // U+2F5A ⽚ → 片 (radical 91: slice)
    0x7259, // U+2F5B ⽛ → 牙 (radical 92: fang)
    0x725B, // U+2F5C ⽜ → 牛 (radical 93: cow)
    0x72AC, // U+2F5D ⽝ → 犬 (radical 94: dog)
    0x7384, // U+2F5E ⽞ → 玄 (radical 95: profound)
    0x7389, // U+2F5F ⽟ → 玉 (radical 96: jade)
    0x74DC, // U+2F60 ⽠ → 瓜 (radical 97: melon)
    0x74E6, // U+2F61 ⽡ → 瓦 (radical 98: tile)
    0x7518, // U+2F62 ⽢ → 甘 (radical 99: sweet)
    0x751F, // U+2F63 ⽣ → 生 (radical 100: life)
    0x7528, // U+2F64 ⽤ → 用 (radical 101: use)
    0x7530, // U+2F65 ⽥ → 田 (radical 102: field)
    0x758B, // U+2F66 ⽦ → 疋 (radical 103: bolt of cloth)
    0x7592, // U+2F67 ⽧ → 疒 (radical 104: sickness)
    0x7676, // U+2F68 ⽨ → 癶 (radical 105: dotted tent)
    0x767D, // U+2F69 ⽩ → 白 (radical 106: white)
    0x76AE, // U+2F6A ⽪ → 皮 (radical 107: skin)
    0x76BF, // U+2F6B ⽫ → 皿 (radical 108: dish)
    0x76EE, // U+2F6C ⽬ → 目 (radical 109: eye)
    0x77DB, // U+2F6D ⽭ → 矛 (radical 110: spear)
    0x77E2, // U+2F6E ⽮ → 矢 (radical 111: arrow)
    0x77F3, // U+2F6F ⽯ → 石 (radical 112: stone)
    0x793A, // U+2F70 ⽰ → 示 (radical 113: spirit)
    0x79B8, // U+2F71 ⽱ → 禸 (radical 114: track)
    0x79BE, // U+2F72 ⽲ → 禾 (radical 115: grain)
    0x7A74, // U+2F73 ⽳ → 穴 (radical 116: cave)
    0x7ACB, // U+2F74 ⽴ → 立 (radical 117: stand)
    0x7AF9, // U+2F75 ⽵ → 竹 (radical 118: bamboo)
    0x7C73, // U+2F76 ⽶ → 米 (radical 119: rice)
    0x7CF8, // U+2F77 ⽷ → 糸 (radical 120: silk)
    0x7F36, // U+2F78 ⽸ → 缶 (radical 121: jar)
    0x7F51, // U+2F79 ⽹ → 网 (radical 122: net)
    0x7F8A, // U+2F7A ⽺ → 羊 (radical 123: sheep)
    0x7FBD, // U+2F7B ⽻ → 羽 (radical 124: feather)
    0x8001, // U+2F7C ⽼ → 老 (radical 125: old)
    0x800C, // U+2F7D ⽽ → 而 (radical 126: and)
    0x8012, // U+2F7E ⽾ → 耒 (radical 127: plow)
    0x8033, // U+2F7F ⽿ → 耳 (radical 128: ear)
    0x807F, // U+2F80 ⾀ → 聿 (radical 129: brush)
    0x8089, // U+2F81 ⾁ → 肉 (radical 130: meat)
    0x81E3, // U+2F82 ⾂ → 臣 (radical 131: minister)
    0x81EA, // U+2F83 ⾃ → 自 (radical 132: self)
    0x81F3, // U+2F84 ⾄ → 至 (radical 133: arrive)
    0x81FC, // U+2F85 ⾅ → 臼 (radical 134: mortar)
    0x820C, // U+2F86 ⾆ → 舌 (radical 135: tongue)
    0x821B, // U+2F87 ⾇ → 舛 (radical 136: oppose)
    0x821F, // U+2F88 ⾈ → 舟 (radical 137: boat)
    0x826E, // U+2F89 ⾉ → 艮 (radical 138: stopping)
    0x8272, // U+2F8A ⾊ → 色 (radical 139: color)
    0x8278, // U+2F8B ⾋ → 艸 (radical 140: grass)
    0x864D, // U+2F8C ⾌ → 虍 (radical 141: tiger)
    0x866B, // U+2F8D ⾍ → 虫 (radical 142: insect)
    0x8840, // U+2F8E ⾎ → 血 (radical 143: blood)
    0x884C, // U+2F8F ⾏ → 行 (radical 144: walk encl)
    0x8863, // U+2F90 ⾐ → 衣 (radical 145: clothes)
    0x897E, // U+2F91 ⾑ → 襾 (radical 146: west)
    0x898B, // U+2F92 ⾒ → 見 (radical 147: see)
    0x89D2, // U+2F93 ⾓ → 角 (radical 148: horn)
    0x8A00, // U+2F94 ⾔ → 言 (radical 149: speech)
    0x8C37, // U+2F95 ⾕ → 谷 (radical 150: valley)
    0x8C46, // U+2F96 ⾖ → 豆 (radical 151: bean)
    0x8C55, // U+2F97 ⾗ → 豕 (radical 152: pig)
    0x8C78, // U+2F98 ⾘ → 豸 (radical 153: badger)
    0x8C9D, // U+2F99 ⾙ → 貝 (radical 154: shell)
    0x8D64, // U+2F9A ⾚ → 赤 (radical 155: red)
    0x8D70, // U+2F9B ⾛ → 走 (radical 156: run)
    0x8DB3, // U+2F9C ⾜ → 足 (radical 157: foot)
    0x8EAB, // U+2F9D ⾝ → 身 (radical 158: body)
    0x8ECA, // U+2F9E ⾞ → 車 (radical 159: cart)
    0x8F9B, // U+2F9F ⾟ → 辛 (radical 160: bitter)
    0x8FB0, // U+2FA0 ⾠ → 辰 (radical 161: morning)
    0x8FB5, // U+2FA1 ⾡ → 辵 (radical 162: walk)
    0x9091, // U+2FA2 ⾢ → 邑 (radical 163: city)
    0x9149, // U+2FA3 ⾣ → 酉 (radical 164: wine)
    0x91C6, // U+2FA4 ⾤ → 釆 (radical 165: distinguish)
    0x91CC, // U+2FA5 ⾥ → 里 (radical 166: village)
    0x91D1, // U+2FA6 ⾦ → 金 (radical 167: gold)
    0x9577, // U+2FA7 ⾧ → 長 (radical 168: long)
    0x9580, // U+2FA8 ⾨ → 門 (radical 169: gate)
    0x961C, // U+2FA9 ⾩ → 阜 (radical 170: mound)
    0x96B6, // U+2FAA ⾪ → 隶 (radical 171: slave)
    0x96B9, // U+2FAB ⾫ → 隹 (radical 172: short-tailed bird)
    0x96E8, // U+2FAC ⾬ → 雨 (radical 173: rain)
    0x9751, // U+2FAD ⾭ → 靑 (radical 174: blue)
    0x975E, // U+2FAE ⾮ → 非 (radical 175: wrong)
    0x9762, // U+2FAF ⾯ → 面 (radical 176: face)
    0x9769, // U+2FB0 ⾰ → 革 (radical 177: leather)
    0x97CB, // U+2FB1 ⾱ → 韋 (radical 178: tanned leather)
    0x97ED, // U+2FB2 ⾲ → 韭 (radical 179: leek)
    0x97F3, // U+2FB3 ⾳ → 音 (radical 180: sound)
    0x9801, // U+2FB4 ⾴ → 頁 (radical 181: leaf)
    0x98A8, // U+2FB5 ⾵ → 風 (radical 182: wind)
    0x98DB, // U+2FB6 ⾶ → 飛 (radical 183: fly)
    0x98DF, // U+2FB7 ⾷ → 食 (radical 184: eat)
    0x9996, // U+2FB8 ⾸ → 首 (radical 185: head)
    0x9999, // U+2FB9 ⾹ → 香 (radical 186: fragrant)
    0x99AC, // U+2FBA ⾺ → 馬 (radical 187: horse)
    0x9AA8, // U+2FBB ⾻ → 骨 (radical 188: bone)
    0x9AD8, // U+2FBC ⾼ → 高 (radical 189: tall)
    0x9ADF, // U+2FBD ⾽ → 髟 (radical 190: hair)
    0x9B25, // U+2FBE ⾾ → 鬥 (radical 191: fight)
    0x9B2F, // U+2FBF ⾿ → 鬯 (radical 192: sacrificial wine)
    0x9B32, // U+2FC0 ⿀ → 鬲 (radical 193: cauldron)
    0x9B3C, // U+2FC1 ⿁ → 鬼 (radical 194: ghost)
    0x9B5A, // U+2FC2 ⿂ → 魚 (radical 195: fish)
    0x9CE5, // U+2FC3 ⿃ → 鳥 (radical 196: bird)
    0x9E75, // U+2FC4 ⿄ → 鹵 (radical 197: salt)
    0x9E7F, // U+2FC5 ⿅ → 鹿 (radical 198: deer)
    0x9EA5, // U+2FC6 ⿆ → 麥 (radical 199: wheat)
    0x9EBB, // U+2FC7 ⿇ → 麻 (radical 200: hemp)
    0x9EC3, // U+2FC8 ⿈ → 黃 (radical 201: yellow)
    0x9ECD, // U+2FC9 ⿉ → 黍 (radical 202: millet)
    0x9ED1, // U+2FCA ⿊ → 黑 (radical 203: black)
    0x9EF9, // U+2FCB ⿋ → 黹 (radical 204: embroidery)
    0x9EFD, // U+2FCC ⿌ → 黽 (radical 205: frog)
    0x9F0E, // U+2FCD ⿍ → 鼎 (radical 206: tripod)
    0x9F13, // U+2FCE ⿎ → 鼓 (radical 207: drum)
    0x9F20, // U+2FCF ⿏ → 鼠 (radical 208: rat)
    0x9F3B, // U+2FD0 ⿐ → 鼻 (radical 209: nose)
    0x9F4A, // U+2FD1 ⿑ → 齊 (radical 210: even)
    0x9F52, // U+2FD2 ⿒ → 齒 (radical 211: tooth)
    0x9F8D, // U+2FD3 ⿓ → 龍 (radical 212: dragon)
    0x9F9C, // U+2FD4 ⿔ → 龜 (radical 213: turtle)
    0x9FA0, // U+2FD5 ⿕ → 龠 (radical 214: flute)
];

/// Map CJK Radicals Supplement (U+2E80–U+2EFF) to CJK Unified Ideographs.
/// Only maps characters that have clear unified equivalents.
fn kangxi_supplement_to_unified(cp: u32) -> Option<char> {
    let unified = match cp {
        0x2E80 => 0x2E81_u32, // Keep as-is (CJK radical repeat) — no clear equivalent
        0x2E81 => 0x5382,     // ⺁ → 厂
        0x2E82 => 0x4E5B,     // ⺂ → 乛 (actually 亅 variant)
        0x2E84 => 0x4E01,     // ⺄ → 丁 (variant)
        0x2E85 => 0x4E85,     // ⺅ → 亅
        0x2E86 => 0x4EBA,     // ⺆ → 人 (standing man variant)
        0x2E87 => 0x4EBA,     // ⺇ → 人 (another variant)
        0x2E88 => 0x4EBA,     // ⺈ → 人
        0x2E89 => 0x5200,     // ⺉ → 刀
        0x2E8A => 0x529B,     // ⺊ → 力
        0x2E8B => 0x52F9,     // ⺋ → 勹
        0x2E8C => 0x5165,     // ⺌ → 入
        0x2E8D => 0x516B,     // ⺍ → 八
        0x2E8E => 0x5182,     // ⺎ → 冂
        0x2E8F => 0x5196,     // ⺏ → 冖
        0x2E90 => 0x51AB,     // ⺐ → 冫
        0x2E91 => 0x5200,     // ⺑ → 刀
        0x2E92 => 0x5200,     // ⺒ → 刀
        0x2E93 => 0x529B,     // ⺓ → 力
        0x2E94 => 0x53E3,     // ⺔ → 口
        0x2E95 => 0x56D7,     // ⺕ → 囗
        0x2E96 => 0x571F,     // ⺖ → 土
        0x2E97 => 0x5915,     // ⺗ → 夕
        0x2E98 => 0x5927,     // ⺘ → 大
        0x2E99 => 0x5973,     // ⺙ → 女
        0x2E9B => 0x5B50,     // ⺛ → 子
        0x2E9C => 0x5C0F,     // ⺜ → 小
        0x2E9D => 0x5C38,     // ⺝ → 尸
        0x2E9E => 0x5C71,     // ⺞ → 山
        0x2E9F => 0x5DDB,     // ⺟ → 巛
        0x2EA0 => 0x5DE5,     // ⺠ → 工
        0x2EA1 => 0x5DF1,     // ⺡ → 己
        0x2EA2 => 0x5DFE,     // ⺢ → 巾
        0x2EA3 => 0x5E72,     // ⺣ → 干
        0x2EA4 => 0x5E7A,     // ⺤ → 幺
        0x2EA5 => 0x5E7F,     // ⺥ → 广
        0x2EA6 => 0x5F13,     // ⺦ → 弓
        0x2EA7 => 0x5FC3,     // ⺧ → 心
        0x2EA8 => 0x5FC3,     // ⺨ → 心 (radical form)
        0x2EA9 => 0x5FC3,     // ⺩ → 心 (variant)
        0x2EAA => 0x6208,     // ⺪ → 戈
        0x2EAB => 0x6236,     // ⺫ → 戶
        0x2EAC => 0x793A,     // ⺬ → 示
        0x2EAD => 0x793A,     // ⺭ → 示 (variant)
        0x2EAE => 0x7CF8,     // ⺮ → 糸
        0x2EAF => 0x7CF8,     // ⺯ → 糸 (variant)
        0x2EB0 => 0x7F8A,     // ⺰ → 羊
        0x2EB1 => 0x8001,     // ⺱ → 老
        0x2EB2 => 0x8089,     // ⺲ → 肉
        0x2EB3 => 0x8278,     // ⺳ → 艸
        0x2EB4 => 0x8278,     // ⺴ → 艸 (variant)
        0x2EB5 => 0x8278,     // ⺵ → 艸 (variant)
        0x2EB6 => 0x864D,     // ⺶ → 虍
        0x2EB7 => 0x866B,     // ⺷ → 虫
        0x2EB8 => 0x884C,     // ⺸ → 行
        0x2EB9 => 0x8863,     // ⺹ → 衣
        0x2EBA => 0x8863,     // ⺺ → 衣 (variant)
        0x2EBB => 0x898B,     // ⺻ → 見
        0x2EBC => 0x8A00,     // ⺼ → 言
        0x2EBD => 0x8C37,     // ⺽ → 谷
        0x2EBE => 0x8C9D,     // ⺾ → 貝
        0x2EBF => 0x8D70,     // ⺿ → 走
        0x2EC0 => 0x8DB3,     // ⻀ → 足
        0x2EC1 => 0x91D1,     // ⻁ → 金
        0x2EC2 => 0x9577,     // ⻂ → 長
        0x2EC3 => 0x9580,     // ⻃ → 門
        0x2EC4 => 0x961C,     // ⻄ → 阜
        0x2EC5 => 0x96B9,     // ⻅ → 隹
        0x2EC6 => 0x96E8,     // ⻆ → 雨
        0x2EC7 => 0x975E,     // ⻇ → 非 (simplified form: 青)
        0x2EC8 => 0x9B5A,     // ⻈ → 魚
        0x2EC9 => 0x9CE5,     // ⻉ → 鳥
        0x2ECA => 0x9EA5,     // ⻊ → 麥
        0x2ECB => 0x9EC3,     // ⻋ → 黃
        0x2ECC => 0x9F52,     // ⻌ → 齒
        0x2ECD => 0x9F8D,     // ⻍ → 龍
        0x2ECE => 0x9F9C,     // ⻎ → 龜
        _ => return None,
    };
    // Don't return self-mapping (e.g., 0x2E80 → 0x2E81)
    if unified >= 0x4E00 {
        char::from_u32(unified)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kangxi_radicals() {
        // Examples from issue #123
        assert_eq!(kangxi_to_unified('⾮'), Some('非')); // U+2FAE → U+975E
        assert_eq!(kangxi_to_unified('⽴'), Some('立')); // U+2F74 → U+7ACB
        assert_eq!(kangxi_to_unified('⼀'), Some('一')); // U+2F00 → U+4E00
        assert_eq!(kangxi_to_unified('⽅'), Some('方')); // U+2F45 → U+65B9
        assert_eq!(kangxi_to_unified('⾼'), Some('高')); // U+2FBC → U+9AD8
        assert_eq!(kangxi_to_unified('⽋'), Some('欠')); // U+2F4B → U+6B20
    }

    #[test]
    fn test_kangxi_supplement() {
        assert_eq!(kangxi_to_unified('⺬'), Some('示')); // U+2EAC → U+793A
    }

    #[test]
    fn test_non_kangxi() {
        assert_eq!(kangxi_to_unified('A'), None);
        assert_eq!(kangxi_to_unified('非'), None); // Already unified
        assert_eq!(kangxi_to_unified('あ'), None); // Hiragana
    }

    #[test]
    fn test_first_and_last_radical() {
        assert_eq!(kangxi_to_unified('\u{2F00}'), Some('一')); // First radical
        assert_eq!(kangxi_to_unified('\u{2FD5}'), Some('龠')); // Last radical
    }

    #[test]
    fn test_full_text_normalization() {
        let input = "⾮完備の場合";
        let expected = "非完備の場合";
        let result: String = input
            .chars()
            .map(|c| kangxi_to_unified(c).unwrap_or(c))
            .collect();
        assert_eq!(result, expected);
    }
}

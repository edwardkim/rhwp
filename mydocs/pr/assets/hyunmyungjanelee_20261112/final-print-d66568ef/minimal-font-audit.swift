import Foundation
import CoreText
let names = CTFontManagerCopyAvailablePostScriptNames() as! [String]
let fonts = names.sorted().map { name -> [String: Any] in
 let font = CTFontCreateWithName(name as CFString, 12, nil)
 let url = CTFontCopyAttribute(font, kCTFontURLAttribute) as? URL
 return ["name": name, "url": url?.path ?? ""]
}
let font = CTFontCreateWithName("AppleSDGothicNeo-Regular" as CFString, 12, nil)
let chars = Array("한글ABC0".utf16)
var glyphs = [CGGlyph](repeating: 0, count: chars.count)
let covered = CTFontGetGlyphsForCharacters(font, chars, &glyphs, chars.count)
let data = try JSONSerialization.data(withJSONObject: ["fonts":fonts, "korean_covered":covered, "glyphs":glyphs], options:[.prettyPrinted,.sortedKeys])
FileHandle.standardOutput.write(data)

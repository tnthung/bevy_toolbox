//! # Grammar
//!
//! * `<TOKEN>*` means repeat 0-inf times separated by `TOKEN`, the last `TOKEN` is optional.
//!
//! ```txt
//! color ::=
//!   | '#' + hex{3}    // #rgb
//!   | '#' + hex{4}    // #rgba
//!   | '#' + hex{6}    // #rrggbb
//!   | '#' + hex{8}    // #rrggbbaa
//!   | 'srgb'   '(' number<','>{3, 4} ')'
//!   | 'linear' '(' number<','>{3, 4} ')'
//!   | 'hsl'    '(' number<','>{3, 4} ')'
//!   | 'hsv'    '(' number<','>{3, 4} ')'
//!   | 'hwb'    '(' number<','>{3, 4} ')'
//!   | 'lab'    '(' number<','>{3, 4} ')'
//!   | 'lch'    '(' number<','>{3, 4} ')'
//!   | 'oklab'  '(' number<','>{3, 4} ')'
//!   | 'oklch'  '(' number<','>{3, 4} ')'
//!   | 'xyz'    '(' number<','>{3, 4} ')'
//!   // too many to list here
//!   | <<<149 CSS named colors>>>
//!   ;
//!
//! hex    ::= '0'..'9' | 'a'..'f' | 'A'..'F' ;
//! number ::= INT | FLOAT ;
//! ```
use crate::*;


pub fn color_impl(input: TokenStream) -> TokenStream {
  if input.is_empty() { return TokenStream::new(); }
  Color::parse.parse(input).unwrap().generate().into()
}


pub enum Color {
  Srgba     (Span, (f32, f32, f32, f32)),
  LinearRgba(Span, (f32, f32, f32, f32)),
  Hsla      (Span, (f32, f32, f32, f32)),
  Hsva      (Span, (f32, f32, f32, f32)),
  Hwba      (Span, (f32, f32, f32, f32)),
  Laba      (Span, (f32, f32, f32, f32)),
  Lcha      (Span, (f32, f32, f32, f32)),
  Oklaba    (Span, (f32, f32, f32, f32)),
  Oklcha    (Span, (f32, f32, f32, f32)),
  Xyza      (Span, (f32, f32, f32, f32)),
  Css       (Span, &'static str, (f32, f32, f32, f32)),
  Unfinished(Option<Ident>),
}

impl Parse for Color {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(Token![#]) {
      let hash = input.parse::<Token![#]>()?;
      let span = input.span();

      // the hex can be ident if starts with a letter
      // or a literal if starts with a number
      let hex = if input.peek(Ident) {
        let token = input.parse::<Ident>()?;
        token.to_string()
      } else if input.peek(LitInt) {
        let token = input.parse::<LitInt>()?;
        token.span().source_text().unwrap_or("000".to_string())
      } else {
        return Err(input.error("expected hex color"));
      };

      // make sure all digits are hex
      if hex.chars().any(|c| !c.is_ascii_hexdigit()) {
        return Err(Error::new(span, "invalid hex color"));
      }

      if hex.len() == 3 {
        let mut chars = hex.chars();

        let r = chars.next().unwrap().to_digit(16).unwrap() as f32 * 0x11 as f32 / 0xff as f32;
        let g = chars.next().unwrap().to_digit(16).unwrap() as f32 * 0x11 as f32 / 0xff as f32;
        let b = chars.next().unwrap().to_digit(16).unwrap() as f32 * 0x11 as f32 / 0xff as f32;

        return Ok(Color::Srgba(hash.span, (r, g, b, 1.0)));
      }

      if hex.len() == 4 {
        let mut chars = hex.chars();

        let r = chars.next().unwrap().to_digit(16).unwrap() as f32 * 0x11 as f32 / 0xff as f32;
        let g = chars.next().unwrap().to_digit(16).unwrap() as f32 * 0x11 as f32 / 0xff as f32;
        let b = chars.next().unwrap().to_digit(16).unwrap() as f32 * 0x11 as f32 / 0xff as f32;
        let a = chars.next().unwrap().to_digit(16).unwrap() as f32 * 0x11 as f32 / 0xff as f32;

        return Ok(Color::Srgba(hash.span, (r, g, b, a)));
      }

      if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f32 / 0xff as f32;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f32 / 0xff as f32;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f32 / 0xff as f32;

        return Ok(Color::Srgba(hash.span, (r, g, b, 1.0)));
      }

      if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f32 / 0xff as f32;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f32 / 0xff as f32;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f32 / 0xff as f32;
        let a = u8::from_str_radix(&hex[6..8], 16).unwrap() as f32 / 0xff as f32;

        return Ok(Color::Srgba(hash.span, (r, g, b, a)));
      }

      return Err(Error::new(span, "invalid hex color"));
    }

    if input.peek(Ident) {
      let token = input.parse::<Ident>()?;

      match token.to_string().as_str() {
        code @ ("srgb" | "linear" | "hsl" | "hsv" | "hwb" | "lab" | "lch" | "oklab" | "oklch" | "xyz") => {
          if !input.peek(Paren) {
            return Err(input.error("expected parenthesis"));
          }

          let content;
          parenthesized!(content in input);
          let components: Vec<f32> = content.parse_terminated(|s| {
            if s.peek(LitFloat) { return Ok(s.parse::<LitFloat>()?.base10_parse()?); }
            if s.peek(LitInt  ) { return Ok(s.parse::<LitInt  >()?.base10_parse()?); }
            return Err(s.error("expected float or integer"));
          }, Token![,])?.into_iter().collect();

          if components.len() != 3
          && components.len() != 4 {
            return Err(input.error("expected 3 or 4 components"));
          }

          let a = components.get(0).copied().unwrap_or(1.0);
          let b = components.get(1).copied().unwrap_or(0.0);
          let c = components.get(2).copied().unwrap_or(0.0);
          let mut alpha = 1.0;

          if components.len() == 4 {
            alpha = components.get(3).copied().unwrap_or(1.0);
          }

          match code {
            "srgb"   => return Ok(Color::Srgba     (token.span(), (a, b, c, alpha))),
            "linear" => return Ok(Color::LinearRgba(token.span(), (a, b, c, alpha))),
            "hsl"    => return Ok(Color::Hsla      (token.span(), (a, b, c, alpha))),
            "hsv"    => return Ok(Color::Hsva      (token.span(), (a, b, c, alpha))),
            "hwb"    => return Ok(Color::Hwba      (token.span(), (a, b, c, alpha))),
            "lab"    => return Ok(Color::Laba      (token.span(), (a, b, c, alpha))),
            "lch"    => return Ok(Color::Lcha      (token.span(), (a, b, c, alpha))),
            "oklab"  => return Ok(Color::Oklaba    (token.span(), (a, b, c, alpha))),
            "oklch"  => return Ok(Color::Oklcha    (token.span(), (a, b, c, alpha))),
            "xyz"    => return Ok(Color::Xyza      (token.span(), (a, b, c, alpha))),
            _ => unreachable!()
          }
        }

        "black"                => return Ok(Color::Css(token.span(), " #000000", (0.0, 0.0, 0.0, 1.0))),
        "silver"               => return Ok(Color::Css(token.span(), " #c0c0c0", (0.7529411764705882, 0.7529411764705882, 0.7529411764705882, 1.0))),
        "gray"                 => return Ok(Color::Css(token.span(), " #808080", (0.5019607843137255, 0.5019607843137255, 0.5019607843137255, 1.0))),
        "white"                => return Ok(Color::Css(token.span(), " #ffffff", (1.0, 1.0, 1.0, 1.0))),
        "maroon"               => return Ok(Color::Css(token.span(), " #800000", (0.5019607843137255, 0.0, 0.0, 1.0))),
        "red"                  => return Ok(Color::Css(token.span(), " #ff0000", (1.0, 0.0, 0.0, 1.0))),
        "purple"               => return Ok(Color::Css(token.span(), " #800080", (0.5019607843137255, 0.0, 0.5019607843137255, 1.0))),
        "fuchsia"              => return Ok(Color::Css(token.span(), " #ff00ff", (1.0, 0.0, 1.0, 1.0))),
        "green"                => return Ok(Color::Css(token.span(), " #008000", (0.0, 0.5019607843137255, 0.0, 1.0))),
        "lime"                 => return Ok(Color::Css(token.span(), " #00ff00", (0.0, 1.0, 0.0, 1.0))),
        "olive"                => return Ok(Color::Css(token.span(), " #808000", (0.5019607843137255, 0.5019607843137255, 0.0, 1.0))),
        "yellow"               => return Ok(Color::Css(token.span(), " #ffff00", (1.0, 1.0, 0.0, 1.0))),
        "navy"                 => return Ok(Color::Css(token.span(), " #000080", (0.0, 0.0, 0.5019607843137255, 1.0))),
        "blue"                 => return Ok(Color::Css(token.span(), " #0000ff", (0.0, 0.0, 1.0, 1.0))),
        "teal"                 => return Ok(Color::Css(token.span(), " #008080", (0.0, 0.5019607843137255, 0.5019607843137255, 1.0))),
        "aqua"                 => return Ok(Color::Css(token.span(), " #00ffff", (0.0, 1.0, 1.0, 1.0))),
        "aliceblue"            => return Ok(Color::Css(token.span(), " #f0f8ff", (0.9411764705882353, 0.9725490196078431, 1.0, 1.0))),
        "antiquewhite"         => return Ok(Color::Css(token.span(), " #faebd7", (0.9803921568627451, 0.9215686274509803, 0.8431372549019608, 1.0))),
        "aquamarine"           => return Ok(Color::Css(token.span(), " #7fffd4", (0.4980392156862745, 1.0, 0.8313725490196079, 1.0))),
        "azure"                => return Ok(Color::Css(token.span(), " #f0ffff", (0.9411764705882353, 1.0, 1.0, 1.0))),
        "beige"                => return Ok(Color::Css(token.span(), " #f5f5dc", (0.9607843137254902, 0.9607843137254902, 0.8627450980392157, 1.0))),
        "bisque"               => return Ok(Color::Css(token.span(), " #ffe4c4", (1.0, 0.8941176470588236, 0.7686274509803922, 1.0))),
        "blanchedalmond"       => return Ok(Color::Css(token.span(), " #ffebcd", (1.0, 0.9215686274509803, 0.803921568627451, 1.0))),
        "blueviolet"           => return Ok(Color::Css(token.span(), " #8a2be2", (0.5411764705882353, 0.16862745098039217, 0.8862745098039215, 1.0))),
        "brown"                => return Ok(Color::Css(token.span(), " #a52a2a", (0.6470588235294118, 0.16470588235294117, 0.16470588235294117, 1.0))),
        "burlywood"            => return Ok(Color::Css(token.span(), " #deb887", (0.8705882352941177, 0.7215686274509804, 0.5294117647058824, 1.0))),
        "cadetblue"            => return Ok(Color::Css(token.span(), " #5f9ea0", (0.37254901960784315, 0.6196078431372549, 0.6274509803921569, 1.0))),
        "chartreuse"           => return Ok(Color::Css(token.span(), " #7fff00", (0.4980392156862745, 1.0, 0.0, 1.0))),
        "chocolate"            => return Ok(Color::Css(token.span(), " #d2691e", (0.8235294117647058, 0.4117647058823529, 0.11764705882352941, 1.0))),
        "coral"                => return Ok(Color::Css(token.span(), " #ff7f50", (1.0, 0.4980392156862745, 0.3137254901960784, 1.0))),
        "cornflowerblue"       => return Ok(Color::Css(token.span(), " #6495ed", (0.39215686274509803, 0.5843137254901961, 0.9294117647058824, 1.0))),
        "cornsilk"             => return Ok(Color::Css(token.span(), " #fff8dc", (1.0, 0.9725490196078431, 0.8627450980392157, 1.0))),
        "crimson"              => return Ok(Color::Css(token.span(), " #dc143c", (0.8627450980392157, 0.0784313725490196, 0.23529411764705882, 1.0))),
        "cyan"                 => return Ok(Color::Css(token.span(), " #00ffff", (0.0, 1.0, 1.0, 1.0))),
        "darkblue"             => return Ok(Color::Css(token.span(), " #00008b", (0.0, 0.0, 0.5450980392156862, 1.0))),
        "darkcyan"             => return Ok(Color::Css(token.span(), " #008b8b", (0.0, 0.5450980392156862, 0.5450980392156862, 1.0))),
        "darkgoldenrod"        => return Ok(Color::Css(token.span(), " #b8860b", (0.7215686274509804, 0.5254901960784314, 0.043137254901960784, 1.0))),
        "darkgray"             => return Ok(Color::Css(token.span(), " #a9a9a9", (0.6627450980392157, 0.6627450980392157, 0.6627450980392157, 1.0))),
        "darkgreen"            => return Ok(Color::Css(token.span(), " #006400", (0.0, 0.39215686274509803, 0.0, 1.0))),
        "darkgrey"             => return Ok(Color::Css(token.span(), " #a9a9a9", (0.6627450980392157, 0.6627450980392157, 0.6627450980392157, 1.0))),
        "darkkhaki"            => return Ok(Color::Css(token.span(), " #bdb76b", (0.7411764705882353, 0.7176470588235294, 0.4196078431372549, 1.0))),
        "darkmagenta"          => return Ok(Color::Css(token.span(), " #8b008b", (0.5450980392156862, 0.0, 0.5450980392156862, 1.0))),
        "darkolivegreen"       => return Ok(Color::Css(token.span(), " #556b2f", (0.3333333333333333, 0.4196078431372549, 0.1843137254901961, 1.0))),
        "darkorange"           => return Ok(Color::Css(token.span(), " #ff8c00", (1.0, 0.5490196078431373, 0.0, 1.0))),
        "darkorchid"           => return Ok(Color::Css(token.span(), " #9932cc", (0.6, 0.19607843137254902, 0.8, 1.0))),
        "darkred"              => return Ok(Color::Css(token.span(), " #8b0000", (0.5450980392156862, 0.0, 0.0, 1.0))),
        "darksalmon"           => return Ok(Color::Css(token.span(), " #e9967a", (0.9137254901960784, 0.5882352941176471, 0.47843137254901963, 1.0))),
        "darkseagreen"         => return Ok(Color::Css(token.span(), " #8fbc8f", (0.5607843137254902, 0.7372549019607844, 0.5607843137254902, 1.0))),
        "darkslateblue"        => return Ok(Color::Css(token.span(), " #483d8b", (0.2823529411764706, 0.23921568627450981, 0.5450980392156862, 1.0))),
        "darkslategray"        => return Ok(Color::Css(token.span(), " #2f4f4f", (0.1843137254901961, 0.30980392156862746, 0.30980392156862746, 1.0))),
        "darkslategrey"        => return Ok(Color::Css(token.span(), " #2f4f4f", (0.1843137254901961, 0.30980392156862746, 0.30980392156862746, 1.0))),
        "darkturquoise"        => return Ok(Color::Css(token.span(), " #00ced1", (0.0, 0.807843137254902, 0.8196078431372549, 1.0))),
        "darkviolet"           => return Ok(Color::Css(token.span(), " #9400d3", (0.5803921568627451, 0.0, 0.8274509803921568, 1.0))),
        "deeppink"             => return Ok(Color::Css(token.span(), " #ff1493", (1.0, 0.0784313725490196, 0.5764705882352941, 1.0))),
        "deepskyblue"          => return Ok(Color::Css(token.span(), " #00bfff", (0.0, 0.7490196078431373, 1.0, 1.0))),
        "dimgray"              => return Ok(Color::Css(token.span(), " #696969", (0.4117647058823529, 0.4117647058823529, 0.4117647058823529, 1.0))),
        "dimgrey"              => return Ok(Color::Css(token.span(), " #696969", (0.4117647058823529, 0.4117647058823529, 0.4117647058823529, 1.0))),
        "dodgerblue"           => return Ok(Color::Css(token.span(), " #1e90ff", (0.11764705882352941, 0.5647058823529412, 1.0, 1.0))),
        "firebrick"            => return Ok(Color::Css(token.span(), " #b22222", (0.6980392156862745, 0.13333333333333333, 0.13333333333333333, 1.0))),
        "floralwhite"          => return Ok(Color::Css(token.span(), " #fffaf0", (1.0, 0.9803921568627451, 0.9411764705882353, 1.0))),
        "forestgreen"          => return Ok(Color::Css(token.span(), " #228b22", (0.13333333333333333, 0.5450980392156862, 0.13333333333333333, 1.0))),
        "gainsboro"            => return Ok(Color::Css(token.span(), " #dcdcdc", (0.8627450980392157, 0.8627450980392157, 0.8627450980392157, 1.0))),
        "ghostwhite"           => return Ok(Color::Css(token.span(), " #f8f8ff", (0.9725490196078431, 0.9725490196078431, 1.0, 1.0))),
        "gold"                 => return Ok(Color::Css(token.span(), " #ffd700", (1.0, 0.8431372549019608, 0.0, 1.0))),
        "goldenrod"            => return Ok(Color::Css(token.span(), " #daa520", (0.8549019607843137, 0.6470588235294118, 0.12549019607843137, 1.0))),
        "greenyellow"          => return Ok(Color::Css(token.span(), " #adff2f", (0.6784313725490196, 1.0, 0.1843137254901961, 1.0))),
        "grey"                 => return Ok(Color::Css(token.span(), " #808080", (0.5019607843137255, 0.5019607843137255, 0.5019607843137255, 1.0))),
        "honeydew"             => return Ok(Color::Css(token.span(), " #f0fff0", (0.9411764705882353, 1.0, 0.9411764705882353, 1.0))),
        "hotpink"              => return Ok(Color::Css(token.span(), " #ff69b4", (1.0, 0.4117647058823529, 0.7058823529411765, 1.0))),
        "indianred"            => return Ok(Color::Css(token.span(), " #cd5c5c", (0.803921568627451, 0.3607843137254902, 0.3607843137254902, 1.0))),
        "indigo"               => return Ok(Color::Css(token.span(), " #4b0082", (0.29411764705882354, 0.0, 0.5098039215686274, 1.0))),
        "ivory"                => return Ok(Color::Css(token.span(), " #fffff0", (1.0, 1.0, 0.9411764705882353, 1.0))),
        "khaki"                => return Ok(Color::Css(token.span(), " #f0e68c", (0.9411764705882353, 0.9019607843137255, 0.5490196078431373, 1.0))),
        "lavender"             => return Ok(Color::Css(token.span(), " #e6e6fa", (0.9019607843137255, 0.9019607843137255, 0.9803921568627451, 1.0))),
        "lavenderblush"        => return Ok(Color::Css(token.span(), " #fff0f5", (1.0, 0.9411764705882353, 0.9607843137254902, 1.0))),
        "lawngreen"            => return Ok(Color::Css(token.span(), " #7cfc00", (0.48627450980392156, 0.9882352941176471, 0.0, 1.0))),
        "lemonchiffon"         => return Ok(Color::Css(token.span(), " #fffacd", (1.0, 0.9803921568627451, 0.803921568627451, 1.0))),
        "lightblue"            => return Ok(Color::Css(token.span(), " #add8e6", (0.6784313725490196, 0.8470588235294118, 0.9019607843137255, 1.0))),
        "lightcoral"           => return Ok(Color::Css(token.span(), " #f08080", (0.9411764705882353, 0.5019607843137255, 0.5019607843137255, 1.0))),
        "lightcyan"            => return Ok(Color::Css(token.span(), " #e0ffff", (0.8784313725490196, 1.0, 1.0, 1.0))),
        "lightgoldenrodyellow" => return Ok(Color::Css(token.span(), " #fafad2", (0.9803921568627451, 0.9803921568627451, 0.8235294117647058, 1.0))),
        "lightgray"            => return Ok(Color::Css(token.span(), " #d3d3d3", (0.8274509803921568, 0.8274509803921568, 0.8274509803921568, 1.0))),
        "lightgreen"           => return Ok(Color::Css(token.span(), " #90ee90", (0.5647058823529412, 0.9333333333333333, 0.5647058823529412, 1.0))),
        "lightgrey"            => return Ok(Color::Css(token.span(), " #d3d3d3", (0.8274509803921568, 0.8274509803921568, 0.8274509803921568, 1.0))),
        "lightpink"            => return Ok(Color::Css(token.span(), " #ffb6c1", (1.0, 0.7137254901960784, 0.7568627450980392, 1.0))),
        "lightsalmon"          => return Ok(Color::Css(token.span(), " #ffa07a", (1.0, 0.6274509803921569, 0.47843137254901963, 1.0))),
        "lightseagreen"        => return Ok(Color::Css(token.span(), " #20b2aa", (0.12549019607843137, 0.6980392156862745, 0.6666666666666666, 1.0))),
        "lightskyblue"         => return Ok(Color::Css(token.span(), " #87cefa", (0.5294117647058824, 0.807843137254902, 0.9803921568627451, 1.0))),
        "lightslategray"       => return Ok(Color::Css(token.span(), " #778899", (0.4666666666666667, 0.5333333333333333, 0.6, 1.0))),
        "lightslategrey"       => return Ok(Color::Css(token.span(), " #778899", (0.4666666666666667, 0.5333333333333333, 0.6, 1.0))),
        "lightsteelblue"       => return Ok(Color::Css(token.span(), " #b0c4de", (0.6901960784313725, 0.7686274509803922, 0.8705882352941177, 1.0))),
        "lightyellow"          => return Ok(Color::Css(token.span(), " #ffffe0", (1.0, 1.0, 0.8784313725490196, 1.0))),
        "limegreen"            => return Ok(Color::Css(token.span(), " #32cd32", (0.19607843137254902, 0.803921568627451, 0.19607843137254902, 1.0))),
        "linen"                => return Ok(Color::Css(token.span(), " #faf0e6", (0.9803921568627451, 0.9411764705882353, 0.9019607843137255, 1.0))),
        "magenta"              => return Ok(Color::Css(token.span(), " #ff00ff", (1.0, 0.0, 1.0, 1.0))),
        "mediumaquamarine"     => return Ok(Color::Css(token.span(), " #66cdaa", (0.4, 0.803921568627451, 0.6666666666666666, 1.0))),
        "mediumblue"           => return Ok(Color::Css(token.span(), " #0000cd", (0.0, 0.0, 0.803921568627451, 1.0))),
        "mediumorchid"         => return Ok(Color::Css(token.span(), " #ba55d3", (0.7294117647058823, 0.3333333333333333, 0.8274509803921568, 1.0))),
        "mediumpurple"         => return Ok(Color::Css(token.span(), " #9370db", (0.5764705882352941, 0.4392156862745098, 0.8588235294117647, 1.0))),
        "mediumseagreen"       => return Ok(Color::Css(token.span(), " #3cb371", (0.23529411764705882, 0.7019607843137254, 0.44313725490196076, 1.0))),
        "mediumslateblue"      => return Ok(Color::Css(token.span(), " #7b68ee", (0.4823529411764706, 0.40784313725490196, 0.9333333333333333, 1.0))),
        "mediumspringgreen"    => return Ok(Color::Css(token.span(), " #00fa9a", (0.0, 0.9803921568627451, 0.6039215686274509, 1.0))),
        "mediumturquoise"      => return Ok(Color::Css(token.span(), " #48d1cc", (0.2823529411764706, 0.8196078431372549, 0.8, 1.0))),
        "mediumvioletred"      => return Ok(Color::Css(token.span(), " #c71585", (0.7803921568627451, 0.08235294117647059, 0.5215686274509804, 1.0))),
        "midnightblue"         => return Ok(Color::Css(token.span(), " #191970", (0.09803921568627451, 0.09803921568627451, 0.4392156862745098, 1.0))),
        "mintcream"            => return Ok(Color::Css(token.span(), " #f5fffa", (0.9607843137254902, 1.0, 0.9803921568627451, 1.0))),
        "mistyrose"            => return Ok(Color::Css(token.span(), " #ffe4e1", (1.0, 0.8941176470588236, 0.8823529411764706, 1.0))),
        "moccasin"             => return Ok(Color::Css(token.span(), " #ffe4b5", (1.0, 0.8941176470588236, 0.7098039215686275, 1.0))),
        "navajowhite"          => return Ok(Color::Css(token.span(), " #ffdead", (1.0, 0.8705882352941177, 0.6784313725490196, 1.0))),
        "oldlace"              => return Ok(Color::Css(token.span(), " #fdf5e6", (0.9921568627450981, 0.9607843137254902, 0.9019607843137255, 1.0))),
        "olivedrab"            => return Ok(Color::Css(token.span(), " #6b8e23", (0.4196078431372549, 0.5568627450980392, 0.13725490196078433, 1.0))),
        "orange"               => return Ok(Color::Css(token.span(), " #ffa500", (1.0, 0.6470588235294118, 0.0, 1.0))),
        "orangered"            => return Ok(Color::Css(token.span(), " #ff4500", (1.0, 0.27058823529411763, 0.0, 1.0))),
        "orchid"               => return Ok(Color::Css(token.span(), " #da70d6", (0.8549019607843137, 0.4392156862745098, 0.8392156862745098, 1.0))),
        "palegoldenrod"        => return Ok(Color::Css(token.span(), " #eee8aa", (0.9333333333333333, 0.9098039215686274, 0.6666666666666666, 1.0))),
        "palegreen"            => return Ok(Color::Css(token.span(), " #98fb98", (0.596078431372549, 0.984313725490196, 0.596078431372549, 1.0))),
        "paleturquoise"        => return Ok(Color::Css(token.span(), " #afeeee", (0.6862745098039216, 0.9333333333333333, 0.9333333333333333, 1.0))),
        "palevioletred"        => return Ok(Color::Css(token.span(), " #db7093", (0.8588235294117647, 0.4392156862745098, 0.5764705882352941, 1.0))),
        "papayawhip"           => return Ok(Color::Css(token.span(), " #ffefd5", (1.0, 0.9372549019607843, 0.8352941176470589, 1.0))),
        "peachpuff"            => return Ok(Color::Css(token.span(), " #ffdab9", (1.0, 0.8549019607843137, 0.7254901960784313, 1.0))),
        "peru"                 => return Ok(Color::Css(token.span(), " #cd853f", (0.803921568627451, 0.5215686274509804, 0.24705882352941178, 1.0))),
        "pink"                 => return Ok(Color::Css(token.span(), " #ffc0cb", (1.0, 0.7529411764705882, 0.796078431372549, 1.0))),
        "plum"                 => return Ok(Color::Css(token.span(), " #dda0dd", (0.8666666666666667, 0.6274509803921569, 0.8666666666666667, 1.0))),
        "powderblue"           => return Ok(Color::Css(token.span(), " #b0e0e6", (0.6901960784313725, 0.8784313725490196, 0.9019607843137255, 1.0))),
        "rebeccapurple"        => return Ok(Color::Css(token.span(), " #663399", (0.4, 0.2, 0.6, 1.0))),
        "rosybrown"            => return Ok(Color::Css(token.span(), " #bc8f8f", (0.7372549019607844, 0.5607843137254902, 0.5607843137254902, 1.0))),
        "royalblue"            => return Ok(Color::Css(token.span(), " #4169e1", (0.2549019607843137, 0.4117647058823529, 0.8823529411764706, 1.0))),
        "saddlebrown"          => return Ok(Color::Css(token.span(), " #8b4513", (0.5450980392156862, 0.27058823529411763, 0.07450980392156863, 1.0))),
        "salmon"               => return Ok(Color::Css(token.span(), " #fa8072", (0.9803921568627451, 0.5019607843137255, 0.4470588235294118, 1.0))),
        "sandybrown"           => return Ok(Color::Css(token.span(), " #f4a460", (0.9568627450980393, 0.6431372549019608, 0.3764705882352941, 1.0))),
        "seagreen"             => return Ok(Color::Css(token.span(), " #2e8b57", (0.1803921568627451, 0.5450980392156862, 0.3411764705882353, 1.0))),
        "seashell"             => return Ok(Color::Css(token.span(), " #fff5ee", (1.0, 0.9607843137254902, 0.9333333333333333, 1.0))),
        "sienna"               => return Ok(Color::Css(token.span(), " #a0522d", (0.6274509803921569, 0.3215686274509804, 0.17647058823529413, 1.0))),
        "skyblue"              => return Ok(Color::Css(token.span(), " #87ceeb", (0.5294117647058824, 0.807843137254902, 0.9215686274509803, 1.0))),
        "slateblue"            => return Ok(Color::Css(token.span(), " #6a5acd", (0.41568627450980394, 0.35294117647058826, 0.803921568627451, 1.0))),
        "slategray"            => return Ok(Color::Css(token.span(), " #708090", (0.4392156862745098, 0.5019607843137255, 0.5647058823529412, 1.0))),
        "slategrey"            => return Ok(Color::Css(token.span(), " #708090", (0.4392156862745098, 0.5019607843137255, 0.5647058823529412, 1.0))),
        "snow"                 => return Ok(Color::Css(token.span(), " #fffafa", (1.0, 0.9803921568627451, 0.9803921568627451, 1.0))),
        "springgreen"          => return Ok(Color::Css(token.span(), " #00ff7f", (0.0, 1.0, 0.4980392156862745, 1.0))),
        "steelblue"            => return Ok(Color::Css(token.span(), " #4682b4", (0.27450980392156865, 0.5098039215686274, 0.7058823529411765, 1.0))),
        "tan"                  => return Ok(Color::Css(token.span(), " #d2b48c", (0.8235294117647058, 0.7058823529411765, 0.5490196078431373, 1.0))),
        "thistle"              => return Ok(Color::Css(token.span(), " #d8bfd8", (0.8470588235294118, 0.7490196078431373, 0.8470588235294118, 1.0))),
        "tomato"               => return Ok(Color::Css(token.span(), " #ff6347", (1.0, 0.38823529411764707, 0.2784313725490196, 1.0))),
        "turquoise"            => return Ok(Color::Css(token.span(), " #40e0d0", (0.25098039215686274, 0.8784313725490196, 0.8156862745098039, 1.0))),
        "violet"               => return Ok(Color::Css(token.span(), " #ee82ee", (0.9333333333333333, 0.5098039215686274, 0.9333333333333333, 1.0))),
        "wheat"                => return Ok(Color::Css(token.span(), " #f5deb3", (0.9607843137254902, 0.8705882352941177, 0.7019607843137254, 1.0))),
        "whitesmoke"           => return Ok(Color::Css(token.span(), " #f5f5f5", (0.9607843137254902, 0.9607843137254902, 0.9607843137254902, 1.0))),
        "yellowgreen"          => return Ok(Color::Css(token.span(), " #9acd32", (0.6039215686274509, 0.803921568627451, 0.19607843137254902, 1.0))),
        "transparent"          => return Ok(Color::Css(token.span(), "transparent", (0.0, 0.0, 0.0, 0.0))),

        _ => return Ok(Color::Unfinished(Some(token))),
      }
    }

    Ok(Color::Unfinished(None))
  }
}

impl Generate for Color {
  fn generate(self) -> proc_macro2::TokenStream {
    let (kind, value) = match self {
      Color::Srgba(span, (r, g, b, a)) => {
        let mut value = quote_spanned! {span=> Srgba};
        value.extend(quote! {::new(#r, #g, #b, #a)});
        (quote! {Srgba}, value)
      }

      Color::LinearRgba(span, (r, g, b, a)) => {
        let mut value = quote_spanned! {span=> LinearRgba};
        value.extend(quote! {::new(#r, #g, #b, #a)});
        (quote! {LinearRgba}, value)
      }

      Color::Hsla(span, (h, s, l, a)) => {
        let mut value = quote_spanned! {span=> Hsla};
        value.extend(quote! {::new(#h, #s, #l, #a)});
        (quote! {Hsla}, value)
      }

      Color::Hsva(span, (h, s, v, a)) => {
        let mut value = quote_spanned! {span=> Hsva};
        value.extend(quote! {::new(#h, #s, #v, #a)});
        (quote! {Hsva}, value)
      }

      Color::Hwba(span, (h, w, b, a)) => {
        let mut value = quote_spanned! {span=> Hwba};
        value.extend(quote! {::new(#h, #w, #b, #a)});
        (quote! {Hwba}, value)
      }

      Color::Laba(span, (l, a, b, _a)) => {
        let mut value = quote_spanned! {span=> Laba};
        value.extend(quote! {::new(#l, #a, #b, #_a)});
        (quote! {Laba}, value)
      }

      Color::Lcha(span, (l, c, h, a)) => {
        let mut value = quote_spanned! {span=> Lcha};
        value.extend(quote! {::new(#l, #c, #h, #a)});
        (quote! {Lcha}, value)
      }

      Color::Oklaba(span, (l, a, b, _a)) => {
        let mut value = quote_spanned! {span=> Oklaba};
        value.extend(quote! {::new(#l, #a, #b, #_a)});
        (quote! {Oklaba}, value)
      }

      Color::Oklcha(span, (l, c, h, a)) => {
        let mut value = quote_spanned! {span=> Oklcha};
        value.extend(quote! {::new(#l, #c, #h, #a)});
        (quote! {Oklcha}, value)
      }

      Color::Xyza(span, (x, y, z, a)) => {
        let mut value = quote_spanned! {span=> Xyza};
        value.extend(quote! {::new(#x, #y, #z, #a)});
        (quote! {Xyza}, value)
      }

      Color::Css(span, code, (r, g, b, a)) => {
        let doc = format!("\
          **Hex** `{code}`\\\n\
          **R**   `{r   }`\\\n\
          **G**   `{g   }`\\\n\
          **B**   `{b   }`\\\n\
          **A**   `{a   }`");

        let mut code = quote! {
          #[doc = #doc]
          struct
        };

        code.extend(quote_spanned! {span=> ColorCode});

        (quote! {Srgba}, quote! {{
          #code;
          Srgba::new(#r, #g, #b, #a)
        }})
      }

      Color::Unfinished(name) => {
        return quote! {{
          #[allow(non_camel_case_types)]
          enum PredefinedColor {
            black,
            silver,
            gray,
            white,
            maroon,
            red,
            purple,
            fuchsia,
            green,
            lime,
            olive,
            yellow,
            navy,
            blue,
            teal,
            aqua,
            aliceblue,
            antiquewhite,
            aquamarine,
            azure,
            beige,
            bisque,
            blanchedalmond,
            blueviolet,
            brown,
            burlywood,
            cadetblue,
            chartreuse,
            chocolate,
            coral,
            cornflowerblue,
            cornsilk,
            crimson,
            cyan,
            darkblue,
            darkcyan,
            darkgoldenrod,
            darkgray,
            darkgreen,
            darkgrey,
            darkkhaki,
            darkmagenta,
            darkolivegreen,
            darkorange,
            darkorchid,
            darkred,
            darksalmon,
            darkseagreen,
            darkslateblue,
            darkslategray,
            darkslategrey,
            darkturquoise,
            darkviolet,
            deeppink,
            deepskyblue,
            dimgray,
            dimgrey,
            dodgerblue,
            firebrick,
            floralwhite,
            forestgreen,
            gainsboro,
            ghostwhite,
            gold,
            goldenrod,
            greenyellow,
            grey,
            honeydew,
            hotpink,
            indianred,
            indigo,
            ivory,
            khaki,
            lavender,
            lavenderblush,
            lawngreen,
            lemonchiffon,
            lightblue,
            lightcoral,
            lightcyan,
            lightgoldenrodyellow,
            lightgray,
            lightgreen,
            lightgrey,
            lightpink,
            lightsalmon,
            lightseagreen,
            lightskyblue,
            lightslategray,
            lightslategrey,
            lightsteelblue,
            lightyellow,
            limegreen,
            linen,
            magenta,
            mediumaquamarine,
            mediumblue,
            mediumorchid,
            mediumpurple,
            mediumseagreen,
            mediumslateblue,
            mediumspringgreen,
            mediumturquoise,
            mediumvioletred,
            midnightblue,
            mintcream,
            mistyrose,
            moccasin,
            navajowhite,
            oldlace,
            olivedrab,
            orange,
            orangered,
            orchid,
            palegoldenrod,
            palegreen,
            paleturquoise,
            palevioletred,
            papayawhip,
            peachpuff,
            peru,
            pink,
            plum,
            powderblue,
            rebeccapurple,
            rosybrown,
            royalblue,
            saddlebrown,
            salmon,
            sandybrown,
            seagreen,
            seashell,
            sienna,
            skyblue,
            slateblue,
            slategray,
            slategrey,
            snow,
            springgreen,
            steelblue,
            tan,
            thistle,
            tomato,
            turquoise,
            violet,
            wheat,
            whitesmoke,
            yellowgreen,

            transparent,

            srgb(f32, f32, f32, f32),
            linear(f32, f32, f32, f32),
            hsl(f32, f32, f32, f32),
            hsv(f32, f32, f32, f32),
            hwb(f32, f32, f32, f32),
            lab(f32, f32, f32, f32),
            lch(f32, f32, f32, f32),
            oklab(f32, f32, f32, f32),
            oklch(f32, f32, f32, f32),
            xyz(f32, f32, f32, f32),
          }

          PredefinedColor::#name
        }}
      }
    };

    quote! {{
      use bevy::color::*;
      Color::#kind(#value)
    }}
  }
}

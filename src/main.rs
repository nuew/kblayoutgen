#[macro_use]
extern crate clap;
extern crate csv;
extern crate rustc_serialize;

use std::io::Write;

#[derive(Debug, RustcDecodable)]
struct KeyRow(char, String, String, String, String);

#[derive(Debug)]
pub struct Key {
  // Input
  pub us_key: char,

  // Outputs
  pub out: char,
  pub shift_out: char,
  pub altgr_out: char,
  pub shift_altgr_out: char
}

impl Key {
  fn new(from: KeyRow) -> Result<Key, ::std::num::ParseIntError> {
    fn description_to_char(desc: String) ->
        Result<char, ::std::num::ParseIntError> {
      let parts: Vec<&str> = desc.split(' ').collect();
      let (_, code_point) = parts[0].split_at(2);
      let character = try!(u32::from_str_radix(code_point, 16));
      Ok(::std::char::from_u32(character).unwrap())
    }

    Ok(Key {
      us_key: from.0,
      out: try!(description_to_char(from.1)),
      shift_out: try!(description_to_char(from.2)),
      altgr_out: try!(description_to_char(from.3)),
      shift_altgr_out: try!(description_to_char(from.4)),
    })
  }
}

#[derive(Debug)]
pub struct Keyboard {
  name: String,
  keys: Vec<Key>
}

impl Keyboard {
  pub fn new<R: ::std::io::Read>(name: String, rdr: R) -> Keyboard {
    let mut keys = Vec::<Key>::new();

    let mut row_number = 0;
    for row in ::csv::Reader::from_reader(rdr).delimiter(b':').decode() {
      let key: KeyRow = row.unwrap();
      keys.push(match Key::new(key) {
        Ok(v) => v,
        Err(e) => {
          println!("ERROR on row {}: {}", row_number, e);
          ::std::process::exit(1);
        }
      });
      row_number += 1;
    }

    Keyboard {
      name: name,
      keys: keys
    }
  }

  pub fn output_xkb<W: Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
    fn key_from_char(us_key: char) -> String {
      String::from(match us_key {
        '`' => "TLDE",
        '1' => "AE01",
        '2' => "AE02",
        '3' => "AE03",
        '4' => "AE04",
        '5' => "AE05",
        '6' => "AE06",
        '7' => "AE07",
        '8' => "AE08",
        '9' => "AE09",
        '0' => "AE10",
        '-' => "AE11",
        '=' => "AE12",
        'q' => "AD01",
        'w' => "AD02",
        'e' => "AD03",
        'r' => "AD04",
        't' => "AD05",
        'y' => "AD06",
        'u' => "AD07",
        'i' => "AD08",
        'o' => "AD09",
        'p' => "AD10",
        '[' => "AD11",
        ']' => "AD12",
        '\\' => "BKSL",
        'a' => "AC01",
        's' => "AC02",
        'd' => "AC03",
        'f' => "AC04",
        'g' => "AC05",
        'h' => "AC06",
        'j' => "AC07",
        'k' => "AC08",
        'l' => "AC09",
        ';' => "AC10",
        '\'' => "AC11",
        'z' => "AB01",
        'x' => "AB02",
        'c' => "AB03",
        'v' => "AB04",
        'b' => "AB05",
        'n' => "AB06",
        'm' => "AB07",
        ',' => "AB08",
        '.' => "AB09",
        '/' => "AB10",
        _ => panic!("{} is not a supported key on the US keyboard.")
      })
    }

    fn code_point_from_char(character: char) -> String {
      format!("U{:X}", character as u32)
    }

    const XKB_HEADER: &'static str = "default partial modifier_keys\nxkb_symbols \"basic\" {
        include \"level3(ralt_switch)\"\n";
    const XKB_FOOTER: &'static str = "};";

    // Write Header
    try!(writer.write_all(XKB_HEADER.as_bytes()));

    // Write Name
    try!(write!(writer, "        name[Group1]= \"{}\";\n\n", self.name));

    // Write Keys
    for key in &self.keys {
      try!(write!(writer,
        "        key <{}> {{ [ {}, {}, {}, {} ] }};\n", 
        key_from_char(key.us_key),
        code_point_from_char(key.out),
        code_point_from_char(key.shift_out),
        code_point_from_char(key.altgr_out),
        code_point_from_char(key.shift_altgr_out)
      ));
    }

    // Write Footer
    try!(writer.write_all(XKB_FOOTER.as_bytes()));

    Ok(())
  }
}

fn main() {
  let matches = clap_app!(kblayoutgen =>
    (@setting ArgRequiredElseHelp)
    (@setting UnifiedHelpMessage)
    (@setting TrailingVarArg)
    (@setting ColorNever)
    (version: env!("CARGO_PKG_VERSION"))
    (author: env!("CARGO_PKG_AUTHORS"))
    (about: env!("CARGO_PKG_DESCRIPTION"))
    (@arg INPUT: +required "The definitions file")
    (@arg NAME: +required +multiple "The layout's name")
    (@arg xkb: -x --xkb +takes_value "Output an XKB layout")
  ).get_matches();

  let keyboard = {
    let file = std::fs::File::open(matches.value_of("INPUT").unwrap()).unwrap();
    let name = {
      let mut iter = matches.values_of("NAME").unwrap();
      let mut name = iter.next().unwrap().to_string();

      for part in iter {
        name.push(' ');
        name.push_str(part);
      }

      name
    };

    Keyboard::new(name, file)
  };
  
  // X Keyboard Output
  if let Some(xkb_out) = matches.value_of("xkb") {
    let mut file = std::fs::File::create(xkb_out).unwrap();
    keyboard.output_xkb(&mut file).unwrap();
  }
}

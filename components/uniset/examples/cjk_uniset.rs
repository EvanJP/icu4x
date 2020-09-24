// Demonstrate usage of UnicodeSet on Chinese, Japanese, Korean
// characters in the Basic Multilingual Plane.
use icu_unicodeset::UnicodeSet;
use icu_unicodeset::UnicodeSetError;

fn main() -> Result<(), UnicodeSetError> {
  let cjk1 = vec![0x4E00, 0x62FF];

  let cjk1_set = UnicodeSet::from_inversion_list(cjk1)?;
}

use changelogen::parse::interpolate;
use jiff::civil::Date;
use semver::Version;

#[test]
fn interpolation_basic() {
    let prev = Version::parse("1.0.0").unwrap();
    let new = Version::parse("1.1.0").unwrap();
    let date = Date::new(2025, 8, 25).unwrap();
    let out = interpolate(
        "Release {{newVersion}} (prev {{previousVersion}}) on {{date}}",
        &prev,
        &new,
        &date,
    );
    assert_eq!(out, format!("Release 1.1.0 (prev 1.0.0) on 2025-08-25"));
}

#[test]
fn interpolation_unknown() {
    let prev = Version::parse("1.0.0").unwrap();
    let new = Version::parse("1.0.1").unwrap();
    let date = Date::new(2025, 8, 25).unwrap();
    let out = interpolate("X {{unknown}} Y", &prev, &new, &date);
    assert_eq!(out, "X {{unknown}} Y");
}

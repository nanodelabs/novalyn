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

#[test]
fn interpolation_date_format() {
    let prev = Version::parse("0.1.0").unwrap();
    let new = Version::parse("0.2.0").unwrap();
    // Leading zeros for month/day enforced
    let date = Date::new(2025, 1, 5).unwrap();
    let out = interpolate("{{date}}", &prev, &new, &date);
    assert_eq!(out, "2025-01-05");
}

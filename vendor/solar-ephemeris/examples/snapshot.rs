//! Prints one sky snapshot as JSON: `cargo run --example snapshot --
//! 2460676.5 52.5 13.4`. Kept from the crates.io layout so the vendored
//! manifest stays byte-identical to the published one.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (jd, lat, lon) = match (args.first(), args.get(1), args.get(2)) {
        (Some(j), Some(la), Some(lo)) => (
            j.parse().expect("jd"),
            la.parse().expect("lat"),
            lo.parse().expect("lon"),
        ),
        _ => {
            eprintln!("usage: snapshot <jd_utc> <lat> <lon_east>");
            std::process::exit(2);
        }
    };
    print!("{}", solar_ephemeris::sky_snapshot_json(jd, lat, lon, 0.0));
}

use super::*;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("beaconrange-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn default_file_is_four() {
    let (c, problems) = parse(DEFAULT_CONFIG);
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(c.multiplier, 4.0);
}

#[test]
fn four_times_vanilla_for_every_level() {
    let c = Config::default();
    for (levels, want) in [(1, 80.0), (2, 120.0), (3, 160.0), (4, 200.0)] {
        assert_eq!(c.scale(vanilla_range(levels)), want, "levels {levels}");
        assert_eq!(c.scale(vanilla_range(levels)), 4.0 * vanilla_range(levels));
    }
}

#[test]
fn reads_multiplier() {
    assert_eq!(parse("multiplier = 2").0.multiplier, 2.0);
    assert_eq!(parse("multiplier=2.5 # comment").0.multiplier, 2.5);
    assert_eq!(parse("multiplier = \"3\"").0.multiplier, 3.0);
    assert_eq!(parse("multiplier = 1").0.multiplier, 1.0);
    assert_eq!(parse("multiplier = 16").0.multiplier, 16.0);
}

#[test]
fn bad_values_keep_default() {
    for text in ["multiplier = 0", "multiplier = -3", "multiplier = 100", "multiplier = abc", "multiplier = NaN", "multiplier = inf"] {
        let (c, problems) = parse(text);
        assert_eq!(c.multiplier, DEFAULT_MULTIPLIER, "{text}");
        assert_eq!(problems.len(), 1, "{text}: {problems:?}");
    }
}

#[test]
fn unknown_keys_and_junk_are_reported() {
    let (c, problems) = parse("radius = 5\njunk line\nmultiplier = 3");
    assert_eq!(c.multiplier, 3.0);
    assert_eq!(problems.len(), 2, "{problems:?}");
}

#[test]
fn empty_file_is_default() {
    assert_eq!(parse("").0, Config::default());
    assert_eq!(parse("# only comments\n\n").0, Config::default());
}

#[test]
fn horizontal_edges() {
    let b = [0, 64, 0];
    let c = Config::default();
    // Level 1: vanilla 20 blocks, now 80. The box spans x - r ..= x + 1 + r.
    let r = c.scale(vanilla_range(1));
    assert!(in_range(b, [80.9, 64.0, 0.5], r));
    assert!(in_range(b, [81.0, 64.0, 0.5], r));
    assert!(!in_range(b, [81.01, 64.0, 0.5], r));
    assert!(in_range(b, [-80.0, 64.0, 0.5], r));
    assert!(!in_range(b, [-80.01, 64.0, 0.5], r));
    assert!(in_range(b, [0.5, 64.0, 81.0], r));
    assert!(!in_range(b, [0.5, 64.0, -80.5], r));
    // The same spot is outside vanilla's area.
    assert!(!in_range(b, [50.0, 64.0, 0.5], vanilla_range(1)));
    assert!(in_range(b, [50.0, 64.0, 0.5], r));
    // Level 4: 200 blocks (vanilla 50), corners too.
    let r4 = c.scale(vanilla_range(4));
    assert!(in_range(b, [200.0, 64.0, -200.0], r4));
    assert!(!in_range(b, [202.0, 64.0, 0.0], r4));
}

#[test]
fn vertical_reach() {
    let b = [10, 64, 10];
    let r = Config::default().scale(vanilla_range(4));
    // Any height above.
    assert!(in_range(b, [10.5, 320.0, 10.5], r));
    // Below: down to y - range (vanilla 50, now 200).
    assert!(in_range(b, [10.5, -64.0, 10.5], r));
    assert!(in_range(b, [10.5, 64.0 - 200.0, 10.5], r));
    assert!(!in_range(b, [10.5, 64.0 - 200.5, 10.5], r));
    assert!(!in_range(b, [10.5, 0.0, 10.5], vanilla_range(4)));
}

#[test]
fn multiplier_one_is_vanilla() {
    let c = parse("multiplier = 1").0;
    for levels in 1..=4 {
        assert_eq!(c.scale(vanilla_range(levels)), vanilla_range(levels));
    }
}

#[test]
fn load_writes_default_file() {
    let dir = temp_dir("write");
    let (c, problems) = load(&dir);
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(c.multiplier, 4.0);
    assert_eq!(fs::read_to_string(dir.join(CONFIG_FILE)).unwrap(), DEFAULT_CONFIG);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn load_reads_existing_file() {
    let dir = temp_dir("read");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(CONFIG_FILE), "multiplier = 6\n").unwrap();
    assert_eq!(load(&dir).0.multiplier, 6.0);
    let _ = fs::remove_dir_all(&dir);
}

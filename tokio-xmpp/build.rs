use rustc_version::version;

fn main() {
    let version = version().unwrap();

    for major in 1..=version.major {
        for minor in 0..=version.minor {
            println!("cargo:rustc-cfg=rustc_least_{}_{}", major, minor);
        }
    }
}

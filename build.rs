fn main() {
    let src = [
        "src/rtree.c",
    ];
    let mut builder = cc::Build::new();
    let build = builder
        .files(src.iter());
    build.compile("rtreec");
}
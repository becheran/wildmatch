use criterion::{black_box, criterion_group, criterion_main, Criterion};
use regex::Regex;
use wildmatch::WildMatch;

const TEXT: &str = "Lorem ipsum dolor sit amet, \
consetetur sadipscing elitr, sed diam nonumy eirmod tempor \
invidunt ut labore et dolore magna aliquyam erat, sed diam \
voluptua. At vero eos et accusam et justo duo dolores et ea \
rebum. Stet clita kasd gubergren, no sea takimata sanctus est \
Lorem ipsum dolor sit amet.";

const TEXT_PATTERN: &str = TEXT;
const TEXT_REGEX: &str = "^Lorem ipsum dolor sit amet, \
consetetur sadipscing elitr, sed diam nonumy eirmod tempor \
invidunt ut labore et dolore magna aliquyam erat, sed diam \
voluptua\\. At vero eos et accusam et justo duo dolores et ea \
rebum\\. Stet clita kasd gubergren, no sea takimata sanctus est \
Lorem ipsum dolor sit amet\\.$";

const COMPLEX_PATTERN: &str = "Lorem?ipsum*dolore*ea* ?????ata*.";
const COMPLEX_REGEX: &str = "^Lorem.ipsum.*dolore.*ea.* .....ata.*\\.$";

const MOST_COMPLEX_PATTERN: &str = "?a*b*?**c?d****?e*f*g*?*h?i*?*?**j*******k";
const MOST_COMPLEX_REGEX: &str =
    "^.a.*b.*..*.*c.d.*.*.*.*.e.*f.*g.*..*h.i.*..*..*.*j.*.*.*.*.*.*.*k$";

pub fn compiling(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiling");

    group.bench_function("compile text pattern", |b| {
        b.iter(|| WildMatch::new(black_box(TEXT_PATTERN)))
    });
    group.bench_function("compile complex pattern", |b| {
        b.iter(|| WildMatch::new(black_box(MOST_COMPLEX_PATTERN)))
    });

    group.bench_function("compile text regex", |b| {
        b.iter(|| Regex::new(black_box(TEXT_REGEX)).unwrap())
    });
    group.bench_function("compile complex regex", |b| {
        b.iter(|| Regex::new(black_box(MOST_COMPLEX_REGEX)).unwrap())
    });
}

pub fn matching(c: &mut Criterion) {
    let pattern1 = WildMatch::new(TEXT_PATTERN);
    let pattern2 = WildMatch::new(COMPLEX_PATTERN);
    let regex1 = Regex::new(TEXT_REGEX).unwrap();
    let regex2 = Regex::new(COMPLEX_REGEX).unwrap();

    let mut group = c.benchmark_group("matching");

    group.bench_function("match text pattern", |b| {
        b.iter(|| pattern1 == black_box(TEXT))
    });
    group.bench_function("match complex pattern", |b| {
        b.iter(|| pattern2 == black_box(TEXT))
    });

    group.bench_function("match text regex", |b| {
        b.iter(|| regex1.is_match(black_box(TEXT)))
    });
    group.bench_function("match complex regex", |b| {
        b.iter(|| regex2.is_match(black_box(TEXT)))
    });
}

criterion_group!(benches, compiling, matching);
criterion_main!(benches);

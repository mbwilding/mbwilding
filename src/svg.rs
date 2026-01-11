use oxvg_ast::parse::roxmltree::parse;
use oxvg_ast::serialize::Node as _;
use oxvg_ast::visitor::Info;
use oxvg_optimiser::Jobs;

// Number formatting constants
const THOUSANDS_THRESHOLD: u32 = 1000;
const THOUSANDS_DIVISOR: f32 = THOUSANDS_THRESHOLD as f32;

/// Optimizes an SVG string
pub fn optimize(svg: &str) -> String {
    parse(svg, |dom, allocator| {
        let jobs = Jobs::default();
        if jobs.run(dom, &Info::new(allocator)).is_ok() {
            dom.serialize().unwrap_or_else(|_| svg.to_string())
        } else {
            svg.to_string()
        }
    })
    .unwrap_or_else(|_| svg.to_string())
}

/// Format a number with k suffix for thousands
pub fn format_number(n: u32) -> String {
    if n >= THOUSANDS_THRESHOLD {
        format!("{:.1}k", n as f32 / THOUSANDS_DIVISOR)
    } else {
        n.to_string()
    }
}

use oxvg_ast::parse::roxmltree::parse;
use oxvg_ast::serialize::Node as _;
use oxvg_ast::visitor::Info;
use oxvg_optimiser::Jobs;

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
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

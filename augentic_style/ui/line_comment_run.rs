fn over_the_cap() {
    // Line one of a comment run.
    // Line two of the run.
    // Line three of the run.
    // Line four is one over the default cap of three.
    let _value = 1;
}

fn at_the_cap() {
    // Line one.
    // Line two.
    // Line three stays within the cap.
    let _value = 2;
}

fn broken_by_code() {
    // Two lines here.
    // Second line.
    let _split = 3;
    // Two more lines after code.
    // Second line again.
    let _value = 4;
}

#[allow(line_comment_run)]
fn suppressed() {
    // Allowed line one.
    // Allowed line two.
    // Allowed line three.
    // Allowed line four would fire without the `#[allow]` above.
    let _value = 5;
}

fn main() {}

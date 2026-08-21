fn tainted() {
    // Phase 3 renamed this seam.
    let _value = 1;
}

/// This function formerly returned a boxed error.
fn doc_tainted() {}

fn clean() {
    // Present-tense description of what the code does now.
    let _value = 2;
}

fn main() {}

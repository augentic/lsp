// The default cap is 25 Unicode scalars per declared name.

fn short_enough() {}

// A rustfmt-wrapped signature: the old line lexer missed these.
fn this_declared_name_is_far_too_long(
    _argument: u32,
) {
}

struct Record {
    field_ok: u32,
    a_field_name_that_is_too_long: u32,
}

enum Outcome {
    Ok,
    AVariantNameThatIsMuchTooLong,
}

trait Contract {
    fn an_associated_name_that_is_too_long(&self);
}

impl Contract for Record {
    fn an_associated_name_that_is_too_long(&self) {}
}

fn main() {}

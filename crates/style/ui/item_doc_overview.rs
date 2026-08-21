/// Overview line one.
/// Overview line two.
/// Overview line three.
/// Overview line four.
/// Overview line five.
/// Overview line six.
/// Overview line seven.
/// Overview line eight.
/// Overview line nine is one over the default cap of eight.
pub fn overlong() {}

/// Short overview.
///
/// # Errors
///
/// Heading bodies are exempt, however long they run.
/// This line does not count.
/// Nor does this one.
/// Nor this one.
/// Nor this one.
/// Nor this one.
/// Nor this one.
/// Nor this one.
/// Nor this one.
pub fn sectioned() -> Result<(), ()> {
    Ok(())
}

/// One line.
///
/// ```
/// let fenced = "exempt";
/// ```
pub fn fenced() {}

fn main() {}

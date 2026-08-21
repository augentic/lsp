//! Line one of a module doc that overruns the cap.
//! Line two keeps going.
//! Line three keeps going.
//! Line four is one over the default cap of three.

mod fenced {
    //! Short overview.
    //! ```
    //! let fenced_code = "does not count";
    //! let more_code = "still exempt";
    //! let even_more = "still exempt";
    //! ```
    //! Second and last prose line.
}

mod fine {
    //! One line is plenty.
}

fn main() {}

# Test compile notes

The RED suite is organized around `spec/Page API.md`. It intentionally describes the target public API, which may differ from the unfinished implementation on this branch.

Compile the test crate before evaluating runtime RED failures. A compiler error is not an acceptable RED result. In particular, keep fixtures and imports synchronized with the public Page API contracts for:

- Generic `App<D>` and direct host database access.
- Four-field routing records and SQLite Store persistence.
- Complete Button and LinkButton property configuration.
- Five-slot Button rows with direct values and `None`.
- Dynamic Frame callbacks without implicit pagination.
- Modal Label-wrapped inputs and typed submission values.

The suite has not been marked GREEN by this note. Run the full test suite after the implementation has been updated to the current contract.

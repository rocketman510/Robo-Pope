//! Modal input definitions and Discord Label composition.
//!
//! ModalInput describes one permitted input. Framework constructs the
//! Serenity Label wrapper and nested input builder internally. Every input
//! has a stable developer-defined ID used for submission lookup.
//!
//! Support Text Input, String/User/Role/Mentionable/Channel Select, File
//! Upload, Radio Group, Checkbox Group, and Checkbox. Keep selected values
//! typed by component kind. File Upload submission returns attachment
//! references; it does not mean bytes are stored in the Modal blueprint.
//!
//! Validate title <=45 characters, at most five top-level components,
//! labels <=45, descriptions <=100, text input maximum <=4000, and option
//! counts <=25 where Discord applies them. Validate every option family,
//! including Radio and Checkbox Groups. Reject duplicate IDs.
//!
//! TODO: Define ModalInput, ModalOption, ModalSelect, and typed input structs.
//! TODO: Define builder methods and useful settings for every input kind.
//! TODO: Validate duplicate IDs and each input's limits.
//! TODO: Build required Discord Label structures internally.

//! Render pipeline and Discord-ready message output.
//!
//! Render in distinct phases:
//! 1. Serialize Page instance data and initialize RenderCtx.
//! 2. Resolve static/dynamic properties.
//! 3. Expand Frame regions into complete heterogeneous components.
//! 4. Validate resolved component structure and property values.
//! 5. Calculate nested component and Text Display costs.
//! 6. Plan pagination without splitting components.
//! 7. Build Serenity's Components V2 representation.
//! 8. Persist execution references required by rendered controls.
//! 9. Return an output object that can be sent/applied to a message.
//!
//! RenderedMessage is the boundary between the declarative framework and
//! Serenity. It carries the required IS_COMPONENTS_V2 flag and a clear
//! attachment handoff for File components. Components V2 must not be mixed
//! with traditional message content/embeds in the ordinary way. It does not
//! expose custom-ID construction or private routing records.
//!
//! Validate after dynamic values and Frame contents resolve. A valid static
//! blueprint may produce invalid dynamic data. Return contextual errors;
//! never truncate content, silently drop components, or panic.
//!
//! Keep separate accounting for total nested components, Text Display
//! characters, direct Container children, Section structure, Action Row
//! buttons, select options, and gallery items. Descriptions have their own
//! limits and do not count against the Text Display-only budget. Cost
//! calculation must be fallible, not based on expect/unwrap.
//!
//! TODO: Define RenderPlan, RenderedMessage, and the render entry points.
//! TODO: Count semantic component types recursively rather than guessing
//!       from serialized JSON keys.
//! TODO: Validate all Discord limits after dynamic resolution.
//! TODO: Preserve attachment metadata for File components.
//! TODO: Define render_all and render_page behavior for Frame pagination.

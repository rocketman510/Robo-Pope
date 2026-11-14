# Discord UI Framework: Page API

## 1. Purpose and terminology

The framework provides a declarative Rust API for building Discord messages and interactions with Components V2.

A **Page** is a reusable definition of a Discord message. It defines the component hierarchy, the properties displayed by those components, and the callbacks that handle interactions.

A **Modal** is a reusable definition of a form opened in response to an interaction.

A **Frame** is a dynamic region inside a Page. It resolves to a collection of components when the Page is rendered.

An **App** owns the registered Page and Modal definitions, the application's typed database value, the framework's persistent interaction Store, and interaction routing.

The API should make building an interface feel like composing ordinary Rust values. Serenity builders, Discord custom IDs, serialization, persistence, and interaction dispatch are framework implementation details.

This specification describes the intended public API and behavior. Where an exact Rust trait signature depends on callback lifetimes or type inference, the required caller-facing behavior is normative even if the internal representation differs.

## 2. Static and dynamic values

A component property may be supplied as either a static value or an asynchronous callback that resolves to that value. Both forms use the same public method or constructor argument. Developers should not need separate constructors or methods for static and dynamic values.

For example, text may be static:

```rust
Page::<Dataless>::new("Welcome")
    .add_text("Welcome to the server.")
```

Or it may be resolved when the Page is rendered:

```rust
Page::<UserData>::new("Welcome")
    .add_text(|ctx, data| async move {
        Ok(format!("Welcome, {}.", data.username))
    })
```

The same static-or-dynamic rule applies to component properties that can be resolved at render time, including labels, styles, optional emoji, disabled states, descriptions, URLs, placeholders, option display text, media properties, colors, and other display settings.

A dynamic property callback receives the framework callback context and the Page's data. It is asynchronous and returns a Result containing the resolved property value. A callback passed directly to a property argument is treated as dynamic; an extra dynamic wrapper should not be required.

Conceptually, a property uses one common value representation:

```rust
enum Value<T> {
    Static(T),
    Dynamic(/* asynchronous callback returning Result<T> */),
}
```

The concrete callback type and its lifetime bounds are implementation details. The observable API behavior is not: an ordinary value and a callback returning that value must be accepted through the same property interface.

A dynamic property callback runs while rendering. It receives access to the App through its context, but there is no active interaction to reply to. Interaction-response methods are available only to execution callbacks that were invoked because a Discord interaction occurred.

A dynamic property controls what a component displays or how it is configured. It does not replace the component's execution callback. Interactive components always declare their execution separately.

## 3. Page and Page data

A Page is a reusable blueprint. It defines message structure, not an individual Discord message instance.

Each Page has a globally unique developer-defined ID and one generic data type, T. That data is shared by all dynamic properties, Frames, and interactive executions belonging to the Page.

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Dataless;

let page = Page::<Dataless>::new("Welcome")
    .add_text("Welcome to the server.")
    .register(app)?;
```

Dataless is a zero-sized placeholder type for a Page that has no application data. It does not represent a special framework feature.

Pages are built before registration. Registration consumes the builder and produces a RegisteredPage<T>. A regular Page<T> can be changed while it is being built. A RegisteredPage<T> is immutable: its component structure and callback definitions cannot be changed after registration.

The registered definition does not hold an App reference. The App owns the definition, and the App is supplied when the Page is rendered.

```rust
let page: RegisteredPage<Dataless> =
    Page::<Dataless>::new("Welcome")
        .add_text("Welcome to the server.")
        .register(app)?;

let message = page.render(&app, Dataless).await?;
```

Rendering requires the App because interactive components must have their routing records written to the framework Store before the message is sent.

### Page data

Page data is the shared, typed input to the Page's dynamic properties and executions.

```rust
#[derive(Serialize, Deserialize)]
pub struct UserLevelData {
    pub user_id: String,
}

let page = Page::<UserLevelData>::new("UserLevel")
    .add_text(|ctx, data| async move {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ?"
        )
        .bind(&data.user_id)
        .fetch_one(&ctx.app.db.pool)
        .await?;

        Ok(format!("Level: {}", user.level))
    })
    .register(app)?;
```

The data passed to render is serialized for persistent interaction routing. When an interaction arrives, the framework reconstructs the Page's data and passes an immutable view of it to the corresponding callback.

Page data is observed, not mutated, by callbacks. If an interaction needs to produce a different Page state, application code creates and renders a new Page data value. The framework does not silently mutate the original data.

The Page data type must implement Serialize and DeserializeOwned. It must also satisfy the concurrency bounds required by the asynchronous callback implementation.

## 4. Component property conversions and fixed slots

### Static and dynamic conversions

Property arguments should accept their ordinary Rust value or a dynamic callback returning that value. Conversion should happen at the public API boundary.

```rust
.add_text("Static text")
.add_text(|ctx, data| async move {
    Ok(format!("Hello, {}.", data.username))
})
```

Do not require callers to construct an internal Value<T> manually.

### Optional text slots

Some Discord components have a fixed number of child slots. The public API should make those slots explicit while still allowing each slot to be static, dynamic, or absent.

A Section has three text slots. The first slot is required; the second and third may be omitted.

```rust
Section::new()
    .set_text(
        "User information",
        None,
        None,
    )
```

All three slots may be populated:

```rust
Section::new()
    .set_text(
        "User information",
        |ctx, data| async move {
            Ok(format!("Account level: {}", data.level))
        },
        "Additional information",
    )
```

The Section text-slot argument type should be a framework conversion type that accepts a string, an owned string, a dynamic text callback, or None. That is how a caller can pass ordinary strings directly while still expressing an absent slot. The API must not require Some("text") around each present string.

### Button-row slots

A button Action Row has exactly five available positions in this API. Each position accepts a Button directly or None if the position is empty.

```rust
Page::<Dataless>::new("Example")
    .add_button_row((
        button1,
        button2,
        None,
        None,
        None,
    ))
```

A fully populated row is:

```rust
.add_button_row((
    button1,
    button2,
    button3,
    button4,
    button5,
))
```

Each row slot uses a small ButtonSlot conversion type. Button converts directly into ButtonSlot, LinkButton converts directly into ButtonSlot, and None converts into an empty slot. A conceptual definition is:

```rust
pub enum ButtonSlot {
    Button(Button),
    Link(LinkButton),
    Empty,
}

impl From<Button> for ButtonSlot {
    fn from(button: Button) -> Self {
        Self::Button(button)
    }
}

impl From<LinkButton> for ButtonSlot {
    fn from(button: LinkButton) -> Self {
        Self::Link(button)
    }
}

impl From<Option<ButtonSlot>> for ButtonSlot {
    fn from(slot: Option<ButtonSlot>) -> Self {
        slot.unwrap_or(Self::Empty)
    }
}

```

The five tuple arguments are each accepted through Into<ButtonSlot>. The tuple therefore accepts Button, LinkButton, or None without requiring Some(...) around present values. This conversion is explicit in the public API contract and keeps the call site simple.

This is deliberate. The public examples and method signatures must consistently use direct Button values and None; they must not alternate between direct values and explicit Some(...) wrappers.

The fixed-size tuple makes the five-position limit visible in the type-level API. The framework creates the Discord Action Row and validates that at least one button is present.

## 5. Components V2 layout and content

The Page API should expose Discord's Components V2 building blocks through composable framework types. It should hide low-level Serenity component serialization.

### Text Display

A Text Display presents Markdown-formatted text. Each call to add_text adds a distinct Text Display; separate calls must not be silently concatenated.

```rust
Page::<Dataless>::new("Example")
    .add_text("# Welcome")
    .add_text("This is a separate text block.")
```

Text may be static or dynamic.

### Section

A Section places one to three Text Displays beside an accessory. The accessory is either a Thumbnail or a Button. The three text positions are explicit slots and each may be static, dynamic, or absent, subject to Discord requiring at least one text child.

```rust
Section::new()
    .set_text(
        "Account information",
        "Additional details",
        None,
    )
    .set_accessory(
        Thumbnail::new("https://example.com/avatar.png")
            .set_description("User avatar")
    )
```

A Button is also a valid Section accessory. Its constructor must use the same complete Button argument list described in the Button section:

```rust
Section::new()
    .set_text("Open the user's profile.", None, None)
    .set_accessory(
        Button::new(
            "View profile",
            "ViewProfile",
            ButtonStyle::Secondary,
            Some("👤".into()),
            false,
            |ctx, data| async move {
                Ok(())
            },
        )
    )
```

The accessory Button has a static execution ID, an explicit optional emoji value, an explicit disabled Boolean, and an execution callback. The properties that support dynamic values must also accept callbacks in place of their static values.

### Container

A Container groups components into a visually distinct block. It supports an optional accent color and spoiler behavior.

```rust
Container::new()
    .set_accent_color(0x5865F2)
    .set_spoiler(false)
    .add_text("Account information")
    .add_separator()
    .add_text("Additional details")
```

A Page may contain multiple independent Containers. A Container is not a nested Page and does not have its own Page data type or registration lifecycle. It should expose the same general composition style as a Page for the component types Discord permits inside a Container.

The supported Container children are Action Rows, Text Displays, Sections, Media Galleries, Separators, and File components. A Container cannot contain another Container. The framework validates the child structure against Discord's rules. The Container builder should expose the same conversion and static-or-dynamic property behavior as the Page builder for every child type that Discord allows in a Container.

### Separator

A Separator adds vertical spacing and can optionally display a divider.

```rust
Separator::new()
    .set_spacing(SeparatorSpacing::Large)
    .set_divider(true)
```

Spacing and divider settings may be static or dynamic.

### Thumbnail

A Thumbnail displays an image, usually as a Section accessory.

```rust
Thumbnail::new("https://example.com/avatar.png")
    .set_description("User avatar")
    .set_spoiler(false)
```

The media URL, description, and spoiler setting follow the static-or-dynamic property rule. Thumbnails support images, not video, and are used as Section accessories.

### Media Gallery

A Media Gallery displays between one and ten media items. Each item may include a description and spoiler setting.

Both replacement and incremental composition should be available. set_media explicitly replaces the gallery's current media collection. add_media appends one item.

```rust
MediaGallery::new()
    .set_media([
        MediaItem::new("https://example.com/image1.png")
            .set_description("First image"),
        MediaItem::new("https://example.com/image2.png")
            .set_description("Second image"),
    ])
```

An item can also be appended:

```rust
.add_media(
    MediaItem::new("https://example.com/image3.png")
        .set_description("Third image")
        .set_spoiler(false)
)
```

The gallery and item properties that can be resolved at render time should support static or dynamic values. The framework validates the final collection and rejects an empty gallery or a gallery containing more than ten items.

### File

A File component exposes an attachment inside a Components V2 message. Its constructor accepts the complete Discord attachment URI, including the `attachment://` prefix.

```rust
File::new("attachment://document.pdf")
    .set_spoiler(false)
```

The stored reference is named `attachment_uri`, because the value includes the URI scheme and is not merely a filename. The URI must identify an attachment included with the outgoing message. The File component references the attachment; it does not upload the file itself.
## 6. Interactive components

Interactive components have a developer-defined execution ID and an explicit asynchronous callback. The execution ID is static and unique within its Page. It identifies the callback in the Page definition; it is not the Discord custom_id sent to Discord. The framework generates that custom_id from the persistent routing identity.

### Button

A Button constructor explicitly defines its label, execution ID, style, optional emoji, disabled state, and execution callback, in that order.

```rust
Button::new(
    "Show Level",
    "ShowLevel",
    ButtonStyle::Secondary,
    Some("📊".into()),
    false,
    |ctx, data: UserLevelData| async move {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ?"
        )
        .bind(&data.user_id)
        .fetch_one(&ctx.app.db.pool)
        .await?;

        ctx.reply(format!("Level: {}", user.level)).await?;

        Ok(())
    },
)
```

The emoji argument is an Option value. Some("📊".into()) supplies an emoji; None means the Button has no emoji. The disabled argument is a Boolean, where true disables the Button and false leaves it enabled.

A Button with no emoji is written explicitly:

```rust
Button::new(
    "Refresh",
    "Refresh",
    ButtonStyle::Secondary,
    None,
    false,
    |ctx, data| async move {
        Ok(())
    },
)
```

The label and style accept a static value or an asynchronous callback returning that value. The emoji is an optional Unicode emoji string, so its value is Option<String>; it may be supplied statically or by an asynchronous callback returning Option<String>. The disabled property is a Boolean and may be supplied statically or by an asynchronous callback returning bool. The execution ID is always a static string, and the execution callback is always explicit.

A dynamic-property example is:

```rust
Button::new(
    |ctx, data| async move {
        Ok(format!("User {}", data.user_id))
    },
    "ShowLevel",
    |ctx, data| async move {
        Ok(ButtonStyle::Secondary)
    },
    |ctx, data| async move {
        Ok(Some("📊".into()))
    },
    |ctx, data| async move {
        Ok(false)
    },
    |ctx, data| async move {
        ctx.reply("Showing your level.").await?;
        Ok(())
    },
)
```

### Link Button

A Link Button opens a URL. It is a separate type because clicking it navigates to the URL and does not produce an interaction callback.

```rust
LinkButton::new(
    "Visit Website",
    "https://example.com",
    Some("🌐".into()),
    false,
)
```

The Link Button's style is always Discord's Link style. Its label, URL, optional emoji, and disabled Boolean each accept a static value or a dynamic callback. Some("🌐".into()) supplies an emoji and None omits it.

Discord's current Button component schema includes disabled for Link-style Buttons as well as callback Buttons. Therefore LinkButton must expose a disabled property. A disabled Link Button remains a link-style component but cannot be activated.

A Link Button has no execution ID and must not create a callback-routing record.

### Button rows

Developers add Buttons and Link Buttons to a Page through a fixed five-slot tuple. The framework creates the Action Row. Each tuple position accepts Button, LinkButton, or None through the ButtonSlot conversion type.

```rust
Page::<Dataless>::new("Actions")
    .add_button_row((
        Button::new(
            "Confirm",
            "Confirm",
            ButtonStyle::Success,
            Some("✅".into()),
            false,
            |ctx, data| async move { Ok(()) },
        ),
        Button::new(
            "Cancel",
            "Cancel",
            ButtonStyle::Secondary,
            None,
            false,
            |ctx, data| async move { Ok(()) },
        ),
        None,
        None,
        None,
    ))
```

Each slot accepts Button or None; callers do not wrap a present Button in Some. A row supports up to five Buttons. Link Buttons are also Buttons in Discord's component model and may be included in a Button row, but they do not have callback IDs or executions.

### Selection menus

A Page exposes one method for adding a selection menu:

```rust
.add_selection_menu(selection_menu)
```

The menu's concrete type determines which Discord selection menu is created. The framework supports:

- String Select
- User Select
- Role Select
- Mentionable Select
- Channel Select

These types should implement a common selection-menu abstraction so the Page does not need a separate add method for each kind.

A String Select has developer-defined option values and a callback that receives the selected values:

```rust
StringSelect::new(
    "Choose a class",
    "ClassSelect",
    [
        SelectOption::new("Warrior", "warrior"),
        SelectOption::new("Mage", "mage"),
        SelectOption::new("Rogue", "rogue"),
    ],
    |ctx, data, selected: Vec<String>| async move {
        // Handle the selected values.
        Ok(())
    },
)
```

The placeholder, option labels, option descriptions, option emoji, and other display properties should support static or dynamic values where appropriate. A SelectOption exposes `.set_description(...)` and `.set_emoji(...)` for its optional description and emoji. The emoji value is `Option<String>`: a static or dynamic value may resolve to `Some(emoji)` or `None`. Developer-defined option values remain the values returned to the callback.

Message selection menus support a disabled Boolean. Modal selection inputs do not support disabled; the framework must reject or prevent that property when building a Modal.

String Select options do not have an independent disabled field in Discord's component schema. The framework must not expose a fake per-option disabled setting or silently discard it. The complete select menu may be disabled in a message.

The selected values are interaction data supplied by Discord. They are not part of the serialized Page data.

Entity select menus expose selected entities in typed form. User, Role, and Channel selections expose their corresponding Discord IDs and resolved entities. Mentionable selections expose a tagged value identifying whether each selected item is a User or Role.

## 7. Frame

A Frame is a dynamic region inside a Page. It is intended for content whose complete component collection is not known when the Page definition is built, such as a settings list, user directory, or catalogue.

The developer supplies a callback that returns a collection of framework elements. The collection may contain heterogeneous supported components.

```rust
Page::<UserListData>::new("UserList")
    .add_text("# Users")
    .add_frame(|ctx, data| async move {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY username"
        )
        .fetch_all(&ctx.app.db.pool)
        .await?;

        let elements = users
            .into_iter()
            .map(|user| {
                Element::text(format!(
                    "{} - Level {}",
                    user.username,
                    user.level
                ))
            })
            .collect();

        Ok(elements)
    })
    .register(app)?;
```

A Frame resolves to a collection of complete elements. Elements may be Text Displays, Sections, Containers, Separators, Media Galleries, Files, Buttons, selection menus, or supported compositions of those components. A common internal Element representation may be used to hold different component types, but developers should compose the elements using the same public component types used elsewhere.

A Frame occupies one declared position in the Page's component hierarchy. Its resolved elements appear at that position in the order returned by the callback. Static Page components before and after it retain their order.

A Frame uses the same Page data type as the rest of its Page. It does not have independent data, registration, or execution state. Interactive elements returned by a Frame still require developer-defined execution IDs that are unique within the Page and stable for the meaning of the action they identify.

A Frame is not an automatic pagination system. The Frame API in this specification does not define page indexes, offsets, ranges, cursors, automatic splitting, Previous or Next buttons, or pagination state. The developer controls which elements the Frame callback returns. The framework resolves and renders that collection as part of the Page and validates it against Discord's message limits. If the resolved Page exceeds those limits, rendering returns an error rather than silently truncating content or inventing pagination behavior.

## 8. Modals

A Modal is a reusable form definition displayed in response to an interaction. It is not a message component or a Page. Submitting it creates a new interaction.

A Modal has a globally unique ID and one generic data type, T, just like a Page.

```rust
#[derive(Serialize, Deserialize)]
pub struct EditUserData {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct EditUserForm {
    pub username: String,
    pub description: String,
}

let modal = Modal::<EditUserData>::new("EditUser")
    .set_title("Edit User")
    .add_text_input(
        "username",
        "Username",
        TextInputStyle::Short,
    )
    .add_text_input(
        "description",
        "Description",
        TextInputStyle::Paragraph,
    )
    .on_submit(|ctx, data, form: EditUserForm| async move {
        sqlx::query(
            "UPDATE users SET username = ?, description = ? WHERE id = ?"
        )
        .bind(&form.username)
        .bind(&form.description)
        .bind(&data.user_id)
        .execute(&ctx.app.db.pool)
        .await?;

        ctx.reply("User updated successfully.").await?;

        Ok(())
    })
    .register(app)?;
```

Modal registration consumes the builder and produces an immutable registered definition. The Modal's generic data is serialized when it is opened and reconstructed when the submission interaction arrives. Submitted form values are separate from the Modal's generic data.

### Opening a Modal

A Modal must be opened as the initial response to an interaction. It is not sent through an ordinary message reply or follow-up.

```rust
Button::new(
    "Edit User",
    "EditUser",
    ButtonStyle::Primary,
    Some("✏️".into()),
    false,
    |ctx, data: EditUserData| async move {
        let modal = ctx.app.modals
            .query("EditUser")?
            .render(&ctx.app, EditUserData {
                user_id: data.user_id.clone(),
            })
            .await?;

        ctx.open_modal(modal).await?;

        Ok(())
    },
)
```

### Modal input components

The Modal builder should expose the input components Discord supports:

- Text Input
- String Select
- User Select
- Role Select
- Mentionable Select
- Channel Select
- File Upload
- Radio Group
- Checkbox Group
- Checkbox

Modal inputs are placed inside Discord Label components. The framework should create the required Label structure from the simple input-building methods. Developers should not need to manually construct the Discord Label wrapper for each input.

Each input has a developer-defined ID. The ID is used to retrieve the submitted value or map it to a field in a typed form struct. Input labels, descriptions, placeholders, option display properties, and other supported properties follow the static-or-dynamic value rule.

### Typed submission data

The framework should deserialize submitted values into a developer-defined Rust struct through Serde. This is the preferred API; manually extracting strings by component ID should not be required for ordinary use.

```rust
#[derive(Deserialize)]
pub struct TicketForm {
    pub subject: String,
    pub details: String,
    pub category: String,
    pub urgent: bool,
}

Modal::<Dataless>::new("CreateTicket")
    .set_title("Create a Ticket")
    .add_text_input(
        "subject",
        "Subject",
        TextInputStyle::Short,
    )
    .add_text_input(
        "details",
        "Describe your issue",
        TextInputStyle::Paragraph,
    )
    .add_string_select(
        "category",
        "Category",
        categories,
    )
    .add_checkbox(
        "urgent",
        "This issue is urgent",
    )
    .on_submit(|ctx, data, form: TicketForm| async move {
        // Use the typed form values.
        Ok(())
    })
    .register(app)?;
```

The mapping from Modal input IDs to form fields is based on the input ID matching the Rust field name. The framework uses Serde to deserialize the values and returns an error for missing required values, invalid values, or incompatible Rust field types.

The value types are:

- Text Input: String (or Option<String> when the input is optional).
- String Select: Vec<String> for submitted option values.
- User Select: Vec<UserId>.
- Role Select: Vec<RoleId>.
- Channel Select: Vec<ChannelId>.
- Mentionable Select: a vector of tagged values identifying UserId or RoleId.
- Radio Group: String for the selected option value, or Option<String> when no selection is required.
- Checkbox Group: Vec<String> containing selected option values.
- Checkbox: bool.
- File Upload: a vector of attachment metadata containing the attachment ID, filename, URL, size, and optional content type.

The raw ModalSubmission accessors remain available when application code needs direct access to the underlying submission or resolved Discord entities. The framework does not download uploaded file bytes as part of typed deserialization.

Modal rendering must resolve dynamic properties and validate the complete Modal before persisting its submit-execution routing record. The record must be persisted before the Modal is sent as the interaction response. A failure before successful validation must not leave a usable routing record for a Modal that was never built.

## 9. Interaction context and callbacks

Every dynamic property callback and interactive execution receives the framework context and the Page or Modal data.

An interactive callback has the general form:

```rust
|ctx, data| async move {
    // Application logic.
    Ok(())
}
```

An execution callback returns Result. It can perform asynchronous work with await, use the App's database, and respond to the Discord interaction.

ctx is a framework context wrapping the relevant Serenity context. It exposes the App through ctx.app and provides the Serenity interaction information and API needed by the callback. The framework context must also provide convenient response methods; exposing a raw Serenity context alone is not sufficient if Serenity does not provide the response operation required by the framework.

Dynamic property callbacks receive a rendering context and Page or Modal data. They do not have an active interaction and cannot call response methods that require one.

Page and Modal data are exposed immutably. The exact Rust reference, lifetime, and ownership representation is an implementation decision. The API must not allow a callback to mutate the original serialized data through the reference it receives.

The context should expose the interaction information relevant to the current callback, including the interacting user, guild, channel, selected values, and Modal submission data where applicable.

The context is temporary. It belongs to the current render or interaction and is never stored inside a Page definition or persisted in the routing Store.

### Interaction response operations

The context should provide operations for the distinct Discord response actions:

- reply: create the initial message response.
- defer: acknowledge the interaction and defer the response.
- edit_message: update the message containing the component interaction.
- edit_original_response: edit the original interaction webhook response.
- followup: create a separate follow-up message.
- delete_response: delete the original interaction webhook response.
- open_modal: open a Modal as the initial interaction response.

Initial response operations are mutually exclusive. Once the interaction has been acknowledged, another initial-response operation must return an error. The framework must not silently substitute one response type for another.

reply, defer, edit_message, and open_modal have distinct Discord semantics and must map to their corresponding response types. edit_original_response, followup, and delete_response operate on the interaction webhook response. Errors from Discord or the HTTP client are returned to the callback.

Response behavior must be testable through an injected or local HTTP test double. Unit tests for response routing must not require a live Discord connection or real bot token.

## 10. App, database, and registration

App is generic over the application's database type:

```rust
pub struct App<D> {
    pub db: D,
    // Registered Page and Modal definitions, the framework Store,
    // and interaction routing.
}
```

The application defines D. This provides strongly typed, named access to its own database API, for example ctx.app.db.messages, ctx.app.db.users, or ctx.app.db.settings. The framework must not require a type-erased get::<T>() lookup or string-based table lookup.

For example, an application may define:

```rust
pub struct BotDatabase {
    pub pool: SqlitePool,
    pub messages: MessageRepository,
    pub users: UserRepository,
}
```

The application may use repositories, a SQLx SQLite pool, or both. SQL is the underlying query interface. The framework should not invent a proprietary query language or require its own ORM. SQLx is the preferred asynchronous Rust integration to evaluate for SQLite connections, raw SQL, and query-builder support. Developers may use SQLx directly or layer another SQL abstraction over it; the framework should not impose a higher-level ORM.

The framework's interaction Store is separate in responsibility from the application's D value. The Store uses SQLite for persistent interaction routing. The application database and the framework Store may share a SQLite file or use separate files/connections; that is a configuration decision.

The App owns the registered Pages and Modals, the framework Store, and the interaction router. Its registries must be able to hold definitions with different generic Page or Modal data types while preserving the concrete D type used by their callbacks. Any type erasure needed for heterogeneous registries is an internal implementation detail. It must not erase the public type of ctx.app.db.

Registration consumes a Page or Modal builder and stores its immutable registered definition in the App. Page IDs and Modal IDs share one application-wide identifier namespace. Registering a duplicate definition ID is an error.

Registration is not restricted to application startup. It may occur whenever the App's ownership and synchronization rules permit it. The specification does not impose a startup-only lifecycle.

Rendering uses the registered definition and the App:

```rust
let message = app.pages
    .query("UserLevel")?
    .render(&app, user_data)
    .await?;
```

The Page definition does not store the App. The App is supplied to rendering and is available to callbacks through ctx.app.

## 11. Persistent interaction routing

Discord returns a component's custom_id when a user interacts with a Button or selection menu. The framework Store maps that custom ID to the registered callback definition and serialized data needed to route the interaction.

The Store is SQLite-backed. Its purpose is to preserve callback routing across process restarts, provided the same Page and Modal definitions are registered again. It is not a user-session database and is not the application's general-purpose database API.

### Routing record

A routing record contains exactly four fields:

```rust
struct RoutingRecord {
    hash: String,
    definition_id: String,
    execution_id: String,
    serialized_data: Vec<u8>,
}
```

definition_id identifies the registered Page or Modal. execution_id identifies the callback within that definition. serialized_data is the serialized Page or Modal data. hash is the deterministic value used as the Discord component's custom_id and the record's lookup key.

There are no additional per-user, per-message, Frame, pagination, timestamp, or expiration fields in a routing record.

### Deterministic custom IDs

The framework computes the hash programmatically from the definition ID, execution ID, and serialized data. It uses a stable serialization format and deterministic hash algorithm. The same three inputs must always produce the same hash, including after a process restart.

The hash must fit Discord's custom_id limit. The implementation may choose the hash algorithm, but it must not use randomness, process-specific salts, timestamps, or message-instance identity as inputs.

When an identical routing identity is rendered again, the Store reuses the same logical record. If a hash already exists, the Store verifies that the other three fields match. If they do not match, the framework reports a hash collision. It must never overwrite a different callback record or route an interaction arbitrarily.

The developer-defined execution ID is the stable callback identity. A component's execution ID must be unique within its Page or Modal. Link Buttons do not have executions and therefore do not have callback-routing records.

### Interaction routing

When an interaction arrives:

1. The App receives the Discord interaction.
2. The App reads the component's custom_id.
3. The Store looks up the corresponding routing record by hash.
4. The App locates the registered Page or Modal using definition_id.
5. The registered definition locates the callback using execution_id.
6. The definition deserializes serialized_data into its own generic data type.
7. The callback runs with the framework context and an immutable reference to the reconstructed data.

The App does not need to know the Page or Modal's generic data type. The registered definition knows its own type and performs deserialization and callback dispatch.

Closures are never serialized. The registered Page or Modal definition owns the executable callback. The routing record contains only the information required to locate that callback and reconstruct its data.

### Statelessness

The framework does not store user identity or message identity in routing records. It does not maintain per-user sessions, per-message sessions, view ownership, login/logout state, or mutable state tracking who is viewing a Page.

Two users rendering the same registered definition with identical serialized data receive the same routing identities. The framework intentionally does not distinguish those renders.

The Store is persistent routing data, not session state. Routing records do not expire and are not removed merely because time has passed. The framework does not implement TTLs, automatic expiration, or session cleanup.

A custom ID is not authorization. Application callbacks remain responsible for checking whether the interacting user is permitted to perform the requested operation.

## 12. Errors and validation

The framework uses ordinary Rust Result values for operations that may fail. It must return useful errors instead of panicking or silently changing the requested component structure.

Errors include:

- Duplicate Page or Modal IDs.
- Duplicate execution IDs within a Page or Modal.
- Unknown Page, Modal, or execution IDs.
- Failed dynamic property resolution.
- Invalid static or resolved component properties.
- Invalid Section, Action Row, Container, or Modal structure.
- Invalid or missing Modal submission values.
- Serialization or deserialization failures.
- Missing routing records.
- Hash collisions.
- Discord API or HTTP errors.
- A resolved Page or Frame that exceeds Discord's component limits.

Errors should retain useful context, particularly the definition ID and execution ID involved.

Static structural violations should be rejected during registration when they can be determined then. Violations that depend on dynamic values must be checked after those values resolve and before the message or Modal is sent.

Fixed-slot APIs should make invalid child counts unrepresentable where practical. Runtime validation remains necessary for constraints that depend on dynamic values or Discord's message-wide limits.

## 13. Discord component constraints

The framework validates the final resolved component tree against Discord's current Components V2 rules. The following constraints are part of the public contract:

### Messages and Components V2

- Components V2 messages must use the IS_COMPONENTS_V2 message flag.
- A Components V2 message may contain up to 40 total components.
- A Button Action Row contains one to five Buttons.
- An Action Row used for a selection menu contains one selection menu.
- A Section contains one to three Text Displays and one accessory.
- A Section accessory is a Button or Thumbnail.
- A Media Gallery contains one to ten media items.
- A component custom_id is one to 100 characters.
- A Button label is at most 80 characters.
- A Link Button URL is at most 512 characters.
- A select placeholder is at most 150 characters.
- A String Select contains at most 25 options.
- A String Select option label, value, and description are each at most 100 characters.
- A Thumbnail description is at most 1024 characters.
- A Media Gallery item description is at most 1024 characters.
- A Container may contain Action Rows, Text Displays, Sections, Media Galleries, Separators, and Files. A Container cannot contain another Container.

A Components V2 message does not use traditional message content or embeds in the usual way. Text is represented with Text Display components and grouped content may use Containers.

Discord's Button schema includes disabled for both callback Buttons and Link-style Buttons. Link Buttons require a URL and do not use a custom_id or invoke an application callback.

### Modals

- Modal inputs are wrapped in Label components.
- A Label's text is at most 45 characters.
- A Label description is at most 100 characters.
- A Text Input's placeholder is at most 100 characters.
- Text Input values and configured maximum length are at most 4000 characters.
- A select menu supports at most 25 options.
- Radio Groups support between two and ten options.
- Checkbox Groups support between one and ten options.
- File Upload supports up to ten files.

Disabled selection menus are supported in messages, not in Modals. The framework must not send a disabled Modal input because Discord rejects disabled components in Modals.

These values are Discord constraints, not arbitrary framework preferences. They must be kept aligned with Discord's current component reference.

## 14. Implementation order

The framework should be implemented in an order that establishes the Page and callback model before adding more complex component types:

1. App<D>, Page builders, registration, and immutable registered definitions.
2. Static and dynamic property conversion and resolution.
3. Page composition, Sections, Containers, Separators, Thumbnails, Media Galleries, and Files.
4. Buttons, Link Buttons, fixed five-slot button rows, and routing persistence.
5. Selection menu abstractions and interaction dispatch.
6. Modal definitions, submission handling, and typed Serde deserialization.
7. Frame resolution and rendering.

Pagination is deliberately not part of the Frame API defined here. It should be designed separately if and when it is needed.

## 15. Complete composition example

The following example shows the intended composition style. Button arguments are complete: label, static execution ID, style, optional emoji, disabled Boolean, and callback. Button-row slots use direct Button values and None, without wrapping Buttons in Some.

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct UserListData {
    pub guild_id: String,
}

let page = Page::<UserListData>::new("UserList")
    .add_container(
        Container::new()
            .set_accent_color(0x5865F2)
            .add_text("# User Directory")
            .add_separator()
            .add_text("Directory entries follow below.")
    )
    .add_frame(|ctx, data| async move {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE guild_id = ? ORDER BY username"
        )
        .bind(&data.guild_id)
        .fetch_all(&ctx.app.db.pool)
        .await?;

        Ok(users.into_iter()
            .map(|user| {
                Element::text(format!(
                    "{} - Level {}",
                    user.username,
                    user.level
                ))
            })
            .collect())
    })
    .add_button_row((
        Button::new(
            "Refresh",
            "RefreshUsers",
            ButtonStyle::Secondary,
            Some("🔄".into()),
            false,
            |ctx, data: UserListData| async move {
                let updated = ctx.app.pages
                    .query("UserList")?
                    .render(&ctx.app, (*data).clone())
                    .await?;

                ctx.edit_message(updated).await?;
                Ok(())
            },
        ),
        Button::new(
            "More Info",
            "MoreInfo",
            ButtonStyle::Secondary,
            None,
            false,
            |ctx, data: UserListData| async move {
                ctx.reply("This is the user directory.").await?;
                Ok(())
            },
        ),
        None,
        None,
        None,
    ))
    .register(app)?;

let message = page.render(&app, UserListData {
    guild_id: guild_id.to_string(),
}).await?;
```

The framework resolves dynamic properties and Frames, validates the completed component tree, persists the routing records for interactive components, and renders the corresponding Discord Components V2 message.

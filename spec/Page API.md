# Discord UI Framework

## Overview

The framework provides a declarative API for building Discord messages and interactions using Components V2.

A Page describes a message. It defines the component hierarchy, the values displayed by those components, and the executions associated with interactive components. `App<D>` manages registered Pages, the application's typed database value, framework routing persistence, rendering, and interaction routing.

The framework should make building a Discord interface feel like composing ordinary Rust values. Serenity's component builders, custom IDs, serialization, and interaction routing are implementation details.

The framework has four main concepts:

- `App<D>`: Owns the application database value of type `D`, registered Pages and Modals, framework persistence, and interaction routing.
	
- `Page<T>`: A blueprint for a Discord message, parameterized by its data.
    
- `Modal<T>`: A blueprint for a form submitted through a Discord interaction.
    
- `Frame<T>`: A dynamic region inside a Page that resolves its own component hierarchy at render time and can paginate its contents.
	

## 1. Page

A Page is a reusable blueprint. It defines the structure of a message, not a particular instance of that message.

Each Page has a globally unique ID and a single generic data type, `T`. This data is available to every dynamic component and interactive execution belonging to the Page.

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Dataless;

`Dataless` is simply a zero-sized placeholder for Pages that have no application data. It has no special framework behavior.

let page = Page::<Dataless>::new("TheIdOfThePage")
    .add_text("Hello, world!")
    .register(app)?;
```

Pages are built before they are registered. Registration consumes the builder and produces a `RegisteredPage<T>`. The registered definition is immutable and does not retain an `App` reference. The App owns it after registration; rendering supplies the App and the Page data.

```rust
let page: RegisteredPage<Dataless> =
    Page::<Dataless>::new("TheIdOfThePage")
        .add_text("Hello, world!")
        .register(app)?;
```

A regular `Page<T>` can be modified. A `RegisteredPage<T>` cannot. This is a typestate distinction: once registered, the Page's definition is fixed.

Rendering is only available on registered Pages.

```rust
let message = page.render(app, Dataless).await?;
```

Rendering requires the App because interactive components must be registered with the application's persistence layer before their message can be sent.

### Page data

The Page's generic type is the shared data scope for its components and executions.

```rust
#[derive(Serialize, Deserialize)]
pub struct UserLevelInfo {
    user_id: String,
}

let page = Page::<UserLevelInfo>::new("UserLevel")
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

The data passed to `render` is serialized and persisted alongside the Page's interactive execution references. When an interaction occurs, the framework reconstructs the original data and passes it to the corresponding execution.

The application database is separate from the framework's interaction Store. `App<D>` is generic over the application's database type, so callbacks access the concrete database API directly through `ctx.app.db`. The application chooses the structure of `D`; it may expose named repositories such as `db.messages` and `db.users`, a SQLx SQLite pool, or both. There is no type-erased `get::<T>()` lookup and no string-based table lookup.

SQLite is the database backend. The framework should expose ordinary SQL access rather than inventing its own query language or ORM. SQLx is the preferred asynchronous Rust integration to investigate: it supports SQLite, raw SQL queries, and a query builder. Developers may use SQLx directly or layer an independent query builder/ORM over it. The framework should not require developers to use a particular higher-level abstraction.

The framework Store is responsible for interaction routing and is distinct from the application's `D` database value. Whether both responsibilities share the same underlying SQLite file or use separate connections/files is an implementation/configuration decision, not a public API requirement.

The data type must implement `Serialize` and `DeserializeOwned`, and satisfy the concurrency requirements of the asynchronous application.

## 2. Static and dynamic values

A component property should accept either a static value or a callback that resolves to that value.

The public API should not require separate constructors or methods for static and dynamic values.

```rust
Page::<Dataless>::new("Example")
    .add_text("This is static text.")
    .add_text(|ctx, data| async move {
        Ok("This is dynamically generated text.".to_string())
    });
```

The same principle applies to component properties.

```rust
Button::new(
    "Show Level",
    "ButtonId",
    ButtonStyle::Secondary,
    |ctx, data| async move {
        Ok(())
    },
);
```

The button's label and style should both accept static values or callbacks. Its disabled state is a static or dynamic Boolean. Its emoji is optional and may be supplied statically or dynamically. Its developer-defined ID is always a static string. The execution remains an explicit callback.

### Into conversions

The implementation should use a common internal representation for values that can be either static or dynamic.

Conceptually:

```rust
enum Value<T> {
    Static(T),
    Dynamic(/* asynchronous callback returning T */),
}
```

Component methods should accept values convertible into this representation.

```rust
.add_text("Hello")
.add_text(dynamic_callback)
```

This should apply consistently to properties that can reasonably be resolved at render time:

- Text content and descriptions.
    
- Button labels and styles.
    
- Select menu placeholders and options.
    
- Thumbnail URLs and descriptions.
    
- Container accent colors.
    
- Media URLs.
    
- Disabled states and other component settings.
    

The default rule is that a property accepts either its ordinary value or an asynchronous callback returning that value, unless there is a specific reason not to.

A callback passed directly to a property method is treated as a dynamic value; developers must not need to wrap it in `dynamic(...)`. The `dynamic(...)` adapter is not required by the ordinary API.

Both dynamic property callbacks and interactive execution callbacks are asynchronous and use the same general callback context and Page-data inputs. Dynamic callbacks resolve a property value; execution callbacks perform an interaction. Dynamic callbacks are evaluated during rendering and do not receive an interaction-specific Serenity context because no interaction has occurred. They receive the framework callback context, which exposes the App and the services available during rendering.

A dynamic value determines what a component looks like. An execution determines what happens when someone interacts with a component.

The latter remains an explicit callback.

### Fixed slots and compile-time component counts

Some Discord components have a fixed maximum number of children. Rather than accepting an arbitrary array and discovering invalid configurations at runtime, the API should expose a fixed number of slots.

Each slot can contain a static value, a dynamic callback, or no value. The API should accept ordinary values through `Into` conversions where Rust permits it, so callers can pass a string directly instead of manually writing `Some("...")`. Explicit `None` remains available when a slot is absent.

For example, a Section supports up to three Text Displays:

```rust
Section::new()
    .set_text(
        "User information",
        None,
        None,
    )
```

A Section with three text blocks:

```rust
Section::new()
    .set_text(
        "User information",
        dynamic_description,
        "Additional information",
    )
```

The same idea applies to button rows:

```rust
.add_button_row((
    Some(button1),
    Some(button2),
    None,
    None,
    None,
))
```

A fully populated row:

```rust
.add_button_row((
    Some(button1),
    Some(button2),
    Some(button3),
    Some(button4),
    Some(button5),
))
```

This makes the maximum number of slots explicit in the API. An omitted slot is represented by `None`, rather than relying on runtime validation to discover an invalid number of children.

The implementation should use a fixed-size representation, such as a tuple or fixed-size array, and convert it into the appropriate Serenity component structure.

The exact conversion types should be tested for Rust inference and ergonomic `Into` behavior before the signatures are finalized. In particular, `None` must not require awkward type annotations.

## 3. Components V2

The framework should expose Discord's Components V2 functionality through a composable API.

The API should model how developers build interfaces, rather than mechanically exposing every Serenity builder method.

### Text Display

A Text Display presents Markdown-formatted text.

```rust
Page::<Dataless>::new("Example")
    .add_text("# Welcome")
    .add_text("This is another text block.");
```

Each call adds a separate Text Display. Text should not be silently concatenated with an existing component.

Dynamic content uses the same method:

```rust
.add_text(|ctx, data| async move {
    Ok(format!("Welcome, {}!", data.username))
})
```

### Section

A Section places text alongside an accessory. Discord supports one to three Text Displays and an accessory.

The public API should expose three explicit text slots.

```rust
Section::new()
    .set_text(
        "User information",
        "Account details",
        None,
    )
    .set_accessory(
        Thumbnail::new("https://example.com/avatar.png")
    )
```

Each text slot can independently be static, dynamic, or absent. The API should accept a string directly through an `Into`-style conversion as well as an explicit absence value, without forcing callers to wrap every present string in `Some`.

The accessory can be a Thumbnail or a Button.

```rust
Section::new()
    .set_text(
        "View additional information.",
        None,
        None,
    )
    .set_accessory(
        Button::new(
            "View",
            "ViewButton",
            ButtonStyle::Secondary,
            |ctx, data| async move {
                Ok(())
            },
        )
    )
```

The framework should ensure that the resulting Section satisfies Discord's required structure.

### Container

A Container groups components into a visually distinct block. It can have an accent color and spoiler behavior.

```rust
Container::new()
    .set_accent_color(0x5865F2)
    .add_text("Account information")
    .add_separator()
    .add_text("Additional details")
```

A Container should support the same general component-building capabilities as a Page, limited by the component types Discord permits inside a Container. Containers are independent component groups, not alternate Pages. A Page may contain multiple Containers, and each Container may contain its own supported components and compositions.

### Separator

A Separator provides visual spacing and can optionally display a divider.

```rust
Separator::new()
    .set_spacing(SeparatorSpacing::Large)
    .set_divider(true)
```

### Thumbnail

A Thumbnail displays a small image, usually as a Section accessory.

```rust
Thumbnail::new("https://example.com/avatar.png")
    .set_description("User avatar")
```

The image source and description should support static or dynamic values.

### Media Gallery

A Media Gallery displays a collection of images or videos.

```rust
MediaGallery::new()
    .add_media("https://example.com/image1.png")
    .add_media("https://example.com/image2.png")
```

Individual media items may have descriptions.

### File

A File component displays an uploaded attachment within a Components V2 message.

```rust
File::new("document.pdf")
```

The framework should associate the component with the corresponding uploaded attachment. Uploading a file and displaying it are related operations, but they are not the same operation.

## 4. Interactive components

Interactive components create executions. Each execution has an ID unique within its Page.

### Button

```rust
Button::new(
    "Show Level",
    "ButtonId",
    ButtonStyle::Secondary,
    |ctx, data: UserLevelInfo| async move {
        let level_info = sqlx::query_as::<_, UserLevel>("SELECT * FROM user_level WHERE user_id = ?")
            .bind(&data.user_id)
            .fetch_one(&ctx.app.db.pool)
            .await?;

        let render = ctx
            .app
            .pages
            .query("ShowUserLevel")?
            .render(ctx.app, ShowUserLevelData {
                level: level_info.level,
            })
            .await?;

        ctx.edit_message(render).await?;

        Ok(())
    },
)
```

The Button has a required, developer-defined static ID. Its label, style, disabled state, and optional emoji can each be static or dynamic. The disabled state is a Boolean. Emoji is optional and may be absent. The execution is always an explicit asynchronous callback. The ID is not a dynamic property.

### Link Button

A Link Button opens a URL rather than invoking an application execution.

It should be a separate component because it does not produce an interaction.

```rust
LinkButton::new(
    "Visit Website",
    "https://example.com",
)
```

Its label and URL should support static or dynamic values. A Link Button should also support a static or dynamic disabled state. It does not have an execution callback.

### Button rows

Developers should not have to manually create Action Rows.

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

The framework creates the underlying Action Row.

Discord supports up to five buttons in an Action Row. Each slot accepts a Button directly or `None` when absent; callers should not have to wrap present buttons in `Some`. The fixed-size tuple or array makes the five-slot maximum explicit. Static and dynamic Button properties remain supported regardless of which slot contains the Button.

### Selection menus

A Page should expose one method for adding a selection menu.

```rust
.add_selection_menu(selection_menu)
```

The selection menu itself determines its type.

The framework should support:

- String Select.
    
- User Select.
    
- Role Select.
    
- Mentionable Select.
    
- Channel Select.
    

These can be represented by distinct selection-menu types implementing a shared trait, or by an enum that unites the supported types.

The important point is that the Page API accepts one selection-menu abstraction rather than requiring separate Page methods for every Discord select type.

A String Select might look like this:

```rust
StringSelect::new(
    "Choose a class",
    "ClassSelect",
    [
        SelectOption::new("Warrior", "warrior"),
        SelectOption::new("Mage", "mage"),
        SelectOption::new("Rogue", "rogue"),
    ],
    |ctx, data, selection| async move {
        // Handle the selected value or values.
        Ok(())
    },
)
```

The option labels, descriptions, and other display properties should support static and dynamic values.

The selected values are interaction data. They are not part of the Page's serialized data.

Entity select menus return Discord entities rather than arbitrary strings. Their callbacks should expose the appropriate selected users, roles, channels, or mentionables.

## 5. Frame

A Frame is a dynamic content region inside a Page. It is intended for collections whose complete contents are not known when the Page is defined, such as a settings list, directory, or catalogue.

The developer supplies a callback that produces a collection of heterogeneous framework elements. The framework resolves that collection during rendering and lays it out within Discord's message limits. The developer should not have to calculate page boundaries or manually split the collection.

```rust
Page::<UserListData>::new("UserList")
    .add_text("# Users")
    .add_frame(|ctx, data| async move {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&ctx.app.db.pool)
            .await?;

        Ok(users.into_iter()
            .map(|user| {
                Element::text(format!("{} - Level {}", user.name, user.level))
            })
            .collect())
    })
    .register(app)?;
```

A Frame resolves to a collection of complete elements. Elements may be Text Displays, Sections, Containers, Separators, media, buttons, selection menus, or supported compositions of those components. The public API should let developers compose these elements naturally; a common internal element representation is an implementation detail.

### Frame position and pagination

Pagination is a layout operation over the Frame's resolved elements. It is not a user session and does not create per-user or per-message state.

The developer should be able to opt a Frame into automatic pagination through a simple configuration method, conceptually:

```rust
.add_frame(frame_callback)
.paginate()
```

The exact builder placement may be adjusted to fit the final Frame API, but pagination belongs to the Frame because it paginates that Frame's collection, not the entire Page definition.

At render time, the framework:

1. Resolves the Frame callback once for that render.
2. Resolves all dynamic properties in the returned elements.
3. Measures the resulting complete component tree against Discord's limits.
4. Splits the Frame collection between complete elements into as many valid pages as required.
5. Repeats the Page's static components on every generated page.
6. Adds framework-generated Previous and Next controls when more than one page exists.

The resolved element tree used for measurement must also be the tree used for rendering. A dynamic callback must not be evaluated twice and produce different values for pagination planning and the actual message.

The layout algorithm must account for the full Discord payload, including nested component counts, message-wide text budgets, Container child limits, Section limits, media limits, and the static Page components repeated on every generated page. Pagination controls also consume component and text budget. If navigation controls are needed, their cost must be reserved before the final page boundaries are calculated.

Pagination splits only between complete elements. It must not silently truncate a component or split an interactive component into invalid pieces. If one indivisible element cannot fit on an otherwise empty page, rendering returns a descriptive error. Any future support for splitting an oversized text element must be explicit and documented.

### Navigation API and behavior

Navigation is generated by the framework; developers should not need to write Previous and Next callbacks themselves. The default navigation consists of a button row appended after the Page's content. Previous and Next are disabled at the first and last page respectively. A Frame that fits on one page has no navigation row.

The current page index is represented as part of the navigation interaction's identity. It is not mutable session state stored for a Discord user or message. Conceptually, a navigation identity is derived from the Page ID, Frame ID, navigation direction, serialized Page data, and the page index from which the navigation is being performed.

When Previous or Next is clicked, the framework derives the target index, reruns the Frame callback using the same serialized Page data, resolves the current collection, recalculates its page boundaries, and renders the target page. It then edits the message containing the interaction. The Frame callback is rerun on every navigation interaction, using the same serialized Page data. Therefore pagination is a stateless view of the collection as it exists at the time of that interaction, not a snapshot of the collection from the first render. If the collection changes, page boundaries and the contents at a given index may change. If the requested target index no longer exists, the framework returns a clear out-of-range pagination error rather than silently clamping the index or selecting an unrelated page.

No per-user identity, message-instance identity, mutable page counter, or pagination session is introduced. Two renders with the same Page definition, Page data, Frame definition, and page index have the same pagination identity and behavior. Different users viewing identical data at the same position are intentionally indistinguishable to the framework.

The navigation custom IDs are backed by the ordinary interaction-routing Store so the framework can resolve them after process restarts. The Frame ID, navigation operation, and source page index are encoded into the generated execution ID. The Page data remains the serialized Page data. These values are part of the deterministic routing identity, not additional Store columns or mutable session state.

### Frame and interactive element identity

Interactive components generated inside a Frame need stable developer-defined keys. Their identities must not depend on their temporary array position, because inserting or removing an earlier element must not redirect an existing component's callback.

Every Frame element containing an interactive component must have a non-empty, developer-defined key. The key identifies that element within its Frame and applies to its interactive descendants. Keys must be unique within the Frame's resolved collection. Missing or duplicate keys are rendering errors. The Frame ID and element key are incorporated into the generated execution ID, together with the component's execution ID. They are not extra routing-record fields. Keys must remain stable when the collection is reordered or when other elements are inserted or removed.

### Frame placement

A Frame occupies one declared position in the Page's component hierarchy. Static components before and after it remain in that order and are repeated on every generated page. The Frame may be composed inside a Container only if the resulting expanded Container remains valid on every page; the layout algorithm must account for the Container's direct-child limit. Unsupported nesting or combinations must produce a clear validation error rather than silently changing the hierarchy.

The initial pagination implementation may require one paginated Frame per Page. Supporting multiple independently paginated Frames in one Page would require defining how their page counts and navigation combine, so that behavior must not be implied until it is specified.

## 6. Modals

A Modal describes a form that Discord displays after an interaction.

A Modal is not a message component. It is opened in response to an interaction and produces a separate interaction when submitted.

A Modal should be its own blueprint type.

```rust
#[derive(Serialize, Deserialize)]
pub struct EditUserData {
    user_id: String,
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
    .on_submit(|ctx, data, submission| async move {
        let username = submission.text("username")?;
        let description = submission.text("description")?;

        sqlx::query("UPDATE users SET username = ?, description = ? WHERE id = ?")
            data.user_id,
            username,
            description,
        ).await?;

        ctx.reply("User updated successfully.").await?;

        Ok(())
    })
    .register(app)?;
```

The Modal's generic data works like a Page's data. It is serialized when the Modal is opened and reconstructed when the form is submitted.

Modal rendering is transactional with respect to framework persistence. The framework resolves all dynamic input properties, validates the complete Modal payload, and builds the rendered Modal before inserting its submit-execution record into the Store. If resolution, validation, or payload construction fails, no submit record is persisted. A successful render persists the record before the Modal is sent as an interaction response.

The submitted form values are separate. They come from Discord and are provided to the submission callback through a `ModalSubmission` object.

### Opening a Modal

A Modal is opened by an existing interaction, usually a button execution.

```rust
Button::new(
    "Edit User",
    "EditUserButton",
    ButtonStyle::Primary,
    |ctx, data: UserLevelInfo| async move {
        let modal = ctx.app.modals
            .query("EditUser")?
            .render(ctx.app, EditUserData {
                user_id: data.user_id,
            })
            .await?;

        ctx.open_modal(modal).await?;

        Ok(())
    },
)
```

Opening a Modal is an interaction response. It is not an ordinary message edit or follow-up.

### Modal components

The Modal API should support the input components Discord permits:

- Text Input.
    
- String Select.
    
- User Select.
    
- Role Select.
    
- Mentionable Select.
    
- Channel Select.
    
- File Upload.
    
- Radio Group.
    
- Checkbox Group.
    
- Checkbox.
    

The API should use simple input definitions and handle Discord's required Label structures internally.

```rust
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
    .add_select(
        "category",
        "Category",
        categories,
    )
    .add_checkbox(
        "urgent",
        "This issue is urgent",
    )
    .on_submit(|ctx, data, submission| async move {
        Ok(())
    })
```

Modal inputs should have stable developer-defined IDs. Those IDs are used to retrieve their submitted values.

### Typed submission data

The long-term goal should be to support deserializing Modal submissions into a developer-defined Rust struct.

```rust
#[derive(Deserialize)]
struct EditUserForm {
    username: String,
    description: String,
}

.on_submit(|ctx, data, form: EditUserForm| async move {
    sqlx::query("UPDATE users SET username = ?, description = ? WHERE id = ?")
        data.user_id,
        form.username,
        form.description,
    ).await?;

    Ok(())
})
```

This avoids requiring developers to manually retrieve strings by component ID and then parse them.

The framework should map component IDs to struct fields and convert submitted values into the appropriate Rust types. Missing or invalid values should return errors.

The field-name mapping is normative:

- Text Input values deserialize as `String`.
- String Select, Radio Group, and Checkbox Group values deserialize as `Vec<String>`, containing submitted option values.
- Checkbox values deserialize as `bool`.
- User, Role, and Channel Select values deserialize as arrays of their Discord ID types (`Vec<UserId>`, `Vec<RoleId>`, and `Vec<ChannelId>`).
- Mentionable Select values deserialize as arrays of tagged values containing the selected kind (`user` or `role`) and its Discord ID.
- File Upload values deserialize as arrays of attachment metadata containing the attachment ID, filename, URL, size, and optional content type. The framework does not download file bytes as part of deserialization.

The raw `ModalSubmission` accessors remain available when application code needs complete typed Serenity entities or attachment objects. A Serde type mismatch, missing required field, or malformed ID is an error; typed values must not be silently discarded or coerced to strings.
The exact mechanism for this should be designed around Serde rather than inventing a second serialization system.

## 7. Interaction Context

Every interactive execution receives `ctx` and the Page's typed `data`.

`ctx` is the framework's context wrapper around the relevant Serenity interaction context. It exposes Serenity's interaction API as needed and also exposes the application through `ctx.app`. The wrapper must provide the operations needed to respond to the interaction; merely exposing a raw Serenity context is not sufficient if Serenity's API does not directly support the framework's required routing and response behavior.

Dynamic property callbacks also receive the framework context and Page data, but they run during rendering rather than in response to a Discord interaction. They therefore cannot use interaction-response operations that require an active interaction.

Callbacks are asynchronous and return `Result`. The intended ergonomic form is an asynchronous closure such as `|ctx, data| async move { ... }`, allowing application code to use `.await` naturally.

The exact ownership and lifetime representation of `ctx` is an implementation decision. The public contract does not prescribe `Arc`, `&mut`, or a particular Serenity context type.

Page data is supplied as an immutable view to callbacks. Callbacks observe the data; they do not mutate the original Page data. The concrete reference, lifetime, and any sharing required for thread safety are implementation decisions.

```rust
|ctx, data| async move {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&data.user_id)
        .fetch_one(&ctx.app.db.pool)
        .await?;

    ctx.reply("Hello!").await?;

    Ok(())
}
```

The context should expose operations relevant to the interaction type:

- Reply to an interaction.
    
- Defer an interaction response.
    
- Edit the message containing the component.
    
- Edit the original interaction response.
    
- Send a follow-up message.
    
- Delete an interaction response.
    
- Open a Modal.
    

Response operations have distinct Discord semantics. `reply`, `defer`, `edit_message`, and `open_modal` are initial interaction responses and may be used only while the interaction has not already been acknowledged. They map respectively to a message response, deferred response, message update, and Modal response. `edit_original_response`, `followup`, and `delete_response` operate on the original interaction webhook response; `followup` creates a separate webhook message. Each method sends its documented payload through the App's configured HTTP client and returns Discord/API errors to the callback. Calling an initial-response method after the interaction has already been acknowledged returns an error. The framework does not silently substitute one response kind for another.

These response operations must be testable through an injected or local HTTP test double. Unit tests must not require a live Discord connection or a real bot token.


The context also exposes interaction information, such as the user who interacted, the guild and channel, selected values, and submitted Modal data.

The context is temporary. It belongs to the current interaction and is never stored inside a Page or persisted in the database.

## 8. App and registration

`App` is generic over the application's database type:

```rust
pub struct App<D> {
    pub db: D,
    // Registered definitions, framework Store, and interaction routing.
}
```

The application supplies its own concrete database type. This allows strongly typed, named access such as `ctx.app.db.messages`, `ctx.app.db.users`, or `ctx.app.db.settings`, without string lookups or a type-erased `get::<T>()` method. The application may expose repositories, a SQLx SQLite pool, or a combination of both.

SQLite is the required database backend. SQL is the underlying query interface. The framework should not invent a proprietary query API or require a custom ORM. SQLx is the preferred asynchronous Rust integration to evaluate for SQLite connections, raw SQL queries, and query-builder support. Higher-level SQL abstractions may be used independently on top of the SQL interface.

The framework interaction Store is a separate responsibility from the application's database API. It stores the custom-ID routing records described below. Whether the Store and application data use the same SQLite file or separate SQLite files/connections is not prescribed by the public API.

The App owns the registered Pages and Modals, the framework Store, and the interaction router. Its registries must hold Pages and Modals with different generic data types while preserving the concrete application database type used by their callbacks. Type erasure needed to hold heterogeneous Page and Modal definitions is an internal implementation detail.

A registered Page or Modal is immutable. Registration consumes the builder and places its definition in the App's registry. Registration is not restricted to application startup; it may occur whenever the API's ownership and synchronization rules permit it. The specification does not impose a startup-only lifecycle.

Page IDs and Modal IDs share one application-wide identifier namespace. Registering a duplicate ID is an error.

Rendering a Page uses its registered definition and the App:

```rust
let message = app.pages
    .query("UserLevel")?
    .render(&app, user_data)
    .await?;
```

The App is supplied to rendering and callbacks; it is not stored inside the Page definition.

## 9. Persistence and interaction routing

Discord returns a component's `custom_id` when a user interacts with it. The framework Store maps that ID to the definition and data needed to route the interaction.

The framework Store is distinct in responsibility from the application's typed database API. It is SQLite-backed and persists routing records so that an interaction can be resolved after a process restart, provided the same Page and Modal definitions have been registered again. Its connection arrangement is an implementation detail.

Each routing record has exactly four fields: the deterministic hash, the Page ID (or Modal ID for a Modal execution), the execution ID, and the serialized Page or Modal data. There are no additional per-user, per-message, Frame, pagination, timestamp, or expiration fields. Frame identity, stable element keys, and pagination operation/position are encoded into the execution ID where needed.

The custom ID is the deterministic hash stored in the routing record. It is computed programmatically from the Page or Modal ID, execution ID, and serialized data using a stable serialization format and deterministic hash algorithm. The same three inputs must always produce the same hash, including across process restarts. The hash must fit Discord's custom-ID limit. The implementation may choose the concrete hash algorithm, but it must not use random IDs, process-specific salts, or time-dependent values. On an existing hash, the Store must verify that the other three fields match; a hash collision must be reported rather than overwriting or misrouting the record.

This deliberately means identical definitions and identical data share interaction identities. The framework does not distinguish users or message instances, and it does not maintain per-user or per-message sessions. The framework's Store is a routing lookup, not a session store.

Closures are never serialized. The registered Page or Modal definition owns the executable callback. Persisted records contain only enough information to identify that callback and reconstruct its typed data.

### Routing

When an interaction arrives:

1. App receives the Discord interaction.
2. App reads the component's `custom_id`.
3. The Store resolves the custom ID to its routing record.
4. App locates the registered Page or Modal definition.
5. The definition locates the requested execution.
6. The definition deserializes the stored data into its own generic type.
7. The callback runs with `ctx` and an immutable reference to the reconstructed data.

The App does not need to know the Page or Modal's generic data type. The registered definition knows its own type and performs deserialization and callback dispatch.

A valid custom ID does not grant permission to perform an operation. Application callbacks remain responsible for authorization.

### Determinism and record reuse

Repeated renders of the same Page definition and identical serialized data reuse the same routing identity for the same execution. Different users rendering those same circumstances are intentionally not distinguished.

The Store must handle an identical identity consistently. It must not create user-specific variants or overwrite a record with semantically different callback data. If two different definitions or execution identities produce the same digest, the framework must detect and report the collision rather than route to an arbitrary callback. Routing records do not expire and are not removed merely because time has passed. The framework does not implement TTLs, automatic expiration, or session cleanup.

## 10. Errors

The framework should use ordinary Rust `Result` values for operations that can fail.

Examples include:

- Registering duplicate Page or Modal IDs.
    
- Registering duplicate execution IDs within a Page.
    
- Rendering a component with invalid configuration.
    
- Resolving an unknown Page or Modal.
    
- Resolving an unknown execution.
    
- Deserializing persisted data.
    
- Retrieving a missing or invalid Modal submission value.
    
- Resolving an invalid Frame.
    
- Encountering a component that cannot fit within Discord's limits.
    
- Sending an invalid Discord component structure.
    

Errors should preserve useful context, especially the Page or Modal ID and execution ID involved.

Fixed-size component slots should prevent invalid child counts from being expressible through the ordinary public API wherever possible. Runtime validation is still necessary for constraints that depend on resolved dynamic values.

## 11. Design principles

**One public interface for static and dynamic values.** A property accepts either its ordinary value or a callback that resolves to that value.

**Executions are explicit.** A button click, selection, or Modal submission always has an explicit callback.

**Page data is shared.** Components and executions belonging to a Page use the same generic data type.

**Structure is declarative.** The Page defines its fixed structure. Frames provide dynamic regions.

**Frames own dynamic collections.** A Frame resolves components at render time and handles pagination without forcing the developer to implement navigation.

**Component limits should be represented in the API.** Fixed-size slots should make Discord's structural constraints explicit rather than relying on avoidable runtime errors.

**Discord implementation details stay hidden.** The framework handles component nesting, custom IDs, serialization, and interaction routing.

**Registered definitions are immutable.** Build first, register once, then render.

**Routing is persistent, sessions are absent.** The framework Store preserves deterministic callback routing across restarts when the same definitions are registered again. It does not track users or message instances.

**Errors are explicit.** Invalid persisted references, missing values, and failed executions return errors instead of panicking.

A String Select option has no independent disabled state in Discord's component payload. The framework must reject an option configured with `SelectOption::set_disabled(true)` rather than silently omit the setting or render an invalid component. A dynamic option-disabled value is checked after it is resolved. Disabling the complete select menu remains supported.


## 12. Discord constraints

The implementation must respect Discord's actual component limits.

For Components V2 messages:

- A message can contain up to 40 total components, including nested components.
    
- Text Display content shares a 4,000-character message-wide budget.
    
- A Section supports one to three Text Displays and one accessory.
    
- A button Action Row supports one to five buttons.
    
- A select menu occupies its own Action Row.
    
- A String Select supports up to 25 options.
    
- A Media Gallery supports up to 10 media items.
    
- A Container supports up to 10 direct child components.
    
- A component's custom ID is limited to 100 characters.
    

Components V2 messages require the `IS_COMPONENTS_V2` flag. Traditional message content and embeds cannot be used in the normal way with Components V2.

For Modals:

- A Modal title is limited to 45 characters.
    
- A Modal supports up to five top-level components.
    
- Input labels are limited to 45 characters.
    
- Label descriptions are limited to 100 characters.
    
- Text Inputs support up to 4,000 characters.
    
- Select menus support up to 25 options.
    

These constraints should be validated by the framework, including after dynamic values and Frame contents have been resolved.

The framework should not assume that a Page is valid merely because its static definition is valid. Dynamic content can exceed the limits at render time.

A Text Input's configured `max_length` cannot exceed 4,000, and its `min_length` cannot exceed its `max_length`. Each Modal Select, Radio Group, and Checkbox Group supports no more than 25 options. Static violations are rejected during registration; dynamic label, placeholder, description, and option values are validated after resolution during rendering.


## 13. Implementation priorities

The framework should be implemented in stages.

1. Page builder and registration.
    
2. Static and dynamic property resolution.
    
3. Component composition and fixed-size slots.
    
4. Button executions and persistence.
    
5. Selection menus and interaction routing.
    
6. Modal registration and submission handling.
    
7. Frame component generation.
    
8. Frame pagination and automatic navigation.
    
9. Typed Modal submission deserialization.
    
10. More advanced Frame data sources and pagination optimizations.
    

This order establishes the core execution and rendering model before adding the more complex dynamic collection and pagination behavior.

The end result should make building a Discord interface mostly ordinary composition:

```rust
let page = Page::<UserListData>::new("UserList")
    .add_container(
        Container::new()
            .add_text("# User Directory")
            .add_separator()
            .add_frame(|ctx, data| async move {
                let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&ctx.app.db.pool)
            .await?;

                Ok(users.into_iter()
                    .map(|user| {
                        Element::text(format!(
                            "{} - Level {}",
                            user.name,
                            user.level
                        ))
                    })
                    .collect())
            })
            .paginate()
    )
    .register(app)?;
```

The framework handles turning this definition into valid Discord messages, splitting dynamic content into pages, persisting interactive executions, and routing future interactions back to the correct Rust callbacks.

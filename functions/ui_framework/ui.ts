import { ContainerBuilder, MessageFlags, TextDisplayBuilder, SectionBuilder, type MessageActionRowComponentBuilder, ButtonStyle, Client, SeparatorSpacingSize, SeparatorBuilder, MediaGalleryBuilder, type InteractionUpdateOptions } from "discord.js";
import { ActionRowBuilder, ButtonBuilder, ThumbnailBuilder, type Interaction } from "discord.js";
import { createHash } from 'crypto';

type Element = Section | ActionRow | TextDisplay | Window | Separator | MediaGallery
type ButtonExecution = (page: Page, interaction: Interaction, data: any) => Promise<void>;
type DynamicProp<T> = T | ((page: Page, index: number, interaction?: Interaction) => Promise<T>);
type DynamicButtonAttributes = {
  style: DynamicProp<ButtonStyle>,
  label?: DynamicProp<string>,
  emoji?: DynamicProp<string>,
  disabled?: DynamicProp<boolean>
};
type DynamicThumbnailAttributes = {
  url: DynamicProp<string>,
}
type DynamicMediaGalleryAttributes = {
  url: DynamicProp<string>,
  description?: DynamicProp<string>,
  spoiler?: DynamicProp<boolean>,
}

export enum ProgressBarSize {
  Small,
  Normal,
  Medium,
  Large,
}

async function resolve_prop<T>(prop: DynamicProp<T>, page: Page, index: number, interaction?: Interaction): Promise<T> {
  return typeof prop === 'function' ? await (prop as Function)(page, index, interaction) : (prop as T);
}

export class Page {
  public customID: string;
  public isContainer: boolean;
  public dynamicStartMax: number;
  public isEphemeral: boolean;
  readonly staticElements: Element[];
  readonly dynamicElements: Element[];
  readonly cache = new Map<string, ButtonExecution>;
  readonly data = new Map<string, any>;

  constructor(custom_id: string, client: Client, is_container?: boolean, is_ephemeral?: boolean, static_elements?: Element[], dynamic_elements?: Element[], dynamic_start_max?: number) {
    client.pages.set(custom_id, this);
    this.customID = custom_id;
    this.isContainer = is_container ?? false;
    this.isEphemeral = is_ephemeral ?? true;
    this.staticElements = static_elements ?? [];
    this.dynamicElements = dynamic_elements ?? [];
    this.dynamicStartMax = dynamic_start_max ?? 5;
    for (const element of this.staticElements) element.bind(this);
    for (const element of this.dynamicElements) element.bind(this);
  }

  public addStaticElement(element: Element): Page { element.bind(this); this.staticElements.push(element); return this; }
  public addDynamicElement(element: Element): Page { element.bind(this); this.dynamicElements.push(element); return this; }

  public async next(index: number): Promise<number> {
    return index + this.dynamicStartMax;
  }

  public async previous(index: number): Promise<number> {
    return index - this.dynamicStartMax
  }

  public async update(index: number, interaction: Interaction): Promise<Page> {
    if (interaction?.isButton()) await interaction.update(await this.render(index, interaction) as InteractionUpdateOptions);
    return this;
  }

  public async render(index: number, interaction?: Interaction) {
    const container = new ContainerBuilder();
    let flags = [];
    flags.push(MessageFlags.IsComponentsV2);
    if (this.isEphemeral) flags.push(MessageFlags.Ephemeral);

    for (const element of this.staticElements) await element.apply(container, index, interaction);
    return { 
        components: this.isContainer ? [container] : container.components, 
        flags,
    };
  }
}




/*
 *
 *ACCESSORIES
 *
 */
export class Button {
  public execution: ButtonExecution;
  public page!: Page;
  public data: DynamicProp<any>;
  public dynAttributes: DynamicButtonAttributes;

  constructor(execution: ButtonExecution, builder: DynamicButtonAttributes, data: DynamicProp<any>) {
    this.execution = execution;
    this.data = data;
    this.dynAttributes = builder;
  }

  public bind(page: Page): Button {
    this.page = page;
    return this;
  }

  public async compute(index: number, interaction?: Interaction): Promise<ButtonBuilder> {
    const attributes = this.dynAttributes;
    const builder = new ButtonBuilder();

    if (!!attributes.label) builder.setLabel(await resolve_prop(attributes.label, this.page, index, interaction));
    if (!!attributes.emoji) builder.setEmoji(await resolve_prop(attributes.emoji, this.page, index, interaction));
    if (!!attributes.disabled) builder.setDisabled(await resolve_prop(attributes.disabled, this.page, index, interaction));
    builder.setStyle(await resolve_prop(attributes.style, this.page, index, interaction))

    const data = await resolve_prop(this.data, this.page, index, interaction);

    const hash = this.cache(data)
    builder.setCustomId("ui-" + this.page.customID + "-" + hash);

    return builder;
  }

  public async apply(container: SectionBuilder, index: number, interaction?: Interaction) {
    container.setButtonAccessory(await this.compute(index, interaction));
  }

  public cache(data: any): string {
    const dataIdentifier = (data && typeof data === 'object' && 'customID' in data) ? data.customID : JSON.stringify(data);
    const hash = createHash("sha256").update(dataIdentifier + this.execution.toString()).digest('base64url');
    this.page.cache.set(hash, this.execution);
    this.page.data.set(hash, data);
    return hash;
  }
}

export class Thumbnail {
  public builder!: ThumbnailBuilder;
  public page!: Page;
  public dynAttributes: DynamicThumbnailAttributes
  constructor(builder: DynamicThumbnailAttributes ) {
    this.dynAttributes = builder;
    this.builder = new ThumbnailBuilder();
  }
  public bind(page: Page) { this.page = page; }
  public async apply(container: SectionBuilder, index: number, interaction?: Interaction) {
    this.builder.setURL(await resolve_prop(this.dynAttributes.url, this.page, index, interaction));
    container.setThumbnailAccessory(this.builder);
  }
}



/*
 *
 * ELEMENTS
 *
 */
export class TextDisplay {
  public builder: TextDisplayBuilder;
  public page!: Page;
  public dynAttributes!: { string: DynamicProp<string> };
  constructor(builder: DynamicProp<string>) {
    this.dynAttributes = { string: builder };
    this.builder = new TextDisplayBuilder();
  }
  public bind(page: Page) { this.page = page; }
  public async apply(container: ContainerBuilder | SectionBuilder, index: number, interaction?: Interaction) {
    this.builder.setContent(await resolve_prop(this.dynAttributes.string, this.page, index, interaction));
    container.addTextDisplayComponents(this.builder);
  }
}

export class Section {
  public builder!: SectionBuilder;
  public page!: Page;
  public dynAttributes!: { text_display: TextDisplay };
  constructor(text: DynamicProp<string>, public accessory: Button | Thumbnail) {
    this.dynAttributes = { text_display: new TextDisplay(text) }
  }
  public bind(page: Page) { this.page = page; this.accessory.bind(page); }
  public async apply(container: ContainerBuilder, index: number, interaction?: Interaction) {
    this.builder = new SectionBuilder();
    await this.dynAttributes.text_display.apply(this.builder, index, interaction);
    await this.accessory.apply(this.builder, index, interaction);
    container.addSectionComponents(this.builder);
  }
}

export class ActionRow {
  public builder: ActionRowBuilder<MessageActionRowComponentBuilder> = new ActionRowBuilder();
  public page!: Page;
  constructor(public accessorys: Button[]) {}
  public bind(page: Page) { this.page = page; for (const a of this.accessorys) a.bind(page); }
  public async apply(container: ContainerBuilder, index: number, interaction?: Interaction) {
    this.builder.setComponents(await Promise.all(this.accessorys.map(a => a.compute(index, interaction))));
    container.addActionRowComponents([this.builder]);
  }
}

export class Window {
  public page!: Page;
  public bind(page: Page) { this.page = page; }
  public async apply(container: ContainerBuilder, index: number, interaction?: Interaction,) {
    const elements = this.page.dynamicElements.slice(index, index + this.page.dynamicStartMax);
    for (const e of elements) await e.apply(container, index, interaction);
  }
}

export class Separator {
  public builder: SeparatorBuilder = new SeparatorBuilder();
  public page!: Page;
  constructor(public divider: DynamicProp<boolean>, public spacing: DynamicProp<SeparatorSpacingSize>) {}
  public bind(page: Page) { this.page = page; }
  public async apply(container: ContainerBuilder, index: number, interaction?: Interaction) {
    this.builder.setDivider(await resolve_prop(this.divider, this.page, index, interaction)).setSpacing(await resolve_prop(this.spacing, this.page, index, interaction))
    container.addSeparatorComponents(this.builder)
  }
}

export class MediaGallery {
  public builder!: MediaGalleryBuilder;
  public page!: Page;
  constructor(public mediaGalleryItems: DynamicMediaGalleryAttributes[]) {}
  public bind(page: Page) { this.page = page; }
  public async apply(container: ContainerBuilder, index: number, interaction?: Interaction) {
    this.builder = new MediaGalleryBuilder();
    for (const media of this.mediaGalleryItems) {
      const spoiler = await resolve_prop(media.spoiler, this.page, index, interaction);
      const description = await resolve_prop(media.description, this.page, index, interaction);
      const url = await resolve_prop(media.url, this.page, index, interaction);

      this.builder.addItems((m) => {m.setURL(url); description ? m.setDescription(description) : null; spoiler ? m.setSpoiler(spoiler) : null; return m;})
    }

    container.addMediaGalleryComponents(this.builder)
  }
}

export class ProgressBar {
  public page!: Page;

  constructor(public dynAttributes: { value: DynamicProp<number>, max: DynamicProp<number>, width?: DynamicProp<number>, size?: DynamicProp<ProgressBarSize> }) {}

  public bind(page: Page) { this.page = page; }
  public async apply(container: ContainerBuilder, index: number, interaction?: Interaction) {
    container.addTextDisplayComponents(new TextDisplayBuilder().setContent(await this.construct(index, interaction)));
  }
  public async construct(index: number, interaction?: Interaction): Promise<string> {
    const max = await resolve_prop(this.dynAttributes.max, this.page, index, interaction);
    const value = await resolve_prop(this.dynAttributes.value, this.page, index, interaction);
    const width = await resolve_prop(this.dynAttributes.width ?? 15, this.page, index, interaction);
    const size = await resolve_prop(this.dynAttributes.size ?? ProgressBarSize.Normal, this.page, index, interaction);
    let buffer = (() => {
      switch (size) {
        case ProgressBarSize.Small:
          return "-# "
        case ProgressBarSize.Normal:
          return ""
        case ProgressBarSize.Medium:
          return "## "
        case ProgressBarSize.Large:
          return "# "
      }
    })();

    for (let i = 0; i < width; i++) {
      const first = i == 0;
      const last = i === width - 1

      if (i / width >= value / max) {
        if (first) buffer += "<:progress_bar_start_empty:1495868840339439836>";
        else if (last) buffer += "<:progress_bar_end_empty:1495868823075688670>";
        else buffer += "<:progress_bar_empty:1495868817648255107>";
      } else {
        if (first) buffer += "<:progress_bar_start_full:1495868845989040268>";
        else if (last) buffer += "<:progress_bar_end_full:1495868834215755988>";
        else buffer += "<:progress_bar_full:1495868805589504261>";
      }
    }

    return buffer;
  }
} 

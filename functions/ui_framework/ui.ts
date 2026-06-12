import { ContainerBuilder, MessageFlags, TextDisplayBuilder, SectionBuilder, type MessageActionRowComponentBuilder, ButtonStyle, Client } from "discord.js";
import { ActionRowBuilder, ButtonBuilder, ThumbnailBuilder, type Interaction, type MessageReplyOptions, type InteractionUpdateOptions } from "discord.js";
import { createHash } from 'crypto';

type Element = Section | ActionRow | TextDisplay | Window
type ButtonExecution = (interaction: Interaction, data: any) => Promise<void>;
type DynamicProp<T> = T | ((interaction: Interaction) => Promise<T>);
type DynamicButtonAttributes = {
  style: DynamicProp<ButtonStyle>,
  label?: DynamicProp<string>,
  emoji?: DynamicProp<string>,
  disabled?: DynamicProp<boolean>
};
type DynamicThumbnailAttributes = {
  url: DynamicProp<string>,
}

async function resolve_prop<T>(prop: DynamicProp<T>, interaction: Interaction): Promise<T> {
  return typeof prop === 'function' ? await (prop as Function)(interaction) : (prop as T);
}

export class Page {
  public customID: string;
  public isContainer: boolean;
  public dynamicStartIndex: number;
  public dynamicStartMax: number;
  readonly staticElements: Element[];
  readonly dynamicElements: Element[];
  readonly cache = new Map<string, ButtonExecution>;
  readonly data = new Map<string, any>;

  constructor(custom_id: string, client: Client, is_container?: boolean, static_elements?: Element[], dynamic_elements?: Element[], dynamic_start_index?: number, dynamic_start_max?: number) {
    client.pages.set(custom_id, this);
    this.customID = custom_id;
    this.isContainer = is_container ?? false;
    this.staticElements = static_elements ?? [];
    this.dynamicElements = dynamic_elements ?? [];
    this.dynamicStartIndex = dynamic_start_index ?? 0;
    this.dynamicStartMax = dynamic_start_max ?? 5;
    for (const element of this.staticElements) element.bind(this);
    for (const element of this.dynamicElements) element.bind(this);
  }

  public addStaticElement(element: Element): Page { element.bind(this); this.staticElements.push(element); return this; }
  public addDynamicElement(element: Element): Page { element.bind(this); this.dynamicElements.push(element); return this; }

  public async next(interaction?: Interaction): Promise<Page> {
    if (this.dynamicStartIndex + this.dynamicStartMax < this.dynamicElements.length) this.dynamicStartIndex += this.dynamicStartMax;
    if (interaction?.isButton()) await interaction.update(await this.render(undefined, interaction));
    return this;
  }

  public async previous(interaction?: Interaction): Promise<Page> {
    if (this.dynamicStartIndex - this.dynamicStartMax >= 0) this.dynamicStartIndex -= this.dynamicStartMax;
    if (interaction?.isButton()) await interaction.update(await this.render(undefined, interaction));
    return this;
  }

  public async render(dyn_index?: number, interaction?: Interaction): Promise<MessageReplyOptions> {
    this.dynamicStartIndex = dyn_index ?? this.dynamicStartIndex ?? 0;
    const container = new ContainerBuilder();
    for (const element of this.staticElements) await element.apply(container, interaction);
    return { 
        components: this.isContainer ? [container] : container.components, 
        flags: MessageFlags.IsComponentsV2 
    };
  }
}

export class Button {
  public builder: ButtonBuilder;
  public execution: ButtonExecution;
  public page!: Page;
  public data: any;
  public dynAttributes: DynamicButtonAttributes | null;

  constructor(execution: ButtonExecution, builder: ButtonBuilder | DynamicButtonAttributes, data: any) {
    this.execution = execution;
    this.data = data;
    if (builder instanceof ButtonBuilder) {
      this.builder = builder;
      this.dynAttributes = null;
    } else {
      this.builder = new ButtonBuilder();
      this.dynAttributes = builder;
    }
  }

  public bind(page: Page): Button {
    this.page = page;
    const hash = this.cache(this.data);
    this.builder.setCustomId("ui-" + this.page.customID + "-" + hash);
    return this;
  }

  public async compute(interaction: Interaction): Promise<ButtonBuilder> {
    if (this.dynAttributes) {
      const builder = new ButtonBuilder();
      if (this.builder.data.custom_id) builder.setCustomId(this.builder.data.custom_id);
      const attrs = this.dynAttributes;
      if (attrs.emoji) builder.setEmoji(await resolve_prop(attrs.emoji, interaction));
      if (attrs.label) builder.setLabel(await resolve_prop(attrs.label, interaction));
      if (attrs.disabled !== undefined) builder.setDisabled(await resolve_prop(attrs.disabled, interaction));
      builder.setStyle(await resolve_prop(attrs.style, interaction));
      return builder;
    }
    return this.builder;
  }

  public async apply(container: SectionBuilder, interaction: Interaction) {
    container.setButtonAccessory(await this.compute(interaction));
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
  public builder: ThumbnailBuilder;
  public page!: Page;
  public dynAttributes: DynamicThumbnailAttributes | null;
  constructor(builder: ThumbnailBuilder | DynamicThumbnailAttributes ) {
    this.builder = builder instanceof ThumbnailBuilder ? builder : new ThumbnailBuilder();
    this.dynAttributes = builder instanceof ThumbnailBuilder ? null : builder;
  }
  public bind(page: Page) { this.page = page; }
  public async apply(container: SectionBuilder, interaction: Interaction) {
    this.dynAttributes ? this.builder.setURL(await resolve_prop(this.dynAttributes.url, interaction)) : null;
    container.setThumbnailAccessory(this.builder);
  }
}

export class TextDisplay {
  public builder: TextDisplayBuilder;
  public page!: Page;
  constructor(builder: TextDisplayBuilder | string) { this.builder = typeof builder === "string" ? new TextDisplayBuilder().setContent(builder) : builder; }
  public bind(page: Page) { this.page = page; }
  public async apply(container: ContainerBuilder | SectionBuilder) { container.addTextDisplayComponents(this.builder); }
}

export class Section {
  public builder: SectionBuilder = new SectionBuilder();
  public page!: Page;
  constructor(text: string, public accessory: Button | Thumbnail) { this.builder.addTextDisplayComponents(t => t.setContent(text)); }
  public bind(page: Page) { this.page = page; this.accessory.bind(page); }
  public async apply(container: ContainerBuilder, interaction: Interaction) {
    await this.accessory.apply(this.builder, interaction);
    container.addSectionComponents(this.builder);
  }
}

export class ActionRow {
  public builder: ActionRowBuilder<MessageActionRowComponentBuilder> = new ActionRowBuilder();
  public page!: Page;
  constructor(public accessorys: Button[]) {}
  public bind(page: Page) { this.page = page; for (const a of this.accessorys) a.bind(page); }
  public async apply(container: ContainerBuilder, interaction: Interaction) {
    this.builder.setComponents(await Promise.all(this.accessorys.map(a => a.compute(interaction))));
    container.addActionRowComponents([this.builder]);
  }
}

export class Window {
  public page!: Page;
  public bind(page: Page) { this.page = page; }
  public async apply(container: ContainerBuilder, interaction: Interaction) {
    const elements = this.page.dynamicElements.slice(this.page.dynamicStartIndex, this.page.dynamicStartIndex + this.page.dynamicStartMax);
    for (const e of elements) await e.apply(container, interaction);
  }
}

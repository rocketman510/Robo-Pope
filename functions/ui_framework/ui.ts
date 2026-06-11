import { ContainerBuilder, MessageFlags, TextDisplayBuilder, SectionBuilder, type MessageActionRowComponentBuilder, ButtonStyle, Client } from "discord.js";
import { ActionRowBuilder, ButtonBuilder, type Interaction, type MessageReplyOptions, type InteractionUpdateOptions, type ThumbnailBuilder } from "discord.js";
import { createHash } from 'crypto';

type Element = Section | ActionRow | TextDisplay | Window
type ButtonExecution = (interaction: Interaction, data: any) => Promise<void>;
type Execution = ButtonExecution

export class Page {
  public customID: string;
  public isContainer: boolean;
  public dynamicStartIndex: number;
  public dynamicStartMax: number;
  readonly staticElements: Element[];
  readonly dynamicElements: Element[];
  readonly cache = new Map<string, Execution>;
  readonly data = new Map<string, any>;

  constructor(custom_id: string, client: Client, is_container?: boolean, static_elements?: Element[], dynamic_elements?: Element[], dynamic_start_index?: number, dynamic_start_max?: number) {
    client.pages.set(custom_id, this)
    this.customID = custom_id;
    this.isContainer = is_container ?? false;
    this.staticElements = static_elements ?? [];
    this.dynamicElements = dynamic_elements ?? [];
    this.dynamicStartIndex = dynamic_start_index ?? 0;
    this.dynamicStartMax = dynamic_start_max ?? 5;

    for (const element of this.staticElements) {
      element.bind(this);
    }
    for (const element of this.dynamicElements) {
      element.bind(this);
    }
  }

  public addStaticElement(element: Element): Page {
    element.bind(this);
    this.staticElements.push(element);
    return this;
  }

  public addDynamicElement(element: Element): Page {
    element.bind(this);
    this.dynamicElements.push(element);
    return this;
  }

  public async next(interaction?: Interaction): Promise<Page> {
    if (this.dynamicStartIndex + this.dynamicStartMax < this.dynamicElements.length) {
      this.dynamicStartIndex += this.dynamicStartMax
    }
    if (!!interaction && interaction.isButton()) {
      await interaction.update(this.render() as InteractionUpdateOptions)
    }
    return this;
  }

  public async previous(interaction?: Interaction): Promise<Page> {
    if (this.dynamicStartIndex - this.dynamicStartMax >= 0) {
      this.dynamicStartIndex -= this.dynamicStartMax
    }
    if (!!interaction && interaction.isButton()) {
      await interaction.update(this.render() as InteractionUpdateOptions)
    }
    return this;
  }

  public render(dyn_index?: number): MessageReplyOptions {
    this.dynamicStartIndex = dyn_index ?? this.dynamicStartIndex ?? 0;
      const container = new ContainerBuilder()

      for (const element of this.staticElements) {
        element.apply(container);
      }

    if (this.isContainer) {
      return { components: [container], flags: MessageFlags.IsComponentsV2 }
    } else {
      return { components: container.components, flags: MessageFlags.IsComponentsV2 }//Seems to work?
    }
  }
}

/*
 *
 * ACCESSORIES:
 *
 * */
export class Button {
  public builder: ButtonBuilder;
  public execution: ButtonExecution;
  public page!: Page;
  public data: any;

  constructor(execution: ButtonExecution, builder: ButtonBuilder | { style: ButtonStyle, label: string, emoji?: string } | { style: ButtonStyle, label?: string, emoji: string }, data: any) {
    this.execution = execution;
    this.data = data;

    if (builder instanceof ButtonBuilder) {
      this.builder = builder;
    } else {
      this.builder = new ButtonBuilder()
        .setStyle(builder.style);

      if (!!builder.label) {
        this.builder.setLabel(builder.label)
      }
      if (!!builder.emoji) {
        this.builder.setEmoji(builder.emoji)
      }
    }
  }

  public bind(page: Page): Button {
    this.page = page;
    const hash = this.cache(this.data);
    this.builder.setCustomId("ui-" + this.page.customID + "-" + hash);
    return this;
  }

  public cache(data: any): string {
    let dataIdentifier: string;

    if (data && typeof data === 'object' && 'customID' in data) {
      dataIdentifier = data.customID;
    } else if (typeof data === 'object' && data !== null) {
      dataIdentifier = JSON.stringify(data);
    } else {
      dataIdentifier = String(data);
    }
    const hash = createHash("sha256").update(dataIdentifier + this.execution.toString()).digest('base64url');
    this.page.cache.set(hash, this.execution)
    this.page.data.set(hash, data)
    return hash;
  }
}

export class Thumbnail {
  public builder: ThumbnailBuilder;
  public page!: Page;

  public bind(page: Page) {
    this.page = page;
  }

  constructor(builder: ThumbnailBuilder) {
    this.builder = builder;
  }
}

/*
 *
 * ELEMENTS:
 *
 * */
export class TextDisplay {
  public builder: TextDisplayBuilder;
  public page!: Page;

  constructor(builder: TextDisplayBuilder | string) {
    if (typeof builder === "string") {
      this.builder = new TextDisplayBuilder().setContent(builder);
    } else {
      this.builder = builder;
    } 
  }

  public bind(page: Page) {
    this.page = page
  }

  public apply(container: ContainerBuilder) {
    container.addTextDisplayComponents(this.builder);
  }
}

export class Section {
  public builder!: SectionBuilder;
  public page!: Page;
  public accessory: Button | Thumbnail;

  constructor(builder: { text: string, accessory: Button | Thumbnail }) {
    this.builder = new SectionBuilder().addTextDisplayComponents((t) => t.setContent(builder.text))
    this.accessory = builder.accessory;
  }

  public bind(page: Page) {
    this.page = page;
    this.accessory.bind(page);

    if (this.accessory instanceof Button) {
      this.builder.setButtonAccessory(this.accessory.builder)
    } else if (this.accessory instanceof Thumbnail) {
      this.builder.setThumbnailAccessory(this.accessory.builder)
    }
  }

  public apply(container: ContainerBuilder) {
    container.addSectionComponents(this.builder);
  }
}

export class ActionRow {
  public builder: ActionRowBuilder<MessageActionRowComponentBuilder>;
  public page!: Page;
  public accessorys: Button[];

  constructor(builder: { accessorys: Button[] }) {
    this.builder = new ActionRowBuilder<MessageActionRowComponentBuilder>()
    this.accessorys = builder.accessorys;
  }

  public bind(page: Page) {
    this.page = page;
    for (const accessory of this.accessorys) {
      accessory.bind(page);
    }

    this.builder.setComponents(this.accessorys.map((v) => v.builder));
  }

  public apply(container: ContainerBuilder) {
    container.addActionRowComponents([this.builder])
  }
}

export class Window {
  public builder!: Element[];
  public page!: Page;

  public bind(page: Page) {
    this.page = page;
  }

  public apply(container: ContainerBuilder) {
    this.builder = this.page.dynamicElements.slice(this.page.dynamicStartIndex, this.page.dynamicStartIndex + this.page.dynamicStartMax)
    for (const element of this.builder) {
      element.apply(container)
    }
  }
}

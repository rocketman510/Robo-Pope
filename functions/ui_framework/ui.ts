import { ContainerBuilder, MessageFlags, TextDisplayBuilder, SectionBuilder, type MessageActionRowComponentBuilder } from "discord.js";
import { ActionRowBuilder, type ButtonBuilder, type Interaction, type MessageReplyOptions, type ThumbnailBuilder } from "discord.js";
import excommunicate from "../../commands/excommunicate";

type Element = TextDisplay | Section | ActionRow

export class Page {
  public customID: string;
  public isContainer: boolean;
  readonly staticElements: Element[];
  readonly dynamicElements: Element[];
  readonly cache = new Map<string, Element>;
  readonly data = new Map<string, any>;

  constructor(custom_id: string, is_container?: boolean, static_elements?: Element[], dynamic_elements?: Element[]) {
    this.customID = custom_id;
    this.isContainer = is_container ?? false;
    this.staticElements = static_elements ?? [];
    this.dynamicElements = dynamic_elements ?? [];

    for (const element of this.staticElements) {
      element.page = this;
    }
    for (const element of this.dynamicElements) {
      element.page = this;
    }
  }

  public addStaticElement(element: Element): Page {
    element.page = this;
    this.staticElements.push(element);
    return this;
  }

  public render(): MessageReplyOptions {
    if (this.isContainer) {
      const container = new ContainerBuilder()

      for (const element of this.staticElements) {
        element.apply(container);
      }

      return { components: [container], flags: MessageFlags.IsComponentsV2 }
    } else {
      let components = [];

      for (const element of this.staticElements) {
        components.push(element.builder);
      }

      return { components, flags: MessageFlags.IsComponentsV2 }
    }
  }
}

type ButtonExecution = (interaction: Interaction) => void;

export class Button {
  public builder: ButtonBuilder;
  public execution: ButtonExecution;
  public page!: Page;

  constructor(execution: ButtonExecution, builder: ButtonBuilder) {
    this.builder = builder;
    this.execution = execution;
  }
}

export class Thumbnail {
  public builder: ThumbnailBuilder;
  public page!: Page;

  constructor(builder: ThumbnailBuilder) {
    this.builder = builder;
  }
}

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

  public apply(container: ContainerBuilder) {
    container.addTextDisplayComponents(this.builder);
  }
}

export class Section {
  public builder!: SectionBuilder;
  public page!: Page;

  constructor(builder: SectionBuilder | { text: string, accessory: Button | Thumbnail }) {
    if (builder instanceof SectionBuilder) {
      this.builder = builder;
    } else {
      if (builder.accessory instanceof Button) {
        this.builder = new SectionBuilder()
          .addTextDisplayComponents((t) => t.setContent(builder.text))
          .setButtonAccessory(builder.accessory.builder);
      } else if (builder.accessory instanceof Thumbnail) {
        this.builder = new SectionBuilder()
          .addTextDisplayComponents((t) => t.setContent(builder.text))
          .setThumbnailAccessory(builder.accessory.builder);
      }
    }
  }

  public apply(container: ContainerBuilder) {
    container.addSectionComponents(this.builder);
  }
}

export class ActionRow {
  public builder: ActionRowBuilder<MessageActionRowComponentBuilder>;
  public page!: Page;

  constructor(builder: ActionRowBuilder<MessageActionRowComponentBuilder> | { accessorys: Button[] }) {
    if (builder instanceof ActionRowBuilder) {
      this.builder = builder;
    } else {
      this.builder = new ActionRowBuilder<MessageActionRowComponentBuilder>()
        .setComponents(builder.accessorys.map((v) => v.builder))
    }
  }

  public apply(container: ContainerBuilder) {
    container.addActionRowComponents([this.builder])
  }
}
